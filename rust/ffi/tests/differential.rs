//! T-02 / R13 — for every shipped sample, the tree read through the C ABI equals the Rust writer's
//! document byte for byte, with and without the opt-in `expr` field. Also T-06 (load errors).

mod support;

use capy_core::capy::Library;
use capy_core::domain::ast_json;
use capy_core::gojson;
use capy_ffi::*;
use std::path::{Path, PathBuf};
use support::*;

fn samples() -> Vec<(String, PathBuf, PathBuf)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let mut out = Vec::new();
    for d in dirs {
        let lib = d.join("lib.capy");
        if !lib.exists() {
            continue;
        }
        let mut scripts: Vec<PathBuf> = std::fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "capy").unwrap_or(false) && p.file_name().unwrap() != "lib.capy")
            .collect();
        scripts.sort();
        for sc in scripts {
            out.push((format!("{}/{}", d.file_name().unwrap().to_string_lossy(), sc.file_stem().unwrap().to_string_lossy()), lib.clone(), sc));
        }
    }
    out
}

fn open_lib(path: &Path) -> *mut CapyLibrary {
    let p = path.to_str().unwrap();
    let mut lib = std::ptr::null_mut();
    let st = unsafe { capy_library_from_file(p.as_ptr(), p.len(), &mut lib, std::ptr::null_mut()) };
    assert_eq!(st, CAPY_OK, "{p}");
    lib
}

#[test]
fn c_view_equals_rust_json_for_every_sample() {
    let all = samples();
    assert!(all.len() > 100, "found only {} sample scripts", all.len());
    let (mut checked, mut with_diag, mut with_expr) = (0, 0, 0);
    for (name, libp, scp) in &all {
        let Ok(rlib) = Library::from_file(libp.to_str().unwrap()) else {
            continue; // libraries that are meant to be rejected are covered by `load_errors_match`
        };
        let src = std::fs::read_to_string(scp).unwrap();
        let rr = rlib.parse(&src);
        let lib = open_lib(libp);
        let mut st = -1;
        let p = Parse(unsafe { capy_parse(lib, src.as_ptr(), src.len(), &mut st) });
        assert_eq!(st, CAPY_OK, "{name}");
        assert_eq!(capy_is_clean(p.0) == 1, rr.is_clean(), "{name}: clean flag");
        for ex in [false, true] {
            let want = ast_json::to_json_with(&rr, ex);
            let got = gojson::marshal(&document(p.0, ex));
            assert_eq!(got, want, "{name} (expr={ex}) differs");
            // and the C JSON dump is the same document
            let (mut buf, mut len) = (std::ptr::null_mut(), 0usize);
            assert_eq!(unsafe { capy_parse_to_json(p.0, if ex { CAPY_JSON_EXPR } else { 0 }, &mut buf, &mut len) }, CAPY_OK);
            let dumped = unsafe { std::slice::from_raw_parts(buf, len) }.to_vec();
            unsafe { capy_buf_free(buf, len) };
            assert_eq!(String::from_utf8(dumped).unwrap(), want, "{name}: capy_parse_to_json");
        }
        with_diag += (!rr.diagnostics.is_empty()) as u32;
        with_expr += ast_json::to_json_with(&rr, true).contains("\"expr\":{") as u32;
        checked += 1;
        unsafe { capy_library_free(lib) };
    }
    eprintln!("differential: {checked} scripts ({with_diag} with diagnostics, {with_expr} with expressions)");
    assert!(checked > 100);
    assert!(with_diag > 0, "the corpus must exercise the diagnostics path");
    assert!(with_expr > 0, "the corpus must exercise the expression path");
}

#[test]
fn default_json_has_no_expr_field() {
    // R12: without the flag the document is what it always was.
    let (_, libp, scp) = samples().into_iter().find(|(n, _, _)| n.starts_with("operator-precedence/")).unwrap();
    let rlib = Library::from_file(libp.to_str().unwrap()).unwrap();
    let rr = rlib.parse(&std::fs::read_to_string(scp).unwrap());
    assert!(!ast_json::to_json(&rr).contains("\"expr\""));
    assert!(ast_json::to_json_with(&rr, true).contains("\"expr\""));
}

#[test]
fn load_errors_match() {
    // T-06 / R7: a grammar that fails to compile reports the five fields the Rust error carries.
    let bad = "function broken\n  arg capture x nonsense_type\nend\n";
    let want = Library::new(bad).expect_err("grammar should be rejected");
    let (mut lib, mut err) = (std::ptr::null_mut(), std::ptr::null_mut());
    let st = unsafe { capy_library_new(bad.as_ptr(), bad.len(), &mut lib, &mut err) };
    assert_eq!(st, CAPY_E_LOAD);
    assert!(lib.is_null());
    assert!(!err.is_null());
    assert_eq!(s(capy_error_msg(err)), want.msg);
    assert_eq!(s(capy_error_hint(err)), want.hint);
    assert_eq!(s(capy_error_file(err)), want.file);
    assert_eq!(capy_error_line(err) as usize, want.line);
    assert_eq!(capy_error_col(err) as usize, want.col);
    unsafe { capy_error_free(err) };
}

#[test]
fn rejected_sample_libraries_are_rejected_over_c_too() {
    for (name, libp, _) in samples() {
        if Library::from_file(libp.to_str().unwrap()).is_ok() {
            continue;
        }
        let p = libp.to_str().unwrap();
        let (mut lib, mut err) = (std::ptr::null_mut(), std::ptr::null_mut());
        let st = unsafe { capy_library_from_file(p.as_ptr(), p.len(), &mut lib, &mut err) };
        assert_eq!(st, CAPY_E_LOAD, "{name}");
        assert!(!err.is_null());
        unsafe { capy_error_free(err) };
    }
}
