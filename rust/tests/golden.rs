//! Port of `cmd/capy/golden_test.go`.
//!
//! Walks `samples/` and runs each (lib.capy, script) pair through the engine,
//! comparing against `<base>.expected.txt` (success) or
//! `<base>.expected-error.txt` (error). This is the acceptance test: it makes the
//! crate self-verifying without needing the Go implementation present.
//!
//! PROP-2026-0002 R12 adds a third, OPTIONAL golden kind:
//! `<base>.expected-ast.txt` holds what `capy ast <lib> <script> 2>&1` prints —
//! the recovered tree plus its diagnostics. It asserts a *different command* on
//! the same input, so a script may carry both an `.expected-error.txt` (what
//! `run` refuses) and an `.expected-ast.txt` (what `parse` recovers); each is
//! counted as its own case. Without this the 0.22.0 parser surface —
//! diagnostics, recovery, `capy ast` — has no coverage in the sample corpus at
//! all.

use capy_core::capy::Library;
use capy_core::domain::ast_text;
use capy_core::orchestrator::run;
use std::path::{Path, PathBuf};

/// Port of `findSamplesRoot` — walk up until a `samples/` directory appears.
fn find_samples_root() -> PathBuf {
    let mut cur = std::env::current_dir().expect("cwd");
    loop {
        let p = cur.join("samples");
        if p.is_dir() {
            return p;
        }
        match cur.parent() {
            Some(parent) => cur = parent.to_path_buf(),
            None => panic!("samples dir not found"),
        }
    }
}

/// Port of `normalize` — strip a UTF-8 BOM and CRLF so the comparison doesn't
/// fail on invisible byte differences.
fn normalize(s: &str) -> String {
    s.trim_start_matches('\u{feff}').replace("\r\n", "\n")
}

/// Collects `(name, lib, script)` for every golden-bearing sample, using the
/// same pairing rule as the Go test: `lib.capy` plus every other `*.capy`.
fn sample_pairs(root: &Path) -> Vec<(String, PathBuf, PathBuf)> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .expect("read samples")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let mut out = Vec::new();
    for dir in dirs {
        let lib = dir.join("lib.capy");
        if !lib.exists() {
            continue;
        }
        let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
            .expect("read sample dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "capy").unwrap_or(false))
            .filter(|p| p.file_name().map(|x| x != "lib.capy").unwrap_or(false))
            .collect();
        scripts.sort();
        for script in scripts {
            let base = script.file_stem().unwrap().to_string_lossy().into_owned();
            let name =
                format!("{}/{}", dir.file_name().unwrap().to_string_lossy(), base);
            out.push((name, lib.clone(), script));
        }
    }
    out
}

