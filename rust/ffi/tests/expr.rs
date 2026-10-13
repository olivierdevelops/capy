//! T-03 / R5 — every `Expr` kind, read through the C functions, equals the Rust writer's `expr_val`.
//!
//! `Call` is not produced by any grammar today (an `any` capture refuses `f(x)`), so the expressions are
//! built by hand and by a seeded generator and wrapped in a `ParseResult`.

mod support;

use capy_core::capy::ParseResult;
use capy_core::domain::ast::*;
use capy_core::domain::ast_json::expr_val;
use capy_core::gojson;
use capy_ffi::*;
use support::*;

fn wrap(exprs: Vec<Expr>) -> ParseResult {
    let mut r = ParseResult::default();
    for (i, e) in exprs.into_iter().enumerate() {
        let mut f = FuncCall::new("probe", i + 1, 1);
        let mut c = CaptureValue::default();
        c.is_expr = true;
        c.text = format!("e{i}");
        c.expr = Some(e);
        f.captures.insert("e".to_string(), c);
        r.tree.stmts.push(f);
    }
    r
}

fn num_i(i: i64) -> Expr {
    Expr::Number(NumberLit { is_int: true, i, f: 0.0 })
}
fn num_f(f: f64) -> Expr {
    Expr::Number(NumberLit { is_int: false, i: 0, f })
}
fn var(parts: &[&str]) -> Expr {
    Expr::Var(parts.iter().map(|p| PathStep::field(*p)).collect())
}

/// A tiny xorshift so the test is seeded and reproducible without a dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn gen(r: &mut Rng, depth: u32) -> Expr {
    let leaf = depth == 0;
    match r.below(if leaf { 5 } else { 12 }) {
        0 => num_i([i64::MIN, i64::MAX, -1, 0, 1, 42, (r.next() as i64) >> 3][r.below(7) as usize]),
        1 => num_f([0.5, -2.25, 1e300, 1e-300, 3.25][r.below(5) as usize]),
        2 => Expr::Str(["", "a", "héllo", "with \"quotes\"", "tab\there", "日本語"][r.below(6) as usize].to_string()),
        3 => Expr::Bool(r.below(2) == 0),
        4 => {
            if r.below(2) == 0 {
                Expr::Null
            } else {
                var(&["a", "b"])
            }
        }
        5 => Expr::Var(
            (0..=r.below(3))
                .map(|i| if i > 0 && r.below(2) == 0 { PathStep::index(gen(r, depth - 1)) } else { PathStep::field(format!("f{i}")) })
                .collect(),
        ),
        6 => Expr::Call(CallExpr { name: vec!["m".into(), "fn".into()][..1 + r.below(2) as usize].to_vec(), args: (0..r.below(4)).map(|_| gen(r, depth - 1)).collect() }),
        7 => Expr::Compare(Box::new(CompareExpr { op: ["<", "==", ">="][r.below(3) as usize].into(), left: gen(r, depth - 1), right: gen(r, depth - 1) })),
        8 => Expr::Binary(Box::new(BinaryExpr { op: ["+", "**", "<=>", "in"][r.below(4) as usize].into(), left: gen(r, depth - 1), right: gen(r, depth - 1) })),
        9 => {
            if r.below(2) == 0 {
                Expr::Not(Box::new(gen(r, depth - 1)))
            } else {
                Expr::Unary(Box::new(UnaryExpr { op: ["-", "~"][r.below(2) as usize].into(), operand: gen(r, depth - 1) }))
            }
        }
        10 => Expr::List((0..r.below(4)).map(|_| gen(r, depth - 1)).collect()),
        _ => {
            let n = r.below(4) as usize;
            Expr::Obj(ObjLit { keys: (0..n).map(|i| format!("k{i}")).collect(), vals: (0..n).map(|_| gen(r, depth - 1)).collect() })
        }
    }
}

fn kind_of(e: &Expr) -> u8 {
    match e {
        Expr::Number(_) => CAPY_EXPR_NUMBER,
        Expr::Str(_) => CAPY_EXPR_STR,
        Expr::Bool(_) => CAPY_EXPR_BOOL,
        Expr::Null => CAPY_EXPR_NULL,
        Expr::Var(_) => CAPY_EXPR_VAR,
        Expr::Call(_) => CAPY_EXPR_CALL,
        Expr::Compare(_) => CAPY_EXPR_COMPARE,
        Expr::Binary(_) => CAPY_EXPR_BINARY,
        Expr::Not(_) => CAPY_EXPR_NOT,
        Expr::Unary(_) => CAPY_EXPR_UNARY,
        Expr::List(_) => CAPY_EXPR_LIST,
        Expr::Obj(_) => CAPY_EXPR_OBJ,
    }
}

