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
use crate::domain::ast::{Block, CaptureValue, ErrorNode, FuncCall, Span};
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

fn capture_val(c: &CaptureValue) -> Val {
    obj(vec![
        ("text", Val::Str(c.text.clone())),
        ("is_expr", Val::Bool(c.is_expr)),
        ("span", span_val(c.span)),
        ("sub", Val::List(c.sub.iter().map(func_call_val).collect())),
    ])
}

fn func_call_val(f: &FuncCall) -> Val {
    let mut captures: BTreeMap<String, Val> = BTreeMap::new();
    for (k, v) in &f.captures {
        captures.insert(k.clone(), capture_val(v));
    }
    let mut sections: BTreeMap<String, Val> = BTreeMap::new();
    for (k, v) in &f.sections {
        sections.insert(k.clone(), block_val(v));
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
                Some(b) => block_val(b),
                None => Val::Null,
            },
        ),
        (
            "closer",
            match &f.closer {
                Some(c) => func_call_val(c),
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

fn block_val(b: &Block) -> Val {
    obj(vec![
        ("stmts", Val::List(b.stmts.iter().map(func_call_val).collect())),
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
    obj(vec![
        ("schema_version", Val::Int(SCHEMA_VERSION)),
        ("tree", block_val(&r.tree)),
        (
            "diagnostics",
            Val::List(r.diagnostics.iter().map(diagnostic_val).collect()),
        ),
    ])
}

/// Compact JSON.
pub fn to_json(r: &ParseResult) -> String {
    gojson::marshal(&parse_result_val(r))
}

/// Indented JSON, for a human reading `capy ast --json`.
pub fn to_json_pretty(r: &ParseResult) -> String {
    gojson::marshal_indent(&parse_result_val(r), "", "  ")
}
