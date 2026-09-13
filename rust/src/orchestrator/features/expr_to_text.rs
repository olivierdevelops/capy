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
