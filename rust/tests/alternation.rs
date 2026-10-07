//! PLAN-2026-0003 / PROP-2026-0004 — ordered alternation in a capture type.
//!
//! ```text
//!   arg capture v call | atom        a capture type may name several library
//!                                    functions, tried left to right
//! ```
//!
//! T-01 … T-05, T-07, T-10, T-11, T-12 of PLAN-2026-0003. Every test names the
//! requirement it validates.

use capy_core::capy::Library;
use capy_core::domain::ast::{Block, FuncCall};

fn load(src: &str) -> Result<Library, capy_core::domain::errors::CapyError> {
    Library::new(src)
}

/// A recursive call grammar: an argument is a nested call OR an atom.
const CALL_LIB: &str = r#"
extension txt

function ret
    arg literal "return"
    arg capture value call
    write `return ${value};
`
end

function call
    bare
    arg capture fname ident
    arg literal "("
    arg capture args operand* sep "," join ", "
    arg literal ")"
    write `${fname}(${args})`
end

function operand
    bare
    arg capture v call | name | num
    write `${v}`
end

function num
    bare
    arg capture n int
    write `${n}`
end

function name
    bare
    arg capture id ident
    write `${id}`
end
"#;

fn first_stmt(lib: &Library, src: &str) -> FuncCall {
    let r = lib.parse(src);
    assert!(r.is_clean(), "expected a clean parse, got {:?}", r.diagnostics);
    r.tree.stmts[0].clone()
}

fn walk<'a>(b: &'a Block, out: &mut Vec<&'a str>) {
    for s in &b.stmts {
        walk_node(s, out);
    }
}

fn walk_node<'a>(f: &'a FuncCall, out: &mut Vec<&'a str>) {
    out.push(f.func.as_str());
    for c in f.captures.values() {
        for s in &c.sub {
            walk_node(s, out);
        }
    }
}

// --- R1 -------------------------------------------------------------------

/// T-01a / R1 — a library with an alternation loads; the introspection reports
/// alternative 1 in `type_` and the rest in `alts` (T-12 / R9).
#[test]
fn alternation_loads_and_is_introspectable() {
    let lib = load(CALL_LIB).expect("alternation must load");
    let info = lib.introspect();
    let operand = info.iter().find(|f| f.name == "operand").unwrap();
    let cap = operand.args.iter().find(|a| a.kind == "capture").unwrap();
    assert_eq!(cap.type_, "call", "type_ keeps meaning alternative 1 (ARCH-001)");
    assert_eq!(cap.alts, vec!["name".to_string(), "num".to_string()]);
}

/// R1 — glued `a|b` is the same choice as `a | b`.
#[test]
fn glued_pipe_is_the_same_choice() {
    let glued = CALL_LIB.replace("call | name | num", "call|name|num");
    let lib = load(&glued).expect("glued alternation must load");
    let info = lib.introspect();
    let operand = info.iter().find(|f| f.name == "operand").unwrap();
    let cap = operand.args.iter().find(|a| a.kind == "capture").unwrap();
    assert_eq!(cap.alts, vec!["name".to_string(), "num".to_string()]);
}

/// R1 — an unknown name in ANY position is a load error that names it.
#[test]
fn unknown_alternative_is_a_load_error_naming_it() {
    for src in [
        CALL_LIB.replace("call | name | num", "nope | name | num"),
        CALL_LIB.replace("call | name | num", "call | nope | num"),
        CALL_LIB.replace("call | name | num", "call | name | nope"),
    ] {
        let msg = load(&src).expect_err("must not load").to_string();
        assert!(msg.contains("nope"), "the error must name the bad alternative: {msg}");
    }
}

/// OQ-08 / ADR-0003 — a built-in type is not an alternative.
#[test]
fn builtin_type_is_not_an_alternative() {
    let src = CALL_LIB.replace("call | name | num", "call | int");
    let msg = load(&src).expect_err("must not load").to_string();
    assert!(msg.contains("int") && msg.contains("not a library function"), "got: {msg}");
}

/// A dangling or empty alternative is a load error, not a silent no-op.
#[test]
fn malformed_alternation_is_rejected() {
    for bad in ["call |", "| call", "call | | num", "call* | num"] {
        let src = CALL_LIB.replace("call | name | num", bad);
        assert!(load(&src).is_err(), "{bad:?} must not load");
    }
}

// --- R5 -------------------------------------------------------------------

/// T-04 / R5 — left recursion through the FIRST alternative.
#[test]
fn left_recursive_first_alternative_is_rejected() {
    let msg = load(
        r#"
extension txt

function expr
    bare
    arg capture e expr | atom
end

function atom
    bare
    arg capture n int
end
"#,
    )
    .expect_err("must not load")
    .to_string();
    assert!(msg.contains("left recursion"), "got: {msg}");
}

