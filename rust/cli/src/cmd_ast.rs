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
use capy_core::domain::ast_json;
use capy_core::domain::ast_text;
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
        // One renderer, shared with `rust/tests/golden.rs` — see
        // `capy_core::domain::ast_text`. The tree goes to stdout so it stays
        // pipeable; diagnostics go to stderr.
        print!("{}", ast_text::render_tree(&result));
        eprint!("{}", ast_text::render_diagnostics(&result));
    }

    // Exit 1 when the parse was not clean, so a script can gate on it.
    Ok(if result.is_clean() { 0 } else { 1 })
}
