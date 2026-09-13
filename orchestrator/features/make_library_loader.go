package orchfeatures

import (
	"fmt"
	"path/filepath"
	"sort"
	"strconv"
	"strings"

	"github.com/olivierdevelops/capy/domain"
	"github.com/olivierdevelops/capy/features"
	"github.com/olivierdevelops/capy/infra"
)

// MakeLibraryLoader compiles a library file into a domain.Library. It accepts
// either YAML (`.yaml`, `.yml`) or Capy-native (`.capy`) library files; both
// formats produce the same in-memory RawLibrary DTO and are mapped via
// mapLibrary.
//
//   - args list → []ArgEntry → []PatternElement
//   - run: snippet → InnerBlock AST (parsed via the outer lexer + inner parser)
//   - types/context/file_template carried through
//
// LoadLibraryFromBytes compiles an in-memory library written in
// Capy's native (.capy) syntax. The `format` argument is reserved
// for future formats; today only "capy" (the default) is supported.
func LoadLibraryFromBytes(format string, src []byte, tokenize func(string) ([]domain.Token, error)) (domain.Library, error) {
	if format != "" && strings.ToLower(format) != "capy" {
		return domain.Library{}, fmt.Errorf("unknown library format %q (only \"capy\" is supported)", format)
	}
	raw, err := infra.CapyLibParser{}.ParseBytes(src)
	if err != nil {
		return domain.Library{}, err
	}
	return mapLibrary(raw, tokenize)
}

func MakeLibraryLoader(tokenize func(string) ([]domain.Token, error)) features.LibraryLoader {
	return features.LibraryLoader{
		Load: func(path string) (domain.Library, error) {
			raw, err := loadRawWithImports(path, map[string]bool{}, infra.CapyLibParser{})
			if err != nil {
				return domain.Library{}, err
			}
			return mapLibrary(raw, tokenize)
		},
	}
}

// loadRawWithImports parses one library file and recursively pulls in any
// `import` directives, merging them into the result. The IMPORTING file
// wins on conflict (so local overrides shadow imports). Cycles error.
//
// Import paths are resolved relative to the file containing the `import`.
func loadRawWithImports(path string, visited map[string]bool, cp infra.CapyLibParser) (infra.RawLibrary, error) {
	abs, err := filepath.Abs(path)
	if err != nil {
		return infra.RawLibrary{}, err
	}
	if visited[abs] {
		return infra.RawLibrary{}, fmt.Errorf("import cycle detected at %s", path)
	}
	visited[abs] = true

	raw, err := cp.ParseFile(path)
	if err != nil {
		return raw, err
	}

	if len(raw.Imports) == 0 {
		return raw, nil
	}

	// Start from a merged-imports base; the local raw overrides at the end.
	dir := filepath.Dir(abs)
	merged := infra.RawLibrary{
		Functions: map[string]infra.RawFunction{},
		Types:     map[string]infra.RawType{},
		Context:   map[string]any{},
		Files:     map[string]string{},
	}
	for _, imp := range raw.Imports {
		impPath := imp
		if !filepath.IsAbs(impPath) {
			impPath = filepath.Join(dir, imp)
		}
		impRaw, err := loadRawWithImports(impPath, visited, cp)
		if err != nil {
			return raw, fmt.Errorf("import %q: %v", imp, err)
		}
		mergeRaw(&merged, impRaw)
	}
	// Local raw wins on conflict.
	mergeRaw(&merged, raw)
	return merged, nil
}

