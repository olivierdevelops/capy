//! Port of `orchestrator/features/make_lexer.go`.
//!
//! The lexer is purely lexical: it identifies words, numbers, strings,
//! brackets, punctuation, and indentation. It does NOT classify any word as a
//! keyword — "if", "loop", "end", "true", etc. are all just `Ident`. The
//! library's patterns decide what those words mean. This is the core of
//! "0 default grammar".

use crate::domain::errors::CapyError;
use crate::domain::token::{Token, TokenKind};
use crate::gofmt;

/// The marker set used when no explicit list is supplied. The engine has NO
/// predefined user-script comment syntax — but Capy's own manifest/inner-DSL
/// format uses `#`, so the no-args entry point keeps `#` to lex manifests and
/// inner-DSL run bodies. User scripts go through [`tokenize_with`] and pass
/// `lib.comments` instead.
pub const DEFAULT_COMMENT_MARKERS: &[&str] = &["#"];

/// Accepted as parts of a `Punct` token. Runs of these are greedily consumed to
/// form multi-char operators like `==`, `!=`, `<=`, `>=`, `->`, `|>`.
///
/// `#` and `\` are accepted so verbatim block bodies (code samples, LaTeX
/// source, shell snippets) tokenise cleanly without the library having to
/// declare them as comment markers — Capy's "no default grammar" stance means
/// `#` is only a comment when the library says so via the `comments` directive,
/// not by accident of the lexer.
const PUNCT_CHARS: &str = "=<>!+-*/%&|^~?:,.;@$#\\";

fn is_punct(b: u8) -> bool {
    PUNCT_CHARS.as_bytes().contains(&b)
}

/// Go's `unicode.IsDigit` tests category Nd. Rust exposes no Nd-only predicate;
/// `is_numeric` (category N) is the closest and agrees on every ASCII digit and
/// on every character the engine realistically sees.
fn is_digit(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_digit()
    } else {
        c.is_numeric()
    }
}

/// Port of `tokenize` — uses [`DEFAULT_COMMENT_MARKERS`].
pub fn tokenize(source: &str) -> Result<Vec<Token>, CapyError> {
    let markers: Vec<String> = DEFAULT_COMMENT_MARKERS.iter().map(|s| s.to_string()).collect();
    tokenize_with(source, &markers)
}

/// Port of `tokenizeWith`.
pub fn tokenize_with(source: &str, comment_markers: &[String]) -> Result<Vec<Token>, CapyError> {
    tokenize_impl(source, comment_markers, false)
}

/// PLAN-2026-0001 R27 — like [`tokenize_with`], but retains comments as
/// `TokenKind::Comment` tokens instead of discarding them.
///
/// Separate from `tokenize_with` on purpose: `tokenize` (the manifest and
/// inner-DSL path) delegates to `tokenize_with`, and those parsers would choke
/// on a token kind they have never seen. Only the user-script path opts in, and
/// the outer parser strips these before matching.
pub fn tokenize_with_trivia(
    source: &str,
    comment_markers: &[String],
) -> Result<Vec<Token>, CapyError> {
    tokenize_impl(source, comment_markers, true)
}

