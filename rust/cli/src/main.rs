//! Port of `cmd/capy/main.go` — the Capy CLI.
//!
//! Subcommands:
//!
//! ```text
//! capy run <library> <script>          transpile a script
//! capy ast <library> <script>           print the parse tree (--json)
//! capy check <library>                 validate a library
//! capy docs <library>                  print auto-generated reference docs
//! capy lib list|which|new|path         manage installed libraries (CAPY_LIBS)
//! capy new <dir> --using <library>     scaffold a new project from a library
//! capy <library-name> <command> [args] dispatch a library command
//! capy init [<dir>]                    legacy: scaffold a starter library
//! capy version
//! capy help [<command>]
//! ```
//!
//! Short forms also supported:
//!
//! ```text
//! capy <library.capy> <script>         positional run (legacy)
//! capy <script.<libname>>              auto-resolves the library by extension
//! ```

mod cmd_build;
mod cmd_ast;
mod cmd_check;
mod cmd_docs;
mod cmd_fmt;
mod cmd_help;
mod cmd_init;
mod cmd_lib;
mod cmd_new;
mod cmd_run;
mod cmd_watch;
mod flags;
mod impl_resolve;
mod lib_path;

use capy_core::gopath;
use capy_core::orchestrator::commands;
use impl_resolve::resolve_lib_with_impl;
use lib_path::{library_name_looks_valid, resolve_lib};

/// Set at build time via `CAPY_VERSION`, mirroring Go's
/// `-ldflags "-X main.version=…"`.
const VERSION: &str = match option_env!("CAPY_VERSION") {
    Some(v) => v,
    None => "dev",
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

/// Port of `dispatch`. Returns the process exit code.
fn dispatch(args: &[String]) -> Result<i32, String> {
    if args.is_empty() {
        print_usage();
        return Ok(0);
    }
    // Shebang-style scripts: `#!/usr/bin/env capy --lib recipe` makes the OS
    // invoke `capy --lib recipe <script>`. Honour --lib as "treat the next
    // positional as a script for this library."
    if args[0] == "--lib" {
        if args.len() < 3 {
            return Err("usage: capy --lib <library> <script> [args...]".to_string());
        }
        let lib_path = resolve_lib(&args[1])?;
        return commands::run_command(&lib_path, "run", &args[2..])
            .map(|_| 0)
            .map_err(|e| e.to_string());
    }
    match args[0].as_str() {
        "run" => return cmd_run::cmd_run(&args[1..]).map(|_| 0),
        "check" => return cmd_check::cmd_check(&args[1..]).map(|_| 0),
        "docs" => return cmd_docs::cmd_docs(&args[1..]).map(|_| 0),
        "init" => return cmd_init::cmd_init(&args[1..]).map(|_| 0),
        "lib" => return cmd_lib::cmd_lib(&args[1..]).map(|_| 0),
        "new" => return cmd_new::cmd_new(&args[1..]).map(|_| 0),
        "watch" => return cmd_watch::cmd_watch(&args[1..]).map(|_| 0),
        // `fmt --check` exits 1 when a file would be reformatted.
        "fmt" => return cmd_fmt::cmd_fmt(&args[1..]),
        "ast" => return cmd_ast::cmd_ast(&args[1..]),
        "build" => return cmd_build::cmd_build(&args[1..]).map(|_| 0),
        "version" | "--version" | "-v" => {
            println!("capy {}", VERSION);
            return Ok(0);
        }
        "help" | "--help" | "-h" => {
            if args.len() > 1 {
                return cmd_help::cmd_help(&args[1]).map(|_| 0);
            }
            print_usage();
            return Ok(0);
        }
        _ => {}
    }
    // Library-name short form: `capy <lib> <command> [args]`. Only if the first
    // arg looks like a library name AND resolves on CAPY_LIBS.
    if library_name_looks_valid(&args[0]) {
        // Pre-scan for --impl <name>; the rest flow to the command unchanged.
        let (impl_flag, rest) = extract_impl_flag(&args[1..]);

        match resolve_lib_with_impl(&args[0], &impl_flag) {
            Ok(r) => {
                // `capy <lib> --help` lists declared commands from the manifest
                // (visible regardless of which impl is picked).
                if !rest.is_empty() && (rest[0] == "--help" || rest[0] == "-h") {
                    return cmd_help::print_library_help(&r.manifest_path).map(|_| 0);
                }
                if rest.is_empty() {
                    return Err(format!(
                        "library {:?} resolved at {}; pick a command (try: run, build, compile, docs)",
                        args[0], r.manifest_path
                    ));
                }
                return commands::run_command(&r.impl_path, &rest[0], &rest[1..])
                    .map(|_| 0)
                    .map_err(|e| e.to_string());
            }
            Err(res_err) => {
                // Resolution failed because of the impl selector (multiple impls,
                // no default) — that error is more useful than "not found".
                if res_err.contains("impl") {
                    return Err(res_err);
                }
                // The first arg didn't resolve as a library. If it also doesn't
                // exist as a file AND the second arg doesn't look like a script
                // path, the user clearly MEANT the short form — report the
                // resolution failure rather than falling through to cmd_run's
                // "no such file".
                if std::fs::metadata(&args[0]).is_err()
                    && (args.len() < 2 || gopath::ext(&args[1]).is_empty())
                {
                    return Err(res_err);
                }
            }
        }
    }
    // File-extension convention: `capy <script.<libname>>` auto-resolves the
    // library by extension.
    if args.len() == 1 {
        if let Some(lib_path) = resolve_lib_from_script_ext(&args[0]) {
            return commands::run_command(&lib_path, "run", &[args[0].clone()])
                .map(|_| 0)
                .map_err(|e| e.to_string());
        }
    }
    // Legacy positional form: `capy <library.capy> <script>`.
    cmd_run::cmd_run(args).map(|_| 0)
}

/// Port of `extractImplFlag`.
///
/// Pulls `--impl <name>` (or `--impl=<name>`) out of args, returning the impl
/// name (empty when absent) and the rest with the flag removed.
fn extract_impl_flag(args: &[String]) -> (String, Vec<String>) {
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    let mut impl_name = String::new();
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        if a == "--impl" || a == "-i" {
            if i + 1 < args.len() {
                impl_name = args[i + 1].clone();
                i += 1;
            }
            i += 1;
            continue;
        }
        if let Some(v) = a.strip_prefix("--impl=") {
            impl_name = v.to_string();
            i += 1;
            continue;
        }
        out.push(a.clone());
        i += 1;
    }
    (impl_name, out)
}

