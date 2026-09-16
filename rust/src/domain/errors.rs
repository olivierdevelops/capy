//! Port of `domain/errors.go`.

use std::fmt;

/// The structured error type produced by the engine. Carries a position so the
/// CLI can render a caret-pointed context block, plus an optional hint
/// describing how to fix the problem.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CapyError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
    /// Optional human-readable suggestion shown beneath the error. Use for typo
    /// corrections, listing valid options, etc.
    pub hint: String,
    /// Source file path when known. Errors from imported files set this to the
    /// import path; top-level errors leave it empty (the CLI fills in the
    /// script path).
    pub file: String,

    /// FIDELITY: Go has TWO error kinds flowing through the engine — `*CapyError`
    /// and plain `fmt.Errorf`/`os` errors — and [`format_with_source`]
    /// type-switches on the difference: a plain error is returned verbatim, while
    /// a `*CapyError` gains the `error: ` prefix, the hint line, and the caret.
    /// Rust funnels both through this one struct, so `plain` records which kind
    /// Go would have produced. Set by [`CapyError::msg`] (the `fmt.Errorf`
    /// equivalent) and cleared by [`CapyError::new`] / [`CapyError::structured`]
    /// (the `&CapyError{…}` equivalents).
    pub plain: bool,
}

impl CapyError {
    /// Port of `domain.NewError` — a structured, positioned engine error.
    pub fn new(line: usize, col: usize, msg: impl Into<String>) -> CapyError {
        CapyError { line, col, msg: msg.into(), plain: false, ..Default::default() }
    }

    /// The `fmt.Errorf` equivalent: a plain error that `format_with_source`
    /// returns verbatim, with no `error: ` prefix and no caret.
    pub fn msg(msg: impl Into<String>) -> CapyError {
        CapyError { msg: msg.into(), plain: true, ..Default::default() }
    }

    /// The positionless `&CapyError{Msg: …}` equivalent — structured, so
    /// `format_with_source` renders the prefix and any hint.
    pub fn structured(msg: impl Into<String>) -> CapyError {
        CapyError { msg: msg.into(), plain: false, ..Default::default() }
    }

    /// Port of `(*CapyError).WithHint`.
    pub fn with_hint(mut self, hint: impl Into<String>) -> CapyError {
        self.hint = hint.into();
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> CapyError {
        self.file = file.into();
        self
    }
}

impl fmt::Display for CapyError {
    /// Port of `(*CapyError).Error`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut loc = String::new();
        if !self.file.is_empty() {
            loc.push_str(&self.file);
        }
        if self.line > 0 {
            if !loc.is_empty() {
                loc.push(':');
            }
            loc.push_str(&self.line.to_string());
            if self.col > 0 {
                loc.push(':');
                loc.push_str(&self.col.to_string());
            }
        }
        if loc.is_empty() {
            write!(f, "{}", self.msg)
        } else {
            write!(f, "{}: {}", loc, self.msg)
        }
    }
}

impl std::error::Error for CapyError {}

/// Port of `domain.FormatWithSource`.
///
/// Renders a `CapyError` with the offending source line, a caret pointing at
/// the column, and an optional hint. If the error has no position, returns the
/// plain error string.
///
/// Example output:
///
/// ```text
/// error: no library function matches token "endpiont"
///   hint: did you mean "endpoint"?
///   3 │     endpiont GET "/users"
///     │     ^
/// ```
pub fn format_with_source(err: &CapyError, source: &str) -> String {
    // Go's type switch: anything that isn't a *CapyError is returned as-is.
    if err.plain {
        return err.to_string();
    }
    let mut b = String::new();
    b.push_str(&format!("error: {}\n", err.msg));
    if !err.hint.is_empty() {
        b.push_str(&format!("  hint: {}\n", err.hint));
    }
    if err.line == 0 || source.is_empty() {
        return b.trim_end_matches('\n').to_string();
    }
    let lines: Vec<&str> = source.split('\n').collect();
    if err.line > lines.len() {
        return b.trim_end_matches('\n').to_string();
    }
    let line = lines[err.line - 1];
    b.push_str(&format!("  {} │ {}\n", err.line, line));
    if err.col > 0 {
        let pad = " ".repeat(err.col - 1);
        b.push_str(&format!("    │ {}^\n", pad));
    }
    b.trim_end_matches('\n').to_string()
}

