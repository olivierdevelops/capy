//! Port of `orchestrator/features/translate_new_shape.go`.

use super::expr_to_text::var_ref_to_text;
use crate::domain::ast::{Expr, InnerBlock, InnerStmt, LoopStmt, Path};
use crate::domain::errors::CapyError;
use crate::gofmt;

/// Port of `translateNewShape`.
///
/// Walks an inner-DSL block (write calls intermixed with state-mutation
/// statements) and returns the residual run AST — the subset of statements that
/// touch state (`set` / `append` / `prepend` / `merge` / `delete` / `call` /
/// `error`).
///
/// The render-bearing statements (`write`, `if`, `for`) live alongside the
/// run-bearing ones in the same source body. At load time the renderer keeps the
/// FULL AST and skips the state mutations at render time; the run AST returned
/// here drives the SEPARATE per-statement run pass.
///
/// Control flow (`if` / `for`) that contains a mix of writes AND state mutations
/// is preserved on the run side with the writes stripped, so both phases see the
/// same iteration / branching shape.
pub fn translate_new_shape(b: &InnerBlock) -> Result<InnerBlock, CapyError> {
    let mut run_stmts: Vec<InnerStmt> = Vec::new();
    for s in &b.stmts {
        extract_run_stmt(s, &mut run_stmts)?;
    }
    Ok(InnerBlock { stmts: run_stmts })
}

/// Port of `extractRunStmt`.
///
/// Walks one statement and appends any state-mutating projection of it to `run`.
/// Pure render statements (`write`) produce no output. Control flow is preserved
/// when it wraps state mutations.
fn extract_run_stmt(s: &InnerStmt, run: &mut Vec<InnerStmt>) -> Result<(), CapyError> {
    match s {
        InnerStmt::Write(_) => Ok(()),
        InnerStmt::If { cond, body, else_ } => {
            let mut body_run: Vec<InnerStmt> = Vec::new();
            for child in &body.stmts {
                extract_run_stmt(child, &mut body_run)?;
            }
            let mut else_run: Vec<InnerStmt> = Vec::new();
            let has_else = else_.is_some();
            if let Some(e) = else_ {
                for child in &e.stmts {
                    extract_run_stmt(child, &mut else_run)?;
                }
            }
            if !body_run.is_empty() || !else_run.is_empty() {
                let else_block = if has_else && !else_run.is_empty() {
                    Some(Box::new(InnerBlock { stmts: else_run }))
                } else {
                    None
                };
                run.push(InnerStmt::If {
                    cond: cond.clone(),
                    body: InnerBlock { stmts: body_run },
                    else_: else_block,
                });
            }
            Ok(())
        }
        InnerStmt::Loop(l) => {
            let mut body_run: Vec<InnerStmt> = Vec::new();
            for child in &l.body.stmts {
                extract_run_stmt(child, &mut body_run)?;
            }
            if !body_run.is_empty() {
                run.push(InnerStmt::Loop(LoopStmt {
                    var: l.var.clone(),
                    key_var: l.key_var.clone(),
                    iter: l.iter.clone(),
                    body: InnerBlock { stmts: body_run },
                }));
            }
            Ok(())
        }
        // All other statements (set/append/prepend/merge/delete/call/error) are
        // state mutations — pass through unchanged.
        other => {
            run.push(other.clone());
            Ok(())
        }
    }
}

/// Port of `renderInnerBlock`.
///
/// Re-serialises an `InnerBlock` back into inner-DSL source text. Used after
/// [`translate_new_shape`] splits the body — the residual run statements flow
/// through the regular run → tokenize → parse pipeline so existing tests don't
/// have to special-case "pre-parsed AST" inputs.
pub fn render_inner_block(b: &InnerBlock) -> String {
    let mut out = String::new();
    for s in &b.stmts {
        render_inner_stmt(s, &mut out, 0);
    }
    out
}