// mergeRaw copies entries from src into dst. Existing keys in dst are
// OVERRIDDEN — call order determines precedence (last-write-wins).
func mergeRaw(dst *infra.RawLibrary, src infra.RawLibrary) {
	if src.Extension != "" {
		dst.Extension = src.Extension
	}
	if src.OutputFile != "" {
		dst.OutputFile = src.OutputFile
	}
	if src.FileTemplate != "" {
		dst.FileTemplate = src.FileTemplate
	}
	for k, v := range src.Functions {
		dst.Functions[k] = v
	}
	for k, v := range src.Types {
		dst.Types[k] = v
	}
	for k, v := range src.Context {
		dst.Context[k] = v
	}
	for k, v := range src.Files {
		dst.Files[k] = v
	}
	// Preprocess directives are unioned; imports widen what the
	// downstream library can opt into without overriding.
	for _, d := range src.Preprocess {
		seen := false
		for _, e := range dst.Preprocess {
			if e == d {
				seen = true
				break
			}
		}
		if !seen {
			dst.Preprocess = append(dst.Preprocess, d)
		}
	}
	// Comments markers are unioned, same rationale as Preprocess.
	for _, c := range src.Comments {
		seen := false
		for _, e := range dst.Comments {
			if e == c {
				seen = true
				break
			}
		}
		if !seen {
			dst.Comments = append(dst.Comments, c)
		}
	}
}

