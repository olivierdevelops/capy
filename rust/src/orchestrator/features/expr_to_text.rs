//! Port of `orchestrator/features/expr_to_text.go`.

use crate::domain::ast::{Expr, PathStep};
use crate::gofmt;

/// Converts a parsed value expression back into a source-like text
/// representation. Capy is a TRANSPILER: a `cond:any` capture in `if x > 0`
/// should appear in the target template as the literal text `x > 0`, not as an
/// evaluated boolean. That is what this function provides.
pub fn expr_to_text(x: &Expr) -> String {
    match x {
        Expr::Number(n) => {
            if n.is_int {
                n.i.to_string()
            } else {
                gofmt::format_float_g(n.f)
            }
        }
        // Re-quote so the surface looks like a string literal. The source token
        // already had quotes stripped by the lexer.
        Expr::Str(v) => gofmt::quote(v),
        Expr::Bool(v) => if *v { "true" } else { "false" }.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Var(steps) => var_ref_to_text(steps, &expr_to_text),
        Expr::Compare(c) => {
            format!("{} {} {}", expr_to_text(&c.left), c.op, expr_to_text(&c.right))
        }
        // PLAN-2026-0002 R10/R12 — round-trip an infix operation.
        //
        // Parentheses are re-inserted wherever a child binds LOOSER than its
        // parent, because the original parentheses are not in the AST. Without
        // this, `(a + b) * c` would render as `a + b * c` — silently the wrong
        // expression, and silently the wrong emitted code. T-27 is the gate.
        Expr::Binary(b) => {
            let p = prec_of(&b.op);
            format!(
                "{} {} {}",
                wrap_if_looser(&b.left, p),
                b.op,
                // Right operand of a left-associative operator needs parens at
                // EQUAL precedence too: `a - (b - c)` is not `a - b - c`.
                wrap_if_looser_or_equal(&b.right, p)
            )
        }
        Expr::Not(x) => format!("not {}", expr_to_text(x)),
        Expr::List(items) => {
            let parts: Vec<String> = items.iter().map(expr_to_text).collect();
            format!("[{}]", parts.join(", "))
        }
        Expr::Obj(o) => {
            let parts: Vec<String> = o
                .keys
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{}: {}", gofmt::quote(k), expr_to_text(&o.vals[i])))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Expr::Call(c) => {
            let args: Vec<String> = c.args.iter().map(expr_to_text).collect();
            format!("({} {})", c.name.join("."), args.join(" "))
        }
    }
}

/// Port of `varRefToText`.
///
/// Renders a step-based `VarRef` back to its source form: the root, then
/// `.field` for field steps and `[expr]` for index steps (the index expression
/// rendered via the supplied recursive renderer). Shared by [`expr_to_text`] and
/// translate's `render_expr` so both round-trip indexed reads like
/// `context.buf[i]` identically.
pub fn var_ref_to_text(steps: &[PathStep], render_expr: &dyn Fn(&Expr) -> String) -> String {
    let mut b = String::new();
    for (i, s) in steps.iter().enumerate() {
        if s.is_index {
            b.push('[');
            if let Some(ix) = &s.index {
                b.push_str(&render_expr(ix));
            }
            b.push(']');
        } else {
            if i > 0 {
                b.push('.');
            }
            b.push_str(&s.field);
        }
    }
    b
}

/// PLAN-2026-0002 — binding power, mirroring `value_parser::binding_power`.
/// Kept in step with it by `round_trip_preserves_structure` (T-27).
fn prec_of(op: &str) -> u8 {
    match op {
        "or" => 1,
        "and" => 2,
        "==" | "!=" | "<" | ">" | "<=" | ">=" => 3,
        "+" | "-" => 4,
        "*" | "/" | "%" => 5,
        _ => 0,
    }
}

fn prec_of_expr(x: &Expr) -> Option<u8> {
    match x {
        Expr::Binary(b) => Some(prec_of(&b.op)),
        Expr::Compare(c) => Some(prec_of(&c.op)),
        _ => None,
    }
}

fn wrap_if_looser(x: &Expr, parent: u8) -> String {
    match prec_of_expr(x) {
        Some(p) if p < parent => format!("({})", expr_to_text(x)),
        _ => expr_to_text(x),
    }
}

fn wrap_if_looser_or_equal(x: &Expr, parent: u8) -> String {
    match prec_of_expr(x) {
        Some(p) if p <= parent => format!("({})", expr_to_text(x)),
        _ => expr_to_text(x),
    }
}
