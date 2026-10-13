//! A walker that reads a parse ONLY through the exported C functions and rebuilds the document that
//! `capy_core::domain::ast_json` writes. Equality with the Rust writer is the proof that the C view is
//! complete and in the same order (R4, R5, R6, R10, R13).
#![allow(dead_code)]

use capy_core::capy::ParseResult;
use capy_core::domain::errors::ContextFrame;
use capy_core::domain::val::Val;
use capy_ffi::*;
use std::collections::BTreeMap;

pub fn s(c: CapyStr) -> String {
    if c.ptr.is_null() {
        String::new()
    } else {
        // SAFETY: the pointer/len pair was just returned by the library and the handle is alive.
        String::from_utf8(unsafe { std::slice::from_raw_parts(c.ptr, c.len) }.to_vec()).expect("utf-8")
    }
}

pub fn opt(c: CapyStr) -> Option<String> {
    if c.ptr.is_null() {
        None
    } else {
        Some(s(c))
    }
}

/// Owns a parse handle; frees it on drop.
pub struct Parse(pub *mut CapyParse);
impl Drop for Parse {
    fn drop(&mut self) {
        unsafe { capy_parse_free(self.0) }
    }
}

/// Wrap an in-memory `ParseResult` as a handle (how the tests reach `Expr` kinds a grammar cannot yet produce).
pub fn handle_of(r: ParseResult) -> Parse {
    Parse(Box::into_raw(Box::new(CapyParse::build(r).expect("build"))))
}

fn obj(pairs: Vec<(&str, Val)>) -> Val {
    let mut m = BTreeMap::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    Val::Obj(m)
}

fn span_of(f: impl FnOnce(*mut CapySpan) -> i32) -> Val {
    let mut sp = CapySpan::default();
    let _ = f(&mut sp);
    if sp.start_line == 0 {
        return Val::Null;
    }
    obj(vec![
        ("start_line", Val::Int(sp.start_line as i64)),
        ("start_col", Val::Int(sp.start_col as i64)),
        ("end_line", Val::Int(sp.end_line as i64)),
        ("end_col", Val::Int(sp.end_col as i64)),
    ])
}

pub fn expr(p: *const CapyParse, e: u32) -> Val {
    let kind = |k: &str, mut rest: Vec<(&str, Val)>| {
        rest.insert(0, ("kind", Val::Str(k.to_string())));
        obj(rest)
    };
    let child = |i: u32| expr(p, capy_expr_child(p, e, i));
    let n = capy_expr_child_count(p, e);
    match capy_expr_kind(p, e) {
        CAPY_EXPR_NUMBER => {
            let mut v = 0i64;
            if unsafe { capy_expr_int(p, e, &mut v) } == 1 {
                kind("number", vec![("is_int", Val::Bool(true)), ("int", Val::Int(v))])
            } else {
                kind("number", vec![("is_int", Val::Bool(false)), ("float", Val::Float(capy_expr_float(p, e)))])
            }
        }
        CAPY_EXPR_STR => kind("str", vec![("value", Val::Str(s(capy_expr_str(p, e))))]),
        CAPY_EXPR_BOOL => kind("bool", vec![("value", Val::Bool(capy_expr_bool(p, e) == 1))]),
        CAPY_EXPR_NULL => kind("null", vec![]),
        CAPY_EXPR_VAR => {
            let steps = (0..capy_var_step_count(p, e))
                .map(|i| {
                    if capy_var_step_is_index(p, e, i) == 1 {
                        let ix = capy_var_step_index(p, e, i);
                        obj(vec![("index", if ix == 0 { Val::Null } else { expr(p, ix) })])
                    } else {
                        obj(vec![("field", Val::Str(s(capy_var_step_field(p, e, i))))])
                    }
                })
                .collect();
            kind("var", vec![("path", Val::List(steps))])
        }
        CAPY_EXPR_CALL => kind(
            "call",
            vec![
                (
                    "name",
                    Val::List((0..capy_expr_name_count(p, e)).map(|i| Val::Str(s(capy_expr_name(p, e, i)))).collect()),
                ),
                ("args", Val::List((0..n).map(child).collect())),
            ],
        ),
        CAPY_EXPR_COMPARE => {
            kind("compare", vec![("op", Val::Str(s(capy_expr_str(p, e)))), ("left", child(0)), ("right", child(1))])
        }
        CAPY_EXPR_BINARY => {
            kind("binary", vec![("op", Val::Str(s(capy_expr_str(p, e)))), ("left", child(0)), ("right", child(1))])
        }
        CAPY_EXPR_NOT => kind("not", vec![("operand", child(0))]),
        CAPY_EXPR_UNARY => kind("unary", vec![("op", Val::Str(s(capy_expr_str(p, e)))), ("operand", child(0))]),
        CAPY_EXPR_LIST => kind("list", vec![("items", Val::List((0..n).map(child).collect()))]),
        CAPY_EXPR_OBJ => kind(
            "obj",
            vec![
                (
                    "keys",
                    Val::List((0..capy_obj_key_count(p, e)).map(|i| Val::Str(s(capy_obj_key(p, e, i)))).collect()),
                ),
                ("values", Val::List((0..n).map(child).collect())),
            ],
        ),
        other => panic!("unknown expression kind {other} for id {e}"),
    }
}

fn capture(p: *const CapyParse, c: u32, ex: bool) -> Val {
    let mut pairs = vec![
        ("text", Val::Str(s(capy_cap_text(p, c)))),
        ("is_expr", Val::Bool(capy_cap_is_expr(p, c) == 1)),
        ("span", span_of(|o| unsafe { capy_cap_span(p, c, o) })),
        ("sub", Val::List((0..capy_cap_sub_count(p, c)).map(|i| stmt(p, capy_cap_sub(p, c, i), ex)).collect())),
    ];
    if ex {
        let e = capy_cap_expr(p, c);
        pairs.push(("expr", if e == 0 { Val::Null } else { expr(p, e) }));
    }
    obj(pairs)
}

