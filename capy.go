// Package capy embeds Capy as a Go library so programs can define their own
// syntax inline without invoking a separate binary or shipping library files.
//
// Quick start:
//
//	lib, err := capy.NewLibrary(`
//	    extension html
//	    function button
//	        arg literal "button"
//	        arg capture label string
//	        template_str "<button>{{ .label }}</button>\n"
//	    end
//	`)
//	if err != nil { log.Fatal(err) }
//
//	out, err := lib.Run(`button "Click me"`)
//	// → <button>"Click me"</button>
//
// The library you pass to NewLibrary is written in Capy's native (`.capy`)
// syntax — the same grammar that drives user scripts. There is no separate
// template / config language; the renderer walks the parsed AST directly.
//
// Why embed Capy in your Go program?
//
//   - You're shipping a CLI that takes a config in a friendlier-than-YAML
//     DSL — write the parser in 50 lines of Capy instead of 500 of Go.
//   - You're building a code generator (Prisma-style schema → migrations)
//     and want users to write `model User { name : string }` instead of
//     calling a Go builder API.
//   - You want hot-swappable grammars: read a library file at startup,
//     let users contribute new ones without recompiling.
package capy

import (
	"sort"
	"strings"

	"github.com/olivierdevelops/capy/domain"
	"github.com/olivierdevelops/capy/features"
	"github.com/olivierdevelops/capy/infra"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

// Library is a compiled, ready-to-run Capy library. Safe to reuse across
// many Run calls; not safe for concurrent mutation but Run itself is
// re-entrant on a fixed Library.
type Library struct {
	lib    domain.Library
	lex    features.Lexer
	parser features.Parser
	eval   features.Evaluator
	host   domain.Host
}

// NewLibrary compiles a library written in Capy's native (`.capy`) syntax
// from an in-memory string. The returned Library is ready to Run any
// number of source scripts.
func NewLibrary(librarySrc string) (*Library, error) {
	return newFromBytes("capy", []byte(librarySrc))
}

// NewLibraryFromFile reads a `.capy` library file from disk.
func NewLibraryFromFile(path string) (*Library, error) {
	lex := orchfeatures.MakeLexer()
	loader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	dl, err := loader.Load(path)
	if err != nil {
		return nil, err
	}
	return assemble(dl, lex), nil
}

func newFromBytes(format string, src []byte) (*Library, error) {
	lex := orchfeatures.MakeLexer()
	dl, err := orchfeatures.LoadLibraryFromBytes(format, src, lex.Tokenize)
	if err != nil {
		return nil, err
	}
	return assemble(dl, lex), nil
}

func assemble(dl domain.Library, lex features.Lexer) *Library {
	return &Library{
		lib:    dl,
		lex:    lex,
		parser: orchfeatures.MakeParser(),
		eval:   orchfeatures.MakeEvaluator(),
		host:   domain.NoOpHost{},
	}
}

// SetHost installs a domain.Host that the library's `env` / `arg` /
// `read_file` inner-DSL primitives will read from. The default after
// NewLibrary is domain.NoOpHost — every primitive returns the empty
// zero value and read_file errors out. Pass infra.OSHost{...} to opt
// in to real os.Getenv / os.Args / os.ReadFile (only do this when the
// library source is trusted).
//
// Safe to call repeatedly; each call replaces the previous host.
func (l *Library) SetHost(h domain.Host) {
	if h == nil {
		h = domain.NoOpHost{}
	}
	l.host = h
	l.eval = orchfeatures.MakeEvaluatorWithHost(h)
}

// Run transpiles a single source script through this library and returns
// the generated output as a string. The library's file_template (if any)
// wraps the output.
//
// Run is safe to call repeatedly on the same Library with different
// sources — each call runs a fresh accumulating context.
func (l *Library) Run(scriptSrc string) (string, error) {
	out, _, err := l.RunMulti(scriptSrc)
	return out, err
}

// RunMulti is like Run but also returns the rendered multi-file map for
// libraries that declared `file "path":` blocks. The map is empty for
// libraries that don't use multi-file output.
//
// Source-level metaprogramming is supported: any `define NAME ... end`
// blocks at the top of the script are extracted and merged into the
// library before evaluation, exactly as the CLI does. Embedded callers
// (including the wasm playground) get the same behavior.
func (l *Library) RunMulti(scriptSrc string) (string, map[string]string, error) {
	// Honour any inclusion directives the library declared via its
	// `preprocess` block. The wasm/embedded sandbox has no filesystem
	// (NoOpHost), so most directives will fail to read — but the
	// engine still defers entirely to the library on what shapes count.
	scriptSrc, err := infra.Preprocess(scriptSrc, ".", l.lib.Preprocess, l.host)
	if err != nil {
		return "", nil, err
	}
	// Extract `define ... end` blocks from the script and merge them
	// into a copy of the library. The CLI does this in
	// orchestrator.RunMulti; replicate it here so embedding/wasm
	// callers also support metaprogramming.
	cleaned, defineLibSrc, err := infra.ExtractDefines(scriptSrc)
	if err != nil {
		return "", nil, err
	}
	libToUse := l.lib
	if defineLibSrc != "" {
		defineLib, err := orchfeatures.LoadLibraryFromBytes("capy", []byte(defineLibSrc), l.lex.Tokenize)
		if err != nil {
			return "", nil, err
		}
		// Shallow-copy the library and overlay source-defined functions.
		// Source defines WIN on conflict (matches CLI behavior).
		merged := libToUse
		merged.Functions = make(map[string]*domain.FuncDef, len(libToUse.Functions)+len(defineLib.Functions))
		for k, v := range libToUse.Functions {
			merged.Functions[k] = v
		}
		for k, v := range defineLib.Functions {
			merged.Functions[k] = v
		}
		libToUse = merged
	}
	toks, err := l.lex.TokenizeWith(cleaned, libToUse.Comments)
	if err != nil {
		return "", nil, err
	}
	prog, err := l.parser.Parse(toks, cleaned, libToUse)
	if err != nil {
		return "", nil, err
	}
	return l.eval.RunMulti(prog, libToUse)
}

// Extension reports the library's declared `extension:` field — useful
// when you want to write the output to a file with the correct suffix.
func (l *Library) Extension() string { return l.lib.Extension }

// OutputFile reports the library's optional `output_file:` field.
func (l *Library) OutputFile() string { return l.lib.OutputFile }

// RenderLibraryDocs returns Markdown reference documentation for the
// given Library — the same format `capy docs <lib>` writes on the
// CLI. Exposed at the top-level package so the wasm bundle and any
// embedded Go program can render docs without depending on the
// internal `domain` import.
func RenderLibraryDocs(lib *Library) string {
	return domain.RenderLibraryDocs(lib.lib)
}

// FunctionNames returns the sorted list of function names declared by
// the library. Useful for diagnostics and auto-discovery.
func (l *Library) FunctionNames() []string {
	out := make([]string, 0, len(l.lib.Functions))
	for name := range l.lib.Functions {
		out = append(out, name)
	}
	// The doc comment promises a sorted list and callers (capy-mcp, capy check)
	// rely on stable output; Go map iteration is randomised, so sort explicitly.
	sort.Strings(out)
	return out
}

// ArgInfo describes one argument in a function's match shape.
type ArgInfo struct {
	// Kind is "literal" or "capture".
	Kind string `json:"kind"`
	// Value is the literal token text (Kind == "literal").
	Value string `json:"value,omitempty"`
	// Name is the capture's bound name (Kind == "capture").
	Name string `json:"name,omitempty"`
	// Type is the capture's declared type (Kind == "capture").
	Type string `json:"type,omitempty"`
	// Description is the optional trailing doc string on the arg line.
	Description string `json:"description,omitempty"`
	// Optional is true for a trailing capture declared with `default`,
	// i.e. one the call site may omit (Kind == "capture").
	Optional bool `json:"optional,omitempty"`
	// Default is the value bound when an Optional capture is omitted.
	Default string `json:"default,omitempty"`
}

// FunctionInfo is the introspected shape of one library function —
// everything an editor needs for autocomplete, hover-docs, syntax
// highlighting, and a reference panel, derived from the library
// itself rather than a hand-maintained parallel catalogue.
type FunctionInfo struct {
	Name        string    `json:"name"`
	Description string    `json:"description,omitempty"`
	Args        []ArgInfo `json:"args"`
	// Block is "" for a non-block function, otherwise one of:
	// "closer:NAME", "open:X close:Y", "dedent", "verbatim:NAME".
	Block    string `json:"block,omitempty"`
	Priority int    `json:"priority,omitempty"`
}

// Introspect returns the declared functions of the library — name,
// description, argument shapes (literal / capture + type), block kind,
// and priority. The data comes straight from the compiled library, so
// an editor can derive its autocomplete / hover / highlight metadata
// instead of hand-maintaining a parallel catalogue that silently
// drifts. Results are sorted by function name for stable output.
func (l *Library) Introspect() []FunctionInfo {
	names := make([]string, 0, len(l.lib.Functions))
	for name := range l.lib.Functions {
		names = append(names, name)
	}
	sort.Strings(names)
	out := make([]FunctionInfo, 0, len(names))
	for _, name := range names {
		fn := l.lib.Functions[name]
		fi := FunctionInfo{
			Name:        fn.Name,
			Description: fn.Description,
			Priority:    fn.Priority,
		}
		for _, a := range fn.Args {
			fi.Args = append(fi.Args, ArgInfo{
				Kind:        a.Kind,
				Value:       a.Value,
				Name:        a.Name,
				Type:        a.Type,
				Description: a.Description,
				Optional:    a.Optional,
				Default:     a.Default,
			})
		}
		if b := fn.Block; b != nil {
			switch {
			case b.IsVerbatim:
				fi.Block = "verbatim:" + b.Closer
			case len(b.Sections) > 0:
				fi.Block = "sections:" + strings.Join(b.Sections, ",") + " closer:" + b.Closer
			case b.IsDedent:
				fi.Block = "dedent"
			case b.Open != "":
				fi.Block = "open:" + b.Open + " close:" + b.Close
			case b.Closer != "":
				fi.Block = "closer:" + b.Closer
			}
		}
		out = append(out, fi)
	}
	return out
}

// CommentMarkers returns the library's declared line-comment markers
// (from its `comments` block). Empty when the library declares none —
// in which case user scripts have no comment syntax. Useful for a
// syntax highlighter that wants to stop hardcoding the marker set.
func (l *Library) CommentMarkers() []string {
	out := make([]string, len(l.lib.Comments))
	copy(out, l.lib.Comments)
	return out
}
