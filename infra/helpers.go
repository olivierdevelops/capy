// Code-generator helpers — small, target-language-aware string-manipulation
// functions called from the inner-DSL renderer (see RenderAST and
// interpolateRender). The helper table used to live behind a Go text/template
// FuncMap; that engine is gone now and the helpers are plain map-of-funcs
// dispatched by name through ApplyHelper.
package infra

import (
	"encoding/json"
	"fmt"
	"sort"
	"strconv"
	"strings"
)

var funcs = map[string]any{
	// indent N: indent every line of s by N spaces. Useful for `body`.
	"indent": func(n int, s string) string {
		pad := strings.Repeat(" ", n)
		lines := strings.Split(s, "\n")
		for i, l := range lines {
			if l == "" {
				continue
			}
			lines[i] = pad + l
		}
		return strings.Join(lines, "\n")
	},
	"lower": strings.ToLower,
	"upper": strings.ToUpper,
	// pascalCase converts a human-readable string ("Habit Tracker", "habit
	// tracker", "habit-tracker", "habit_tracker") into a PascalCase
	// identifier ("HabitTracker"). Useful when target languages require an
	// identifier and the source has a friendly display name.
	"pascalCase": func(s any) string {
		text := toStringAny(s)
		// First strip surrounding quotes from string captures.
		if len(text) >= 2 && (text[0] == '"' || text[0] == '\'' || text[0] == '`') && text[0] == text[len(text)-1] {
			text = text[1 : len(text)-1]
		}
		// Split on space, dash, underscore.
		var b strings.Builder
		nextUpper := true
		for _, r := range text {
			if r == ' ' || r == '_' || r == '-' || r == '.' {
				nextUpper = true
				continue
			}
			if nextUpper {
				b.WriteRune(toUpperRune(r))
				nextUpper = false
			} else {
				b.WriteRune(r)
			}
		}
		return b.String()
	},
	// camelCase is pascalCase with the first char lowered.
	"camelCase": func(s any) string {
		text := toStringAny(s)
		if len(text) >= 2 && (text[0] == '"' || text[0] == '\'' || text[0] == '`') && text[0] == text[len(text)-1] {
			text = text[1 : len(text)-1]
		}
		var b strings.Builder
		nextUpper := false
		first := true
		for _, r := range text {
			if r == ' ' || r == '_' || r == '-' || r == '.' {
				nextUpper = true
				continue
			}
			if first {
				b.WriteRune(toLowerRune(r))
				first = false
				continue
			}
			if nextUpper {
				b.WriteRune(toUpperRune(r))
				nextUpper = false
			} else {
				b.WriteRune(r)
			}
		}
		return b.String()
	},
	// snakeCase converts to lower_snake_case.
	"snakeCase": func(s any) string {
		text := toStringAny(s)
		if len(text) >= 2 && (text[0] == '"' || text[0] == '\'' || text[0] == '`') && text[0] == text[len(text)-1] {
			text = text[1 : len(text)-1]
		}
		var b strings.Builder
		for i, r := range text {
			switch r {
			case ' ', '-', '.':
				b.WriteRune('_')
			default:
				if i > 0 && r >= 'A' && r <= 'Z' {
					b.WriteRune('_')
				}
				b.WriteRune(toLowerRune(r))
			}
		}
		return b.String()
	},
	// dasherize converts snake_case to kebab-case. Useful for CSS property
	// names where the lexer doesn't allow hyphens in identifiers.
	"dasherize": func(s any) string {
		return strings.ReplaceAll(toStringAny(s), "_", "-")
	},
	// unquote strips one layer of surrounding "...", '...', or `...` if
	// present. Useful when the source uses string literals but the target
	// doesn't want the quotes in the output (markdown headings, etc.).
	"unquote": func(s any) string {
		text := toStringAny(s)
		if len(text) >= 2 {
			first, last := text[0], text[len(text)-1]
			if (first == '"' || first == '\'' || first == '`') && first == last {
				return text[1 : len(text)-1]
			}
		}
		return text
	},
	// unescape reverses Go string escaping. Capy preserves the source's
	// backslash sequences verbatim through the lexer and re-quotes via
	// strconv.Quote, so a captured "Hello\n" surfaces in templates as
	// `"Hello\\n"`. Use unescape when the TARGET wants the actual escape
	// sequence (e.g. assembler .ascii / .asciz directives, C string
	// literals, JSON-on-the-wire). Wraps in quotes first if missing.
	"unescape": func(s any) string {
		text := toStringAny(s)
		if len(text) < 2 || text[0] != '"' || text[len(text)-1] != '"' {
			text = "\"" + text + "\""
		}
		v, err := strconv.Unquote(text)
		if err != nil {
			return toStringAny(s)
		}
		return v
	},
	// trimSuffix removes a trailing string if present. Useful for joining
	// generators that emit comma-suffixed lines: drop the dangling comma
	// at the end of a list with `{{ .body | trimSuffix ",\n" }}\n`.
	"trimSuffix": func(suffix string, s any) string {
		text := toStringAny(s)
		return strings.TrimSuffix(text, suffix)
	},
	"trimPrefix": func(prefix string, s any) string {
		text := toStringAny(s)
		return strings.TrimPrefix(text, prefix)
	},
	"join": func(sep string, items []any) string {
		parts := make([]string, 0, len(items))
		for _, x := range items {
			parts = append(parts, toStringAny(x))
		}
		return strings.Join(parts, sep)
	},
	// split breaks a string into a list at each occurrence of SEP.
	// Argument order matches strings.Split (string first, separator
	// second) so it reads naturally inline: `range (split .text "\n")`.
	"split": func(s any, sep string) []string {
		return strings.Split(toStringAny(s), sep)
	},
	// nonEmpty filters a string list down to entries that aren't blank
	// after trimming whitespace. Handy when iterating over `read_file`
	// output without producing trailing empty rows.
	"nonEmpty": func(items []string) []string {
		out := make([]string, 0, len(items))
		for _, s := range items {
			if strings.TrimSpace(s) != "" {
				out = append(out, s)
			}
		}
		return out
	},
	// toQuoted wraps a string in JSON-style double quotes (good for Python too).
	"toQuoted": func(s any) string {
		b, _ := json.Marshal(toStringAny(s))
		return string(b)
	},
	// escapeHtml replaces the five characters every HTML emitter
	// has to neutralise — `& < > " '` — with their HTML entities.
	// Ampersand is rewritten FIRST so the entities introduced for
	// the other four aren't re-escaped on a second pass. Use as
	// `${escapeHtml user_value}` inside any `<…>${…}…</…>`
	// template to neutralise XSS-style injection from a free-form
	// capture.
	//
	// The verbose name is deliberate: `${html x}` read as "make this
	// HTML" instead of "escape this FOR HTML", which is the opposite
	// of what the helper does. `escapeHtml` makes the intent obvious
	// at the call site.
	"escapeHtml": func(s any) string {
		r := strings.NewReplacer(
			"&", "&amp;",
			"<", "&lt;",
			">", "&gt;",
			"\"", "&quot;",
			"'", "&#39;",
		)
		return r.Replace(toStringAny(s))
	},
	// decoded delivers the user-intended string from a quoted `string`
	// capture: strips the outer quote (if present) and fully resolves
	// Go-style escape sequences (`\"`, `\n`, `\t`, `\\`, `\xNN`,
	// `\uNNNN`). Use this instead of `unquote` when the captured
	// content might contain escaped quotes or whitespace that need
	// the user-intended form — `p "He said \"hi\""` becomes
	// `He said "hi"`, not `He said \"hi\"`. Existing libraries that
	// want the source-text form unchanged keep using `${text}` /
	// `${unquote text}`.
	"decoded": func(s any) string {
		text := toStringAny(s)
		// A `string` capture's text is the strconv.Quote'd source form
		// (ExprToText quotes the StringLit), so it carries TWO escape
		// layers: the Quote layer + the user's own source escapes. Peel
		// the Quote layer with strconv.Unquote (always valid Go-string
		// syntax, so this never fails for a real string capture), then
		// run the lenient byte scanner for the residual user escapes —
		// the scanner tolerates bare `"` (the §4c bug) which strconv
		// can't. If Unquote fails (e.g. a bare-ident capture, or an
		// already-decoded value), fall straight through to the scanner.
		if unq, err := strconv.Unquote(text); err == nil {
			return decodeEscapes(unq)
		}
		return decodeEscapes(text)
	},
	// asString normalises a capture to exactly ONE valid JSON string,
	// quoting iff it isn't already a string literal. This is the
	// interpolation verb that was missing (missing.md §3): an `any` /
	// `raw` capture can hold either a bare token (`foo`, `42`, `true`)
	// or a quoted string (`"foo"`), and neither `${x}` nor `${toJSON x}`
	// emits correct JSON for both — the former leaves a bare ident
	// unquoted (invalid JSON), the latter re-quotes an already-quoted
	// string (leaking literal `"` into the value). asString resolves the
	// captured text to its user-intended form (peeling one quote layer
	// and decoding escapes, exactly like `decoded`) and then re-encodes
	// it as a single JSON string. So `exec echo foo` and
	// `exec echo "foo"` both interpolate to `"foo"`.
	"asString": func(s any) string {
		text := toStringAny(s)
		var v string
		if unq, err := strconv.Unquote(text); err == nil {
			v = decodeEscapes(unq)
		} else {
			v = decodeEscapes(text)
		}
		b, _ := json.Marshal(v)
		return string(b)
	},
	// toPyLit formats any value as a Python literal.
	"toPyLit": pyLit,
	// toJSON marshals any value to compact JSON (good for config-file output).
	"toJSON": func(v any) string {
		b, _ := json.Marshal(v)
		return string(b)
	},
	// toJSONIndent marshals with indentation.
	"toJSONIndent": func(v any) string {
		b, _ := json.MarshalIndent(v, "", "  ")
		return string(b)
	},
	// add returns a + b. Both args coerced to int64. Useful for running
	// totals inside `range` blocks (e.g. summing pages in a reading log).
	"add": func(a, b any) int64 { return toInt(a) + toInt(b) },
	"sub": func(a, b any) int64 { return toInt(a) - toInt(b) },
	"mul": func(a, b any) int64 { return toInt(a) * toInt(b) },
	// div returns the integer quotient a / b (truncated toward zero),
	// or 0 when b == 0 so a template never panics on bad input. Pairs
	// with `mod` for the arithmetic that running-total bookkeeping (add)
	// alone can't express — most importantly alignment rounding in code
	// generators (see `align`).
	"div": func(a, b any) int64 {
		d := toInt(b)
		if d == 0 {
			return 0
		}
		return toInt(a) / d
	},
	// mod returns the remainder a % b (Go's truncated remainder), or 0
	// when b == 0. With `div` this completes the four integer ops a
	// back-end needs: `mod off a` tells you how far past an a-byte
	// boundary an offset sits.
	"mod": func(a, b any) int64 {
		d := toInt(b)
		if d == 0 {
			return 0
		}
		return toInt(a) % d
	},
	// align rounds n UP to the next multiple of a (the classic
	// `(n + a - 1) / a * a`), or returns n unchanged when a <= 0. This is
	// the single op that struct-field layout and stack-frame sizing need
	// and that `add`/`sub`/`mul` couldn't reach: `align 9 4` → 12,
	// `align 13 8` → 16. Having it as a named helper means a Capy library
	// can emit ABI-correct offsets without open-coding the formula.
	"align": func(n, a any) int64 {
		al := toInt(a)
		if al <= 0 {
			return toInt(n)
		}
		v := toInt(n)
		return (v + al - 1) / al * al
	},
	// percent returns (numerator / denominator) * 100 as an int, clamped
	// to [0, 100]. Handy for progress bars in HTML output.
	"percent": func(n, d any) int64 {
		den := toInt(d)
		if den == 0 {
			return 0
		}
		p := toInt(n) * 100 / den
		if p < 0 {
			return 0
		}
		if p > 100 {
			return 100
		}
		return p
	},
	// stars renders an integer as that many filled-star characters plus
	// the remainder out of five as outlined stars. Useful for rating
	// displays in non-programmer DSLs (reading logs, restaurant lists).
	"stars": func(n any) string {
		k := int(toInt(n))
		if k < 0 {
			k = 0
		}
		if k > 5 {
			k = 5
		}
		return strings.Repeat("★", k) + strings.Repeat("☆", 5-k)
	},
}

