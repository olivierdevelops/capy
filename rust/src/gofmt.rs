//! Go-compatible formatting primitives.
//!
//! Not a port of a single Go file — these reproduce the pieces of Go's
//! `strconv` / `fmt` that the engine's observable output depends on. Rust's
//! native `{}` / `{:?}` differ from Go's in ways that would break goldens:
//!
//!   * `strconv.FormatFloat(f, 'g', -1, 64)` — used by `toString` in
//!     `orchestrator/features/inner_evaluator.go`.
//!   * `fmt.Sprintf("%v", …)` — the fallthrough in `toString`,
//!     `toStringAny` and `pyLit`.
//!   * `fmt.Sprintf("%q", …)` — used by `pyLit` for object keys.

/// Port of `strconv.FormatFloat(f, 'g', -1, 64)`.
///
/// Go's shortest-'g' picks %e when the decimal exponent is `< -4` or `>= 6`
/// (for shortest precision Go fixes the crossover precision at 6 — see
/// `ftoa.go`), otherwise %f. Rust's `{}` never switches to exponent form and
/// its `{:e}` always does, so neither matches on its own; we take Rust's
/// shortest round-trip digits from `{:e}` and re-apply Go's decision rule.
pub fn format_float_g(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 { "+Inf".to_string() } else { "-Inf".to_string() };
    }

    // Rust's LowerExp yields shortest round-trip digits: "-1.2345e3".
    let s = format!("{:e}", f);
    let (mant, exp_s) = s.split_once('e').expect("LowerExp always emits 'e'");
    let exp: i32 = exp_s.parse().expect("LowerExp exponent is an integer");
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_end_matches('0');
    // Zero collapses to a single "0" digit with exponent 0.
    let (digits, exp) = if digits.is_empty() { ("0", 0) } else { (digits, exp) };
    let nd = digits.len() as i32;

    let mut out = String::new();
    if neg {
        out.push('-');
    }

    // Go: `exp := digs.dp - 1`, use %e if `exp < -4 || exp >= eprec` (eprec=6).
    if !(-4..6).contains(&exp) {
        out.push_str(&digits[..1]);
        if nd > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        if exp < 0 {
            out.push('-');
        } else {
            out.push('+');
        }
        let ae = exp.abs();
        // Go pads the exponent to at least two digits.
        if ae < 10 {
            out.push('0');
        }
        out.push_str(&ae.to_string());
    } else {
        let dp = exp + 1; // decimal point position within `digits`
        if dp <= 0 {
            out.push_str("0.");
            for _ in 0..(-dp) {
                out.push('0');
            }
            out.push_str(digits);
        } else if dp >= nd {
            out.push_str(digits);
            for _ in 0..(dp - nd) {
                out.push('0');
            }
        } else {
            out.push_str(&digits[..dp as usize]);
            out.push('.');
            out.push_str(&digits[dp as usize..]);
        }
    }
    out
}

/// Port of `strconv.Quote` / `fmt.Sprintf("%q", s)`.
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{b}' => out.push_str("\\v"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values produced by Go:
    //   strconv.FormatFloat(v, 'g', -1, 64)
    #[test]
    fn float_g_matches_go() {
        let cases: &[(f64, &str)] = &[
            (0.0, "0"),
            (-0.0, "-0"),
            (1.0, "1"),
            (-1.0, "-1"),
            (1.5, "1.5"),
            (0.1, "0.1"),
            (100000.0, "100000"),
            (1000000.0, "1e+06"),
            (1234567.0, "1.234567e+06"),
            (999999.0, "999999"),
            (0.0001, "0.0001"),
            (0.00001, "1e-05"),
            (1e20, "1e+20"),
            (1e-20, "1e-20"),
            (3.141592653589793, "3.141592653589793"),
            (2.0 / 3.0, "0.6666666666666666"),
            (1e21, "1e+21"),
            (123.456, "123.456"),
            (-0.5, "-0.5"),
        ];
        for (v, want) in cases {
            assert_eq!(&format_float_g(*v), want, "format_float_g({v})");
        }
    }

    #[test]
    fn quote_matches_go() {
        assert_eq!(quote("hi"), "\"hi\"");
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("a\nb"), "\"a\\nb\"");
        assert_eq!(quote("a\\b"), "\"a\\\\b\"");
        assert_eq!(quote("\t"), "\"\\t\"");
    }
}