fn tokenize_impl(
    source: &str,
    comment_markers: &[String],
    keep_comments: bool,
) -> Result<Vec<Token>, CapyError> {
    let mut toks: Vec<Token> = Vec::new();
    let mut indents: Vec<usize> = vec![0];
    let mut bracket: i64 = 0;

    let lines = merge_backtick_lines(&split_lines(source));
    for (li, raw) in lines.iter().enumerate() {
        let mut line: &str = raw;
        // `start_col` carries the source-absolute column at which `line`'s
        // tokenisation begins. When leading indent is stripped at bracket == 0
        // we'd otherwise lose track of the source column — tokens on a stripped
        // line would report col 1 while tokens on a bracket-mode line would
        // report their raw source column. Keeping `start_col = stripped + 1`
        // makes col consistently source-absolute across both cases, which the
        // verbatim-body and tail captures rely on.
        let mut start_col = 1usize;
        if bracket == 0 {
            let mut indent = 0usize;
            let mut i = 0usize;
            let b = line.as_bytes();
            while i < b.len() {
                match b[i] {
                    b' ' => {
                        indent += 1;
                        i += 1;
                    }
                    b'\t' => {
                        indent += 4;
                        i += 1;
                    }
                    _ => break,
                }
            }
            let rest = line[i..].trim();
            if rest.is_empty() || has_comment_prefix(rest, comment_markers) {
                // R27: a whole-line comment is retained as trivia before the
                // line is skipped. It carries no INDENT/DEDENT, exactly as
                // before — only the token is new.
                if keep_comments && !rest.is_empty() {
                    toks.push(Token {
                        kind: TokenKind::Comment,
                        text: rest.to_string(),
                        line: li + 1,
                        col: i + 1,
                        width: rest.len(),
                    });
                }
                continue;
            }
            // Indentation is tracked as raw column WIDTH, not fixed 4-space
            // levels. Any consistent indentation works — 2 spaces, 8 spaces, a
            // tab, whatever — as long as deeper nesting uses more indent than
            // its parent. `indents` is a stack of widths (base 0). Deeper than
            // the top → one INDENT; shallower → pop DEDENTs until we reach a
            // matching-or-shallower width. This keeps INDENT/DEDENT balanced
            // for any well-nested source.
            let top = *indents.last().unwrap();
            if indent > top {
                indents.push(indent);
                toks.push(Token {
                    kind: TokenKind::Indent,
                    text: String::new(),
                    line: li + 1,
                    col: 0,
                    width: 0,
                });
            } else if indent < top {
                while indents.len() > 1 && indent < *indents.last().unwrap() {
                    indents.pop();
                    toks.push(Token {
                        kind: TokenKind::Dedent,
                        text: String::new(),
                        line: li + 1,
                        col: 0,
                        width: 0,
                    });
                }
                // Partial dedent landing between two known widths: open a fresh
                // level so the line still belongs somewhere, rather than
                // erroring "unindent does not match" (lenient).
                if indent > *indents.last().unwrap() {
                    indents.push(indent);
                    toks.push(Token {
                        kind: TokenKind::Indent,
                        text: String::new(),
                        line: li + 1,
                        col: 0,
                        width: 0,
                    });
                }
            }
            start_col = i + 1;
            line = &line[i..];
        }

        let (new_toks, open_delta) =
            tokenize_line(keep_comments, line, li + 1, comment_markers, start_col)?;
        toks.extend(new_toks);
        bracket += open_delta;
        if bracket < 0 {
            return Err(CapyError::msg(format!("line {}: unmatched closing bracket", li + 1)));
        }
        // Always emit NEWLINE at end of a logical line. Inside brackets the
        // value parsers (parse_obj_lit / parse_list_lit / paren sub-call) skip
        // them; block parsers (for `{...}` style blocks) use them as statement
        // boundaries.
        toks.push(Token {
            kind: TokenKind::Newline,
            text: String::new(),
            line: li + 1,
            col: 0,
            width: 0,
        });
    }
    while indents.len() > 1 {
        indents.pop();
        toks.push(Token {
            kind: TokenKind::Dedent,
            text: String::new(),
            line: 0,
            col: 0,
            width: 0,
        });
    }
    toks.push(Token { kind: TokenKind::Eof, text: String::new(), line: 0, col: 0, width: 0 });
    Ok(toks)
}

fn split_lines(s: &str) -> Vec<String> {
    s.replace("\r\n", "\n").split('\n').map(|x| x.to_string()).collect()
}

