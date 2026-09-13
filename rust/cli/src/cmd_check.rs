//! Port of `cmd/capy/cmd_check.go`.

use capy_core::gopath;
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

/// Port of `cmdCheck`.
///
/// Parses + validates a library file without running any source. Useful for
/// libraries-as-data CI.
pub fn cmd_check(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: capy check <library.yaml>".to_string());
    }
    let path = &args[0];
    std::fs::metadata(path).map_err(|e| gopath::io_error("stat", path, &e))?;
    let lib = make_library_loader::load_library(path, make_lexer::tokenize)
        .map_err(|e| e.to_string())?;
    println!("ok — {} function(s), {} type(s)", lib.functions.len(), lib.types.len());
    // Go iterates the maps unsorted here, so its listing order varies per run;
    // BTreeMap makes it alphabetical.
    for name in lib.functions.keys() {
        println!("  function {}", name);
    }
    for name in lib.types.keys() {
        println!("  type     {}", name);
    }
    Ok(())
}
