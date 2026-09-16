//! Port of `orchestrator/features/make_library_loader.go`.
//!
//! Compiles a library file into a [`Library`]:
//!
//! * args list → `Vec<ArgEntry>` → `Vec<PatternElement>`
//! * body snippet → `InnerBlock` AST (parsed via the outer lexer + inner parser)
//! * types/context/file_template carried through
//!
//! DETERMINISM NOTE: the Go original iterates `lib.Functions` (a Go map) during
//! cross-reference validation and returns on the FIRST failure, so when a library
//! has several invalid functions, *which* error the user sees varies per run.
//! Likewise `typeNames` and the `did you mean` candidate list are built from
//! unsorted map keys. `BTreeMap` here makes all three deterministic
//! (alphabetical) — strictly better, and the only divergence in this file.

use super::inner_parser::parse_inner;
use super::translate_new_shape::{render_inner_block, translate_new_shape};
use crate::domain::ast::InnerBlock;
use crate::domain::errors::{suggest_closest, CapyError};
use crate::domain::library::{
    ArgEntry, BlockSpec, CloseSegment, FuncDef, Library, Lookahead, PatternElement, TypeDef,
};
use crate::domain::token::{Token, TokenKind};
use crate::domain::{CommandArg, CommandDef, CommandFlag, ImplDef};
use crate::gofmt;
use crate::gopath;
use crate::infra::capy_lib_parser::{CapyLibParser, NEW_SHAPE};
use crate::infra::raw_library::{RawArg, RawFunction, RawLibrary};
use std::collections::{BTreeMap, BTreeSet};

/// The tokenizer callback the loader threads through (Go passes
/// `func(string) ([]domain.Token, error)`).
pub type Tokenize = fn(&str) -> Result<Vec<Token>, CapyError>;

/// Built-in capture types, in the order Go lists them.
pub const BUILTIN_TYPES: &[&str] = &[
    "any", "ident", "raw", "tail", "word", "dotted_ident", "string", "int", "float", "bool",
];

/// Port of `LoadLibraryFromBytes`.
///
/// Compiles an in-memory library written in Capy's native (`.capy`) syntax. The
/// `format` argument is reserved for future formats; today only "capy" (the
/// default) is supported.
pub fn load_library_from_bytes(
    format: &str,
    src: &[u8],
    tokenize: Tokenize,
) -> Result<Library, CapyError> {
    if !format.is_empty() && format.to_lowercase() != "capy" {
        return Err(CapyError::msg(format!(
            "unknown library format {} (only \"capy\" is supported)",
            gofmt::quote(format)
        )));
    }
    let raw = CapyLibParser.parse_bytes(src).map_err(CapyError::msg)?;
    map_library(raw, tokenize)
}

/// Port of the `MakeLibraryLoader(...).Load` closure.
pub fn load_library(path: &str, tokenize: Tokenize) -> Result<Library, CapyError> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let raw = load_raw_with_imports(path, &mut visited)?;
    map_library(raw, tokenize)
}

/// Port of `loadRawWithImports`.
///
/// Parses one library file and recursively pulls in any `import` directives,
/// merging them into the result. The IMPORTING file wins on conflict (so local
/// overrides shadow imports). Cycles error. Import paths are resolved relative to
/// the file containing the `import`.
fn load_raw_with_imports(
    path: &str,
    visited: &mut BTreeSet<String>,
) -> Result<RawLibrary, CapyError> {
    let abs = abs_path(path);
    if visited.contains(&abs) {
        return Err(CapyError::msg(format!("import cycle detected at {}", path)));
    }
    visited.insert(abs.clone());

    let raw = CapyLibParser.parse_file(path).map_err(CapyError::msg)?;

    if raw.imports.is_empty() {
        return Ok(raw);
    }

    // Start from a merged-imports base; the local raw overrides at the end.
    let dir = gopath::dir(&abs);
    let mut merged = RawLibrary::default();
    for imp in &raw.imports {
        let imp_path =
            if gopath::is_abs(imp) { imp.clone() } else { gopath::join(&[&dir, imp]) };
        let imp_raw = load_raw_with_imports(&imp_path, visited)
            .map_err(|e| CapyError::msg(format!("import {}: {}", gofmt::quote(imp), e)))?;
        merge_raw(&mut merged, imp_raw);
    }
    // Local raw wins on conflict.
    merge_raw(&mut merged, raw);
    Ok(merged)
}

