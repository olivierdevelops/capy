//! Differential-testing aid: mirrors `cmd/cmddump` (Go) exactly.

use capy_core::domain::val::Val;
use capy_core::gofmt::quote as q;
use capy_core::orchestrator::command_args::{command_help, parse_command_args};
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

fn repr_any(v: &Val) -> String {
    match v {
        Val::Null => "nil".to_string(),
        Val::Str(s) => format!("s:{}", q(s)),
        Val::Bool(b) => format!("b:{}", b),
        other => format!("?:{}", other.format_v()),
    }
}

/// Go prints a `[]string` with `%v` as `[a b c]`.
fn gov(v: &[String]) -> String {
    format!("[{}]", v.join(" "))
}

fn argv_grid() -> Vec<Vec<String>> {
    let raw: Vec<Vec<&str>> = vec![
        vec![],
        vec!["one"],
        vec!["one", "two"],
        vec!["one", "two", "three"],
        vec!["--verbose"],
        vec!["--out", "x.txt"],
        vec!["--out=x.txt"],
        vec!["--out"],
        vec!["--nope"],
        vec!["--verbose", "one"],
        vec!["one", "--verbose", "two"],
        vec!["-h"],
    ];
    raw.into_iter()
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .collect()
}

fn main() {
    let mut out = String::new();
    for lib_path in std::env::args().skip(1) {
        out.push_str(&format!("LIB\t{}\n", lib_path));
        let lib = match make_library_loader::load_library(&lib_path, make_lexer::tokenize) {
            Err(e) => {
                out.push_str(&format!("LOADERR\t{}\n", e));
                continue;
            }
            Ok(l) => l,
        };
        for cmd in lib.commands.values() {
            out.push_str(&format!(
                "CMD\t{}\tdesc={}\tnargs={}\tnflags={}\n",
                q(&cmd.name),
                q(&cmd.description),
                cmd.args.len(),
                cmd.flags.len()
            ));
            out.push_str(&format!("HELP\n{}\nENDHELP\n", command_help(&lib, cmd)));
            for argv in argv_grid() {
                match parse_command_args(cmd, &argv) {
                    Err(e) => {
                        out.push_str(&format!("  ARGS {} -> ERR {}\n", gov(&argv), e));
                        continue;
                    }
                    Ok(p) => {
                        out.push_str(&format!("  ARGS {} -> pos{{", gov(&argv)));
                        for (i, (k, v)) in p.pos.iter().enumerate() {
                            if i > 0 {
                                out.push(',');
                            }
                            out.push_str(&format!("{}={}", q(k), repr_any(v)));
                        }
                        out.push_str("} flags{");
                        for (i, (k, v)) in p.flags.iter().enumerate() {
                            if i > 0 {
                                out.push(',');
                            }
                            out.push_str(&format!("{}={}", q(k), repr_any(v)));
                        }
                        out.push_str(&format!("}} extra{}\n", gov(&p.extra)));
                    }
                }
            }
        }
    }
    print!("{}", out);
}
