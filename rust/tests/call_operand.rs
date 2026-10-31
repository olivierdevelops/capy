//! A call written `f(args)`, with the `(` touching the name, is an operand of an expression when
//! an operator is next to it or it sits in another call's arguments: `n * fact(n - 1)` is a tree
//! whose right child is the call. A bare `f(x)` is not parsed here; the grammar rules that read it
//! (a statement call, `def a = add(1, 2)`, a `case Config(cfg)` pattern) keep their trees.
//! Requested by Capyler (PROP-2026-0007 review, calls inside arithmetic).

use capy_core::domain::ast::Expr;
use capy_core::orchestrator::features::{make_lexer, value_parser};

fn parse(src: &str) -> Result<(Expr, usize), String> {
    let toks = make_lexer::tokenize(src).expect("lex");
    let mut r = value_parser::SliceReader::new(toks);
    let e = value_parser::parse_value(&mut r, &[]).map_err(|e| e.to_string())?;
    let left = {
        use value_parser::TokReader;
        let mut n = 0;
        while !matches!(r.peek().kind, capy_core::domain::token::TokenKind::Eof | capy_core::domain::token::TokenKind::Newline) {
            r.advance();
            n += 1;
        }
        n
    };
    Ok((e, left))
}

fn shape(e: &Expr) -> String {
    match e {
        Expr::Binary(b) => format!("({} {} {})", shape(&b.left), b.op, shape(&b.right)),
        Expr::Compare(c) => format!("({} {} {})", shape(&c.left), c.op, shape(&c.right)),
        Expr::Unary(u) => format!("({} {})", u.op, shape(&u.operand)),
        Expr::Not(x) => format!("(not {})", shape(x)),
        Expr::Call(c) => format!(
            "{}[{}]",
            c.name.join("."),
            c.args.iter().map(shape).collect::<Vec<_>>().join(", ")
        ),
        Expr::Var(steps) => steps.iter().map(|s| s.field.clone()).collect::<Vec<_>>().join("."),
        Expr::Number(n) if n.is_int => n.i.to_string(),
        other => format!("{other:?}"),
    }
}

fn tree(src: &str) -> String {
    let (e, left) = parse(src).expect("parse");
    assert_eq!(left, 0, "{src}: tokens left unread");
    shape(&e)
}

#[test]
fn a_call_is_the_right_operand() {
    assert_eq!(tree("n * fact(n - 1)"), "(n * fact[(n - 1)])");
    assert_eq!(tree("1 + f(2)"), "(1 + f[2])");
}

#[test]
fn a_call_is_the_left_operand() {
    assert_eq!(tree("two() + 3 * 4"), "(two[] + (3 * 4))");
    assert_eq!(tree("f(1) == 1"), "(f[1] == 1)");
}

#[test]
fn precedence_holds_around_calls() {
    assert_eq!(tree("1 + f(2) * 3"), "(1 + (f[2] * 3))");
    assert_eq!(tree("f(1) * 2 + g(3)"), "((f[1] * 2) + g[3])");
    assert_eq!(tree("inc(1) < inc(2)"), "(inc[1] < inc[2])");
}

#[test]
fn arguments_are_full_expressions_and_calls_nest() {
    assert_eq!(tree("add(1 + 2, 3) * 2"), "(add[(1 + 2), 3] * 2)");
    assert_eq!(tree("1 + add(1, mul(2, 3))"), "(1 + add[1, mul[2, 3]])");
}

#[test]
fn unary_and_not_reach_calls() {
    assert_eq!(tree("-f(1)"), "(- f[1])");
    assert_eq!(tree("yes() and not yes()"), "(yes[] and (not yes[]))");
}

#[test]
fn a_bare_call_is_left_to_the_grammar() {
    // No operator and no enclosing call: the primary is the name only, and `(` stays unread.
    let (e, left) = parse("add(1, 2)").expect("parse");
    assert_eq!(shape(&e), "add");
    assert!(left > 0);
    let (e, _) = parse("Config(cfg)").expect("parse");
    assert_eq!(shape(&e), "Config");
}

#[test]
fn a_space_before_the_bracket_keeps_the_prefix_call() {
    let (e, _) = parse("1 + one (2)").expect("parse");
    assert!(!shape(&e).contains("one["), "{}", shape(&e));
}
