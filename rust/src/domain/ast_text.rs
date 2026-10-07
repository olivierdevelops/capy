//! PROP-2026-0002 R12 — the human-readable rendering of a [`ParseResult`].
//!
//! This is the text `capy ast` prints without `--json`. It lives in `capy-core`
//! rather than in the CLI for one reason: `rust/tests/golden.rs` is an
//! integration test of `capy-core` and cannot reach into `capy-cli`, so a
//! golden that checked a *second* renderer would be asserting a format no user
//! ever sees. One renderer, two callers.
//!
//! The split into [`render_tree`] and [`render_diagnostics`] mirrors the two
//! streams the CLI writes: the tree goes to stdout so it stays pipeable, the
//! diagnostics go to stderr. A caller that wants what a terminal shows
//! concatenates them, which is exactly what an `<base>.expected-ast.txt` golden
//! holds.

use crate::capy::ParseResult;
use crate::domain::ast::{Block, FuncCall};
use crate::domain::errors::Severity;
use std::fmt::Write as _;

/// The parse tree: one line per node, two spaces per level, error regions last.
///
/// Every line ends in `\n`, including the last, so the output concatenates
/// cleanly with [`render_diagnostics`].
pub fn render_tree(result: &ParseResult) -> String {
    let mut out = String::new();
    for st in &result.tree.stmts {
        write_node(&mut out, st, 0);
    }
    for e in &result.tree.errors {
        let _ = writeln!(
            out,
            "<error> {}:{}-{}:{}  {} token(s) skipped",
            e.span.start_line,
            e.span.start_col,
            e.span.end_line,
            e.span.end_col,
            e.tokens.len()
        );
    }
    out
}

/// One line per diagnostic: `severity[CODE] line:col: message`.
///
/// Empty when the parse was clean, so appending it to a clean tree adds
/// nothing.
pub fn render_diagnostics(result: &ParseResult) -> String {
    let mut out = String::new();
    for d in &result.diagnostics {
        let _ = writeln!(
            out,
            "{}[{}] {}:{}: {}",
            // Exhaustive on purpose. `Severity` is `#[non_exhaustive]` for
            // *downstream* crates; in-crate a new variant must fail to compile
            // here rather than silently render as something else.
            match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            d.code,
            d.primary.start_line,
            d.primary.start_col,
            d.full_message()
        );
    }
    out
}

/// Tree followed by diagnostics — what `capy ast … 2>&1` shows, and the form a
/// golden file stores.
pub fn render(result: &ParseResult) -> String {
    let mut out = render_tree(result);
    out.push_str(&render_diagnostics(result));
    out
}

fn write_node(out: &mut String, f: &FuncCall, depth: usize) {
    let pad = "  ".repeat(depth);
    let _ = writeln!(
        out,
        "{pad}{} {}:{}-{}:{}",
        f.func, f.span.start_line, f.span.start_col, f.span.end_line, f.span.end_col
    );
    for c in &f.leading_comments {
        let _ = writeln!(
            out,
            "{pad}  # comment {}:{}-{}:{}",
            c.start_line, c.start_col, c.end_line, c.end_col
        );
    }
    for (name, cap) in &f.captures {
        if cap.sub.is_empty() {
            let _ = writeln!(
                out,
                "{pad}  {name} = {:?} {}:{}",
                cap.text, cap.span.start_line, cap.span.start_col
            );
        } else {
            let _ = writeln!(out, "{pad}  {name}:");
            for s in &cap.sub {
                write_node(out, s, depth + 2);
            }
        }
    }
    if let Some(b) = &f.body {
        write_block(out, b, depth + 1);
    }
    if let Some(c) = &f.closer {
        write_node(out, c, depth);
    }
}

fn write_block(out: &mut String, b: &Block, depth: usize) {
    for st in &b.stmts {
        write_node(out, st, depth);
    }
}
