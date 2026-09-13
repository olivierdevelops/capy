//! Port of `cmd/capy/cmd_help.go`.

use capy_core::gopath;
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

/// Port of `printLibraryHelp` — loads a library and prints its declared commands.
pub fn print_library_help(lib_path: &str) -> Result<(), String> {
    let lib = make_library_loader::load_library(lib_path, make_lexer::tokenize)
        .map_err(|e| e.to_string())?;
    let lib_name = if lib.lib_name.is_empty() {
        let base = gopath::base(lib_path);
        let ext = gopath::ext(lib_path);
        base.strip_suffix(&ext).unwrap_or(&base).to_string()
    } else {
        lib.lib_name.clone()
    };
    if !lib.lib_version.is_empty() {
        println!("{} {}", lib_name, lib.lib_version);
    } else {
        println!("{}", lib_name);
    }
    if !lib.description.is_empty() {
        println!("{}", lib.description);
    }
    println!();
    println!("COMMANDS");
    if lib.commands.is_empty() {
        println!("    (none declared — the default `run` renders to stdout)");
    }
    // BTreeMap iterates sorted, matching Go's sort.Strings.
    for (n, c) in &lib.commands {
        let desc = if c.description.is_empty() { "(no description)" } else { &c.description };
        println!("    {:<12}  {}", n, desc);
    }
    println!("\nRun `capy {} <command> --help` for command-specific help.", lib_name);
    Ok(())
}

/// Port of `cmdHelp`.
pub fn cmd_help(name: &str) -> Result<(), String> {
    match name {
        "run" => println!(
            r#"capy run <library.yaml> <script.capy>

Transpile a script against a library. Output goes to stdout unless the
library sets `output_file:` or you pass --out.

Flags:
  --out <path>    write output to this file instead of stdout
  --no-color      disable ANSI escape codes (reserved)
  --debug         verbose engine tracing (reserved)"#
        ),
        "check" => println!(
            r#"capy check <library.yaml>

Parse and validate a library file without running anything. Reports
loaded functions and types if valid, or a structured error otherwise."#
        ),
        "init" => println!(
            r#"capy init [<dir>]

Scaffold a starter project (lib.yaml + script.capy + README.md) in the
given directory (default '.'). Refuses to overwrite existing files."#
        ),
        "version" => println!(
            r#"capy version

Print the version string baked in at build time."#
        ),
        other => {
            eprintln!("no help for {:?}", other);
            super::print_usage();
        }
    }
    Ok(())
}
