//! Port of `infra/define_extractor_test.go`.

use capy_core::infra::define_extractor::extract_defines;

#[test]
fn none() {
    let src = "foo bar\nbaz\n";
    let (out, lib) = extract_defines(src).unwrap();
    assert_eq!(out, src, "source should be unchanged when there are no defines");
    assert_eq!(lib, "", "expected empty synthetic library");
}

#[test]
fn one_block() {
    let src = r#"define greet
    arg literal "greet"
    arg capture name string
    template_str "Hello, {{ .name | unquote }}!\n"
end

greet "World"
greet "Alice"
"#;
    let (cleaned, lib) = extract_defines(src).unwrap();
    assert!(
        !cleaned.contains("define greet"),
        "define block was not removed from source:\n{cleaned}"
    );
    assert!(
        cleaned.contains("greet \"World\""),
        "calls were stripped along with the define:\n{cleaned}"
    );
    assert!(
        lib.contains("function greet"),
        "synthetic library missing the function:\n{lib}"
    );
}

#[test]
fn multiple_blocks() {
    let src = r#"define a
    arg literal "a"
    template_str "a\n"
end

define b
    arg literal "b"
    template_str "b\n"
end

a
b
"#;
    let (cleaned, lib) = extract_defines(src).unwrap();
    assert!(!cleaned.contains("define "), "not all defines stripped: {cleaned}");
    assert!(lib.contains("function a"), "library missing function a:\n{lib}");
    assert!(lib.contains("function b"), "library missing function b:\n{lib}");
}

#[test]
fn unclosed_block() {
    let src = "define greet\n    arg literal \"greet\"\n    template_str \"\"\n";
    let err = extract_defines(src).expect_err("expected a missing-end error");
    assert!(
        err.contains("missing matching `end`"),
        "expected missing-end error, got {err:?}"
    );
}

#[test]
fn malformed_body() {
    let src = "define greet\n    arg whatever \"x\"\nend\n";
    extract_defines(src).expect_err("expected error for malformed body");
}

#[test]
fn bad_name() {
    // `define` followed by a string literal isn't a valid name; the extractor
    // should refuse it rather than passing garbage to the .capy library parser.
    let src = "define \"not-an-ident\"\n    template_str \"\"\nend\n";
    extract_defines(src).expect_err("expected error for non-identifier define name");
}

#[test]
fn ignores_indented_define() {
    // A `define` that's indented (inside a block body) is part of the
    // surrounding template — not a new top-level define.
    let src = "function outer\n    template:\n        define this should not be parsed as a block\nend\n";
    let (out, lib) = extract_defines(src).unwrap();
    assert_eq!(lib, "", "expected no defines extracted, got: {lib}");
    assert!(
        out.contains("define this"),
        "indented `define` was incorrectly stripped:\n{out}"
    );
}
