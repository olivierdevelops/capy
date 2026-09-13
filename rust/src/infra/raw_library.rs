//! Port of `infra/raw_library.go`.
//!
//! `RawLibrary` is the parser-output DTO. Both the `.capy` parser and any future
//! format adapter produce values of this shape; the orchestrator's loader maps
//! it into a [`crate::domain::library::Library`].
//!
//! The DTO carries STRING-form template fields (`file_template`, `files`) for
//! parser tests and debug printing, plus new-shape inner-DSL `body` strings that
//! the renderer-side AST path actually uses.

use crate::domain::val::Val;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct RawLibrary {
    pub extension: String,
    pub output_file: String,
    pub description: String,
    pub types: BTreeMap<String, RawType>,
    pub context: BTreeMap<String, Val>,
    pub functions: BTreeMap<String, RawFunction>,
    pub file_template: String,

    /// Multi-file output. Map of relative-path → body source.
    pub files: BTreeMap<String, String>,

    /// Relative paths to other library files whose functions, types, and context
    /// get merged in before this library's own declarations (which take
    /// precedence on conflict).
    pub imports: Vec<String>,

    /// Names of source-level inclusion directives this library opts into (e.g.
    /// `["@import", "@include"]`). When empty, no preprocessing runs. Keeps
    /// "zero predefined grammar" honest.
    pub preprocess: Vec<String>,

    /// Line-comment markers the library opts into for user scripts. Empty list →
    /// user scripts have NO comment syntax. Mirrors `preprocess`: the engine
    /// ships zero defaults and the library must declare what to recognise.
    pub comments: Vec<String>,

    /// Manifest fields (set when the library declares them at the top level of
    /// its `.capy` file — `name "X"`, `version "X"`).
    pub lib_name: String,
    pub lib_version: String,

    /// Commands declared by the library — `command "X" … end` blocks at the top
    /// level. The body is collected as raw text; the loader parses it via the
    /// inner-DSL parser.
    pub commands: BTreeMap<String, RawCommand>,

    /// Implementations: `impl "NAME" "FILE" … end` blocks at the top level. Each
    /// entry points at a sibling `.capy` file that provides the actual
    /// functions. When the map is non-empty, the manifest file itself only
    /// carries metadata + commands.
    pub impls: BTreeMap<String, RawImpl>,

    /// Names the impl chosen when neither `--impl` nor `CAPY_IMPL` picks one. If
    /// empty + `impls` is non-empty + only one impl declared, that one is used;
    /// otherwise an error.
    pub default_impl: String,
}

/// One `impl "NAME" "FILE" … end` declaration.
#[derive(Debug, Clone, Default)]
pub struct RawImpl {
    pub name: String,
    pub file: String,
    pub description: String,
    pub version: String,
    pub is_default: bool,
}

/// One `command "X" … end` declaration.
#[derive(Debug, Clone, Default)]
pub struct RawCommand {
    pub description: String,
    pub body: String,
    pub args: Vec<RawCommandArg>,
    pub flags: Vec<RawCommandFlag>,
}

/// A positional argument declaration.
#[derive(Debug, Clone, Default)]
pub struct RawCommandArg {
    pub name: String,
    pub required: bool,
    pub description: String,
}

/// A flag declaration.
#[derive(Debug, Clone, Default)]
pub struct RawCommandFlag {
    pub name: String,
    pub description: String,
    pub default: String,
    pub is_bool: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RawFunction {
    pub description: String,
    pub args: Vec<RawArg>,
    pub block: Option<RawBlock>,
    pub priority: i64,

    /// Opts the function out of the auto-name-prepend rule. With this flag set, a
    /// function declared with only `arg capture` entries matches purely by shape
    /// — useful for grammars whose data lines have no leading keyword (e.g.
    /// `"1" "2" "3"` as a row of button labels).
    pub bare: bool,

    /// The function body — inner-DSL statements including `write` calls. The
    /// loader parses it into the `FuncDef`'s `template_ast` + `run_ast`.
    pub body: String,

    /// The `when_followed_by indent` / `when_not_followed_by indent` lookahead
    /// predicates. At most one is set.
    pub followed_by_indent: bool,
    pub not_followed_by_indent: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RawBlock {
    pub closer: String,
    pub open: String,
    pub close: String,
    /// Body ends at the first DEDENT after the opener, with no closer keyword.
    /// Used for indent-only blocks (CSS-style rules, YAML-style sections).
    pub is_dedent: bool,
    /// Body is captured as raw source bytes (no nested parsing) until the named
    /// `closer` keyword. Used for code blocks, embedded HTML, or anywhere the
    /// body is data not grammar.
    pub is_verbatim: bool,
    /// Interior section keywords for a multi-section block (e.g.
    /// `["rescue","finally"]`). When set, the block is closed by `closer` and
    /// each section introduces its own indented sub-body.
    pub sections: Vec<String>,
    /// Segments of a (possibly capture-bound) multi-token sequence closer, set
    /// by `block_close_seq "</" name ">"`. Each segment is a literal (quoted in
    /// source) or a capture reference (bare identifier). Empty unless used.
    pub close_seq: Vec<RawCloseSeg>,
}

/// One segment of a `block_close_seq` directive: a quoted literal
/// (`is_ref == false`) or a bare capture-name reference (`is_ref == true`).
#[derive(Debug, Clone, Default)]
pub struct RawCloseSeg {
    pub text: String,
    pub is_ref: bool,
}

/// An args-list entry. The `kind` discriminator is required.
#[derive(Debug, Clone, Default)]
pub struct RawArg {
    pub kind: String,
    pub value: String,
    pub name: String,
    pub type_: String,
    pub description: String,
    /// Marks a trailing capture that may be omitted at the call site; `default`
    /// is the source-form value bound when it's omitted. Only meaningful when
    /// `kind == "capture"`. Set by the `default "…"` suffix on an `arg capture`
    /// line.
    pub optional: bool,
    pub default: String,
    /// `repeat` is `""` / `"*"` / `"+"` (set by a `*` or `+` suffix on the type);
    /// `sep` is the optional INPUT separator literal from a trailing `sep "X"`
    /// (consumed between repetitions while parsing); `join` is the optional
    /// OUTPUT separator from a trailing `join "X"` (inserted between rendered
    /// sub-results). `sep` and `join` are independent.
    pub repeat: String,
    pub sep: String,
    pub join: String,
}

#[derive(Debug, Clone, Default)]
pub struct RawType {
    pub description: String,
    pub base: String,
    pub pattern: String,
    pub options: Vec<String>,
    /// Declare a type as a delimited group: when the outer parser captures a
    /// value of this type it consumes the open delimiter, walks tokens (with
    /// balanced nesting) until the matching close delimiter, and joins
    /// everything in between as source-form text. Use for inline syntax like
    /// Markdown's `[label](url)`, `**bold**`, `` `code` ``, or LaTeX's `\cmd{x}`.
    pub group_open: String,
    pub group_close: String,
}
