//! Measures the Rust engine's per-run cost natively (no wasm), for comparison.
use capy_core::capy::Library;
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let lib_src = std::fs::read_to_string(&a[0]).unwrap();
    let script = std::fs::read_to_string(&a[1]).unwrap();
    let lib = Library::new(&lib_src).unwrap();
    for _ in 0..5 {
        lib.run(&script).unwrap();
    }
    let n = 200;
    let t = Instant::now();
    for _ in 0..n {
        lib.run(&script).unwrap();
    }
    println!("native Rust per Run: {:?}", t.elapsed() / n);
}