#[test]
fn golden_samples_match() {
    let root = find_samples_root();
    let pairs = sample_pairs(&root);
    assert!(!pairs.is_empty(), "no sample pairs discovered under {:?}", root);

    let mut pass = 0usize;
    let mut skip = 0usize;
    let mut updated = 0usize;
    let mut failures: Vec<String> = Vec::new();

    // Port of the Go suite's `-update` flag. `cargo test` has no clean way to
    // pass a custom flag through to a test binary, so this is an env var:
    //
    //     CAPY_UPDATE_GOLDENS=1 cargo test --test golden
    //
    // Like the Go original it only refreshes goldens that ALREADY EXIST —
    // create an empty placeholder first when adding a brand-new sample, so a
    // typo in a filename can't silently mint a golden that asserts nothing.
    let update = std::env::var("CAPY_UPDATE_GOLDENS").unwrap_or_default() == "1";

    for (name, lib, script) in &pairs {
        let dir = script.parent().unwrap();
        let base = script.file_stem().unwrap().to_string_lossy().into_owned();
        let ok_path = dir.join(format!("{}.expected.txt", base));
        let err_path = dir.join(format!("{}.expected-error.txt", base));

        // Optional AST golden — an ADDITIONAL assertion, not an alternative to
        // the run-based ones below. Rendered by `capy_core::domain::ast_text`,
        // the same module `capy ast` prints through, so this golden can never
        // drift from what a user sees in a terminal.
        let ast_path = dir.join(format!("{}.expected-ast.txt", base));
        if ast_path.exists() {
            match Library::from_file(&lib.to_string_lossy()) {
                Err(e) => failures.push(format!("{name}: AST golden — library failed to load: {e}")),
                Ok(library) => match std::fs::read_to_string(script) {
                    Err(e) => failures.push(format!("{name}: AST golden — script unreadable: {e}")),
                    Ok(src) => {
                        let got = ast_text::render(&library.parse(&src));
                        let want = std::fs::read_to_string(&ast_path).unwrap();
                        let (g, w) = (normalize(&got), normalize(&want));
                        if g == w {
                            pass += 1;
                        } else if update {
                            std::fs::write(&ast_path, &got).unwrap();
                            updated += 1;
                        } else {
                            failures
                                .push(format!("{name}: AST mismatch\n{}", first_diff(&w, &g)));
                        }
                    }
                },
            }
        }

        let result = run::run(&lib.to_string_lossy(), &script.to_string_lossy());

        if err_path.exists() {
            let want = std::fs::read_to_string(&err_path).unwrap().trim().to_string();
            match result {
                Ok(out) => failures.push(format!(
                    "{name}: expected error per golden, got success ({} bytes)",
                    out.len()
                )),
                Err(e) => {
                    let got = e.to_string().trim().to_string();
                    if got == want {
                        pass += 1;
                    } else if update {
                        std::fs::write(&err_path, format!("{got}\n")).unwrap();
                        updated += 1;
                    } else {
                        failures.push(format!(
                            "{name}: error mismatch\n    want: {want:?}\n    got:  {got:?}"
                        ));
                    }
                }
            }
        } else if ok_path.exists() {
            let want = std::fs::read_to_string(&ok_path).unwrap();
            match result {
                Err(e) => {
                    failures.push(format!("{name}: expected success, got error: {e}"))
                }
                Ok(out) => {
                    let (g, w) = (normalize(&out), normalize(&want));
                    if g == w {
                        pass += 1;
                    } else if update {
                        std::fs::write(&ok_path, &out).unwrap();
                        updated += 1;
                    } else {
                        failures.push(format!("{name}: output mismatch\n{}", first_diff(&w, &g)));
                    }
                }
            }
        } else {
            // Matches the Go test's `t.Skipf("no golden file for %s")`.
            skip += 1;
        }
    }

    if update {
        eprintln!("goldens: {pass} already matched, {updated} REWRITTEN, {skip} skipped (no golden file)");
    }
    eprintln!("goldens: {pass} passed, {skip} skipped (no golden file), {} failed", failures.len());
    assert!(
        failures.is_empty(),
        "{} golden failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    // Guard against the suite silently shrinking to nothing.
    assert!(pass >= 100, "only {pass} goldens ran — did sample discovery break?");
}

/// Renders the first differing line, the way the Go test's `diffSummary` does.
fn first_diff(want: &str, got: &str) -> String {
    let wl: Vec<&str> = want.split('\n').collect();
    let gl: Vec<&str> = got.split('\n').collect();
    for i in 0..wl.len().max(gl.len()) {
        let a = wl.get(i).copied().unwrap_or("<missing>");
        let b = gl.get(i).copied().unwrap_or("<missing>");
        if a != b {
            return format!("    line {}\n    want: {a:?}\n    got:  {b:?}", i + 1);
        }
    }
    "    (no line-level difference found)".to_string()
}

/// The multi-file path the golden harness structurally cannot express: every
/// sample must at least run without error through `run_multi`.
#[test]
fn multi_file_samples_render() {
    let root = find_samples_root();
    let mut rendered_files = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for (name, lib, script) in sample_pairs(&root) {
        match run::run_multi(&lib.to_string_lossy(), &script.to_string_lossy()) {
            // A sample may legitimately fail (e.g. a `mismatch.capy` fixture);
            // only assert on the ones with a success golden.
            Err(_) => {
                let dir = script.parent().unwrap();
                let base = script.file_stem().unwrap().to_string_lossy().into_owned();
                if dir.join(format!("{}.expected.txt", base)).exists() {
                    failures.push(name);
                }
            }
            Ok((_, files)) => rendered_files += files.len(),
        }
    }
    assert!(failures.is_empty(), "run_multi failed for: {failures:?}");
    assert!(rendered_files > 0, "no multi-file outputs exercised");
    eprintln!("multi-file: {rendered_files} generated files rendered");
}
