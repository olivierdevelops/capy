//! Port of `orchestrator/commands.go`.

use super::command_args::{command_help, parse_command_args};
use super::features::{inner_evaluator::InnerEvaluator, make_lexer, make_library_loader};
use super::run;
use crate::domain::errors::CapyError;
use crate::domain::val::Val;
use crate::domain::{CaptureValue, Library};
use crate::gofmt;
use crate::gopath;
use crate::infra::os_host::OsHost;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Port of `RunCommand`.
///
/// Looks up command `cmd_name` on the library at `library_path` and runs its
/// body. Positional args after the command name are exposed to the body via
/// `args.0`, `args.1`, … and `args` (list).
///
/// The command body has access to:
///   * a `compile <script>` primitive that runs the library on a script
///   * shell-like host primitives (write_file, mktemp, exec, …)
///   * standard inner-DSL state mutation (set / append / let / if / for)
pub fn run_command(
    library_path: &str,
    cmd_name: &str,
    args: &[String],
) -> Result<(), CapyError> {
    // Trust check: warn when the library is NOT on CAPY_LIBS. Library commands
    // can shell out arbitrarily; surface the surprise early.
    if should_warn_untrusted(library_path) {
        eprintln!(
            "warning: library {} is not on CAPY_LIBS — its commands can shell out / write files / read env",
            gofmt::quote(library_path)
        );
    }

    let lib = make_library_loader::load_library(library_path, make_lexer::tokenize)?;
    let cmd = match lib.commands.get(cmd_name) {
        None => {
            return Err(CapyError::msg(format!(
                "library {} has no command {} (declared: [{}])",
                gofmt::quote(library_path),
                gofmt::quote(cmd_name),
                sorted_command_names(&lib).join(" ")
            )))
        }
        Some(c) => c.clone(),
    };

    // `--help` / `-h` short-circuit.
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", command_help(&lib, &cmd));
        return Ok(());
    }

    // Parse declared positional args + flags. Anything not consumed overflows
    // into `extra` (still surfaced via context.args).
    let parsed = parse_command_args(&cmd, args)?;

    let host = Rc::new(OsHost {
        user_args: args.to_vec(),
        base_dir: gopath::dir(library_path),
    });

    // Build a fresh execution context.
    let mut ctx: BTreeMap<String, Val> = BTreeMap::new();
    ctx.insert("lib_path".to_string(), Val::str(library_path));
    ctx.insert("lib_dir".to_string(), Val::Str(gopath::dir(library_path)));
    ctx.insert("lib_name".to_string(), Val::Str(name_or_path(&lib, library_path)));
    ctx.insert("lib_version".to_string(), Val::Str(lib.lib_version.clone()));
    ctx.insert("args".to_string(), any_list(args));
    ctx.insert("extra".to_string(), any_list(&parsed.extra));
    // Declared positional args appear under their declared names
    // (e.g. `arg "script"` → context.script).
    for (name, val) in parsed.pos {
        ctx.insert(name, val);
    }
    // Declared flags appear under context.flags.NAME (trimmed of dashes).
    let mut flags_map: BTreeMap<String, Val> = BTreeMap::new();
    for (name, val) in parsed.flags {
        flags_map.insert(name, val);
    }
    ctx.insert("flags".to_string(), Val::Obj(flags_map));
    // Backwards-compatible numeric aliases for libraries that haven't declared
    // positionals.
    for (i, a) in args.iter().enumerate() {
        let key = format!("arg{}", i);
        ctx.entry(key).or_insert_with(|| Val::Str(a.clone()));
    }

    // Eval the body.
    let mut ev = InnerEvaluator::new(ctx, host);
    // The command env carries the library handle so `compile script` can reach
    // back into the library to render.
    let lib_path_owned = library_path.to_string();
    ev.on_unknown_call = Some(Rc::new(move |name: &str, cargs: &[Val]| {
        command_dispatch(&lib_path_owned, name, cargs)
    }));
    let empty: BTreeMap<String, CaptureValue> = BTreeMap::new();
    ev.exec(&cmd.body, &empty)
}

