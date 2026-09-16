//! PLAN-2026-0001 — T-23 / T-24. Left-recursion guard (R0, R0b).
//!
//! Before this guard, a library whose function-as-type capture led back to
//! itself passed `capy check` with `ok` and then killed the process:
//!
//! ```text
//! thread 'main' has overflowed its stack
//! fatal runtime error: stack overflow, aborting        rc=134
//! ```
//!
//! An embedder cannot catch that as `Err` — it takes down the host. These tests
//! pin that every such library is now rejected at LOAD time with a normal
//! `CapyError`, and that libraries which are merely recursive (not *left*
//! recursive) still load.

use capy_core::capy::Library;

fn load(src: &str) -> Result<Library, capy_core::domain::errors::CapyError> {
    Library::new(src)
}

/// T-23 — the exact P-07 reproduction. `expr` matches `expr` at argument 1.
#[test]
fn direct_left_recursion_is_rejected() {
    let err = load(
        r#"
extension txt

function expr
    arg capture lhs expr
    arg literal "+"
    arg capture rhs term
end

function term
    arg capture n any
end
"#,
    )
    .expect_err("a left-recursive library must not load");
    let msg = err.to_string();
    assert!(msg.contains("left recursion"), "got: {msg}");
    assert!(msg.contains("expr"), "the cycle must name the function: {msg}");
}

/// T-23 — indirect cycle: `expr` -> `term` -> `expr`. Nothing is consumed on
/// either hop, so the descent is just as unbounded as the direct case.
///
/// Both functions declare a literal of their own, which matters: a function that
/// declares NO literal gets its own name auto-prepended as one (see
/// `single_capture_function_is_not_left_recursive`), and that prepended literal
/// consumes a token, which breaks the cycle.
#[test]
fn indirect_left_recursion_is_rejected() {
    let err = load(
        r#"
extension txt

function expr
    arg capture lhs term
    arg literal "+"
    arg capture rhs term
end

function term
    arg capture inner expr
    arg literal "*"
    arg capture rest expr
end
"#,
    )
    .expect_err("an indirectly left-recursive library must not load");
    let msg = err.to_string();
    assert!(msg.contains("left recursion"), "got: {msg}");
}

/// A function that declares no `arg literal` has its own name prepended as one
/// by the loader — `function term / arg capture inner expr` compiles to the
/// pattern `term <inner>`. That leading literal consumes a token, so such a
/// function can never be the left-recursive hop, however it is referenced.
///
/// Pinned as a test because it is the difference between the library above
/// (rejected) and this one (fine), and it is not obvious from the source.
#[test]
fn single_capture_function_is_not_left_recursive() {
    load(
        r#"
extension txt

function expr
    arg capture lhs term
    arg literal "+"
    arg capture rhs term
end

function term
    arg capture inner expr
end
"#,
    )
    .expect("the auto-prepended name literal consumes a token, so this is not left recursion");
}

/// A literal in front means a token is consumed before the descent, so the
/// recursion is bounded by the input. This MUST still load — rejecting it would
/// break the documented way to write a nested grammar.
#[test]
fn right_recursion_still_loads() {
    load(
        r#"
extension txt

function expr
    arg literal "("
    arg capture inner expr
    arg literal ")"
    write `[${inner}]`
end

function atom
    arg capture n any
    write `${n}`
end
"#,
    )
    .expect("a right-recursive library must still load");
}

/// A function-as-type capture that is not part of any cycle is ordinary and
/// must keep working.
#[test]
fn non_recursive_function_as_type_still_loads() {
    load(
        r#"
extension txt

function pair
    arg literal "pair"
    arg capture a item
    arg capture b item
end

function item
    arg capture n any
end
"#,
    )
    .expect("a non-recursive nonterminal must still load");
}

/// T-24 — whatever the library, loading must terminate and must never abort the
/// process. A chain of nonterminals N deep is the shape most likely to recurse
/// in the guard itself, which is why the guard uses an explicit stack.
#[test]
fn deep_nonterminal_chain_terminates() {
    let mut src = String::from("extension txt\n\n");
    const N: usize = 200;
    for i in 0..N {
        src.push_str(&format!(
            "function f{i}\n    arg literal \"f{i}\"\n    arg capture x f{next}\nend\n\n",
            next = i + 1
        ));
    }
    src.push_str(&format!("function f{N}\n    arg capture n any\nend\n"));
    // Must return — Ok or Err, but it must not hang or abort.
    let _ = load(&src);
}

/// The guard must not trip on a self-reference that sits AFTER something that
/// consumes, which is the ordinary way to write a repeating construct.
#[test]
fn self_reference_after_a_literal_is_fine() {
    load(
        r#"
extension txt

function list
    arg literal "list"
    arg capture head any
    arg capture tail list
end
"#,
    )
    .expect("a self-reference behind a literal must still load");
}

/// T-24 — **deeply nested input must not abort the process either.**
///
/// The load-time guard only stops a *library* from recursing without consuming.
/// It cannot stop a valid right-recursive grammar being fed pathological input:
/// 20 000 nested parentheses overflowed the native stack and exited rc=134,
/// which an embedder cannot catch. The parser now bounds its descent instead.
#[test]
fn deeply_nested_input_errors_instead_of_aborting() {
    let lib = Library::new(
        r#"
extension txt

function expr
    arg literal "("
    arg capture inner expr
    arg literal ")"
    write `[${inner}]`
end

function atom
    arg literal "x"
    write `x`
end
"#,
    )
    .expect("a right-recursive library is valid and must load");

    const N: usize = 20_000;
    let script = format!("{}x{}\n", "(".repeat(N), ")".repeat(N));
    // The contract is that this RETURNS. Whether it parses or reports an error
    // is secondary; aborting the host process is not an option.
    let result = lib.run(&script);
    assert!(
        result.is_err(),
        "input nested {N} deep should be refused, not accepted"
    );
}

/// Ordinary nesting must keep working. Deep *block* nesting is the real-world
/// shape — a nonterminal chain cannot bottom out in Capy, because a capture of
/// type `expr` matches only the function named `expr`, with no alternation.
#[test]
fn ordinary_block_nesting_still_parses() {
    let lib = Library::new(
        r#"
extension txt

function wrap
    arg literal "wrap"
    block_closer end
    write `[${body}]`
end

function leaf
    arg literal "leaf"
    write `L`
end

function end
end
"#,
    )
    .expect("library compiles");

    const N: usize = 30;
    let mut src = String::new();
    for i in 0..N {
        src.push_str(&format!("{}wrap\n", "    ".repeat(i)));
    }
    src.push_str(&format!("{}leaf\n", "    ".repeat(N)));
    for i in (0..N).rev() {
        src.push_str(&format!("{}end\n", "    ".repeat(i)));
    }
    let out = lib.run(&src).expect("30 nested blocks must parse");
    assert!(out.contains('L'), "the leaf must render: {out}");
}
