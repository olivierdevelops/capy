//! Port of `domain/library.go`.

use super::ast::InnerBlock;
use super::command::CommandDef;
use super::impl_def::ImplDef;
use super::val::Val;
use std::collections::BTreeMap;

/// The result of loading a `.capy` library file — the entire grammar plus
/// accumulation rules for one source-language → target-output transpilation.
///
/// Surface-syntax conventions (block start/end, statement terminator, arg
/// separator) are fixed for now — INDENT/DEDENT for blocks, NEWLINE for
/// statements, whitespace for args.
#[derive(Debug, Clone, Default)]
pub struct Library {
    pub extension: String,
    pub output_file: String,
    /// Free-form summary of what the library is for — shown at the top of
    /// `capy docs <library>` output.
    pub description: String,
    pub functions: BTreeMap<String, FuncDef>,
    pub types: BTreeMap<String, TypeDef>,
    /// Initial context values (lists, maps, scalars).
    pub context: BTreeMap<String, Val>,

    /// Parsed write-style body of the library's `file_template … end` block.
    /// The renderer walks this directly. `None` when the library has no
    /// file_template (renderer uses the top-level body verbatim).
    pub file_template_ast: Option<InnerBlock>,

    /// Multi-file output declaration: each entry is a relative path (may
    /// contain write-style `${...}` interpolations for dynamic naming, resolved
    /// at render time) → the parsed write-style AST of that file's body.
    ///
    /// When non-empty AND the CLI is invoked with `--out-dir`, the engine writes
    /// every entry to disk and ignores `output_file`. When empty, behaviour
    /// falls back to the `file_template_ast` → stdout (or `output_file`) path.
    pub files_ast: BTreeMap<String, InnerBlock>,

    /// Commands declared in the library's manifest. Each maps a verb name
    /// (e.g. "build", "serve", "new") to a `CommandDef`. The CLI dispatches
    /// `capy <lib> <name>` to the matching command.
    pub commands: BTreeMap<String, CommandDef>,

    /// Canonical name declared in the manifest.
    pub lib_name: String,
    /// Semver string.
    pub lib_version: String,

    /// Every implementation the library declared via `impl "NAME" "FILE" … end`
    /// blocks in its manifest. When non-empty, the manifest file itself carries
    /// no functions — the real ones live in the selected impl's file.
    pub impls: BTreeMap<String, ImplDef>,
    /// Name from the manifest's `default` directive.
    pub default_impl: String,
    /// Populated by the CLI after selection.
    pub selected_impl: String,

    /// Source-level inclusion directives the library OPTS INTO. The engine
    /// ships zero default preprocessing — if empty, lines like
    /// `@import "x.capy"` are NOT recognised and flow into the lexer as
    /// ordinary tokens. A library that wants text-level file inclusion declares
    /// the directives explicitly:
    ///
    /// ```text
    /// preprocess
    ///     include "@import"
    ///     include "@include"
    /// end
    /// ```
    ///
    /// Each declared directive is processed identically (read the quoted path,
    /// splice the file's bytes into the source, recurse). This keeps Capy's
    /// "zero predefined grammar" promise intact: even directives that look
    /// universal come from the library, not the engine.
    pub preprocess: Vec<String>,

    /// Line-comment markers the library opts into for its USER SCRIPTS. The
    /// engine ships zero default comment syntax — if empty, a `#` or `//` at
    /// the start of a script line is NOT a comment.
    ///
    /// This declaration ONLY affects user-script lexing. The manifest itself,
    /// including inner-DSL `run:` bodies and command bodies, always uses `#`.
    pub comments: Vec<String>,
}

/// A single library-defined source-language construct.
///
/// ```text
/// args:         declarative match shape (literals + typed captures)
/// template_ast: write-style body — renders to output text on match
/// run_ast:      state-mutation projection of the body — runs after render
/// block:        when set, this function opens a body block closed by block.closer
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FuncDef {
    pub name: String,
    /// Free-form, surfaced by `capy docs`.
    pub description: String,
    pub args: Vec<ArgEntry>,
    /// Compiled from `args`.
    pub elements: Vec<PatternElement>,

    /// Parsed write-style body. The renderer walks this directly —
    /// state-mutation statements inside it are render-side no-ops (they are
    /// handled by `run_ast`).
    pub template_ast: Option<InnerBlock>,

    /// State-mutation projection of the same body — `set` / `append` /
    /// `prepend` / `merge` / `delete` / `call` / `error` statements, with
    /// control flow that contains them preserved. Runs AFTER the render pass
    /// for each function call.
    pub run_ast: Option<InnerBlock>,

    pub block: Option<BlockSpec>,
    pub priority: i64,

    /// When set, a predicate the parser checks after the header matches: the
    /// candidate only applies if the token following its statement satisfies
    /// the predicate.
    pub lookahead: Option<Lookahead>,
}

