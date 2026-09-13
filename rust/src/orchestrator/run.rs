//! Port of `orchestrator/run.go`.

use super::features::{
    make_evaluator, make_lexer, make_library_loader, make_parser,
};
use crate::domain::errors::CapyError;
use crate::gopath;
use crate::infra::os_host::OsHost;
use crate::infra::{define_extractor, preprocessor};
use std::collections::BTreeMap;
use std::fs;
use std::rc::Rc;

/// Port of `Run`.
///
/// Loads a library from disk, reads a script, and produces the transpiled output
/// as a string.
pub fn run(library_path: &str, script_path: &str) -> Result<String, CapyError> {
    let (out, _) = run_multi(library_path, script_path)?;
    Ok(out)
}

/// Port of `RunMulti`.
pub fn run_multi(
    library_path: &str,
    script_path: &str,
) -> Result<(String, BTreeMap<String, String>), CapyError> {
    run_multi_with_args(library_path, script_path, &[])
}

/// Port of `stripShebang`.
///
/// Removes a leading `#!` line if present, so scripts can be made executable via
/// `#!/usr/bin/env capy --lib X` without confusing the lexer.
fn strip_shebang(src: &[u8]) -> &[u8] {
    if src.len() < 2 || src[0] != b'#' || src[1] != b'!' {
        return src;
    }
    for i in 0..src.len() {
        if src[i] == b'\n' {
            return &src[i + 1..];
        }
    }
    // Go returns nil when the shebang line never terminates.
    &[]
}

/// Port of `RunMultiWithArgs`.
///
/// Passes positional CLI args through to the inner `arg`/`args`/`arg_count` host
/// primitives.
pub fn run_multi_with_args(
    library_path: &str,
    script_path: &str,
    user_args: &[String],
) -> Result<(String, BTreeMap<String, String>), CapyError> {
    let raw_src = fs::read(script_path)
        .map_err(|e| CapyError::msg(gopath::io_error("open", script_path, &e)))?;
    let src = strip_shebang(&raw_src).to_vec();
    let host = Rc::new(OsHost {
        user_args: user_args.to_vec(),
        base_dir: gopath::dir(script_path),
    });

    // Library FIRST — its `preprocess` declarations are needed before we can know
    // which (if any) source-level inclusion directives are allowed. Capy has no
    // built-in preprocessor; everything is opt-in per library.
    let mut lib = make_library_loader::load_library(library_path, make_lexer::tokenize)?;

    // Now expand any inclusion directives the library declared. With no
    // `preprocess` block, this returns the source unchanged.
    let src_str = String::from_utf8_lossy(&src).into_owned();
    let mut expanded =
        preprocessor::preprocess(&src_str, &gopath::dir(script_path), &lib.preprocess, host.as_ref())
            .map_err(CapyError::msg)?;

    // Extract any `define NAME … end` blocks (metaprogramming): the source can
    // introduce new functions for the rest of itself to use.
    let (cleaned, define_lib_src) =
        define_extractor::extract_defines(&expanded).map_err(CapyError::msg)?;
    expanded = cleaned;
    // Merge source-defined functions into the library. Source defines WIN on
    // conflict — `define foo … end` in the script overrides `function foo`.
    if !define_lib_src.is_empty() {
        let define_lib = make_library_loader::load_library_from_bytes(
            "capy",
            define_lib_src.as_bytes(),
            make_lexer::tokenize,
        )
        .map_err(|e| CapyError::msg(format!("define block: {}", e)))?;
        for (name, f) in define_lib.functions {
            lib.functions.insert(name, f);
        }
    }
    let toks = make_lexer::tokenize_with(&expanded, &lib.comments)?;
    let prog = make_parser::parse(toks, &expanded, &lib)?;
    make_evaluator::run_multi(&prog, &lib, host)
}

/// Port of `RunStrings`.
///
/// Like [`run`] but takes the library and script contents directly.
/// `library_path` is used only to resolve relative paths inside `import`
/// directives — pass an empty string if the library has none.
pub fn run_strings(
    library_src: &str,
    library_path: &str,
    script_src: &str,
) -> Result<String, CapyError> {
    // Spill the library source to a temp file so the loader's file-based API
    // (which resolves `import` paths relative to the file) has a stable basis.
    let mut lib_path = library_path.to_string();
    let mut tmp_guard: Option<String> = None;
    if lib_path.is_empty() {
        let host = OsHost::default();
        use crate::domain::host::Host;
        let p = host.mk_temp(".capy")?;
        fs::write(&p, library_src.as_bytes())
            .map_err(|e| CapyError::msg(gopath::io_error("open", &p, &e)))?;
        lib_path = p.clone();
        tmp_guard = Some(p);
    }

    let result = (|| -> Result<String, CapyError> {
        let lib = make_library_loader::load_library(&lib_path, make_lexer::tokenize)?;
        let toks = make_lexer::tokenize_with(script_src, &lib.comments)?;
        let prog = make_parser::parse(toks, script_src, &lib)?;
        make_evaluator::run(&prog, &lib)
    })();

    if let Some(p) = tmp_guard {
        let _ = fs::remove_file(p);
    }
    result
}
