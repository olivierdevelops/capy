//! C ABI for the Capy engine (PROP-2026-0008, ADR-0005).
//!
//! ```text
//!   C host ──► CapyLibrary (compile once, read-only, thread-shareable)
//!                 │ capy_parse
//!                 ▼
//!              CapyParse (single owner) ──► ids (u32) + borrowed UTF-8 (ptr, len)
//! ```
//!
//! Rules every export follows:
//!
//! * strings go in as `(ptr, len)`, never NUL-terminated; they come out as [`CapyStr`], borrowed from the
//!   handle they were read from and dead after that handle is freed;
//! * an id is valid for the parse that produced it; a bad id yields `0` / `{NULL, 0}`, never undefined behaviour;
//! * a Rust panic never crosses the boundary: it poisons the handle (see [`arena::with`]).
//!
//! Freeing a handle twice, using one after its free, or passing a pointer/length that does not describe
//! readable memory is a caller contract violation that no library can detect.
#![allow(clippy::missing_safety_doc)]

mod arena;
mod diag;
mod expr;
mod json;

pub use arena::{
    CapyId, CapyParse, CapySpan, CapyStr, CAPY_E_ARG, CAPY_E_KIND, CAPY_E_LIMIT, CAPY_E_LOAD, CAPY_E_NULL, CAPY_E_PANIC,
    CAPY_E_UTF8, CAPY_OK,
};
pub use diag::*;
pub use expr::*;
pub use json::*;
use arena::with;
use capy_core::capy::Library;
use capy_core::domain::errors::CapyError as CoreError;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// The ABI version. Within one version, changes are additive only.
pub const CAPY_ABI_VERSION: u32 = 1;

/// A compiled grammar. Read-only after creation; may be shared by many threads.
pub struct CapyLibrary {
    lib: Library,
}

/// Why a grammar failed to load.
pub struct CapyError {
    msg: String,
    hint: String,
    file: String,
    line: u32,
    col: u32,
}

