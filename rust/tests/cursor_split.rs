//! A candidate that fails after splitting a merged punctuation token must not
//! change what later candidates see (PROP-2026-0007 P-09 / E8, R12).
//!
//! The lexer merges `><` into one punct token; a library literal `">"` matches its
//! prefix and used to rewrite the token in place to `<`. If that candidate then
//! failed, `restore` reset only the position, so the next candidate saw `<`.

use capy_core::capy::Library;

const LIB_BOTH: &str = r#"
extension txt

function a
    priority 10
    arg literal ">"
    arg literal "zzz"
    write `A
`
end

function b
    arg literal "><"
    arg capture w ident
    write `B ${w}
`
end
"#;

const LIB_B_ONLY: &str = r#"
extension txt

function b
    arg literal "><"
    arg capture w ident
    write `B ${w}
`
end
"#;

#[test]
fn control_without_the_failing_candidate() {
    let lib = Library::new(LIB_B_ONLY).expect("lib");
    assert_eq!(lib.run(">< foo\n").expect("run"), "B foo\n");
}

#[test]
fn failed_split_candidate_does_not_corrupt_the_token_stream() {
    let lib = Library::new(LIB_BOTH).expect("lib");
    // `a` splits `><` at ">" then fails on "zzz"; `b` must still see `><`.
    assert_eq!(lib.run(">< foo\n").expect("run"), "B foo\n");
}

#[test]
fn successful_split_still_works() {
    // `>` then `<` as two literals: the prefix split must keep working when nothing fails.
    let lib = Library::new(
        "extension txt\nfunction t\n    arg literal \">\"\n    arg literal \"<\"\n    arg capture w ident\n    write `T ${w}\n`\nend\n",
    )
    .expect("lib");
    assert_eq!(lib.run(">< foo\n").expect("run"), "T foo\n");
}