/// Port of `commandEnv.dispatch`.
///
/// The fallback for unknown inner calls. Implements command-only primitives that
/// need access to the library:
///   * `compile SCRIPT_PATH` → run the library on a script
///   * `call CMD_NAME arg…`  → invoke another command of THIS library
fn command_dispatch(
    library_path: &str,
    name: &str,
    args: &[Val],
) -> Result<(Option<Val>, bool), CapyError> {
    match name {
        "compile" => {
            // compile SCRIPT_PATH → output string.
            if args.len() != 1 {
                return Err(CapyError::msg("compile expects 1 arg (script path)"));
            }
            let mut script_path = args[0].format_v();
            if !gopath::is_abs(&script_path) {
                let cwd = std::env::current_dir()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                script_path = gopath::join(&[&cwd, &script_path]);
            }
            let out = run::run(library_path, &script_path)?;
            Ok((Some(Val::Str(out)), true))
        }
        "call" => {
            // call CMD_NAME arg… — invoke another command of the same library.
            // Returns "" so it works as a statement or as a value expression.
            if args.is_empty() {
                return Err(CapyError::msg("call expects at least 1 arg (command name)"));
            }
            let target = args[0].format_v();
            let call_args: Vec<String> = args[1..].iter().map(|a| a.format_v()).collect();
            run_command(library_path, &target, &call_args)?;
            Ok((Some(Val::Str(String::new())), true))
        }
        _ => Ok((None, false)),
    }
}

/// Port of `shouldWarnUntrusted`.
///
/// Returns true when `library_path` is NOT a descendant of any directory in
/// `CAPY_LIBS` (or the default fallbacks). Suppressed via `CAPY_TRUST=1`.
fn should_warn_untrusted(library_path: &str) -> bool {
    if std::env::var("CAPY_TRUST").unwrap_or_default() == "1" {
        return false;
    }
    let abs = abs_path(library_path);
    for dir in lib_search_path_runtime() {
        let da = abs_path(&dir);
        // Trust if library_path is under any search-path dir.
        if let Some(rel) = rel_path(&da, &abs) {
            if !rel.starts_with("..") {
                return false;
            }
        }
    }
    true
}

/// Port of `libSearchPathRuntime`.
fn lib_search_path_runtime() -> Vec<String> {
    let env = std::env::var("CAPY_LIBS").unwrap_or_default();
    if !env.is_empty() {
        let sep = if is_windows() { ';' } else { ':' };
        return env
            .split(sep)
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
    }
    // CWD is part of the default search path: a project's local library is
    // trusted without requiring CAPY_LIBS to be set.
    let mut out: Vec<String> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.to_string_lossy().into_owned());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    out.push(gopath::join(&[&home, ".config", "capy", "libs"]));
    out.push(gopath::join(&[&home, ".capy", "libs"]));
    out.push(gopath::join(&[&home, "Library", "Application Support", "Capy", "libs"]));
    out
}

/// Port of `isWindows` — note Go checks the `OS` env var, not the build target.
fn is_windows() -> bool {
    std::env::var("OS").unwrap_or_default().to_lowercase().contains("windows")
}

/// Port of `sortedCommandNames`.
fn sorted_command_names(lib: &Library) -> Vec<String> {
    // `BTreeMap` keys already iterate sorted, matching Go's insertion sort.
    lib.commands.keys().cloned().collect()
}

/// Port of `nameOrPath`.
fn name_or_path(lib: &Library, library_path: &str) -> String {
    if !lib.lib_name.is_empty() {
        return lib.lib_name.clone();
    }
    let base = gopath::base(library_path);
    let ext = gopath::ext(library_path);
    base.strip_suffix(&ext).unwrap_or(&base).to_string()
}

/// Port of `anyList` — Go widens `[]string` to `[]any` here.
fn any_list(ss: &[String]) -> Val {
    Val::List(ss.iter().map(|s| Val::Str(s.clone())).collect())
}

/// Port of `filepath.Abs`.
fn abs_path(path: &str) -> String {
    if gopath::is_abs(path) {
        return gopath::clean(path);
    }
    match std::env::current_dir() {
        Ok(cwd) => gopath::join(&[&cwd.to_string_lossy(), path]),
        Err(_) => gopath::clean(path),
    }
}

/// Port of `filepath.Rel`, restricted to the absolute-path case this uses.
fn rel_path(base: &str, target: &str) -> Option<String> {
    let b = gopath::clean(base);
    let t = gopath::clean(target);
    if b == t {
        return Some(".".to_string());
    }
    let bs: Vec<&str> = b.split('/').filter(|s| !s.is_empty()).collect();
    let ts: Vec<&str> = t.split('/').filter(|s| !s.is_empty()).collect();
    let mut i = 0usize;
    while i < bs.len() && i < ts.len() && bs[i] == ts[i] {
        i += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in i..bs.len() {
        parts.push("..".to_string());
    }
    for s in &ts[i..] {
        parts.push(s.to_string());
    }
    if parts.is_empty() {
        return Some(".".to_string());
    }
    Some(parts.join("/"))
}
