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
