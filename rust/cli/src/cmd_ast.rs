//! `capy ast` — print a script's parse tree (PLAN-2026-0002 R7).
//!
//! Two modes. The default is an indented tree for a human reading a terminal;
//! `--json` emits the machine-readable form documented in `docs/ast-json.md`,
//! on stdout and nothing else, so it can be piped straight into `jq`.
//!
//! Unlike `capy run`, this does not fail on a broken script: parsing recovers,
//! so a file with mistakes still yields a tree plus the diagnostics explaining
//! what went wrong. The exit code says whether the parse was clean.

use super::flags::{self, reorder_flags_first, Spec};
use capy_core::capy::Library;
use capy_core::domain::ast::{Block, FuncCall};
use capy_core::domain::ast_json;
use capy_core::gopath;

const AST_SPEC: Spec = Spec { bools: &["json"], strings: &[] };

pub fn cmd_ast(args: &[String]) -> Result<i32, String> {
    let args = reorder_flags_first(args);
    let fs = flags::parse(&AST_SPEC, &args)?;
    let pos = &fs.positionals;
    if pos.len() != 2 {
        return Err("usage: capy ast <library> <script> [--json]".to_string());
    }
    let (lib_path, script_path) = (&pos[0], &pos[1]);

    let lib = Library::from_file(lib_path).map_err(|e| e.to_string())?;
    let src = std::fs::read_to_string(script_path)
        .map_err(|e| gopath::io_error("open", script_path, &e))?;

    let result = lib.parse(&src);

    if fs.bool("json") {
        println!("{}", ast_json::to_json_pretty(&result));
    } else {
        for st in &result.tree.stmts {
            print_node(st, 0);
        }
        for e in &result.tree.errors {
            println!(
                "<error> {}:{}-{}:{}  {} token(s) skipped",
                e.span.start_line,
                e.span.start_col,
                e.span.end_line,
                e.span.end_col,
                e.tokens.len()
            );
        }
        // Diagnostics go to stderr so the tree on stdout stays pipeable even in
        // tree mode.
        for d in &result.diagnostics {
            eprintln!(
                "{}[{}] {}:{}: {}",
                match d.severity {
                    capy_core::domain::errors::Severity::Error => "error",
                    capy_core::domain::errors::Severity::Warning => "warning",
                    _ => "note",
                },
                d.code,
                d.primary.start_line,
                d.primary.start_col,
                d.full_message()
            );
        }
    }

    // Exit 1 when the parse was not clean, so a script can gate on it.
    Ok(if result.is_clean() { 0 } else { 1 })
}

fn print_node(f: &FuncCall, depth: usize) {
    let pad = "  ".repeat(depth);
    println!(
        "{pad}{} {}:{}-{}:{}",
        f.func, f.span.start_line, f.span.start_col, f.span.end_line, f.span.end_col
    );
    for c in &f.leading_comments {
        println!("{pad}  # comment {}:{}-{}:{}", c.start_line, c.start_col, c.end_line, c.end_col);
    }
    for (name, cap) in &f.captures {
        if cap.sub.is_empty() {
            println!(
                "{pad}  {name} = {:?} {}:{}",
                cap.text, cap.span.start_line, cap.span.start_col
            );
        } else {
            println!("{pad}  {name}:");
            for s in &cap.sub {
                print_node(s, depth + 2);
            }
        }
    }
    if let Some(b) = &f.body {
        print_block(b, depth + 1);
    }
    if let Some(c) = &f.closer {
        print_node(c, depth);
    }
}

fn print_block(b: &Block, depth: usize) {
    for st in &b.stmts {
        print_node(st, depth);
    }
}
