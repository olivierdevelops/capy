//! Port of `orchestrator/features/make_parser.go`.
//!
//! The outer pattern matcher. It walks the token stream and at each statement
//! boundary tries each library function's compiled `elements` in priority order;
//! the first complete match wins.

use super::expr_to_text::expr_to_text;
use super::value_parser::{clear_depth_error, parse_value, take_depth_error, TokReader};
use crate::domain::ast::{ErrorNode, Span, Block, CaptureValue, FuncCall};
use crate::domain::errors::{codes, suggest_closest, CapyError, ContextFrame, Diagnostic};
use crate::domain::library::{CloseSegment, FuncDef, Library, PatternElement, TypeDef};
use crate::domain::token::{Token, TokenKind};
use crate::gofmt;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Port of `MakeParser(...).Parse`.
/// PLAN-2026-0002 R20/R22 — parse, recovering from statement failures.
///
/// Returns the tree AND every diagnostic. The tree is always present; broken
/// regions appear in `Block.errors` while the statements around them are intact,
/// which is what lets an editor keep working on a buffer that is mid-edit.
pub fn parse_recovering(
    toks: Vec<Token>,
    src: &str,
    lib: &Library,
) -> (Block, Vec<Diagnostic>) {
    parse_recovering_with(toks, src, lib, None)
}

/// As [`parse_recovering`], reusing precomputed candidates (PROP-2026-0007 E4a).
pub(crate) fn parse_recovering_with(
    toks: Vec<Token>,
    src: &str,
    lib: &Library,
    cands: Option<Arc<Candidates>>,
) -> (Block, Vec<Diagnostic>) {
    match parse_impl(toks, src, lib, true, cands) {
        Ok((b, d)) => (b, d),
        // A failure with recovery ON is a parse that could not even start.
        Err(e) => (
            Block::default(),
            vec![Diagnostic::error(
                codes::NO_MATCH,
                Span::new(e.line, e.col, e.line, e.col + 1),
                e.msg,
            )],
        ),
    }
}

pub fn parse(toks: Vec<Token>, src: &str, lib: &Library) -> Result<Block, CapyError> {
    parse_impl(toks, src, lib, false, None).map(|(b, _)| b)
}

/// As [`parse`], reusing precomputed candidates (PROP-2026-0007 E4a).
pub(crate) fn parse_with(
    toks: Vec<Token>,
    src: &str,
    lib: &Library,
    cands: &Arc<Candidates>,
) -> Result<Block, CapyError> {
    parse_impl(toks, src, lib, false, Some(cands.clone())).map(|(b, _)| b)
}

fn parse_impl(
    toks: Vec<Token>,
    src: &str,
    lib: &Library,
    recover: bool,
    cands: Option<Arc<Candidates>>,
) -> Result<(Block, Vec<Diagnostic>), CapyError> {
    // PLAN-2026-0001 R27 — pull comment trivia OUT of the stream before anything
    // matches against it. The matcher is a token-index machine; letting a token
    // kind it has never seen through would change what parses, which is the one
    // thing this plan may not do (R12). Each retained comment is filed under the
    // index of the next real token, which is the statement it leads.
    let mut leading: BTreeMap<usize, Vec<Span>> = BTreeMap::new();
    // PROP-2026-0007 E9: with no comment token this loop is the identity, so skip the copy.
    let toks: Vec<Token> = if !toks.iter().any(|t| t.kind == TokenKind::Comment) {
        // The rebuilt vector below had exact capacity; keep that, so the lexer's
        // growth slack (about 24 MB at 625 000 tokens) is not held for the whole parse.
        let mut toks = toks;
        toks.shrink_to_fit();
        toks
    } else {
        let mut clean: Vec<Token> = Vec::with_capacity(toks.len());
        let mut pending: Vec<Span> = Vec::new();
        for t in toks {
            if t.kind == TokenKind::Comment {
                let width = if t.width > 0 { t.width } else { t.text.len() };
                pending.push(Span::new(t.line, t.col, t.line, t.col + width));
                continue;
            }
            if !pending.is_empty() && t.col != 0 {
                // Attach to the next SOURCE token; structural tokens (NEWLINE,
                // INDENT, DEDENT) have col 0 and are not where a comment leads.
                leading.entry(clean.len()).or_default().append(&mut pending);
            }
            clean.push(t);
        }
        clean
    };
    // E4a: candidate construction and sorting happen once per library, not once per run.
    let cands = cands.unwrap_or_else(|| Arc::new(Candidates::build(lib)));
    let mut pp = OuterP {
        toks,
        pos: 0,
        depth: 0,
        leading,
        furthest: Furthest::default(),
        depth_err: None,
        fail_code: codes::NO_MATCH,
        ctx_stack: Vec::new(),
        diagnostics: Vec::new(),
        last_reported: None,
        recover,
        cands,
        src_lines: split_source_lines(src),
        seq_depth: 0,
        undo: Vec::new(),
    };
    let block = pp.parse_program(false, "")?;
    Ok((block, pp.diagnostics))
}

/// Port of `startsWithLiteral`.
fn starts_with_literal(p: &FuncDef) -> bool {
    !p.elements.is_empty() && !p.elements[0].is_capture
}

/// Port of `literalLength`.
fn literal_length(p: &FuncDef) -> usize {
    let mut n = 0usize;
    for e in &p.elements {
        if !e.is_capture {
            n += 1;
        } else {
            break;
        }
    }
    n
}

/// Port of `splitSourceLines` — mirrors the lexer's `splitLines`: normalise CRLF,
/// then split on `\n`. Keeps line indexing aligned with token line numbers.
fn split_source_lines(s: &str) -> Vec<String> {
    s.replace("\r\n", "\n").split('\n').map(|x| x.to_string()).collect()
}

/// PLAN-2026-0002 R15 — what the matcher wanted at the token it gave up on.
///
/// The library enumerates every legal statement shape, so at any token the set
/// of things that could come next is computable directly — a general parser
/// generator has to reconstruct it from item sets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expectation {
    /// An exact token, e.g. `)`.
    Literal(String),
    /// A token of a kind, e.g. an identifier.
    Kind(String),
    /// A nonterminal — a capture whose type names another library function.
    Nonterminal(String),
    /// One of a declared type's `options`.
    OneOf(Vec<String>),
    /// A closing delimiter, carrying where the opener was.
    CloseDelim { close: String, open_line: usize, open_col: usize },
    /// A block's closing keyword.
    BlockEnd(String),
}

impl Expectation {
    fn describe(&self) -> String {
        match self {
            Expectation::Literal(l) => format!("`{l}`"),
            Expectation::Kind(k) => format!("a {k}"),
            Expectation::Nonterminal(n) => format!("a `{n}`"),
            Expectation::OneOf(opts) => {
                format!("one of: {}", opts.join(", "))
            }
            Expectation::CloseDelim { close, .. } => format!("`{close}`"),
            Expectation::BlockEnd(e) => format!("`{e}` to close the block"),
        }
    }
}

/// PLAN-2026-0002 R14 — the furthest-progressing match attempt.
///
/// Every shape is tried; on total failure the old code reported the statement's
/// first token, which told the user only what they already knew. This remembers
/// which attempt got furthest and what it wanted there. Deliberately NOT restored
/// on backtrack — that is the whole point.
#[derive(Debug, Default, Clone)]
struct Furthest {
    /// Token index reached. 0 means nothing recorded yet.
    index: usize,
    /// What every attempt that tied at `index` wanted, in the order recorded.
    /// Kept as references and only turned into the `BTreeSet<Expectation>` of
    /// the old code when a diagnostic is actually built (PROP-2026-0007 E2): most
    /// statements never fail, so allocating strings per rejected candidate was pure cost.
    lazy: Vec<LazyExp>,
    /// Where the matcher was; see `ContextFrame`. Resolved to frames on demand.
    ctx: Vec<LiteFrame>,
}

/// A `ContextFrame` without the allocation: which shape, and which argument.
/// The argument name is `func.elements[arg_index].name`, as the old code copied it.
#[derive(Debug, Clone)]
struct LiteFrame {
    func: Arc<FuncDef>,
    arg_index: usize,
}

/// A recorded expectation, resolved lazily. `Lit` is the common one: a shape's
/// literal that the input did not have.
#[derive(Debug, Clone)]
enum LazyExp {
    Lit { func: Arc<FuncDef>, elem: usize },
    Owned(Expectation),
}

impl Furthest {
    /// The old `expected` set, rebuilt: same members, same ordering and de-duplication.
    fn expected_set(&self) -> BTreeSet<Expectation> {
        self.lazy
            .iter()
            .map(|l| match l {
                LazyExp::Lit { func, elem } => Expectation::Literal(func.elements[*elem].literal.clone()),
                LazyExp::Owned(e) => e.clone(),
            })
            .collect()
    }

    /// The old `context` vector, rebuilt.
    fn context_frames(&self) -> Vec<ContextFrame> {
        self.ctx
            .iter()
            .map(|f| {
                let name = f.func.elements.get(f.arg_index).map(|e| e.name.clone()).unwrap_or_default();
                ContextFrame::new(&f.func.name, f.arg_index, name)
            })
            .collect()
    }
}

/// PLAN-2026-0001 R0 — how deep a nonterminal descent may go before the parser
/// gives up with an error instead of exhausting the native stack.
///
/// The load-time guard (`reject_left_recursion`) stops a library from recursing
/// without consuming input, but it cannot stop *input* from being deeply nested:
/// a perfectly valid right-recursive grammar fed 20 000 nested parentheses
/// overflowed the stack and aborted with rc=134, which an embedder cannot catch.
/// 64 is far beyond any hand-written source, and low enough to stay inside the 2 MiB
/// stack a Rust test thread gets (the main thread gets 8 MiB, so a limit tuned to
/// the main thread alone would still abort under `cargo test`).
const MAX_PARSE_DEPTH: usize = 64;

