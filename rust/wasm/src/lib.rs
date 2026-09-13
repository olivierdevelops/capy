//! The wasm boundary: a minimal linear-memory ABI for hosts (wazero, browsers)
//! to drive the engine without cgo.
//!
//! Deliberately NOT Emscripten/embind. `wasm32-unknown-unknown` gives raw wasm
//! with no libc, so the contract is three exports and a length-prefixed buffer:
//!
//! ```text
//! capy_alloc(len)            -> ptr      reserve `len` bytes in wasm memory
//! capy_free(ptr, len)                    release a previous allocation
//! capy_run(lp, ll, sp, sl)   -> ptr      transpile; returns a result buffer
//! capy_introspect(lp, ll)    -> ptr      library metadata as JSON
//! capy_docs(lp, ll)          -> ptr      Markdown docs
//! capy_version()             -> ptr      engine version string
//! ```
//!
//! Every returned `ptr` addresses a buffer laid out as:
//!
//! ```text
//! [0..4)   u32 little-endian payload length
//! [4..4+n) payload bytes (UTF-8)
//! ```
//!
//! The host reads the length, copies the payload out, then calls
//! `capy_free(ptr, 4 + n)`. The payload is JSON for every entry point so one
//! decoder covers success and failure:
//!
//! ```json
//! {"ok":true,"output":"…","files":{"path":"…"}}
//! {"ok":false,"error":"…","hint":"…","line":3,"col":5}
//! ```
//!
//! This boundary is STATELESS — no handles, no cross-call lifetime. That is what
//! keeps a wasm host simple compared to a stateful C++ library, where the host
//! must pool instances and marshal object handles.

use capy_core::capy::Library;
use capy_core::domain::errors::{format_with_source, CapyError};
use capy_core::gojson::marshal_string as jstr;

/// Reserve `len` bytes of wasm linear memory and hand the host the pointer.
///
/// # Safety
/// The host must eventually pass the same `(ptr, len)` back to [`capy_free`].
#[no_mangle]
pub extern "C" fn capy_alloc(len: u32) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Release memory previously handed out by [`capy_alloc`] or a result buffer.
///
/// # Safety
/// `ptr`/`len` must exactly match a prior allocation.
#[no_mangle]
pub unsafe extern "C" fn capy_free(ptr: *mut u8, len: u32) {
    if ptr.is_null() {
        return;
    }
    drop(Vec::from_raw_parts(ptr, 0, len as usize));
}

/// Packs a payload into a freshly allocated `[u32 len][bytes]` buffer and leaks
/// it for the host to read and then free.
fn into_result_buffer(payload: String) -> *mut u8 {
    let bytes = payload.into_bytes();
    let n = bytes.len() as u32;
    let mut buf: Vec<u8> = Vec::with_capacity(4 + bytes.len());
    buf.extend_from_slice(&n.to_le_bytes());
    buf.extend_from_slice(&bytes);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Reads a host-supplied UTF-8 string out of linear memory without taking
/// ownership — the host still owns and frees the input buffers.
///
/// # Safety
/// `ptr`/`len` must describe an initialised UTF-8 range.
unsafe fn read_str(ptr: *const u8, len: u32) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let slice = std::slice::from_raw_parts(ptr, len as usize);
    String::from_utf8_lossy(slice).into_owned()
}

/// Port of `errResult` in `cmd/capy-wasm/main.go`: `{ok, error, hint}`.
fn err_result(msg: &str, hint: &str) -> String {
    format!(
        "{{\"ok\":false,\"error\":{},\"hint\":{}}}",
        jstr(msg),
        jstr(hint)
    )
}

/// Port of `errResultFromErr`.
///
/// A structured engine error carries its bare message plus position and the
/// caret-rendered `pretty` view the playground displays. A plain error (Go's
/// non-`*CapyError` branch) carries only `error`.
fn err_result_from_err(e: &CapyError, source: &str) -> String {
    if e.plain {
        return format!("{{\"ok\":false,\"error\":{}}}", jstr(&e.to_string()));
    }
    format!(
        "{{\"ok\":false,\"error\":{},\"hint\":{},\"line\":{},\"col\":{},\"pretty\":{}}}",
        // Go sends `ce.Msg` — the bare message, WITHOUT the `line:col:` prefix
        // that `Error()` adds.
        jstr(&e.msg),
        jstr(&e.hint),
        e.line,
        e.col,
        jstr(&format_with_source(e, source))
    )
}

