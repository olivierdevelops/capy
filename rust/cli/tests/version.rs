//! PLAN-2026-0004 — `capy version` must identify the build.
//!
//! Every local build used to print `capy dev`, so a stale binary on PATH could not
//! be told from a current one (TRBL-2026-0001). Without a stamped `CAPY_VERSION`
//! the CLI now reports the crate version.

use std::process::Command;

fn run(arg: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_capy")).arg(arg).output().expect("run capy");
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn version_reports_the_crate_version_when_not_stamped() {
    // CI stamps CAPY_VERSION only for release builds; tests build unstamped.
    if option_env!("CAPY_VERSION").is_some() {
        return;
    }
    assert_eq!(run("version"), format!("capy {}", env!("CARGO_PKG_VERSION")));
    assert_ne!(run("version"), "capy dev");
}

#[test]
fn dash_dash_version_agrees_with_version() {
    assert_eq!(run("--version"), run("version"));
}