/// Port of `renderInnerStmt`.
fn render_inner_stmt(s: &InnerStmt, out: &mut String, indent: usize) {
    let prefix = "    ".repeat(indent);
    match s {
        InnerStmt::Set { target, value } => out.push_str(&format!(
            "{}set {} {}\n",
            prefix,
            render_path(target),
            render_expr(value)
        )),
        InnerStmt::Append { target, value } => out.push_str(&format!(
            "{}append {} {}\n",
            prefix,
            render_path(target),
            render_expr(value)
        )),
        InnerStmt::Prepend { target, value } => out.push_str(&format!(
            "{}prepend {} {}\n",
            prefix,
            render_path(target),
            render_expr(value)
        )),
        InnerStmt::Merge { target, value } => out.push_str(&format!(
            "{}merge {} {}\n",
            prefix,
            render_path(target),
            render_expr(value)
        )),
        InnerStmt::Delete { target } => {
            out.push_str(&format!("{}delete {}\n", prefix, render_path(target)))
        }
        InnerStmt::Call(call) => {
            // `(name args...)` — the lisp-style call shape the inner parser
            // already accepts.
            out.push_str(&format!("{}({}", prefix, call.name.join(".")));
            for a in &call.args {
                out.push(' ');
                out.push_str(&render_expr(a));
            }
            out.push_str(")\n");
        }
        InnerStmt::If { cond, body, else_ } => {
            out.push_str(&format!("{}if {}\n", prefix, render_expr(cond)));
            for c in &body.stmts {
                render_inner_stmt(c, out, indent + 1);
            }
            if let Some(e) = else_ {
                out.push_str(&format!("{}else\n", prefix));
                for c in &e.stmts {
                    render_inner_stmt(c, out, indent + 1);
                }
            }
            out.push_str(&format!("{}end\n", prefix));
        }
        InnerStmt::Loop(l) => {
            if !l.key_var.is_empty() {
                out.push_str(&format!(
                    "{}loop {}, {} in {}\n",
                    prefix,
                    l.key_var,
                    l.var,
                    render_expr(&l.iter)
                ));
            } else {
                out.push_str(&format!(
                    "{}loop {} in {}\n",
                    prefix,
                    l.var,
                    render_expr(&l.iter)
                ));
            }
            for c in &l.body.stmts {
                render_inner_stmt(c, out, indent + 1);
            }
            out.push_str(&format!("{}end\n", prefix));
        }
        // Go's renderInnerStmt has no WriteStmt arm, so a write renders to
        // nothing. (translateNewShape strips them before this runs.)
        InnerStmt::Write(_) => {}
    }
}

/// Port of `renderPath`.
fn render_path(p: &Path) -> String {
    let mut out = p.root.clone();
    for st in &p.steps {
        if st.is_index {
            out.push('[');
            if let Some(ix) = &st.index {
                out.push_str(&render_expr(ix));
            }
            out.push(']');
        } else {
            out.push('.');
            out.push_str(&st.field);
        }
    }
    out
}

/// Port of `renderExpr`.
///
/// Differs from `expr_to_text` in two places: `not` renders parenthesised, and
/// an unrecognised node yields "" rather than `%v`.
pub fn render_expr(e: &Expr) -> String {
    match e {
        // PLAN-2026-0002 R10 — infix operations round-trip through the same
        // precedence-aware renderer the outer path uses, so a captured
        // expression is reconstructed with its structure intact.
        Expr::Binary(_) => super::expr_to_text::expr_to_text(e),
        Expr::Str(v) => gofmt::quote(v),
        Expr::Number(n) => {
            if n.is_int {
                n.i.to_string()
            } else {
                gofmt::format_float_g(n.f)
            }
        }
        Expr::Bool(v) => if *v { "true" } else { "false" }.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Var(steps) => var_ref_to_text(steps, &render_expr),
        Expr::Call(c) => {
            let mut out = format!("({}", c.name.join("."));
            for a in &c.args {
                out.push(' ');
                out.push_str(&render_expr(a));
            }
            out.push(')');
            out
        }
        Expr::Not(x) => format!("(not {})", render_expr(x)),
        Expr::Compare(c) => {
            format!("{} {} {}", render_expr(&c.left), c.op, render_expr(&c.right))
        }
        Expr::List(items) => {
            let parts: Vec<String> = items.iter().map(render_expr).collect();
            format!("[{}]", parts.join(", "))
        }
        Expr::Obj(o) => {
            let parts: Vec<String> = o
                .keys
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{}: {}", gofmt::quote(k), render_expr(&o.vals[i])))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}