/// `capy_run(libPtr, libLen, scriptPtr, scriptLen) -> resultPtr`
///
/// Transpiles `script` through `lib`. Runs with the sandboxed host, so `env` /
/// `read_file` / `exec` are unavailable — exactly the posture the playground and
/// default embedded callers want.
///
/// # Safety
/// Both pointer/length pairs must describe initialised UTF-8 ranges.
#[no_mangle]
pub unsafe extern "C" fn capy_run(
    lib_ptr: *const u8,
    lib_len: u32,
    script_ptr: *const u8,
    script_len: u32,
) -> *mut u8 {
    let lib_src = read_str(lib_ptr, lib_len);
    let script_src = read_str(script_ptr, script_len);
    let payload = match Library::new(&lib_src) {
        Err(e) => err_result_from_err(&e, &script_src),
        Ok(lib) => match lib.run_multi(&script_src) {
            Err(e) => err_result_from_err(&e, &script_src),
            Ok((output, files)) => {
                let mut s = String::from("{\"ok\":true,\"output\":");
                s.push_str(&jstr(&output));
                s.push_str(",\"files\":{");
                for (i, (k, v)) in files.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    s.push_str(&jstr(k));
                    s.push(':');
                    s.push_str(&jstr(v));
                }
                s.push_str("},\"extension\":");
                s.push_str(&jstr(lib.extension()));
                s.push('}');
                s
            }
        },
    };
    into_result_buffer(payload)
}

/// `capy_introspect(libPtr, libLen) -> resultPtr`
///
/// Returns the library's declared functions so an editor can derive
/// autocomplete / hover-docs instead of hand-maintaining a catalogue.
///
/// # Safety
/// The pointer/length pair must describe an initialised UTF-8 range.
#[no_mangle]
pub unsafe extern "C" fn capy_introspect(lib_ptr: *const u8, lib_len: u32) -> *mut u8 {
    if lib_ptr.is_null() && lib_len == 0 {
        return into_result_buffer(err_result("capyIntrospect expects (libSrc)", ""));
    }
    let lib_src = read_str(lib_ptr, lib_len);
    let payload = match Library::new(&lib_src) {
        Err(e) => err_result_from_err(&e, &lib_src),
        Ok(lib) => {
            let mut s = String::from("{\"ok\":true,\"functions\":[");
            for (i, fi) in lib.introspect().iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str("{\"name\":");
                s.push_str(&jstr(&fi.name));
                s.push_str(",\"description\":");
                s.push_str(&jstr(&fi.description));
                s.push_str(",\"block\":");
                s.push_str(&jstr(&fi.block));
                s.push_str(&format!(",\"priority\":{}", fi.priority));
                s.push_str(",\"args\":[");
                for (j, a) in fi.args.iter().enumerate() {
                    if j > 0 {
                        s.push(',');
                    }
                    s.push_str("{\"kind\":");
                    s.push_str(&jstr(&a.kind));
                    s.push_str(",\"value\":");
                    s.push_str(&jstr(&a.value));
                    s.push_str(",\"name\":");
                    s.push_str(&jstr(&a.name));
                    s.push_str(",\"type\":");
                    s.push_str(&jstr(&a.type_));
                    s.push_str(",\"description\":");
                    s.push_str(&jstr(&a.description));
                    s.push('}');
                }
                s.push_str("]}");
            }
            s.push_str("],\"comments\":[");
            for (i, c) in lib.comment_markers().iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&jstr(c));
            }
            s.push_str("]}");
            s
        }
    };
    into_result_buffer(payload)
}

/// `capy_docs(libPtr, libLen) -> resultPtr` — Markdown reference docs.
///
/// # Safety
/// The pointer/length pair must describe an initialised UTF-8 range.
#[no_mangle]
pub unsafe extern "C" fn capy_docs(lib_ptr: *const u8, lib_len: u32) -> *mut u8 {
    let lib_src = read_str(lib_ptr, lib_len);
    let payload = match Library::new(&lib_src) {
        Err(e) => err_result_from_err(&e, &lib_src),
        Ok(lib) => format!(
            "{{\"ok\":true,\"docs\":{}}}",
            jstr(&capy_core::capy::render_library_docs(&lib))
        ),
    };
    into_result_buffer(payload)
}

/// `capy_version() -> resultPtr`
///
/// The JS shim unwraps `output` from this envelope, so the browser-facing
/// signature is a bare version string.
///
/// `CAPY_VERSION` is read at compile time so CI can stamp in `git describe`
/// (the playground toolbar shows the release, not the crate version). This is
/// the replacement for the Go build's `-ldflags "-X main.version=…"`. Falls back
/// to the crate version for an ordinary local build.
#[no_mangle]
pub extern "C" fn capy_version() -> *mut u8 {
    let version = match option_env!("CAPY_VERSION") {
        Some(v) if !v.is_empty() => v,
        _ => env!("CARGO_PKG_VERSION"),
    };
    into_result_buffer(format!("{{\"ok\":true,\"output\":{}}}", jstr(version)))
}