/// A single args-list entry with an explicit kind discriminator.
/// `kind = "literal"` → only `value` is meaningful.
/// `kind = "capture"` → only `name` and `type` are meaningful.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArgEntry {
    /// `"literal"` | `"capture"`
    pub kind: String,
    pub value: String,
    pub name: String,
    pub type_: String,
    /// Optional, only meaningful when `kind == "capture"`.
    pub description: String,
    /// Marks a trailing capture that may be omitted at the call site.
    pub optional: bool,
    /// The source-form value bound when omitted.
    pub default: String,
    /// Repetition quantifier for a function-typed capture: `""` (exactly one),
    /// `"*"` (zero or more), or `"+"` (one or more). Only meaningful when
    /// `type_` names a library function.
    pub repeat: String,
    /// A required input separator literal between repetitions.
    pub sep: String,
    /// Inserted between the rendered sub-results on output.
    pub join: String,
}

/// Marks a function as a block opener. Two modes:
///
/// 1. Named-closer mode (default): the body is delimited by INDENT/DEDENT; the
///    closer function runs after.
/// 2. Delimiter mode (`open`/`close`): the body is delimited by exact tokens.
///    No closer function involved. Useful for `for x in 40 { … }` style.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockSpec {
    pub closer: String,
    pub open: String,
    pub close: String,
    pub is_dedent: bool,
    pub is_verbatim: bool,
    /// When non-empty, makes this a multi-token sequence-closed block: the body
    /// is a free-flowing sequence of statements (newlines and indentation are
    /// insignificant) terminated by the exact token sequence here. Unlike
    /// `open`/`close` (single-character delimiters) the closer may span several
    /// lexical tokens — e.g. `</div>` lexes to `["</", "div", ">"]`. Each block
    /// demands its own sequence, so a stray `</p>` inside a `<div>` body fails
    /// to match and surfaces as a parse error (mismatched-nesting detection).
    /// This is what makes angle-bracket matched-pair HTML authorable.
    ///
    /// Each segment is either a fixed literal (pre-tokenized into `tokens`) or
    /// a reference to a capture bound by the opener (`ref_`). A reference
    /// segment lets the closer DEPEND ON the opener — an opener capturing
    /// `name` can declare `block_close_seq "</" name ">"`, so `<div>` is closed
    /// only by `</div>` and `<p>` only by `</p>`, generically.
    pub close_seq: Vec<CloseSegment>,
    /// When non-empty, makes this a multi-section block (e.g.
    /// `try … rescue … finally … end`). Each entry is an interior section
    /// keyword that appears at the opener's indent and introduces its own
    /// indented sub-body. The main body and each section body are rendered
    /// independently and exposed to the template as `${body}` and a local named
    /// after each section keyword (`${rescue}`, `${finally}`). The block is
    /// closed by `closer`.
    pub sections: Vec<String>,
}

/// One piece of a capture-bound block closer (`BlockSpec::close_seq`). Exactly
/// one of `tokens` / `ref_` is set: `tokens` is a fixed, pre-tokenized literal
/// run (e.g. `"</"` → `["</"]`); `ref_` names a capture whose bound text is
/// substituted as a single token at parse time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CloseSegment {
    pub tokens: Vec<String>,
    pub ref_: String,
}

/// Gates a candidate on what follows its header, after the trailing newline.
/// Enables context-sensitive keyword reuse: e.g. a flat `os "X"` allowlist
/// entry (`when_not_followed_by indent`) coexisting with an `os "X"` that opens
/// an indented conditional block (`when_followed_by indent`). At most one of
/// the two fields is set.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Lookahead {
    /// `when_followed_by indent`
    pub require_indent: bool,
    /// `when_not_followed_by indent`
    pub forbid_indent: bool,
}

/// One compiled token in the function's match shape.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PatternElement {
    pub is_capture: bool,
    pub literal: String,
    pub name: String,
    pub cap_type: String,
    /// Applies to optional trailing capture elements: when the statement ends
    /// before this element is reached, the matcher binds `default` instead of
    /// consuming a token.
    pub optional: bool,
    pub default: String,
    /// `""` / `"*"` / `"+"` — see `ArgEntry::repeat`.
    pub repeat: String,
    /// Optional input separator literal between repetitions.
    pub sep: String,
    /// Optional output separator inserted between rendered sub-results.
    pub join: String,
    /// Marks that `cap_type` names a library function (resolved at load time)
    /// rather than a built-in/declared type.
    pub is_func: bool,
}

/// A library-defined argument type. Three optional fields applied in order at
/// validation time: `base` → `pattern` → `options`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeDef {
    pub name: String,
    /// Optional, surfaced by `capy docs`.
    pub description: String,
    /// `any` | `string` | `int` | `float` | `bool`
    pub base: String,
    /// Optional regex on the value's string form.
    pub pattern: String,
    /// Optional enum membership.
    pub options: Vec<String>,
    /// `group_open` / `group_close` mark a type as a delimited inline group.
    /// When set, the capture machinery walks tokens between the open and close
    /// delimiters (with depth tracking) and returns the joined source-form
    /// text. Constraint fields (`base` / `pattern` / `options`) are mutually
    /// exclusive with these.
    pub group_open: String,
    pub group_close: String,
}
