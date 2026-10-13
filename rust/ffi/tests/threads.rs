//! T-04 / R2 — one compiled library, several threads, each parsing: results are identical and nothing races.

mod support;

use capy_core::capy::Library;
use capy_core::domain::ast_json;
use capy_core::gojson;
use capy_ffi::*;
use support::*;

#[test]
fn one_library_many_threads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/operator-precedence");
    let libp = root.join("lib.capy");
    let lp = libp.to_str().unwrap();
    let src = std::fs::read_to_string(root.join("script.capy")).unwrap();
    let want = ast_json::to_json_with(&Library::from_file(lp).unwrap().parse(&src), true);

    let mut lib = std::ptr::null_mut();
    assert_eq!(unsafe { capy_library_from_file(lp.as_ptr(), lp.len(), &mut lib, std::ptr::null_mut()) }, CAPY_OK);
    let addr = lib as usize; // the library handle is read-only after creation and may be shared

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let (src, want) = (src.clone(), want.clone());
            std::thread::spawn(move || {
                let lib = addr as *const CapyLibrary;
                for _ in 0..500 {
                    let mut st = -1;
                    let p = Parse(unsafe { capy_parse(lib, src.as_ptr(), src.len(), &mut st) });
                    assert_eq!(st, CAPY_OK);
                    assert_eq!(gojson::marshal(&document(p.0, true)), want);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker thread");
    }
    unsafe { capy_library_free(lib) };
}
