package infra

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

// CapyLibParser reads a library written in Capy's native syntax (a `.capy`
// library file) and produces the same RawLibrary DTO as the YAML parser.
//
// Surface grammar (indentation-sensitive, line-based):
//
//	extension <STR>
//	output_file <STR>
//
//	function <NAME>
//	    priority <INT>
//	    arg literal <STR>
//	    arg capture <NAME> <TYPE>
//	    block_closer <NAME>
//	    block_open <STR> close <STR>
//	    template_str <STR>            # single-line inline template
//	    template:                      # multi-line block; ends at dedent
//	        ...
//	    run:                           # multi-line block; ends at dedent
//	        ...
//	end
//
//	file_template:
//	    ...
//
// Strings use double quotes with Go-style escapes (\n \t \" \\). Bare words
// are accepted for `extension`, `output_file`, capture types, and names.
type CapyLibParser struct{}

func (CapyLibParser) ParseFile(path string) (RawLibrary, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return RawLibrary{}, err
	}
	return parseCapyLib(string(b))
}

// ParseBytes parses a Capy-native library directly from in-memory bytes.
// Used by the embedding API (top-level `capy` package).
func (CapyLibParser) ParseBytes(b []byte) (RawLibrary, error) {
	return parseCapyLib(string(b))
}

// ParseString is the ergonomic in-memory entry point.
func (CapyLibParser) ParseString(s string) (RawLibrary, error) {
	return parseCapyLib(s)
}

func parseCapyLib(src string) (RawLibrary, error) {
	lines := strings.Split(strings.ReplaceAll(src, "\r\n", "\n"), "\n")
	p := &capyLibParser{lines: lines, lineNo: 0}
	return p.parseTop()
}

// rewriteTemplateBlocks transforms the `template ... end` sugar into
// the equivalent multi-line `write ` ... ` ` literal. Pure syntactic
// rewrite: the output bytes are what a library author would have
// written by hand, so the downstream merge → dedent → inner-DSL
// parse → render pipeline doesn't need to know this sugar exists.
//
//	template
//	    <div>
//	      ${escapeHtml text}
//	    </div>
//	end
//
// becomes (synth, before backtick merging):
//
//	write ` <div>
//	  ${escapeHtml text}
//	</div>
//	`
//
// Semantics:
//   - Opener: bareword `template` on its own line, no trailing args.
//   - Closer: bareword `end` at the same indent as `template`. Inner
//     `end` lines at a deeper indent are body content. Nested
//     `template … end` at the same indent are balanced via depth.
//   - Body: captured verbatim, then auto-dedented by the smallest
//     non-blank leading indent so the captured text starts flush-
//     left. ${…} interpolation runs at render time, same as a
//     hand-authored `write ` ... ` `.
//   - Backticks (`) inside the body are escaped (`\“) so the synth
//     is a valid backtick literal.
//
// baseLineNo is the source line number of `raws[0]` — used to make
// `missing end` errors point at the right line.
func rewriteTemplateBlocks(raws []string, baseLineNo int) ([]string, error) {
	var out []string
	inBacktick := false
	i := 0
	for i < len(raws) {
		ln := raws[i]
		if !inBacktick && strings.TrimSpace(ln) == "template" {
			openIndent := leadingWhitespace(ln)
			// Walk forward until we hit `end` at the same indent, with
			// depth tracking for nested `template … end` blocks AND
			// backtick state so a literal `template`/`end` inside an
			// existing `write ` ... ` ` doesn't trigger.
			depth := 1
			bodyEnd := -1
			innerBT := false
			for j := i + 1; j < len(raws); j++ {
				bl := raws[j]
				if innerBT {
					innerBT = updateBacktickState(bl, innerBT)
					continue
				}
				bs := strings.TrimSpace(bl)
				bli := leadingWhitespace(bl)
				if bs == "template" && bli == openIndent {
					depth++
				} else if bs == "end" && bli == openIndent {
					depth--
					if depth == 0 {
						bodyEnd = j
						break
					}
				}
				innerBT = updateBacktickState(bl, innerBT)
			}
			if bodyEnd < 0 {
				return nil, fmt.Errorf("line %d: template block has no matching `end` at the same indent", baseLineNo+i+1)
			}
			body := raws[i+1 : bodyEnd]
			// Auto-dedent: strip the smallest non-blank leading indent.
			bodyMin := -1
			for _, bl := range body {
				if strings.TrimSpace(bl) == "" {
					continue
				}
				bi := leadingWhitespace(bl)
				if bodyMin < 0 || bi < bodyMin {
					bodyMin = bi
				}
			}
			if bodyMin < 0 {
				bodyMin = 0
			}
			dedented := make([]string, len(body))
			for k, bl := range body {
				if len(bl) >= bodyMin {
					dedented[k] = escapeForBacktick(bl[bodyMin:])
				} else {
					dedented[k] = escapeForBacktick(bl)
				}
			}
			// Emit the synth at the opener's indent. The opening
			// backtick goes immediately after `write ` on the first
			// line; body lines flow flush-left; the closing backtick
			// sits on its own flush-left line. mergeBackticksInLines
			// then collapses the whole thing into one logical line
			// the same way it would for a hand-authored multi-line
			// backtick.
			prefix := strings.Repeat(" ", openIndent)
			if len(dedented) == 0 {
				out = append(out, prefix+"write ``")
			} else {
				out = append(out, prefix+"write `"+dedented[0])
				for _, bl := range dedented[1:] {
					out = append(out, bl)
				}
				out = append(out, "`")
			}
			i = bodyEnd + 1
			continue
		}
		out = append(out, ln)
		inBacktick = updateBacktickState(ln, inBacktick)
		i++
	}
	return out, nil
}