fn stmt(p: *const CapyParse, n: u32, ex: bool) -> Val {
    let mut caps = BTreeMap::new();
    for i in 0..capy_node_capture_count(p, n) {
        let name = s(capy_node_capture_name(p, n, i));
        // R10: enumeration is in the Rust (BTreeMap, ascending) order, not merely complete
        if let Some(prev) = caps.keys().next_back() {
            assert!(*prev < name, "capture names out of order: {prev} then {name}");
        }
        // the by-name lookup must agree with enumeration
        let by_name = unsafe { capy_node_capture(p, n, name.as_ptr(), name.len()) };
        caps.insert(name, capture(p, by_name, ex));
    }
    let mut secs = BTreeMap::new();
    for i in 0..capy_node_section_count(p, n) {
        let name = s(capy_node_section_name(p, n, i));
        if let Some(prev) = secs.keys().next_back() {
            assert!(*prev < name, "section names out of order: {prev} then {name}");
        }
        let by_name = unsafe { capy_node_section(p, n, name.as_ptr(), name.len()) };
        assert_eq!(by_name, capy_node_section_at(p, n, i), "section by name == by index");
        secs.insert(name, block(p, by_name, ex));
    }
    let body = capy_node_body(p, n);
    let closer = capy_node_closer(p, n);
    obj(vec![
        ("func", Val::Str(s(capy_node_func(p, n)))),
        ("span", span_of(|o| unsafe { capy_node_span(p, n, o) })),
        ("captures", Val::Obj(caps)),
        (
            "leading_comments",
            Val::List(
                (0..capy_node_comment_count(p, n))
                    .map(|i| span_of(|o| unsafe { capy_node_comment_span(p, n, i, o) }))
                    .collect(),
            ),
        ),
        ("body", if body == 0 { Val::Null } else { block(p, body, ex) }),
        ("closer", if closer == 0 { Val::Null } else { stmt(p, closer, ex) }),
        ("sections", Val::Obj(secs)),
    ])
}

fn block(p: *const CapyParse, b: u32, ex: bool) -> Val {
    let mut text = CapyStr { ptr: std::ptr::null(), len: 0 };
    let verb = unsafe { capy_block_verbatim(p, b, &mut text) } == 1;
    obj(vec![
        ("stmts", Val::List((0..capy_block_len(p, b)).map(|i| stmt(p, capy_block_stmt(p, b, i), ex)).collect())),
        (
            "errors",
            Val::List(
                (0..capy_block_error_count(p, b))
                    .map(|i| {
                        let e = capy_block_error(p, b, i);
                        obj(vec![
                            ("span", span_of(|o| unsafe { capy_err_span(p, e, o) })),
                            ("diagnostic_index", Val::Int(capy_err_diag_index(p, e) as i64)),
                            (
                                "tokens",
                                Val::List(
                                    (0..capy_err_token_count(p, e)).map(|j| Val::Str(s(capy_err_token_text(p, e, j)))).collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("is_verbatim", Val::Bool(verb)),
        ("verbatim_text", Val::Str(if verb { s(text) } else { String::new() })),
    ])
}

fn diagnostic(p: *const CapyParse, i: u32) -> Val {
    let sev = match capy_diag_severity(p, i) {
        0 => "error",
        1 => "warning",
        other => panic!("severity {other}"),
    };
    // The JSON `message` is msg plus the LAST context frame's description; compose it from the pieces.
    let mut message = s(capy_diag_message(p, i));
    let nctx = capy_diag_context_count(p, i);
    let mut ctx = Vec::new();
    for j in 0..nctx {
        let (mut shape, mut name) = (CapyStr { ptr: std::ptr::null(), len: 0 }, CapyStr { ptr: std::ptr::null(), len: 0 });
        let mut ix = 0u32;
        assert_eq!(unsafe { capy_diag_context(p, i, j, &mut shape, &mut ix, &mut name) }, CAPY_OK);
        let (shape, name) = (s(shape), s(name));
        if j + 1 == nctx {
            message = format!("{} {}", message, ContextFrame::new(shape.clone(), ix as usize, name.clone()).describe());
        }
        ctx.push(obj(vec![
            ("shape", Val::Str(shape)),
            ("arg_index", Val::Int(ix as i64)),
            ("arg_name", Val::Str(name)),
        ]));
    }
    obj(vec![
        ("severity", Val::Str(sev.to_string())),
        ("code", Val::Str(s(capy_diag_code(p, i)))),
        ("message", Val::Str(message)),
        ("primary", span_of(|o| unsafe { capy_diag_primary(p, i, o) })),
        (
            "labels",
            Val::List(
                (0..capy_diag_label_count(p, i))
                    .map(|j| {
                        let mut sp = CapySpan::default();
                        let text = s(unsafe { capy_diag_label(p, i, j, &mut sp) });
                        obj(vec![
                            (
                                "span",
                                span_of(|o| {
                                    unsafe { *o = sp };
                                    0
                                }),
                            ),
                            ("text", Val::Str(text)),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("help", opt(capy_diag_help(p, i)).map(Val::Str).unwrap_or(Val::Null)),
        ("context", Val::List(ctx)),
    ])
}

/// The whole document, read only through the C functions.
pub fn document(p: *const CapyParse, ex: bool) -> Val {
    obj(vec![
        ("schema_version", Val::Int(1)),
        ("tree", block(p, capy_root(p), ex)),
        ("diagnostics", Val::List((0..capy_diag_count(p)).map(|i| diagnostic(p, i)).collect())),
    ])
}
