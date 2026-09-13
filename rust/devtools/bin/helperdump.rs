//! Differential-testing aid: mirrors `cmd/helperdump` (Go) exactly.

use capy_core::domain::val::Val;
use capy_core::gofmt;
use capy_core::infra::helpers;
use std::collections::BTreeMap;

fn corpus() -> Vec<Val> {
    let mut m = BTreeMap::new();
    m.insert("b".to_string(), Val::Int(2));
    m.insert("a".to_string(), Val::str("x"));
    vec![
        Val::Null,
        Val::str(""),
        Val::str("hello world"),
        Val::str("\"quoted\""),
        Val::str("'single'"),
        Val::str("`backtick`"),
        Val::str("Habit Tracker"),
        Val::str("habit-tracker_name.thing"),
        Val::str("camelCaseAlready"),
        Val::str("a<b>&c\"d'e"),
        Val::str("line1\nline2"),
        Val::str("He said \\\"hi\\\""),
        Val::str("tab\there"),
        Val::str("\"esc\\nseq\""),
        Val::str("héllo wörld"),
        Val::str("日本語"),
        Val::str("ß İ ﬁ"),
        Val::str("42"),
        Val::str("-17"),
        Val::str("3.5"),
        Val::str(" 42 "),
        Val::str("not_a_number"),
        Val::Int(7),
        Val::Int(-3),
        Val::Int(0),
        Val::Float(1.5),
        Val::Float(1000000.0),
        Val::Bool(true),
        Val::Bool(false),
        Val::List(vec![Val::str("a"), Val::str("b"), Val::str("c")]),
        Val::StrList(vec![
            "a".to_string(),
            "".to_string(),
            "  ".to_string(),
            "b".to_string(),
        ]),
        Val::List(vec![Val::Int(1), Val::str("two"), Val::Null, Val::Bool(true)]),
        Val::Obj(m),
        Val::Str(format!("{}u00e9 and {}x41", 92u8 as char, 92u8 as char)),
        Val::str("0"),
        Val::str("9223372036854775808"),
    ]
}

fn repr(v: &Val) -> String {
    match v {
        Val::Null => "nil".to_string(),
        Val::Str(s) => format!("s:{}", gofmt::quote(s)),
        Val::Int(i) => format!("i:{}", i),
        Val::Float(f) => format!("f:{}", gofmt::format_float_g(*f)),
        Val::Bool(b) => format!("b:{}", b),
        Val::StrList(x) => {
            let parts: Vec<String> = x.iter().map(|s| gofmt::quote(s)).collect();
            format!("ss:[{}]", parts.join(","))
        }
        Val::List(x) => {
            let parts: Vec<String> = x.iter().map(repr).collect();
            format!("as:[{}]", parts.join(","))
        }
        Val::Obj(m) => {
            let parts: Vec<String> =
                m.iter().map(|(k, val)| format!("{}:{}", gofmt::quote(k), repr(val))).collect();
            format!("m:{{{}}}", parts.join(","))
        }
    }
}

const HELPER_NAMES: &[&str] = &[
    "indent", "lower", "upper", "pascalCase", "camelCase", "snakeCase", "dasherize", "unquote",
    "unescape", "trimSuffix", "trimPrefix", "join", "split", "nonEmpty", "toQuoted", "escapeHtml",
    "decoded", "asString", "toPyLit", "toJSON", "toJSONIndent", "add", "sub", "mul", "div", "mod",
    "align", "percent", "stars",
];

const PAIR_LEFT: &[usize] =
    &[0, 1, 2, 9, 10, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 29, 30, 32];
const PAIR_RIGHT: &[usize] = &[1, 2, 17, 22, 24, 29, 30];

fn main() {
    let c = corpus();
    let mut out = String::new();
    let mut emit = |name: &str, args: &[Val]| {
        let (res, ok, err) = helpers::apply_helper(name, args);
        let reprs: Vec<String> = args.iter().map(repr).collect();
        let es = err.unwrap_or_default();
        let rs = match &res {
            Some(v) => repr(v),
            None => "-".to_string(),
        };
        out.push_str(&format!(
            "H\t{}\t[{}]\tok={}\terr={}\tres={}\n",
            name,
            reprs.join(" | "),
            ok,
            gofmt::quote(&es),
            rs
        ));
    };
    for name in HELPER_NAMES {
        for v in &c {
            emit(name, std::slice::from_ref(v));
        }
        for &i in PAIR_LEFT {
            for &j in PAIR_RIGHT {
                emit(name, &[c[i].clone(), c[j].clone()]);
            }
        }
        emit(name, &[]);
        emit(name, &[c[2].clone(), c[2].clone(), c[2].clone()]);
    }
    emit("definitelyNotAHelper", &[Val::str("x")]);
    print!("{}", out);
}
