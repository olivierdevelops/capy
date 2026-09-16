//! Port of `domain/ast.go`.
//!
//! Go's `Expr` and `InnerStmt` are empty marker interfaces dispatched via type
//! switch; both become Rust enums. Struct shapes and field names are preserved
//! one-for-one so the evaluator port reads against the same model.

use std::collections::BTreeMap;

/// PLAN-2026-0002 R22 — a region of source that could not be parsed.
///
/// Recovery keeps going after a failed statement so the user sees every mistake
/// in one run, and an editor gets a usable tree from a buffer that is mid-edit.
/// The skipped tokens are kept verbatim so tooling can still highlight them.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ErrorNode {
    pub span: Span,
    /// The tokens that were skipped, verbatim.
    pub tokens: Vec<crate::domain::token::Token>,
    /// Index into the `ParseResult`'s diagnostics list.
    pub diagnostic_index: usize,
}

/// A user-script program is a `Block` of `FuncCall`s.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub stmts: Vec<FuncCall>,
    /// Marks a body that wasn't parsed as nested statements — instead the
    /// body's raw source bytes were captured into `verbatim_text`. Used by
    /// functions declared `block_verbatim`. The renderer surfaces
    /// `verbatim_text` via `${body}` exactly like a parsed block body's
    /// rendered output.
    pub is_verbatim: bool,
    pub verbatim_text: String,
    /// PLAN-2026-0002 R22 — regions that failed to parse.
    ///
    /// Deliberately a PARALLEL field: `stmts` keeps its type `Vec<FuncCall>`, so
    /// every existing walker still compiles and still sees a correct — if
    /// incomplete — list of statements. Widening `stmts` to an enum would have
    /// broken every consumer, and `#[non_exhaustive]` does not protect against a
    /// field changing type. Ordering against `stmts` is recoverable from spans.
    ///
    /// Empty means the parse was clean.
    pub errors: Vec<ErrorNode>,
}

/// PLAN-2026-0001 R1 — the source range a node came from.
///
/// `FuncCall.line`/`col` (which predate this) mark only where a statement
/// *starts*, and nothing at all marked where a capture or a nested node came
/// from: interior nodes were stamped `line: 0, col: 0`. A consumer therefore
/// could not underline a construct, and could not point at the operand a rule
/// was actually about.
///
/// Positions are 1-indexed and source-absolute, matching `Token.line`/`col`.
/// `end_col` is EXCLUSIVE — it is the column one past the last byte of the
/// span's final token — so `end_col - start_col` is a width on a single line.
///
/// `#[non_exhaustive]`: byte offsets are a planned addition (see the deviation
/// recorded in PLAN-2026-0001). Constructing a `Span` outside this crate goes
/// through [`Span::new`] so that addition cannot break a consumer.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Span {
        Span { start_line, start_col, end_line, end_col }
    }

    /// A span that was never populated. `start_line == 0` is the sentinel — no
    /// real source position is on line 0, which is what made the old
    /// `line: 0, col: 0` stamping detectable in the first place.
    pub fn is_unset(&self) -> bool {
        self.start_line == 0
    }

    /// The smallest span covering both. An unset operand is ignored rather than
    /// dragging the result back to line 0.
    pub fn join(a: Span, b: Span) -> Span {
        if a.is_unset() {
            return b;
        }
        if b.is_unset() {
            return a;
        }
        let start = if (b.start_line, b.start_col) < (a.start_line, a.start_col) {
            (b.start_line, b.start_col)
        } else {
            (a.start_line, a.start_col)
        };
        let end = if (b.end_line, b.end_col) > (a.end_line, a.end_col) {
            (b.end_line, b.end_col)
        } else {
            (a.end_line, a.end_col)
        };
        Span { start_line: start.0, start_col: start.1, end_line: end.0, end_col: end.1 }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct FuncCall {
    /// Index into the library's function table. Go stores `*FuncDef`; an index
    /// avoids a self-referential borrow while resolving to the same definition.
    pub func: String,
    pub captures: BTreeMap<String, CaptureValue>,
    /// Present when the matched function declares a block.
    pub body: Option<Box<Block>>,
    pub closer: Option<Box<FuncCall>>,
    /// Parsed sub-bodies of a multi-section block, keyed by section keyword.
    /// Absent sections are not present; the renderer defaults them to "".
    pub sections: BTreeMap<String, Block>,
    /// 1-indexed source position of the statement's first token. Exposed to
    /// templates as the render locals `line` and `col`. Unchanged by
    /// PLAN-2026-0001 (R5) — `span.start` carries the same position.
    pub line: usize,
    pub col: usize,
    /// PLAN-2026-0001 R2 — the full source range of this statement, including
    /// its block body and closer when it has them.
    pub span: Span,
    /// PLAN-2026-0001 R27 — spans of the comments immediately preceding this
    /// statement, in source order. The statement's own `span` EXCLUDES them.
    pub leading_comments: Vec<Span>,
}

impl FuncCall {
    pub fn new(func: impl Into<String>, line: usize, col: usize) -> FuncCall {
        FuncCall {
            func: func.into(),
            captures: BTreeMap::new(),
            span: Span::new(line, col, line, col),
            leading_comments: Vec::new(),
            body: None,
            closer: None,
            sections: BTreeMap::new(),
            line,
            col,
        }
    }
}

/// The bound value for a named capture in a matched `FuncCall`.
/// Identifier/raw captures carry text; expression-typed captures carry an
/// unevaluated `Expr` the evaluator resolves at render time.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CaptureValue {
    pub is_expr: bool,
    pub expr: Option<Expr>,
    pub text: String,
    /// Matched sub-`FuncCall`(s) when this capture's declared type is another
    /// library function (function-as-type / named nonterminal). A single
    /// function-typed capture yields one entry; a repeated capture (`type*` /
    /// `type+`) yields one per occurrence. When non-empty the capture is a
    /// structural match, not a flat token.
    pub sub: Vec<FuncCall>,
    /// PLAN-2026-0001 R3 — the source range of exactly the tokens this value was
    /// captured from.
    pub span: Span,
}