// ApplyHelper invokes a template helper by name from outside the
// template renderer (e.g. the inner-DSL expression evaluator, so
// libraries can write `set context.total (add context.total n)`
// and `${add x y}` interpolations get the same semantics as if
// they were called in template position). Returns ok=false if
// the name isn't a known helper.
func ApplyHelper(name string, args []any) (result any, ok bool, err error) {
	fn, found := funcs[name]
	if !found {
		return nil, false, nil
	}
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("helper %q: %v", name, r)
		}
	}()
	switch f := fn.(type) {
	case func(any) string:
		if len(args) != 1 {
			return nil, true, fmt.Errorf("helper %q: expected 1 arg, got %d", name, len(args))
		}
		return f(args[0]), true, nil
	case func(string) string:
		if len(args) != 1 {
			return nil, true, fmt.Errorf("helper %q: expected 1 arg, got %d", name, len(args))
		}
		return f(toStringAny(args[0])), true, nil
	case func(a, b any) int64:
		if len(args) != 2 {
			return nil, true, fmt.Errorf("helper %q: expected 2 args, got %d", name, len(args))
		}
		return f(args[0], args[1]), true, nil
	case func(int, string) string:
		if len(args) != 2 {
			return nil, true, fmt.Errorf("helper %q: expected 2 args, got %d", name, len(args))
		}
		return f(int(toInt(args[0])), toStringAny(args[1])), true, nil
	case func(string, any) string:
		if len(args) != 2 {
			return nil, true, fmt.Errorf("helper %q: expected 2 args, got %d", name, len(args))
		}
		return f(toStringAny(args[0]), args[1]), true, nil
	case func(string, []any) string:
		if len(args) != 2 {
			return nil, true, fmt.Errorf("helper %q: expected 2 args, got %d", name, len(args))
		}
		items, _ := args[1].([]any)
		return f(toStringAny(args[0]), items), true, nil
	case func(any, string) []string:
		if len(args) != 2 {
			return nil, true, fmt.Errorf("helper %q: expected 2 args, got %d", name, len(args))
		}
		return f(args[0], toStringAny(args[1])), true, nil
	case func([]string) []string:
		if len(args) != 1 {
			return nil, true, fmt.Errorf("helper %q: expected 1 arg, got %d", name, len(args))
		}
		ss, _ := args[0].([]string)
		return f(ss), true, nil
	}
	return nil, false, nil
}