/// Port of `filepath.Abs` for the cycle-detection key.
fn abs_path(path: &str) -> String {
    if gopath::is_abs(path) {
        return gopath::clean(path);
    }
    match std::env::current_dir() {
        Ok(cwd) => gopath::join(&[&cwd.to_string_lossy(), path]),
        Err(_) => gopath::clean(path),
    }
}

/// Port of `mergeRaw`.
///
/// Copies entries from `src` into `dst`. Existing keys in `dst` are OVERRIDDEN —
/// call order determines precedence (last-write-wins).
fn merge_raw(dst: &mut RawLibrary, src: RawLibrary) {
    if !src.extension.is_empty() {
        dst.extension = src.extension.clone();
    }
    if !src.output_file.is_empty() {
        dst.output_file = src.output_file.clone();
    }
    if !src.file_template.is_empty() {
        dst.file_template = src.file_template.clone();
    }
    for (k, v) in src.functions {
        dst.functions.insert(k, v);
    }
    for (k, v) in src.types {
        dst.types.insert(k, v);
    }
    for (k, v) in src.context {
        dst.context.insert(k, v);
    }
    for (k, v) in src.files {
        dst.files.insert(k, v);
    }
    // Preprocess directives are unioned; imports widen what the downstream
    // library can opt into without overriding.
    for d in src.preprocess {
        if !dst.preprocess.contains(&d) {
            dst.preprocess.push(d);
        }
    }
    // Comment markers are unioned, same rationale as preprocess.
    for c in src.comments {
        if !dst.comments.contains(&c) {
            dst.comments.push(c);
        }
    }
}

/// Port of `mapLibrary`'s `parseMaybeNew` closure.
///
/// Detects the new-shape sentinel that the `.capy` parser stashes on
/// `file_template` / `files` entries and parses the body into an AST the renderer
/// walks directly.
fn parse_maybe_new(
    s: &str,
    label: &str,
    tokenize: Tokenize,
) -> Result<Option<InnerBlock>, CapyError> {
    if !s.starts_with(NEW_SHAPE) {
        return Ok(None);
    }
    let body = &s[NEW_SHAPE.len()..];
    let toks = tokenize(body)
        .map_err(|e| CapyError::msg(format!("{}: parsing body: {}", label, e)))?;
    let ast =
        parse_inner(toks).map_err(|e| CapyError::msg(format!("{}: parsing body: {}", label, e)))?;
    // File-template / file blocks may not contain state-mutation statements (the
    // context is already finalised when they render; mutations would silently do
    // nothing). Reject early.
    let residual = translate_new_shape(&ast)
        .map_err(|e| CapyError::msg(format!("{}: {}", label, e)))?;
    if !residual.stmts.is_empty() {
        return Err(CapyError::msg(format!(
            "{}: state-mutation statements (set/append/…) aren't allowed here — file blocks only render",
            label
        )));
    }
    Ok(Some(ast))
}