func mapLibrary(r infra.RawLibrary, tokenize func(string) ([]domain.Token, error)) (domain.Library, error) {
	// Detect the new-shape sentinel that the .capy parser stashes on
	// FileTemplate / Files entries and parse the body into an AST that
	// the renderer walks directly. Format: "\x00NEW_SHAPE\x00<src>".
	const sentinel = "\x00NEW_SHAPE\x00"
	parseMaybeNew := func(s, label string) (*domain.InnerBlock, error) {
		if !strings.HasPrefix(s, sentinel) {
			return nil, nil
		}
		body := s[len(sentinel):]
		toks, err := tokenize(body)
		if err != nil {
			return nil, fmt.Errorf("%s: parsing body: %v", label, err)
		}
		ast, err := ParseInner(toks)
		if err != nil {
			return nil, fmt.Errorf("%s: parsing body: %v", label, err)
		}
		// File-template / file blocks may not contain state-mutation
		// statements (the context is already finalised when they
		// render; mutations would silently do nothing). Reject early.
		residual, err := translateNewShape(ast)
		if err != nil {
			return nil, fmt.Errorf("%s: %v", label, err)
		}
		if len(residual.Stmts) > 0 {
			return nil, fmt.Errorf("%s: state-mutation statements (set/append/…) aren't allowed here — file blocks only render", label)
		}
		return &ast, nil
	}
	ftAST, err := parseMaybeNew(r.FileTemplate, "file_template")
	if err != nil {
		return domain.Library{}, err
	}
	filesAST := map[string]*domain.InnerBlock{}
	if len(r.Files) > 0 {
		for path, body := range r.Files {
			ast, err := parseMaybeNew(body, "file "+strconv.Quote(path))
			if err != nil {
				return domain.Library{}, err
			}
			if ast != nil {
				filesAST[path] = ast
			}
		}
	}
	lib := domain.Library{
		Extension:       r.Extension,
		OutputFile:      r.OutputFile,
		Description:     r.Description,
		Functions:       map[string]*domain.FuncDef{},
		Types:           map[string]domain.TypeDef{},
		Context:         r.Context,
		FileTemplateAST: ftAST,
		FilesAST:        filesAST,
		Preprocess:      r.Preprocess,
		Comments:        r.Comments,
		Commands:        map[string]*domain.CommandDef{},
		LibName:         r.LibName,
		LibVersion:      r.LibVersion,
		Impls:           map[string]*domain.ImplDef{},
		DefaultImpl:     r.DefaultImpl,
	}
	for name, im := range r.Impls {
		lib.Impls[name] = &domain.ImplDef{
			Name:        name,
			File:        im.File,
			Description: im.Description,
			Version:     im.Version,
			IsDefault:   im.IsDefault,
		}
	}
	if lib.FilesAST == nil {
		lib.FilesAST = map[string]*domain.InnerBlock{}
	}
	if lib.Context == nil {
		lib.Context = map[string]any{}
	}

	for name, t := range r.Types {
		// Group types are mutually exclusive with constraint types.
		// Either you declare a delimited capture (open + close) or
		// you declare a constraint (base / pattern / options) — not
		// both. Reject the mixed form early with a clear error.
		isGroup := t.GroupOpen != "" || t.GroupClose != ""
		if isGroup {
			if t.GroupOpen == "" || t.GroupClose == "" {
				return lib, fmt.Errorf("type %q: group_open and group_close must BOTH be set", name)
			}
			if t.Base != "" || t.Pattern != "" || len(t.Options) > 0 {
				return lib, fmt.Errorf("type %q: group types cannot also declare base/pattern/options", name)
			}
		}
		lib.Types[name] = domain.TypeDef{
			Name:        name,
			Description: t.Description,
			Base:        t.Base,
			Pattern:     t.Pattern,
			Options:     t.Options,
			GroupOpen:   t.GroupOpen,
			GroupClose:  t.GroupClose,
		}
	}

	for name, f := range r.Functions {
		fd, err := compileFunction(name, f, tokenize)
		if err != nil {
			return lib, err
		}
		lib.Functions[name] = fd
	}

	// Compile library commands (declared via `command "X" ... end`
	// blocks in the .capy manifest). Body is inner-DSL with shell-
	// like primitives the evaluator surfaces.
	for name, c := range r.Commands {
		cd := &domain.CommandDef{
			Name:        name,
			Description: c.Description,
			BodyRaw:     c.Body,
		}
		for _, a := range c.Args {
			cd.Args = append(cd.Args, domain.CommandArg{
				Name:        a.Name,
				Required:    a.Required,
				Description: a.Description,
			})
		}
		for _, f := range c.Flags {
			cd.Flags = append(cd.Flags, domain.CommandFlag{
				Name:        f.Name,
				Description: f.Description,
				Default:     f.Default,
				IsBool:      f.IsBool,
			})
		}
		if strings.TrimSpace(c.Body) != "" {
			toks, err := tokenize(c.Body)
			if err != nil {
				return lib, fmt.Errorf("command %q: parsing body: %v", name, err)
			}
			ast, err := ParseInner(toks)
			if err != nil {
				return lib, fmt.Errorf("command %q: parsing body: %v", name, err)
			}
			cd.Body = ast
		}
		lib.Commands[name] = cd
	}

	// Validate cross-references after all functions are loaded. Iterate in
	// NAME ORDER: validation returns on the first failure, so unsorted map
	// iteration meant a library with two invalid functions reported a
	// different error on each run.
	validationOrder := make([]string, 0, len(lib.Functions))
	for name := range lib.Functions {
		validationOrder = append(validationOrder, name)
	}
	sort.Strings(validationOrder)
	for _, fdName := range validationOrder {
		fd := lib.Functions[fdName]
		// Optional captures must be trailing: once an optional arg is
		// declared, every later arg must also be optional (otherwise
		// the matcher couldn't know whether a supplied value fills the
		// optional or the required-after-it).
		seenOptional := false
		for _, a := range fd.Args {
			if a.Kind != "capture" {
				if seenOptional {
					return lib, fmt.Errorf("function %q: literal arg %q cannot follow an optional capture", fd.Name, a.Value)
				}
				continue
			}
			if a.Optional {
				seenOptional = true
			} else if seenOptional {
				return lib, fmt.Errorf("function %q: required capture %q cannot follow an optional capture (optional args must be trailing)", fd.Name, a.Name)
			}
		}
		for _, a := range fd.Args {
			if a.Kind != "capture" {
				continue
			}
			// A capture's type may name another LIBRARY FUNCTION
			// (function-as-type / named nonterminal). That's valid even
			// though it's neither a built-in nor a declared `type`.
			if _, isFunc := lib.Functions[a.Type]; isFunc {
				// `sep` / `join` only have meaning when the capture repeats.
				if a.Repeat == "" && a.Sep != "" {
					return lib, fmt.Errorf("function %q: capture %q uses `sep` but is not repeated (add `*` or `+`)", fd.Name, a.Name)
				}
				if a.Repeat == "" && a.Join != "" {
					return lib, fmt.Errorf("function %q: capture %q uses `join` but is not repeated (add `*` or `+`)", fd.Name, a.Name)
				}
				continue
			}
			// Repetition (`type*` / `type+`) and `sep` only make sense for
			// function-typed captures.
			if a.Repeat != "" {
				return lib, fmt.Errorf("function %q: capture %q uses a repetition suffix but its type %q is not a library function", fd.Name, a.Name, a.Type)
			}
			if a.Sep != "" {
				return lib, fmt.Errorf("function %q: capture %q uses `sep` but its type %q is not a library function", fd.Name, a.Name, a.Type)
			}
			if a.Join != "" {
				return lib, fmt.Errorf("function %q: capture %q uses `join` but its type %q is not a library function", fd.Name, a.Name, a.Type)
			}
			if !validType(a.Type, lib.Types) {
				ce := &domain.CapyError{Msg: fmt.Sprintf("function %q: capture %q has unknown type %q", fd.Name, a.Name, a.Type)}
				// Suggest the closest known type (built-ins + library-declared).
				cands := []string{"any", "ident", "raw", "tail", "word", "dotted_ident", "string", "int", "float", "bool"}
				for n := range lib.Types {
					cands = append(cands, n)
				}
				for n := range lib.Functions {
					cands = append(cands, n)
				}
				if best := domain.SuggestClosest(a.Type, cands, 2); best != "" {
					ce.Hint = fmt.Sprintf("did you mean %q?", best)
				} else {
					ce.Hint = fmt.Sprintf("built-in types: any, ident, raw, tail, word, dotted_ident, string, int, float, bool; declared types: %v", typeNames(lib.Types))
				}
				return lib, ce
			}
		}
		// Resolve IsFunc on compiled pattern elements now that all function
		// names are known: a capture whose type names a library function is
		// a structural (nonterminal) match, not a flat token.
		for i := range fd.Elements {
			el := &fd.Elements[i]
			if el.IsCapture {
				if _, isFunc := lib.Functions[el.CapType]; isFunc {
					el.IsFunc = true
				}
			}
		}
		if fd.Block != nil {
			// Four modes (exactly one must be set):
			//   block_closer NAME      — keyword-closed, nested parsing
			//   block_open X close Y   — delimiter-pair, nested parsing
			//   block_dedent           — indent-closed, nested parsing
			//   block_verbatim NAME    — keyword-closed, body is raw bytes
			hasCloser := fd.Block.Closer != "" && !fd.Block.IsVerbatim
			hasDelim := fd.Block.Open != "" && fd.Block.Close != ""
			hasDedent := fd.Block.IsDedent
			hasVerbatim := fd.Block.IsVerbatim
			hasSeq := len(fd.Block.CloseSeq) > 0
			modes := 0
			if hasCloser {
				modes++
			}
			if hasDelim {
				modes++
			}
			if hasDedent {
				modes++
			}
			if hasVerbatim {
				modes++
			}
			if hasSeq {
				modes++
			}
			if modes != 1 {
				return lib, fmt.Errorf("function %q: block must set exactly one of block_closer, block_open/close, block_dedent, block_verbatim, or block_close_seq", fd.Name)
			}
			// Both keyword-closed modes require the closer function to
			// exist.
			if (hasCloser || hasVerbatim) && fd.Block.Closer != "" {
				if _, ok := lib.Functions[fd.Block.Closer]; !ok {
					return lib, fmt.Errorf("function %q: block.closer %q not found", fd.Name, fd.Block.Closer)
				}
			}
			// Each block_close_seq ref segment must name a capture bound
			// by this function's opener.
			if hasSeq {
				for _, seg := range fd.Block.CloseSeq {
					if seg.Ref == "" {
						continue
					}
					found := false
					for _, a := range fd.Args {
						if a.Kind == "capture" && a.Name == seg.Ref {
							found = true
							break
						}
					}
					if !found {
						return lib, fmt.Errorf("function %q: block_close_seq references unknown capture %q", fd.Name, seg.Ref)
					}
				}
			}
		}
	}

	return lib, nil
}

