//! A `-` that touches the value before it is subtraction; a `-` after a space that touches a
//! number is that number's sign. So `a-1`, `f(x)-1` and `xs[0]-1` subtract, while the
//! space-separated arguments of a statement such as `on_key "ArrowUp" player 0 -4` keep `-4`
//! as one value. Requested by Capyler (PROP-2026-0007 §11.2, engine request 1).

use capy_core::domain::ast::Expr;
use capy_core::domain::token::TokenKind;
use capy_core::orchestrator::features::{make_lexer, value_parser};

fn tokens(src: &str) -> Vec<String> {
    make_lexer::tokenize(src)
        .expect("lex")
        .into_iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .map(|t| t.text)
        .collect()
}

fn parse(src: &str) -> Expr {
    let toks = make_lexer::tokenize(src).expect("lex");
    let mut r = value_parser::SliceReader::new(toks);
    value_parser::parse_value(&mut r, &[]).expect("parse")
}

fn shape(e: &Expr) -> String {
    match e {
        Expr::Binary(b) => format!("({} {} {})", shape(&b.left), b.op, shape(&b.right)),
        Expr::Compare(c) => format!("({} {} {})", shape(&c.left), c.op, shape(&c.right)),
        Expr::Unary(u) => format!("({} {})", u.op, shape(&u.operand)),
        Expr::Var(steps) => steps
            .iter()
            .map(|s| s.field.clone())
            .collect::<Vec<_>>()
            .join("."),
        Expr::Number(n) if n.is_int => n.i.to_string(),
        Expr::List(items) => format!(
            "[{}]",
            items.iter().map(shape).collect::<Vec<_>>().join(", ")
        ),
        other => format!("{other:?}"),
    }
}

#[test]
fn touching_minus_is_an_operator_token() {
    assert_eq!(tokens("a-1"), ["a", "-", "1"]);
    assert_eq!(tokens("(x)-1"), ["(", "x", ")", "-", "1"]);
    assert_eq!(tokens("[x]-1"), ["[", "x", "]", "-", "1"]);
    assert_eq!(tokens("\"s\"-1"), ["s", "-", "1"]);
}

#[test]
fn spaced_or_leading_minus_stays_the_sign() {
    assert_eq!(tokens("a -1"), ["a", "-1"]);
    assert_eq!(tokens("0 -4"), ["0", "-4"]);
    assert_eq!(tokens("x = -1"), ["x", "=", "-1"]);
    assert_eq!(tokens("(-1)"), ["(", "-1", ")"]);
    assert_eq!(tokens("a * -1"), ["a", "*", "-1"]);
}

#[test]
fn touching_minus_subtracts_in_expressions() {
    let cases = [
        ("a-1", "(a - 1)"),
        ("a - 1", "(a - 1)"),
        ("b*2-1", "((b * 2) - 1)"),
        ("a-1-2", "((a - 1) - 2)"),
        ("2 - -3", "(2 - -3)"),
        ("-6 + 2", "(-6 + 2)"),
        ("- 6", "(- 6)"),
    ];
    for (src, want) in cases {
        assert_eq!(shape(&parse(src)), want, "{src}");
    }
}

#[test]
fn smallest_int_literal_is_preserved() {
    match parse("-9223372036854775808") {
        Expr::Number(n) => assert!(n.is_int && n.i == i64::MIN, "got {n:?}"),
        other => panic!("want the i64::MIN literal, got {other:?}"),
    }
    assert_eq!(
        shape(&parse("1 + -9223372036854775808")),
        "(1 + -9223372036854775808)"
    );
}

#[test]
fn list_items_keep_a_touching_sign() {
    // List items are parsed one primary at a time, so `[a-1]` is still two items.
    assert_eq!(shape(&parse("[a-1]")), "[a, -1]");
    assert_eq!(shape(&parse("[1 -1]")), "[1, -1]");
}
