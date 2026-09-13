//! Port of `infra/helpers.go`.
//!
//! Code-generator helpers — small, target-language-aware string-manipulation
//! functions called from the inner-DSL renderer. Dispatched by name through
//! [`apply_helper`].
//!
//! Go stores these in a `map[string]any` of closures and dispatches by type
//! switching on the closure's signature. That type switch is what enforces
//! arity, and it is reproduced here literally — including the two places where
//! Go uses a type *assertion* on a slice argument (`join` wants `[]any`,
//! `nonEmpty` wants `[]string`) and silently yields an empty result when the
//! caller passes the other slice kind.

use crate::domain::val::Val;
use crate::gofmt;
use crate::gojson;

/// The signature groups from Go's `ApplyHelper` type switch.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Sig {
    /// `func(any) string`
    AnyToStr,
    /// `func(string) string`
    StrToStr,
    /// `func(a, b any) int64`
    TwoAnyToInt,
    /// `func(int, string) string`
    IntStrToStr,
    /// `func(string, any) string`
    StrAnyToStr,
    /// `func(string, []any) string`
    StrListAnyToStr,
    /// `func(any, string) []string`
    AnyStrToStrList,
    /// `func([]string) []string`
    StrSliceToStrSlice,
}

/// The `funcs` map's keys and their signature groups, in the order they appear
/// in `infra/helpers.go`.
const FUNCS: &[(&str, Sig)] = &[
    ("indent", Sig::IntStrToStr),
    ("lower", Sig::StrToStr),
    ("upper", Sig::StrToStr),
    ("pascalCase", Sig::AnyToStr),
    ("camelCase", Sig::AnyToStr),
    ("snakeCase", Sig::AnyToStr),
    ("dasherize", Sig::AnyToStr),
    ("unquote", Sig::AnyToStr),
    ("unescape", Sig::AnyToStr),
    ("trimSuffix", Sig::StrAnyToStr),
    ("trimPrefix", Sig::StrAnyToStr),
    ("join", Sig::StrListAnyToStr),
    ("split", Sig::AnyStrToStrList),
    ("nonEmpty", Sig::StrSliceToStrSlice),
    ("toQuoted", Sig::AnyToStr),
    ("escapeHtml", Sig::AnyToStr),
    ("decoded", Sig::AnyToStr),
    ("asString", Sig::AnyToStr),
    ("toPyLit", Sig::AnyToStr),
    ("toJSON", Sig::AnyToStr),
    ("toJSONIndent", Sig::AnyToStr),
    ("add", Sig::TwoAnyToInt),
    ("sub", Sig::TwoAnyToInt),
    ("mul", Sig::TwoAnyToInt),
    ("div", Sig::TwoAnyToInt),
    ("mod", Sig::TwoAnyToInt),
    ("align", Sig::TwoAnyToInt),
    ("percent", Sig::TwoAnyToInt),
    ("stars", Sig::AnyToStr),
];

/// Every helper name, for `did you mean …?` hints and docs generation.
pub fn helper_names() -> Vec<String> {
    let mut v: Vec<String> = FUNCS.iter().map(|(n, _)| n.to_string()).collect();
    v.sort();
    v
}

pub fn is_helper(name: &str) -> bool {
    FUNCS.iter().any(|(n, _)| *n == name)
}