// leadingWhitespace returns the count of leading space-or-tab bytes.
// Used purely for indent-equality comparisons; mixed tabs/spaces are
// the author's responsibility (matches the rest of the lib parser).
func leadingWhitespace(s string) int {
	n := 0
	for n < len(s) && (s[n] == ' ' || s[n] == '\t') {
		n++
	}
	return n
}

// updateBacktickState scans one line and toggles `inBacktick` on each
// unescaped backtick. Matches the same logic the body collector uses
// when tracking multi-line backtick state.
func updateBacktickState(ln string, inBacktick bool) bool {
	for i := 0; i < len(ln); i++ {
		c := ln[i]
		if c == '\\' && i+1 < len(ln) {
			i++
			continue
		}
		if c == '`' {
			inBacktick = !inBacktick
		}
	}
	return inBacktick
}

// escapeForBacktick prepares one body line for embedding inside a
// backtick literal: backslashes double, backticks gain a backslash.
// `${...}` interpolation markers pass through untouched — they're
// what makes this sugar useful (vs `block_verbatim`, which is the
// interpolation-OFF sibling).
func escapeForBacktick(s string) string {
	if !strings.ContainsAny(s, "`\\") {
		return s
	}
	var b strings.Builder
	b.Grow(len(s) + 4)
	for i := 0; i < len(s); i++ {
		switch s[i] {
		case '\\':
			b.WriteString(`\\`)
		case '`':
			b.WriteString("\\`")
		default:
			b.WriteByte(s[i])
		}
	}
	return b.String()
}

// mergeBackticksInLines collapses multi-line backtick literals into
// single logical lines (with embedded newlines escaped as `\n` so
// the line-based parser doesn't split on them). Only applied to the
// slice of lines belonging to a new-shape function body — running
// it globally would eat backticks inside `template:` blocks (e.g.
// Markdown code fences), which are raw text, not string delimiters.
func mergeBackticksInLines(lines []string) []string {
	var out []string
	var cur strings.Builder
	inBacktick := false
	for _, ln := range lines {
		if inBacktick {
			// We're carrying an open backtick from a previous line.
			// Re-attach this line with a literal `\n` separator and
			// continue scanning.
			cur.WriteString(`\n`)
		} else if cur.Len() > 0 {
			out = append(out, cur.String())
			cur.Reset()
		}
		// Walk this line; toggle inBacktick on each (unescaped) `.
		for i := 0; i < len(ln); i++ {
			c := ln[i]
			if c == '\\' && i+1 < len(ln) && inBacktick {
				cur.WriteByte(c)
				cur.WriteByte(ln[i+1])
				i++
				continue
			}
			if c == '`' {
				inBacktick = !inBacktick
			}
			cur.WriteByte(c)
		}
	}
	if cur.Len() > 0 {
		out = append(out, cur.String())
	}
	return out
}

type capyLibParser struct {
	lines  []string
	lineNo int
}

func (p *capyLibParser) errf(format string, args ...any) error {
	return fmt.Errorf("line %d: "+format, append([]any{p.lineNo}, args...)...)
}

// peekLine returns the next non-blank, non-comment line WITHOUT advancing.
// Returns (line, indent, ok). A comment is `#` as the first non-space char.
func (p *capyLibParser) peekLine() (string, int, bool) {
	for i := p.lineNo; i < len(p.lines); i++ {
		raw := p.lines[i]
		stripped := strings.TrimSpace(raw)
		if stripped == "" || strings.HasPrefix(stripped, "#") {
			continue
		}
		indent := indentOf(raw)
		return raw, indent, true
	}
	return "", 0, false
}

// nextLine returns the next non-blank, non-comment line and advances past it.
func (p *capyLibParser) nextLine() (string, int, bool) {
	for p.lineNo < len(p.lines) {
		raw := p.lines[p.lineNo]
		p.lineNo++
		stripped := strings.TrimSpace(raw)
		if stripped == "" || strings.HasPrefix(stripped, "#") {
			continue
		}
		return raw, indentOf(raw), true
	}
	return "", 0, false
}

