//! Expression accessors — the typed `Expr` tree that the AST JSON omits by default (P-04).
//!
//! Children by kind (index order):
//!
//! | kind    | children                              | `capy_expr_str`   |
//! |---------|---------------------------------------|-------------------|
//! | binary  | left, right                           | operator          |
//! | compare | left, right                           | operator          |
//! | unary   | operand                               | operator          |
//! | not     | operand                               |                   |
//! | list    | items                                 |                   |
//! | call    | arguments (name via `capy_expr_name`) |                   |
//! | obj     | values (keys via `capy_obj_key`)      |                   |
//! | var     | one slot per path step; 0 for a field step, the index expression for an index step | |
//! | str     |                                       | the string        |

use crate::arena::{with, CapyId, CapyStr};
use crate::{put, CapyParse};
use capy_core::domain::ast::Expr;

pub const CAPY_EXPR_NUMBER: u8 = 1;
pub const CAPY_EXPR_STR: u8 = 2;
pub const CAPY_EXPR_BOOL: u8 = 3;
pub const CAPY_EXPR_NULL: u8 = 4;
pub const CAPY_EXPR_VAR: u8 = 5;
pub const CAPY_EXPR_CALL: u8 = 6;
pub const CAPY_EXPR_COMPARE: u8 = 7;
pub const CAPY_EXPR_BINARY: u8 = 8;
pub const CAPY_EXPR_NOT: u8 = 9;
pub const CAPY_EXPR_UNARY: u8 = 10;
pub const CAPY_EXPR_LIST: u8 = 11;
pub const CAPY_EXPR_OBJ: u8 = 12;

fn sat(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// One of the `CAPY_EXPR_*` kinds, or 0 for an invalid id.
#[no_mangle]
pub extern "C" fn capy_expr_kind(p: *const CapyParse, e: CapyId) -> u8 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, x)) => match x {
            Expr::Number(_) => CAPY_EXPR_NUMBER,
            Expr::Str(_) => CAPY_EXPR_STR,
            Expr::Bool(_) => CAPY_EXPR_BOOL,
            Expr::Null => CAPY_EXPR_NULL,
            Expr::Var(_) => CAPY_EXPR_VAR,
            Expr::Call(_) => CAPY_EXPR_CALL,
            Expr::Compare(_) => CAPY_EXPR_COMPARE,
            Expr::Binary(_) => CAPY_EXPR_BINARY,
            Expr::Not(_) => CAPY_EXPR_NOT,
            Expr::Unary(_) => CAPY_EXPR_UNARY,
            Expr::List(_) => CAPY_EXPR_LIST,
            Expr::Obj(_) => CAPY_EXPR_OBJ,
        },
        None => 0,
    })
}

/// 1 and `*out` when the expression is an integer number; 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn capy_expr_int(p: *const CapyParse, e: CapyId, out: *mut i64) -> i32 {
    let r = with(p, None, |p| match p.expr(e) {
        Some((_, Expr::Number(n))) if n.is_int => Some(n.i),
        _ => None,
    });
    match r {
        Some(v) => {
            put(out, v);
            1
        }
        None => 0,
    }
}

/// The float value of a non-integer number; 0.0 for anything else.
#[no_mangle]
pub extern "C" fn capy_expr_float(p: *const CapyParse, e: CapyId) -> f64 {
    with(p, 0.0, |p| match p.expr(e) {
        Some((_, Expr::Number(n))) if !n.is_int => n.f,
        _ => 0.0,
    })
}

#[no_mangle]
pub extern "C" fn capy_expr_bool(p: *const CapyParse, e: CapyId) -> i32 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, Expr::Bool(b))) => *b as i32,
        _ => 0,
    })
}

/// A string literal's value, or the operator text of `binary`, `compare` and `unary`.
#[no_mangle]
pub extern "C" fn capy_expr_str(p: *const CapyParse, e: CapyId) -> CapyStr {
    with(p, CapyStr::NONE, |p| match p.expr(e) {
        Some((_, Expr::Str(s))) => CapyStr::of(s),
        Some((_, Expr::Binary(b))) => CapyStr::of(&b.op),
        Some((_, Expr::Compare(c))) => CapyStr::of(&c.op),
        Some((_, Expr::Unary(u))) => CapyStr::of(&u.op),
        _ => CapyStr::NONE,
    })
}

#[no_mangle]
pub extern "C" fn capy_expr_child_count(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, 0, |p| p.expr(e).map(|(en, _)| en.a.1).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_expr_child(p: *const CapyParse, e: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| p.expr(e).map(|(en, _)| p.kid(en.a, i)).unwrap_or(0))
}

/// Segments of a call's dotted name.
#[no_mangle]
pub extern "C" fn capy_expr_name_count(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, Expr::Call(c))) => sat(c.name.len()),
        _ => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_expr_name(p: *const CapyParse, e: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| match p.expr(e) {
        Some((_, Expr::Call(c))) => c.name.get(i as usize).map(|s| CapyStr::of(s)).unwrap_or(CapyStr::NONE),
        _ => CapyStr::NONE,
    })
}

#[no_mangle]
pub extern "C" fn capy_var_step_count(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, Expr::Var(s))) => sat(s.len()),
        _ => 0,
    })
}

/// 1 for an index step `[expr]`, 0 for a field step `.name` (and for an invalid step).
#[no_mangle]
pub extern "C" fn capy_var_step_is_index(p: *const CapyParse, e: CapyId, i: u32) -> i32 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, Expr::Var(s))) => s.get(i as usize).map(|st| st.is_index as i32).unwrap_or(0),
        _ => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_var_step_field(p: *const CapyParse, e: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| match p.expr(e) {
        Some((_, Expr::Var(s))) => match s.get(i as usize) {
            Some(st) if !st.is_index => CapyStr::of(&st.field),
            _ => CapyStr::NONE,
        },
        _ => CapyStr::NONE,
    })
}

/// The index expression of step `i`, or 0.
#[no_mangle]
pub extern "C" fn capy_var_step_index(p: *const CapyParse, e: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| match p.expr(e) {
        Some((en, Expr::Var(_))) => p.kid(en.a, i),
        _ => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_obj_key_count(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, 0, |p| match p.expr(e) {
        Some((_, Expr::Obj(o))) => sat(o.keys.len()),
        _ => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_obj_key(p: *const CapyParse, e: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| match p.expr(e) {
        Some((_, Expr::Obj(o))) => o.keys.get(i as usize).map(|s| CapyStr::of(s)).unwrap_or(CapyStr::NONE),
        _ => CapyStr::NONE,
    })
}