/// T-04 / R5 — a cycle reachable ONLY through the second alternative. This is
/// the test that fails if the guard walks only `cap_type` (RK-03).
#[test]
fn left_recursion_through_second_alternative_is_rejected() {
    let msg = load(
        r#"
extension txt

function expr
    bare
    arg capture e atom | expr
end

function atom
    bare
    arg literal "x"
end
"#,
    )
    .expect_err("must not load")
    .to_string();
    assert!(msg.contains("left recursion"), "got: {msg}");
    assert!(msg.contains("expr"), "the cycle must name the function: {msg}");
}

/// R5 — a recursion that consumes first is NOT left recursive and still loads.
#[test]
fn right_recursive_alternation_loads() {
    load(CALL_LIB).expect("call consumes a name and `(` before recursing");
}

// --- R2 -------------------------------------------------------------------

/// T-01 / R2 — first match wins, and the order is observable. Two alternatives
/// match the same token; swapping them swaps which one the tree records. This
/// fails if the matcher ignores declaration order (QUAL-003).
#[test]
fn first_matching_alternative_wins() {
    let src = |order: &str| {
        format!(
            r#"
extension txt

function stmt
    arg literal "use"
    arg capture v {order}
    write `use ${{v}}
`
end

function first_id
    bare
    arg capture a ident
    write `first:${{a}}`
end

function second_id
    bare
    arg capture b ident
    write `second:${{b}}`
end
"#
        )
    };
    let forward = load(&src("first_id | second_id")).unwrap();
    let reverse = load(&src("second_id | first_id")).unwrap();
    assert_eq!(forward.run("use x\n").unwrap(), "use first:x\n");
    assert_eq!(reverse.run("use x\n").unwrap(), "use second:x\n");
    let tree = first_stmt(&forward, "use x\n");
    assert_eq!(tree.captures["v"].sub[0].func, "first_id");
}

/// T-01 — a failed earlier alternative is rewound, not leaked. `call` consumes
/// `f` before failing on the missing `(`; `name` must still see `f`.
#[test]
fn failed_alternative_is_rewound() {
    let lib = load(CALL_LIB).unwrap();
    assert_eq!(lib.run("return add(x, 7)\n").unwrap(), "return add(x, 7);\n");
}

// --- R3 / R4 / UC-01 / UC-02 ----------------------------------------------

/// T-02 / UC-01 — nested calls, the motivating case. Fails on 0.22.0.
#[test]
fn nested_calls_parse() {
    let lib = load(CALL_LIB).unwrap();
    assert_eq!(
        lib.run("return add(3, mul(4, 5))\n").unwrap(),
        "return add(3, mul(4, 5));\n"
    );
}

/// T-02 / UC-02 — a 5-deep nest, and R3's repetition with `sep` / `join`.
#[test]
fn five_deep_nest_parses() {
    let lib = load(CALL_LIB).unwrap();
    assert_eq!(
        lib.run("return f(g(h(i(j(1)))), k(2, 3))\n").unwrap(),
        "return f(g(h(i(j(1)))), k(2, 3));\n"
    );
}

/// T-03 / R4 — depth 70 hits the nesting bound. The contract `recursion_guard.rs`
/// pins still holds (an `Err`, never a panic or stack overflow), and since 0.24.0 the
/// error NAMES the bound instead of surfacing as a generic `expected` message
/// (PLAN-2026-0004 R1).
#[test]
fn seventy_deep_nest_hits_the_existing_bound() {
    let lib = load(CALL_LIB).unwrap();
    let n = 70;
    let src = format!("return {}1{}\n", "f(".repeat(n), ")".repeat(n));
    let err = lib.run(&src).expect_err("must exceed the depth bound");
    let msg = err.to_string();
    assert!(msg.contains("nesting too deep"), "the bound must be named: {msg}");
    assert!(msg.contains("call | name | num"), "and say what it was matching: {msg}");
}

/// PLAN-2026-0004 R1 — `capy ast` reports the bound as `E0003`, not `E0001`.
#[test]
fn depth_bound_is_reported_as_e0003() {
    let lib = load(CALL_LIB).unwrap();
    let n = 40;
    let src = format!("return {}1{}\n", "f(".repeat(n), ")".repeat(n));
    let r = lib.parse(&src);
    assert!(!r.is_clean());
    assert_eq!(r.diagnostics.len(), 1, "{:?}", r.diagnostics);
    assert_eq!(r.diagnostics[0].code, "E0003");
    assert!(r.diagnostics[0].msg.contains("nesting too deep"));
}

/// PLAN-2026-0004 R1 — the line before the bound still parses, and the bound does not
/// leak into the next statement: a later, ordinary failure is `E0001`.
#[test]
fn depth_error_does_not_leak_into_the_next_statement() {
    let lib = load(CALL_LIB).unwrap();
    let deep = format!("return {}1{}\n", "f(".repeat(40), ")".repeat(40));
    let src = format!("return add(1, 2)\n{deep}return add(3, +)\n");
    let r = lib.parse(&src);
    let codes: Vec<&str> = r.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(codes, ["E0003", "E0001"], "{:?}", r.diagnostics);
}

