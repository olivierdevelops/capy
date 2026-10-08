//! E13 + E14 — fused scanner and direct emit (PROP-2026-0007, architecture C prototype).
//!
//! For the leading run of a script whose statements are flat, single-line, and built only from the
//! shapes below, this recognises each line straight from the source bytes — no `Vec<Token>`, no cursor
//! machine. A statement becomes either
//!  * a compact `Direct` record (E14) when its function only `write`s plain `${capture}` / `${line}` /
//!    `${col}` templates, so it can be rendered without ever building a `FuncCall`; or
//!  * the SAME `FuncCall` the lexer + parser would build (E13), rendered by the evaluator as usual.
//!
//! At the first line it cannot DECIDE WITH PROOF it stops, and the general lexer/parser continues from
//! that line (absolute line numbers). It never restarts and never re-enters (R17 (c)).
//!
//! It may not guess:
//!  * token boundaries come from the lexer's own helper functions, tried in the lexer's own branch
//!    order, so a character can never be classified differently;
//!  * candidates are tried in the original precedence order via the E4 dispatch lists, which are a
//!    sound superset of what could match. A candidate this module cannot compile that appears ahead
//!    of the match is "may match" → hand off (R17 (b), the exclusion proof);
//!  * no candidate matching → hand off, so the general path produces the diagnostic;
//!  * rendering happens only AFTER the rest of the script has parsed, so a parse error anywhere still
//!    wins over an evaluation error earlier in the file, exactly as before;
//!  * argument validation is the evaluator's own `check_type`.
//!
//! Compiled shapes: literals; `ident`, `raw`, a final `tail`; and a final expression capture
//! (`int`/`float`/`string`/`bool`/`any`) that is exactly one number or string token at end of line
//! (anything longer might continue as an expression, which only `parse_value` may decide).

use super::expr_to_text::expr_to_text;
use super::inner_evaluator::{compile_template, CAtom, Part};
use super::make_evaluator::OuterEval;
use super::make_lexer as lx;
use super::make_parser::Candidates;
use crate::domain::ast::{Block, CaptureValue, Expr, FuncCall, InnerStmt, NumberLit, Span};
use crate::domain::errors::CapyError;
use crate::domain::library::{FuncDef, Library};
use crate::domain::token::TokenKind;
use crate::gofmt;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

/// Most captures a compiled function may have (keeps per-line state on the stack).
const MAXC: usize = 8;

pub(crate) enum Op {
    Lit(String),
    CapIdent(String),
    CapRaw(String),
    CapTail(String),
    CapExpr(String),
}

/// E14: how to render a function that only `write`s plain templates.
pub(crate) struct Direct {
    parts: Vec<DPart>,
    checks: Vec<Check>,
    fname: String,
}

enum DPart {
    Static(String),
    Cap(usize),
    Line,
    Col,
}

struct Check {
    slot: usize,
    arg_name: String,
    type_: String,
}

pub(crate) struct CompiledFn {
    pub name: String,
    pub ops: Vec<Op>,
    direct: Option<Direct>,
}

/// `None` = this function is not handled here (and so counts as "may match" whenever it is a candidate).
pub(crate) fn compile(f: &FuncDef, lib: &Library) -> Option<CompiledFn> {
    if f.block.is_some() || f.lookahead.is_some() || f.elements.is_empty() {
        return None;
    }
    let mut ops = Vec::with_capacity(f.elements.len());
    let mut caps = 0usize;
    let last = f.elements.len() - 1;
    for (i, e) in f.elements.iter().enumerate() {
        if !e.is_capture {
            if e.literal.is_empty() {
                return None;
            }
            ops.push(Op::Lit(e.literal.clone()));
            continue;
        }
        if e.is_func
            || e.optional
            || !e.repeat.is_empty()
            || !e.alts.is_empty()
            || lib.types.contains_key(&e.cap_type)
            || lib.functions.contains_key(&e.cap_type)
        {
            return None;
        }
        let n = e.name.clone();
        ops.push(match e.cap_type.as_str() {
            "ident" => Op::CapIdent(n),
            "raw" => Op::CapRaw(n),
            "tail" if i == last => Op::CapTail(n),
            "int" | "float" | "string" | "bool" | "any" if i == last => Op::CapExpr(n),
            _ => return None,
        });
        caps += 1;
    }
    if caps > MAXC {
        return None;
    }
    let direct = build_direct(f, &ops);
    Some(CompiledFn { name: f.name.clone(), ops, direct })
}