/// Port of `mapLibrary`.
pub fn map_library(r: RawLibrary, tokenize: Tokenize) -> Result<Library, CapyError> {
    let ft_ast = parse_maybe_new(&r.file_template, "file_template", tokenize)?;
    let mut files_ast: BTreeMap<String, InnerBlock> = BTreeMap::new();
    for (path, body) in &r.files {
        let label = format!("file {}", gofmt::quote(path));
        if let Some(ast) = parse_maybe_new(body, &label, tokenize)? {
            files_ast.insert(path.clone(), ast);
        }
    }

    let mut lib = Library {
        extension: r.extension.clone(),
        output_file: r.output_file.clone(),
        description: r.description.clone(),
        functions: BTreeMap::new(),
        types: BTreeMap::new(),
        context: r.context.clone(),
        file_template_ast: ft_ast,
        files_ast,
        preprocess: r.preprocess.clone(),
        comments: r.comments.clone(),
        commands: BTreeMap::new(),
        lib_name: r.lib_name.clone(),
        lib_version: r.lib_version.clone(),
        impls: BTreeMap::new(),
        default_impl: r.default_impl.clone(),
        ..Default::default()
    };

    for (name, im) in &r.impls {
        lib.impls.insert(
            name.clone(),
            ImplDef {
                name: name.clone(),
                file: im.file.clone(),
                description: im.description.clone(),
                version: im.version.clone(),
                is_default: im.is_default,
            },
        );
    }

    for (name, t) in &r.types {
        // Group types are mutually exclusive with constraint types. Either you
        // declare a delimited capture (open + close) or a constraint
        // (base / pattern / options) — not both.
        let is_group = !t.group_open.is_empty() || !t.group_close.is_empty();
        if is_group {
            if t.group_open.is_empty() || t.group_close.is_empty() {
                return Err(CapyError::msg(format!(
                    "type {}: group_open and group_close must BOTH be set",
                    gofmt::quote(name)
                )));
            }
            if !t.base.is_empty() || !t.pattern.is_empty() || !t.options.is_empty() {
                return Err(CapyError::msg(format!(
                    "type {}: group types cannot also declare base/pattern/options",
                    gofmt::quote(name)
                )));
            }
        }
        lib.types.insert(
            name.clone(),
            TypeDef {
                name: name.clone(),
                description: t.description.clone(),
                base: t.base.clone(),
                pattern: t.pattern.clone(),
                options: t.options.clone(),
                group_open: t.group_open.clone(),
                group_close: t.group_close.clone(),
            },
        );
    }

    for (name, f) in &r.functions {
        let fd = compile_function(name, f, tokenize)?;
        lib.functions.insert(name.clone(), fd);
    }

    // Compile library commands (declared via `command "X" … end` blocks).
    for (name, c) in &r.commands {
        let mut cd = CommandDef {
            name: name.clone(),
            description: c.description.clone(),
            body_raw: c.body.clone(),
            ..Default::default()
        };
        for a in &c.args {
            cd.args.push(CommandArg {
                name: a.name.clone(),
                required: a.required,
                description: a.description.clone(),
            });
        }
        for f in &c.flags {
            cd.flags.push(CommandFlag {
                name: f.name.clone(),
                description: f.description.clone(),
                default: f.default.clone(),
                is_bool: f.is_bool,
            });
        }
        if !c.body.trim().is_empty() {
            let toks = tokenize(&c.body).map_err(|e| {
                CapyError::msg(format!(
                    "command {}: parsing body: {}",
                    gofmt::quote(name),
                    e
                ))
            })?;
            let ast = parse_inner(toks).map_err(|e| {
                CapyError::msg(format!(
                    "command {}: parsing body: {}",
                    gofmt::quote(name),
                    e
                ))
            })?;
            cd.body = ast;
        }
        lib.commands.insert(name.clone(), cd);
    }

    validate_cross_references(&mut lib)?;
    Ok(lib)
}

