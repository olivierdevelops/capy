//! The optional second layer: the whole parse as JSON text (PROP-2026-0008 D2, R12).

use crate::arena::{with, CAPY_E_ARG, CAPY_E_NULL, CAPY_E_PANIC, CAPY_OK};
use crate::{put, CapyParse};
use capy_core::domain::ast_json;

/// Add the typed `expr` field to every capture.
pub const CAPY_JSON_EXPR: u32 = 1;
/// Indent the output like `capy ast --json`.
pub const CAPY_JSON_PRETTY: u32 = 2;

/// Write the document into a new buffer: `*out` / `*len`. Free it with `capy_buf_free(*out, *len)`.
/// Unknown flag bits are refused with `CAPY_E_ARG`.
#[no_mangle]
pub unsafe extern "C" fn capy_parse_to_json(p: *const CapyParse, flags: u32, out: *mut *mut u8, len: *mut usize) -> i32 {
    if out.is_null() || len.is_null() {
        return CAPY_E_NULL;
    }
    put(out, std::ptr::null_mut());
    put(len, 0);
    if flags & !(CAPY_JSON_EXPR | CAPY_JSON_PRETTY) != 0 {
        return CAPY_E_ARG;
    }
    let text = with(p, None, |p| {
        let ex = flags & CAPY_JSON_EXPR != 0;
        Some(if flags & CAPY_JSON_PRETTY != 0 {
            ast_json::to_json_pretty_with(p.result(), ex)
        } else {
            ast_json::to_json_with(p.result(), ex)
        })
    });
    match text {
        Some(t) => {
            let b: Box<[u8]> = t.into_bytes().into_boxed_slice();
            put(len, b.len());
            put(out, Box::into_raw(b) as *mut u8);
            CAPY_OK
        }
        None if p.is_null() => CAPY_E_NULL,
        None => CAPY_E_PANIC,
    }
}

/// Release a buffer returned by `capy_parse_to_json`.
#[no_mangle]
pub unsafe extern "C" fn capy_buf_free(buf: *mut u8, len: usize) {
    if !buf.is_null() {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(buf, len)));
    }
}
