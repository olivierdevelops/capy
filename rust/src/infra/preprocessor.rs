//! Port of `infra/preprocessor.go`.
//!
//! Walks a source string line by line and expands any inclusion directives the
//! LIBRARY has opted into. Each directive name (e.g. `"@import"`, `"@include"`)
//! when seen at the start of a source line is replaced by the contents of the
//! referenced file (recursively).
//!
//! The engine ships with NO default directives — pass an empty `directives`
//! slice and this is a no-op. This keeps Capy's "zero predefined grammar"
//! promise intact: even universal-looking constructs like `@import` are opt-in
//! per library.
//!
//! Reads go through the supplied [`Host`], NOT the filesystem directly. That
//! matters for the sandbox: `capy::Library` defaults to `NoOpHost`, so an
//! embedded caller (or the wasm build) that never granted filesystem access
//! cannot be made to read arbitrary files by a library declaring `@import`.
//!
//! Paths are resolved against `dir` and then made ABSOLUTE, so the host reads
//! exactly that path instead of re-prefixing its own `base_dir`.

use crate::domain::host::Host;
use crate::gopath;
use std::collections::BTreeSet;

/// Port of `Preprocess`.
pub fn preprocess(
    source: &str,
    dir: &str,
    directives: &[String],
    host: &dyn Host,
) -> Result<String, String> {
    if directives.is_empty() {
        return Ok(source.to_string());
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    preprocess_inner(source, dir, directives, &mut visited, host)
}

fn preprocess_inner(
    source: &str,
    dir: &str,
    directives: &[String],
    visited: &mut BTreeSet<String>,
    host: &dyn Host,
) -> Result<String, String> {
    let mut out = String::new();
    let lines: Vec<&str> = source.split('\n').collect();
    for (i, line) in lines.iter().enumerate() {
        // Find leading indent so imported content can be re-indented to match
        // the `@import` line's column. This is what authors expect: imports nest
        // inside whatever block they appear in.
        let lb = line.as_bytes();
        let mut indent_len = 0usize;
        while indent_len < lb.len() && (lb[indent_len] == b' ' || lb[indent_len] == b'\t') {
            indent_len += 1;
        }
        let trimmed = &line[indent_len..];
        if let Some((d, path)) = match_import(trimmed, directives) {
            let abs_path = resolve_path(&path, dir);
            if visited.contains(&abs_path) {
                return Err(format!("line {}: import cycle: {}", i + 1, abs_path));
            }
            let b = host.read_file(&abs_path).map_err(|e| {
                format!("line {}: {} {}: {}", i + 1, d, crate::gofmt::quote(&path), e)
            })?;
            visited.insert(abs_path.clone());
            let expanded =
                preprocess_inner(&b, &gopath::dir(&abs_path), directives, visited, host)?;
            visited.remove(&abs_path);
            let indent = &line[..indent_len];
            for (j, l) in expanded.trim_end_matches('\n').split('\n').enumerate() {
                if j > 0 {
                    out.push('\n');
                }
                if l.trim().is_empty() {
                    continue; // keep blank lines blank
                }
                out.push_str(indent);
                out.push_str(l);
            }
            if i < lines.len() - 1 {
                out.push('\n');
            }
            continue;
        }
        out.push_str(line);
        if i < lines.len() - 1 {
            out.push('\n');
        }
    }
    Ok(out)
}

/// Port of `matchImport`.
///
/// Recognises any directive in `directives` (e.g. `@import`, `@include`, or
/// library-chosen names like `@use`) at the start of a line, followed by a
/// quoted path.
fn match_import(line: &str, directives: &[String]) -> Option<(String, String)> {
    for d in directives {
        if !line.starts_with(d.as_str()) {
            continue;
        }
        let rest = line[d.len()..].trim();
        let rb = rest.as_bytes();
        if rb.len() < 2 || rb[0] != b'"' {
            continue;
        }
        // Find the closing quote.
        let end = match rest[1..].find('"') {
            None => continue,
            Some(e) => e,
        };
        let path = rest[1..1 + end].to_string();
        // Allow a trailing comment after the directive.
        let after = rest[2 + end..].trim();
        if !after.is_empty() && !after.starts_with('#') {
            continue;
        }
        return Some((d.clone(), path));
    }
    None
}

/// Port of `resolvePath`. The result is absolute so the host reads exactly this
/// path rather than resolving it against its own `base_dir` a second time.
fn resolve_path(path: &str, dir: &str) -> String {
    if gopath::is_abs(path) {
        return gopath::clean(path);
    }
    let joined = gopath::join(&[dir, path]);
    if gopath::is_abs(&joined) {
        return joined;
    }
    match std::env::current_dir() {
        Ok(cwd) => gopath::join(&[&cwd.to_string_lossy(), &joined]),
        Err(_) => joined,
    }
}