/// The post-load validation pass from the tail of `mapLibrary`.
fn validate_cross_references(lib: &mut Library) -> Result<(), CapyError> {
    let func_names: BTreeSet<String> = lib.functions.keys().cloned().collect();
    let type_names_sorted: Vec<String> = lib.types.keys().cloned().collect();

    for fd in lib.functions.values() {
        // Optional captures must be trailing: once an optional arg is declared,
        // every later arg must also be optional (otherwise the matcher couldn't
        // know whether a supplied value fills the optional or the
        // required-after-it).
        let mut seen_optional = false;
        for a in &fd.args {
            if a.kind != "capture" {
                if seen_optional {
                    return Err(CapyError::msg(format!(
                        "function {}: literal arg {} cannot follow an optional capture",
                        gofmt::quote(&fd.name),
                        gofmt::quote(&a.value)
                    )));
                }
                continue;
            }
            if a.optional {
                seen_optional = true;
            } else if seen_optional {
                return Err(CapyError::msg(format!(
                    "function {}: required capture {} cannot follow an optional capture (optional args must be trailing)",
                    gofmt::quote(&fd.name),
                    gofmt::quote(&a.name)
                )));
            }
        }
        for a in &fd.args {
            if a.kind != "capture" {
                continue;
            }
            // A capture's type may name another LIBRARY FUNCTION
            // (function-as-type / named nonterminal). That's valid even though
            // it's neither a built-in nor a declared `type`.
            if func_names.contains(&a.type_) {
                // `sep` / `join` only have meaning when the capture repeats.
                if a.repeat.is_empty() && !a.sep.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {}: capture {} uses `sep` but is not repeated (add `*` or `+`)",
                        gofmt::quote(&fd.name),
                        gofmt::quote(&a.name)
                    )));
                }
                if a.repeat.is_empty() && !a.join.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {}: capture {} uses `join` but is not repeated (add `*` or `+`)",
                        gofmt::quote(&fd.name),
                        gofmt::quote(&a.name)
                    )));
                }
                continue;
            }
            // Repetition (`type*` / `type+`) and `sep` only make sense for
            // function-typed captures.
            if !a.repeat.is_empty() {
                return Err(CapyError::msg(format!(
                    "function {}: capture {} uses a repetition suffix but its type {} is not a library function",
                    gofmt::quote(&fd.name), gofmt::quote(&a.name), gofmt::quote(&a.type_)
                )));
            }
            if !a.sep.is_empty() {
                return Err(CapyError::msg(format!(
                    "function {}: capture {} uses `sep` but its type {} is not a library function",
                    gofmt::quote(&fd.name), gofmt::quote(&a.name), gofmt::quote(&a.type_)
                )));
            }
            if !a.join.is_empty() {
                return Err(CapyError::msg(format!(
                    "function {}: capture {} uses `join` but its type {} is not a library function",
                    gofmt::quote(&fd.name), gofmt::quote(&a.name), gofmt::quote(&a.type_)
                )));
            }
            if !valid_type(&a.type_, &lib.types) {
                let mut ce = CapyError::structured(format!(
                    "function {}: capture {} has unknown type {}",
                    gofmt::quote(&fd.name),
                    gofmt::quote(&a.name),
                    gofmt::quote(&a.type_)
                ));
                // Suggest the closest known type (built-ins + library-declared).
                let mut cands: Vec<String> =
                    BUILTIN_TYPES.iter().map(|s| s.to_string()).collect();
                cands.extend(type_names_sorted.iter().cloned());
                cands.extend(func_names.iter().cloned());
                match suggest_closest(&a.type_, &cands, 2) {
                    Some(best) => {
                        ce.hint = format!("did you mean {}?", gofmt::quote(&best));
                    }
                    None => {
                        ce.hint = format!(
                            "built-in types: any, ident, raw, tail, word, dotted_ident, string, int, float, bool; declared types: [{}]",
                            type_names_sorted.join(" ")
                        );
                    }
                }
                return Err(ce);
            }
        }

        if let Some(block) = &fd.block {
            // Four modes (exactly one must be set):
            //   block_closer NAME      — keyword-closed, nested parsing
            //   block_open X close Y   — delimiter-pair, nested parsing
            //   block_dedent           — indent-closed, nested parsing
            //   block_verbatim NAME    — keyword-closed, body is raw bytes
            let has_closer = !block.closer.is_empty() && !block.is_verbatim;
            let has_delim = !block.open.is_empty() && !block.close.is_empty();
            let has_dedent = block.is_dedent;
            let has_verbatim = block.is_verbatim;
            let has_seq = !block.close_seq.is_empty();
            let modes = [has_closer, has_delim, has_dedent, has_verbatim, has_seq]
                .iter()
                .filter(|x| **x)
                .count();
            if modes != 1 {
                return Err(CapyError::msg(format!(
                    "function {}: block must set exactly one of block_closer, block_open/close, block_dedent, block_verbatim, or block_close_seq",
                    gofmt::quote(&fd.name)
                )));
            }
            // Both keyword-closed modes require the closer function to exist.
            if (has_closer || has_verbatim) && !block.closer.is_empty()
                && !func_names.contains(&block.closer)
            {
                return Err(CapyError::msg(format!(
                    "function {}: block.closer {} not found",
                    gofmt::quote(&fd.name),
                    gofmt::quote(&block.closer)
                )));
            }
            // Each block_close_seq ref segment must name a capture bound by this
            // function's opener.
            if has_seq {
                for seg in &block.close_seq {
                    if seg.ref_.is_empty() {
                        continue;
                    }
                    let found = fd
                        .args
                        .iter()
                        .any(|a| a.kind == "capture" && a.name == seg.ref_);
                    if !found {
                        return Err(CapyError::msg(format!(
                            "function {}: block_close_seq references unknown capture {}",
                            gofmt::quote(&fd.name),
                            gofmt::quote(&seg.ref_)
                        )));
                    }
                }
            }
        }
    }

    // Resolve `is_func` on compiled pattern elements now that all function names
    // are known: a capture whose type names a library function is a structural
    // (nonterminal) match, not a flat token.
    for fd in lib.functions.values_mut() {
        for el in fd.elements.iter_mut() {
            if el.is_capture && func_names.contains(&el.cap_type) {
                el.is_func = true;
            }
        }
    }

    // PLAN-2026-0001 R0b: reject left recursion here, while the function-as-type
    // edges have just been resolved and are all in hand.
    reject_left_recursion(lib)?;
    Ok(())
}

