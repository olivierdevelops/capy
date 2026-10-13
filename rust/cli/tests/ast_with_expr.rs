//! PLAN-2026-0005 / T-10 — `capy ast --json --with-expr` adds the typed expression and nothing else.

use std::path::PathBuf;
use std::process::{Command, Output};

fn sample(name: &str) -> (PathBuf, PathBuf) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples").join(name);
    (d.join("lib.capy"), d.join("script.capy"))
}

fn ast(args: &[&str]) -> Output {
    let (lib, script) = sample("operator-precedence");
    Command::new(env!("CARGO_BIN_EXE_capy"))
        .arg("ast")
        .arg(&lib)
        .arg(&script)
        .args(args)
        .output()
        .expect("run capy")
}

/// Remove every `"expr": …` member from a pretty-printed document, by dropping its lines. The writer
/// puts each member on its own line(s), so tracking brace depth is enough.
fn strip_expr(doc: &str) -> String {
    let mut out = Vec::new();
    let mut skip_depth: i32 = 0;
    for line in doc.lines() {
        if skip_depth > 0 {
            skip_depth += line.matches(['{', '[']).count() as i32 - line.matches(['}', ']']).count() as i32;
            continue;
        }
        if line.trim_start().starts_with("\"expr\":") {
            skip_depth = line.matches(['{', '[']).count() as i32 - line.matches(['}', ']']).count() as i32;
            // the member before `expr` ends with a comma that the stripped document does not have
            if let Some(last) = out.pop() {
                let last: String = last;
                out.push(last.trim_end_matches(',').to_string());
            }
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

#[test]
fn default_json_has_no_expr_member() {
    let o = ast(&["--json"]);
    assert!(o.status.success());
    let doc = String::from_utf8(o.stdout).unwrap();
    assert!(!doc.contains("\"expr\""), "default document must be unchanged");
}

#[test]
fn with_expr_adds_the_member_and_only_that() {
    let plain = String::from_utf8(ast(&["--json"]).stdout).unwrap();
    let rich = String::from_utf8(ast(&["--json", "--with-expr"]).stdout).unwrap();
    assert!(rich.contains("\"expr\""));
    assert!(rich.contains("\"kind\": \"binary\""), "operator-precedence has binary expressions");
    assert_eq!(strip_expr(&rich).trim(), plain.trim());
}

#[test]
fn with_expr_without_json_is_refused() {
    let o = ast(&["--with-expr"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("--with-expr needs --json"));
}