/// The library's functions in precedence order, plus a first-token index
/// (PROP-2026-0007 E4a/E4b/E4c). Built once per library.
///
/// The index only NARROWS the candidate list; it never reorders it. A statement
/// is still tried against its candidates in the original sorted order, so
/// "first match wins" is unchanged.
pub(crate) struct Candidates {
    /// Sorted by priority, literal-start, literal run, name (the old per-run sort).
    fns: Vec<Arc<FuncDef>>,
    by_name: BTreeMap<String, Arc<FuncDef>>,
    types: BTreeMap<String, TypeDef>,
    /// Functions whose first element is a literal, keyed by that literal. Ascending indices.
    by_lit: HashMap<String, Vec<u32>>,
    /// Functions whose first element is not a plain literal, ascending, by the token
    /// kind of the statement: an `Ident`, a `Str`, or anything else.
    wild_ident: Vec<u32>,
    wild_str: Vec<u32>,
    wild_other: Vec<u32>,
    /// Longest first-literal, for the merged-punctuation prefix lookup.
    max_lit_len: usize,
    /// E13: each function compiled for the fused scanner, parallel to `fns` (`None` = not handled there).
    compiled: Vec<Option<super::fast_path::CompiledFn>>,
}

/// What token kinds a function's first element can begin with.
enum FirstKind {
    /// A literal `text` (kinds Ident, Punct and the brackets, per `match_literal`).
    Literal(String),
    /// `ident` / `dotted_ident`: an `Ident` token only (`capture_value`).
    IdentOnly,
    /// `raw`: an `Ident` or a `Str` token.
    IdentOrStr,
    /// Anything else — expression-typed (`int`, `string`, `any`, …), custom types,
    /// nonterminals, optional captures, `tail`, `word`, and functions with no elements.
    /// Not narrowed: those types do NOT restrict the first token's kind.
    Any,
}

fn first_kind(f: &FuncDef) -> FirstKind {
    let Some(e) = f.elements.first() else { return FirstKind::Any };
    if !e.is_capture {
        if e.literal.is_empty() {
            return FirstKind::Any;
        }
        return FirstKind::Literal(e.literal.clone());
    }
    if e.is_func || e.optional {
        return FirstKind::Any;
    }
    match e.cap_type.as_str() {
        "ident" | "dotted_ident" => FirstKind::IdentOnly,
        "raw" => FirstKind::IdentOrStr,
        _ => FirstKind::Any,
    }
}

impl Candidates {
    pub(crate) fn build(lib: &Library) -> Candidates {
        let mut fns: Vec<Arc<FuncDef>> = lib.functions.values().cloned().map(Arc::new).collect();
        fns.sort_by(|a, b| {
            // Priority descending.
            if a.priority != b.priority {
                return b.priority.cmp(&a.priority);
            }
            // Literal-starting candidates first.
            let li = starts_with_literal(a);
            let lj = starts_with_literal(b);
            if li != lj {
                return lj.cmp(&li);
            }
            // Longer literal runs first.
            let (x, y) = (literal_length(a), literal_length(b));
            if x != y {
                return y.cmp(&x);
            }
            // Final tiebreaker on name so candidate ordering is TOTAL and
            // deterministic. Without this, functions that tie on
            // (priority, literal-start, literal-length) kept Go's randomized
            // map-iteration order — making any keyword collision a heisenbug.
            a.name.cmp(&b.name)
        });
        let by_name: BTreeMap<String, Arc<FuncDef>> =
            fns.iter().map(|f| (f.name.clone(), f.clone())).collect();
        let mut by_lit: HashMap<String, Vec<u32>> = HashMap::new();
        let (mut wild_ident, mut wild_str, mut wild_other) = (Vec::new(), Vec::new(), Vec::new());
        let mut max_lit_len = 0usize;
        for (i, f) in fns.iter().enumerate() {
            let i = i as u32;
            match first_kind(f) {
                FirstKind::Literal(l) => {
                    max_lit_len = max_lit_len.max(l.len());
                    by_lit.entry(l).or_default().push(i);
                }
                FirstKind::IdentOnly => wild_ident.push(i),
                FirstKind::IdentOrStr => {
                    wild_ident.push(i);
                    wild_str.push(i);
                }
                FirstKind::Any => {
                    wild_ident.push(i);
                    wild_str.push(i);
                    wild_other.push(i);
                }
            }
        }
        let compiled = fns.iter().map(|f| super::fast_path::compile(f, lib)).collect();
        Candidates {
            fns,
            by_name,
            compiled,
            types: lib.types.clone(),
            by_lit,
            wild_ident,
            wild_str,
            wild_other,
            max_lit_len,
        }
    }

    /// Indices of literal-first functions that could match a statement beginning
    /// with this token: exact text, or — for a merged punctuation run — any literal
    /// that is a strict prefix of it (`match_literal` splits such runs).
    fn bucket(&self, kind: TokenKind, text: &str) -> Cow<'_, [u32]> {
        let literal_kind = matches!(
            kind,
            TokenKind::Ident
                | TokenKind::Punct
                | TokenKind::LParen
                | TokenKind::RParen
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::LBrack
                | TokenKind::RBrack
        );
        if !literal_kind {
            return Cow::Borrowed(&[]);
        }
        let exact = self.by_lit.get(text).map(|v| v.as_slice()).unwrap_or(&[]);
        if kind != TokenKind::Punct || text.len() < 2 {
            return Cow::Borrowed(exact);
        }
        // Prefix lookups: every proper prefix of the merged run that is a literal.
        let mut all: Vec<u32> = exact.to_vec();
        let upper = text.len().min(self.max_lit_len + 1);
        for n in 1..upper {
            if text.is_char_boundary(n) {
                if let Some(v) = self.by_lit.get(&text[..n]) {
                    all.extend_from_slice(v);
                }
            }
        }
        all.sort_unstable();
        all.dedup();
        Cow::Owned(all)
    }

    /// Name → shared definition, for the evaluator (no per-run deep clone).
    pub(crate) fn funcs(&self) -> &BTreeMap<String, Arc<FuncDef>> {
        &self.by_name
    }

    /// The dispatch lists for a statement whose first token is `kind`/`text` (see `parse_stmt`).
    pub(crate) fn lists(&self, kind: TokenKind, text: &str) -> (Cow<'_, [u32]>, &[u32]) {
        (self.bucket(kind, text), self.wild(kind))
    }

    pub(crate) fn compiled(&self, i: usize) -> Option<&super::fast_path::CompiledFn> {
        self.compiled[i].as_ref()
    }

    fn wild(&self, kind: TokenKind) -> &[u32] {
        match kind {
            TokenKind::Ident => &self.wild_ident,
            TokenKind::Str => &self.wild_str,
            _ => &self.wild_other,
        }
    }
}

struct OuterP {
    toks: Vec<Token>,
    pos: usize,
    /// Current nonterminal-descent depth; see [`MAX_PARSE_DEPTH`].
    depth: usize,
    /// R27 — comment spans, keyed by the index of the token they lead.
    leading: BTreeMap<usize, Vec<Span>>,
    /// R14 — furthest-failure record; survives backtracking.
    furthest: Furthest,
    /// PLAN-2026-0004 — the nesting-bound error for the statement being parsed.
    /// `match_one` turns every sub-failure into a rewind, which used to swallow
    /// the bound's own message; it is remembered here so a statement that fails
    /// *because* of the bound can say so.
    depth_err: Option<CapyError>,
    /// Diagnostic code for the failure `parse_stmt` last returned.
    fail_code: &'static str,
    /// R26 — the shape/argument the matcher is currently inside.
    ctx_stack: Vec<LiteFrame>,
    /// R20 — diagnostics collected while recovering. Empty ⇒ clean parse.
    diagnostics: Vec<Diagnostic>,
    /// R19 — token index of the last reported diagnostic, for the spacing rule.
    last_reported: Option<usize>,
    /// R20 — recovery is opt-in, so `Library::run` keeps its first-error
    /// behaviour (R24) while `Library::parse` collects everything.
    recover: bool,
    cands: Arc<Candidates>,
    /// The original source split into lines (1-indexed via `src_lines[line-1]`).
    /// Used by `parse_verbatim_body` to capture a `block_verbatim` body as the raw
    /// byte range — preserving blank lines and comment-marker lines that produce
    /// no tokens.
    src_lines: Vec<String>,
    /// Counts how many sequence-closed blocks (`block_close_seq`) are currently
    /// open. Inside such a block, statements are packed on a line with no newline
    /// separators (`<p>"hi"</p>`), so a flat function need not be followed by a
    /// NEWLINE/EOF — leftover tokens simply become the next statement. `> 0`
    /// relaxes that requirement in `parse_stmt`.
    seq_depth: usize,
    /// Original tokens overwritten by punctuation splits, oldest first (PROP-2026-0007 E8).
    undo: Vec<(usize, Token)>,
}

/// See [`OuterP::mark`].
#[derive(Clone, Copy)]
struct Mark {
    pos: usize,
    undo: usize,
}

impl TokReader for OuterP {
    /// Go indexes `p.toks[p.pos]` directly; saturating on the terminal EOF token
    /// is equivalent for well-formed streams and avoids a panic otherwise.
    fn peek(&self) -> Token {
        if self.pos < self.toks.len() {
            self.toks[self.pos].clone()
        } else {
            self.toks.last().cloned().unwrap_or(Token {
                kind: TokenKind::Eof,
                text: String::new(),
                line: 0,
                col: 0,
                width: 0,
            })
        }
    }
    fn advance(&mut self) -> Token {
        let t = self.peek();
        if self.pos < self.toks.len() {
            self.pos += 1;
        }
        t
    }
    fn save(&self) -> usize {
        self.pos
    }
    fn restore(&mut self, s: usize) {
        self.pos = s;
    }
}