/// PLAN-2026-0001 R0/R0b — reject a left-recursive library at load time.
///
/// A function-as-type capture (`arg capture lhs expr`, where `expr` is another
/// library function) makes the matcher descend into that function. When the
/// descent happens before anything has been consumed, and the chain leads back
/// to where it started, the matcher recurses forever: the process dies with
/// `fatal runtime error: stack overflow` and **rc=134**, which an embedder
/// cannot catch as `Err`. `capy check` used to report `ok` on such a library,
/// so the author only found out at run time, in someone else's process.
///
/// A "left edge" `F -> G` exists when `F` can reach `G`'s match attempt without
/// consuming a token: walk `F`'s elements from the front, following any that may
/// match empty, and stop at the first element that must consume one.
fn reject_left_recursion(lib: &Library) -> Result<(), CapyError> {
    // Adjacency, built only from leading positions that consume nothing.
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (name, fd) in &lib.functions {
        let mut targets: Vec<&str> = Vec::new();
        for el in &fd.elements {
            if !el.is_capture {
                break; // a literal consumes a token — nothing past it is "left"
            }
            if el.is_func {
                targets.push(el.cap_type.as_str());
            }
            // Only a capture that MAY match empty leaves the position untouched
            // for the element after it. Anything else consumes, so stop.
            if !(el.repeat == "*" || el.optional) {
                break;
            }
        }
        edges.insert(name.as_str(), targets);
    }

    // Iterative DFS with an explicit stack: the whole point is not to recurse.
    // White/grey/black colouring — grey means "on the current path", so meeting
    // grey is the cycle.
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        White,
        Grey,
        Black,
    }
    let mut mark: BTreeMap<&str, Mark> = edges.keys().map(|k| (*k, Mark::White)).collect();

    for root in edges.keys() {
        if mark[root] != Mark::White {
            continue;
        }
        // (node, index of the next edge to try) plus the current path.
        let mut stack: Vec<(&str, usize)> = vec![(root, 0)];
        let mut path: Vec<&str> = vec![root];
        mark.insert(root, Mark::Grey);

        while let Some((node, edge_i)) = stack.pop() {
            let Some(next) = edges.get(node).and_then(|t| t.get(edge_i)) else {
                mark.insert(node, Mark::Black);
                path.pop();
                continue;
            };
            stack.push((node, edge_i + 1));
            match mark.get(next).copied() {
                // Edge into a function that is not declared: the existing
                // unknown-type check reports that; ignore it here.
                None => {}
                Some(Mark::Black) => {}
                Some(Mark::Grey) => {
                    // Cycle. Report it from where it closes, so the printed
                    // chain reads in the order the matcher would descend.
                    let start = path.iter().position(|n| n == next).unwrap_or(0);
                    let mut cycle: Vec<&str> = path[start..].to_vec();
                    cycle.push(next);
                    return Err(CapyError::msg(format!(
                        "function {}: left recursion — it can match itself without consuming a token (cycle: {}). \
Rewrite the rule so something is consumed first: put a literal before the capture, or make the recursion trail \
(right-recursive) instead of lead",
                        gofmt::quote(next),
                        cycle.join(" -> ")
                    )));
                }
                Some(Mark::White) => {
                    mark.insert(next, Mark::Grey);
                    path.push(next);
                    stack.push((next, 0));
                }
            }
        }
    }
    Ok(())
}

