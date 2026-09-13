//! Port of `cmd/capy/cmd_watch.go`.
//!
//! Polls a set of files for mtime changes and re-runs a chosen command whenever
//! any of them updates. Polling (vs. an OS notification API) keeps the dependency
//! list small and behaves the same on every platform.

use super::cmd_run::library_has_command;
use super::lib_path::resolve_lib;
use capy_core::gopath;
use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Port of `cmdWatch`.
pub fn cmd_watch(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: capy watch <library> [<script> | <command> [args...]]".to_string());
    }

    // Resolve the library — by name (CAPY_LIBS) or by direct path.
    let first = &args[0];
    let rest = &args[1..];
    let (lib_path, lib_dir) = match resolve_lib(first) {
        Ok(p) => {
            let d = gopath::dir(&p);
            (p, d)
        }
        Err(_) => {
            if std::fs::metadata(first).is_ok() {
                (first.clone(), gopath::dir(first))
            } else {
                return Err(format!("library {:?} not found (CAPY_LIBS or path)", first));
            }
        }
    };

    // Determine the watched paths + the command to execute.
    let mut watched: Vec<String> = Vec::new();
    let argv: Vec<String>;
    if library_has_command(&lib_path, "run") {
        // Library has commands declared → assume the user wants
        // `capy <lib> <cmd> [args]`. If the first remaining arg names a declared
        // command, use it; otherwise fall back to `run` with all of `rest`.
        let mut cmd_name = "run".to_string();
        let mut cmd_args: &[String] = rest;
        if !rest.is_empty() && library_has_command(&lib_path, &rest[0]) {
            cmd_name = rest[0].clone();
            cmd_args = &rest[1..];
        }
        let mut v = vec![first.clone(), cmd_name];
        v.extend_from_slice(cmd_args);
        argv = v;
        // Watch every script path mentioned in cmd_args that exists.
        for a in cmd_args {
            if let Ok(st) = std::fs::metadata(a) {
                if !st.is_dir() {
                    watched.push(a.clone());
                }
            }
        }
    } else {
        // Legacy form: `capy run <library> <script>`.
        if rest.is_empty() {
            return Err("usage: capy watch <library> <script>".to_string());
        }
        let mut v = vec!["run".to_string(), lib_path.clone()];
        v.extend_from_slice(rest);
        argv = v;
        for a in rest {
            if let Ok(st) = std::fs::metadata(a) {
                if !st.is_dir() {
                    watched.push(a.clone());
                }
            }
        }
    }

    // Always watch every .capy file in the library's directory.
    if !lib_dir.is_empty() {
        walk_capy_files(&lib_dir, &mut watched);
    }

    if watched.is_empty() {
        return Err("watch: no files to watch".to_string());
    }

    eprintln!(
        "👀 watching {} file(s); re-runs on save (Ctrl-C to exit)",
        watched.len()
    );
    for w in &watched {
        eprintln!("    {}", w);
    }

    // First run.
    run_once(&argv);
    // Poll loop.
    let mut last = snapshot_mtimes(&watched);
    loop {
        std::thread::sleep(Duration::from_millis(250));
        let cur = snapshot_mtimes(&watched);
        if cur != last {
            last = cur;
            eprintln!("\n--- change detected — re-running ---");
            run_once(&argv);
        }
    }
}

/// Port of the `filepath.WalkDir` call that collects `.capy` files.
fn walk_capy_files(dir: &str, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Err(_) => return,
        Ok(e) => e,
    };
    for e in entries.filter_map(|e| e.ok()) {
        let p = e.path();
        let ps = p.to_string_lossy().into_owned();
        if p.is_dir() {
            walk_capy_files(&ps, out);
            continue;
        }
        if ps.ends_with(".capy") {
            out.push(ps);
        }
    }
}

/// Port of `runOnce` — re-invokes this same binary so the watch loop stays a thin
/// supervisor. Non-zero exits are ignored: the next change might fix it.
fn run_once(argv: &[String]) {
    let self_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "capy".to_string());
    let _ = Command::new(self_exe).args(argv).status();
}

/// Port of `snapshotMtimes`. A `BTreeMap` compares by value, which is what
/// `sameMtimes` did by hand.
fn snapshot_mtimes(paths: &[String]) -> BTreeMap<String, SystemTime> {
    let mut out = BTreeMap::new();
    for p in paths {
        if let Ok(st) = std::fs::metadata(p) {
            if let Ok(m) = st.modified() {
                out.insert(p.clone(), m);
            }
        }
    }
    out
}
