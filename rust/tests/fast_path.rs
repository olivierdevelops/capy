//! The fused scanner + direct emit path (`Library::run`) must produce exactly what the general
//! lexer/parser/evaluator path produces (`run_strings`), wherever the scanner stops
//! (PROP-2026-0007 E13/E14, R17).

use capy_core::capy::Library;
use capy_core::orchestrator::run::run_strings;

const LIB: &str = r#"
extension txt

function import
    arg literal "import"
    arg capture name ident
    write `I ${name} @${line}:${col}
`
end

function say
    arg literal "say"
    arg capture msg string
    write `S ${msg}
`
end

function assign
    bare
    arg capture k ident
    arg literal "="
    arg capture v int
    write `A ${k}=${v}
`
end

function block
    arg literal "block"
    arg capture n ident
    block_closer end
    write `B ${n} {
${body}}
`
end

function end
    arg literal "end"
end

function other
    bare
    arg capture rest tail
    write `O ${rest}
`
end
"#;

fn same(script: &str) {
    let lib = Library::new(LIB).expect("lib");
    let fast = lib.run(script).map_err(|e| e.msg);
    let general = run_strings(LIB, "", script).map_err(|e| e.msg);
    assert_eq!(fast, general, "script: {script:?}");
}

#[test]
fn fully_decidable_scripts() {
    same("import a\nsay \"hi\"\nx = 5\nplain words here\n");
    same("");
    same("\n\n   \n");
}

#[test]
fn handoff_at_start_middle_and_end() {
    same("import\ttabbed\nimport b\nsay \"x\"\n");
    same("import a\nsay \"x\"\nimport\ttabbed\nx = 1\n");
    same("import a\nsay \"x\"\nx = 1\nimport\ttabbed");
}

#[test]
fn indented_blocks_and_comments_after_a_decided_prefix() {
    same("import a\n# note\nblock b\n    import c\n    say \"d\"\nend\nx = 2\n");
    same("import a # trailing comment\nsay \"x\" # another\n");
}

#[test]
fn number_and_string_edge_cases() {
    same("x = 014\ny = -0\nz = 1.5\nw = -7\nsay 'single'\nsay \"a b\"\nsay \"esc \\\" q\"\n");
    same("x = 5 + 1\nsay \"a\" + \"b\"\n");
}

#[test]
fn crlf_and_unicode() {
    same("import a\r\nsay \"x\"\r\nplain\r\n");
    same("import é\nsay \"ü\"\nnote ünïcode text\n");
}

#[test]
fn errors_and_their_precedence_are_unchanged() {
    // no function matches (second line is not provably excluded and nothing accepts it): same error
    same("import a\n)\n");
    // a lexer error late in the file must win over everything before it
    same("import a\nsay \"x\"\nbad `unterminated\n");
    // an evaluation error early (int capture given a string) and a parse error late: the parse error wins
    same("x = \"not an int\"\nimport a\n)\n");
    // evaluation error with a clean parse
    same("say 5\n");
}

#[test]
fn a_define_block_still_works_through_library_run() {
    // `run_strings` does not extract `define` blocks; `Library::run` does (unchanged by the scanner,
    // which stands aside for any script that defines something). Expected text taken from the baseline.
    let lib = Library::new(LIB).expect("lib");
    let out = lib
        .run("import a\ndefine extra\n    arg literal \"extra\"\n    write `E\n`\nend\nextra\n")
        .expect("run");
    assert_eq!(out, "I a @1:1\nE\n");
}
