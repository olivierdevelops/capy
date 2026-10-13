//! M-02 / M-03 (PLAN-2026-0005) — what the id table costs, and what one accessor call costs.
//!
//!   cargo run --profile ffi -p capy-ffi --example bench
//!
//! M-02: over every shipped sample script, time `Library::parse` alone against `capy_parse` (which is
//! `Library::parse` plus the flatten) plus its free, 15 interleaved samples, each sample being 20 passes
//! over the corpus. Reports min / median / max and the median overhead.
//! M-03: 10^7 calls of `capy_node_func` on one parse.

use capy_core::capy::Library;
use capy_ffi::*;
use std::path::PathBuf;
use std::time::Instant;

fn corpus() -> Vec<(Library, *mut CapyLibrary, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let mut out = Vec::new();
    for d in dirs {
        let lib = d.join("lib.capy");
        let Ok(rl) = Library::from_file(lib.to_str().unwrap()) else { continue };
        let lp = lib.to_str().unwrap();
        let mut c = std::ptr::null_mut();
        assert_eq!(unsafe { capy_library_from_file(lp.as_ptr(), lp.len(), &mut c, std::ptr::null_mut()) }, CAPY_OK);
        let mut scripts: Vec<PathBuf> = std::fs::read_dir(&d).unwrap().filter_map(|e| e.ok()).map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "capy").unwrap_or(false) && p.file_name().unwrap() != "lib.capy").collect();
        scripts.sort();
        for s in scripts {
            let src = std::fs::read_to_string(&s).unwrap();
            // each script keeps its own handle pair so library compilation is never timed
            let mut c2 = std::ptr::null_mut();
            assert_eq!(unsafe { capy_library_from_file(lp.as_ptr(), lp.len(), &mut c2, std::ptr::null_mut()) }, CAPY_OK);
            out.push((Library::from_file(lp).unwrap(), c2, src));
        }
        unsafe { capy_library_free(c) };
        drop(rl);
    }
    out
}

fn stats(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[0], v[v.len() / 2], v[v.len() - 1])
}

fn main() {
    let items = corpus();
    println!("corpus: {} scripts", items.len());
    let passes = 20;
    let (mut rust, mut ffi) = (Vec::new(), Vec::new());
    for _ in 0..15 {
        let t = Instant::now();
        for _ in 0..passes {
            for (lib, _, src) in &items {
                std::hint::black_box(lib.parse(src));
            }
        }
        rust.push(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        for _ in 0..passes {
            for (_, c, src) in &items {
                let mut st = 0;
                let p = unsafe { capy_parse(*c, src.as_ptr(), src.len(), &mut st) };
                std::hint::black_box(p);
                unsafe { capy_parse_free(p) };
            }
        }
        ffi.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let (r, f) = (stats(&mut rust.clone()), stats(&mut ffi.clone()));
    println!("M-02 Library::parse        min {:.2} ms  median {:.2} ms  max {:.2} ms  (spread {:.1} %)", r.0, r.1, r.2, (r.2 - r.0) / r.1 * 100.0);
    println!("M-02 capy_parse + free     min {:.2} ms  median {:.2} ms  max {:.2} ms  (spread {:.1} %)", f.0, f.1, f.2, (f.2 - f.0) / f.1 * 100.0);
    println!("M-02 median overhead       {:+.1} %", (f.1 / r.1 - 1.0) * 100.0);

    // M-03
    let (_, c, _) = &items[0];
    let src = "x\n";
    let _ = src;
    let (_, c2, s2) = items.iter().find(|(_, _, s)| !s.trim().is_empty()).unwrap();
    let _ = c;
    let mut st = 0;
    let p = unsafe { capy_parse(*c2, s2.as_ptr(), s2.len(), &mut st) };
    let root = capy_root(p);
    let n = capy_block_stmt(p, root, 0);
    let calls = 10_000_000u64;
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = Instant::now();
        let mut acc = 0usize;
        for _ in 0..calls {
            acc += std::hint::black_box(capy_node_func(std::hint::black_box(p), std::hint::black_box(n))).len;
        }
        std::hint::black_box(acc);
        best = best.min(t.elapsed().as_secs_f64() * 1e9 / calls as f64);
    }
    println!("M-03 capy_node_func        best of 5: {:.2} ns/call", best);
    unsafe { capy_parse_free(p) };
}