/// Port of `strconv.QuoteRune` / `fmt.Sprintf("%q", r)` for a single rune.
pub fn quote_rune(c: char) -> String {
    let inner = match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '\u{7}' => "\\a".to_string(),
        '\u{8}' => "\\b".to_string(),
        '\u{b}' => "\\v".to_string(),
        '\u{c}' => "\\f".to_string(),
        c if (c as u32) < 0x20 || (c as u32) == 0x7f => format!("\\x{:02x}", c as u32),
        // Go prints printable non-ASCII runes literally.
        c => c.to_string(),
    };
    format!("'{}'", inner)
}

/// Port of `strconv.Unquote`.
///
/// The unit error type is deliberate: Go's `strconv.Unquote` returns an opaque
/// `ErrSyntax` that every caller in this codebase checks only for presence
/// (`decoded` / `asString` / `unescape` branch on success, never on the reason).
/// A richer error type would be unused surface.
#[allow(clippy::result_unit_err)]
///
/// The *failure* cases are as load-bearing as the successes: `decoded` and
/// `asString` in `infra/helpers.go` branch on whether this returns an error, so
/// being more permissive than Go would silently change their output.
pub fn unquote(s: &str) -> Result<String, ()> {
    let b = s.as_bytes();
    if b.len() < 2 {
        return Err(());
    }
    let quote = b[0];
    if b[b.len() - 1] != quote {
        return Err(());
    }
    let inner = &s[1..s.len() - 1];

    match quote {
        b'`' => {
            // Raw strings admit no escapes, cannot contain a backquote, and
            // carriage returns are discarded from the value.
            if inner.contains('`') {
                return Err(());
            }
            Ok(inner.replace('\r', ""))
        }
        b'\'' => {
            // A rune literal must hold exactly one rune.
            let (v, rest) = unquote_char(inner, b'\'')?;
            if !rest.is_empty() {
                return Err(());
            }
            Ok(v.to_string())
        }
        b'"' => {
            // Go rejects a literal newline inside an interpreted string.
            if inner.contains('\n') {
                return Err(());
            }
            let mut out = String::with_capacity(inner.len());
            let mut rest = inner;
            while !rest.is_empty() {
                let (c, r) = unquote_char(rest, b'"')?;
                out.push(c);
                rest = r;
            }
            Ok(out)
        }
        _ => Err(()),
    }
}

/// Port of `strconv.UnquoteChar` — decodes one character, returning it plus the
/// remainder of the input.
fn unquote_char(s: &str, quote: u8) -> Result<(char, &str), ()> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(());
    }
    // Easy cases first.
    if b[0] == quote && (quote == b'\'' || quote == b'"') {
        return Err(());
    }
    if b[0] == b'\n' {
        return Err(());
    }
    if b[0] != b'\\' {
        let c = s.chars().next().ok_or(())?;
        return Ok((c, &s[c.len_utf8()..]));
    }
    if b.len() < 2 {
        return Err(());
    }
    let e = b[1];
    let tail = &s[2..];
    match e {
        b'a' => Ok(('\u{7}', tail)),
        b'b' => Ok(('\u{8}', tail)),
        b'f' => Ok(('\u{c}', tail)),
        b'n' => Ok(('\n', tail)),
        b'r' => Ok(('\r', tail)),
        b't' => Ok(('\t', tail)),
        b'v' => Ok(('\u{b}', tail)),
        b'\\' => Ok(('\\', tail)),
        // Go: `\'` is valid ONLY inside a rune literal, `\"` only inside a
        // string. Getting this wrong makes Unquote succeed where Go fails.
        b'\'' | b'"' => {
            if e != quote {
                return Err(());
            }
            Ok((e as char, tail))
        }
        b'x' => {
            if tail.len() < 2 {
                return Err(());
            }
            let v = u8::from_str_radix(&tail[..2], 16).map_err(|_| ())?;
            // Go yields a raw BYTE here, not a rune. Callers rebuild a string
            // from bytes; representing it as char would re-encode as UTF-8.
            Ok((v as char, &tail[2..]))
        }
        b'u' | b'U' => {
            let n = if e == b'u' { 4 } else { 8 };
            if tail.len() < n {
                return Err(());
            }
            let hex = &tail[..n];
            if !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(());
            }
            let v = u32::from_str_radix(hex, 16).map_err(|_| ())?;
            // Go rejects surrogate halves and out-of-range values.
            let c = char::from_u32(v).ok_or(())?;
            Ok((c, &tail[n..]))
        }
        b'0'..=b'7' => {
            if s.len() < 4 {
                return Err(());
            }
            let oct = &s[1..4];
            if !oct.bytes().all(|c| (b'0'..=b'7').contains(&c)) {
                return Err(());
            }
            let v = u32::from_str_radix(oct, 8).map_err(|_| ())?;
            if v > 255 {
                return Err(());
            }
            Ok((v as u8 as char, &s[4..]))
        }
        _ => Err(()),
    }
}