/// Port of `compileFunction`.
fn compile_function(
    name: &str,
    f: &RawFunction,
    tokenize: Tokenize,
) -> Result<FuncDef, CapyError> {
    let mut args = compile_args(&f.args, name)?;
    // Auto-name-prepend rule. The `bare` directive opts out — useful for
    // shape-only functions (e.g. a row of bare string literals) whose source has
    // no leading keyword to anchor on.
    let has_literal = args.iter().any(|a| a.kind == "literal");
    if !has_literal && !f.bare {
        let mut prefixed = vec![ArgEntry {
            kind: "literal".to_string(),
            value: name.to_string(),
            ..Default::default()
        }];
        prefixed.extend(args);
        args = prefixed;
    }

    let elements = compile_elements(&args);

    // New-shape body (unified write/state block) → translate into the equivalent
    // template AST + run AST before constructing the FuncDef.
    let mut run = String::new();
    let mut template_ast: Option<InnerBlock> = None;
    if !f.body.trim().is_empty() {
        let toks = tokenize(&f.body).map_err(|e| {
            CapyError::msg(format!(
                "function {}: parsing body: {}",
                gofmt::quote(name),
                e
            ))
        })?;
        let ast = parse_inner(toks).map_err(|e| {
            CapyError::msg(format!(
                "function {}: parsing body: {}",
                gofmt::quote(name),
                e
            ))
        })?;
        let run_ast = translate_new_shape(&ast)
            .map_err(|e| CapyError::msg(format!("function {}: {}", gofmt::quote(name), e)))?;
        // Stash the full AST for direct rendering. The renderer treats
        // state-mutation statements (set/append/…) as no-ops — those are handled
        // separately via the run AST below.
        template_ast = Some(ast);
        // Re-serialise the run AST as inner-DSL source text — the path below
        // re-parses it. (Slightly wasteful but keeps one code path.)
        run = render_inner_block(&run_ast);
    }

    let mut fd = FuncDef {
        name: name.to_string(),
        description: f.description.clone(),
        args,
        elements,
        template_ast,
        priority: f.priority,
        ..Default::default()
    };

    if let Some(rb) = &f.block {
        let mut bs = BlockSpec {
            closer: rb.closer.clone(),
            open: rb.open.clone(),
            close: rb.close.clone(),
            is_dedent: rb.is_dedent,
            is_verbatim: rb.is_verbatim,
            sections: rb.sections.clone(),
            ..Default::default()
        };
        if !rb.close_seq.is_empty() {
            let mut segs: Vec<CloseSegment> = Vec::new();
            for raw in &rb.close_seq {
                if raw.is_ref {
                    // A bare capture-name reference; resolved against the
                    // opener's bound captures at parse time.
                    segs.push(CloseSegment { tokens: Vec::new(), ref_: raw.text.clone() });
                    continue;
                }
                // A fixed literal — pre-tokenize it the way the user-script lexer
                // will (e.g. "</div>" → ["</", "div", ">"]).
                let toks = lexeme_texts(&raw.text, tokenize).map_err(|e| {
                    CapyError::msg(format!(
                        "function {}: block_close_seq: {}",
                        gofmt::quote(name),
                        e
                    ))
                })?;
                if toks.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {}: block_close_seq segment {} lexes to no tokens",
                        gofmt::quote(name),
                        gofmt::quote(&raw.text)
                    )));
                }
                segs.push(CloseSegment { tokens: toks, ref_: String::new() });
            }
            bs.close_seq = segs;
        }
        fd.block = Some(bs);
    }

    if f.followed_by_indent && f.not_followed_by_indent {
        return Err(CapyError::msg(format!(
            "function {}: cannot set both when_followed_by and when_not_followed_by",
            gofmt::quote(name)
        )));
    }
    if f.followed_by_indent || f.not_followed_by_indent {
        fd.lookahead = Some(Lookahead {
            require_indent: f.followed_by_indent,
            forbid_indent: f.not_followed_by_indent,
        });
    }

    if !run.trim().is_empty() {
        let toks = tokenize(&run).map_err(|e| {
            CapyError::msg(format!("function {}: parsing run: {}", gofmt::quote(name), e))
        })?;
        let ast = parse_inner(toks).map_err(|e| {
            CapyError::msg(format!("function {}: parsing run: {}", gofmt::quote(name), e))
        })?;
        fd.run_ast = Some(ast);
    }

    Ok(fd)
}

