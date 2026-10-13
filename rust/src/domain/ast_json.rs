//! PLAN-2026-0002 R8/R9 — serialize a parse result as JSON.
//!
//! Uses the crate's own [`gojson`](crate::gojson) writer rather than serde:
//! `capy-core` has exactly one dependency (`regex`) and keeps it. The engine
//! only ever *writes* JSON, so a full serialization framework would be a large
//! dependency tree bought for one direction of one feature.
//!
//! The shape is documented in `docs/ast-json.md`; `SCHEMA_VERSION` is the
//! contract. Fields may be ADDED without a bump; anything removed or given a new
//! meaning bumps it.

use crate::capy::ParseResult;
use crate::domain::ast::{Block, CaptureValue, ErrorNode, Expr, FuncCall, PathStep, Span};
use crate::domain::errors::{Diagnostic, Severity};
use crate::domain::val::Val;
use crate::gojson;
use std::collections::BTreeMap;

/// Bumped only for a breaking change to the shape below.
pub const SCHEMA_VERSION: i64 = 1;

fn obj(pairs: Vec<(&str, Val)>) -> Val {
    let mut m: BTreeMap<String, Val> = BTreeMap::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    Val::Obj(m)
}

/// A span, or `null` when unset. Unset means nothing was consumed — an optional
/// capture that took its default — and is deliberately not rendered as zeros,
/// which a consumer would mistake for a real position.
fn span_val(s: Span) -> Val {
    if s.is_unset() {
        return Val::Null;
    }
    obj(vec![
        ("start_line", Val::Int(s.start_line as i64)),
        ("start_col", Val::Int(s.start_col as i64)),
        ("end_line", Val::Int(s.end_line as i64)),
        ("end_col", Val::Int(s.end_col as i64)),
    ])
}

/// An expression as JSON — the opt-in `expr` field (PROP-2026-0008 R12).
///
/// Every variant of [`Expr`] has a `kind`; the rest of the object depends on it. Only written when
/// the caller asks, so the default document is byte-identical to what it was before this existed.
pub fn expr_val(e: &Expr) -> Val {
    fn kind(k: &str, mut rest: Vec<(&str, Val)>) -> Val {
        rest.insert(0, ("kind", Val::Str(k.to_string())));
        obj(rest)
    }
    fn step_val(s: &PathStep) -> Val {
        if s.is_index {
            match &s.index {
                Some(ix) => obj(vec![("index", expr_val(ix))]),
                None => obj(vec![("index", Val::Null)]),
            }
        } else {
            obj(vec![("field", Val::Str(s.field.clone()))])
        }
    }
    match e {
        Expr::Number(n) => {
            if n.is_int {
                kind("number", vec![("is_int", Val::Bool(true)), ("int", Val::Int(n.i))])
            } else {
                kind("number", vec![("is_int", Val::Bool(false)), ("float", Val::Float(n.f))])
            }
        }
        Expr::Str(s) => kind("str", vec![("value", Val::Str(s.clone()))]),
        Expr::Bool(b) => kind("bool", vec![("value", Val::Bool(*b))]),
        Expr::Null => kind("null", vec![]),
        Expr::Var(steps) => kind("var", vec![("path", Val::List(steps.iter().map(step_val).collect()))]),
        Expr::Call(c) => kind(
            "call",
            vec![
                ("name", Val::List(c.name.iter().map(|n| Val::Str(n.clone())).collect())),
                ("args", Val::List(c.args.iter().map(expr_val).collect())),
            ],
        ),
        Expr::Compare(c) => kind(
            "compare",
            vec![("op", Val::Str(c.op.clone())), ("left", expr_val(&c.left)), ("right", expr_val(&c.right))],
        ),
        Expr::Binary(b) => kind(
            "binary",
            vec![("op", Val::Str(b.op.clone())), ("left", expr_val(&b.left)), ("right", expr_val(&b.right))],
        ),
        Expr::Not(x) => kind("not", vec![("operand", expr_val(x))]),
        Expr::Unary(u) => kind("unary", vec![("op", Val::Str(u.op.clone())), ("operand", expr_val(&u.operand))]),
        Expr::List(items) => kind("list", vec![("items", Val::List(items.iter().map(expr_val).collect()))]),
        Expr::Obj(o) => kind(
            "obj",
            vec![
                ("keys", Val::List(o.keys.iter().map(|k| Val::Str(k.clone())).collect())),
                ("values", Val::List(o.vals.iter().map(expr_val).collect())),
            ],
        ),
    }
}