/// Port of `ApplyHelper`.
///
/// Invokes a template helper by name from outside the template renderer (e.g.
/// the inner-DSL expression evaluator, so libraries can write
/// `set context.total (add context.total n)` and `${add x y}` interpolations get
/// the same semantics as if they were called in template position).
///
/// Mirrors Go's `(result, ok, err)` triple: `ok == false` means the name is not
/// a known helper.
pub fn apply_helper(name: &str, args: &[Val]) -> (Option<Val>, bool, Option<String>) {
    let sig = match FUNCS.iter().find(|(n, _)| *n == name) {
        None => return (None, false, None),
        Some((_, s)) => *s,
    };

    // Arity checks, with Go's exact wording ("1 arg" vs "2 args").
    let want = match sig {
        Sig::AnyToStr | Sig::StrToStr | Sig::StrSliceToStrSlice => 1,
        _ => 2,
    };
    if args.len() != want {
        let unit = if want == 1 { "arg" } else { "args" };
        return (
            None,
            true,
            Some(format!(
                "helper {}: expected {} {}, got {}",
                gofmt::quote(name),
                want,
                unit,
                args.len()
            )),
        );
    }

    let result: Val = match sig {
        Sig::AnyToStr => Val::Str(call_any_to_str(name, &args[0])),
        Sig::StrToStr => Val::Str(call_str_to_str(name, &to_string_any(&args[0]))),
        Sig::TwoAnyToInt => Val::Int(call_two_any_to_int(name, &args[0], &args[1])),
        Sig::IntStrToStr => {
            // Go: f(int(toInt(args[0])), toStringAny(args[1]))
            let n = to_int(&args[0]);
            if n < 0 {
                // strings.Repeat panics on a negative count; ApplyHelper's
                // deferred recover turns the panic into this exact message.
                // Go's `defer recover()` sets only `err`; the named `ok`
                // return keeps its zero value, so a panicking helper reports
                // ok=false — NOT true as the arity errors do.
                return (
                    None,
                    false,
                    Some(format!(
                        "helper {}: strings: negative Repeat count",
                        gofmt::quote(name)
                    )),
                );
            }
            Val::Str(indent(n, &to_string_any(&args[1])))
        }
        Sig::StrAnyToStr => {
            let a = to_string_any(&args[0]);
            match name {
                "trimSuffix" => Val::Str(trim_suffix(&a, &args[1])),
                "trimPrefix" => Val::Str(trim_prefix(&a, &args[1])),
                _ => unreachable!(),
            }
        }
        Sig::StrListAnyToStr => {
            // Go: items, _ := args[1].([]any) — a FAILED assertion yields nil,
            // so passing a []string here joins nothing and returns "".
            let sep = to_string_any(&args[0]);
            let items: Vec<Val> = match &args[1] {
                Val::List(v) => v.clone(),
                _ => Vec::new(),
            };
            Val::Str(join(&sep, &items))
        }
        Sig::AnyStrToStrList => {
            // split(s any, sep string) []string
            let sep = to_string_any(&args[1]);
            Val::StrList(split(&args[0], &sep))
        }
        Sig::StrSliceToStrSlice => {
            // Go: ss, _ := args[0].([]string) — a []any argument fails the
            // assertion and yields an empty result.
            let ss: Vec<String> = match &args[0] {
                Val::StrList(v) => v.clone(),
                _ => Vec::new(),
            };
            Val::StrList(non_empty(&ss))
        }
    };
    (Some(result), true, None)
}

fn call_str_to_str(name: &str, s: &str) -> String {
    match name {
        "lower" => to_lower_go(s),
        "upper" => to_upper_go(s),
        _ => unreachable!(),
    }
}

fn call_two_any_to_int(name: &str, a: &Val, b: &Val) -> i64 {
    match name {
        "add" => to_int(a).wrapping_add(to_int(b)),
        "sub" => to_int(a).wrapping_sub(to_int(b)),
        "mul" => to_int(a).wrapping_mul(to_int(b)),
        "div" => {
            let d = to_int(b);
            if d == 0 {
                0
            } else {
                to_int(a).wrapping_div(d)
            }
        }
        "mod" => {
            let d = to_int(b);
            if d == 0 {
                0
            } else {
                to_int(a).wrapping_rem(d)
            }
        }
        "align" => {
            let al = to_int(b);
            if al <= 0 {
                return to_int(a);
            }
            let v = to_int(a);
            (v + al - 1) / al * al
        }
        "percent" => {
            let den = to_int(b);
            if den == 0 {
                return 0;
            }
            // Go writes this as an if-chain; clamp is the same arithmetic.
            (to_int(a) * 100 / den).clamp(0, 100)
        }
        _ => unreachable!(),
    }
}

fn call_any_to_str(name: &str, v: &Val) -> String {
    match name {
        "pascalCase" => pascal_case(v),
        "camelCase" => camel_case(v),
        "snakeCase" => snake_case(v),
        "dasherize" => dasherize(v),
        "unquote" => unquote_helper(v),
        "unescape" => unescape(v),
        "toQuoted" => to_quoted(v),
        "escapeHtml" => escape_html(v),
        "decoded" => decoded(v),
        "asString" => as_string(v),
        "toPyLit" => py_lit(v),
        "toJSON" => gojson::marshal(v),
        "toJSONIndent" => gojson::marshal_indent(v, "", "  "),
        "stars" => stars(v),
        _ => unreachable!(),
    }
}

// --- individual helpers, in source order -----------------------------------