// isIdent reports whether s looks like a bareword identifier — used to
// distinguish a TYPE token from a trailing description string in
// `arg capture NAME TYPE [DESC]`. Our tokenizer already strips quotes
// from string tokens, so a description like "An email address" arrives
// here as a single token with spaces in it (which fails isIdent).
func isIdent(s string) bool {
	if s == "" {
		return false
	}
	for i, c := range s {
		if i == 0 {
			if !(c == '_' || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
				return false
			}
			continue
		}
		if !(c == '_' || (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
			return false
		}
	}
	return true
}

func indentOf(s string) int {
	n := 0
	for _, c := range s {
		if c == ' ' {
			n++
		} else if c == '\t' {
			n += 4
		} else {
			break
		}
	}
	return n
}

func (p *capyLibParser) parseTop() (RawLibrary, error) {
	lib := RawLibrary{
		Functions: map[string]RawFunction{},
		Types:     map[string]RawType{},
		Context:   map[string]any{},
		Files:     map[string]string{},
		Commands:  map[string]RawCommand{},
	}

	for {
		line, indent, ok := p.peekLine()
		if !ok {
			break
		}
		if indent != 0 {
			return lib, p.errf("unexpected indentation at top level: %q", strings.TrimSpace(line))
		}
		tokens, err := tokenizeLibLine(line)
		if err != nil {
			return lib, p.errf("%v", err)
		}
		if len(tokens) == 0 {
			p.nextLine()
			continue
		}
		switch tokens[0] {
		case "name":
			// `name "STR"` — manifest field.
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("name requires a string value")
			}
			lib.LibName = tokens[1]
		case "version":
			// `version "STR"` — manifest field (semver string).
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("version requires a string value")
			}
			lib.LibVersion = tokens[1]
		case "command":
			// `command "NAME" ... end` block.
			if len(tokens) < 2 {
				return lib, p.errf("command requires a name string")
			}
			cmdName := tokens[1]
			p.nextLine()
			cmd, err := p.parseCommandBlock()
			if err != nil {
				return lib, err
			}
			lib.Commands[cmdName] = cmd
		case "impl":
			// `impl "NAME" "FILE" ... end` block declaring an
			// implementation of the library's interface.
			if len(tokens) < 3 {
				return lib, p.errf("impl requires name + file string, e.g. `impl \"d3\" \"impl/d3.capy\"`")
			}
			name := tokens[1]
			file := tokens[2]
			p.nextLine()
			ri, err := p.parseImplBlock()
			if err != nil {
				return lib, err
			}
			ri.Name = name
			ri.File = file
			if lib.Impls == nil {
				lib.Impls = map[string]RawImpl{}
			}
			lib.Impls[name] = ri
			if ri.IsDefault {
				lib.DefaultImpl = name
			}
		case "default_impl":
			// `default_impl "NAME"` alternative to the inline
			// `default` directive inside an impl block.
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("default_impl requires a string name")
			}
			lib.DefaultImpl = tokens[1]
		case "extension":
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("extension requires a value")
			}
			lib.Extension = tokens[1]
		case "output_file":
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("output_file requires a value")
			}
			lib.OutputFile = tokens[1]
		case "description":
			p.nextLine()
			if len(tokens) < 2 {
				return lib, p.errf("description requires a string")
			}
			lib.Description = tokens[1]
		case "function":
			if len(tokens) < 2 {
				return lib, p.errf("function requires a name")
			}
			name := tokens[1]
			p.nextLine()
			fn, err := p.parseFunction()
			if err != nil {
				return lib, err
			}
			lib.Functions[name] = fn
		case "type":
			if len(tokens) < 2 {
				return lib, p.errf("type requires a name")
			}
			name := tokens[1]
			p.nextLine()
			td, err := p.parseType()
			if err != nil {
				return lib, err
			}
			lib.Types[name] = td
		case "context":
			p.nextLine()
			if err := p.parseContext(lib.Context); err != nil {
				return lib, err
			}
		case "import":
			p.nextLine()
			if len(tokens) != 2 {
				return lib, p.errf("import requires one path string")
			}
			lib.Imports = append(lib.Imports, tokens[1])
		case "preprocess":
			// preprocess
			//     include "@import"
			//     include "@include"
			// end
			//
			// Each `include "X"` line opts the library into recognising
			// X as a text-level inclusion directive at the start of a
			// source line. With no `preprocess` block, NO directive
			// works — Capy stays true to "zero predefined grammar".
			p.nextLine()
			for {
				ln, indent, ok := p.peekLine()
				if !ok {
					return lib, p.errf("unexpected EOF inside preprocess")
				}
				toks, err := tokenizeLibLine(ln)
				if err != nil {
					return lib, p.errf("%v", err)
				}
				// `end` at indent 0 closes the block (matches context's
				// behaviour).
				if indent == 0 && len(toks) == 1 && toks[0] == "end" {
					p.nextLine()
					break
				}
				if indent == 0 {
					return lib, p.errf("expected `end` to close preprocess, got %q", strings.TrimSpace(ln))
				}
				if len(toks) == 0 {
					p.nextLine()
					continue
				}
				if len(toks) == 2 && toks[0] == "include" {
					lib.Preprocess = append(lib.Preprocess, toks[1])
					p.nextLine()
					continue
				}
				return lib, p.errf("inside preprocess: expected `include \"@NAME\"` or `end`, got %q", strings.TrimSpace(ln))
			}
		case "comments":
			// comments
			//     line "#"
			//     line "//"
			// end
			//
			// Each `line "MARKER"` opts user-script lexing into
			// recognising MARKER as a line comment. Empty block →
			// no comment syntax (default — Capy has zero predefined
			// grammar, comments included).
			p.nextLine()
			for {
				ln, indent, ok := p.peekLine()
				if !ok {
					return lib, p.errf("unexpected EOF inside comments")
				}
				toks, err := tokenizeLibLine(ln)
				if err != nil {
					return lib, p.errf("%v", err)
				}
				if indent == 0 && len(toks) == 1 && toks[0] == "end" {
					p.nextLine()
					break
				}
				if indent == 0 {
					return lib, p.errf("expected `end` to close comments, got %q", strings.TrimSpace(ln))
				}
				if len(toks) == 0 {
					p.nextLine()
					continue
				}
				if len(toks) == 2 && toks[0] == "line" {
					lib.Comments = append(lib.Comments, toks[1])
					p.nextLine()
					continue
				}
				return lib, p.errf("inside comments: expected `line \"MARKER\"` or `end`, got %q", strings.TrimSpace(ln))
			}
		case "file":
			// `file "path" ... end` — multi-file output. The body is
			// an inner-DSL write-style block (same grammar as
			// `file_template` and `function` bodies).
			if len(tokens) < 2 {
				return lib, p.errf("file requires a path string")
			}
			path := tokens[1]
			p.nextLine()
			body, err := p.parseFunctionBodyStatements(0)
			if err != nil {
				return lib, err
			}
			// Consume the closing `end` line.
			if ln, indent, ok := p.peekLine(); ok && indent == 0 {
				toks, _ := tokenizeLibLine(ln)
				if len(toks) == 1 && toks[0] == "end" {
					p.nextLine()
				}
			}
			// Stash the raw body with the sentinel prefix; the loader
			// parses it as inner-DSL and the renderer walks the AST.
			lib.Files[path] = "\x00NEW_SHAPE\x00" + body
		case "file_template":
			// New shape: file_template ... end with an inner-DSL body
			// (sequence of `write`/`for`/`if`/`set` statements). The
			// loader runs the same translator used for function bodies.
			p.nextLine()
			body, err := p.parseFunctionBodyStatements(0)
			if err != nil {
				return lib, err
			}
			// Consume the closing `end`.
			if ln, indent, ok := p.peekLine(); ok && indent == 0 {
				toks, _ := tokenizeLibLine(ln)
				if len(toks) == 1 && toks[0] == "end" {
					p.nextLine()
				}
			}
			lib.FileTemplate = "\x00NEW_SHAPE\x00" + body
		default:
			return lib, p.errf("unknown top-level directive: %q", tokens[0])
		}
	}
	return lib, nil
}