/// 31 call levels are still accepted: the bound itself did not move.
#[test]
fn thirty_one_levels_still_parse() {
    let lib = load(CALL_LIB).unwrap();
    let n = 31;
    let src = format!("return {}1{}\n", "f(".repeat(n), ")".repeat(n));
    assert!(lib.run(&src).is_ok());
}

// --- R6 -------------------------------------------------------------------

/// T-05 / R6 — an input matching no alternative reports ALL of them.
#[test]
fn failure_reports_the_union_of_alternatives() {
    let lib = load(CALL_LIB).unwrap();
    let r = lib.parse("return add(3, +)\n");
    assert!(!r.is_clean());
    let msg = r.diagnostics.iter().map(|d| d.full_message()).collect::<Vec<_>>().join("\n");
    for alt in ["call", "name", "num"] {
        assert!(msg.contains(&format!("`{alt}`")), "missing `{alt}` in: {msg}");
    }
}

// --- R8 -------------------------------------------------------------------

/// T-07 / R8 — `sub[].func` identifies the matched alternative; no schema
/// field is added and `schema_version` stays 1.
#[test]
fn discriminator_names_the_matched_alternative() {
    let lib = load(CALL_LIB).unwrap();
    let r = lib.parse("return add(3, mul(4, x))\n");
    assert!(r.is_clean(), "{:?}", r.diagnostics);
    let mut funcs = Vec::new();
    walk(&r.tree, &mut funcs);
    assert_eq!(
        funcs,
        ["ret", "call", "operand", "num", "operand", "call", "operand", "num", "operand", "name"]
    );
    let json = capy_core::domain::ast_json::to_json_pretty(&r);
    assert!(json.contains("\"schema_version\": 1"), "{json}");
    assert!(json.contains("\"func\": \"call\""), "{json}");
}

// --- UC-06 ----------------------------------------------------------------

const PARAM_LIB: &str = r#"
extension txt

function def_fn
    arg literal "def"
    arg capture name ident
    arg literal "("
    arg capture ps mut_param | plain_param* sep "," join ", "
    arg literal ")"
    write `${name}(${ps})
`
end

function mut_param
    bare
    arg literal "mut"
    arg capture pname ident
    arg literal ":"
    arg capture ptype ident
    write `&${pname}: ${ptype}`
end

function plain_param
    bare
    arg capture pname ident
    arg literal ":"
    arg capture ptype ident
    write `${pname}: ${ptype}`
end
"#;

/// T-10 / UC-06 — mixed marked and unmarked parameters. On 0.22.0 this input is
/// rejected (``expected `mut`, found "n"``); see PROP-2026-0004 P-05.
#[test]
fn mixed_parameter_markers_parse() {
    let lib = load(PARAM_LIB).unwrap();
    assert_eq!(
        lib.run("def f(mut c: Counter, n: int)\n").unwrap(),
        "f(&c: Counter, n: int)\n"
    );
    assert_eq!(lib.run("def g(n: int, mut c: Counter)\n").unwrap(), "g(n: int, &c: Counter)\n");
    let t = first_stmt(&lib, "def f(mut c: Counter, n: int)\n");
    let funcs: Vec<&str> = t.captures["ps"].sub.iter().map(|s| s.func.as_str()).collect();
    assert_eq!(funcs, ["mut_param", "plain_param"]);
}

// --- OQ-01 ----------------------------------------------------------------

/// T-11 / OQ-01 — `|` is the choice token in a capture TYPE; the multi-character
/// source token `|>` is unaffected because the type position is library syntax.
#[test]
fn pipe_in_type_position_does_not_collide_with_pipe_arrow() {
    let lib = load(
        r#"
extension txt

function pipe
    arg capture lhs ident
    arg literal "|>"
    arg capture rhs side
    write `${lhs} => ${rhs}
`
end

function side
    bare
    arg capture v left | right
    write `${v}`
end

function left
    bare
    arg literal "L"
    arg capture n ident
    write `l:${n}`
end

function right
    bare
    arg literal "R"
    arg capture n ident
    write `r:${n}`
end
"#,
    )
    .expect("a library with both `|` choice and a `|>` literal must load");
    assert_eq!(lib.run("x |> R y\n").unwrap(), "x => r:y\n");
}

// --- R9 / T-12 ------------------------------------------------------------

/// T-12 / R9 — `capy docs` prints the whole ordered choice, with the pipes
/// escaped for the Markdown table it renders into.
#[test]
fn docs_print_the_union() {
    let lib = load(CALL_LIB).unwrap();
    let docs = capy_core::capy::render_library_docs(&lib);
    // Escaped, so the pipes do not read as Markdown table-cell separators.
    assert!(docs.contains("`call \\| name \\| num`"), "got:\n{docs}");
}