/// `indent N`: indent every line of s by N spaces. Useful for `body`.
fn indent(n: i64, s: &str) -> String {
    // Callers reject a negative count before reaching here (see apply_helper),
    // matching Go's panic-to-error path.
    let pad = " ".repeat(n.max(0) as usize);
    let lines: Vec<String> = s
        .split('\n')
        .map(|l| if l.is_empty() { l.to_string() } else { format!("{}{}", pad, l) })
        .collect();
    lines.join("\n")
}

/// Go's `strings.ToLower` uses SIMPLE per-rune case mapping. Rust's
/// `str::to_lowercase` uses FULL mapping, which differs for a handful of
/// characters (`İ` → `i̇` rather than `i`). Where the full mapping would expand
/// to more than one char, Go leaves the rune unchanged, so we do too.
fn to_lower_go(s: &str) -> String {
    s.chars()
        .map(|c| {
            // U+0130 is the only character with an unconditional MULTI-char
            // lowercase mapping in SpecialCasing.txt; Go's simple mapping
            // sends it to plain 'i'.
            if c == '\u{0130}' {
                return 'i';
            }
            let mut it = c.to_lowercase();
            let first = it.next().unwrap_or(c);
            if it.next().is_some() {
                c
            } else {
                first
            }
        })
        .collect()
}

/// Counterpart of [`to_lower_go`] — `ß` stays `ß` under Go's simple mapping
/// rather than becoming `SS`.
fn to_upper_go(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mut it = c.to_uppercase();
            let first = it.next().unwrap_or(c);
            if it.next().is_some() {
                c
            } else {
                first
            }
        })
        .collect()
}

/// Port of the ASCII-only `toUpperRune`.
fn to_upper_rune(r: char) -> char {
    if r.is_ascii_lowercase() {
        ((r as u8) - 32) as char
    } else {
        r
    }
}

/// Port of the ASCII-only `toLowerRune`.
fn to_lower_rune(r: char) -> char {
    if r.is_ascii_uppercase() {
        ((r as u8) + 32) as char
    } else {
        r
    }
}

/// Strips one layer of matched surrounding quotes, as the case helpers do.
/// Mirrors Go's byte-indexed check (`len(text) >= 2 && text[0] == text[len-1]`).
fn strip_matched_quotes(text: &str) -> &str {
    let b = text.as_bytes();
    if b.len() >= 2 {
        let first = b[0];
        if (first == b'"' || first == b'\'' || first == b'`') && first == b[b.len() - 1] {
            return &text[1..text.len() - 1];
        }
    }
    text
}

/// Converts a human-readable string ("Habit Tracker", "habit tracker",
/// "habit-tracker", "habit_tracker") into a PascalCase identifier
/// ("HabitTracker").
fn pascal_case(v: &Val) -> String {
    let text = to_string_any(v);
    let text = strip_matched_quotes(&text);
    let mut b = String::new();
    let mut next_upper = true;
    for r in text.chars() {
        if r == ' ' || r == '_' || r == '-' || r == '.' {
            next_upper = true;
            continue;
        }
        if next_upper {
            b.push(to_upper_rune(r));
            next_upper = false;
        } else {
            b.push(r);
        }
    }
    b
}

/// `camelCase` is `pascalCase` with the first char lowered.
fn camel_case(v: &Val) -> String {
    let text = to_string_any(v);
    let text = strip_matched_quotes(&text);
    let mut b = String::new();
    let mut next_upper = false;
    let mut first = true;
    for r in text.chars() {
        if r == ' ' || r == '_' || r == '-' || r == '.' {
            next_upper = true;
            continue;
        }
        if first {
            b.push(to_lower_rune(r));
            first = false;
            continue;
        }
        if next_upper {
            b.push(to_upper_rune(r));
            next_upper = false;
        } else {
            b.push(r);
        }
    }
    b
}

/// Converts to `lower_snake_case`.
fn snake_case(v: &Val) -> String {
    let text = to_string_any(v);
    let text = strip_matched_quotes(&text);
    let mut b = String::new();
    // Go iterates `for i, r := range text` where `i` is the BYTE offset; the
    // `i > 0` guard therefore just means "not the first rune".
    for (i, r) in text.char_indices() {
        match r {
            ' ' | '-' | '.' => b.push('_'),
            _ => {
                if i > 0 && r.is_ascii_uppercase() {
                    b.push('_');
                }
                b.push(to_lower_rune(r));
            }
        }
    }
    b
}

