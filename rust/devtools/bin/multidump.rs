//! Differential-testing aid: mirrors `cmd/multidump` (Go) exactly.

use capy_core::orchestrator::run;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i + 1 < args.len() {
        let (lib, script) = (&args[i], &args[i + 1]);
        i += 2;
        out.push_str(&format!("PAIR\t{}\t{}\n", lib, script));
        match run::run_multi(lib, script) {
            Err(e) => out.push_str(&format!("ERR\t{}\n", e)),
            Ok((primary, files)) => {
                out.push_str(&format!(
                    "OUTPUT\t{} bytes\n{}\nENDOUTPUT\n",
                    primary.len(),
                    primary
                ));
                for (k, v) in &files {
                    out.push_str(&format!("FILE\t{}\t{} bytes\n{}\nENDFILE\n", k, v.len(), v));
                }
            }
        }
    }
    print!("{}", out);
}