// parseFunction reads everything until a matching `end` at indent 0 (function
// keyword's column). The `function NAME` line has already been consumed.
func (p *capyLibParser) parseFunction() (RawFunction, error) {
	var fn RawFunction
	var block RawBlock
	blockSet := false

	for {
		line, indent, ok := p.peekLine()
		if !ok {
			return fn, p.errf("unexpected EOF inside function")
		}
		if indent == 0 {
			// must be the closing `end`
			tokens, err := tokenizeLibLine(line)
			if err != nil {
				return fn, p.errf("%v", err)
			}
			if len(tokens) == 1 && tokens[0] == "end" {
				p.nextLine()
				if blockSet {
					fn.Block = &block
				}
				return fn, nil
			}
			return fn, p.errf("expected `end` to close function, got %q", strings.TrimSpace(line))
		}

		tokens, err := tokenizeLibLine(line)
		if err != nil {
			return fn, p.errf("%v", err)
		}
		if len(tokens) == 0 {
			p.nextLine()
			continue
		}

		switch tokens[0] {
		case "description":
			p.nextLine()
			if len(tokens) != 2 {
				return fn, p.errf("description requires one string argument")
			}
			fn.Description = tokens[1]
		case "priority":
			p.nextLine()
			if len(tokens) != 2 {
				return fn, p.errf("priority requires one integer argument")
			}
			n, err := strconv.Atoi(tokens[1])
			if err != nil {
				return fn, p.errf("priority: %v", err)
			}
			fn.Priority = n
		case "bare":
			p.nextLine()
			if len(tokens) != 1 {
				return fn, p.errf("bare takes no arguments")
			}
			fn.Bare = true
		case "when_followed_by":
			p.nextLine()
			if len(tokens) != 2 || tokens[1] != "indent" {
				return fn, p.errf("when_followed_by currently supports only `indent` (e.g. `when_followed_by indent`)")
			}
			fn.FollowedByIndent = true
		case "when_not_followed_by":
			p.nextLine()
			if len(tokens) != 2 || tokens[1] != "indent" {
				return fn, p.errf("when_not_followed_by currently supports only `indent` (e.g. `when_not_followed_by indent`)")
			}
			fn.NotFollowedByIndent = true
		case "arg":
			p.nextLine()
			if len(tokens) < 2 {
				return fn, p.errf("arg requires a kind")
			}
			switch tokens[1] {
			case "literal":
				// `arg literal "TEXT"` or `arg literal "TEXT" "DESCRIPTION"`
				if len(tokens) < 3 || len(tokens) > 4 {
					return fn, p.errf("arg literal requires a value (and optional description string)")
				}
				ra := RawArg{Kind: "literal", Value: tokens[2]}
				if len(tokens) == 4 {
					ra.Description = tokens[3]
				}
				fn.Args = append(fn.Args, ra)
			case "capture":
				// `arg capture NAME [TYPE] [default "VALUE"] [DESCRIPTION]`
				if len(tokens) < 3 {
					return fn, p.errf("arg capture NAME [TYPE] [default \"VALUE\"] [DESCRIPTION]")
				}
				a := RawArg{Kind: "capture", Name: tokens[2], Type: "any"}
				rest := tokens[3:]
				idx := 0
				// Optional TYPE, with an optional repetition suffix
				// (`type*` / `type+`) for function-typed captures. The
				// token must not be the `default` or `sep` keyword.
				if idx < len(rest) && rest[idx] != "default" && rest[idx] != "sep" && rest[idx] != "join" {
					typ := rest[idx]
					if strings.HasSuffix(typ, "*") {
						a.Repeat = "*"
						typ = strings.TrimSuffix(typ, "*")
					} else if strings.HasSuffix(typ, "+") {
						a.Repeat = "+"
						typ = strings.TrimSuffix(typ, "+")
					}
					if isIdent(typ) {
						a.Type = typ
						idx++
					} else if a.Repeat != "" {
						return fn, p.errf("arg capture: repetition suffix requires a function/type name before %q", rest[idx])
					}
				}
				// Optional `sep "VALUE"` — a separator literal between
				// repetitions of a function-typed capture.
				if idx < len(rest) && rest[idx] == "sep" {
					if idx+1 >= len(rest) {
						return fn, p.errf("arg capture: `sep` requires a value")
					}
					a.Sep = rest[idx+1]
					idx += 2
				}
				// Optional `join "VALUE"` -- an OUTPUT separator inserted
				// between rendered sub-results of a repeated capture.
				if idx < len(rest) && rest[idx] == "join" {
					if idx+1 >= len(rest) {
						return fn, p.errf("arg capture: `join` requires a value")
					}
					a.Join = rest[idx+1]
					idx += 2
				}
				// Optional `default "VALUE"` — marks the arg optional.
				if idx < len(rest) && rest[idx] == "default" {
					if idx+1 >= len(rest) {
						return fn, p.errf("arg capture: `default` requires a value")
					}
					a.Optional = true
					a.Default = rest[idx+1]
					idx += 2
				}
				// Optional trailing DESCRIPTION (one token).
				if idx < len(rest) {
					a.Description = rest[idx]
					idx++
				}
				if idx < len(rest) {
					return fn, p.errf("arg capture: unexpected extra tokens after NAME [TYPE] [default \"VALUE\"] [DESCRIPTION]")
				}
				fn.Args = append(fn.Args, a)
			default:
				return fn, p.errf("unknown arg kind %q", tokens[1])
			}
		case "block_closer":
			p.nextLine()
			if len(tokens) != 2 {
				return fn, p.errf("block_closer requires a function name")
			}
			block.Closer = tokens[1]
			blockSet = true
		case "block_dedent":
			p.nextLine()
			if len(tokens) != 1 {
				return fn, p.errf("block_dedent takes no arguments")
			}
			block.IsDedent = true
			blockSet = true
		case "block_verbatim":
			// `block_verbatim <CLOSER>` — body is captured as raw
			// source bytes (no nested parsing) until the named closer
			// keyword appears at the parent indent. Used for code
			// blocks, embedded HTML, anywhere the body is data not
			// grammar.
			p.nextLine()
			if len(tokens) != 2 {
				return fn, p.errf("block_verbatim requires a closer-function name (e.g. `block_verbatim end`)")
			}
			block.Closer = tokens[1]
			block.IsVerbatim = true
			blockSet = true
		case "block_sections":
			// `block_sections SECTION... closer CLOSER` — a multi-section
			// block (try/rescue/finally). Each SECTION keyword appears at
			// the opener's indent and introduces its own indented sub-body;
			// the block ends at CLOSER. Renders to `${body}` (main) plus a
			// local per section (`${rescue}`, `${finally}`).
			p.nextLine()
			if len(tokens) < 4 || tokens[len(tokens)-2] != "closer" {
				return fn, p.errf("block_sections SECTION... closer CLOSER (e.g. `block_sections rescue finally closer end`)")
			}
			block.Closer = tokens[len(tokens)-1]
			block.Sections = append([]string(nil), tokens[1:len(tokens)-2]...)
			blockSet = true
		case "block_open":
			p.nextLine()
			if len(tokens) != 4 || tokens[2] != "close" {
				return fn, p.errf("block_open <OPEN> close <CLOSE>")
			}
			block.Open = tokens[1]
			block.Close = tokens[3]
			blockSet = true
		case "block_close_seq":
			// `block_close_seq "</div>"` (fixed) or
			// `block_close_seq "</" name ">"` (capture-bound) — the body is
			// a free-flowing sequence of statements terminated by the
			// closing sequence. Quoted segments are literals; bare
			// identifiers reference captures bound by the opener, so the
			// closer can depend on the opener (matched-pair HTML, generic
			// over tag name, with mismatch detection).
			segs, err := parseCloseSeqSegs(line)
			if err != nil {
				return fn, p.errf("%v", err)
			}
			if len(segs) == 0 {
				return fn, p.errf("block_close_seq requires a closing sequence (e.g. `block_close_seq \"</div>\"` or `block_close_seq \"</\" name \">\"`)")
			}
			p.nextLine()
			block.CloseSeq = segs
			blockSet = true
		default:
			// New-shape body: anything that isn't one of the recognised
			// header directives is taken to be an inner-DSL statement
			// (write / set / append / for / if / …). Collect all such
			// lines (and any indented continuations) until the closing
			// `end` of the function at indent 0.
			body, err := p.parseFunctionBodyStatements(indent)
			if err != nil {
				return fn, err
			}
			fn.Body = body
		}
	}
}