/// Port of `mergeBacktickLines`.
///
/// Collapses multi-line backtick literals into single logical lines with
/// embedded `\n` escapes, so the line-based [`tokenize_with`] loop sees one
/// continuous string instead of several "unterminated" ones. Mirrors the
/// behaviour the library-file parser applies to function bodies.
///
/// The merged form is byte-equivalent to writing the original on a single line
/// with `\n` escape sequences in place of real newlines. `${decoded x}`
/// recovers the original newlines at render time.
fn merge_backtick_lines(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_backtick = false;
    for ln in lines {
        if in_backtick {
            cur.push_str("\\n");
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        let b = ln.as_bytes();
        let mut i = 0usize;
        while i < b.len() {
            let c = b[i];
            if c == b'\\' && i + 1 < b.len() && in_backtick {
                cur.push(c as char);
                cur.push(b[i + 1] as char);
                i += 2;
                continue;
            }
            if c == b'`' {
                in_backtick = !in_backtick;
            }
            // Bytes are pushed individually in Go; reassemble via raw bytes so
            // multi-byte sequences survive intact.
            unsafe { cur.as_mut_vec().push(c) };
            i += 1;
        }
        if !in_backtick {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Port of `hasCommentPrefix`. Empty markers list → never a comment.
fn has_comment_prefix(s: &str, markers: &[String]) -> bool {
    markers.iter().any(|m| !m.is_empty() && s.starts_with(m.as_str()))
}

/// Port of `tokenizeLine`. Returns the tokens plus the net bracket delta.
fn tokenize_line(
    keep_comments: bool,
    line: &str,
    line_no: usize,
    comment_markers: &[String],
    start_col: usize,
) -> Result<(Vec<Token>, i64), CapyError> {
    let mut toks: Vec<Token> = Vec::new();
    let mut open: i64 = 0;
    let mut i = 0usize;
    let mut col = start_col;
    let b = line.as_bytes();

    while i < b.len() {
        // Decode the next rune so non-ASCII letters / symbols / emoji don't get
        // bit-truncated to garbage. ASCII chars are width 1 and the single-byte
        // fast paths stay correct.
        let r = line[i..].chars().next().unwrap();

        let push_delim = |toks: &mut Vec<Token>, kind: TokenKind, text: &str| {
            toks.push(Token {
                kind,
                text: text.to_string(),
                line: line_no,
                col,
                width: 1,
            });
        };

        if r == ' ' || r == '\t' {
            i += 1;
            col += 1;
        } else if has_comment_prefix(&line[i..], comment_markers) {
            // R27: retain the trailing comment, then skip it as before.
            if keep_comments {
                let rest = line[i..].trim_end();
                toks.push(Token {
                    kind: TokenKind::Comment,
                    text: rest.to_string(),
                    line: line_no,
                    col,
                    width: rest.len(),
                });
            }
            i = b.len();
        } else if r == '"' || r == '\'' {
            let (s, n) = read_string(&line[i..], r as u8)
                .map_err(|e| CapyError::msg(format!("line {}: {}", line_no, e)))?;
            toks.push(Token { kind: TokenKind::Str, text: s, line: line_no, col, width: n });
            i += n;
            col += n;
        } else if r == '`' {
            let (s, n) = read_string(&line[i..], b'`')
                .map_err(|e| CapyError::msg(format!("line {}: {}", line_no, e)))?;
            toks.push(Token { kind: TokenKind::Template, text: s, line: line_no, col, width: n });
            i += n;
            col += n;
        } else if r == '(' {
            push_delim(&mut toks, TokenKind::LParen, "(");
            i += 1;
            col += 1;
            open += 1;
        } else if r == ')' {
            push_delim(&mut toks, TokenKind::RParen, ")");
            i += 1;
            col += 1;
            open -= 1;
        } else if r == '[' {
            push_delim(&mut toks, TokenKind::LBrack, "[");
            i += 1;
            col += 1;
            open += 1;
        } else if r == ']' {
            push_delim(&mut toks, TokenKind::RBrack, "]");
            i += 1;
            col += 1;
            open -= 1;
        } else if r == '{' {
            push_delim(&mut toks, TokenKind::LBrace, "{");
            i += 1;
            col += 1;
            open += 1;
        } else if r == '}' {
            push_delim(&mut toks, TokenKind::RBrace, "}");
            i += 1;
            col += 1;
            open -= 1;
        } else if is_digit(r)
            // NOTE: the Go guard ends in `|| true`, which makes the entire
            // trailing parenthesised group dead. The effective condition is
            // just "a digit, or a '-' immediately followed by a digit".
            // Go casts `line[i+1]` (a byte) to a rune, so only ASCII digits
            // qualify for the lookahead.
            || (r == '-' && i + 1 < b.len() && (b[i + 1] as char).is_ascii_digit())
        {
            let n = read_number(&line[i..]);
            toks.push(Token {
                kind: TokenKind::Number,
                text: line[i..i + n].to_string(),
                line: line_no,
                col,
                width: n,
            });
            i += n;
            col += n;
        } else if is_ident_start(r) {
            let n = read_ident(&line[i..]);
            toks.push(Token {
                kind: TokenKind::Ident,
                text: line[i..i + n].to_string(),
                line: line_no,
                col,
                width: n,
            });
            i += n;
            col += n;
        } else if (r as u32) < 0x80 && is_punct(r as u8) {
            let n = read_punct(&line[i..]);
            toks.push(Token {
                kind: TokenKind::Punct,
                text: line[i..i + n].to_string(),
                line: line_no,
                col,
                width: n,
            });
            i += n;
            col += n;
        } else {
            return Err(CapyError::msg(format!(
                "line {} col {}: unexpected character {}",
                line_no,
                col,
                gofmt::quote_rune(r)
            )));
        }
    }
    Ok((toks, open))
}

/// Port of `readString`. Returns the content with quotes stripped (escape
/// sequences left intact) and the total byte width including both quotes.
fn read_string(s: &str, quote: u8) -> Result<(String, usize), String> {
    let b = s.as_bytes();
    if b.is_empty() || b[0] != quote {
        return Err("expected quote".to_string());
    }
    let mut out: Vec<u8> = Vec::new();
    let mut i = 1usize;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            out.push(b[i]);
            out.push(b[i + 1]);
            i += 2;
            continue;
        }
        if c == quote {
            return Ok((String::from_utf8_lossy(&out).into_owned(), i + 1));
        }
        out.push(c);
        i += 1;
    }
    Err("unterminated string".to_string())
}

/// Port of `readNumber`. Go tests each byte with `unicode.IsDigit(rune(c))`, so
/// only ASCII digits are consumed.
fn read_number(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 0usize;
    if !b.is_empty() && b[0] == b'-' {
        i += 1;
    }
    let mut dot = false;
    while i < b.len() {
        let c = b[i];
        if c == b'.' && !dot {
            // Only consume a dot if the next char is a digit (avoid `a.b`).
            if i + 1 < b.len() && b[i + 1].is_ascii_digit() {
                dot = true;
                i += 1;
                continue;
            }
            break;
        }
        if !c.is_ascii_digit() {
            break;
        }
        i += 1;
    }
    i
}

/// Port of `readPunct`.
fn read_punct(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() && is_punct(b[i]) {
        i += 1;
    }
    i
}

/// Port of `isIdentStart`.
///
/// Accepts ASCII letters, underscore, AND any non-ASCII rune. The lexer is
/// purely lexical and Capy makes no assumptions about how users write content —
/// em-dashes, smart quotes, accented Latin, CJK, and emoji must all flow
/// through bare-prose positions without erroring. ASCII punctuation that
/// participates in literal matches is checked separately BEFORE this predicate.
fn is_ident_start(r: char) -> bool {
    r == '_' || r.is_alphabetic() || (r as u32) >= 0x80
}

/// Port of `isIdentPart`.
fn is_ident_part(r: char) -> bool {
    r == '_' || r.is_alphabetic() || is_digit(r) || (r as u32) >= 0x80
}

/// Port of `readIdent`. Walks runes (not bytes) so a multi-byte sequence like
/// `é` is consumed as one rune and the returned byte length covers the whole
/// encoded form.
fn read_ident(s: &str) -> usize {
    let mut i = 0usize;
    for c in s.chars() {
        if !is_ident_part(c) {
            break;
        }
        i += c.len_utf8();
    }
    i
}