impl OuterP {
    /// The current token by reference; the saturating EOF behaviour of `peek`
    /// (PROP-2026-0007 E1: reads that only inspect a token no longer clone it).
    fn cur(&self) -> &Token {
        static EOF: Token = Token { kind: TokenKind::Eof, text: String::new(), line: 0, col: 0, width: 0 };
        self.toks.get(self.pos).or_else(|| self.toks.last()).unwrap_or(&EOF)
    }
    fn peek_kind(&self) -> TokenKind {
        self.cur().kind
    }
    fn peek_text(&self) -> &str {
        &self.cur().text
    }
    fn peek_line(&self) -> usize {
        self.cur().line
    }
    fn peek_col(&self) -> usize {
        self.cur().col
    }
    /// `advance` for callers that ignore the token it returns.
    fn bump(&mut self) {
        if self.pos < self.toks.len() {
            self.pos += 1;
        }
    }

    /// A backtracking point: the position AND how many in-place token rewrites
    /// (punctuation splits) existed, so `rewind` can undo the ones made since.
    /// `TokReader::save`/`restore` stay position-only: the value parser never splits.
    fn mark(&self) -> Mark {
        Mark { pos: self.pos, undo: self.undo.len() }
    }
    fn rewind(&mut self, m: Mark) {
        while self.undo.len() > m.undo {
            let (idx, tok) = self.undo.pop().expect("len checked");
            self.toks[idx] = tok;
        }
        self.pos = m.pos;
    }

