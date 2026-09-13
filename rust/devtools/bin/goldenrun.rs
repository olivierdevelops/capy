//! Runs the repo's golden samples through the Rust engine and compares against
//! the checked-in `.expected.txt` / `.expected-error.txt` files — the same
//! pairing rule as `cmd/capy/golden_test.go`.

use capy_core::orchestrator::run;

fn normalize(s: &str) -> String {
    // Mirrors golden_test.go's normalize: strip a UTF-8 BOM and CRLF.
    s.trim_start_matches('\u{feff}').replace("\r\n", "\n")
}

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| "samples".to_string());
    let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(&root)
        .expect("read samples dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let (mut pass, mut fail, mut skip) = (0usize, 0usize, 0usize);
    let mut failures: Vec<String> = Vec::new();

    for dir in dirs {
        let lib = dir.join("lib.capy");
        if !lib.exists() {
            continue;
        }
        let mut scripts: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "capy").unwrap_or(false))
            .filter(|p| p.file_name().map(|x| x != "lib.capy").unwrap_or(false))
            .collect();
        scripts.sort();

        for script in scripts {
            let base = script.file_stem().unwrap().to_string_lossy().into_owned();
            let name = format!("{}/{}", dir.file_name().unwrap().to_string_lossy(), base);
            let ok_path = dir.join(format!("{}.expected.txt", base));
            let err_path = dir.join(format!("{}.expected-error.txt", base));

            let result = run::run(&lib.to_string_lossy(), &script.to_string_lossy());

            if err_path.exists() {
                let want = std::fs::read_to_string(&err_path).unwrap().trim().to_string();
                match result {
                    Ok(out) => {
                        fail += 1;
                        failures.push(format!(
                            "{}: expected error, got success ({} bytes)",
                            name,
                            out.len()
                        ));
                    }
                    Err(e) => {
                        let got = e.to_string().trim().to_string();
                        if got == want {
                            pass += 1;
                        } else {
                            fail += 1;
                            failures.push(format!(
                                "{}: error mismatch\n      want: {:?}\n      got:  {:?}",
                                name, want, got
                            ));
                        }
                    }
                }
            } else if ok_path.exists() {
                let want = std::fs::read_to_string(&ok_path).unwrap();
                match result {
                    Err(e) => {
                        fail += 1;
                        failures.push(format!("{}: expected success, got error: {}", name, e));
                    }
                    Ok(out) => {
                        if normalize(&out) == normalize(&want) {
                            pass += 1;
                        } else {
                            fail += 1;
                            let w = normalize(&want);
                            let g = normalize(&out);
                            let wl: Vec<&str> = w.split('\n').collect();
                            let gl: Vec<&str> = g.split('\n').collect();
                            let mut first = String::from("(identical lines?)");
                            for i in 0..wl.len().max(gl.len()) {
                                let a = wl.get(i).copied().unwrap_or("<missing>");
                                let b = gl.get(i).copied().unwrap_or("<missing>");
                                if a != b {
                                    first = format!(
                                        "line {}\n      want: {:?}\n      got:  {:?}",
                                        i + 1,
                                        a,
                                        b
                                    );
                                    break;
                                }
                            }
                            failures.push(format!("{}: output mismatch at {}", name, first));
                        }
                    }
                }
            } else {
                skip += 1;
            }
        }
    }

    println!("PASS {}  FAIL {}  SKIP {}", pass, fail, skip);
    if !failures.is_empty() {
        println!("\n--- failures ---");
        for f in failures.iter().take(25) {
            println!("  {}", f);
        }
        if failures.len() > 25 {
            println!("  … and {} more", failures.len() - 25);
        }
        std::process::exit(1);
    }
}
