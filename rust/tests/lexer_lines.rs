//! Line handling in the lexer must not change what a script means (PROP-2026-0007 E5, first part):
//! CRLF is normalised, multi-line backtick templates still merge, blank lines survive.

use capy_core::capy::Library;

const LIB: &str = "extension txt\n\nfunction say\n    arg literal \"say\"\n    arg capture m string\n    write `line one\nline two ${m}\n`\nend\n";

#[test]
fn crlf_and_lf_scripts_render_identically() {
    let lib = Library::new(LIB).expect("lib");
    let lf = lib.run("say \"a\"\n\nsay \"b\"\n").expect("lf");
    let crlf = lib.run("say \"a\"\r\n\r\nsay \"b\"\r\n").expect("crlf");
    assert_eq!(lf, crlf);
    assert_eq!(lf, "line one\nline two \"a\"\nline one\nline two \"b\"\n");
}

#[test]
fn crlf_library_source_loads_like_lf() {
    let crlf_lib = LIB.replace('\n', "\r\n");
    let a = Library::new(LIB).expect("lf lib").run("say \"x\"\n").expect("run");
    let b = Library::new(&crlf_lib).expect("crlf lib").run("say \"x\"\n").expect("run");
    assert_eq!(a, b);
}

#[test]
fn a_script_with_no_trailing_newline_and_with_blank_lines() {
    let lib = Library::new(LIB).expect("lib");
    assert_eq!(
        lib.run("\n\nsay \"z\"").expect("run"),
        "line one\nline two \"z\"\n"
    );
}