// parseFunctionBodyStatements collects the remaining lines of a
// function (from the current line until the closing `end` at indent
// 0) as raw text. The text is later handed to the inner-DSL parser,
// then to the translator that splits it into Template + Run.
//
// The first line is captured at its existing column; subsequent
// lines are emitted verbatim (preserving their relative indent so
// nested for/if blocks survive). Trailing whitespace is preserved
// because backtick literals carry significant whitespace.
// parseImplBlock reads everything from after the `impl "NAME"
// "FILE"` header until a matching `end` at indent 0. Recognised
// directives inside the body:
//
//	description "STR"
//	version     "STR"
//	default                 # mark as the default impl
func (p *capyLibParser) parseImplBlock() (RawImpl, error) {
	var ri RawImpl
	for {
		ln, indent, ok := p.peekLine()
		if !ok {
			return ri, p.errf("unexpected EOF inside impl")
		}
		toks, err := tokenizeLibLine(ln)
		if err != nil {
			return ri, p.errf("%v", err)
		}
		if indent == 0 && len(toks) == 1 && toks[0] == "end" {
			p.nextLine()
			break
		}
		if indent == 0 {
			return ri, p.errf("expected `end` to close impl, got %q", strings.TrimSpace(ln))
		}
		if len(toks) == 0 {
			p.nextLine()
			continue
		}
		switch toks[0] {
		case "description":
			if len(toks) < 2 {
				return ri, p.errf("description requires a string value")
			}
			ri.Description = toks[1]
		case "version":
			if len(toks) < 2 {
				return ri, p.errf("version requires a string value")
			}
			ri.Version = toks[1]
		case "default":
			ri.IsDefault = true
		default:
			return ri, p.errf("unknown directive inside impl: %q (allowed: description / version / default)", toks[0])
		}
		p.nextLine()
	}
	return ri, nil
}