    /// Port of `parseProgram`.
    fn parse_program(&mut self, in_block: bool, closer_name: &str) -> Result<Block, CapyError> {
        let mut stmts: Vec<FuncCall> = Vec::new();
        let mut errors: Vec<ErrorNode> = Vec::new();
        // `stray_indents` counts INDENT tokens that appeared mid-body without a
        // block-opener directive in front of them — i.e. the user nested content
        // deeper than the block's anchor purely for visual styling. Each stray
        // INDENT must be paired with a matching DEDENT before the real
        // body-closing DEDENT is reached. If the matching DEDENT is immediately
        // followed by the body's closer keyword, that closer is also part of the
        // cosmetic nesting and is consumed as a no-op.
        let mut stray_indents = 0usize;
        loop {
            let k = self.peek_kind();
            if k == TokenKind::Eof {
                break;
            }
            if k == TokenKind::Newline {
                self.bump();
                continue;
            }
            if k == TokenKind::Indent {
                // A stray indent — content deeper than the surrounding block
                // without a real opener. Treat as a no-op so user-side
                // indentation is purely cosmetic.
                self.bump();
                stray_indents += 1;
                continue;
            }
            if k == TokenKind::Dedent {
                if stray_indents > 0 {
                    self.bump();
                    stray_indents -= 1;
                    // If the user mirrored their stray INDENT with a stray `end`
                    // keyword, consume it (and its newline) so the real block
                    // closer is reached.
                    if in_block && self.at_closer(closer_name) {
                        if let Some(cp) = self.cands.by_name.get(closer_name).cloned() {
                            let _ = self.try_match(&cp);
                        }
                        self.consume_newlines();
                    }
                    continue;
                }
                break;
            }
            if in_block && self.at_closer(closer_name) {
                break;
            }
            // R20 — with recovery on, a failed statement becomes a diagnostic
            // plus an error node and parsing continues. Without it the error
            // propagates exactly as before, so `Library::run` is unchanged (R24).
            let before = self.pos;
            match self.parse_stmt() {
                Ok(st) => stmts.push(st),
                Err(e) if self.recover => {
                    let report = self.should_report(before);
                    // Reset the furthest record per statement, or a later, worse
                    // guess would stick to every subsequent failure.
                    self.furthest = Furthest::default();
                    let end = self.resync();
                    if end == before {
                        // Nothing consumed — refuse to spin.
                        self.pos = before + 1;
                    }
                    if report {
                        let span = self.span_between(before, self.pos.max(before + 1));
                        let code = std::mem::replace(&mut self.fail_code, codes::NO_MATCH);
                        let d = Diagnostic::error(code, span, e.msg.clone());
                        self.diagnostics.push(d);
                        self.last_reported = Some(before);
                        errors.push(ErrorNode {
                            span,
                            tokens: self.toks[before..self.pos.min(self.toks.len())].to_vec(),
                            diagnostic_index: self.diagnostics.len() - 1,
                            ..Default::default()
                        });
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Block { stmts, errors, ..Default::default() })
    }

    /// Port of `atCloser`.
    fn at_closer(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let cp = match self.cands.by_name.get(name).cloned() {
            None => return false,
            Some(c) => c,
        };
        let saved = self.mark();
        let ok = self.try_match(&cp).is_ok();
        self.rewind(saved);
        ok
    }

    /// Port of `parseStmt`.
    fn parse_stmt(&mut self) -> Result<FuncCall, CapyError> {
        // A bound error from an earlier, finished statement must not colour this one.
        self.depth_err = None;
        self.fail_code = codes::NO_MATCH;
        let start_tok = self.peek();
        // PLAN-2026-0001 R2 — index of the statement's first token, so the span
        // can be extended over the block body and closer once they are parsed.
        let stmt_start = self.pos;
        let start_line = start_tok.line;
        let start_col = start_tok.col;
        // `block_err` remembers the FIRST error from a candidate that matched its
        // header and opened a block but then failed to parse its body. We
        // backtrack out of such failures so a flat function can still match the
        // same leading keyword. But if NO later candidate matches either, this
        // remembered error is more informative than the generic "no library
        // function matches" — so it is surfaced at the end.
        let mut block_err: Option<CapyError> = None;
        // Index-walk with a per-candidate Arc bump instead of cloning the whole
        // candidate vector on every statement.
        // E4b/E4c: only candidates whose first element could match this token are tried, still in
        // the original precedence order (two ascending lists, merged by index).
        let cands = self.cands.clone();
        let (tk, bucket) = {
            let t = self.cur();
            (t.kind, cands.bucket(t.kind, &t.text))
        };
        let wild = cands.wild(tk);
        let (mut bi, mut wi) = (0usize, 0usize);
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
            let f = &cands.fns[ci].clone();
            let saved = self.mark();
            let mut inst = match self.try_match(f) {
                Err(_) => {
                    self.rewind(saved);
                    continue;
                }
                Ok(i) => i,
            };
            // Stamp the statement's source position so templates can read it via
            // the `line` / `col` render locals.
            inst.line = start_line;
            inst.col = start_col;
            // R27 — comments that led this statement. The statement's own span
            // deliberately EXCLUDES them: a formatter needs the node's range
            // without its comments, and the comments' ranges separately.
            if let Some(c) = self.leading.get(&stmt_start) {
                inst.leading_comments = c.clone();
            }

            // Delimiter-mode block: opener is followed directly by `open` (no
            // newline). Named-closer block: opener is followed by NEWLINE then
            // INDENT. Non-block: opener is followed by NEWLINE.
            if let Some(block) = &f.block {
                if !block.open.is_empty() {
                    // Expect the open delimiter on the same line.
                    if !self.matches_delim(&block.open) {
                        self.rewind(saved);
                        continue;
                    }
                    let body = match self.parse_delim_block(&block.open, &block.close) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok(b) => b,
                    };
                    inst.body = Some(Box::new(body));
                    // Expect NEWLINE after the closing delimiter.
                    if self.peek_kind() != TokenKind::Newline
                        && self.peek_kind() != TokenKind::Eof
                    {
                        return Err(CapyError::msg(format!(
                            "line {}: expected newline after block close",
                            self.peek_line()
                        )));
                    }
                    self.consume_newlines();
                    return Ok(inst);
                }

                // Sequence-closed block: opener is followed directly by the body
                // (no required newline), terminated by the multi-token close_seq.
                // This is the angle-bracket HTML model — `<p>…</p>`.
                if !block.close_seq.is_empty() {
                    // Resolve the closing sequence: literal segments contribute
                    // their pre-tokenized tokens; a ref segment substitutes the
                    // source text of the opener-bound capture as a single token.
                    // This is what makes the closer DEPEND ON the opener.
                    let seq = match resolve_close_seq(&block.close_seq, &inst.captures) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok(s) => s,
                    };
                    let body = match self.parse_seq_block(&seq, start_line) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok(b) => b,
                    };
                    inst.body = Some(Box::new(body));
                    self.consume_newlines();
                    return Ok(inst);
                }
            }

            // Inside a sequence-closed block, statements are not newline-delimited
            // (`<p>"a"<b>"b"</b></p>`): a flat function may be followed
            // immediately by the next statement's tokens or the block's closing
            // sequence. Outside such a block, require the usual NEWLINE/EOF
            // terminator so partial matches are rejected.
            if self.seq_depth == 0
                && self.peek_kind() != TokenKind::Newline
                && self.peek_kind() != TokenKind::Eof
            {
                self.rewind(saved);
                continue;
            }
            self.consume_newlines();

            // Lookahead: gate the candidate on whether an indented block follows.
            // Lets a flat keyword (when_not_followed_by indent) and a block
            // keyword (when_followed_by indent) share the same literal and
            // disambiguate purely by position.
            if let Some(la) = &f.lookahead {
                let is_indent = self.peek_kind() == TokenKind::Indent;
                if (la.require_indent && !is_indent) || (la.forbid_indent && is_indent) {
                    self.rewind(saved);
                    continue;
                }
            }

            if let Some(block) = &f.block {
                if block.is_verbatim {
                    match self.parse_verbatim_body(&block.closer, start_line) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok((text, closer)) => {
                            // Stash the raw body bytes on the FuncCall via a
                            // synthetic block whose verbatim_text field carries
                            // the text. The renderer surfaces this via `${body}`
                            // exactly like a parsed block body.
                            inst.body = Some(Box::new(Block {
                                stmts: Vec::new(),
                                is_verbatim: true,
                                verbatim_text: text,
                                ..Default::default()
                            }));
                            inst.closer = closer.map(Box::new);
                        }
                    }
                } else if !block.sections.is_empty() {
                    match self.parse_sectioned_body(&block.sections, &block.closer) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok((body, sections, closer)) => {
                            inst.body = body.map(Box::new);
                            inst.sections = sections;
                            inst.closer = closer.map(Box::new);
                        }
                    }
                } else {
                    match self.parse_block_body(&block.closer) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.rewind(saved);
                            continue;
                        }
                        Ok((body, closer)) => {
                            inst.body = Some(Box::new(body));
                            inst.closer = closer.map(Box::new);
                        }
                    }
                }
            }
            // R2 — the span now covers the WHOLE statement: opener, body and
            // closer. try_match could only see as far as the opener's own
            // elements, because the body had not been parsed yet.
            inst.span = self.span_between(stmt_start, self.pos);
            return Ok(inst);
        }

        // A candidate matched a block opener but its body failed to parse, and
        // nothing else matched either — surface that error (it points at the real
        // problem inside the body, not the generic "no match").
        if let Some(e) = block_err {
            return Err(e);
        }
        // PLAN-2026-0004 — if the statement died against the nesting bound, say
        // so (E0003) instead of whatever shape-level expectation the rewind left.
        if let Some(e) = self.depth_err.take() {
            self.fail_code = codes::NESTING_TOO_DEEP;
            return Err(e);
        }
        // PLAN-2026-0002 R14 — if some shape got PAST the first token, report
        // where it stopped and what it wanted, instead of the statement's first
        // token. That is the difference between "no function matched" and
        // "expected `)` to close the parameter list".
        if self.furthest.index > stmt_start && !self.furthest.lazy.is_empty() {
            return Err(self.furthest_error());
        }

        // Nothing got anywhere: the first token itself is unrecognised. Build a
        // "did you mean…?" hint from the closest literal-starting function name.
        let mut err = CapyError::new(
            start_line,
            start_col,
            format!("no library function matches token {}", gofmt::quote(&start_tok.text)),
        );
        let literals: Vec<String> = self
            .cands
            .fns
            .iter()
            .filter(|f| !f.elements.is_empty() && !f.elements[0].is_capture)
            .map(|f| f.elements[0].literal.clone())
            .collect();
        if let Some(best) = suggest_closest(&start_tok.text, &literals, 2) {
            err.hint = format!("did you mean {}?", gofmt::quote(&best));
        } else if !literals.is_empty() {
            // Show what IS valid as a fallback hint.
            let shown = if literals.len() > 6 { &literals[..6] } else { &literals[..] };
            err.hint =
                format!("library functions start with one of: {}", shown.join(", "));
        }
        Err(err)
    }

    /// Port of `matchesDelim` — peeks-and-consumes a single-token delimiter.
    fn matches_delim(&mut self, d: &str) -> bool {
        if self.peek_text() == d {
            self.bump();
            return true;
        }
        false
    }

    /// Port of `parseDelimBlock`.
    ///
    /// Parses statements until `close` is reached, then consumes it. Newlines
    /// inside are statement boundaries.
    fn parse_delim_block(&mut self, open: &str, close: &str) -> Result<Block, CapyError> {
        self.consume_newlines();
        let mut stmts: Vec<FuncCall> = Vec::new();
        loop {
            if self.peek_text() == close {
                self.bump();
                return Ok(Block { stmts, ..Default::default() });
            }
            if self.peek_kind() == TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "unexpected EOF inside {}...{} block",
                    gofmt::quote(open),
                    gofmt::quote(close)
                )));
            }
            if self.peek_kind() == TokenKind::Newline {
                self.bump();
                continue;
            }
            stmts.push(self.parse_stmt()?);
        }
    }

    /// Port of `parseSeqBlock`.
    ///
    /// Parses a free-flowing sequence of statements until the exact token sequence
    /// `seq` is reached, then consumes it. NEWLINE, INDENT and DEDENT between
    /// statements are insignificant (HTML-style: the structure comes from the
    /// tags, not the indentation). Because each block carries its own closing
    /// sequence, a stray closer for a DIFFERENT tag fails to match here and
    /// surfaces as a parse error on the next statement — that is the
    /// mismatched-nesting detection HTML relies on.
    fn parse_seq_block(&mut self, seq: &[String], opener_line: usize) -> Result<Block, CapyError> {
        self.seq_depth += 1;
        let result = self.parse_seq_block_inner(seq, opener_line);
        self.seq_depth -= 1;
        result
    }

    fn parse_seq_block_inner(
        &mut self,
        seq: &[String],
        opener_line: usize,
    ) -> Result<Block, CapyError> {
        let closer = seq.join("");
        let mut stmts: Vec<FuncCall> = Vec::new();
        loop {
            if self.try_consume_seq(seq) {
                return Ok(Block { stmts, ..Default::default() });
            }
            let k = self.peek_kind();
            if k == TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "line {}: unexpected end of input: expected closing {}",
                    opener_line,
                    gofmt::quote(&closer)
                )));
            }
            if k == TokenKind::Newline || k == TokenKind::Indent || k == TokenKind::Dedent {
                self.bump();
                continue;
            }
            stmts.push(self.parse_stmt()?);
        }
    }

    /// Port of `tryConsumeSeq`.
    ///
    /// Matches the multi-token closing sequence `seq` (e.g. `["</","div",">"]`)
    /// against the upcoming tokens and, on a match, consumes exactly those tokens.
    ///
    /// Matching is token-based and whitespace-tolerant (mirroring how the opener's
    /// literals match), so `</p>` and `</p >` both close a `<p>`. The one special
    /// case is the FINAL element: the lexer greedily packs runs of punctuation
    /// into one token, so `</p></div>` lexes as `</ p ></ div >` — the closing `>`
    /// of `</p>` and the opening `</` of `</div>` share a single `></` token. When
    /// the last element is a strict prefix of such a merged punctuation run, it is
    /// matched and the leftover suffix is written back as a new token (with
    /// adjusted col/width) so it can open the next tag. (A mid-sequence merge
    /// cannot occur: a tag name is an identifier, which always breaks the
    /// punctuation run.)
    fn try_consume_seq(&mut self, seq: &[String]) -> bool {
        let mut i = self.pos;
        let mut split_idx: i64 = -1;
        let mut split_tok: Option<Token> = None;
        for (n, el) in seq.iter().enumerate() {
            if i >= self.toks.len() {
                return false;
            }
            let tok = self.toks[i].clone();
            match tok.kind {
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => {
                    return false
                }
                _ => {}
            }
            if &tok.text == el {
                i += 1;
                continue;
            }
            if tok.kind == TokenKind::Punct
                && n == seq.len() - 1
                && tok.text.len() > el.len()
                && tok.text.starts_with(el.as_str())
            {
                let mut rem = tok.clone();
                rem.text = tok.text[el.len()..].to_string();
                rem.col = tok.col + el.len();
                rem.width = rem.text.len();
                split_idx = i as i64;
                split_tok = Some(rem);
                break;
            }
            return false;
        }
        if split_idx >= 0 {
            let idx = split_idx as usize;
            self.undo.push((idx, self.toks[idx].clone()));
            self.toks[idx] = split_tok.unwrap();
            self.pos = idx;
            return true;
        }
        self.pos = i;
        true
    }

    fn consume_newlines(&mut self) {
        while self.peek_kind() == TokenKind::Newline {
            self.bump();
        }
    }

    /// PLAN-2026-0002 R19 — suppression window and cap.
    ///
    /// Configurable rather than contractual: these are tuning decisions, and the
    /// proposal deliberately left the values open.
    const SUPPRESS_WITHIN: usize = 3;
    const MAX_DIAGNOSTICS: usize = 20;

    /// R19 — should this failure be reported, or is it cascade noise from the one
    /// just reported?
    fn should_report(&self, at: usize) -> bool {
        if self.diagnostics.len() >= Self::MAX_DIAGNOSTICS {
            return false;
        }
        match self.last_reported {
            Some(prev) => at.saturating_sub(prev) > Self::SUPPRESS_WITHIN,
            None => true,
        }
    }

    /// R21 — skip to a point where parsing can sensibly resume.
    ///
    /// Priority order matters, and rule 1 is the one that stops a single missing
    /// `)` from swallowing the rest of the file:
    ///   1. delimiter balance — never resync while inside an unclosed bracket
    ///   2. statement boundary — a NEWLINE at depth 0, or a token that begins
    ///      some known shape (the shape table makes that set exact)
    ///   3. block boundary — DEDENT
    ///   4. EOF
    fn resync(&mut self) -> usize {
        let start = self.pos;
        let mut depth: i64 = 0;
        // Newlines crossed while still unbalanced. A delimiter that is never
        // closed would otherwise hold `depth` above zero to EOF and the scan
        // would swallow the whole file — the precise failure rule 1 exists to
        // prevent. After a line break we therefore stop trusting the balance and
        // accept a statement-start token instead.
        let mut unbalanced_newlines = 0usize;
        while self.pos < self.toks.len() {
            let t = &self.toks[self.pos];
            match t.kind {
                TokenKind::Eof => break,
                TokenKind::LParen | TokenKind::LBrack | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBrack | TokenKind::RBrace => {
                    depth -= 1;
                    // Closing back past where we started is itself a resync point.
                    if depth < 0 {
                        self.pos += 1;
                        break;
                    }
                }
                TokenKind::Newline => {
                    if depth <= 0 {
                        self.pos += 1;
                        break;
                    }
                    unbalanced_newlines += 1;
                }
                TokenKind::Dedent if depth <= 0 => break,
                TokenKind::Ident if self.pos > start => {
                    // Rule 2, second clause: the set of tokens that can BEGIN a
                    // statement is known exactly from the shape table, so this is
                    // precise rather than heuristic.
                    //
                    // Accepted at depth 0, or once a line break has happened
                    // inside an unclosed delimiter — at that point the delimiter
                    // is far more likely to be missing than to be spanning lines
                    // deliberately, and the alternative is losing the file.
                    if (depth <= 0 || unbalanced_newlines > 0) && self.starts_a_shape(&t.text) {
                        return self.pos;
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        self.pos
    }

    /// Does `text` begin some declared shape?
    fn starts_a_shape(&self, text: &str) -> bool {
        self.cands.fns.iter().any(|f| {
            f.elements.first().map(|e| !e.is_capture && e.literal == text).unwrap_or(false)
        })
    }

    /// PLAN-2026-0002 R14 — turn the furthest record into an error.
    ///
    /// Expectations are unioned, so when several shapes died at the same token
    /// the reader sees every alternative rather than whichever was tried last.
    fn furthest_error(&self) -> CapyError {
        let tok = match self.toks.get(self.furthest.index).or_else(|| self.toks.last()) {
            Some(t) => t.clone(),
            None => return CapyError::msg("no library function matches"),
        };
        let expected = self.furthest.expected_set();
        let wanted: Vec<String> = expected.iter().map(|e| e.describe()).collect();
        let list = match wanted.len() {
            1 => wanted[0].clone(),
            2 => format!("{} or {}", wanted[0], wanted[1]),
            _ => format!("{}, or {}", wanted[..wanted.len() - 1].join(", "), wanted[wanted.len() - 1]),
        };
        let found = if tok.kind == TokenKind::Newline || tok.kind == TokenKind::Eof {
            "end of statement".to_string()
        } else {
            gofmt::quote(&tok.text)
        };
        let mut d = Diagnostic::error(
            codes::NO_MATCH,
            Span::new(tok.line, tok.col, tok.line, tok.col + tok.width.max(1)),
            format!("expected {list}, found {found}"),
        )
        .with_context(self.furthest.context_frames());

        // R17 — if what was wanted is a closing delimiter, label where it opened.
        for e in &expected {
            if let Expectation::CloseDelim { close, open_line, open_col } = e {
                if *open_line > 0 {
                    d = d.with_label(crate::domain::errors::Label::new(
                        Span::new(*open_line, *open_col, *open_line, open_col + 1),
                        format!("unclosed `{}` opened here", opening_of(close)),
                    ));
                }
            }
            // R16 — reuse the existing did-you-mean machinery for `options`.
            if let Expectation::OneOf(opts) = e {
                if let Some(best) = suggest_closest(&tok.text, opts, 2) {
                    d = d.with_help(format!("did you mean {}?", gofmt::quote(&best)));
                }
            }
        }
        d.to_capy_error()
    }

    /// PLAN-2026-0002 R14 — record a rejection, applying the replace/union/discard
    /// rule. Further replaces; equal-distance unions (which is what produces
    /// "expected an expression, `(`, or an identifier"); nearer is discarded.
    fn note_failure(&mut self, expected: Expectation) {
        self.note_lazy(LazyExp::Owned(expected));
    }

    /// The common rejection: shape `f` wanted its literal at element `elem` and
    /// the input did not have it. Records references, allocates nothing.
    fn note_literal(&mut self, f: &Arc<FuncDef>, elem: usize) {
        self.note_lazy(LazyExp::Lit { func: f.clone(), elem });
    }

    fn note_lazy(&mut self, le: LazyExp) {
        let at = self.pos;
        match at.cmp(&self.furthest.index) {
            std::cmp::Ordering::Greater => {
                self.furthest.index = at;
                self.furthest.lazy.clear();
                self.furthest.lazy.push(le);
                self.furthest.ctx = self.ctx_stack.clone();
            }
            std::cmp::Ordering::Equal => {
                self.furthest.lazy.push(le);
                if self.furthest.ctx.is_empty() {
                    self.furthest.ctx = self.ctx_stack.clone();
                }
            }
            std::cmp::Ordering::Less => {}
        }
    }

    /// PLAN-2026-0001 R1–R4 — the span covering `toks[from .. to)`.
    ///
    /// `Token` carries `line`, `col` and `width`; the end column is
    /// `col + width`, exclusive. `width` is documented as "zero means unset,
    /// fall back to `text.len()`", so that fallback is applied here rather than
    /// at every call site. Structural tokens (NEWLINE, INDENT, DEDENT, EOF) have
    /// `col == 0` and are skipped — they are not source the user wrote, and
    /// including them would drag a span back to column 0.
    fn span_between(&self, from: usize, to: usize) -> Span {
        let mut first: Option<&Token> = None;
        let mut last: Option<&Token> = None;
        for t in self.toks.iter().take(to.min(self.toks.len())).skip(from) {
            if t.col == 0 {
                continue;
            }
            if first.is_none() {
                first = Some(t);
            }
            last = Some(t);
        }
        match (first, last) {
            (Some(f), Some(l)) => {
                let width = if l.width > 0 { l.width } else { l.text.len() };
                Span::new(f.line, f.col, l.line, l.col + width)
            }
            // Nothing consumed — an optional capture that bound its default, or
            // a zero-width match. Leave it unset rather than inventing a range.
            _ => Span::default(),
        }
    }

    /// Port of `tryMatch`.
    fn try_match(&mut self, f: &Arc<FuncDef>) -> Result<FuncCall, CapyError> {
        // PLAN-2026-0001 R2/R4 — where this shape's match begins. Previously the
        // FuncCall built at the end of this function was stamped
        // `line: 0, col: 0`, which is what made every nested node unlocatable.
        let match_start = self.pos;
        // R26 — everything rejected below happens "in <shape>", and for a capture
        // "in argument <name> of <shape>".
        self.ctx_stack.push(LiteFrame { func: f.clone(), arg_index: 0 });
        let out = self.try_match_inner(f, match_start);
        self.ctx_stack.pop();
        out
    }

    fn try_match_inner(&mut self, f: &Arc<FuncDef>, match_start: usize) -> Result<FuncCall, CapyError> {
        let mut caps: BTreeMap<String, CaptureValue> = BTreeMap::new();
        for i in 0..f.elements.len() {
            if let Some(top) = self.ctx_stack.last_mut() {
                top.arg_index = i;
            }
            // PROP-2026-0007 E3: borrow the element; `f` is not borrowed from `self`.
            let el = &f.elements[i];
            // Auto-skip an optional comma between consecutive captures.
            if i > 0
                && el.is_capture
                && f.elements[i - 1].is_capture
                && self.peek_kind() == TokenKind::Punct
                && self.peek_text() == ","
            {
                self.bump();
            }
            if !el.is_capture {
                if !self.match_literal(&el.literal) {
                    // R14/R15 — record what this shape wanted here before
                    // backtracking discards the attempt.
                    self.note_literal(f, i);
                    // Every caller of `try_match` discards the error text, so it is not formatted.
                    return Err(CapyError::msg(""));
                }
                continue;
            }
            // Function-as-type capture (named nonterminal): the capture's type
            // names another library function. Match that function's shape —
            // possibly repeated (`type*` / `type+`) with an optional separator
            // literal — and store the matched sub-FuncCall(s).
            if el.is_func {
                let cap_start = self.pos;
                let mut val = self.capture_func_type(el)?;
                val.span = self.span_between(cap_start, self.pos);
                caps.insert(el.name.clone(), val);
                continue;
            }
            // Optional capture: if the statement has ended (no value to consume),
            // bind the declared default and fill any remaining optional captures
            // with theirs. Optional args are validated to be trailing, so once an
            // end-of-statement boundary is hit here every remaining element is an
            // optional capture.
            if el.optional && self.at_statement_end() {
                for rest in &f.elements[i..] {
                    if rest.is_capture {
                        caps.insert(
                            rest.name.clone(),
                            default_capture(&rest.cap_type, &rest.default),
                        );
                    }
                }
                break;
            }
            // `stop` is read only by the generic expression path and by custom types
            // that delegate to it; these built-ins return before touching it.
            let stop = if matches!(el.cap_type.as_str(), "ident" | "raw" | "tail" | "word" | "dotted_ident") {
                Vec::new()
            } else {
                next_literals(&f.elements[i + 1..])
            };
            let cap_start = self.pos;
            let mut val = self.capture_value(&el.cap_type, &stop)?;
            val.span = self.span_between(cap_start, self.pos);
            caps.insert(el.name.clone(), val);
        }
        let span = self.span_between(match_start, self.pos);
        Ok(FuncCall {
            func: f.name.clone(),
            captures: caps,
            body: None,
            closer: None,
            sections: BTreeMap::new(),
            // R4: real positions, not the zeros this site used to stamp. R5: the
            // `line`/`col` render locals keep their meaning — they are the
            // span's start.
            line: span.start_line,
            col: span.start_col,
            span,
            leading_comments: Vec::new(),
        })
    }

    /// Port of `captureFuncType`.
    ///
    /// Matches a function-as-type capture (named nonterminal). `el.cap_type` names
    /// a library function; `el.repeat` selects the arity:
    ///
    /// ```text
    /// ""  exactly one occurrence
    /// "+" one or more
    /// "*" zero or more
    /// ```
    ///
    /// `el.sep`, when set, is a separator literal required between successive
    /// occurrences. A zero-progress guard prevents an infinite loop if a
    /// sub-function can match while consuming no tokens.
    fn capture_func_type(&mut self, el: &PatternElement) -> Result<CaptureValue, CapyError> {
        // R0 — bound the descent. Without this, deeply nested input aborts the
        // process rather than returning an error.
        if self.depth >= MAX_PARSE_DEPTH {
            let t = self.peek();
            let err = CapyError::new(
                t.line,
                t.col,
                format!(
                    "nesting too deep (limit {MAX_PARSE_DEPTH}) while matching {} — \
the source nests further than the parser will follow",
                    gofmt::quote(&el.alternatives().collect::<Vec<_>>().join(" | "))
                ),
            );
            // Keep the first (innermost-reached) one for this statement.
            if self.depth_err.is_none() {
                self.depth_err = Some(err.clone());
            }
            return Err(err);
        }
        self.depth += 1;
        let out = self.capture_func_type_inner(el);
        self.depth -= 1;
        out
    }

    fn capture_func_type_inner(&mut self, el: &PatternElement) -> Result<CaptureValue, CapyError> {
        // PROP-2026-0004 — `cap_type` is alternative 1 and `el.alts` the rest, so
        // an ordinary single-type capture has exactly one target and behaves as
        // it always did.
        let mut targets: Vec<Arc<FuncDef>> = Vec::new();
        for n in el.alternatives() {
            match self.cands.by_name.get(n).cloned() {
                None => {
                    return Err(CapyError::msg(format!(
                        "internal: function-typed capture {} references unknown function {}",
                        gofmt::quote(&el.name),
                        gofmt::quote(n)
                    )))
                }
                Some(t) => targets.push(t),
            }
        }
        let names = el.alternatives().collect::<Vec<_>>().join(" | ");

        // Exactly-one (no repetition): a single mandatory match.
        if el.repeat.is_empty() {
            match self.match_alt(&targets) {
                None => {
                    // Note every alternative; the furthest-failure union is what
                    // the diagnostic prints (R6).
                    for t in &targets {
                        self.note_failure(Expectation::Nonterminal(t.name.clone()));
                    }
                    return Err(CapyError::msg(format!("expected {names}")));
                }
                Some(fc) => {
                    return Ok(CaptureValue { sub: vec![fc], ..Default::default() });
                }
            }
        }

        // Repeated: gather as many occurrences as match, separated by `sep`.
        let mut subs: Vec<FuncCall> = Vec::new();
        loop {
            if !subs.is_empty() && !el.sep.is_empty() {
                // A separator is required between occurrences; if it's not
                // present, the repetition is over.
                let sep = self.mark();
                if !self.match_literal(&el.sep) {
                    self.rewind(sep);
                    break;
                }
                match self.match_alt(&targets) {
                    None => {
                        // Separator consumed but no following item — roll back the
                        // separator so it isn't lost.
                        self.rewind(sep);
                        break;
                    }
                    Some(fc) => {
                        subs.push(fc);
                        continue;
                    }
                }
            }
            match self.match_alt(&targets) {
                None => break,
                Some(fc) => subs.push(fc),
            }
        }
        if el.repeat == "+" && subs.is_empty() {
            for t in &targets {
                self.note_failure(Expectation::Nonterminal(t.name.clone()));
            }
            return Err(CapyError::msg(format!("expected at least one {names}")));
        }
        Ok(CaptureValue { sub: subs, ..Default::default() })
    }

    /// PROP-2026-0004 R2 — ordered choice: try each alternative in declaration
    /// order and return the first that matches. Rewind is local to this capture
    /// position (`match_one` restores the token cursor on failure), so a failed
    /// alternative never leaks consumed input into the next one.
    ///
    /// Deliberately notes no expectation itself: a repetition that simply ends
    /// is not a failure, and R7 requires existing diagnostics stay byte-identical.
    /// The mandatory call sites note **every** alternative, which the
    /// furthest-failure record unions for the diagnostic (R6).
    fn match_alt(&mut self, targets: &[Arc<FuncDef>]) -> Option<FuncCall> {
        targets.iter().find_map(|t| self.match_one(t))
    }

    /// The `matchOne` closure from `captureFuncType`.
    fn match_one(&mut self, target: &Arc<FuncDef>) -> Option<FuncCall> {
        let saved = self.mark();
        let start_pos = self.pos;
        match self.try_match(target) {
            Err(_) => {
                self.rewind(saved);
                None
            }
            Ok(mut fc) => {
                if self.pos == start_pos {
                    self.rewind(saved);
                    return None;
                }
                fc.func = target.name.clone();
                Some(fc)
            }
        }
    }

    /// Port of `atStatementEnd`.
    fn at_statement_end(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent
        )
    }

    /// Port of `matchLiteral`.
    fn match_literal(&mut self, lit: &str) -> bool {
        // Decide by reference: a failed attempt (the common case) must not clone the token.
        let (kind, exact, split) = {
            let t = self.cur();
            (
                t.kind,
                t.text == lit,
                t.kind == TokenKind::Punct && t.text.len() > lit.len() && t.text.starts_with(lit),
            )
        };
        match kind {
            TokenKind::Ident
            | TokenKind::Punct
            | TokenKind::LParen
            | TokenKind::RParen
            | TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::LBrack
            | TokenKind::RBrack => {
                if exact {
                    self.bump();
                    return true;
                }
                // The lexer greedily merges runs of punctuation into one token, so
                // an angle-bracket tag boundary like `><` arrives as a single `><`
                // punct token. When a punct literal is a strict prefix of such a
                // merged run, match the prefix and write the suffix back as the
                // next token (adjusted col/width) so it can be matched next.
                if split {
                    let t = self.cur().clone();
                    let mut rem = t.clone();
                    rem.text = t.text[lit.len()..].to_string();
                    rem.col = t.col + lit.len();
                    rem.width = rem.text.len();
                    self.undo.push((self.pos, t));
                    self.toks[self.pos] = rem;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Port of `captureValue`.
    fn capture_value(&mut self, t: &str, stop: &[String]) -> Result<CaptureValue, CapyError> {
        match t {
            "ident" => {
                let tok = self.peek();
                if tok.kind != TokenKind::Ident {
                    return Err(CapyError::msg(format!(
                        "expected identifier, got {}",
                        gofmt::quote(&tok.text)
                    )));
                }
                self.bump();
                return Ok(CaptureValue { text: tok.text, ..Default::default() });
            }
            "raw" => {
                let tok = self.peek();
                if tok.kind == TokenKind::Ident || tok.kind == TokenKind::Str {
                    self.bump();
                    return Ok(CaptureValue { text: tok.text, ..Default::default() });
                }
                return Err(CapyError::msg(format!(
                    "expected raw token, got {}",
                    gofmt::quote(&tok.text)
                )));
            }
            "tail" => {
                // Capture every remaining token on the current statement as a
                // single source-text string, reconstructed using the original
                // column positions so `20px` (no source whitespace) stays joined
                // while `1px solid red` keeps its spaces.
                //
                // Quoted tokens are re-emitted WITH their quotes so a spaced,
                // quoted argument survives as one slot. Spacing is computed from
                // each token's source width (which counts the quotes),
                // independent of how long the re-emitted text is.
                let mut b = String::new();
                let mut prev_end: i64 = -1;
                loop {
                    let k = self.peek_kind();
                    if k == TokenKind::Newline
                        || k == TokenKind::Eof
                        || k == TokenKind::Dedent
                        || k == TokenKind::Indent
                    {
                        break;
                    }
                    let tok = self.advance();
                    if prev_end >= 0 && (tok.col as i64) > prev_end {
                        // Preserve only as many spaces as appeared in source.
                        b.push_str(&" ".repeat((tok.col as i64 - prev_end) as usize));
                    }
                    b.push_str(&token_source_text(&tok));
                    prev_end = (tok.col + token_width(&tok)) as i64;
                }
                if b.is_empty() {
                    return Err(CapyError::msg("expected tail value, got end of line"));
                }
                return Ok(CaptureValue { text: b, ..Default::default() });
            }
            "word" => {
                // A shell-style bare word: a MAXIMAL run of adjacent tokens with
                // no intervening source whitespace, joined into one value. Lets
                // `--oneline`, `-f`, `k8s/deploy.yaml`, `name=^web$`, and
                // hyphenated names like `restart-api` capture as ONE token even
                // though the lexer splits them on `-`, `/`, `=`, `.` etc. Unlike
                // `tail`, it stops at the first whitespace gap.
                let k = self.peek_kind();
                if k == TokenKind::Newline
                    || k == TokenKind::Eof
                    || k == TokenKind::Dedent
                    || k == TokenKind::Indent
                {
                    return Err(CapyError::msg("expected word, got end of statement"));
                }
                let mut wb = String::new();
                let mut prev_end: i64 = -1;
                loop {
                    let tok = self.peek();
                    match tok.kind {
                        TokenKind::Newline
                        | TokenKind::Eof
                        | TokenKind::Dedent
                        | TokenKind::Indent => {
                            return Ok(CaptureValue { text: wb, ..Default::default() });
                        }
                        _ => {}
                    }
                    if prev_end >= 0 && (tok.col as i64) > prev_end {
                        // A whitespace gap in the source ends the word.
                        return Ok(CaptureValue { text: wb, ..Default::default() });
                    }
                    self.bump();
                    wb.push_str(&tok.text);
                    // NOTE: Go uses len(tok.Text) here, not tokenWidth — so a
                    // quoted token's stripped length is used. Preserved.
                    prev_end = (tok.col + tok.text.len()) as i64;
                }
            }
            "dotted_ident" => {
                // A dotted identifier path: IDENT ( "." IDENT )*. The lexer treats
                // `.` as punctuation, so a bare `ident` capture stops at the first
                // segment; this type consumes the whole `err.kind` / `a.b.c` chain
                // and returns it joined with dots. Requires no surrounding
                // whitespace around the dots.
                let tok = self.peek();
                if tok.kind != TokenKind::Ident {
                    return Err(CapyError::msg(format!(
                        "expected dotted identifier, got {}",
                        gofmt::quote(&tok.text)
                    )));
                }
                let mut db = String::new();
                db.push_str(&self.advance().text);
                let mut end = tok.col + tok.text.len();
                while self.peek_kind() == TokenKind::Punct
                    && self.peek_text() == "."
                    && self.peek_col() == end
                {
                    let dot = self.advance();
                    let seg = self.peek();
                    if seg.kind != TokenKind::Ident || seg.col != dot.col + 1 {
                        return Err(CapyError::msg(format!(
                            "expected identifier after {} in dotted identifier",
                            gofmt::quote(&format!("{}.", db))
                        )));
                    }
                    db.push('.');
                    db.push_str(&self.advance().text);
                    end = seg.col + seg.text.len();
                }
                return Ok(CaptureValue { text: db, ..Default::default() });
            }
            _ => {}
        }

        // Library-defined types. Three sub-cases:
        //   * group_open/group_close set → delimited capture
        //   * base set → dispatch to the base type's token-capture rules
        //   * otherwise → behave like `raw`
        if let Some(td) = self.cands.types.get(t).cloned() {
            if !td.group_open.is_empty() {
                return self.capture_group(&td);
            }
            if !td.base.is_empty() && td.base != "any" {
                return self.capture_value(&td.base, stop);
            }
            let tok = self.peek();
            if tok.kind == TokenKind::Ident
                || tok.kind == TokenKind::Str
                || tok.kind == TokenKind::Number
            {
                self.bump();
                return Ok(CaptureValue { text: tok.text, ..Default::default() });
            }
            return Err(CapyError::msg(format!(
                "expected ident, string, or number for type {}, got {}",
                gofmt::quote(t),
                gofmt::quote(&tok.text)
            )));
        }

        // Built-in value kinds: parse a value expression (with comparison tail).
        // BOTH the parsed Expr and the source-text rendering are stored — the
        // latter is what shows up in templates so a `cond:any` in `if x > 0`
        // emits the literal text "x > 0".
        //
        // INC-2026-0002 — if the expression parser hit its nesting bound, remember that error for the
        // statement exactly as `capture_func_type` remembers the capture bound, so the statement is
        // reported as `E0003` instead of a generic "no match" (the grouping rule discards the inner error).
        clear_depth_error();
        let x = match parse_value(self, stop) {
            Ok(x) => x,
            Err(e) => {
                if let Some(de) = take_depth_error() {
                    if self.depth_err.is_none() {
                        self.depth_err = Some(de);
                    }
                }
                return Err(e);
            }
        };
        let text = expr_to_text(&x);
        Ok(CaptureValue { is_expr: true, expr: Some(x), text, sub: Vec::new(), span: Span::default() })
    }

    /// Port of `captureGroup`.
    ///
    /// Walks tokens between a group type's open and close delimiters, joining the
    /// in-between source text with column-preserved spacing (same algorithm as
    /// `tail`). Balanced nesting: each occurrence of `group_open` increments
    /// depth, each `group_close` decrements; the capture only terminates when
    /// depth returns to zero. Supports multi-line groups.
    fn capture_group(&mut self, td: &TypeDef) -> Result<CaptureValue, CapyError> {
        if !self.consume_literal(&td.group_open) {
            return Err(CapyError::msg(format!(
                "expected group open {}, got {}",
                gofmt::quote(&td.group_open),
                gofmt::quote(self.peek_text())
            )));
        }
        let mut b = String::new();
        let mut depth = 1i64;
        let mut prev_end: i64 = -1;
        let mut prev_line: i64 = -1;
        loop {
            let tok = self.peek();
            match tok.kind {
                TokenKind::Eof => {
                    return Err(CapyError::msg(format!(
                        "unterminated group: expected {} before end of input",
                        gofmt::quote(&td.group_close)
                    )));
                }
                TokenKind::Newline => {
                    // Multi-line groups: keep newlines as literal newline
                    // characters in the captured text. Reset intra-line state.
                    b.push('\n');
                    self.bump();
                    prev_end = -1;
                    continue;
                }
                TokenKind::Indent | TokenKind::Dedent => {
                    // These structural tokens contribute no characters to the
                    // captured text; their effect is encoded in the next token's
                    // col on the new line.
                    self.bump();
                    continue;
                }
                _ => {}
            }
            if tok.text == td.group_close {
                depth -= 1;
                if depth == 0 {
                    self.bump();
                    return Ok(CaptureValue { text: b, ..Default::default() });
                }
                // A nested close: include it in the captured text.
            } else if tok.text == td.group_open {
                depth += 1;
            }
            // Inter-token spacing: if still on the same line as the previous
            // token, pad to the source column. After a newline `prev_end` was
            // reset so we start fresh at column 0.
            if prev_line == tok.line as i64 && prev_end >= 0 && (tok.col as i64) > prev_end {
                b.push_str(&" ".repeat((tok.col as i64 - prev_end) as usize));
            } else if prev_line != tok.line as i64 && tok.col > 1 && !b.is_empty() {
                // Cross-line: leading indent on a continuation line.
                b.push_str(&" ".repeat(tok.col - 1));
            }
            let text = token_source_form(&tok);
            b.push_str(&text);
            prev_end = (tok.col + text.len()) as i64;
            prev_line = tok.line as i64;
            self.bump();
        }
    }

    /// Port of `consumeLiteral`.
    fn consume_literal(&mut self, lit: &str) -> bool {
        let t = self.peek();
        match t.kind {
            TokenKind::Ident
            | TokenKind::Punct
            | TokenKind::LParen
            | TokenKind::RParen
            | TokenKind::LBrace
            | TokenKind::RBrace
            | TokenKind::LBrack
            | TokenKind::RBrack => {
                if t.text == lit {
                    self.bump();
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Port of `parseVerbatimBody`.
    ///
    /// Captures the body of a `block_verbatim`-declared function as raw source
    /// text. INDENT-balanced nested blocks are counted but the body is NEVER
    /// re-parsed against the library's functions.
    fn parse_verbatim_body(
        &mut self,
        closer_name: &str,
        opener_line: usize,
    ) -> Result<(String, Option<FuncCall>), CapyError> {
        if self.peek_kind() != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented block (verbatim, closer={})",
                self.peek_line(),
                closer_name
            )));
        }
        self.bump();

        // Walk tokens to advance past the body and find the closing DEDENT's
        // line. Track INDENT/DEDENT balance so nested indentation doesn't end the
        // block early. The text is NOT reconstructed from these tokens — the body
        // is sliced from the raw source so blank lines and comment-marker lines,
        // which produce no tokens, are preserved byte-for-byte.
        let mut depth = 0i64;
        // Go initialises this to -1 defensively; every path that leaves the loop
        // either returns or assigns it, so a definite-assignment binding is
        // equivalent and keeps the dead store out.
        let closer_line: i64;
        loop {
            match self.peek_kind() {
                TokenKind::Eof => {
                    return Err(CapyError::msg(format!(
                        "line {}: unterminated verbatim block, expected closer {}",
                        self.peek_line(),
                        gofmt::quote(closer_name)
                    )));
                }
                TokenKind::Indent => {
                    depth += 1;
                    self.bump();
                    continue;
                }
                TokenKind::Dedent => {
                    if depth == 0 {
                        closer_line = self.peek_line() as i64;
                        self.bump();
                        break;
                    }
                    depth -= 1;
                    self.bump();
                    continue;
                }
                _ => {}
            }
            self.bump();
        }

        let text = self.verbatim_slice(opener_line as i64, closer_line);

        let cp = match self.cands.by_name.get(closer_name).cloned() {
            None => {
                return Err(CapyError::msg(format!(
                    "library function {} (verbatim closer) not found",
                    gofmt::quote(closer_name)
                )))
            }
            Some(c) => c,
        };
        let saved = self.mark();
        let closer_inst = match self.try_match(&cp) {
            Err(_) => {
                self.rewind(saved);
                return Err(CapyError::msg(format!(
                    "line {}: expected closer {}",
                    self.peek_line(),
                    gofmt::quote(closer_name)
                )));
            }
            Ok(c) => c,
        };
        if self.peek_kind() != TokenKind::Newline && self.peek_kind() != TokenKind::Eof {
            return Err(CapyError::msg(format!(
                "line {}: closer must end the line",
                self.peek_line()
            )));
        }
        self.consume_newlines();
        Ok((text, Some(closer_inst)))
    }

    /// Port of `verbatimSlice`.
    ///
    /// Returns the raw source lines strictly between the opener line and the
    /// closer line (both 1-indexed), dedented by the smallest leading-whitespace
    /// run across non-blank body lines so the captured text starts flush-left.
    /// Because it reads the original source — not the token stream — blank lines
    /// and comment-marker lines are preserved exactly.
    fn verbatim_slice(&self, opener_line: i64, closer_line: i64) -> String {
        let n = self.src_lines.len() as i64;
        let mut closer_line = closer_line;
        if closer_line <= 0 || closer_line > n + 1 {
            // Defensive: a trailing-EOF dedent has no line; treat the rest of the
            // source as the body.
            closer_line = n + 1;
        }
        // Body = 1-indexed lines (opener_line+1 .. closer_line-1)
        //      = 0-indexed slice [opener_line : closer_line-1].
        let mut start = opener_line;
        let mut end = closer_line - 1;
        if start < 0 {
            start = 0;
        }
        if end > n {
            end = n;
        }
        if start >= end {
            return String::new();
        }
        let body = &self.src_lines[start as usize..end as usize];

        let mut min_indent: i64 = -1;
        for ln in body {
            if ln.trim().is_empty() {
                continue;
            }
            let lb = ln.as_bytes();
            let mut i = 0usize;
            while i < lb.len() && (lb[i] == b' ' || lb[i] == b'\t') {
                i += 1;
            }
            if min_indent < 0 || (i as i64) < min_indent {
                min_indent = i as i64;
            }
        }
        if min_indent < 0 {
            min_indent = 0;
        }
        let mi = min_indent as usize;
        let out: Vec<String> = body
            .iter()
            .map(|ln| if ln.len() >= mi { ln[mi..].to_string() } else { ln.clone() })
            .collect();
        out.join("\n") + "\n"
    }

    /// Port of `parseBlockBody`.
    fn parse_block_body(
        &mut self,
        closer_name: &str,
    ) -> Result<(Block, Option<FuncCall>), CapyError> {
        if self.peek_kind() != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented block (closer={})",
                self.peek_line(),
                closer_name
            )));
        }
        self.bump();

        let body = self.parse_program(true, closer_name)?;
        if self.peek_kind() != TokenKind::Dedent {
            return Err(CapyError::msg(format!(
                "line {}: expected end of block",
                self.peek_line()
            )));
        }
        self.bump();
        // Dedent-only block: no closer keyword to match. Useful for CSS-style
        // selectors and other DSLs where a body is delimited purely by
        // indentation.
        if closer_name.is_empty() {
            return Ok((body, None));
        }
        let cp = match self.cands.by_name.get(closer_name).cloned() {
            None => {
                return Err(CapyError::msg(format!(
                    "library function {} (closer) not found",
                    gofmt::quote(closer_name)
                )))
            }
            Some(c) => c,
        };
        let saved = self.mark();
        let closer_inst = match self.try_match(&cp) {
            Err(_) => {
                self.rewind(saved);
                return Err(CapyError::msg(format!(
                    "line {}: expected closer {}",
                    self.peek_line(),
                    gofmt::quote(closer_name)
                )));
            }
            Ok(c) => c,
        };
        if self.peek_kind() != TokenKind::Newline && self.peek_kind() != TokenKind::Eof {
            return Err(CapyError::msg(format!(
                "line {}: closer must end the line",
                self.peek_line()
            )));
        }
        self.consume_newlines();
        Ok((body, Some(closer_inst)))
    }

    /// Port of `parseIndentedRegion`.
    ///
    /// Consumes one INDENT…DEDENT region as a nested program. The caller must have
    /// already verified the next token is an INDENT.
    fn parse_indented_region(&mut self) -> Result<Block, CapyError> {
        if self.peek_kind() != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented body",
                self.peek_line()
            )));
        }
        self.bump();
        let body = self.parse_program(true, "")?;
        if self.peek_kind() != TokenKind::Dedent {
            return Err(CapyError::msg(format!(
                "line {}: expected end of indented body",
                self.peek_line()
            )));
        }
        self.bump();
        Ok(body)
    }

    /// Port of `parseSectionedBody`.
    ///
    /// Parses a multi-section block:
    ///
    /// ```text
    /// try
    ///     <main body>
    /// rescue
    ///     <rescue body>
    /// finally
    ///     <finally body>
    /// end
    /// ```
    ///
    /// The opener's main body (if any) comes first as an indented region, then
    /// zero or more interior section keywords (each at the opener's indent, each
    /// introducing its own indented sub-body), then the closer keyword. Sections
    /// may appear in any order and any subset; each may appear at most once.
    #[allow(clippy::type_complexity)]
    fn parse_sectioned_body(
        &mut self,
        sections: &[String],
        closer_name: &str,
    ) -> Result<(Option<Block>, BTreeMap<String, Block>, Option<FuncCall>), CapyError> {
        let is_section: Vec<&String> = sections.iter().collect();

        let mut main_body: Option<Block> = None;
        if self.peek_kind() == TokenKind::Indent {
            main_body = Some(self.parse_indented_region()?);
        }

        let mut sec_bodies: BTreeMap<String, Block> = BTreeMap::new();
        loop {
            let tok = self.peek();
            if tok.kind == TokenKind::Ident && is_section.iter().any(|s| **s == tok.text) {
                let name = tok.text.clone();
                if sec_bodies.contains_key(&name) {
                    return Err(CapyError::msg(format!(
                        "line {}: duplicate section {}",
                        tok.line,
                        gofmt::quote(&name)
                    )));
                }
                self.bump();
                if self.peek_kind() != TokenKind::Newline {
                    return Err(CapyError::msg(format!(
                        "line {}: section {} must be alone on its line",
                        tok.line,
                        gofmt::quote(&name)
                    )));
                }
                self.consume_newlines();
                if self.peek_kind() == TokenKind::Indent {
                    let b = self.parse_indented_region()?;
                    sec_bodies.insert(name, b);
                } else {
                    sec_bodies.insert(name, Block::default());
                }
                continue;
            }
            // Not a section — must be the closer.
            let cp = match self.cands.by_name.get(closer_name).cloned() {
                None => {
                    return Err(CapyError::msg(format!(
                        "library function {} (closer) not found",
                        gofmt::quote(closer_name)
                    )))
                }
                Some(c) => c,
            };
            let saved = self.mark();
            let closer_inst = match self.try_match(&cp) {
                Err(_) => {
                    self.rewind(saved);
                    return Err(CapyError::msg(format!(
                        "line {}: expected closer {} or one of sections [{}]",
                        tok.line,
                        gofmt::quote(closer_name),
                        sections.join(" ")
                    )));
                }
                Ok(c) => c,
            };
            if self.peek_kind() != TokenKind::Newline && self.peek_kind() != TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "line {}: closer must end the line",
                    self.peek_line()
                )));
            }
            self.consume_newlines();
            return Ok((main_body, sec_bodies, Some(closer_inst)));
        }
    }
}

