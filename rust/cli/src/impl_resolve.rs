//! Port of `cmd/capy/impl_resolve.go`.

use super::lib_path::resolve_lib;
use capy_core::domain::library::Library;
use capy_core::gopath;
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

/// The result of [`resolve_lib_with_impl`].
pub struct Resolved {
    /// Absolute path to the impl's `.capy` file — what the loader actually reads.
    pub impl_path: String,
    /// The manifest path, so callers can show "selected from" in verbose modes.
    pub manifest_path: String,
    pub lib: Library,
}

/// Port of `resolveLibWithImpl`.
///
/// Resolves a library by name (or path) AND picks an implementation. Precedence
/// (highest wins):
///
/// 1. `--impl <name>` CLI flag
/// 2. `CAPY_IMPL_<UPPER-LIB-NAME>` env var
/// 3. `CAPY_IMPL` env var (generic)
/// 4. The manifest's `default` directive
/// 5. If the library declares exactly ONE impl, use it.
///
/// For libraries with no impl declarations, the manifest path IS the impl path.
pub fn resolve_lib_with_impl(lib_name: &str, impl_flag: &str) -> Result<Resolved, String> {
    let manifest_path = resolve_lib(lib_name)?;
    // Load the manifest enough to see its declared impls.
    let mut lib = make_library_loader::load_library(&manifest_path, make_lexer::tokenize)
        .map_err(|e| e.to_string())?;
    if lib.impls.is_empty() {
        // No impls declared — the manifest file IS the library.
        return Ok(Resolved {
            impl_path: manifest_path.clone(),
            manifest_path,
            lib,
        });
    }

    // Pick.
    let mut wanted = impl_flag.to_string();
    if wanted.is_empty() {
        // Env-var per-library.
        if let Ok(v) = std::env::var(format!("CAPY_IMPL_{}", env_upper(&lib.lib_name))) {
            if !v.is_empty() {
                wanted = v;
            }
        }
    }
    if wanted.is_empty() {
        if let Ok(v) = std::env::var("CAPY_IMPL") {
            if !v.is_empty() {
                wanted = v;
            }
        }
    }
    if wanted.is_empty() {
        wanted = lib.default_impl.clone();
    }
    if wanted.is_empty() && lib.impls.len() == 1 {
        // Only one declared — use it.
        wanted = lib.impls.keys().next().cloned().unwrap_or_default();
    }
    if wanted.is_empty() {
        return Err(format!(
            "library {:?} has multiple impls but no default; pass --impl <name> (choices: {})",
            lib_name,
            sorted_impl_keys(&lib).join(", ")
        ));
    }
    let im = match lib.impls.get(&wanted) {
        None => {
            return Err(format!(
                "library {:?} has no impl {:?} (choices: {})",
                lib_name,
                wanted,
                sorted_impl_keys(&lib).join(", ")
            ))
        }
        Some(im) => im.clone(),
    };
    // Resolve the impl file path relative to the manifest's dir.
    let mut impl_path = im.file.clone();
    if !gopath::is_abs(&impl_path) {
        impl_path = gopath::join(&[&gopath::dir(&manifest_path), &impl_path]);
    }
    lib.selected_impl = wanted;
    Ok(Resolved { impl_path, manifest_path, lib })
}

/// Port of `envUpper` — ASCII upper-case, with `-` and space becoming `_`.
fn env_upper(s: &str) -> String {
    s.bytes()
        .map(|c| match c {
            b'a'..=b'z' => (c - 32) as char,
            b'-' | b' ' => '_',
            _ => c as char,
        })
        .collect()
}

/// Port of `sortedImplKeys` — `BTreeMap` already iterates sorted.
pub fn sorted_impl_keys(lib: &Library) -> Vec<String> {
    lib.impls.keys().cloned().collect()
}