/// Port of `domain.SuggestClosest`.
///
/// Returns the entry from `candidates` with the smallest edit distance to
/// `target`, provided it is within `max_dist`. Returns `None` when nothing is
/// close enough. Powers "did you mean X?" hints.
pub fn suggest_closest(target: &str, candidates: &[String], max_dist: usize) -> Option<String> {
    let mut best: Option<&str> = None;
    let mut best_dist = max_dist + 1;
    for c in candidates {
        let d = edit_distance(target, c);
        if d < best_dist {
            best_dist = d;
            best = Some(c);
        }
    }
    if best_dist > max_dist {
        return None;
    }
    best.map(|s| s.to_string())
}

/// Port of `domain.SuggestClosestSorted`. The Go version collects a map keyset
/// and sorts before iterating so the picked candidate is deterministic when
/// several tie on distance; callers here pass a `BTreeSet`/sorted slice, which
/// already iterates in sorted order.
pub fn suggest_closest_sorted<'a, I>(target: &str, keys: I, max_dist: usize) -> Option<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let mut v: Vec<String> = keys.into_iter().cloned().collect();
    v.sort();
    suggest_closest(target, &v, max_dist)
}

/// Port of `editDistance` — Levenshtein over BYTES, matching the Go original
/// (which indexes `a[i-1]`/`b[j-1]` on the raw string, not runes).
fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let (la, lb) = (a.len(), b.len());
    if la == 0 {
        return lb;
    }
    if lb == 0 {
        return la;
    }
    let mut prev: Vec<usize> = (0..=lb).collect();
    let mut curr: Vec<usize> = vec![0; lb + 1];
    for i in 1..=la {
        curr[0] = i;
        for j in 1..=lb {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[lb]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_string_formats_position() {
        assert_eq!(CapyError::new(3, 5, "boom").to_string(), "3:5: boom");
        assert_eq!(CapyError::new(3, 0, "boom").to_string(), "3: boom");
        assert_eq!(CapyError::msg("boom").to_string(), "boom");
        assert_eq!(
            CapyError::new(3, 5, "boom").with_file("a.capy").to_string(),
            "a.capy:3:5: boom"
        );
    }

    #[test]
    fn plain_errors_pass_through_unformatted() {
        // Mirrors Go returning `err.Error()` for a non-*CapyError.
        let e = CapyError::msg("open nope.capy: no such file or directory");
        assert_eq!(
            format_with_source(&e, "irrelevant source"),
            "open nope.capy: no such file or directory"
        );
    }

    #[test]
    fn format_with_source_draws_caret() {
        let err = CapyError::new(2, 5, "no match").with_hint("did you mean \"endpoint\"?");
        let src = "line one\n    endpiont GET\nline three";
        let got = format_with_source(&err, src);
        assert_eq!(
            got,
            "error: no match\n  hint: did you mean \"endpoint\"?\n  2 │     endpiont GET\n    │     ^"
        );
    }

    #[test]
    fn suggest_closest_within_distance() {
        let cands = vec!["endpoint".to_string(), "model".to_string()];
        assert_eq!(suggest_closest("endpiont", &cands, 2), Some("endpoint".to_string()));
        assert_eq!(suggest_closest("zzzzzzzz", &cands, 2), None);
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("endpiont", "endpoint"), 2);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }
}

// ── PLAN-2026-0002 — diagnostics (R18, R26) ─────────────────────────────────

use crate::domain::ast::Span;

/// How serious a diagnostic is.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A secondary annotation on a diagnostic — the part that turns one caret into
/// the two-span form:
///
/// ```text
///  1 | fn add(x: int, y: int {
///    |       -              ^ expected `)` here      ← primary
///    |       |
///    |       unclosed `(` opened here                ← label
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub span: Span,
    pub text: String,
}

impl Label {
    pub fn new(span: Span, text: impl Into<String>) -> Label {
        Label { span, text: text.into() }
    }
}

/// Where the matcher was when it gave up. Without this, "expected `)`" cannot
/// say *which* `)` — the reported shape may not even be the one the author
/// meant, which is the known weakness of furthest-failure reporting.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ContextFrame {
    /// The library function being matched.
    pub shape: String,
    /// 0-based index of the argument under consideration.
    pub arg_index: usize,
    /// The argument's name, when it has one.
    pub arg_name: String,
}