/// Port of `resolveCloseSeq`.
///
/// Flattens a `[]CloseSegment` into the concrete token sequence that closes this
/// particular block instance. Literal segments contribute their fixed
/// pre-tokenized tokens; a ref segment substitutes the source text of the named
/// opener-bound capture as a single token.
fn resolve_close_seq(
    segs: &[CloseSegment],
    caps: &BTreeMap<String, CaptureValue>,
) -> Result<Vec<String>, CapyError> {
    let mut out: Vec<String> = Vec::new();
    for seg in segs {
        if !seg.ref_.is_empty() {
            match caps.get(&seg.ref_) {
                None => {
                    return Err(CapyError::msg(format!(
                        "block_close_seq references capture {} which is not bound",
                        gofmt::quote(&seg.ref_)
                    )))
                }
                Some(cv) => out.push(cv.text.clone()),
            }
            continue;
        }
        out.extend(seg.tokens.iter().cloned());
    }
    Ok(out)
}

/// Port of `defaultCapture`.
///
/// Builds the `CaptureValue` bound when an optional arg is omitted. The text is
/// stored in the SAME source-text form a real capture of that type would carry,
/// so `${x}` and `${decoded x}` behave identically whether the arg was supplied
/// or defaulted: a `string`-typed default is re-quoted; other kinds keep the raw
/// value.
fn default_capture(cap_type: &str, def: &str) -> CaptureValue {
    if cap_type == "string" {
        return CaptureValue { text: gofmt::quote(def), ..Default::default() };
    }
    CaptureValue { text: def.to_string(), ..Default::default() }
}