fn cap_name(op: &Op) -> Option<&str> {
    match op {
        Op::Lit(_) => None,
        Op::CapIdent(n) | Op::CapRaw(n) | Op::CapTail(n) | Op::CapExpr(n) => Some(n),
    }
}

/// A function is direct-renderable if it has no run pass and its template is only `write`s of
/// literal text, escapes, and `${name}` where `name` is one of its captures (or `line` / `col`).
fn build_direct(f: &FuncDef, ops: &[Op]) -> Option<Direct> {
    if f.run_ast.is_some() {
        return None;
    }
    // capture slots in order; a repeated name resolves to the last, as the capture map would
    let mut slot_of: HashMap<&str, usize> = HashMap::new();
    let mut slot = 0usize;
    for op in ops {
        if let Some(n) = cap_name(op) {
            slot_of.insert(n, slot);
            slot += 1;
        }
    }
    let mut parts: Vec<DPart> = Vec::new();
    if let Some(t) = &f.template_ast {
        for st in &t.stmts {
            let InnerStmt::Write(Expr::Str(s)) = st else { return None };
            for p in compile_template(s)? {
                match p {
                    Part::Static(t) => parts.push(DPart::Static(t)),
                    Part::Interp(stages) => {
                        if stages.len() != 1 || stages[0].atoms.len() != 1 {
                            return None;
                        }
                        let CAtom::Name(n) = &stages[0].atoms[0] else { return None };
                        // captures are inserted into the locals after line/col, so they win
                        parts.push(if let Some(&s) = slot_of.get(n.as_str()) {
                            DPart::Cap(s)
                        } else if n == "line" {
                            DPart::Line
                        } else if n == "col" {
                            DPart::Col
                        } else {
                            return None;
                        });
                    }
                }
            }
        }
    }
    let mut checks = Vec::new();
    for a in &f.args {
        if a.kind != "capture" {
            continue;
        }
        let slot = *slot_of.get(a.name.as_str())?;
        checks.push(Check { slot, arg_name: a.name.clone(), type_: a.type_.clone() });
    }
    Some(Direct { parts, checks, fname: f.name.clone() })
}

#[derive(Clone, Copy)]
struct STok {
    kind: TokenKind,
    /// Byte range in the line (a string includes its quotes).
    start: usize,
    end: usize,
    /// For a string: the content range (no quotes, no escapes).
    cs: usize,
    ce: usize,
}

/// Tokenise one indent-0 line the way `tokenize_line` does. `false` = not decidable here.
fn tokenize(line: &str, markers: &[String], out: &mut Vec<STok>) -> bool {
    out.clear();
    let b = line.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let r = line[i..].chars().next().unwrap();
        if r == ' ' {
            i += 1;
        } else if lx::has_comment_prefix(&line[i..], markers) {
            break;
        } else if r == '"' || r == '\'' {
            let q = r as u8;
            let mut j = i + 1;
            loop {
                if j >= b.len() {
                    return false; // unterminated: the lexer reports it
                }
                let c = b[j];
                if c == b'\\' {
                    return false; // escapes change the content; leave them to the lexer
                }
                if c == q {
                    break;
                }
                j += 1;
            }
            out.push(STok { kind: TokenKind::Str, start: i, end: j + 1, cs: i + 1, ce: j });
            i = j + 1;
        } else if r == '`' || matches!(r, '(' | ')' | '[' | ']' | '{' | '}') {
            return false;
        } else if lx::is_digit(r) || (r == '-' && i + 1 < b.len() && (b[i + 1] as char).is_ascii_digit()) {
            let n = lx::read_number(&line[i..]);
            out.push(STok { kind: TokenKind::Number, start: i, end: i + n, cs: 0, ce: 0 });
            i += n;
        } else if lx::is_ident_start(r) {
            let n = lx::read_ident(&line[i..]);
            out.push(STok { kind: TokenKind::Ident, start: i, end: i + n, cs: 0, ce: 0 });
            i += n;
        } else if (r as u32) < 0x80 && lx::is_punct(r as u8) {
            let n = lx::read_punct(&line[i..]);
            out.push(STok { kind: TokenKind::Punct, start: i, end: i + n, cs: 0, ce: 0 });
            i += n;
        } else {
            return false;
        }
    }
    true
}