/// Port of `lexemeTexts`.
///
/// Tokenizes `src` and returns the text of each lexeme token, skipping structural
/// tokens (NEWLINE / INDENT / DEDENT / EOF). Used to pre-compute a multi-token
/// block closer sequence (e.g. `"</div>"` → `["</", "div", ">"]`) at
/// library-load time.
fn lexeme_texts(src: &str, tokenize: Tokenize) -> Result<Vec<String>, CapyError> {
    let toks = tokenize(src)?;
    let mut out: Vec<String> = Vec::new();
    for t in toks {
        match t.kind {
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => continue,
            _ => out.push(t.text),
        }
    }
    Ok(out)
}

/// Port of `compileArgs`.
fn compile_args(raws: &[RawArg], fname: &str) -> Result<Vec<ArgEntry>, CapyError> {
    let mut out: Vec<ArgEntry> = Vec::new();
    for (i, r) in raws.iter().enumerate() {
        match r.kind.as_str() {
            "literal" => {
                if r.value.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {} arg {}: kind=literal requires value",
                        gofmt::quote(fname),
                        i
                    )));
                }
                if !r.name.is_empty() || !r.type_.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {} arg {}: kind=literal cannot have name/type",
                        gofmt::quote(fname),
                        i
                    )));
                }
                out.push(ArgEntry {
                    kind: "literal".to_string(),
                    value: r.value.clone(),
                    description: r.description.clone(),
                    ..Default::default()
                });
            }
            "capture" => {
                if r.name.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {} arg {}: kind=capture requires name",
                        gofmt::quote(fname),
                        i
                    )));
                }
                if !r.value.is_empty() {
                    return Err(CapyError::msg(format!(
                        "function {} arg {}: kind=capture cannot have value",
                        gofmt::quote(fname),
                        i
                    )));
                }
                let t = if r.type_.is_empty() { "any".to_string() } else { r.type_.clone() };
                out.push(ArgEntry {
                    kind: "capture".to_string(),
                    name: r.name.clone(),
                    type_: t,
                    description: r.description.clone(),
                    optional: r.optional,
                    default: r.default.clone(),
                    repeat: r.repeat.clone(),
                    sep: r.sep.clone(),
                    join: r.join.clone(),
                    ..Default::default()
                });
            }
            other => {
                return Err(CapyError::msg(format!(
                    "function {} arg {}: unknown or missing kind {} (must be \"literal\" or \"capture\")",
                    gofmt::quote(fname), i, gofmt::quote(other)
                )));
            }
        }
    }
    Ok(out)
}

/// Port of `compileElements`.
fn compile_elements(args: &[ArgEntry]) -> Vec<PatternElement> {
    let mut out: Vec<PatternElement> = Vec::new();
    for a in args {
        if a.kind == "literal" {
            // Split on `.` so dotted names (scene.create_sphere) match how the
            // outer lexer tokenises them.
            out.extend(split_literal(&a.value));
        } else {
            out.push(PatternElement {
                is_capture: true,
                name: a.name.clone(),
                cap_type: a.type_.clone(),
                optional: a.optional,
                default: a.default.clone(),
                repeat: a.repeat.clone(),
                sep: a.sep.clone(),
                join: a.join.clone(),
                ..Default::default()
            });
        }
    }
    out
}

/// Port of `splitLiteral`.
///
/// Converts `"scene.create_sphere"` into 3 pattern elements; non-dotted literals
/// (including multi-char operators like `:=`, `->`) pass through whole.
fn split_literal(lit: &str) -> Vec<PatternElement> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for c in lit.bytes() {
        if c == b'.' {
            if !cur.is_empty() {
                parts.push(String::from_utf8_lossy(&cur).into_owned());
                cur.clear();
            }
            parts.push(".".to_string());
            continue;
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        parts.push(String::from_utf8_lossy(&cur).into_owned());
    }
    parts
        .into_iter()
        .map(|p| PatternElement { literal: p, ..Default::default() })
        .collect()
}

/// Port of `validType`.
fn valid_type(t: &str, types: &BTreeMap<String, TypeDef>) -> bool {
    if BUILTIN_TYPES.contains(&t) {
        return true;
    }
    types.contains_key(t)
}
