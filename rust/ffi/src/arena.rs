//! The parse handle: an owned `ParseResult` plus a flat table of ids that point into it.
//!
//! ```text
//!   Box<ParseResult>  ── never moved, never mutated after construction
//!         ▲
//!         │ raw pointers (one per tree node)
//!   entries: Vec<Entry>      id = tag << 24 | (index + 1)        0 = none
//!   kids:    Vec<u32>        child index lists, referenced by (start, len) from an Entry
//! ```
//!
//! Every accessor in this crate resolves an id through [`CapyParse::entry`] — null, wrong parse, out of
//! range and wrong kind are all rejected there — and runs under [`with`], which turns a panic into a
//! poisoned handle instead of an unwind into C.

use capy_core::capy::ParseResult;
use capy_core::domain::ast::{Block, CaptureValue, ErrorNode, Expr, FuncCall, Span};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const CAPY_OK: i32 = 0;
pub const CAPY_E_NULL: i32 = 1;
pub const CAPY_E_ARG: i32 = 2;
pub const CAPY_E_KIND: i32 = 3;
pub const CAPY_E_UTF8: i32 = 4;
pub const CAPY_E_PANIC: i32 = 5;
pub const CAPY_E_LIMIT: i32 = 6;
pub const CAPY_E_LOAD: i32 = 7;

/// Ids use 24 bits of index; the top 8 bits are a per-parse tag.
const INDEX_BITS: u32 = 24;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;

/// An id into one parse's tree (block, statement, capture, expression or error node). 0 is "none".
/// Valid only for the `CapyParse` that produced it.
pub type CapyId = u32;

/// A borrowed UTF-8 string. `ptr == NULL` means absent; a non-NULL pointer with `len == 0` is an empty
/// string. Not NUL-terminated. Valid until the owning handle is freed.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CapyStr {
    pub ptr: *const u8,
    pub len: usize,
}

impl CapyStr {
    pub const NONE: CapyStr = CapyStr { ptr: std::ptr::null(), len: 0 };
    pub fn of(s: &str) -> CapyStr {
        CapyStr { ptr: s.as_ptr(), len: s.len() }
    }
}

/// A source range: 1-indexed line/column, `end_col` exclusive, `start_line == 0` means unset.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CapySpan {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