fn span(line_no: usize, a: &STok, z: &STok) -> Span {
    Span::new(line_no, a.start + 1, line_no, z.end + 1)
}

#[derive(Clone, Copy)]
enum RecKind {
    None,
    Plain,
    Int(i64),
    Float(f64),
    Str,
}

/// What one capture matched, held on the stack until we know whether to build a `FuncCall`.
#[derive(Clone, Copy)]
struct Rec {
    kind: RecKind,
    /// Token index range [ts, te).
    ts: usize,
    te: usize,
    /// Byte range in the line of the text the capture's value renders as.
    text: (usize, usize),
    /// True when that text is exactly what the parser's value would render as (so it can be emitted from the source).
    direct_ok: bool,
}

const NO_REC: Rec = Rec { kind: RecKind::None, ts: 0, te: 0, text: (0, 0), direct_ok: false };

/// An integer token whose canonical form (`i64::to_string`) is its own source text.
fn canonical_int(s: &str) -> bool {
    s.parse::<i64>().is_ok() && s != "-0" && !(s.len() > 1 && (s.starts_with('0') || s.starts_with("-0")))
}

/// `Ok(Some(n))` matched with `n` captures recorded; `Ok(None)` this shape does not match;
/// `Err(())` cannot be decided here.
fn match_ops(cf: &CompiledFn, line: &str, toks: &[STok], recs: &mut [Rec; MAXC]) -> Result<Option<usize>, ()> {
    let mut ti = 0usize;
    let mut n = 0usize;
    for op in &cf.ops {
        match op {
            Op::Lit(l) => {
                let Some(t) = toks.get(ti) else { return Ok(None) };
                if !matches!(t.kind, TokenKind::Ident | TokenKind::Punct) {
                    return Ok(None);
                }
                let text = &line[t.start..t.end];
                if text == l {
                    ti += 1;
                } else if t.kind == TokenKind::Punct && text.len() > l.len() && text.starts_with(l.as_str()) {
                    return Err(()); // a literal that splits a merged punctuation run
                } else {
                    return Ok(None);
                }
            }
            Op::CapIdent(_) => {
                let Some(t) = toks.get(ti) else { return Ok(None) };
                if t.kind != TokenKind::Ident {
                    return Ok(None);
                }
                recs[n] = Rec { kind: RecKind::Plain, ts: ti, te: ti + 1, text: (t.start, t.end), direct_ok: true };
                n += 1;
                ti += 1;
            }
            Op::CapRaw(_) => {
                let Some(t) = toks.get(ti) else { return Ok(None) };
                let text = match t.kind {
                    TokenKind::Ident => (t.start, t.end),
                    TokenKind::Str => (t.cs, t.ce),
                    _ => return Ok(None),
                };
                recs[n] = Rec { kind: RecKind::Plain, ts: ti, te: ti + 1, text, direct_ok: true };
                n += 1;
                ti += 1;
            }
            Op::CapTail(_) => {
                if ti >= toks.len() {
                    return Ok(None);
                }
                let rest = &toks[ti..];
                // A quoted argument is re-emitted with `"`; only a `"`-quoted string is its own source text.
                if rest.iter().any(|t| t.kind == TokenKind::Str && line.as_bytes()[t.start] != b'"') {
                    return Err(());
                }
                recs[n] = Rec {
                    kind: RecKind::Plain,
                    ts: ti,
                    te: toks.len(),
                    text: (toks[ti].start, toks[toks.len() - 1].end),
                    direct_ok: true,
                };
                n += 1;
                ti = toks.len();
            }
            Op::CapExpr(_) => {
                let Some(t) = toks.get(ti) else { return Ok(None) };
                if ti + 1 != toks.len() {
                    return Err(()); // might continue as an expression: only parse_value may decide
                }
                let rec = match t.kind {
                    TokenKind::Number => {
                        let s = &line[t.start..t.end];
                        if let Ok(i) = s.parse::<i64>() {
                            Rec { kind: RecKind::Int(i), ts: ti, te: ti + 1, text: (t.start, t.end), direct_ok: canonical_int(s) }
                        } else if let Ok(f) = s.parse::<f64>() {
                            Rec { kind: RecKind::Float(f), ts: ti, te: ti + 1, text: (t.start, t.end), direct_ok: false }
                        } else {
                            return Err(());
                        }
                    }
                    TokenKind::Str => {
                        let c = &line.as_bytes()[t.cs..t.ce];
                        // expr_to_text(Str) = gofmt::quote(content): equal to the source only for a
                        // `"`-quoted string of plain printable ASCII.
                        let plain = line.as_bytes()[t.start] == b'"' && c.iter().all(|&b| (0x20..0x7f).contains(&b));
                        Rec { kind: RecKind::Str, ts: ti, te: ti + 1, text: (t.start, t.end), direct_ok: plain }
                    }
                    _ => return Err(()),
                };
                recs[n] = rec;
                n += 1;
                ti += 1;
            }
        }
    }
    if ti != toks.len() {
        return Ok(None); // a flat statement must end the line
    }
    Ok(Some(n))
}