// --- Expression AST (used by both outer template captures and inner DSL) ---

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(NumberLit),
    Str(String),
    Bool(bool),
    Null,
    /// A read-position access chain: a root name followed by any mix of
    /// `.field` and `[expr]` index steps. `steps[0]` is always a field step
    /// holding the root name (e.g. "context" or a local var). Mirrors the
    /// write-side `Path`/`PathStep` model so reads and writes navigate maps
    /// and lists identically — `${context.buf[i]}` resolves the same element
    /// `set context.buf[i] …` wrote.
    Var(Vec<PathStep>),
    Call(CallExpr),
    Compare(Box<CompareExpr>),
    /// PLAN-2026-0002 R10 — an infix operation. `op` is one of
    /// `* / % + - and or`; comparisons keep using [`Expr::Compare`] so the
    /// existing evaluator path is untouched.
    Binary(Box<BinaryExpr>),
    Not(Box<Expr>),
    List(Vec<Expr>),
    Obj(ObjLit),
}

/// PLAN-2026-0002 R10 — an infix operation with its two operands.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub op: String,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLit {
    pub is_int: bool,
    pub i: i64,
    pub f: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub name: Vec<String>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompareExpr {
    pub op: String,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjLit {
    pub keys: Vec<String>,
    pub vals: Vec<Expr>,
}

// --- Inner DSL AST (the `run:` language; hardcoded in the engine) ---

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InnerBlock {
    pub stmts: Vec<InnerStmt>,
}

/// A dotted/indexed access chain rooted at a name.
///
/// ```text
/// context.imports    → Path { root: "context", steps: [Field("imports")] }
/// context.vars[name] → Path { root: "context", steps: [Field("vars"), Index(Var(name))] }
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    pub root: String,
    pub steps: Vec<PathStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathStep {
    pub is_index: bool,
    pub field: String,
    pub index: Option<Box<Expr>>,
}

impl PathStep {
    pub fn field(name: impl Into<String>) -> PathStep {
        PathStep { is_index: false, field: name.into(), index: None }
    }
    pub fn index(e: Expr) -> PathStep {
        PathStep { is_index: true, field: String::new(), index: Some(Box::new(e)) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InnerStmt {
    Set { target: Path, value: Expr },
    Append { target: Path, value: Expr },
    Prepend { target: Path, value: Expr },
    Merge { target: Path, value: Expr },
    Delete { target: Path },
    If { cond: Expr, body: InnerBlock, else_: Option<Box<InnerBlock>> },
    Loop(LoopStmt),
    Call(CallExpr),
    /// Appends the rendered value to the current function's output buffer.
    /// Used by the unified `write` block design; the translator in
    /// `orchestrator/features/make_library_loader.go` processes these out
    /// before the engine sees them.
    Write(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopStmt {
    /// The value variable name. For two-var loops over maps this is the value
    /// side; for two-var loops over lists, the element side.
    pub var: String,
    /// Optional key/index variable. Empty for one-var loops. When set:
    /// iterating a list, it is the integer index; iterating a map, the key.
    pub key_var: String,
    pub iter: Expr,
    pub body: InnerBlock,
}
