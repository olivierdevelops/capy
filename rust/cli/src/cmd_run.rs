//! Port of `cmd/capy/cmd_run.go`.

use super::flags::{self, Spec};
use capy_core::domain::errors::format_with_source;
use capy_core::gopath;
use capy_core::orchestrator::features::{make_lexer, make_library_loader};
use capy_core::orchestrator::{commands, run};
use std::collections::BTreeMap;
use std::io::Write;

const RUN_SPEC: Spec = Spec {
    bools: &["debug", "no-color"],
    strings: &["out", "out-dir", "zip", "lib"],
};

/// Port of `cmdRun`.
pub fn cmd_run(args: &[String]) -> Result<(), String> {
    let fs = flags::parse(&RUN_SPEC, args)?;
    let pos = &fs.positionals;
    let legacy_lib = fs.str("lib");

    let lib_path: String;
    let script_path: String;
    let mut user_args: Vec<String> = Vec::new();

    if !legacy_lib.is_empty() && !pos.is_empty() {
        lib_path = legacy_lib.to_string();
        script_path = pos[0].clone();
        user_args = pos[1..].to_vec();
    } else if pos.len() >= 2 {
        lib_path = pos[0].clone();
        script_path = pos[1].clone();
        // Anything beyond <library> <script> is a positional arg the library can
        // consume via the inner `arg N` primitive.
        user_args = pos[2..].to_vec();
    } else if pos.len() == 1 {
        // One-arg form: `capy run <script.ext>` — auto-resolve the library.
        script_path = pos[0].clone();
        lib_path = auto_resolve_lib(&script_path)?;
        // If the resolved library declares a `command "run"`, dispatch through it
        // so users get the library's full `run` behaviour — including
        // post-render side-effects like opening a browser.
        if library_has_command(&lib_path, "run") {
            return commands::run_command(&lib_path, "run", &[script_path])
                .map_err(|e| e.to_string());
        }
    } else {
        return Err(
            "usage: capy run [--out-dir DIR | --zip ARCHIVE.zip] [<library>] <script> [args...]"
                .to_string(),
        );
    }

    // Read the source for nice error formatting.
    let src = std::fs::read_to_string(&script_path).unwrap_or_default();

    let (output, files) = run::run_multi_with_args(&lib_path, &script_path, &user_args)
        .map_err(|e| format_with_source(&e, &src))?;

    // Multi-file output: each file is either written under --out-dir or bundled
    // into --zip. Exactly one of those should be set.
    if !files.is_empty() {
        let zip_path = fs.str("zip");
        let out_dir = fs.str("out-dir");
        if !zip_path.is_empty() && !out_dir.is_empty() {
            return Err("use --zip OR --out-dir, not both".to_string());
        }
        if !zip_path.is_empty() {
            return write_zip(zip_path, &files);
        }
        if !out_dir.is_empty() {
            return write_tree(out_dir, &files);
        }
        return Err(format!(
            "library declared {} `file \"...\":` block(s); pass --out-dir DIR or --zip ARCHIVE.zip to write them",
            files.len()
        ));
    }

    // Single output. Precedence: --out flag → library's `output_file` → stdout.
    let out_flag = fs.str("out");
    if !out_flag.is_empty() {
        return std::fs::write(out_flag, output.as_bytes())
            .map_err(|e| gopath::io_error("open", out_flag, &e));
    }
    let lib_out = peek_output_file(&lib_path);
    if !lib_out.is_empty() {
        let full = if gopath::is_abs(&lib_out) {
            lib_out.clone()
        } else {
            gopath::join(&[&gopath::dir(&script_path), &lib_out])
        };
        return std::fs::write(&full, output.as_bytes())
            .map_err(|e| gopath::io_error("open", &full, &e));
    }
    print!("{}", output);
    let _ = std::io::stdout().flush();
    Ok(())
}

/// Port of `peekOutputFile` — reloads the library just to read `output_file`.
fn peek_output_file(lib_path: &str) -> String {
    match make_library_loader::load_library(lib_path, make_lexer::tokenize) {
        Ok(lib) => lib.output_file,
        Err(_) => String::new(),
    }
}

/// Port of `libraryHasCommand` (defined in `cmd_watch.go`) — a cheap text scan
/// that avoids a full library load just to check.
pub fn library_has_command(lib_path: &str, name: &str) -> bool {
    match std::fs::read_to_string(lib_path) {
        Ok(s) => s.contains(&format!("command \"{}\"", name)),
        Err(_) => false,
    }
}

/// Port of `writeTree`.
///
/// Writes every (path, content) pair under `root`, creating subdirectories as
/// needed. `BTreeMap` iteration keeps the logging order deterministic, matching
/// Go's explicit sort.
fn write_tree(root: &str, files: &BTreeMap<String, String>) -> Result<(), String> {
    for (rel, content) in files {
        let full = gopath::join(&[root, rel]);
        let parent = gopath::dir(&full);
        std::fs::create_dir_all(&parent)
            .map_err(|e| format!("mkdir {}: {}", parent, gopath::io_error("mkdir", &parent, &e)))?;
        std::fs::write(&full, content.as_bytes())
            .map_err(|e| format!("write {}: {}", full, gopath::io_error("open", &full, &e)))?;
        eprintln!("wrote {} ({} bytes)", full, content.len());
    }
    Ok(())
}

/// Port of `writeZip`.
///
/// Bundles every (path, content) into one zip archive. Paths inside the archive
/// use forward slashes — POSIX convention, and what Windows tools expect too.
fn write_zip(zip_file: &str, files: &BTreeMap<String, String>) -> Result<(), String> {
    let parent = gopath::dir(zip_file);
    std::fs::create_dir_all(&parent)
        .map_err(|e| gopath::io_error("mkdir", &parent, &e))?;
    let f = std::fs::File::create(zip_file)
        .map_err(|e| gopath::io_error("open", zip_file, &e))?;
    let mut zw = zip::ZipWriter::new(f);
    // Go's archive/zip defaults to Deflate.
    let opts: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (rel, content) in files {
        let zip_rel = rel.replace('\\', "/");
        zw.start_file(zip_rel.clone(), opts)
            .map_err(|e| format!("zip entry {}: {}", zip_rel, e))?;
        zw.write_all(content.as_bytes())
            .map_err(|e| format!("zip write {}: {}", zip_rel, e))?;
    }
    let f = zw.finish().map_err(|e| e.to_string())?;
    let size = f.metadata().map(|m| m.len()).unwrap_or(0);
    eprintln!("wrote {} ({} entries, {} bytes)", zip_file, files.len(), size);
    Ok(())
}

/// Port of `autoResolveLib`.
///
/// Finds the library to use when the user invokes `capy run` with only a script
/// path. Resolution order, relative to the script's directory:
///
/// 1. `<ext>.capy` — extension-name match (`main.interface` → `interface.capy`)
/// 2. `lib.capy` — conventional default name
fn auto_resolve_lib(script_path: &str) -> Result<String, String> {
    let dir = gopath::dir(script_path);
    let ext = gopath::ext(script_path);
    let ext = ext.strip_prefix('.').unwrap_or(&ext);
    let mut candidates: Vec<String> = Vec::new();
    if !ext.is_empty() {
        candidates.push(gopath::join(&[&dir, &format!("{}.capy", ext)]));
    }
    candidates.push(gopath::join(&[&dir, "lib.capy"]));
    for c in &candidates {
        if std::fs::metadata(c).is_ok() {
            return Ok(c.clone());
        }
    }
    Err(format!(
        "no library found for {:?} (looked for: {})",
        script_path,
        candidates.join(", ")
    ))
}