/// The `FuncCall` the parser would build.
fn build_call(cf: &CompiledFn, line: &str, toks: &[STok], line_no: usize, recs: &[Rec; MAXC], n: usize) -> FuncCall {
    let mut caps: BTreeMap<String, CaptureValue> = BTreeMap::new();
    let mut names = cf.ops.iter().filter_map(cap_name);
    for r in recs.iter().take(n) {
        let name = names.next().expect("one name per recorded capture");
        let (a, z) = (&toks[r.ts], &toks[r.te - 1]);
        let sp = span(line_no, a, z);
        let cv = match r.kind {
            RecKind::Int(i) => {
                let x = Expr::Number(NumberLit { is_int: true, i, f: 0.0 });
                let text = expr_to_text(&x);
                CaptureValue { is_expr: true, expr: Some(x), text, sub: Vec::new(), span: sp }
            }
            RecKind::Float(f) => {
                let x = Expr::Number(NumberLit { is_int: false, i: 0, f });
                let text = expr_to_text(&x);
                CaptureValue { is_expr: true, expr: Some(x), text, sub: Vec::new(), span: sp }
            }
            RecKind::Str => {
                let x = Expr::Str(line[a.cs..a.ce].to_string());
                let text = expr_to_text(&x);
                CaptureValue { is_expr: true, expr: Some(x), text, sub: Vec::new(), span: sp }
            }
            _ => CaptureValue { text: line[r.text.0..r.text.1].to_string(), span: sp, ..Default::default() },
        };
        caps.insert(name.to_string(), cv);
    }
    let (first, last) = (&toks[0], &toks[toks.len() - 1]);
    FuncCall {
        func: cf.name.clone(),
        captures: caps,
        body: None,
        closer: None,
        sections: BTreeMap::new(),
        line: line_no,
        col: first.start + 1,
        span: span(line_no, first, last),
        leading_comments: Vec::new(),
    }
}

pub(crate) enum Item {
    Call(Box<FuncCall>),
    /// A statement rendered straight from the source: function index (into the candidates), position,
    /// and where its capture ranges start in `Scan::caps`.
    Direct { f: u32, line: u32, col: u32, caps: u32 },
}

pub(crate) struct Scan {
    pub items: Vec<Item>,
    /// Flat (start, end) byte ranges into the whole source, `Direct` items point into this.
    pub caps: Vec<(u32, u32)>,
    /// The first line (1-based) the scanner did not decide; `None` = the whole script was decided.
    pub resume_line: Option<usize>,
}