// parseCommandBlock reads everything from after the
// `command "NAME"` header line until a matching `end` at indent 0.
// Currently recognises one header directive — `description "STR"`
// — followed by an inner-DSL body that the loader hands to the
// inner-DSL parser. The body uses the same multi-line-backtick
// merging as function bodies.
func (p *capyLibParser) parseCommandBlock() (RawCommand, error) {
	var cmd RawCommand
	// First pass: collect optional header directives (description,
	// future arg / flag declarations) until we hit a non-header
	// statement, then everything after is the body.
	headerDone := false
	var bodyRaws []string
	inBacktick := false
	for {
		if p.lineNo >= len(p.lines) {
			return cmd, p.errf("unexpected EOF inside command")
		}
		ln := p.lines[p.lineNo]
		// While inside a multi-line backtick, every line belongs to
		// the body regardless of indent.
		if inBacktick {
			bodyRaws = append(bodyRaws, ln)
			p.lineNo++
			for i := 0; i < len(ln); i++ {
				c := ln[i]
				if c == '\\' && i+1 < len(ln) {
					i++
					continue
				}
				if c == '`' {
					inBacktick = !inBacktick
				}
			}
			continue
		}
		stripped := strings.TrimSpace(ln)
		if stripped == "" || strings.HasPrefix(stripped, "#") {
			if headerDone {
				bodyRaws = append(bodyRaws, ln)
			}
			p.lineNo++
			continue
		}
		leading := 0
		for leading < len(ln) && (ln[leading] == ' ' || ln[leading] == '\t') {
			leading++
		}
		if leading == 0 {
			// indent 0 — should be the closing `end`.
			toks, err := tokenizeLibLine(ln)
			if err != nil {
				return cmd, p.errf("%v", err)
			}
			if len(toks) == 1 && toks[0] == "end" {
				p.nextLine()
				break
			}
			return cmd, p.errf("expected `end` to close command, got %q", stripped)
		}
		// Indented line. Check if it's a recognised header directive
		// (allowed until headerDone flips).
		toks, err := tokenizeLibLine(ln)
		if err != nil {
			return cmd, p.errf("%v", err)
		}
		if !headerDone && len(toks) >= 1 {
			switch toks[0] {
			case "description":
				if len(toks) < 2 {
					return cmd, p.errf("description requires a string value")
				}
				cmd.Description = toks[1]
				p.nextLine()
				continue
			case "arg":
				// arg "name"                          (optional, no description)
				// arg "name" required                 (required, no description)
				// arg "name" required "description"
				// arg "name" optional "description"
				if len(toks) < 2 {
					return cmd, p.errf("arg requires a name string")
				}
				ra := RawCommandArg{Name: toks[1]}
				if len(toks) >= 3 {
					switch toks[2] {
					case "required":
						ra.Required = true
					case "optional":
						ra.Required = false
					default:
						return cmd, p.errf("arg: expected `required` or `optional` after name, got %q", toks[2])
					}
				}
				if len(toks) >= 4 {
					ra.Description = toks[3]
				}
				cmd.Args = append(cmd.Args, ra)
				p.nextLine()
				continue
			case "flag":
				// flag "--name"
				// flag "--name" "description"
				// flag "--name" "description" default "value"
				// flag "--name" bool "description"
				if len(toks) < 2 {
					return cmd, p.errf("flag requires a name string")
				}
				rf := RawCommandFlag{Name: toks[1]}
				i := 2
				if i < len(toks) && toks[i] == "bool" {
					rf.IsBool = true
					i++
				}
				if i < len(toks) {
					rf.Description = toks[i]
					i++
				}
				if i+1 < len(toks) && toks[i] == "default" {
					rf.Default = toks[i+1]
				}
				cmd.Flags = append(cmd.Flags, rf)
				p.nextLine()
				continue
			}
			// First non-header line marks the body's start.
			headerDone = true
		}
		// Body line. Track backtick toggles so multi-line literals
		// keep the inner statements grouped properly.
		bodyRaws = append(bodyRaws, ln)
		p.lineNo++
		for i := 0; i < len(ln); i++ {
			c := ln[i]
			if c == '\\' && i+1 < len(ln) {
				i++
				continue
			}
			if c == '`' {
				inBacktick = !inBacktick
			}
		}
	}
	// Merge backticks then dedent.
	bodyRaws = mergeBackticksInLines(bodyRaws)
	minIndent := -1
	for _, ln := range bodyRaws {
		if strings.TrimSpace(ln) == "" {
			continue
		}
		leading := 0
		for leading < len(ln) && (ln[leading] == ' ' || ln[leading] == '\t') {
			leading++
		}
		if minIndent < 0 || leading < minIndent {
			minIndent = leading
		}
	}
	if minIndent < 0 {
		return cmd, nil
	}
	for i, ln := range bodyRaws {
		if len(ln) >= minIndent {
			bodyRaws[i] = ln[minIndent:]
		}
	}
	cmd.Body = strings.Join(bodyRaws, "\n") + "\n"
	return cmd, nil
}

