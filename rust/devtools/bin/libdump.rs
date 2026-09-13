//! Differential-testing aid: mirrors `cmd/libdump` (Go) exactly.

use capy_core::domain::val::Val;
use capy_core::gofmt::quote as q;
use capy_core::infra::capy_lib_parser::CapyLibParser;

fn ctx_val(v: &Val) -> String {
    match v {
        Val::Null => "nil".to_string(),
        Val::Str(s) => format!("s:{}", q(s)),
        Val::Int(i) => format!("i:{}", i),
        Val::Float(f) => format!("f:{}", capy_core::gofmt::format_float_g(*f)),
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

/// Go prints a `[]string` with `%v` as `[a b c]`.
fn gov(v: &[String]) -> String {
    format!("[{}]", v.join(" "))
}

fn main() {
    let p = CapyLibParser;
    let mut out = String::new();
    for path in std::env::args().skip(1) {
        out.push_str(&format!("FILE\t{}\n", path));
        let lib = match p.parse_file(&path) {
            Err(e) => {
                out.push_str(&format!("ERR\t{}\n", e));
                continue;
            }
            Ok(l) => l,
        };
        out.push_str(&format!(
            "ext\t{}\noutfile\t{}\ndesc\t{}\nlibname\t{}\nlibver\t{}\ndefimpl\t{}\n",
            q(&lib.extension),
            q(&lib.output_file),
            q(&lib.description),
            q(&lib.lib_name),
            q(&lib.lib_version),
            q(&lib.default_impl)
        ));
        out.push_str(&format!(
            "imports\t{}\npreprocess\t{}\ncomments\t{}\n",
            gov(&lib.imports),
            gov(&lib.preprocess),
            gov(&lib.comments)
        ));
        out.push_str(&format!("filetemplate\t{}\n", q(&lib.file_template)));
        for (k, v) in &lib.files {
            out.push_str(&format!("file\t{}\t{}\n", q(k), q(v)));
        }
        for (k, v) in &lib.context {
            out.push_str(&format!("ctx\t{}\t{}\n", q(k), ctx_val(v)));
        }
        for (k, t) in &lib.types {
            out.push_str(&format!(
                "type\t{}\tdesc={}\tbase={}\tpat={}\topts={}\tgo={}\tgc={}\n",
                q(k),
                q(&t.description),
                q(&t.base),
                q(&t.pattern),
                gov(&t.options),
                q(&t.group_open),
                q(&t.group_close)
            ));
        }
        for (k, im) in &lib.impls {
            out.push_str(&format!(
                "impl\t{}\tname={}\tfile={}\tdesc={}\tver={}\tdef={}\n",
                q(k),
                q(&im.name),
                q(&im.file),
                q(&im.description),
                q(&im.version),
                im.is_default
            ));
        }
        for (k, c) in &lib.commands {
            out.push_str(&format!(
                "cmd\t{}\tdesc={}\tbody={}\n",
                q(k),
                q(&c.description),
                q(&c.body)
            ));
            for a in &c.args {
                out.push_str(&format!(
                    "  cmdarg\tname={}\treq={}\tdesc={}\n",
                    q(&a.name),
                    a.required,
                    q(&a.description)
                ));
            }
            for fl in &c.flags {
                out.push_str(&format!(
                    "  cmdflag\tname={}\tdesc={}\tdef={}\tbool={}\n",
                    q(&fl.name),
                    q(&fl.description),
                    q(&fl.default),
                    fl.is_bool
                ));
            }
        }
        for (k, f) in &lib.functions {
            out.push_str(&format!(
                "fn\t{}\tdesc={}\tprio={}\tbare={}\tfbi={}\tnfbi={}\n",
                q(k),
                q(&f.description),
                f.priority,
                f.bare,
                f.followed_by_indent,
                f.not_followed_by_indent
            ));
            out.push_str(&format!("  body\t{}\n", q(&f.body)));
            for a in &f.args {
                out.push_str(&format!(
                    "  arg\tkind={}\tval={}\tname={}\ttype={}\tdesc={}\topt={}\tdef={}\trep={}\tsep={}\tjoin={}\n",
                    q(&a.kind), q(&a.value), q(&a.name), q(&a.type_), q(&a.description),
                    a.optional, q(&a.default), q(&a.repeat), q(&a.sep), q(&a.join)
                ));
            }
            if let Some(b) = &f.block {
                out.push_str(&format!(
                    "  block\tcloser={}\topen={}\tclose={}\tded={}\tverb={}\tsections={}\n",
                    q(&b.closer),
                    q(&b.open),
                    q(&b.close),
                    b.is_dedent,
                    b.is_verbatim,
                    gov(&b.sections)
                ));
                for sg in &b.close_seq {
                    out.push_str(&format!("    seg\ttext={}\tref={}\n", q(&sg.text), sg.is_ref));
                }
            }
        }
    }
    print!("{}", out);
}