/// `direct = false` builds a `FuncCall` for every statement (the structural-differential mode).
pub(crate) fn scan(cands: &Candidates, markers: &[String], src: &str, direct: bool) -> Scan {
    let mut items: Vec<Item> = Vec::new();
    let mut caps: Vec<(u32, u32)> = Vec::new();
    let mut toks: Vec<STok> = Vec::with_capacity(32);
    if src.len() >= u32::MAX as usize {
        return Scan { items, caps, resume_line: Some(1) };
    }
    let mut off = 0usize;
    for (idx, line) in src.split('\n').enumerate() {
        let line_no = idx + 1;
        let line_off = off;
        off += line.len() + 1;
        let rest = line.trim();
        if rest.is_empty() {
            continue;
        }
        // Raw-byte conditions that change how the lexer treats a line, and indentation.
        if line.bytes().any(|c| c == b'\r' || c == b'\t' || c == b'`') || line.starts_with(' ') {
            return Scan { items, caps, resume_line: Some(line_no) };
        }
        if lx::has_comment_prefix(rest, markers) {
            continue;
        }
        if !tokenize(line, markers, &mut toks) || toks.is_empty() {
            return Scan { items, caps, resume_line: Some(line_no) };
        }
        let first = toks[0];
        let ftext = match first.kind {
            TokenKind::Str => &line[first.cs..first.ce],
            _ => &line[first.start..first.end],
        };
        let (bucket, wild) = cands.lists(first.kind, ftext);
        let (mut bi, mut wi) = (0usize, 0usize);
        let mut done = false;
        let mut recs = [NO_REC; MAXC];
        loop {
            let ci = match (bucket.get(bi), wild.get(wi)) {
                (Some(&b), Some(&w)) => {
                    if b < w {
                        bi += 1;
                        b
                    } else {
                        wi += 1;
                        w
                    }
                }
                (Some(&b), None) => {
                    bi += 1;
                    b
                }
                (None, Some(&w)) => {
                    wi += 1;
                    w
                }
                (None, None) => break,
            } as usize;
            let Some(cf) = cands.compiled(ci) else {
                // an uncompiled candidate may match: not provably excluded
                return Scan { items, caps, resume_line: Some(line_no) };
            };
            match match_ops(cf, line, &toks, &mut recs) {
                Ok(Some(n)) => {
                    let all_direct = recs.iter().take(n).all(|r| r.direct_ok);
                    if direct && all_direct && cf.direct.is_some() {
                        let base = caps.len() as u32;
                        for r in recs.iter().take(n) {
                            caps.push(((line_off + r.text.0) as u32, (line_off + r.text.1) as u32));
                        }
                        items.push(Item::Direct { f: ci as u32, line: line_no as u32, col: (first.start + 1) as u32, caps: base });
                    } else {
                        items.push(Item::Call(Box::new(build_call(cf, line, &toks, line_no, &recs, n))));
                    }
                    done = true;
                    break;
                }
                Ok(None) => continue,
                Err(()) => return Scan { items, caps, resume_line: Some(line_no) },
            }
        }
        if !done {
            return Scan { items, caps, resume_line: Some(line_no) }; // nothing matched: the general path reports it
        }
    }
    Scan { items, caps, resume_line: None }
}

/// Render the scanned prefix, then `rest` (the statements the general path parsed after the handoff).
pub(crate) fn render(
    sc: &Scan,
    ev: &mut OuterEval<'_>,
    cands: &Candidates,
    src: &str,
    rest: &Block,
) -> Result<String, CapyError> {
    let mut out = String::new();
    for it in &sc.items {
        match it {
            Item::Call(c) => out.push_str(&ev.render_call(c)?),
            Item::Direct { f, line, col, caps } => {
                let Some(d) = cands.compiled(*f as usize).and_then(|c| c.direct.as_ref()) else {
                    return Err(CapyError::msg("internal: direct item without a direct template"));
                };
                let base = *caps as usize;
                let text = |slot: usize| -> &str {
                    let (a, z) = sc.caps[base + slot];
                    &src[a as usize..z as usize]
                };
                // validate_args, for the arguments that apply to a scanned statement
                for ck in &d.checks {
                    if let Err(e) = ev.check_arg(&ck.type_, text(ck.slot)) {
                        let msg = format!(
                            "function {} arg {}: {}",
                            gofmt::quote(&d.fname),
                            gofmt::quote(&ck.arg_name),
                            e.msg
                        );
                        if e.plain {
                            return Err(CapyError::msg(msg));
                        }
                        return Err(CapyError { msg, hint: e.hint, plain: false, ..Default::default() });
                    }
                }
                for p in &d.parts {
                    match p {
                        DPart::Static(t) => out.push_str(t),
                        DPart::Cap(s) => out.push_str(text(*s)),
                        DPart::Line => {
                            let _ = write!(out, "{}", line);
                        }
                        DPart::Col => {
                            let _ = write!(out, "{}", col);
                        }
                    }
                }
            }
        }
    }
    out.push_str(&ev.render_block(rest)?);
    Ok(out)
}
