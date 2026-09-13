//! Port of the parts of Go's `encoding/json` the helpers depend on.
//!
//! Two behaviours here are easy to get wrong and both are observable in golden
//! output:
//!
//!   * Go's `json.Marshal` HTML-escapes by default — `<`, `>` and `&` become
//!     `\u003c`, `\u003e`, `\u0026`. Most Rust JSON encoders do not.
//!   * Go encodes JSON floats "as if by ES6 number to string conversion",
//!     which is a *different* rule from `%v` / `FormatFloat(…, 'g', …)`.
//!   * Go marshals maps with keys sorted.

use crate::domain::val::Val;

const HEX: &[u8] = b"0123456789abcdef";

/// Port of Go's `encodeState.string` with `escapeHTML = true` (the
/// `json.Marshal` default).
pub fn marshal_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Go escapes <, >, and & because they can lead to security holes
            // when user-controlled strings are rendered into JSON and served to
            // some browsers.
            '<' | '>' | '&' => {
                let b = c as u32;
                out.push_str("\\u00");
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0xF) as usize] as char);
            }
            // All other bytes below 0x20.
            c if (c as u32) < 0x20 => {
                let b = c as u32;
                out.push_str("\\u00");
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0xF) as usize] as char);
            }
            // Go escapes U+2028 / U+2029, which are valid JSON but break
            // JavaScript parsers.
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Port of Go's JSON float encoding (`encoding/json`'s `floatEncoder`).
///
/// Deliberately different from [`crate::gofmt::format_float_g`]: JSON uses 'f'
/// format unless the magnitude is `< 1e-6` or `>= 1e21`, then 'e' — and then
/// strips a leading zero from a two-digit negative exponent (`e-09` → `e-9`).
pub fn marshal_float(f: f64) -> String {
    if f.is_nan() || f.is_infinite() {
        // Go returns an UnsupportedValueError; the helpers ignore marshal
        // errors and yield the empty string for the whole value.
        return String::new();
    }
    let abs = f.abs();
    let use_e = abs != 0.0 && !(1e-6..1e21).contains(&abs);
    let mut b = if use_e {
        // Shortest round-trip mantissa/exponent, Go's 'e' with precision -1.
        let s = format!("{:e}", f);
        let (mant, exp_s) = s.split_once('e').unwrap();
        let exp: i32 = exp_s.parse().unwrap();
        let sign = if exp < 0 { '-' } else { '+' };
        let ae = exp.abs();
        // Go's AppendFloat pads the exponent to at least two digits.
        if ae < 10 {
            format!("{}e{}0{}", mant, sign, ae)
        } else {
            format!("{}e{}{}", mant, sign, ae)
        }
    } else {
        format_float_f_shortest(f)
    };
    if use_e {
        // Clean up e-09 to e-9.
        let bytes = b.as_bytes();
        let n = bytes.len();
        if n >= 4 && bytes[n - 4] == b'e' && bytes[n - 3] == b'-' && bytes[n - 2] == b'0' {
            let last = bytes[n - 1] as char;
            b.truncate(n - 2);
            b.push(last);
        }
    }
    b
}

/// Go's `strconv.AppendFloat(f, 'f', -1, 64)` — shortest round-trip digits in
/// positional notation, never exponent form.
fn format_float_f_shortest(f: f64) -> String {
    let s = format!("{:e}", f);
    let (mant, exp_s) = s.split_once('e').unwrap();
    let exp: i32 = exp_s.parse().unwrap();
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_end_matches('0');
    let (digits, exp) = if digits.is_empty() { ("0", 0) } else { (digits, exp) };
    let nd = digits.len() as i32;
    let dp = exp + 1;
    let mut out = String::new();
    if neg {
        out.push('-');
    }
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
    out
}

/// Port of `json.Marshal` over the runtime value model. Maps emit with keys
/// sorted, which `BTreeMap` gives for free.
pub fn marshal(v: &Val) -> String {
    let mut out = String::new();
    write_value(v, &mut out);
    out
}

fn write_value(v: &Val, out: &mut String) {
    match v {
        Val::Null => out.push_str("null"),
        Val::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Val::Str(s) => out.push_str(&marshal_string(s)),
        Val::Int(i) => out.push_str(&i.to_string()),
        Val::Float(f) => out.push_str(&marshal_float(*f)),
        Val::List(items) => {
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(it, out);
            }
            out.push(']');
        }
        Val::StrList(items) => {
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&marshal_string(it));
            }
            out.push(']');
        }
        Val::Obj(m) => {
            out.push('{');
            for (i, (k, val)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&marshal_string(k));
                out.push(':');
                write_value(val, out);
            }
            out.push('}');
        }
    }
}

/// Port of `json.MarshalIndent(v, "", "  ")`.
///
/// Go compacts first and then re-indents; the observable result is that empty
/// arrays and objects stay `[]` / `{}` on one line.
pub fn marshal_indent(v: &Val, prefix: &str, indent: &str) -> String {
    let mut out = String::new();
    write_indent_value(v, prefix, indent, 0, &mut out);
    out
}

fn write_indent_value(v: &Val, prefix: &str, indent: &str, depth: usize, out: &mut String) {
    let nl = |out: &mut String, d: usize| {
        out.push('\n');
        out.push_str(prefix);
        for _ in 0..d {
            out.push_str(indent);
        }
    };
    match v {
        Val::List(items) if !items.is_empty() => {
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                nl(out, depth + 1);
                write_indent_value(it, prefix, indent, depth + 1, out);
            }
            nl(out, depth);
            out.push(']');
        }
        Val::StrList(items) if !items.is_empty() => {
            out.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                nl(out, depth + 1);
                out.push_str(&marshal_string(it));
            }
            nl(out, depth);
            out.push(']');
        }
        Val::Obj(m) if !m.is_empty() => {
            out.push('{');
            for (i, (k, val)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                nl(out, depth + 1);
                out.push_str(&marshal_string(k));
                out.push_str(": ");
                write_indent_value(val, prefix, indent, depth + 1, out);
            }
            nl(out, depth);
            out.push('}');
        }
        other => write_value(other, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marshal_string_html_escapes_like_go() {
        // json.Marshal("a<b>c&d") == `"a\u003cb\u003ec\u0026d"`
        assert_eq!(marshal_string("a<b>c&d"), "\"a\\u003cb\\u003ec\\u0026d\"");
        assert_eq!(marshal_string("plain"), "\"plain\"");
        assert_eq!(marshal_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(marshal_string("a\nb"), "\"a\\nb\"");
        assert_eq!(marshal_string("\u{1}"), "\"\\u0001\"");
        // DEL is in Go's safeSet and is emitted literally.
        assert_eq!(marshal_string("\u{7f}"), "\"\u{7f}\"");
        assert_eq!(marshal_string("é"), "\"é\"");
    }

    #[test]
    fn marshal_float_matches_go_json() {
        assert_eq!(marshal_float(1.0), "1");
        assert_eq!(marshal_float(1.5), "1.5");
        assert_eq!(marshal_float(1000000.0), "1000000");
        assert_eq!(marshal_float(1e21), "1e+21");
        assert_eq!(marshal_float(1e-7), "1e-7");
        assert_eq!(marshal_float(0.0), "0");
    }
}
