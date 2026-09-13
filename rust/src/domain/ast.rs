//! Port of `domain/ast.go`.
//!
//! Go's `Expr` and `InnerStmt` are empty marker interfaces dispatched via type
//! switch; both become Rust enums. Struct shapes and field names are preserved
//! one-for-one so the evaluator port reads against the same model.

use std::collections::BTreeMap;

/// A user-script program is a `Block` of `FuncCall`s.
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
}

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
    /// templates as the render locals `line` and `col`.
    pub line: usize,
    pub col: usize,
}

impl FuncCall {
    pub fn new(func: impl Into<String>, line: usize, col: usize) -> FuncCall {
        FuncCall {
            func: func.into(),
            captures: BTreeMap::new(),
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
    Not(Box<Expr>),
    List(Vec<Expr>),
    Obj(ObjLit),
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
