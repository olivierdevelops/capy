//! Differential-testing aid: mirrors `cmd/embeddump` (Go) exactly.

use capy_core::capy::{render_library_docs, Library};
use capy_core::gofmt::quote as q;

fn gov(v: &[String]) -> String {
    format!("[{}]", v.join(" "))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i + 1 < args.len() {
        let (lib_path, script_path) = (&args[i], &args[i + 1]);
        i += 2;
        out.push_str(&format!("PAIR\t{}\t{}\n", lib_path, script_path));
        let lib = match Library::from_file(lib_path) {
            Err(e) => {
                out.push_str(&format!("NEWERR\t{}\n", e));
                continue;
            }
            Ok(l) => l,
        };
        out.push_str(&format!(
            "ext={} outfile={}\n",
            q(lib.extension()),
            q(lib.output_file())
        ));
        out.push_str(&format!("names={}\n", gov(&lib.function_names())));
        out.push_str(&format!("comments={}\n", gov(&lib.comment_markers())));
        for fi in lib.introspect() {
            out.push_str(&format!(
                "fn\t{}\tdesc={}\tblock={}\tprio={}\n",
                q(&fi.name),
                q(&fi.description),
                q(&fi.block),
                fi.priority
            ));
            for a in &fi.args {
                out.push_str(&format!(
                    "  arg\tkind={}\tval={}\tname={}\ttype={}\tdesc={}\topt={}\tdef={}\n",
                    q(&a.kind), q(&a.value), q(&a.name), q(&a.type_),
                    q(&a.description), a.optional, q(&a.default)
                ));
            }
        }
        out.push_str(&format!("DOCS\n{}\nENDDOCS\n", render_library_docs(&lib)));
        let src = match std::fs::read_to_string(script_path) {
            Err(_) => {
                out.push_str("READERR\n");
                continue;
            }
            Ok(s) => s,
        };
        match lib.run_multi(&src) {
            Err(e) => out.push_str(&format!("RUNERR\t{}\n", e)),
            Ok((primary, files)) => {
                out.push_str(&format!("OUT\t{}\n{}\nENDOUT\n", primary.len(), primary));
                for (k, v) in &files {
                    out.push_str(&format!("FILE\t{}\t{}\n{}\nENDFILE\n", k, v.len(), v));
                }
            }
        }
    }
    print!("{}", out);
}