/// Converts `snake_case` to `kebab-case`. Useful for CSS property names where
/// the lexer doesn't allow hyphens in identifiers.
fn dasherize(v: &Val) -> String {
    to_string_any(v).replace('_', "-")
}

/// Strips one layer of surrounding `"…"`, `'…'`, or `` `…` `` if present.
fn unquote_helper(v: &Val) -> String {
    let text = to_string_any(v);
    strip_matched_quotes(&text).to_string()
}

/// Reverses Go string escaping. Capy preserves the source's backslash sequences
/// verbatim through the lexer and re-quotes via `strconv.Quote`, so a captured
/// `"Hello\n"` surfaces in templates as `"Hello\\n"`. Wraps in quotes first if
/// missing.
fn unescape(v: &Val) -> String {
    let original = to_string_any(v);
    let b = original.as_bytes();
    let text = if b.len() < 2 || b[0] != b'"' || b[b.len() - 1] != b'"' {
        format!("\"{}\"", original)
    } else {
        original.clone()
    };
    match gofmt::unquote(&text) {
        Ok(s) => s,
        Err(_) => original,
    }
}

/// Removes a trailing string if present.
fn trim_suffix(suffix: &str, v: &Val) -> String {
    let text = to_string_any(v);
    match text.strip_suffix(suffix) {
        // Go's TrimSuffix is a no-op for an empty suffix.
        Some(t) if !suffix.is_empty() => t.to_string(),
        _ => text,
    }
}

fn trim_prefix(prefix: &str, v: &Val) -> String {
    let text = to_string_any(v);
    match text.strip_prefix(prefix) {
        Some(t) if !prefix.is_empty() => t.to_string(),
        _ => text,
    }
}

fn join(sep: &str, items: &[Val]) -> String {
    let parts: Vec<String> = items.iter().map(to_string_any).collect();
    parts.join(sep)
}

/// Breaks a string into a list at each occurrence of SEP. Argument order
/// matches `strings.Split` (string first, separator second).
fn split(v: &Val, sep: &str) -> Vec<String> {
    let s = to_string_any(v);
    if sep.is_empty() {
        // strings.Split with an empty separator splits after each UTF-8 rune.
        return s.chars().map(|c| c.to_string()).collect();
    }
    s.split(sep).map(|x| x.to_string()).collect()
}

/// Filters a string list down to entries that aren't blank after trimming
/// whitespace.
fn non_empty(items: &[String]) -> Vec<String> {
    items.iter().filter(|s| !s.trim().is_empty()).cloned().collect()
}

/// Wraps a string in JSON-style double quotes (good for Python too).
fn to_quoted(v: &Val) -> String {
    gojson::marshal_string(&to_string_any(v))
}