func (p *capyLibParser) parseFunctionBodyStatements(startIndent int) (string, error) {
	// Collect raw lines (NOT skipping blanks/comments via peekLine —
	// we need every line to track multi-line backtick state). Stop
	// at the next non-blank, non-backtick-continuation line at
	// indent 0 (the function's closing `end`).
	var raws []string
	inBacktick := false
	// tmplStack holds the indent of each open `template … end` block.
	// While the stack is non-empty we're inside a template body, where
	// a column-0 line is content (e.g. a flush-left `${indent 2 body}`)
	// — NOT the function's closing `end`. Without this, such a line
	// terminated body collection early (missing2.md §4a).
	var tmplStack []int
	for {
		if p.lineNo >= len(p.lines) {
			return "", p.errf("unexpected EOF inside function body")
		}
		ln := p.lines[p.lineNo]
		// While inside a multi-line backtick, every line belongs to
		// the body regardless of indent. Track toggles on every
		// unescaped ` in the line.
		if inBacktick {
			raws = append(raws, ln)
			p.lineNo++
			for i := 0; i < len(ln); i++ {
				c := ln[i]
				if c == '\\' && i+1 < len(ln) {
					i++
					continue
				}
				if c == '`' {
					inBacktick = !inBacktick
				}
			}
			continue
		}
		stripped := strings.TrimSpace(ln)
		// Blank line OR comment line inside the body: keep going.
		if stripped == "" || strings.HasPrefix(stripped, "#") {
			raws = append(raws, ln)
			p.lineNo++
			continue
		}
		// Compute indent for non-blank.
		leading := 0
		for leading < len(ln) && (ln[leading] == ' ' || ln[leading] == '\t') {
			leading++
		}
		// Track `template … end` nesting. A bare `template` opens a
		// block; a bare `end` at the matching indent closes it. (Both
		// are still collected as ordinary body lines below — the
		// template→write rewrite happens in a later pass.)
		if leading > 0 && strings.TrimSpace(ln) == "template" {
			tmplStack = append(tmplStack, leading)
		} else if len(tmplStack) > 0 && strings.TrimSpace(ln) == "end" &&
			leading == tmplStack[len(tmplStack)-1] {
			tmplStack = tmplStack[:len(tmplStack)-1]
		}
		if leading == 0 && len(tmplStack) == 0 {
			// indent==0 outside any template body = the function's
			// closing `end`. Stop here.
			break
		}
		raws = append(raws, ln)
		p.lineNo++
		// After consuming the line, scan for unescaped backticks to
		// learn if we're now inside one (so subsequent column-0 lines
		// keep being collected as the backtick body).
		for i := 0; i < len(ln); i++ {
			c := ln[i]
			if c == '\\' && i+1 < len(ln) {
				i++
				continue
			}
			if c == '`' {
				inBacktick = !inBacktick
			}
		}
	}
	// Sugar pass: rewrite `template ... end` blocks into the
	// equivalent multi-line `write ` ... ` ` literal BEFORE merging
	// backticks. The synth is byte-equivalent to what a library
	// author could have hand-written, so everything downstream
	// (merge, dedent, inner-DSL parse, render) stays unchanged.
	rewritten, err := rewriteTemplateBlocks(raws, p.lineNo-len(raws))
	if err != nil {
		return "", err
	}
	raws = rewritten
	// Merge multi-line backticks inside the collected body slice.
	raws = mergeBackticksInLines(raws)
	// Strip the deepest common leading indent so the inner-DSL parser
	// (which expects column-0 statements) is happy.
	minIndent := -1
	for _, ln := range raws {
		if strings.TrimSpace(ln) == "" {
			continue
		}
		leading := 0
		for leading < len(ln) && (ln[leading] == ' ' || ln[leading] == '\t') {
			leading++
		}
		if minIndent < 0 || leading < minIndent {
			minIndent = leading
		}
	}
	if minIndent < 0 {
		return "", nil
	}
	for i, ln := range raws {
		if len(ln) >= minIndent {
			raws[i] = ln[minIndent:]
		}
	}
	return strings.Join(raws, "\n") + "\n", nil
}

// parseContext reads the body of a `context ... end` block. Each line is
//
//	NAME [ ] | { } | <STRING> | <NUMBER>
//
// where `[]` means initial empty list, `{}` means empty map, a quoted string
// means a string literal default, and a bare number means a numeric default.
// More elaborate shapes belong in YAML.
func (p *capyLibParser) parseContext(ctx map[string]any) error {
	for {
		line, indent, ok := p.peekLine()
		if !ok {
			return p.errf("unexpected EOF inside context")
		}
		if indent == 0 {
			tokens, err := tokenizeLibLine(line)
			if err != nil {
				return p.errf("%v", err)
			}
			if len(tokens) == 1 && tokens[0] == "end" {
				p.nextLine()
				return nil
			}
			return p.errf("expected `end` to close context, got %q", strings.TrimSpace(line))
		}
		tokens, err := tokenizeLibLine(line)
		if err != nil {
			return p.errf("%v", err)
		}
		if len(tokens) == 0 {
			p.nextLine()
			continue
		}
		p.nextLine()
		if len(tokens) < 2 {
			return p.errf("context entry needs `NAME VALUE`, got %v", tokens)
		}
		name := tokens[0]
		rest := tokens[1:]
		val, err := parseContextValue(rest)
		if err != nil {
			return p.errf("context %q: %v", name, err)
		}
		ctx[name] = val
	}
}

func parseContextValue(toks []string) (any, error) {
	if len(toks) == 1 {
		t := toks[0]
		if t == "[]" {
			return []any{}, nil
		}
		if t == "{}" {
			return map[string]any{}, nil
		}
		if t == "true" {
			return true, nil
		}
		if t == "false" {
			return false, nil
		}
		// numeric?
		if n, err := strconv.ParseInt(t, 10, 64); err == nil {
			return n, nil
		}
		if f, err := strconv.ParseFloat(t, 64); err == nil {
			return f, nil
		}
		// fallback: string
		return t, nil
	}
	if toks[0] == "[" && toks[len(toks)-1] == "]" {
		// inline list of scalars: [ a b c ]
		out := make([]any, 0, len(toks)-2)
		for _, t := range toks[1 : len(toks)-1] {
			v, err := parseContextValue([]string{t})
			if err != nil {
				return nil, err
			}
			out = append(out, v)
		}
		return out, nil
	}
	return strings.Join(toks, " "), nil
}