fn capture_val(c: &CaptureValue, ex: bool) -> Val {
    let mut pairs = vec![
        ("text", Val::Str(c.text.clone())),
        ("is_expr", Val::Bool(c.is_expr)),
        ("span", span_val(c.span)),
        ("sub", Val::List(c.sub.iter().map(|f| func_call_val(f, ex)).collect())),
    ];
    if ex {
        pairs.push((
            "expr",
            match &c.expr {
                Some(e) => expr_val(e),
                None => Val::Null,
            },
        ));
    }
    obj(pairs)
}

fn func_call_val(f: &FuncCall, ex: bool) -> Val {
    let mut captures: BTreeMap<String, Val> = BTreeMap::new();
    for (k, v) in &f.captures {
        captures.insert(k.clone(), capture_val(v, ex));
    }
    let mut sections: BTreeMap<String, Val> = BTreeMap::new();
    for (k, v) in &f.sections {
        sections.insert(k.clone(), block_val(v, ex));
    }
    obj(vec![
        ("func", Val::Str(f.func.clone())),
        ("span", span_val(f.span)),
        ("captures", Val::Obj(captures)),
        (
            "leading_comments",
            Val::List(f.leading_comments.iter().map(|s| span_val(*s)).collect()),
        ),
        (
            "body",
            match &f.body {
                Some(b) => block_val(b, ex),
                None => Val::Null,
            },
        ),
        (
            "closer",
            match &f.closer {
                Some(c) => func_call_val(c, ex),
                None => Val::Null,
            },
        ),
        ("sections", Val::Obj(sections)),
    ])
}

fn error_node_val(e: &ErrorNode) -> Val {
    obj(vec![
        ("span", span_val(e.span)),
        ("diagnostic_index", Val::Int(e.diagnostic_index as i64)),
        (
            "tokens",
            Val::List(e.tokens.iter().map(|t| Val::Str(t.text.clone())).collect()),
        ),
    ])
}

fn block_val(b: &Block, ex: bool) -> Val {
    obj(vec![
        ("stmts", Val::List(b.stmts.iter().map(|f| func_call_val(f, ex)).collect())),
        ("errors", Val::List(b.errors.iter().map(error_node_val).collect())),
        ("is_verbatim", Val::Bool(b.is_verbatim)),
        ("verbatim_text", Val::Str(b.verbatim_text.clone())),
    ])
}

fn diagnostic_val(d: &Diagnostic) -> Val {
    obj(vec![
        (
            "severity",
            Val::Str(
                match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                }
                .to_string(),
            ),
        ),
        ("code", Val::Str(d.code.to_string())),
        ("message", Val::Str(d.full_message())),
        ("primary", span_val(d.primary)),
        (
            "labels",
            Val::List(
                d.labels
                    .iter()
                    .map(|l| obj(vec![("span", span_val(l.span)), ("text", Val::Str(l.text.clone()))]))
                    .collect(),
            ),
        ),
        (
            "help",
            match &d.help {
                Some(h) => Val::Str(h.clone()),
                None => Val::Null,
            },
        ),
        (
            "context",
            Val::List(
                d.context
                    .iter()
                    .map(|c| {
                        obj(vec![
                            ("shape", Val::Str(c.shape.clone())),
                            ("arg_index", Val::Int(c.arg_index as i64)),
                            ("arg_name", Val::Str(c.arg_name.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// The whole parse result as a `Val`, ready to marshal.
pub fn parse_result_val(r: &ParseResult) -> Val {
    parse_result_val_with(r, false)
}

/// Like [`parse_result_val`]; with `include_expr` each capture also carries its typed `expr`
/// (PROP-2026-0008 R12). `false` is byte-identical to the document before the option existed.
pub fn parse_result_val_with(r: &ParseResult, include_expr: bool) -> Val {
    obj(vec![
        ("schema_version", Val::Int(SCHEMA_VERSION)),
        ("tree", block_val(&r.tree, include_expr)),
        (
            "diagnostics",
            Val::List(r.diagnostics.iter().map(diagnostic_val).collect()),
        ),
    ])
}

/// Compact JSON.
pub fn to_json(r: &ParseResult) -> String {
    to_json_with(r, false)
}

/// Compact JSON, optionally with the `expr` field.
pub fn to_json_with(r: &ParseResult, include_expr: bool) -> String {
    gojson::marshal(&parse_result_val_with(r, include_expr))
}

/// Indented JSON, for a human reading `capy ast --json`.
pub fn to_json_pretty(r: &ParseResult) -> String {
    to_json_pretty_with(r, false)
}

/// Indented JSON, optionally with the `expr` field.
pub fn to_json_pretty_with(r: &ParseResult, include_expr: bool) -> String {
    gojson::marshal_indent(&parse_result_val_with(r, include_expr), "", "  ")
}