/// Replaces the five characters every HTML emitter has to neutralise —
/// `& < > " '` — with their HTML entities. Ampersand is rewritten FIRST so the
/// entities introduced for the other four aren't re-escaped on a second pass.
fn escape_html(v: &Val) -> String {
    // Go uses strings.NewReplacer, which scans once and never revisits
    // replacement output — so a single pass in one sweep is equivalent.
    let s = to_string_any(v);
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Delivers the user-intended string from a quoted `string` capture: strips the
/// outer quote (if present) and fully resolves Go-style escape sequences.
fn decoded(v: &Val) -> String {
    let text = to_string_any(v);
    // A `string` capture's text is the strconv.Quote'd source form, so it
    // carries TWO escape layers: the Quote layer + the user's own source
    // escapes. Peel the Quote layer first, then run the lenient byte scanner
    // for the residual user escapes — the scanner tolerates a bare `"` which
    // strconv cannot. If Unquote fails, fall straight through to the scanner.
    match gofmt::unquote(&text) {
        Ok(unq) => decode_escapes(&unq),
        Err(_) => decode_escapes(&text),
    }
}

/// Normalises a capture to exactly ONE valid JSON string, quoting iff it isn't
/// already a string literal. So `exec echo foo` and `exec echo "foo"` both
/// interpolate to `"foo"`.
fn as_string(v: &Val) -> String {
    let text = to_string_any(v);
    let s = match gofmt::unquote(&text) {
        Ok(unq) => decode_escapes(&unq),
        Err(_) => decode_escapes(&text),
    };
    gojson::marshal_string(&s)
}

/// Renders an integer as that many filled-star characters plus the remainder
/// out of five as outlined stars.
fn stars(v: &Val) -> String {
    // Go writes this as two ifs; clamp is the same arithmetic.
    let k = to_int(v).clamp(0, 5);
    "★".repeat(k as usize) + &"☆".repeat((5 - k) as usize)
}

/// Port of `toInt` — coerces common numeric types to i64. Tolerates strings
/// holding digit sequences.
fn to_int(v: &Val) -> i64 {
    match v {
        Val::Int(i) => *i,
        Val::Float(f) => *f as i64,
        Val::Str(s) => parse_int_go(s.trim()),
        _ => 0,
    }
}

/// Go's `strconv.ParseInt(s, 10, 64)` ignoring the error — a failed parse
/// yields 0, and a range error yields the clamped value Go returns.
fn parse_int_go(s: &str) -> i64 {
    match s.parse::<i64>() {
        Ok(v) => v,
        Err(e) => {
            // Go returns MaxInt64/MinInt64 on range errors, 0 on syntax errors.
            if *e.kind() == std::num::IntErrorKind::PosOverflow {
                i64::MAX
            } else if *e.kind() == std::num::IntErrorKind::NegOverflow {
                i64::MIN
            } else {
                0
            }
        }
    }
}

/// Port of `pyLit` — formats any value as a Python literal.
fn py_lit(v: &Val) -> String {
    match v {
        Val::Null => "None".to_string(),
        Val::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Val::Str(s) => gojson::marshal_string(s),
        Val::List(items) => {
            let parts: Vec<String> = items.iter().map(py_lit).collect();
            format!("[{}]", parts.join(", "))
        }
        Val::Obj(m) => {
            // NOTE: the Go original iterates the map WITHOUT sorting, so its key
            // order is randomised per run for multi-key maps. BTreeMap makes
            // this deterministic (sorted) — strictly better, and the only
            // intentional behavioural divergence in this file.
            let parts: Vec<String> =
                m.iter().map(|(k, val)| format!("{}: {}", gofmt::quote(k), py_lit(val))).collect();
            format!("{{{}}}", parts.join(", "))
        }
        // Go's pyLit has no []string case, so it falls through to %v.
        other => other.format_v(),
    }
}

/// Port of `toStringAny` — only nil and string are special-cased; everything
/// else goes through `%v`.
fn to_string_any(v: &Val) -> String {
    match v {
        Val::Null => String::new(),
        Val::Str(s) => s.clone(),
        other => other.format_v(),
    }
}

/// Port of `decodeEscapes`.
///
/// Resolves Go-style escape sequences WITHOUT using `strconv.Unquote` — so it
/// never chokes on a bare (unescaped) `"` the way wrapping in quotes and
/// unquoting does. That bare-quote case is ubiquitous in HTML (`class="…"`).
///
/// One layer of matched surrounding quotes is stripped first if present. Then
/// the body is scanned byte-by-byte:
///
/// ```text
/// \n \t \r \\ \" \' \`  → the obvious single char
/// \xNN                  → one byte
/// \uNNNN / \UNNNNNNNN   → the UTF-8 encoding of the rune
/// \<other>              → the literal next char (lenient)
/// ```
///
/// Anything that isn't a recognised escape is copied verbatim, including stray
/// quotes.
pub fn decode_escapes(s: &str) -> String {
    let s = strip_matched_quotes(s);
    if !s.contains('\\') {
        return s.to_string();
    }
    let b = s.as_bytes();
    // Go writes raw BYTES (`\xNN` can emit an invalid UTF-8 fragment), so build
    // a byte buffer and convert at the end exactly as Go's string(bytes) does.
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c != b'\\' || i + 1 >= b.len() {
            out.push(c);
            i += 1;
            continue;
        }
        i += 1;
        let e = b[i];
        match e {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'\\' | b'"' | b'\'' | b'`' => out.push(e),
            b'x' => {
                if i + 2 < b.len() {
                    if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                        out.push(v);
                        i += 3;
                        continue;
                    }
                }
                out.push(e);
            }
            b'u' | b'U' => {
                let n = if e == b'u' { 4 } else { 8 };
                if i + n < b.len() {
                    if let Ok(v) = u32::from_str_radix(&s[i + 1..i + 1 + n], 16) {
                        // Go's WriteRune emits U+FFFD for an invalid rune.
                        let ch = char::from_u32(v).unwrap_or('\u{FFFD}');
                        let mut buf = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        i += n + 1;
                        continue;
                    }
                }
                out.push(e);
            }
            // Unknown escape — keep the next char literally (lenient).
            _ => out.push(e),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
