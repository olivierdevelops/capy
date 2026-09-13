//! Port of `cmd/capy/cmd_docs.go`.

use super::flags::{self, Spec};
use capy_core::domain::docs::render_library_docs;
use capy_core::gopath;
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

const DOCS_SPEC: Spec = Spec { bools: &[], strings: &["out"] };

/// Port of `cmdDocs`.
///
/// Parses the library and emits Markdown reference documentation listing every
/// function, type, and the arg descriptions authors attached via `description`.
pub fn cmd_docs(args: &[String]) -> Result<(), String> {
    let fs = flags::parse(&DOCS_SPEC, args)?;
    if fs.positionals.len() != 1 {
        return Err("usage: capy docs [--out path] <library>".to_string());
    }
    let lib_path = &fs.positionals[0];
    std::fs::metadata(lib_path).map_err(|e| gopath::io_error("stat", lib_path, &e))?;
    let lib = make_library_loader::load_library(lib_path, make_lexer::tokenize)
        .map_err(|e| e.to_string())?;
    let md = render_library_docs(&lib);
    let out = fs.str("out");
    if !out.is_empty() {
        return std::fs::write(out, md.as_bytes())
            .map_err(|e| gopath::io_error("open", out, &e));
    }
    print!("{}", md);
    Ok(())
}
