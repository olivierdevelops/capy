//! PLAN-2026-0002 — T-07, T-08, T-27. Operator precedence (R10, R11, R12).
//!
//! T-27 is the important one. `expr_to_text` round-trips a captured expression
//! back to source text, and the original parentheses are NOT in the AST — so a
//! renderer that forgets to re-insert them turns `(a + b) * c` into
//! `a + b * c`: silently the wrong expression, and silently the wrong emitted
//! code. No golden can catch it, because arithmetic was a parse error before
//! this change, so no golden contains any.

use capy_core::domain::ast::Expr;
use capy_core::orchestrator::features::expr_to_text::expr_to_text;
use capy_core::orchestrator::features::{make_lexer, value_parser};

/// Parse a bare value expression.
fn parse(src: &str) -> Expr {
    let toks = make_lexer::tokenize(src).expect("lex");
    let mut r = value_parser::SliceReader::new(toks);
    value_parser::parse_value(&mut r, &[]).expect("parse")
}

/// Structural shape, with explicit parentheses at every level. Used instead of
/// string equality so the assertion is about the TREE, not about formatting.
fn shape(e: &Expr) -> String {
    match e {
        Expr::Binary(b) => format!("({} {} {})", shape(&b.left), b.op, shape(&b.right)),
        Expr::Compare(c) => format!("({} {} {})", shape(&c.left), c.op, shape(&c.right)),
        Expr::Not(x) => format!("(not {})", shape(x)),
        Expr::Var(steps) => steps
            .iter()
            .map(|s| s.field.clone())
            .collect::<Vec<_>>()
            .join("."),
        Expr::Number(n) => {
            if n.is_int {
                n.i.to_string()
            } else {
                format!("{}", n.f)
            }
        }
        Expr::Str(s) => format!("{s:?}"),
        Expr::Bool(b) => b.to_string(),
        Expr::Null => "null".into(),
        other => format!("{other:?}"),
    }
}

/// T-07 — precedence and associativity.
#[test]
fn precedence_and_associativity() {
    let cases = [
        ("a * b + c", "((a * b) + c)"),
        ("a + b * c", "(a + (b * c))"),
        ("a - b - c", "((a - b) - c)"), // left-associative
        ("a + b + c", "((a + b) + c)"),
        ("a * b / c", "((a * b) / c)"),
        ("(a + b) * c", "((a + b) * c)"), // parens override
        ("a + (b + c)", "(a + (b + c))"),
    ];
    for (src, want) in cases {
        assert_eq!(shape(&parse(src)), want, "parsing {src:?}");
    }
}

/// T-11 — comparison is now precedence-ordered rather than one flat step.
#[test]
fn comparison_binds_looser_than_arithmetic() {
    assert_eq!(shape(&parse("a + 1 == b * 2")), "((a + 1) == (b * 2))");
    assert_eq!(shape(&parse("a < b + 1")), "(a < (b + 1))");
}

/// `and` / `or` bind looser than comparison.
#[test]
fn boolean_operators_bind_loosest() {
    assert_eq!(shape(&parse("a == 1 and b == 2")), "((a == 1) and (b == 2))");
    assert_eq!(
        shape(&parse("a and b or c")),
        "((a and b) or c)",
        "`and` binds tighter than `or`"
    );
}

/// **T-27 — the round-trip property.**
///
/// `parse(render(parse(E))) == parse(E)`, structurally. Catches dropped and
/// misplaced parentheses, which are invisible to every other test.
#[test]
fn round_trip_preserves_structure() {
    let corpus = [
        "a * b + c",
        "a + b * c",
        "(a + b) * c",
        "a * (b + c)",
        "a - b - c",
        "a - (b - c)", // ← the one a naive renderer gets wrong
        "(a - b) - c",
        "a / b / c",
        "a / (b / c)",
        "a + b * c - d / e",
        "(a + b) * (c - d)",
        "a == b + 1",
        "(a == b) == c",
        "a and b or c",
        "a or (b and c)",
        "(a or b) and c",
        "a * b % c",
        "1 + 2 * 3 - 4",
        "x.y + z.w * 2",
        "not a and b",
    ];
    for src in corpus {
        let once = parse(src);
        let rendered = expr_to_text(&once);
        let twice = parse(&rendered);
        assert_eq!(
            shape(&once),
            shape(&twice),
            "round trip changed the tree\n  source:   {src}\n  rendered: {rendered}"
        );
    }
}

/// The specific corruption T-27 exists to prevent, called out on its own so a
/// failure is unmistakable.
#[test]
fn parentheses_are_not_lost() {
    let e = parse("(a + b) * c");
    let rendered = expr_to_text(&e);
    assert!(
        rendered.contains('('),
        "the grouping parentheses were dropped: {rendered:?} — this silently \
         changes the emitted code from (a+b)*c to a+b*c"
    );
    assert_eq!(shape(&parse(&rendered)), "((a + b) * c)");
}

/// Right operand of a left-associative operator needs parentheses at EQUAL
/// precedence too: `a - (b - c)` is not `a - b - c`.
#[test]
fn equal_precedence_right_operand_keeps_parentheses() {
    let e = parse("a - (b - c)");
    let rendered = expr_to_text(&e);
    assert_eq!(
        shape(&parse(&rendered)),
        "(a - (b - c))",
        "rendered as {rendered:?}"
    );
}
