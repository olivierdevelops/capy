//! Diagnostics and error nodes — everything `ParseResult` reports about what did not parse.

use crate::arena::{with, CapyId, CapySpan, CapyStr, CAPY_E_ARG, CAPY_E_KIND, CAPY_E_NULL, CAPY_OK};
use crate::{put, CapyParse};
use capy_core::domain::errors::Severity;

fn sat(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[no_mangle]
pub extern "C" fn capy_diag_count(p: *const CapyParse) -> u32 {
    with(p, 0, |p| sat(p.result().diagnostics.len()))
}

/// `E0001`… — stable, machine-readable.
#[no_mangle]
pub extern "C" fn capy_diag_code(p: *const CapyParse, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.result().diagnostics.get(i as usize).map(|d| CapyStr::of(d.code)).unwrap_or(CapyStr::NONE)
    })
}

/// 0 error, 1 warning, 2 for an invalid index, 3 for a severity this ABI version does not know.
#[no_mangle]
pub extern "C" fn capy_diag_severity(p: *const CapyParse, i: u32) -> i32 {
    with(p, 2, |p| match p.result().diagnostics.get(i as usize) {
        Some(d) => match d.severity {
            Severity::Error => 0,
            Severity::Warning => 1,
            // `Severity` is #[non_exhaustive]: a severity added later reads as "unknown" to ABI 1 hosts.
            _ => 3,
        },
        None => 2,
    })
}

/// The message without its context clause; `capy_diag_context` carries the frames.
#[no_mangle]
pub extern "C" fn capy_diag_message(p: *const CapyParse, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.result().diagnostics.get(i as usize).map(|d| CapyStr::of(&d.msg)).unwrap_or(CapyStr::NONE)
    })
}

/// The help text, or `{NULL, 0}` when there is none.
#[no_mangle]
pub extern "C" fn capy_diag_help(p: *const CapyParse, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.result()
            .diagnostics
            .get(i as usize)
            .and_then(|d| d.help.as_deref())
            .map(CapyStr::of)
            .unwrap_or(CapyStr::NONE)
    })
}

#[no_mangle]
pub unsafe extern "C" fn capy_diag_primary(p: *const CapyParse, i: u32, out: *mut CapySpan) -> i32 {
    let (st, sp) = with(p, (CAPY_E_NULL, CapySpan::default()), |p| match p.result().diagnostics.get(i as usize) {
        Some(d) => (CAPY_OK, CapySpan::of(d.primary)),
        None => (CAPY_E_ARG, CapySpan::default()),
    });
    put(out, sp);
    st
}

#[no_mangle]
pub extern "C" fn capy_diag_label_count(p: *const CapyParse, i: u32) -> u32 {
    with(p, 0, |p| p.result().diagnostics.get(i as usize).map(|d| sat(d.labels.len())).unwrap_or(0))
}

/// Label `j` of diagnostic `i`: its text is returned, its span is written to `out`.
#[no_mangle]
pub unsafe extern "C" fn capy_diag_label(p: *const CapyParse, i: u32, j: u32, out: *mut CapySpan) -> CapyStr {
    let (s, sp) = with(p, (CapyStr::NONE, CapySpan::default()), |p| {
        match p.result().diagnostics.get(i as usize).and_then(|d| d.labels.get(j as usize)) {
            Some(l) => (CapyStr::of(&l.text), CapySpan::of(l.span)),
            None => (CapyStr::NONE, CapySpan::default()),
        }
    });
    put(out, sp);
    s
}

#[no_mangle]
pub extern "C" fn capy_diag_context_count(p: *const CapyParse, i: u32) -> u32 {
    with(p, 0, |p| p.result().diagnostics.get(i as usize).map(|d| sat(d.context.len())).unwrap_or(0))
}

/// Context frame `j`: `*shape`, `*arg_index`, `*arg_name`. Returns `CAPY_OK` or `CAPY_E_ARG`.
#[no_mangle]
pub unsafe extern "C" fn capy_diag_context(
    p: *const CapyParse,
    i: u32,
    j: u32,
    shape: *mut CapyStr,
    arg_index: *mut u32,
    arg_name: *mut CapyStr,
) -> i32 {
    let r = with(p, None, |p| {
        p.result()
            .diagnostics
            .get(i as usize)
            .and_then(|d| d.context.get(j as usize))
            .map(|f| (CapyStr::of(&f.shape), sat(f.arg_index), CapyStr::of(&f.arg_name)))
    });
    match r {
        Some((s, ix, nm)) => {
            put(shape, s);
            put(arg_index, ix);
            put(arg_name, nm);
            CAPY_OK
        }
        None => {
            put(shape, CapyStr::NONE);
            put(arg_index, 0);
            put(arg_name, CapyStr::NONE);
            CAPY_E_ARG
        }
    }
}

// ------------------------------------------------------------------------------------------ error nodes

#[no_mangle]
pub unsafe extern "C" fn capy_err_span(p: *const CapyParse, e: CapyId, out: *mut CapySpan) -> i32 {
    let (st, sp) = with(p, (CAPY_E_NULL, CapySpan::default()), |p| match p.err(e) {
        Some(n) => (CAPY_OK, CapySpan::of(n.span)),
        None => (CAPY_E_KIND, CapySpan::default()),
    });
    put(out, sp);
    st
}

/// Index into the diagnostics list; `u32::MAX` for an invalid id.
#[no_mangle]
pub extern "C" fn capy_err_diag_index(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, u32::MAX, |p| p.err(e).map(|n| sat(n.diagnostic_index)).unwrap_or(u32::MAX))
}

#[no_mangle]
pub extern "C" fn capy_err_token_count(p: *const CapyParse, e: CapyId) -> u32 {
    with(p, 0, |p| p.err(e).map(|n| sat(n.tokens.len())).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_err_token_text(p: *const CapyParse, e: CapyId, i: u32) -> CapyStr {
    with(p, CapyStr::NONE, |p| {
        p.err(e).and_then(|n| n.tokens.get(i as usize)).map(|t| CapyStr::of(&t.text)).unwrap_or(CapyStr::NONE)
    })
}

#[no_mangle]
pub extern "C" fn capy_err_token_line(p: *const CapyParse, e: CapyId, i: u32) -> u32 {
    with(p, 0, |p| p.err(e).and_then(|n| n.tokens.get(i as usize)).map(|t| sat(t.line)).unwrap_or(0))
}

#[no_mangle]
pub extern "C" fn capy_err_token_col(p: *const CapyParse, e: CapyId, i: u32) -> u32 {
    with(p, 0, |p| p.err(e).and_then(|n| n.tokens.get(i as usize)).map(|t| sat(t.col)).unwrap_or(0))
}

// Kept so the `CAPY_E_KIND` import is used by the id-kind checks above in one place.
#[allow(dead_code)]
const _: i32 = CAPY_E_KIND;
