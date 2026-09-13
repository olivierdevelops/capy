//! Port of `cmd/capy/lib_path.go`.

use capy_core::gopath;
use std::collections::BTreeMap;

/// Port of `libSearchPath`.
///
/// Returns the resolved `CAPY_LIBS` list. When the env var is set it overrides
/// everything. When unset, the current working directory is the first entry,
/// followed by XDG-style per-platform defaults — so a local `interface.capy` next
/// to your script is discoverable without any environment setup.
pub fn lib_search_path() -> Vec<String> {
    if let Ok(env) = std::env::var("CAPY_LIBS") {
        if !env.is_empty() {
            let sep = if cfg!(windows) { ';' } else { ':' };
            return env
                .split(sep)
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
        }
    }
    // Defaults. CWD goes first so a project's local library always wins over a
    // globally-installed one of the same name.
    let mut out: Vec<String> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.to_string_lossy().into_owned());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    if cfg!(target_os = "macos") {
        out.push(gopath::join(&[&home, "Library", "Application Support", "Capy", "libs"]));
    } else if cfg!(windows) {
        let appdata = match std::env::var("APPDATA") {
            Ok(a) if !a.is_empty() => a,
            _ => gopath::join(&[&home, "AppData", "Roaming"]),
        };
        out.push(gopath::join(&[&appdata, "Capy", "libs"]));
    } else {
        let xdg = match std::env::var("XDG_CONFIG_HOME") {
            Ok(x) if !x.is_empty() => x,
            _ => gopath::join(&[&home, ".config"]),
        };
        out.push(gopath::join(&[&xdg, "capy", "libs"]));
        out.push(gopath::join(&[&home, ".capy", "libs"]));
    }
    out
}

/// Port of `resolveLib`.
///
/// Finds a library by name on the search path, returning the path to the `.capy`
/// file the loader will read. Search rules:
///
/// 1. Direct file: `<name>.capy` in any search dir.
/// 2. Directory form: `<name>/<name>.capy`.
/// 3. Directory + lib.capy: `<name>/lib.capy` (pre-manifest convention).
pub fn resolve_lib(name: &str) -> Result<String, String> {
    for dir in lib_search_path() {
        if let Some(path) = try_resolve(&dir, name) {
            return Ok(path);
        }
    }
    // Fallback: current working directory.
    if let Some(path) = try_resolve(".", name) {
        return Ok(path);
    }
    Err(format!(
        "library {:?} not found on CAPY_LIBS ({})",
        name,
        lib_search_path().join(":")
    ))
}

/// Port of `tryResolve`.
pub fn try_resolve(dir: &str, name: &str) -> Option<String> {
    let candidates = [
        gopath::join(&[dir, &format!("{}.capy", name)]),
        gopath::join(&[dir, name, &format!("{}.capy", name)]),
        gopath::join(&[dir, name, "lib.capy"]),
    ];
    for p in candidates {
        if let Ok(st) = std::fs::metadata(&p) {
            if !st.is_dir() {
                return Some(p);
            }
        }
    }
    None
}

/// Port of `listInstalledLibs`.
///
/// Walks every search path and returns (name → path) for each library found.
/// A `BTreeMap` keeps `capy lib list` output sorted, which Go achieves with an
/// explicit `sort.Strings` at the call site.
pub fn list_installed_libs() -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for dir in lib_search_path() {
        let entries = match std::fs::read_dir(&dir) {
            Err(_) => continue,
            Ok(e) => e,
        };
        for e in entries.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().into_owned();
            let is_dir = e.path().is_dir();
            if is_dir {
                let inner = gopath::join(&[&dir, &name, &format!("{}.capy", name)]);
                if std::fs::metadata(&inner).is_ok() {
                    out.entry(name.clone()).or_insert(inner);
                    continue;
                }
                let inner = gopath::join(&[&dir, &name, "lib.capy"]);
                if std::fs::metadata(&inner).is_ok() {
                    out.entry(name.clone()).or_insert(inner);
                }
                continue;
            }
            if let Some(libname) = name.strip_suffix(".capy") {
                out.entry(libname.to_string())
                    .or_insert_with(|| gopath::join(&[&dir, &name]));
            }
        }
    }
    out
}

/// Port of `libraryNameLooksValid`.
///
/// True when `s` is plausibly a library name (no path separators, no leading
/// dots). Used in dispatch to disambiguate `capy <subcommand>` from `capy <lib>`.
pub fn library_name_looks_valid(s: &str) -> bool {
    if s.is_empty() || s.starts_with('.') || s.starts_with('-') {
        return false;
    }
    !s.chars().any(|c| c == '/' || c == '\\' || c == ' ' || c == ':')
}