// Test-only: make the next library load or parse panic, to prove the boundary catches it (R9, T-08).
#[cfg(test)]
thread_local! {
    static FORCE_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[inline]
fn test_panic_point() {
    #[cfg(test)]
    if FORCE_PANIC.with(|f| f.replace(false)) {
        panic!("forced panic (test)");
    }
}

pub(crate) unsafe fn put<T>(out: *mut T, v: T) {
    if !out.is_null() {
        *out = v;
    }
}

pub(crate) unsafe fn bytes<'a>(p: *const u8, n: usize) -> Option<&'a [u8]> {
    if n == 0 {
        Some(&[])
    } else if p.is_null() {
        None
    } else {
        Some(std::slice::from_raw_parts(p, n))
    }
}

fn sat(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------------------------------------- version

#[no_mangle]
pub extern "C" fn capy_abi_version() -> u32 {
    CAPY_ABI_VERSION
}

/// 1 when this build converts a Rust panic into `CAPY_E_PANIC`; 0 when built with `panic = "abort"`
/// (a panic then aborts the process). Build with `--profile ffi` to get 1.
#[no_mangle]
pub extern "C" fn capy_panics_are_caught() -> i32 {
    cfg!(panic = "unwind") as i32
}

// -------------------------------------------------------------------------------------------- library

unsafe fn load(
    f: impl FnOnce() -> Result<Library, CoreError>,
    out: *mut *mut CapyLibrary,
    err: *mut *mut CapyError,
) -> i32 {
    if out.is_null() {
        return CAPY_E_NULL;
    }
    put(out, std::ptr::null_mut());
    put(err, std::ptr::null_mut());
    match catch_unwind(AssertUnwindSafe(|| {
        test_panic_point();
        f()
    })) {
        Ok(Ok(lib)) => {
            put(out, Box::into_raw(Box::new(CapyLibrary { lib })));
            CAPY_OK
        }
        Ok(Err(e)) => {
            put(
                err,
                Box::into_raw(Box::new(CapyError {
                    msg: e.msg,
                    hint: e.hint,
                    file: e.file,
                    line: sat(e.line),
                    col: sat(e.col),
                })),
            );
            CAPY_E_LOAD
        }
        Err(_) => CAPY_E_PANIC,
    }
}

/// Compile a grammar from UTF-8 text. On failure `*out` is NULL, the status is `CAPY_E_LOAD` and, when
/// `err` is non-NULL, `*err` holds the reason (free it with `capy_error_free`).
#[no_mangle]
pub unsafe extern "C" fn capy_library_new(
    grammar: *const u8,
    n: usize,
    out: *mut *mut CapyLibrary,
    err: *mut *mut CapyError,
) -> i32 {
    let Some(b) = bytes(grammar, n) else { return CAPY_E_NULL };
    let Ok(text) = std::str::from_utf8(b) else { return CAPY_E_UTF8 };
    load(|| Library::new(text), out, err)
}

/// Compile a grammar from a file path (UTF-8).
#[no_mangle]
pub unsafe extern "C" fn capy_library_from_file(
    path: *const u8,
    n: usize,
    out: *mut *mut CapyLibrary,
    err: *mut *mut CapyError,
) -> i32 {
    let Some(b) = bytes(path, n) else { return CAPY_E_NULL };
    let Ok(text) = std::str::from_utf8(b) else { return CAPY_E_UTF8 };
    load(|| Library::from_file(text), out, err)
}

#[no_mangle]
pub unsafe extern "C" fn capy_library_free(lib: *mut CapyLibrary) {
    if !lib.is_null() {
        drop(Box::from_raw(lib));
    }
}

// ----------------------------------------------------------------------------------------------- error

/// A handle from C as a reference. NULL is `None`; anything else is the caller's promise that it came
/// from this library and has not been freed.
fn deref<'a, T>(p: *const T) -> Option<&'a T> {
    // SAFETY: see above; the only unchecked part is the caller contract documented in the header.
    unsafe { p.as_ref() }
}

fn err_str(e: *const CapyError, f: impl FnOnce(&CapyError) -> &str) -> CapyStr {
    deref(e).map(|e| CapyStr::of(f(e))).unwrap_or(CapyStr::NONE)
}

#[no_mangle]
pub extern "C" fn capy_error_msg(e: *const CapyError) -> CapyStr {
    err_str(e, |e| &e.msg)
}
#[no_mangle]
pub extern "C" fn capy_error_hint(e: *const CapyError) -> CapyStr {
    err_str(e, |e| &e.hint)
}
#[no_mangle]
pub extern "C" fn capy_error_file(e: *const CapyError) -> CapyStr {
    err_str(e, |e| &e.file)
}
#[no_mangle]
pub extern "C" fn capy_error_line(e: *const CapyError) -> u32 {
    deref(e).map(|e| e.line).unwrap_or(0)
}
#[no_mangle]
pub extern "C" fn capy_error_col(e: *const CapyError) -> u32 {
    deref(e).map(|e| e.col).unwrap_or(0)
}
#[no_mangle]
pub unsafe extern "C" fn capy_error_free(e: *mut CapyError) {
    if !e.is_null() {
        drop(Box::from_raw(e));
    }
}

// ------------------------------------------------------------------------------------------------ parse

/// Parse `src` (UTF-8; invalid bytes are replaced, as in the wasm ABI). Returns a handle for every input —
/// a broken script yields diagnostics, not NULL. NULL only with a non-OK `*status`.
#[no_mangle]
pub unsafe extern "C" fn capy_parse(lib: *const CapyLibrary, src: *const u8, n: usize, status: *mut i32) -> *mut CapyParse {
    let (st, p) = parse_impl(lib, src, n);
    put(status, st);
    p
}

unsafe fn parse_impl(lib: *const CapyLibrary, src: *const u8, n: usize) -> (i32, *mut CapyParse) {
    if lib.is_null() {
        return (CAPY_E_NULL, std::ptr::null_mut());
    }
    let Some(b) = bytes(src, n) else { return (CAPY_E_NULL, std::ptr::null_mut()) };
    let r = catch_unwind(AssertUnwindSafe(|| {
        test_panic_point();
        let text = String::from_utf8_lossy(b);
        CapyParse::build((*lib).lib.parse(&text))
    }));
    match r {
        Ok(Ok(p)) => (CAPY_OK, Box::into_raw(Box::new(p))),
        Ok(Err(code)) => (code, std::ptr::null_mut()),
        Err(_) => (CAPY_E_PANIC, std::ptr::null_mut()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn capy_parse_free(p: *mut CapyParse) {
    if !p.is_null() {
        drop(Box::from_raw(p));
    }
}

/// 1 when nothing failed to parse, else 0 (also 0 for a NULL or poisoned handle).
#[no_mangle]
pub extern "C" fn capy_is_clean(p: *const CapyParse) -> i32 {
    with(p, 0, |p| p.result().is_clean() as i32)
}

/// 1 once a panic has been caught on this handle; every accessor then returns its zero value.
#[no_mangle]
pub extern "C" fn capy_parse_is_poisoned(p: *const CapyParse) -> i32 {
    deref(p).map(|p| p.is_poisoned() as i32).unwrap_or(0)
}

/// The top-level block id.
#[no_mangle]
pub extern "C" fn capy_root(p: *const CapyParse) -> CapyId {
    with(p, 0, |p| p.root())
}

// ------------------------------------------------------------------------------------------------ block

#[no_mangle]
pub extern "C" fn capy_block_len(p: *const CapyParse, b: CapyId) -> u32 {
    with(p, 0, |p| p.block(b).map(|(e, _)| e.a.1).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_block_stmt(p: *const CapyParse, b: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| p.block(b).map(|(e, _)| p.kid(e.a, i)).unwrap_or(0))
}

/// 1 and the raw text when the block is `block_verbatim`; else 0 and `{NULL, 0}`.
#[no_mangle]
pub unsafe extern "C" fn capy_block_verbatim(p: *const CapyParse, b: CapyId, out: *mut CapyStr) -> i32 {
    let (v, s) = with(p, (0, CapyStr::NONE), |p| match p.block(b) {
        Some((_, blk)) if blk.is_verbatim => (1, CapyStr::of(&blk.verbatim_text)),
        _ => (0, CapyStr::NONE),
    });
    put(out, s);
    v
}

#[no_mangle]
pub extern "C" fn capy_block_error_count(p: *const CapyParse, b: CapyId) -> u32 {
    with(p, 0, |p| p.block(b).map(|(e, _)| e.b.1).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_block_error(p: *const CapyParse, b: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| p.block(b).map(|(e, _)| p.kid(e.b, i)).unwrap_or(0))
}

// --------------------------------------------------------------------------------------------- statement

#[no_mangle]
pub extern "C" fn capy_node_func(p: *const CapyParse, n: CapyId) -> CapyStr {
    with(p, CapyStr::NONE, |p| p.stmt(n).map(|(_, f)| CapyStr::of(&f.func)).unwrap_or(CapyStr::NONE))
}

#[no_mangle]
pub unsafe extern "C" fn capy_node_span(p: *const CapyParse, n: CapyId, out: *mut CapySpan) -> i32 {
    let (st, sp) = with(p, (CAPY_E_NULL, CapySpan::default()), |p| match p.stmt(n) {
        Some((_, f)) => (CAPY_OK, CapySpan::of(f.span)),
        None => (CAPY_E_KIND, CapySpan::default()),
    });
    put(out, sp);
    st
}

#[no_mangle]
pub extern "C" fn capy_node_line(p: *const CapyParse, n: CapyId) -> u32 {
    with(p, 0, |p| p.stmt(n).map(|(_, f)| sat(f.line)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_node_col(p: *const CapyParse, n: CapyId) -> u32 {
    with(p, 0, |p| p.stmt(n).map(|(_, f)| sat(f.col)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_node_capture_count(p: *const CapyParse, n: CapyId) -> u32 {
    with(p, 0, |p| p.stmt(n).map(|(e, _)| e.a.1).unwrap_or(0))
}

/// The `i`-th capture name, in the same (sorted) order as `capy ast --json`.
#[no_mangle]
pub extern "C" fn capy_node_capture_name(p: *const CapyParse, n: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.stmt(n)
            .and_then(|(_, f)| f.captures.keys().nth(i as usize))
            .map(|k| CapyStr::of(k))
            .unwrap_or(CapyStr::NONE)
    })
}

/// The capture named `name`, or 0.
#[no_mangle]
pub unsafe extern "C" fn capy_node_capture(p: *const CapyParse, n: CapyId, name: *const u8, nlen: usize) -> CapyId {
    let Some(nb) = bytes(name, nlen) else { return 0 };
    let Ok(nm) = std::str::from_utf8(nb) else { return 0 };
    with(p, 0, |p| match p.stmt(n) {
        Some((e, f)) => match f.captures.keys().position(|k| k == nm) {
            Some(i) => p.kid(e.a, i as u32),
            None => 0,
        },
        None => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_node_body(p: *const CapyParse, n: CapyId) -> CapyId {
    with(p, 0, |p| p.stmt(n).map(|(e, _)| p.out(e.x)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_node_closer(p: *const CapyParse, n: CapyId) -> CapyId {
    with(p, 0, |p| p.stmt(n).map(|(e, _)| p.out(e.y)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_node_section_count(p: *const CapyParse, n: CapyId) -> u32 {
    with(p, 0, |p| p.stmt(n).map(|(e, _)| e.b.1).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_node_section_name(p: *const CapyParse, n: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.stmt(n)
            .and_then(|(_, f)| f.sections.keys().nth(i as usize))
            .map(|k| CapyStr::of(k))
            .unwrap_or(CapyStr::NONE)
    })
}

#[no_mangle]
pub extern "C" fn capy_node_section_at(p: *const CapyParse, n: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| p.stmt(n).map(|(e, _)| p.kid(e.b, i)).unwrap_or(0))
}

#[no_mangle]
pub unsafe extern "C" fn capy_node_section(p: *const CapyParse, n: CapyId, name: *const u8, nlen: usize) -> CapyId {
    let Some(nb) = bytes(name, nlen) else { return 0 };
    let Ok(nm) = std::str::from_utf8(nb) else { return 0 };
    with(p, 0, |p| match p.stmt(n) {
        Some((e, f)) => match f.sections.keys().position(|k| k == nm) {
            Some(i) => p.kid(e.b, i as u32),
            None => 0,
        },
        None => 0,
    })
}

#[no_mangle]
pub extern "C" fn capy_node_comment_count(p: *const CapyParse, n: CapyId) -> u32 {
    with(p, 0, |p| p.stmt(n).map(|(_, f)| sat(f.leading_comments.len())).unwrap_or(0))
}

#[no_mangle]
pub unsafe extern "C" fn capy_node_comment_span(p: *const CapyParse, n: CapyId, i: u32, out: *mut CapySpan) -> i32 {
    let (st, sp) = with(p, (CAPY_E_NULL, CapySpan::default()), |p| match p.stmt(n) {
        Some((_, f)) => match f.leading_comments.get(i as usize) {
            Some(s) => (CAPY_OK, CapySpan::of(*s)),
            None => (CAPY_E_ARG, CapySpan::default()),
        },
        None => (CAPY_E_KIND, CapySpan::default()),
    });
    put(out, sp);
    st
}

// ------------------------------------------------------------------------------------------------ capture

#[no_mangle]
pub extern "C" fn capy_cap_text(p: *const CapyParse, c: CapyId) -> CapyStr {
    with(p, CapyStr::NONE, |p| p.cap(c).map(|(_, v)| CapyStr::of(&v.text)).unwrap_or(CapyStr::NONE))
}

#[no_mangle]
pub unsafe extern "C" fn capy_cap_span(p: *const CapyParse, c: CapyId, out: *mut CapySpan) -> i32 {
    let (st, sp) = with(p, (CAPY_E_NULL, CapySpan::default()), |p| match p.cap(c) {
        Some((_, v)) => (CAPY_OK, CapySpan::of(v.span)),
        None => (CAPY_E_KIND, CapySpan::default()),
    });
    put(out, sp);
    st
}

#[no_mangle]
pub extern "C" fn capy_cap_is_expr(p: *const CapyParse, c: CapyId) -> i32 {
    with(p, 0, |p| p.cap(c).map(|(_, v)| v.is_expr as i32).unwrap_or(0))
}

/// The expression id behind an expression-typed capture, or 0.
#[no_mangle]
pub extern "C" fn capy_cap_expr(p: *const CapyParse, c: CapyId) -> CapyId {
    with(p, 0, |p| p.cap(c).map(|(e, _)| p.out(e.x)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_cap_sub_count(p: *const CapyParse, c: CapyId) -> u32 {
    with(p, 0, |p| p.cap(c).map(|(e, _)| e.a.1).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_cap_sub(p: *const CapyParse, c: CapyId, i: u32) -> CapyId {
    with(p, 0, |p| p.cap(c).map(|(e, _)| p.kid(e.a, i)).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R9 / T-08 — a panic during a load or a parse comes back as `CAPY_E_PANIC`, never as an unwind into C.
    #[test]
    #[cfg(panic = "unwind")]
    fn panics_in_load_and_parse_become_status_codes() {
        let g = b"extension txt\n";
        let (mut lib, mut err) = (std::ptr::null_mut(), std::ptr::null_mut());

        FORCE_PANIC.with(|f| f.set(true));
        assert_eq!(unsafe { capy_library_new(g.as_ptr(), g.len(), &mut lib, &mut err) }, CAPY_E_PANIC);
        assert!(lib.is_null() && err.is_null());

        // the next call is unaffected: the flag was one-shot and nothing is left poisoned
        assert_eq!(unsafe { capy_library_new(g.as_ptr(), g.len(), &mut lib, &mut err) }, CAPY_OK);

        FORCE_PANIC.with(|f| f.set(true));
        let mut st = -1;
        let p = unsafe { capy_parse(lib, b"x".as_ptr(), 1, &mut st) };
        assert!(p.is_null());
        assert_eq!(st, CAPY_E_PANIC);

        let mut st = -1;
        let p = unsafe { capy_parse(lib, b"".as_ptr(), 0, &mut st) };
        assert!(!p.is_null() && st == CAPY_OK);
        unsafe {
            capy_parse_free(p);
            capy_library_free(lib);
        }
    }
}