fn sat(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl CapySpan {
    pub fn of(s: Span) -> CapySpan {
        CapySpan {
            start_line: sat(s.start_line),
            start_col: sat(s.start_col),
            end_line: sat(s.end_line),
            end_col: sat(s.end_col),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Node {
    Block(*const Block),
    Stmt(*const FuncCall),
    Cap(*const CaptureValue),
    Expr(*const Expr),
    Err(*const ErrorNode),
}

/// What the `a`, `b`, `x`, `y` fields hold, by kind:
///
/// | kind  | a (start,len in `kids`) | b                 | x          | y            |
/// |-------|-------------------------|-------------------|------------|--------------|
/// | Block | statements              | error nodes       |            |              |
/// | Stmt  | captures (name order)   | sections (name order) | body block | closer stmt |
/// | Cap   | sub statements          |                   | expr       |              |
/// | Expr  | children (see expr.rs)  |                   |            |              |
/// | Err   |                         |                   |            |              |
///
/// Child references are raw `index + 1` values (0 = none); the tag is added on the way out.
#[derive(Clone, Copy)]
pub struct Entry {
    pub node: Node,
    pub a: (u32, u32),
    pub b: (u32, u32),
    pub x: u32,
    pub y: u32,
}

impl Entry {
    fn new(node: Node) -> Entry {
        Entry { node, a: (0, 0), b: (0, 0), x: 0, y: 0 }
    }
}

pub struct CapyParse {
    result: Box<ParseResult>,
    entries: Vec<Entry>,
    kids: Vec<u32>,
    tag: u32,
    poisoned: AtomicBool,
}

// SAFETY: the raw pointers in `entries` point into `result`, which is boxed, never moved and never
// mutated after `build`; every access is a shared read. Sharing a `&CapyParse` across threads is
// therefore sound. (Freeing while another thread reads is a caller contract violation, as in C.)
unsafe impl Send for CapyParse {}
unsafe impl Sync for CapyParse {}

static NEXT_TAG: AtomicU32 = AtomicU32::new(0);

impl CapyParse {
    /// Flatten `result`. Iterative: a breadth-first queue over `entries`, so nesting depth cannot
    /// overflow the stack. Fails with `CAPY_E_LIMIT` when the tree needs more than 2^24 - 2 ids.
    pub fn build(result: ParseResult) -> Result<CapyParse, i32> {
        let result = Box::new(result);
        let mut entries: Vec<Entry> = Vec::new();
        let mut kids: Vec<u32> = Vec::new();

        fn push(entries: &mut Vec<Entry>, node: Node) -> Result<u32, i32> {
            if entries.len() >= (INDEX_MASK - 1) as usize {
                return Err(CAPY_E_LIMIT);
            }
            entries.push(Entry::new(node));
            Ok(entries.len() as u32) // index + 1
        }

        push(&mut entries, Node::Block(&result.tree as *const Block))?;
        let mut i = 0;
        while i < entries.len() {
            let node = entries[i].node;
            let mut e = entries[i];
            // SAFETY: each pointer was taken from a reference into `result` (boxed, alive, immutable).
            match node {
                Node::Block(p) => {
                    let b = unsafe { &*p };
                    let s = kids.len() as u32;
                    for st in &b.stmts {
                        kids.push(push(&mut entries, Node::Stmt(st))?);
                    }
                    e.a = (s, b.stmts.len() as u32);
                    let s = kids.len() as u32;
                    for er in &b.errors {
                        kids.push(push(&mut entries, Node::Err(er))?);
                    }
                    e.b = (s, b.errors.len() as u32);
                }
                Node::Stmt(p) => {
                    let f = unsafe { &*p };
                    let s = kids.len() as u32;
                    for c in f.captures.values() {
                        kids.push(push(&mut entries, Node::Cap(c))?);
                    }
                    e.a = (s, f.captures.len() as u32);
                    let s = kids.len() as u32;
                    for blk in f.sections.values() {
                        kids.push(push(&mut entries, Node::Block(blk))?);
                    }
                    e.b = (s, f.sections.len() as u32);
                    if let Some(b) = &f.body {
                        e.x = push(&mut entries, Node::Block(&**b))?;
                    }
                    if let Some(c) = &f.closer {
                        e.y = push(&mut entries, Node::Stmt(&**c))?;
                    }
                }
                Node::Cap(p) => {
                    let c = unsafe { &*p };
                    let s = kids.len() as u32;
                    for st in &c.sub {
                        kids.push(push(&mut entries, Node::Stmt(st))?);
                    }
                    e.a = (s, c.sub.len() as u32);
                    if let Some(x) = &c.expr {
                        e.x = push(&mut entries, Node::Expr(x))?;
                    }
                }
                Node::Expr(p) => {
                    let x = unsafe { &*p };
                    let s = kids.len() as u32;
                    let mut n = 0u32;
                    let mut add = |entries: &mut Vec<Entry>, kids: &mut Vec<u32>, c: Option<&Expr>| -> Result<(), i32> {
                        kids.push(match c {
                            Some(c) => push(entries, Node::Expr(c))?,
                            None => 0,
                        });
                        n += 1;
                        Ok(())
                    };
                    match x {
                        Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Null => {}
                        Expr::Var(steps) => {
                            for st in steps {
                                let ix = if st.is_index { st.index.as_deref() } else { None };
                                add(&mut entries, &mut kids, ix)?;
                            }
                        }
                        Expr::Call(c) => {
                            for a in &c.args {
                                add(&mut entries, &mut kids, Some(a))?;
                            }
                        }
                        Expr::Compare(c) => {
                            add(&mut entries, &mut kids, Some(&c.left))?;
                            add(&mut entries, &mut kids, Some(&c.right))?;
                        }
                        Expr::Binary(c) => {
                            add(&mut entries, &mut kids, Some(&c.left))?;
                            add(&mut entries, &mut kids, Some(&c.right))?;
                        }
                        Expr::Not(c) => add(&mut entries, &mut kids, Some(&**c))?,
                        Expr::Unary(u) => add(&mut entries, &mut kids, Some(&u.operand))?,
                        Expr::List(items) => {
                            for it in items {
                                add(&mut entries, &mut kids, Some(it))?;
                            }
                        }
                        Expr::Obj(o) => {
                            for v in &o.vals {
                                add(&mut entries, &mut kids, Some(v))?;
                            }
                        }
                    }
                    e.a = (s, n);
                }
                Node::Err(_) => {}
            }
            entries[i] = e;
            i += 1;
        }
        // 1..=255: never 0, so no valid id is 0.
        let tag = NEXT_TAG.fetch_add(1, Ordering::Relaxed) % 255 + 1;
        Ok(CapyParse { result, entries, kids, tag, poisoned: AtomicBool::new(false) })
    }

    pub fn result(&self) -> &ParseResult {
        &self.result
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Relaxed)
    }

    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Relaxed)
    }

    /// An id as handed to C: the stored `index + 1` with this parse's tag on top. 0 stays 0.
    pub fn out(&self, raw: u32) -> u32 {
        if raw == 0 {
            0
        } else {
            (self.tag << INDEX_BITS) | raw
        }
    }

    pub fn root(&self) -> u32 {
        self.out(1)
    }

    /// Resolve an id from C. `None` for 0, a foreign tag, or an index out of range.
    pub fn entry(&self, id: u32) -> Option<&Entry> {
        if id == 0 || id >> INDEX_BITS != self.tag {
            return None;
        }
        let raw = id & INDEX_MASK;
        if raw == 0 {
            return None;
        }
        self.entries.get(raw as usize - 1)
    }

    /// The `i`-th id of a child list.
    pub fn kid(&self, range: (u32, u32), i: u32) -> u32 {
        if i >= range.1 {
            return 0;
        }
        self.kids.get((range.0 + i) as usize).map(|&r| self.out(r)).unwrap_or(0)
    }

    pub fn block(&self, id: u32) -> Option<(&Entry, &Block)> {
        let e = self.entry(id)?;
        match e.node {
            // SAFETY: see the type-level comment; the pointer targets the boxed result.
            Node::Block(p) => Some((e, unsafe { &*p })),
            _ => None,
        }
    }

    pub fn stmt(&self, id: u32) -> Option<(&Entry, &FuncCall)> {
        let e = self.entry(id)?;
        match e.node {
            Node::Stmt(p) => Some((e, unsafe { &*p })),
            _ => None,
        }
    }

    pub fn cap(&self, id: u32) -> Option<(&Entry, &CaptureValue)> {
        let e = self.entry(id)?;
        match e.node {
            Node::Cap(p) => Some((e, unsafe { &*p })),
            _ => None,
        }
    }

    pub fn expr(&self, id: u32) -> Option<(&Entry, &Expr)> {
        let e = self.entry(id)?;
        match e.node {
            Node::Expr(p) => Some((e, unsafe { &*p })),
            _ => None,
        }
    }

    pub fn err(&self, id: u32) -> Option<&ErrorNode> {
        match self.entry(id)?.node {
            Node::Err(p) => Some(unsafe { &*p }),
            _ => None,
        }
    }
}

/// Run `f` against a parse handle. A null or poisoned handle yields `default`; a panic poisons the
/// handle and also yields `default`. This is the single place the boundary is guarded (R8, R9).
pub(crate) fn with<T>(p: *const CapyParse, default: T, f: impl FnOnce(&CapyParse) -> T) -> T {
    if p.is_null() {
        return default;
    }
    // SAFETY: non-null; the caller promises it came from `capy_parse` and has not been freed.
    let p = unsafe { &*p };
    if p.is_poisoned() {
        return default;
    }
    match catch_unwind(AssertUnwindSafe(|| f(p))) {
        Ok(v) => v,
        Err(_) => {
            p.poison();
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> CapyParse {
        CapyParse::build(ParseResult::default()).unwrap()
    }

    /// R9 / T-08: a panic inside an accessor must come back as the default value and poison the handle.
    #[test]
    #[cfg(panic = "unwind")]
    fn panic_is_caught_and_poisons_the_handle() {
        let p = empty();
        let v = with(&p as *const CapyParse, 7u32, |_| -> u32 { panic!("forced") });
        assert_eq!(v, 7);
        assert!(p.is_poisoned());
        // every later call returns the default without running the closure
        let v = with(&p as *const CapyParse, 9u32, |_| 1);
        assert_eq!(v, 9);
    }

    #[test]
    fn null_handle_yields_the_default() {
        assert_eq!(with(std::ptr::null(), 3u32, |_| 1), 3);
    }

    #[test]
    fn id_zero_foreign_tag_and_out_of_range_are_rejected() {
        let p = empty();
        let root = p.root();
        assert!(p.entry(root).is_some());
        assert!(p.entry(0).is_none());
        assert!(p.entry(root ^ (1 << INDEX_BITS)).is_none(), "foreign tag");
        assert!(p.entry((p.tag << INDEX_BITS) | 9999).is_none(), "out of range");
        assert!(p.entry(p.tag << INDEX_BITS).is_none(), "tag only");
    }

    #[test]
    fn kind_mismatch_is_rejected() {
        let p = empty();
        assert!(p.block(p.root()).is_some());
        assert!(p.stmt(p.root()).is_none());
        assert!(p.expr(p.root()).is_none());
    }
}
