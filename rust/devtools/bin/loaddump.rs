//! Differential-testing aid: mirrors `cmd/loaddump` (Go) exactly.

use capy_core::domain::ast::{Expr, InnerBlock, InnerStmt, Path, PathStep};
use capy_core::domain::val::Val;
use capy_core::gofmt::{format_float_g, quote as q};
use capy_core::orchestrator::features::{make_lexer, make_library_loader};

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
                .map(|(i, k)| format!("{}={}", q(k), expr(&o.vals[i])))
                .collect();
            format!("obj{{{}}}", parts.join(","))
        }
    }
}

fn steps(ss: &[PathStep]) -> String {
    ss.iter()
        .map(|s| {
            if s.is_index {
                format!("[{}]", s.index.as_ref().map(|e| expr(e)).unwrap_or_default())
            } else {
                format!(".{}", s.field)
            }
        })
        .collect::<Vec<_>>()
        .join("")
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

fn ctx_val(v: &Val) -> String {
    match v {
        Val::Null => "nil".to_string(),
        Val::Str(s) => format!("s:{}", q(s)),
        Val::Int(i) => format!("i:{}", i),
        Val::Float(f) => format!("f:{}", format_float_g(*f)),
        Val::Bool(b) => format!("b:{}", b),
        Val::List(x) => {
            let parts: Vec<String> = x.iter().map(ctx_val).collect();
            format!("as:[{}]", parts.join(","))
        }
        Val::StrList(x) => {
            let parts: Vec<String> = x.iter().map(|s| format!("s:{}", q(s))).collect();
            format!("as:[{}]", parts.join(","))
        }
        Val::Obj(m) => {
            let parts: Vec<String> =
                m.iter().map(|(k, val)| format!("{}:{}", q(k), ctx_val(val))).collect();
            format!("m:{{{}}}", parts.join(","))
        }
    }
}

fn gov(v: &[String]) -> String {
    format!("[{}]", v.join(" "))
}

fn main() {
    let mut out = String::new();
    for p in std::env::args().skip(1) {
        out.push_str(&format!("FILE\t{}\n", p));
        let lib = match make_library_loader::load_library(&p, make_lexer::tokenize) {
            Err(e) => {
                out.push_str(&format!("ERR\t{}\n", e));
                if !e.hint.is_empty() {
                    out.push_str(&format!("HINT\t{}\n", e.hint));
                }
                continue;
            }
            Ok(l) => l,
        };
        out.push_str(&format!(
            "ext={} out={} desc={} name={} ver={} defimpl={}\n",
            q(&lib.extension),
            q(&lib.output_file),
            q(&lib.description),
            q(&lib.lib_name),
            q(&lib.lib_version),
            q(&lib.default_impl)
        ));
        out.push_str(&format!(
            "preprocess={} comments={}\n",
            gov(&lib.preprocess),
            gov(&lib.comments)
        ));
        if let Some(ft) = &lib.file_template_ast {
            out.push_str(&format!("FILETEMPLATE\n{}", block(ft, "  ")));
        }
        for (k, v) in &lib.files_ast {
            out.push_str(&format!("FILEAST\t{}\n{}", q(k), block(v, "  ")));
        }
        for (k, v) in &lib.context {
            out.push_str(&format!("ctx\t{}\t{}\n", q(k), ctx_val(v)));
        }
        for t in lib.types.values() {
            out.push_str(&format!(
                "type\t{}\tbase={}\tpat={}\topts={}\tgo={}\tgc={}\n",
                q(&t.name),
                q(&t.base),
                q(&t.pattern),
                gov(&t.options),
                q(&t.group_open),
                q(&t.group_close)
            ));
        }
        for f in lib.functions.values() {
            out.push_str(&format!(
                "fn\t{}\tprio={}\tdesc={}\n",
                q(&f.name),
                f.priority,
                q(&f.description)
            ));
            for a in &f.args {
                out.push_str(&format!(
                    "  arg\tkind={}\tval={}\tname={}\ttype={}\topt={}\tdef={}\trep={}\tsep={}\tjoin={}\n",
                    q(&a.kind), q(&a.value), q(&a.name), q(&a.type_), a.optional,
                    q(&a.default), q(&a.repeat), q(&a.sep), q(&a.join)
                ));
            }
            for e in &f.elements {
                out.push_str(&format!(
                    "  el\tcap={}\tlit={}\tname={}\ttype={}\topt={}\tdef={}\trep={}\tsep={}\tjoin={}\tisfn={}\n",
                    e.is_capture, q(&e.literal), q(&e.name), q(&e.cap_type), e.optional,
                    q(&e.default), q(&e.repeat), q(&e.sep), q(&e.join), e.is_func
                ));
            }
            if let Some(b) = &f.block {
                out.push_str(&format!(
                    "  block\tcloser={}\topen={}\tclose={}\tded={}\tverb={}\tsec={}\n",
                    q(&b.closer),
                    q(&b.open),
                    q(&b.close),
                    b.is_dedent,
                    b.is_verbatim,
                    gov(&b.sections)
                ));
                for sg in &b.close_seq {
                    out.push_str(&format!("    seg\ttoks={}\tref={}\n", gov(&sg.tokens), q(&sg.ref_)));
                }
            }
            if let Some(la) = &f.lookahead {
                out.push_str(&format!(
                    "  lookahead\treq={}\tforbid={}\n",
                    la.require_indent, la.forbid_indent
                ));
            }
            if let Some(t) = &f.template_ast {
                out.push_str(&format!("  TEMPLATE\n{}", block(t, "    ")));
            }
            if let Some(r) = &f.run_ast {
                out.push_str(&format!("  RUN\n{}", block(r, "    ")));
            }
        }
        for c in lib.commands.values() {
            out.push_str(&format!(
                "cmd\t{}\tdesc={}\n{}",
                q(&c.name),
                q(&c.description),
                block(&c.body, "  ")
            ));
        }
    }
    print!("{}", out);
}
