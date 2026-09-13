//! Differential-testing aid: mirrors `cmd/innerdump` (Go) exactly.

use capy_core::domain::ast::{Expr, InnerBlock, InnerStmt, Path, PathStep};
use capy_core::gofmt;
use capy_core::orchestrator::features::{inner_parser, make_lexer};

fn expr(x: &Expr) -> String {
    match x {
        Expr::Number(n) => {
            if n.is_int {
                format!("int({})", n.i)
            } else {
                format!("flt({})", gofmt::format_float_g(n.f))
            }
        }
        Expr::Str(v) => format!("str({})", gofmt::quote(v)),
        Expr::Bool(b) => format!("bool({})", b),
        Expr::Null => "null".to_string(),
        Expr::Var(s) => format!("var({})", steps(s)),
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
                .map(|(i, k)| format!("{}={}", gofmt::quote(k), expr(&o.vals[i])))
                .collect();
            format!("obj{{{}}}", parts.join(","))
        }
    }
}

fn steps(ss: &[PathStep]) -> String {
    let parts: Vec<String> = ss
        .iter()
        .map(|s| {
            if s.is_index {
                format!("[{}]", s.index.as_ref().map(|e| expr(e)).unwrap_or_default())
            } else {
                format!(".{}", s.field)
            }
        })
        .collect();
    parts.join("")
}

fn path(p: &Path) -> String {
    format!("{}{}", p.root, steps(&p.steps))
}

fn block(b: &InnerBlock, ind: &str) -> String {
    b.stmts.iter().map(|s| stmt(s, ind)).collect()
}

fn stmt(s: &InnerStmt, ind: &str) -> String {
    match s {
        InnerStmt::Set { target, value } => {
            format!("{}set {} = {}\n", ind, path(target), expr(value))
        }
        InnerStmt::Append { target, value } => {
            format!("{}append {} = {}\n", ind, path(target), expr(value))
        }
        InnerStmt::Prepend { target, value } => {
            format!("{}prepend {} = {}\n", ind, path(target), expr(value))
        }
        InnerStmt::Merge { target, value } => {
            format!("{}merge {} = {}\n", ind, path(target), expr(value))
        }
        InnerStmt::Delete { target } => format!("{}delete {}\n", ind, path(target)),
        InnerStmt::Write(v) => format!("{}write {}\n", ind, expr(v)),
        InnerStmt::Call(c) => format!("{}call {}\n", ind, expr(&Expr::Call(c.clone()))),
        InnerStmt::If { cond, body, else_ } => {
            let inner = format!("{}  ", ind);
            let mut out = format!("{}if {}\n{}", ind, expr(cond), block(body, &inner));
            if let Some(e) = else_ {
                out.push_str(&format!("{}else\n{}", ind, block(e, &inner)));
            }
            out + &format!("{}endif\n", ind)
        }
        InnerStmt::Loop(l) => {
            let inner = format!("{}  ", ind);
            format!(
                "{}loop key={} var={} in {}\n{}{}endloop\n",
                ind,
                l.key_var,
                l.var,
                expr(&l.iter),
                block(&l.body, &inner),
                ind
            )
        }
    }
}

fn main() {
    let mut out = String::new();
    for p in std::env::args().skip(1) {
        match std::fs::read_to_string(&p) {
            Err(_) => {
                out.push_str(&format!("FILE\t{}\nREADERR\n", p));
                continue;
            }
            Ok(src) => {
                out.push_str(&format!("FILE\t{}\n", p));
                match make_lexer::tokenize(&src) {
                    Err(e) => out.push_str(&format!("LEXERR\t{}\n", e)),
                    Ok(toks) => match inner_parser::parse_inner(toks) {
                        Err(e) => out.push_str(&format!("PARSEERR\t{}\n", e)),
                        Ok(blk) => out.push_str(&block(&blk, "")),
                    },
                }
            }
        }
    }
    print!("{}", out);
}
