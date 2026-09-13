//! Port of `infra/preprocessor_test.go`.

use capy_core::domain::host::Host;
use capy_core::infra::os_host::OsHost;
use capy_core::infra::preprocessor::preprocess;
use std::path::PathBuf;

/// Stand-in for Go's `t.TempDir()` — a fresh directory removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let base = std::env::temp_dir().join(format!(
            "capy-pp-{}-{}-{:?}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("create temp dir");
        TempDir(base)
    }
    fn path(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
    /// Port of the test helper `writeTmp`.
    fn write(&self, name: &str, content: &str) {
        let p = self.0.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir parent");
        }
        std::fs::write(&p, content.as_bytes()).expect("write temp file");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Tests grant real filesystem access explicitly; the sandbox default is
/// `NoOpHost`, which is what the new `sandbox_blocks_import` test covers.
fn host() -> OsHost {
    OsHost::default()
}

fn directives() -> Vec<String> {
    vec!["@import".to_string(), "@include".to_string()]
}

#[test]
fn no_imports() {
    let got = preprocess("hello\nworld\n", "/tmp", &directives(), &host()).unwrap();
    assert_eq!(got, "hello\nworld\n", "expected passthrough");
}

#[test]
fn basic_import() {
    let d = TempDir::new("basic");
    d.write("shared.capy", "shared line one\nshared line two\n");
    let main = "before\n@import \"shared.capy\"\nafter\n";
    let got = preprocess(main, &d.path(), &directives(), &host()).unwrap();
    assert_eq!(got, "before\nshared line one\nshared line two\nafter\n");
}

#[test]
fn indentation_preserved() {
    let d = TempDir::new("indent");
    d.write("inner.capy", "first\nsecond\n");
    let main = "outer\n    @import \"inner.capy\"\n";
    let got = preprocess(main, &d.path(), &directives(), &host()).unwrap();
    // Each non-blank line of the import should carry 4 leading spaces.
    for line in got.split('\n') {
        if line.is_empty() || line == "outer" {
            continue;
        }
        assert!(line.starts_with("    "), "expected 4-space indent, got {line:?}");
    }
}

#[test]
fn nested_import() {
    let d = TempDir::new("nested");
    d.write("a.capy", "from-a\n@import \"b.capy\"\nback-from-a\n");
    d.write("b.capy", "from-b\n");
    let got = preprocess("@import \"a.capy\"\n", &d.path(), &directives(), &host()).unwrap();
    assert!(got.contains("from-a"), "missing from-a:\n{got}");
    assert!(got.contains("from-b"), "missing from-b:\n{got}");
    assert!(got.contains("back-from-a"), "missing back-from-a:\n{got}");
}

#[test]
fn detects_cycle() {
    let d = TempDir::new("cycle");
    d.write("a.capy", "@import \"b.capy\"\n");
    d.write("b.capy", "@import \"a.capy\"\n");
    let err = preprocess("@import \"a.capy\"\n", &d.path(), &directives(), &host())
        .expect_err("expected a cycle error");
    assert!(err.contains("cycle"), "expected cycle error, got {err:?}");
}

#[test]
fn missing_file() {
    let d = TempDir::new("missing");
    preprocess("@import \"nope.capy\"\n", &d.path(), &directives(), &host())
        .expect_err("expected error for missing file");
}

#[test]
fn include_is_synonym() {
    let d = TempDir::new("synonym");
    d.write("x.capy", "included\n");
    let got = preprocess("@include \"x.capy\"\n", &d.path(), &directives(), &host()).unwrap();
    assert!(got.contains("included"), "@include didn't expand: {got:?}");
}

#[test]
fn lines_without_import_are_untouched() {
    // `@import broken` (no quoted path) must NOT be treated as an import.
    let d = TempDir::new("untouched");
    let src = "@import broken\nother line\n";
    let got = preprocess(src, &d.path(), &directives(), &host()).unwrap();
    assert_eq!(got, src, "malformed import should be passthrough");
}

#[test]
fn no_directives_is_noop() {
    // With no library-declared directives, every `@import` line passes through
    // untouched — even when the file exists.
    let d = TempDir::new("nodirs");
    d.write("x.capy", "should-not-appear\n");
    let src = "@import \"x.capy\"\nkept\n";
    let got = preprocess(src, &d.path(), &[], &host()).unwrap();
    assert_eq!(got, src, "expected source unchanged when no directives declared");
}

#[test]
fn unknown_directive_ignored() {
    // Library declares only @use; @import lines stay as plain text.
    let d = TempDir::new("unknown");
    d.write("x.capy", "inlined\n");
    let src = "@import \"x.capy\"\n@use \"x.capy\"\n";
    let got = preprocess(src, &d.path(), &["@use".to_string()], &host()).unwrap();
    assert!(
        got.contains("@import \"x.capy\""),
        "@import should be passthrough when not declared; got {got:?}"
    );
    assert!(got.contains("inlined"), "@use should expand; got {got:?}");
}

/// The sandbox must refuse `@import` entirely — this is the hole the Host
/// routing closes. Previously the preprocessor read the filesystem directly, so
/// an embedded caller with NoOpHost could still be made to read arbitrary files.
#[test]
fn sandbox_blocks_import() {
    use capy_core::domain::host::NoOpHost;
    let d = TempDir::new("sandbox");
    d.write("secret.capy", "SHOULD NOT BE READ\n");
    let err = preprocess("@import \"secret.capy\"\n", &d.path(), &directives(), &NoOpHost)
        .expect_err("NoOpHost must refuse the read");
    assert!(
        !err.contains("SHOULD NOT BE READ"),
        "the file's contents must never surface"
    );
    assert!(
        err.contains("no filesystem access"),
        "expected the sandbox refusal, got {err:?}"
    );
}
