//! PLAN-2026-0002 — T-14…T-22, T-25, T-26, T-28.
//!
//! Diagnostics (R14–R18, R26) and recovery (R19–R24).
//!
//! Before this, a failed parse said "no library function matches token X" and
//! stopped — the message named what failed, never what was wanted, pointed at
//! the statement rather than the token, and the user fixed one error per run.

use capy_core::capy::Library;

const FN_LIB: &str = r#"
extension txt

function fn
    arg literal "fn"
    arg capture name ident
    arg literal "("
    arg capture params any
    arg literal ")"
    block_closer end
    write `f`
end

function let
    arg literal "let"
    arg capture name ident
    arg literal "="
    arg capture value any
    write `l`
end

function end
end
"#;

const GREET_LIB: &str = r#"
extension txt

function greet
    arg literal "greet"
    arg capture who any
    write `hi ${who}
`
end
"#;

/// T-14 — the furthest attempt is reported, not the statement's first token.
#[test]
fn furthest_attempt_is_reported() {
    let lib = Library::new(FN_LIB).expect("library compiles");
    let err = lib.run("fn add(x\n    end\n").expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("expected `)`"),
        "must name what was wanted, got: {msg}"
    );
    assert!(
        !msg.contains("no library function matches"),
        "must NOT fall back to the generic message: {msg}"
    );
}

/// T-28 / R26 — the context frame says which shape the matcher was inside.
#[test]
fn diagnostic_carries_the_context_frame() {
    let lib = Library::new(FN_LIB).expect("library compiles");
    let err = lib.run("fn add(x\n    end\n").expect_err("should fail");
    let msg = err.to_string();
    assert!(msg.contains("in `fn`"), "must name the shape: {msg}");
}

/// T-15 — when several shapes die at the same token, the expectations union so
/// the reader sees every alternative rather than whichever was tried last.
#[test]
fn expectations_union_at_a_tie() {
    let lib = Library::new(
        r#"
extension txt

function a
    arg literal "x"
    arg literal "p"
end

function b
    arg literal "x"
    arg literal "q"
end

function c
    arg literal "x"
    arg literal "r"
end
"#,
    )
    .expect("library compiles");
    let err = lib.run("x zzz\n").expect_err("should fail");
    let msg = err.to_string();
    for wanted in ["`p`", "`q`", "`r`"] {
        assert!(msg.contains(wanted), "expected {wanted} in the union: {msg}");
    }
}

/// The furthest report must still name the offending token.
#[test]
fn diagnostic_names_what_was_found() {
    let lib = Library::new(FN_LIB).expect("library compiles");
    let err = lib.run("fn add(x\n    end\n").expect_err("should fail");
    assert!(
        err.to_string().contains("found"),
        "must say what was found: {err}"
    );
}

/// T-18 / R20 — recovery reports every broken statement, not just the first.
#[test]
fn recovery_reports_every_broken_statement() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let r = lib.parse("greet a\nbogus one\ngreet b\nalso bad\ngreet c\n");
    assert_eq!(r.tree.stmts.len(), 3, "the three good statements parsed");
    assert_eq!(r.diagnostics.len(), 2, "both mistakes reported in one run");
    assert_eq!(r.tree.errors.len(), 2, "both skipped regions recorded");
    assert!(!r.is_clean());
}

/// T-21 / R22 — statements AFTER a broken one are intact. This is what lets an
/// editor keep working on a buffer that is mid-edit.
#[test]
fn statements_after_a_failure_still_parse() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let r = lib.parse("bogus\ngreet after\n");
    assert_eq!(r.tree.stmts.len(), 1, "the statement after the error parsed");
    assert_eq!(r.tree.stmts[0].func, "greet");
    assert_eq!(r.tree.stmts[0].captures["who"].text, "after");
}

/// R22 — `Block.stmts` keeps its type, so an existing walker still compiles and
/// sees a correct (if incomplete) statement list. Errors are opt-in.
#[test]
fn an_old_walker_still_sees_only_statements() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let r = lib.parse("greet a\nbogus\ngreet b\n");
    // Exactly what a consumer written before recovery existed would do.
    let names: Vec<&str> = r.tree.stmts.iter().map(|s| s.func.as_str()).collect();
    assert_eq!(names, vec!["greet", "greet"], "a degraded but correct view");
}

/// T-20 / R21 — a missing delimiter must not swallow the rest of the file.
/// Delimiter balance is checked before the statement boundary for this reason.
#[test]
fn a_missing_delimiter_does_not_eat_the_file() {
    let lib = Library::new(FN_LIB).expect("library compiles");
    let mut src = String::from("fn add(x\n");
    for i in 0..30 {
        src.push_str(&format!("let v{i} = {i}\n"));
    }
    let r = lib.parse(&src);
    assert!(
        r.tree.stmts.len() > 20,
        "recovery consumed the file: only {} statements survived",
        r.tree.stmts.len()
    );
}

/// T-19 / R19 — one broken construct produces one diagnostic, not a cascade.
#[test]
fn cascade_is_suppressed() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let r = lib.parse("完全に違う\ngreet ok\n");
    assert!(
        r.diagnostics.len() <= 2,
        "one broken construct should not produce a wall: {:?}",
        r.diagnostics.len()
    );
}

/// T-22 / R19 — output is capped, so a very broken file does not print
/// thousands of lines.
#[test]
fn diagnostics_are_capped() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let mut src = String::new();
    for i in 0..200 {
        src.push_str(&format!("bogus{i} x\n"));
    }
    let r = lib.parse(&src);
    assert!(
        r.diagnostics.len() <= 20,
        "cap not applied: {} diagnostics",
        r.diagnostics.len()
    );
}

/// T-22 / R23 — emission refuses when the tree has unparsed regions. Rendering a
/// partial parse would silently omit whatever failed.
#[test]
fn emission_refuses_a_partial_parse() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let err = lib
        .run("greet a\nbogus\ngreet b\n")
        .expect_err("must not emit from a broken parse");
    let _ = err;
}

/// T-25 / R24 — `run` keeps its behaviour: first error, no output, no recovery.
#[test]
fn run_is_unchanged_by_recovery() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    assert!(lib.run("bogus\n").is_err());
    let good = lib.run("greet world\n").expect("a clean script still runs");
    assert_eq!(good, "hi world\n");
}

/// T-26 / R12 — any valid program yields zero diagnostics and no error nodes.
#[test]
fn a_valid_program_is_clean() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    let r = lib.parse("greet a\ngreet b\ngreet c\n");
    assert!(r.is_clean());
    assert!(r.diagnostics.is_empty());
    assert!(r.tree.errors.is_empty());
    assert_eq!(r.tree.stmts.len(), 3);
}

/// Recovery must terminate even when nothing can be consumed.
#[test]
fn recovery_always_terminates() {
    let lib = Library::new(GREET_LIB).expect("library compiles");
    // Every line is unparseable; the parser must not spin.
    let src = "?\n".repeat(50);
    let r = lib.parse(&src);
    assert!(r.tree.stmts.is_empty());
    assert!(!r.diagnostics.is_empty());
}