impl ContextFrame {
    pub fn new(shape: impl Into<String>, arg_index: usize, arg_name: impl Into<String>) -> ContextFrame {
        ContextFrame { shape: shape.into(), arg_index, arg_name: arg_name.into() }
    }

    /// Rendered as the trailing clause of a diagnostic message.
    pub fn describe(&self) -> String {
        if self.arg_name.is_empty() {
            format!("in `{}`", self.shape)
        } else {
            format!("in argument `{}` of `{}`", self.arg_name, self.shape)
        }
    }
}

/// A parse diagnostic.
///
/// `CapyError` remains the engine's error type and `Library::run` still returns
/// one; `Diagnostic` is what a recovering parse collects, and carries the things
/// `CapyError` has no room for: a severity, a stable machine-readable code, a
/// primary *range* rather than a point, secondary labels, and the matcher
/// context.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// Stable and machine-readable, e.g. `E0001`. Consumers may match on it, so
    /// a code's meaning does not change once published.
    pub code: &'static str,
    pub primary: Span,
    pub labels: Vec<Label>,
    pub msg: String,
    pub help: Option<String>,
    pub context: Vec<ContextFrame>,
}

/// Diagnostic codes. Stable once published; add, never repurpose.
pub mod codes {
    /// No library function matches the statement.
    pub const NO_MATCH: &str = "E0001";
    /// A delimiter was opened and never closed.
    pub const UNCLOSED_DELIM: &str = "E0002";
    /// Source nests deeper than the parser will follow.
    pub const NESTING_TOO_DEEP: &str = "E0003";
}

impl Diagnostic {
    pub fn error(code: &'static str, primary: Span, msg: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            code,
            primary,
            labels: Vec::new(),
            msg: msg.into(),
            help: None,
            context: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: Label) -> Diagnostic {
        self.labels.push(label);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Diagnostic {
        self.help = Some(help.into());
        self
    }

    pub fn with_context(mut self, context: Vec<ContextFrame>) -> Diagnostic {
        self.context = context;
        self
    }

    /// Lossy conversion to the engine's error type, for `Library::run`, which
    /// keeps its signature. The code and labels are folded into the text so
    /// nothing is silently dropped from what the user sees.
    pub fn to_capy_error(&self) -> CapyError {
        let mut e = CapyError::new(self.primary.start_line, self.primary.start_col, self.full_message());
        if let Some(h) = &self.help {
            e.hint = h.clone();
        }
        e
    }

    /// The message with its context clause appended, if any.
    pub fn full_message(&self) -> String {
        match self.context.last() {
            Some(f) => format!("{} {}", self.msg, f.describe()),
            None => self.msg.clone(),
        }
    }
}

/// Render a diagnostic with its source, primary caret and secondary labels.
///
/// Extends what [`format_with_source`] draws for a `CapyError`; a diagnostic
/// with no labels renders the same shape, so existing output does not change.
pub fn format_diagnostic(d: &Diagnostic, source: &str, file: &str) -> String {
    let mut b = String::new();
    let sev = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    b.push_str(&format!("{}[{}]: {}\n", sev, d.code, d.full_message()));
    if d.primary.start_line > 0 {
        let where_ = if file.is_empty() { String::new() } else { format!("{file}:") };
        b.push_str(&format!(
            "  --> {}{}:{}\n",
            where_, d.primary.start_line, d.primary.start_col
        ));
    }
    let lines: Vec<&str> = source.split('\n').collect();
    let mut draw = |span: Span, marker: char, text: &str| {
        if span.start_line == 0 || span.start_line > lines.len() {
            return;
        }
        let line = lines[span.start_line - 1];
        let width = span.end_col.saturating_sub(span.start_col).max(1);
        let pad = " ".repeat(span.start_col.saturating_sub(1));
        let carets: String = std::iter::repeat(marker).take(width).collect();
        b.push_str(&format!("  {} │ {}\n", span.start_line, line));
        b.push_str(&format!("    │ {}{} {}\n", pad, carets, text));
    };
    draw(d.primary, '^', "");
    for l in &d.labels {
        draw(l.span, '-', &l.text);
    }
    if let Some(h) = &d.help {
        b.push_str(&format!("  help: {h}\n"));
    }
    b.trim_end_matches('\n').to_string()
}