fn collect_kinds(e: &Expr, seen: &mut [bool; 13]) {
    seen[kind_of(e) as usize] = true;
    match e {
        Expr::Var(s) => s.iter().for_each(|st| {
            if let Some(i) = &st.index {
                collect_kinds(i, seen)
            }
        }),
        Expr::Call(c) => c.args.iter().for_each(|a| collect_kinds(a, seen)),
        Expr::Compare(c) => {
            collect_kinds(&c.left, seen);
            collect_kinds(&c.right, seen)
        }
        Expr::Binary(c) => {
            collect_kinds(&c.left, seen);
            collect_kinds(&c.right, seen)
        }
        Expr::Not(x) => collect_kinds(x, seen),
        Expr::Unary(u) => collect_kinds(&u.operand, seen),
        Expr::List(v) => v.iter().for_each(|a| collect_kinds(a, seen)),
        Expr::Obj(o) => o.vals.iter().for_each(|a| collect_kinds(a, seen)),
        _ => {}
    }
}

fn check(exprs: Vec<Expr>) {
    let want: Vec<String> = exprs.iter().map(|e| gojson::marshal(&expr_val(e))).collect();
    let h = handle_of(wrap(exprs));
    let root = capy_root(h.0);
    assert_eq!(capy_block_len(h.0, root) as usize, want.len());
    for (i, w) in want.iter().enumerate() {
        let st = capy_block_stmt(h.0, root, i as u32);
        let cap = unsafe { capy_node_capture(h.0, st, b"e".as_ptr(), 1) };
        let e = capy_cap_expr(h.0, cap);
        assert_ne!(e, 0, "capture {i} carries an expression");
        assert_eq!(&gojson::marshal(&expr(h.0, e)), w, "expression {i}");
    }
}

#[test]
fn every_kind_by_hand() {
    check(vec![
        num_i(0),
        num_i(i64::MIN),
        num_i(i64::MAX),
        num_f(-0.5),
        Expr::Str("héllo".into()),
        Expr::Bool(true),
        Expr::Bool(false),
        Expr::Null,
        var(&["a", "b", "c"]),
        Expr::Call(CallExpr { name: vec!["a".into(), "b".into()], args: vec![num_i(1), Expr::Str("x".into())] }),
        Expr::Compare(Box::new(CompareExpr { op: "<".into(), left: var(&["x"]), right: num_i(3) })),
        Expr::Binary(Box::new(BinaryExpr { op: "**".into(), left: num_i(2), right: num_i(3) })),
        Expr::Not(Box::new(Expr::Bool(false))),
        Expr::Unary(Box::new(UnaryExpr { op: "~".into(), operand: num_i(7) })),
        Expr::List(vec![num_i(1), Expr::Null, Expr::List(vec![])]),
        Expr::Obj(ObjLit { keys: vec!["a".into(), "b".into()], vals: vec![num_i(1), Expr::Str("two".into())] }),
        Expr::Var(vec![PathStep::field("a"), PathStep::index(num_i(2)), PathStep::field("c")]),
    ]);
}

#[test]
fn an_index_step_without_an_expression_reads_as_zero() {
    // PathStep's fields are public, so `is_index` with `index: None` is constructible; the C view must not crash.
    let weird = Expr::Var(vec![PathStep::field("a"), PathStep { is_index: true, field: String::new(), index: None }]);
    check(vec![weird]);
}

#[test]
fn seeded_random_trees_equal_the_rust_writer() {
    let mut r = Rng(0x9E3779B97F4A7C15);
    let mut seen = [false; 13];
    let mut all = Vec::new();
    for _ in 0..3000 {
        let e = gen(&mut r, 4);
        collect_kinds(&e, &mut seen);
        all.push(e);
    }
    for k in 1..=12u8 {
        assert!(seen[k as usize], "generator never produced kind {k}");
    }
    for chunk in all.chunks(250) {
        check(chunk.to_vec());
    }
}

#[test]
fn wrong_kind_and_foreign_ids_read_as_zero() {
    let h = handle_of(wrap(vec![num_i(1)]));
    let other = handle_of(wrap(vec![num_i(1)]));
    let root = capy_root(h.0);
    assert_eq!(capy_expr_kind(h.0, root), 0, "a block id is not an expression");
    assert_eq!(capy_expr_kind(h.0, 0), 0);
    assert_eq!(capy_expr_kind(h.0, 123456), 0);
    let st = capy_block_stmt(h.0, root, 0);
    let cap = unsafe { capy_node_capture(h.0, st, b"e".as_ptr(), 1) };
    let e = capy_cap_expr(h.0, cap);
    assert_eq!(capy_expr_kind(h.0, e), CAPY_EXPR_NUMBER);
    // an id from `h` used against `other` is rejected (best effort tag check; the tags differ here)
    assert_eq!(capy_expr_kind(other.0, e), 0);
    let mut v = 99i64;
    assert_eq!(unsafe { capy_expr_int(h.0, root, &mut v) }, 0);
    assert_eq!(v, 99, "out is untouched on failure");
}