func compileFunction(name string, f infra.RawFunction, tokenize func(string) ([]domain.Token, error)) (*domain.FuncDef, error) {
	args, err := compileArgs(f.Args, name)
	if err != nil {
		return nil, err
	}
	// Auto-name-prepend rule. The `bare` directive opts out — useful
	// for shape-only functions (e.g. a row of bare string literals)
	// whose source has no leading keyword to anchor on.
	hasLiteral := false
	for _, a := range args {
		if a.Kind == "literal" {
			hasLiteral = true
			break
		}
	}
	if !hasLiteral && !f.Bare {
		args = append([]domain.ArgEntry{{Kind: "literal", Value: name}}, args...)
	}

	elements := compileElements(args)

	// New-shape `body:` (unified write/state block) → translate into
	// the equivalent TemplateAST + Run AST before constructing the FuncDef.
	var run string
	var templateAST *domain.InnerBlock
	if strings.TrimSpace(f.Body) != "" {
		toks, err := tokenize(f.Body)
		if err != nil {
			return nil, fmt.Errorf("function %q: parsing body: %v", name, err)
		}
		ast, err := ParseInner(toks)
		if err != nil {
			return nil, fmt.Errorf("function %q: parsing body: %v", name, err)
		}
		runAST, err := translateNewShape(ast)
		if err != nil {
			return nil, fmt.Errorf("function %q: %v", name, err)
		}
		// Stash the full AST for direct rendering. The renderer treats
		// state-mutation statements (set/append/…) as no-ops — those
		// are handled separately via the run AST below. WriteStmt /
		// IfStmt / LoopStmt are the only render-bearing forms.
		astCopy := ast
		templateAST = &astCopy
		// Re-serialise the run AST as inner-DSL source text — the
		// later "if Run is non-empty, tokenize+parse" path below
		// re-parses it. (Slightly wasteful but keeps one code path.)
		run = renderInnerBlock(runAST)
	}

	fd := &domain.FuncDef{
		Name:        name,
		Description: f.Description,
		Args:        args,
		Elements:    elements,
		TemplateAST: templateAST,
		Priority:    f.Priority,
	}
	if f.Block != nil {
		fd.Block = &domain.BlockSpec{Closer: f.Block.Closer, Open: f.Block.Open, Close: f.Block.Close, IsDedent: f.Block.IsDedent, IsVerbatim: f.Block.IsVerbatim, Sections: f.Block.Sections}
		if len(f.Block.CloseSeq) > 0 {
			var segs []domain.CloseSegment
			for _, raw := range f.Block.CloseSeq {
				if raw.IsRef {
					// A bare capture-name reference; resolved against the
					// opener's bound captures at parse time.
					segs = append(segs, domain.CloseSegment{Ref: raw.Text})
					continue
				}
				// A fixed literal — pre-tokenize it the way the user-script
				// lexer will (e.g. "</div>" → ["</", "div", ">"]).
				toks, err := lexemeTexts(raw.Text, tokenize)
				if err != nil {
					return nil, fmt.Errorf("function %q: block_close_seq: %v", name, err)
				}
				if len(toks) == 0 {
					return nil, fmt.Errorf("function %q: block_close_seq segment %q lexes to no tokens", name, raw.Text)
				}
				segs = append(segs, domain.CloseSegment{Tokens: toks})
			}
			fd.Block.CloseSeq = segs
		}
	}
	if f.FollowedByIndent && f.NotFollowedByIndent {
		return nil, fmt.Errorf("function %q: cannot set both when_followed_by and when_not_followed_by", name)
	}
	if f.FollowedByIndent || f.NotFollowedByIndent {
		fd.Lookahead = &domain.Lookahead{RequireIndent: f.FollowedByIndent, ForbidIndent: f.NotFollowedByIndent}
	}

	if strings.TrimSpace(run) != "" {
		toks, err := tokenize(run)
		if err != nil {
			return nil, fmt.Errorf("function %q: parsing run: %v", name, err)
		}
		ast, err := ParseInner(toks)
		if err != nil {
			return nil, fmt.Errorf("function %q: parsing run: %v", name, err)
		}
		fd.RunAST = &ast
	}

	return fd, nil
}