// parseType reads `type NAME` body lines until matching `end` at column 0.
// Body lines accept `base TYPE`, `pattern STRING`, `options STR STR ...`.
func (p *capyLibParser) parseType() (RawType, error) {
	var td RawType
	for {
		line, indent, ok := p.peekLine()
		if !ok {
			return td, p.errf("unexpected EOF inside type")
		}
		if indent == 0 {
			tokens, err := tokenizeLibLine(line)
			if err != nil {
				return td, p.errf("%v", err)
			}
			if len(tokens) == 1 && tokens[0] == "end" {
				p.nextLine()
				return td, nil
			}
			return td, p.errf("expected `end` to close type, got %q", strings.TrimSpace(line))
		}
		tokens, err := tokenizeLibLine(line)
		if err != nil {
			return td, p.errf("%v", err)
		}
		if len(tokens) == 0 {
			p.nextLine()
			continue
		}
		switch tokens[0] {
		case "description":
			p.nextLine()
			if len(tokens) != 2 {
				return td, p.errf("description requires one string argument")
			}
			td.Description = tokens[1]
		case "base":
			p.nextLine()
			if len(tokens) != 2 {
				return td, p.errf("base requires a type name")
			}
			td.Base = tokens[1]
		case "pattern":
			p.nextLine()
			if len(tokens) != 2 {
				return td, p.errf("pattern requires one regex string")
			}
			td.Pattern = tokens[1]
		case "options":
			p.nextLine()
			if len(tokens) < 2 {
				return td, p.errf("options requires one or more values")
			}
			td.Options = append(td.Options, tokens[1:]...)
		case "group_open":
			p.nextLine()
			if len(tokens) != 2 {
				return td, p.errf("group_open requires one delimiter string")
			}
			td.GroupOpen = tokens[1]
		case "group_close":
			p.nextLine()
			if len(tokens) != 2 {
				return td, p.errf("group_close requires one delimiter string")
			}
			td.GroupClose = tokens[1]
		default:
			return td, p.errf("unknown directive inside type: %q", tokens[0])
		}
	}
}

func stripIndent(s string, n int) string {
	i := 0
	stripped := 0
	for i < len(s) && stripped < n {
		c := s[i]
		if c == ' ' {
			stripped++
			i++
		} else if c == '\t' {
			stripped += 4
			i++
		} else {
			break
		}
	}
	return s[i:]
}

// tokenizeLibLine splits a line into tokens, respecting double-quoted strings
// with Go-style escapes. The result is the list of token *contents* (quotes
// removed, escapes decoded). Comments after `#` (outside strings) are dropped.
// parseCloseSeqSegs parses the arguments of a `block_close_seq` directive
// line into segments, preserving which were QUOTED (→ literal) vs BARE
// (→ capture reference). It must read the raw line because
// tokenizeLibLine de-quotes, erasing that distinction. The leading
// `block_close_seq` keyword is skipped.
func parseCloseSeqSegs(line string) ([]RawCloseSeg, error) {
	s := strings.TrimRight(line, " \t")
	i := 0
	for i < len(s) && (s[i] == ' ' || s[i] == '\t') {
		i++
	}
	var segs []RawCloseSeg
	first := true
	for i < len(s) {
		c := s[i]
		switch {
		case c == ' ' || c == '\t':
			i++
		case c == '#':
			i = len(s)
		case c == '"':
			j := i + 1
			var b strings.Builder
			for j < len(s) && s[j] != '"' {
				if s[j] == '\\' && j+1 < len(s) {
					switch s[j+1] {
					case 'n':
						b.WriteByte('\n')
					case 't':
						b.WriteByte('\t')
					case 'r':
						b.WriteByte('\r')
					case '"':
						b.WriteByte('"')
					case '\\':
						b.WriteByte('\\')
					default:
						b.WriteByte(s[j+1])
					}
					j += 2
					continue
				}
				b.WriteByte(s[j])
				j++
			}
			if j >= len(s) {
				return nil, fmt.Errorf("block_close_seq: unterminated string literal")
			}
			if !first { // skip the directive keyword (always bare, position 0)
				segs = append(segs, RawCloseSeg{Text: b.String(), IsRef: false})
			}
			first = false
			i = j + 1
		default:
			j := i
			for j < len(s) && s[j] != ' ' && s[j] != '\t' && s[j] != '#' {
				j++
			}
			word := s[i:j]
			if first {
				// This is the `block_close_seq` keyword itself.
				first = false
			} else {
				segs = append(segs, RawCloseSeg{Text: word, IsRef: true})
			}
			i = j
		}
	}
	return segs, nil
}

func tokenizeLibLine(line string) ([]string, error) {
	var toks []string
	s := strings.TrimRight(line, " \t")
	i := 0
	// skip leading indent
	for i < len(s) && (s[i] == ' ' || s[i] == '\t') {
		i++
	}
	for i < len(s) {
		c := s[i]
		switch {
		case c == ' ' || c == '\t':
			i++
		case c == '#':
			return toks, nil
		case c == '"':
			j := i + 1
			var b strings.Builder
			for j < len(s) && s[j] != '"' {
				if s[j] == '\\' && j+1 < len(s) {
					switch s[j+1] {
					case 'n':
						b.WriteByte('\n')
					case 't':
						b.WriteByte('\t')
					case 'r':
						b.WriteByte('\r')
					case '"':
						b.WriteByte('"')
					case '\\':
						b.WriteByte('\\')
					default:
						b.WriteByte(s[j+1])
					}
					j += 2
					continue
				}
				b.WriteByte(s[j])
				j++
			}
			if j >= len(s) {
				return nil, fmt.Errorf("unterminated string literal")
			}
			toks = append(toks, b.String())
			i = j + 1
		default:
			j := i
			for j < len(s) && s[j] != ' ' && s[j] != '\t' && s[j] != '#' {
				j++
			}
			toks = append(toks, s[i:j])
			i = j
		}
	}
	return toks, nil
}