/// Port of `nextLiterals`.
fn next_literals(rest: &[PatternElement]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in rest {
        if !e.is_capture {
            out.push(e.literal.clone());
        } else {
            break;
        }
    }
    out
}

/// Port of `tokenWidth` — the raw source span in bytes, falling back to the text
/// length when the lexer left width unset (width is authoritative for quoted
/// strings, where text has the quotes stripped).
fn token_width(tok: &Token) -> usize {
    if tok.width > 0 {
        tok.width
    } else {
        tok.text.len()
    }
}

/// Port of `tokenSourceText`.
///
/// Returns a token's text the way it appeared in source. For string/template
/// tokens the lexer strips the surrounding quotes from text; this re-adds them so
/// a `tail` capture preserves the slot boundary of a spaced, quoted argument.
fn token_source_text(tok: &Token) -> String {
    match tok.kind {
        TokenKind::Str => requote(&tok.text, b'"'),
        TokenKind::Template => requote(&tok.text, b'`'),
        _ => tok.text.clone(),
    }
}

/// Port of `requote`.
fn requote(inner: &str, quote: u8) -> String {
    let mut out: Vec<u8> = Vec::new();
    out.push(quote);
    let b = inner.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            // Preserve an existing escape sequence as-is.
            out.push(c);
            out.push(b[i + 1]);
            i += 2;
            continue;
        }
        if c == quote {
            out.push(b'\\');
        }
        out.push(c);
        i += 1;
    }
    out.push(quote);
    String::from_utf8_lossy(&out).into_owned()
}

/// Port of `tokenSourceForm`.
///
/// Reproduces a token's source-text representation. For most kinds the text field
/// is exact; for strings the quote characters the lexer stripped are re-added.
fn token_source_form(t: &Token) -> String {
    match t.kind {
        TokenKind::Str => format!("\"{}\"", t.text),
        TokenKind::Template => format!("`{}`", t.text),
        _ => t.text.clone(),
    }
}

/// The opening delimiter that pairs with `close`, for the label text.
fn opening_of(close: &str) -> &str {
    match close {
        ")" => "(",
        "]" => "[",
        "}" => "{",
        other => other,
    }
}