// lexemeTexts tokenizes src and returns the .Text of each lexeme token,
// skipping structural tokens (NEWLINE / INDENT / DEDENT / EOF). Used to
// pre-compute a multi-token block closer sequence (e.g. "</div>" →
// ["</", "div", ">"]) at library-load time.
func lexemeTexts(src string, tokenize func(string) ([]domain.Token, error)) ([]string, error) {
	toks, err := tokenize(src)
	if err != nil {
		return nil, err
	}
	var out []string
	for _, t := range toks {
		switch t.Kind {
		case domain.TokNewline, domain.TokIndent, domain.TokDedent, domain.TokEOF:
			continue
		}
		out = append(out, t.Text)
	}
	return out, nil
}

func compileArgs(raws []infra.RawArg, fname string) ([]domain.ArgEntry, error) {
	var out []domain.ArgEntry
	for i, r := range raws {
		switch r.Kind {
		case "literal":
			if r.Value == "" {
				return nil, fmt.Errorf("function %q arg %d: kind=literal requires value", fname, i)
			}
			if r.Name != "" || r.Type != "" {
				return nil, fmt.Errorf("function %q arg %d: kind=literal cannot have name/type", fname, i)
			}
			out = append(out, domain.ArgEntry{Kind: "literal", Value: r.Value, Description: r.Description})
		case "capture":
			if r.Name == "" {
				return nil, fmt.Errorf("function %q arg %d: kind=capture requires name", fname, i)
			}
			if r.Value != "" {
				return nil, fmt.Errorf("function %q arg %d: kind=capture cannot have value", fname, i)
			}
			t := r.Type
			if t == "" {
				t = "any"
			}
			out = append(out, domain.ArgEntry{Kind: "capture", Name: r.Name, Type: t, Description: r.Description, Optional: r.Optional, Default: r.Default, Repeat: r.Repeat, Sep: r.Sep, Join: r.Join})
		default:
			return nil, fmt.Errorf("function %q arg %d: unknown or missing kind %q (must be \"literal\" or \"capture\")", fname, i, r.Kind)
		}
	}
	return out, nil
}