// toInt coerces common numeric types to int64. Tolerates strings holding
// digit sequences so `{{ add .x 1 }}` works even when .x came from a
// string-typed capture.
func toInt(v any) int64 {
	switch x := v.(type) {
	case int:
		return int64(x)
	case int64:
		return x
	case float64:
		return int64(x)
	case string:
		n, _ := strconv.ParseInt(strings.TrimSpace(x), 10, 64)
		return n
	}
	return 0
}

func pyLit(v any) string {
	switch x := v.(type) {
	case nil:
		return "None"
	case bool:
		if x {
			return "True"
		}
		return "False"
	case string:
		b, _ := json.Marshal(x)
		return string(b)
	case []any:
		parts := make([]string, 0, len(x))
		for _, it := range x {
			parts = append(parts, pyLit(it))
		}
		return "[" + strings.Join(parts, ", ") + "]"
	case map[string]any:
		// Sort keys: Go map iteration is randomised, so without this the same
		// input produced a different Python literal on each run — i.e. a
		// transpiler with non-reproducible output.
		keys := make([]string, 0, len(x))
		for k := range x {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		parts := make([]string, 0, len(keys))
		for _, k := range keys {
			parts = append(parts, fmt.Sprintf("%q: %s", k, pyLit(x[k])))
		}
		return "{" + strings.Join(parts, ", ") + "}"
	}
	return fmt.Sprintf("%v", v)
}

func toUpperRune(r rune) rune {
	if r >= 'a' && r <= 'z' {
		return r - 32
	}
	return r
}
func toLowerRune(r rune) rune {
	if r >= 'A' && r <= 'Z' {
		return r + 32
	}
	return r
}

func toStringAny(v any) string {
	switch x := v.(type) {
	case nil:
		return ""
	case string:
		return x
	}
	return fmt.Sprintf("%v", v)
}

// decodeEscapes resolves Go-style escape sequences in a string WITHOUT
// using strconv.Unquote — so it never chokes on a bare (unescaped) `"`
// the way `"` + text + `"` + Unquote does. That bare-quote case is
// ubiquitous in HTML (`class="…"`) and was the reason `${decoded src}`
// left literal `\n` behind when the captured string contained a quote.
//
// One layer of matched surrounding quotes (", ', or `) is stripped
// first if present. Then the body is scanned byte-by-byte:
//
//	\n \t \r \\ \" \' \`  → the obvious single char
//	\xNN                  → one byte
//	\uNNNN / \UNNNNNNNN   → the UTF-8 encoding of the rune
//	\<other>              → the literal next char (lenient)
//
// Anything that isn't a recognised escape is copied verbatim,
// including stray quotes.
func decodeEscapes(s string) string {
	if len(s) >= 2 {
		if q := s[0]; (q == '"' || q == '\'' || q == '`') && s[len(s)-1] == q {
			s = s[1 : len(s)-1]
		}
	}
	if !strings.ContainsRune(s, '\\') {
		return s
	}
	var b strings.Builder
	b.Grow(len(s))
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c != '\\' || i+1 >= len(s) {
			b.WriteByte(c)
			continue
		}
		i++
		switch e := s[i]; e {
		case 'n':
			b.WriteByte('\n')
		case 't':
			b.WriteByte('\t')
		case 'r':
			b.WriteByte('\r')
		case '\\', '"', '\'', '`':
			b.WriteByte(e)
		case 'x':
			if i+2 < len(s) {
				if v, err := strconv.ParseUint(s[i+1:i+3], 16, 8); err == nil {
					b.WriteByte(byte(v))
					i += 2
					continue
				}
			}
			b.WriteByte(e)
		case 'u', 'U':
			n := 4
			if e == 'U' {
				n = 8
			}
			if i+n < len(s) {
				if v, err := strconv.ParseUint(s[i+1:i+1+n], 16, 32); err == nil {
					b.WriteRune(rune(v))
					i += n
					continue
				}
			}
			b.WriteByte(e)
		default:
			// Unknown escape — keep the next char literally (lenient).
			b.WriteByte(e)
		}
	}
	return b.String()
}
