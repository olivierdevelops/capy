//! Differential-testing aid: mirrors `cmd/parsedump` (Go) exactly.

use capy_core::domain::ast::{Block, Expr, FuncCall};
use capy_core::gofmt::{format_float_g, quote as q};
use capy_core::orchestrator::features::{make_lexer, make_library_loader, make_parser};

fn expr(x: &Expr) -> String {
    match x {
        Expr::Number(n) => {
            if n.is_int {
                format!("int({})", n.i)
            } else {
                format!("flt({})", format_float_g(n.f))
            }
        }
        Expr::Str(v) => format!("str({})", q(v)),
        Expr::Bool(b) => format!("bool({})", b),
        Expr::Null => "null".to_string(),
        Expr::Var(steps) => {
            let parts: Vec<String> = steps
                .iter()
                .map(|s| {
                    if s.is_index {
                        format!("[{}]", s.index.as_ref().map(|e| expr(e)).unwrap_or_default())
                    } else {
                        format!(".{}", s.field)
                    }
                })
                .collect();
            format!("var({})", parts.join(""))
        }
        Expr::Call(c) => {
            let args: Vec<String> = c.args.iter().map(expr).collect();
            format!("call({};{})", c.name.join("."), args.join(","))
        }
        Expr::Compare(c) => format!("cmp({};{};{})", c.op, expr(&c.left), expr(&c.right)),
        Expr::Not(x) => format!("not({})", expr(x)),
        Expr::List(items) => {
            let v: Vec<String> = items.iter().map(expr).collect();
            format!("list[{}]", v.join(","))
        }
        Expr::Obj(o) => {
            let parts: Vec<String> = o
                .keys
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{}={}", q(k), expr(&o.vals[i])))
                .collect();
            format!("obj{{{}}}", parts.join(","))
        }
    }
}

fn dump_block(out: &mut String, b: &Block, ind: &str) {
    if b.is_verbatim {
        out.push_str(&format!("{}VERBATIM {}\n", ind, q(&b.verbatim_text)));
        return;
    }
    for c in &b.stmts {
        dump_call(out, c, ind);
    }
}

fn dump_call(out: &mut String, c: &FuncCall, ind: &str) {
    out.push_str(&format!("{}call {} @{}:{}\n", ind, c.func, c.line, c.col));
    for (k, cv) in &c.captures {
        let e = match (&cv.is_expr, &cv.expr) {
            (true, Some(x)) => expr(x),
            _ => "-".to_string(),
        };
        out.push_str(&format!(
            "{}  cap {} text={} isexpr={} expr={} nsub={}\n",
            ind,
            q(k),
            q(&cv.text),
            cv.is_expr,
            e,
            cv.sub.len()
        ));
        let deeper = format!("{}    ", ind);
        for s in &cv.sub {
            dump_call(out, s, &deeper);
        }
    }
    if let Some(body) = &c.body {
        out.push_str(&format!("{}  BODY\n", ind));
        dump_block(out, body, &format!("{}    ", ind));
    }
    for (s, blk) in &c.sections {
        out.push_str(&format!("{}  SECTION {}\n", ind, q(s)));
        dump_block(out, blk, &format!("{}    ", ind));
    }
    if let Some(closer) = &c.closer {
        out.push_str(&format!("{}  CLOSER\n", ind));
        dump_call(out, closer, &format!("{}    ", ind));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i + 1 < args.len() {
        let (lib_path, script_path) = (&args[i], &args[i + 1]);
        i += 2;
        out.push_str(&format!("PAIR\t{}\t{}\n", lib_path, script_path));
        let lib = match make_library_loader::load_library(lib_path, make_lexer::tokenize) {
            Err(e) => {
                out.push_str(&format!("LOADERR\t{}\n", e));
                continue;
            }
            Ok(l) => l,
        };
        let src = match std::fs::read_to_string(script_path) {
            Err(_) => {
                out.push_str("READERR\n");
                continue;
            }
            Ok(s) => s,
        };
        let toks = match make_lexer::tokenize_with(&src, &lib.comments) {
            Err(e) => {
                out.push_str(&format!("LEXERR\t{}\n", e));
                continue;
            }
            Ok(t) => t,
        };
        match make_parser::parse(toks, &src, &lib) {
            Err(e) => {
                out.push_str(&format!("PARSEERR\t{}\n", e));
                if !e.hint.is_empty() {
                    out.push_str(&format!("HINT\t{}\n", e.hint));
                }
            }
            Ok(blk) => dump_block(&mut out, &blk, ""),
        }
    }
    print!("{}", out);
}