func compileElements(args []domain.ArgEntry) []domain.PatternElement {
	var out []domain.PatternElement
	for _, a := range args {
		if a.Kind == "literal" {
			// Split on `.` so dotted names (scene.create_sphere) match how the
			// outer lexer tokenises them.
			out = append(out, splitLiteral(a.Value)...)
		} else {
			out = append(out, domain.PatternElement{IsCapture: true, Name: a.Name, CapType: a.Type, Optional: a.Optional, Default: a.Default, Repeat: a.Repeat, Sep: a.Sep, Join: a.Join})
		}
	}
	return out
}

// splitLiteral converts "scene.create_sphere" into 3 PatternElements; non-dotted
// literals (including multi-char operators like `:=`, `->`) pass through whole.
func splitLiteral(lit string) []domain.PatternElement {
	parts := []string{}
	cur := strings.Builder{}
	flush := func() {
		if cur.Len() > 0 {
			parts = append(parts, cur.String())
			cur.Reset()
		}
	}
	for i := 0; i < len(lit); i++ {
		c := lit[i]
		if c == '.' {
			flush()
			parts = append(parts, ".")
			continue
		}
		cur.WriteByte(c)
	}
	flush()
	out := make([]domain.PatternElement, 0, len(parts))
	for _, p := range parts {
		out = append(out, domain.PatternElement{Literal: p})
	}
	return out
}

// typeNames returns the sorted list of declared type names; used in error
// hints when a capture references an unknown type.
func typeNames(types map[string]domain.TypeDef) []string {
	out := make([]string, 0, len(types))
	for n := range types {
		out = append(out, n)
	}
	return out
}

func validType(t string, types map[string]domain.TypeDef) bool {
	switch t {
	case "any", "ident", "raw", "tail", "word", "dotted_ident", "string", "int", "float", "bool":
		return true
	}
	_, ok := types[t]
	return ok
}