/// Port of `resolveLibFromScriptExt` — `cake.recipe` → look up `recipe`.
fn resolve_lib_from_script_ext(script_path: &str) -> Option<String> {
    let ext = gopath::ext(script_path);
    if ext.is_empty() || ext == ".capy" {
        return None;
    }
    let libname = ext.strip_prefix('.').unwrap_or(&ext);
    resolve_lib(libname).ok()
}

/// Port of `printUsage`.
pub fn print_usage() {
    println!(
        r#"capy — a transpiler engine driven by a Capy library

Usage:
  capy run <library> <script>            transpile a script
  capy ast <library> <script> [--json]   print the parse tree
  capy check <library>                   validate a library
  capy docs <library>                    print auto-generated reference docs
  capy lib list                          list installed libraries (CAPY_LIBS)
  capy lib which <name>                  show full path of a library
  capy lib new <name>                    scaffold a new library
  capy lib path                          print the library search path
  capy new <dir> --using <library>       scaffold a new project from a library
  capy <library> <command> [args]        dispatch a library command
  capy version                           print version
  capy help [<command>]                  show detailed help

Examples:
  CAPY_LIBS=~/.capy/libs capy lib list
  capy lib new recipe
  capy recipe run examples/hello.recipe
  capy cake.recipe                       # auto-detects library by extension

See https://olivierdevelops.github.io/capy/ for documentation."#
    );
}
