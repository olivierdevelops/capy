//! Port of `infra/capy_lib_parser.go`.
//!
//! Reads a library written in Capy's native syntax (a `.capy` library file) and
//! produces a [`RawLibrary`] DTO.
//!
//! Surface grammar (indentation-sensitive, line-based):
//!
//! ```text
//! extension <STR>
//! output_file <STR>
//!
//! function <NAME>
//!     priority <INT>
//!     arg literal <STR>
//!     arg capture <NAME> <TYPE>
//!     block_closer <NAME>
//!     block_open <STR> close <STR>
//!     <inner-DSL body statements>
//! end
//!
//! file_template
//!     ...
//! end
//! ```
//!
//! Strings use double quotes with Go-style escapes (`\n \t \" \\`). Bare words
//! are accepted for `extension`, `output_file`, capture types, and names.

use super::raw_library::*;
use crate::domain::val::Val;
use crate::gofmt;
use crate::gopath;
use std::collections::BTreeMap;
use std::fs;

/// The sentinel the parser stamps on new-shape bodies so the loader knows to
/// route them through the inner-DSL parser.
pub const NEW_SHAPE: &str = "\u{0}NEW_SHAPE\u{0}";

#[derive(Debug, Clone, Copy, Default)]
pub struct CapyLibParser;

impl CapyLibParser {
    pub fn parse_file(&self, path: &str) -> Result<RawLibrary, String> {
        let b = fs::read_to_string(path).map_err(|e| gopath::io_error("open", path, &e))?;
        parse_capy_lib(&b)
    }

    /// Parses a Capy-native library directly from in-memory bytes. Used by the
    /// embedding API.
    pub fn parse_bytes(&self, b: &[u8]) -> Result<RawLibrary, String> {
        parse_capy_lib(&String::from_utf8_lossy(b))
    }

    /// The ergonomic in-memory entry point.
    pub fn parse_string(&self, s: &str) -> Result<RawLibrary, String> {
        parse_capy_lib(s)
    }
}

/// Port of `parseCapyLib`.
pub fn parse_capy_lib(src: &str) -> Result<RawLibrary, String> {
    let normalised = src.replace("\r\n", "\n");
    let lines: Vec<String> = normalised.split('\n').map(|s| s.to_string()).collect();
    let mut p = CapyLibParserState { lines, line_no: 0 };
    p.parse_top()
}

// --- template-block sugar ---------------------------------------------------

/// Port of `rewriteTemplateBlocks`.
///
/// Transforms the `template … end` sugar into the equivalent multi-line
/// `` write `…` `` literal. Pure syntactic rewrite: the output bytes are what a
/// library author would have written by hand, so the downstream merge → dedent →
/// inner-DSL parse → render pipeline doesn't need to know this sugar exists.
///
/// `base_line_no` is the source line number of `raws[0]` — used to make
/// `missing end` errors point at the right line.
// Indexing is load-bearing here: the scan jumps over `\X` escape pairs and
// re-enters at a computed offset.
#[allow(clippy::needless_range_loop)]
fn rewrite_template_blocks(raws: &[String], base_line_no: usize) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_backtick = false;
    let mut i = 0usize;
    while i < raws.len() {
        let ln = &raws[i];
        if !in_backtick && ln.trim() == "template" {
            let open_indent = leading_whitespace(ln);
            // Walk forward until `end` at the same indent, with depth tracking
            // for nested `template … end` blocks AND backtick state so a literal
            // `template`/`end` inside an existing `` write `…` `` doesn't trigger.
            let mut depth = 1i64;
            let mut body_end: i64 = -1;
            let mut inner_bt = false;
            for j in (i + 1)..raws.len() {
                let bl = &raws[j];
                if inner_bt {
                    inner_bt = update_backtick_state(bl, inner_bt);
                    continue;
                }
                let bs = bl.trim();
                let bli = leading_whitespace(bl);
                if bs == "template" && bli == open_indent {
                    depth += 1;
                } else if bs == "end" && bli == open_indent {
                    depth -= 1;
                    if depth == 0 {
                        body_end = j as i64;
                        break;
                    }
                }
                inner_bt = update_backtick_state(bl, inner_bt);
            }
            if body_end < 0 {
                return Err(format!(
                    "line {}: template block has no matching `end` at the same indent",
                    base_line_no + i + 1
                ));
            }
            let body = &raws[i + 1..body_end as usize];
            // Auto-dedent: strip the smallest non-blank leading indent.
            let mut body_min: i64 = -1;
            for bl in body {
                if bl.trim().is_empty() {
                    continue;
                }
                let bi = leading_whitespace(bl) as i64;
                if body_min < 0 || bi < body_min {
                    body_min = bi;
                }
            }
            if body_min < 0 {
                body_min = 0;
            }
            let body_min = body_min as usize;
            let dedented: Vec<String> = body
                .iter()
                .map(|bl| {
                    if bl.len() >= body_min {
                        escape_for_backtick(&bl[body_min..])
                    } else {
                        escape_for_backtick(bl)
                    }
                })
                .collect();
            // Emit the synth at the opener's indent. The opening backtick goes
            // immediately after `write ` on the first line; body lines flow
            // flush-left; the closing backtick sits on its own flush-left line.
            let prefix = " ".repeat(open_indent);
            if dedented.is_empty() {
                out.push(format!("{}write ``", prefix));
            } else {
                out.push(format!("{}write `{}", prefix, dedented[0]));
                for bl in &dedented[1..] {
                    out.push(bl.clone());
                }
                out.push("`".to_string());
            }
            i = body_end as usize + 1;
            continue;
        }
        out.push(ln.clone());
        in_backtick = update_backtick_state(ln, in_backtick);
        i += 1;
    }
    Ok(out)
}

/// Port of `leadingWhitespace` — count of leading space-or-tab BYTES. Used purely
/// for indent-equality comparisons; mixed tabs/spaces are the author's
/// responsibility (matches the rest of the lib parser).
fn leading_whitespace(s: &str) -> usize {
    let b = s.as_bytes();
    let mut n = 0usize;
    while n < b.len() && (b[n] == b' ' || b[n] == b'\t') {
        n += 1;
    }
    n
}

/// Port of `updateBacktickState` — toggles on each unescaped backtick.
fn update_backtick_state(ln: &str, mut in_backtick: bool) -> bool {
    let b = ln.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            i += 2;
            continue;
        }
        if c == b'`' {
            in_backtick = !in_backtick;
        }
        i += 1;
    }
    in_backtick
}

/// Port of `escapeForBacktick`.
///
/// Prepares one body line for embedding inside a backtick literal: backslashes
/// double, backticks gain a backslash. `${…}` interpolation markers pass through
/// untouched — they're what makes this sugar useful.
fn escape_for_backtick(s: &str) -> String {
    if !s.contains('`') && !s.contains('\\') {
        return s.to_string();
    }
    let mut b = String::with_capacity(s.len() + 4);
    for c in s.bytes() {
        match c {
            b'\\' => b.push_str("\\\\"),
            b'`' => b.push_str("\\`"),
            _ => unsafe { b.as_mut_vec().push(c) },
        }
    }
    b
}

/// Port of `mergeBackticksInLines`.
///
/// Collapses multi-line backtick literals into single logical lines (with
/// embedded newlines escaped as `\n` so the line-based parser doesn't split on
/// them). Only applied to the slice of lines belonging to a new-shape function
/// body — running it globally would eat backticks inside raw-text blocks.
fn merge_backticks_in_lines(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_backtick = false;
    for ln in lines {
        if in_backtick {
            // Carrying an open backtick from a previous line: re-attach this
            // line with a literal `\n` separator and continue scanning.
            cur.push_str("\\n");
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        let b = ln.as_bytes();
        let mut i = 0usize;
        while i < b.len() {
            let c = b[i];
            if c == b'\\' && i + 1 < b.len() && in_backtick {
                unsafe {
                    let v = cur.as_mut_vec();
                    v.push(c);
                    v.push(b[i + 1]);
                }
                i += 2;
                continue;
            }
            if c == b'`' {
                in_backtick = !in_backtick;
            }
            unsafe { cur.as_mut_vec().push(c) };
            i += 1;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// --- the parser state machine ----------------------------------------------

struct CapyLibParserState {
    lines: Vec<String>,
    line_no: usize,
}

impl CapyLibParserState {
    /// Port of `errf` — prefixes `line N: ` using the current cursor.
    fn errf(&self, msg: impl AsRef<str>) -> String {
        format!("line {}: {}", self.line_no, msg.as_ref())
    }

    /// Port of `peekLine` — next non-blank, non-comment line WITHOUT advancing.
    /// A comment is `#` as the first non-space char.
    fn peek_line(&self) -> Option<(String, usize)> {
        for i in self.line_no..self.lines.len() {
            let raw = &self.lines[i];
            let stripped = raw.trim();
            if stripped.is_empty() || stripped.starts_with('#') {
                continue;
            }
            return Some((raw.clone(), indent_of(raw)));
        }
        None
    }

    /// Port of `nextLine` — next non-blank, non-comment line, advancing past it.
    fn next_line(&mut self) -> Option<(String, usize)> {
        while self.line_no < self.lines.len() {
            let raw = self.lines[self.line_no].clone();
            self.line_no += 1;
            let stripped = raw.trim();
            if stripped.is_empty() || stripped.starts_with('#') {
                continue;
            }
            let ind = indent_of(&raw);
            return Some((raw, ind));
        }
        None
    }

    /// Port of `parseTop`.
    fn parse_top(&mut self) -> Result<RawLibrary, String> {
        let mut lib = RawLibrary::default();

        loop {
            let (line, indent) = match self.peek_line() {
                None => break,
                Some(x) => x,
            };
            if indent != 0 {
                return Err(self.errf(format!(
                    "unexpected indentation at top level: {}",
                    gofmt::quote(line.trim())
                )));
            }
            let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
            if tokens.is_empty() {
                self.next_line();
                continue;
            }
            match tokens[0].as_str() {
                "name" => {
                    // `name "STR"` — manifest field.
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("name requires a string value"));
                    }
                    lib.lib_name = tokens[1].clone();
                }
                "version" => {
                    // `version "STR"` — manifest field (semver string).
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("version requires a string value"));
                    }
                    lib.lib_version = tokens[1].clone();
                }
                "command" => {
                    // `command "NAME" … end` block.
                    if tokens.len() < 2 {
                        return Err(self.errf("command requires a name string"));
                    }
                    let cmd_name = tokens[1].clone();
                    self.next_line();
                    let cmd = self.parse_command_block()?;
                    lib.commands.insert(cmd_name, cmd);
                }
                "impl" => {
                    // `impl "NAME" "FILE" … end`.
                    if tokens.len() < 3 {
                        return Err(self.errf(
                            "impl requires name + file string, e.g. `impl \"d3\" \"impl/d3.capy\"`",
                        ));
                    }
                    let name = tokens[1].clone();
                    let file = tokens[2].clone();
                    self.next_line();
                    let mut ri = self.parse_impl_block()?;
                    ri.name = name.clone();
                    ri.file = file;
                    let is_default = ri.is_default;
                    lib.impls.insert(name.clone(), ri);
                    if is_default {
                        lib.default_impl = name;
                    }
                }
                "default_impl" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("default_impl requires a string name"));
                    }
                    lib.default_impl = tokens[1].clone();
                }
                "extension" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("extension requires a value"));
                    }
                    lib.extension = tokens[1].clone();
                }
                "output_file" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("output_file requires a value"));
                    }
                    lib.output_file = tokens[1].clone();
                }
                "description" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("description requires a string"));
                    }
                    lib.description = tokens[1].clone();
                }
                "function" => {
                    if tokens.len() < 2 {
                        return Err(self.errf("function requires a name"));
                    }
                    let name = tokens[1].clone();
                    self.next_line();
                    let fnv = self.parse_function()?;
                    lib.functions.insert(name, fnv);
                }
                "type" => {
                    if tokens.len() < 2 {
                        return Err(self.errf("type requires a name"));
                    }
                    let name = tokens[1].clone();
                    self.next_line();
                    let td = self.parse_type()?;
                    lib.types.insert(name, td);
                }
                "context" => {
                    self.next_line();
                    let mut ctx = std::mem::take(&mut lib.context);
                    let r = self.parse_context(&mut ctx);
                    lib.context = ctx;
                    r?;
                }
                "import" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("import requires one path string"));
                    }
                    lib.imports.push(tokens[1].clone());
                }
                "preprocess" => {
                    // Each `include "X"` line opts the library into recognising X
                    // as a text-level inclusion directive at the start of a
                    // source line. With no `preprocess` block, NO directive works
                    // — Capy stays true to "zero predefined grammar".
                    self.next_line();
                    loop {
                        let (ln, indent) = match self.peek_line() {
                            None => return Err(self.errf("unexpected EOF inside preprocess")),
                            Some(x) => x,
                        };
                        let toks = tokenize_lib_line(&ln).map_err(|e| self.errf(e))?;
                        // `end` at indent 0 closes the block.
                        if indent == 0 && toks.len() == 1 && toks[0] == "end" {
                            self.next_line();
                            break;
                        }
                        if indent == 0 {
                            return Err(self.errf(format!(
                                "expected `end` to close preprocess, got {}",
                                gofmt::quote(ln.trim())
                            )));
                        }
                        if toks.is_empty() {
                            self.next_line();
                            continue;
                        }
                        if toks.len() == 2 && toks[0] == "include" {
                            lib.preprocess.push(toks[1].clone());
                            self.next_line();
                            continue;
                        }
                        return Err(self.errf(format!(
                            "inside preprocess: expected `include \"@NAME\"` or `end`, got {}",
                            gofmt::quote(ln.trim())
                        )));
                    }
                }
                "comments" => {
                    // Each `line "MARKER"` opts user-script lexing into
                    // recognising MARKER as a line comment. Empty block → no
                    // comment syntax (the default).
                    self.next_line();
                    loop {
                        let (ln, indent) = match self.peek_line() {
                            None => return Err(self.errf("unexpected EOF inside comments")),
                            Some(x) => x,
                        };
                        let toks = tokenize_lib_line(&ln).map_err(|e| self.errf(e))?;
                        if indent == 0 && toks.len() == 1 && toks[0] == "end" {
                            self.next_line();
                            break;
                        }
                        if indent == 0 {
                            return Err(self.errf(format!(
                                "expected `end` to close comments, got {}",
                                gofmt::quote(ln.trim())
                            )));
                        }
                        if toks.is_empty() {
                            self.next_line();
                            continue;
                        }
                        if toks.len() == 2 && toks[0] == "line" {
                            lib.comments.push(toks[1].clone());
                            self.next_line();
                            continue;
                        }
                        return Err(self.errf(format!(
                            "inside comments: expected `line \"MARKER\"` or `end`, got {}",
                            gofmt::quote(ln.trim())
                        )));
                    }
                }
                "file" => {
                    // `file "path" … end` — multi-file output. The body is an
                    // inner-DSL write-style block.
                    //
                    // NOTE: Go indexes tokens[1] with no length check and would
                    // panic on a bare `file`; this reports an error instead.
                    if tokens.len() < 2 {
                        return Err(self.errf("file requires a path string"));
                    }
                    let path = tokens[1].clone();
                    self.next_line();
                    let body = self.parse_function_body_statements(0)?;
                    // Consume the closing `end` line.
                    if let Some((ln, indent)) = self.peek_line() {
                        if indent == 0 {
                            let toks = tokenize_lib_line(&ln).unwrap_or_default();
                            if toks.len() == 1 && toks[0] == "end" {
                                self.next_line();
                            }
                        }
                    }
                    // Stash the raw body with the sentinel prefix.
                    lib.files.insert(path, format!("{}{}", NEW_SHAPE, body));
                }
                "file_template" => {
                    // New shape: `file_template … end` with an inner-DSL body.
                    self.next_line();
                    let body = self.parse_function_body_statements(0)?;
                    if let Some((ln, indent)) = self.peek_line() {
                        if indent == 0 {
                            let toks = tokenize_lib_line(&ln).unwrap_or_default();
                            if toks.len() == 1 && toks[0] == "end" {
                                self.next_line();
                            }
                        }
                    }
                    lib.file_template = format!("{}{}", NEW_SHAPE, body);
                }
                other => {
                    return Err(
                        self.errf(format!("unknown top-level directive: {}", gofmt::quote(other)))
                    );
                }
            }
        }
        Ok(lib)
    }

    /// Port of `parseFunction` — reads everything until a matching `end` at
    /// indent 0. The `function NAME` line has already been consumed.
    fn parse_function(&mut self) -> Result<RawFunction, String> {
        let mut fnv = RawFunction::default();
        let mut block = RawBlock::default();
        let mut block_set = false;

        loop {
            let (line, indent) = match self.peek_line() {
                None => return Err(self.errf("unexpected EOF inside function")),
                Some(x) => x,
            };
            if indent == 0 {
                // Must be the closing `end`.
                let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
                if tokens.len() == 1 && tokens[0] == "end" {
                    self.next_line();
                    if block_set {
                        fnv.block = Some(block);
                    }
                    return Ok(fnv);
                }
                return Err(self.errf(format!(
                    "expected `end` to close function, got {}",
                    gofmt::quote(line.trim())
                )));
            }

            let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
            if tokens.is_empty() {
                self.next_line();
                continue;
            }

            match tokens[0].as_str() {
                "description" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("description requires one string argument"));
                    }
                    fnv.description = tokens[1].clone();
                }
                "priority" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("priority requires one integer argument"));
                    }
                    match tokens[1].parse::<i64>() {
                        Ok(n) => fnv.priority = n,
                        Err(_) => {
                            // Mirrors Go's `strconv.Atoi` error text.
                            return Err(self.errf(format!(
                                "priority: strconv.Atoi: parsing {}: invalid syntax",
                                gofmt::quote(&tokens[1])
                            )));
                        }
                    }
                }
                "bare" => {
                    self.next_line();
                    if tokens.len() != 1 {
                        return Err(self.errf("bare takes no arguments"));
                    }
                    fnv.bare = true;
                }
                "when_followed_by" => {
                    self.next_line();
                    if tokens.len() != 2 || tokens[1] != "indent" {
                        return Err(self.errf("when_followed_by currently supports only `indent` (e.g. `when_followed_by indent`)"));
                    }
                    fnv.followed_by_indent = true;
                }
                "when_not_followed_by" => {
                    self.next_line();
                    if tokens.len() != 2 || tokens[1] != "indent" {
                        return Err(self.errf("when_not_followed_by currently supports only `indent` (e.g. `when_not_followed_by indent`)"));
                    }
                    fnv.not_followed_by_indent = true;
                }
                "arg" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("arg requires a kind"));
                    }
                    match tokens[1].as_str() {
                        "literal" => {
                            // `arg literal "TEXT"` or with a description.
                            if tokens.len() < 3 || tokens.len() > 4 {
                                return Err(self.errf(
                                    "arg literal requires a value (and optional description string)",
                                ));
                            }
                            let mut ra = RawArg {
                                kind: "literal".to_string(),
                                value: tokens[2].clone(),
                                ..Default::default()
                            };
                            if tokens.len() == 4 {
                                ra.description = tokens[3].clone();
                            }
                            fnv.args.push(ra);
                        }
                        "capture" => {
                            // `arg capture NAME [TYPE] [sep "X"] [join "X"]
                            //  [default "VALUE"] [DESCRIPTION]`
                            if tokens.len() < 3 {
                                return Err(self.errf(
                                    "arg capture NAME [TYPE] [default \"VALUE\"] [DESCRIPTION]",
                                ));
                            }
                            let mut a = RawArg {
                                kind: "capture".to_string(),
                                name: tokens[2].clone(),
                                type_: "any".to_string(),
                                ..Default::default()
                            };
                            let rest = &tokens[3..];
                            let mut idx = 0usize;
                            // Optional TYPE, with an optional repetition suffix
                            // (`type*` / `type+`) for function-typed captures.
                            if idx < rest.len()
                                && rest[idx] != "default"
                                && rest[idx] != "sep"
                                && rest[idx] != "join"
                            {
                                let mut typ = rest[idx].clone();
                                if typ.ends_with('*') {
                                    a.repeat = "*".to_string();
                                    typ = typ.trim_end_matches('*').to_string();
                                } else if typ.ends_with('+') {
                                    a.repeat = "+".to_string();
                                    typ = typ.trim_end_matches('+').to_string();
                                }
                                if is_ident(&typ) {
                                    a.type_ = typ;
                                    idx += 1;
                                } else if !a.repeat.is_empty() {
                                    return Err(self.errf(format!(
                                        "arg capture: repetition suffix requires a function/type name before {}",
                                        gofmt::quote(&rest[idx])
                                    )));
                                }
                            }
                            // Optional `sep "VALUE"`.
                            if idx < rest.len() && rest[idx] == "sep" {
                                if idx + 1 >= rest.len() {
                                    return Err(self.errf("arg capture: `sep` requires a value"));
                                }
                                a.sep = rest[idx + 1].clone();
                                idx += 2;
                            }
                            // Optional `join "VALUE"`.
                            if idx < rest.len() && rest[idx] == "join" {
                                if idx + 1 >= rest.len() {
                                    return Err(self.errf("arg capture: `join` requires a value"));
                                }
                                a.join = rest[idx + 1].clone();
                                idx += 2;
                            }
                            // Optional `default "VALUE"` — marks the arg optional.
                            if idx < rest.len() && rest[idx] == "default" {
                                if idx + 1 >= rest.len() {
                                    return Err(
                                        self.errf("arg capture: `default` requires a value")
                                    );
                                }
                                a.optional = true;
                                a.default = rest[idx + 1].clone();
                                idx += 2;
                            }
                            // Optional trailing DESCRIPTION (one token).
                            if idx < rest.len() {
                                a.description = rest[idx].clone();
                                idx += 1;
                            }
                            if idx < rest.len() {
                                return Err(self.errf("arg capture: unexpected extra tokens after NAME [TYPE] [default \"VALUE\"] [DESCRIPTION]"));
                            }
                            fnv.args.push(a);
                        }
                        other => {
                            return Err(
                                self.errf(format!("unknown arg kind {}", gofmt::quote(other)))
                            );
                        }
                    }
                }
                "block_closer" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("block_closer requires a function name"));
                    }
                    block.closer = tokens[1].clone();
                    block_set = true;
                }
                "block_dedent" => {
                    self.next_line();
                    if tokens.len() != 1 {
                        return Err(self.errf("block_dedent takes no arguments"));
                    }
                    block.is_dedent = true;
                    block_set = true;
                }
                "block_verbatim" => {
                    // Body is captured as raw source bytes (no nested parsing)
                    // until the named closer keyword at the parent indent.
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("block_verbatim requires a closer-function name (e.g. `block_verbatim end`)"));
                    }
                    block.closer = tokens[1].clone();
                    block.is_verbatim = true;
                    block_set = true;
                }
                "block_sections" => {
                    // `block_sections SECTION… closer CLOSER` — a multi-section
                    // block (try/rescue/finally).
                    self.next_line();
                    if tokens.len() < 4 || tokens[tokens.len() - 2] != "closer" {
                        return Err(self.errf("block_sections SECTION... closer CLOSER (e.g. `block_sections rescue finally closer end`)"));
                    }
                    block.closer = tokens[tokens.len() - 1].clone();
                    block.sections = tokens[1..tokens.len() - 2].to_vec();
                    block_set = true;
                }
                "block_open" => {
                    self.next_line();
                    if tokens.len() != 4 || tokens[2] != "close" {
                        return Err(self.errf("block_open <OPEN> close <CLOSE>"));
                    }
                    block.open = tokens[1].clone();
                    block.close = tokens[3].clone();
                    block_set = true;
                }
                "block_close_seq" => {
                    // Quoted segments are literals; bare identifiers reference
                    // captures bound by the opener, so the closer can depend on
                    // the opener (matched-pair HTML, generic over tag name).
                    let segs = parse_close_seq_segs(&line).map_err(|e| self.errf(e))?;
                    if segs.is_empty() {
                        return Err(self.errf("block_close_seq requires a closing sequence (e.g. `block_close_seq \"</div>\"` or `block_close_seq \"</\" name \">\"`)"));
                    }
                    self.next_line();
                    block.close_seq = segs;
                    block_set = true;
                }
                _ => {
                    // New-shape body: anything that isn't a recognised header
                    // directive is taken to be an inner-DSL statement.
                    let body = self.parse_function_body_statements(indent)?;
                    fnv.body = body;
                }
            }
        }
    }

    /// Port of `parseImplBlock`.
    fn parse_impl_block(&mut self) -> Result<RawImpl, String> {
        let mut ri = RawImpl::default();
        loop {
            let (ln, indent) = match self.peek_line() {
                None => return Err(self.errf("unexpected EOF inside impl")),
                Some(x) => x,
            };
            let toks = tokenize_lib_line(&ln).map_err(|e| self.errf(e))?;
            if indent == 0 && toks.len() == 1 && toks[0] == "end" {
                self.next_line();
                break;
            }
            if indent == 0 {
                return Err(self.errf(format!(
                    "expected `end` to close impl, got {}",
                    gofmt::quote(ln.trim())
                )));
            }
            if toks.is_empty() {
                self.next_line();
                continue;
            }
            match toks[0].as_str() {
                "description" => {
                    if toks.len() < 2 {
                        return Err(self.errf("description requires a string value"));
                    }
                    ri.description = toks[1].clone();
                }
                "version" => {
                    if toks.len() < 2 {
                        return Err(self.errf("version requires a string value"));
                    }
                    ri.version = toks[1].clone();
                }
                "default" => ri.is_default = true,
                other => {
                    return Err(self.errf(format!(
                        "unknown directive inside impl: {} (allowed: description / version / default)",
                        gofmt::quote(other)
                    )));
                }
            }
            self.next_line();
        }
        Ok(ri)
    }

    /// Port of `parseCommandBlock`.
    fn parse_command_block(&mut self) -> Result<RawCommand, String> {
        let mut cmd = RawCommand::default();
        // First pass: collect optional header directives until a non-header
        // statement, then everything after is the body.
        let mut header_done = false;
        let mut body_raws: Vec<String> = Vec::new();
        let mut in_backtick = false;
        loop {
            if self.line_no >= self.lines.len() {
                return Err(self.errf("unexpected EOF inside command"));
            }
            let ln = self.lines[self.line_no].clone();
            // While inside a multi-line backtick, every line belongs to the body
            // regardless of indent.
            if in_backtick {
                body_raws.push(ln.clone());
                self.line_no += 1;
                in_backtick = update_backtick_state(&ln, in_backtick);
                continue;
            }
            let stripped = ln.trim();
            if stripped.is_empty() || stripped.starts_with('#') {
                if header_done {
                    body_raws.push(ln.clone());
                }
                self.line_no += 1;
                continue;
            }
            let leading = leading_whitespace(&ln);
            if leading == 0 {
                // indent 0 — should be the closing `end`.
                let toks = tokenize_lib_line(&ln).map_err(|e| self.errf(e))?;
                if toks.len() == 1 && toks[0] == "end" {
                    self.next_line();
                    break;
                }
                return Err(self.errf(format!(
                    "expected `end` to close command, got {}",
                    gofmt::quote(stripped)
                )));
            }
            // Indented line. Check for a recognised header directive.
            let toks = tokenize_lib_line(&ln).map_err(|e| self.errf(e))?;
            if !header_done && !toks.is_empty() {
                match toks[0].as_str() {
                    "description" => {
                        if toks.len() < 2 {
                            return Err(self.errf("description requires a string value"));
                        }
                        cmd.description = toks[1].clone();
                        self.next_line();
                        continue;
                    }
                    "arg" => {
                        if toks.len() < 2 {
                            return Err(self.errf("arg requires a name string"));
                        }
                        let mut ra = RawCommandArg { name: toks[1].clone(), ..Default::default() };
                        if toks.len() >= 3 {
                            match toks[2].as_str() {
                                "required" => ra.required = true,
                                "optional" => ra.required = false,
                                other => {
                                    return Err(self.errf(format!(
                                        "arg: expected `required` or `optional` after name, got {}",
                                        gofmt::quote(other)
                                    )));
                                }
                            }
                        }
                        if toks.len() >= 4 {
                            ra.description = toks[3].clone();
                        }
                        cmd.args.push(ra);
                        self.next_line();
                        continue;
                    }
                    "flag" => {
                        if toks.len() < 2 {
                            return Err(self.errf("flag requires a name string"));
                        }
                        let mut rf = RawCommandFlag { name: toks[1].clone(), ..Default::default() };
                        let mut i = 2usize;
                        if i < toks.len() && toks[i] == "bool" {
                            rf.is_bool = true;
                            i += 1;
                        }
                        if i < toks.len() {
                            rf.description = toks[i].clone();
                            i += 1;
                        }
                        if i + 1 < toks.len() && toks[i] == "default" {
                            rf.default = toks[i + 1].clone();
                        }
                        cmd.flags.push(rf);
                        self.next_line();
                        continue;
                    }
                    _ => {}
                }
                // First non-header line marks the body's start.
                header_done = true;
            }
            // Body line. Track backtick toggles.
            body_raws.push(ln.clone());
            self.line_no += 1;
            in_backtick = update_backtick_state(&ln, in_backtick);
        }
        // Merge backticks then dedent.
        let mut body_raws = merge_backticks_in_lines(&body_raws);
        let mut min_indent: i64 = -1;
        for ln in &body_raws {
            if ln.trim().is_empty() {
                continue;
            }
            let leading = leading_whitespace(ln) as i64;
            if min_indent < 0 || leading < min_indent {
                min_indent = leading;
            }
        }
        if min_indent < 0 {
            return Ok(cmd);
        }
        let mi = min_indent as usize;
        for ln in body_raws.iter_mut() {
            if ln.len() >= mi {
                *ln = ln[mi..].to_string();
            }
        }
        cmd.body = body_raws.join("\n") + "\n";
        Ok(cmd)
    }

    /// Port of `parseFunctionBodyStatements`.
    ///
    /// Collects the remaining lines of a function (from the current line until
    /// the closing `end` at indent 0) as raw text. The text is later handed to
    /// the inner-DSL parser, then to the translator that splits it into
    /// Template + Run.
    fn parse_function_body_statements(&mut self, _start_indent: usize) -> Result<String, String> {
        // Collect raw lines (NOT skipping blanks/comments via peek_line — every
        // line is needed to track multi-line backtick state). Stop at the next
        // non-blank, non-backtick-continuation line at indent 0.
        let mut raws: Vec<String> = Vec::new();
        let mut in_backtick = false;
        // `tmpl_stack` holds the indent of each open `template … end` block.
        // While non-empty we're inside a template body, where a column-0 line is
        // content (e.g. a flush-left `${indent 2 body}`) — NOT the function's
        // closing `end`.
        let mut tmpl_stack: Vec<usize> = Vec::new();
        loop {
            if self.line_no >= self.lines.len() {
                return Err(self.errf("unexpected EOF inside function body"));
            }
            let ln = self.lines[self.line_no].clone();
            // While inside a multi-line backtick, every line belongs to the body
            // regardless of indent.
            if in_backtick {
                raws.push(ln.clone());
                self.line_no += 1;
                in_backtick = update_backtick_state(&ln, in_backtick);
                continue;
            }
            let stripped = ln.trim();
            // Blank line OR comment line inside the body: keep going.
            if stripped.is_empty() || stripped.starts_with('#') {
                raws.push(ln.clone());
                self.line_no += 1;
                continue;
            }
            let leading = leading_whitespace(&ln);
            // Track `template … end` nesting. Both lines are still collected as
            // ordinary body lines — the template→write rewrite is a later pass.
            if leading > 0 && stripped == "template" {
                tmpl_stack.push(leading);
            } else if !tmpl_stack.is_empty()
                && stripped == "end"
                && leading == *tmpl_stack.last().unwrap()
            {
                tmpl_stack.pop();
            }
            if leading == 0 && tmpl_stack.is_empty() {
                // indent == 0 outside any template body = the function's closing
                // `end`. Stop here.
                break;
            }
            raws.push(ln.clone());
            self.line_no += 1;
            in_backtick = update_backtick_state(&ln, in_backtick);
        }
        // Sugar pass: rewrite `template … end` blocks into the equivalent
        // multi-line `` write `…` `` literal BEFORE merging backticks.
        let base = self.line_no.saturating_sub(raws.len());
        let rewritten = rewrite_template_blocks(&raws, base)?;
        // Merge multi-line backticks inside the collected body slice.
        let mut raws = merge_backticks_in_lines(&rewritten);
        // Strip the deepest common leading indent so the inner-DSL parser
        // (which expects column-0 statements) is happy.
        let mut min_indent: i64 = -1;
        for ln in &raws {
            if ln.trim().is_empty() {
                continue;
            }
            let leading = leading_whitespace(ln) as i64;
            if min_indent < 0 || leading < min_indent {
                min_indent = leading;
            }
        }
        if min_indent < 0 {
            return Ok(String::new());
        }
        let mi = min_indent as usize;
        for ln in raws.iter_mut() {
            if ln.len() >= mi {
                *ln = ln[mi..].to_string();
            }
        }
        Ok(raws.join("\n") + "\n")
    }

    /// Port of `parseContext`.
    ///
    /// Reads the body of a `context … end` block. Each line is
    /// `NAME [ ] | { } | <STRING> | <NUMBER>`.
    fn parse_context(&mut self, ctx: &mut BTreeMap<String, Val>) -> Result<(), String> {
        loop {
            let (line, indent) = match self.peek_line() {
                None => return Err(self.errf("unexpected EOF inside context")),
                Some(x) => x,
            };
            if indent == 0 {
                let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
                if tokens.len() == 1 && tokens[0] == "end" {
                    self.next_line();
                    return Ok(());
                }
                return Err(self.errf(format!(
                    "expected `end` to close context, got {}",
                    gofmt::quote(line.trim())
                )));
            }
            let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
            if tokens.is_empty() {
                self.next_line();
                continue;
            }
            self.next_line();
            if tokens.len() < 2 {
                // Go formats the token slice with %v → `[a b c]`.
                return Err(self.errf(format!(
                    "context entry needs `NAME VALUE`, got [{}]",
                    tokens.join(" ")
                )));
            }
            let name = tokens[0].clone();
            let rest = &tokens[1..];
            let val = parse_context_value(rest)
                .map_err(|e| self.errf(format!("context {}: {}", gofmt::quote(&name), e)))?;
            ctx.insert(name, val);
        }
    }

    /// Port of `parseType`.
    ///
    /// Reads `type NAME` body lines until a matching `end` at column 0. Body
    /// lines accept `base TYPE`, `pattern STRING`, `options STR STR …`.
    fn parse_type(&mut self) -> Result<RawType, String> {
        let mut td = RawType::default();
        loop {
            let (line, indent) = match self.peek_line() {
                None => return Err(self.errf("unexpected EOF inside type")),
                Some(x) => x,
            };
            if indent == 0 {
                let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
                if tokens.len() == 1 && tokens[0] == "end" {
                    self.next_line();
                    return Ok(td);
                }
                return Err(self.errf(format!(
                    "expected `end` to close type, got {}",
                    gofmt::quote(line.trim())
                )));
            }
            let tokens = tokenize_lib_line(&line).map_err(|e| self.errf(e))?;
            if tokens.is_empty() {
                self.next_line();
                continue;
            }
            match tokens[0].as_str() {
                "description" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("description requires one string argument"));
                    }
                    td.description = tokens[1].clone();
                }
                "base" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("base requires a type name"));
                    }
                    td.base = tokens[1].clone();
                }
                "pattern" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("pattern requires one regex string"));
                    }
                    td.pattern = tokens[1].clone();
                }
                "options" => {
                    self.next_line();
                    if tokens.len() < 2 {
                        return Err(self.errf("options requires one or more values"));
                    }
                    td.options.extend_from_slice(&tokens[1..]);
                }
                "group_open" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("group_open requires one delimiter string"));
                    }
                    td.group_open = tokens[1].clone();
                }
                "group_close" => {
                    self.next_line();
                    if tokens.len() != 2 {
                        return Err(self.errf("group_close requires one delimiter string"));
                    }
                    td.group_close = tokens[1].clone();
                }
                other => {
                    return Err(self.errf(format!(
                        "unknown directive inside type: {}",
                        gofmt::quote(other)
                    )));
                }
            }
        }
    }
}

/// Port of `isIdent`.
///
/// Reports whether `s` looks like a bareword identifier — used to distinguish a
/// TYPE token from a trailing description string in
/// `arg capture NAME TYPE [DESC]`. The tokenizer already strips quotes from
/// string tokens, so a description like `"An email address"` arrives here as a
/// single token with spaces in it (which fails this check).
pub fn is_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    for (i, c) in s.chars().enumerate() {
        if i == 0 {
            if !(c == '_' || c.is_ascii_lowercase() || c.is_ascii_uppercase()) {
                return false;
            }
            continue;
        }
        if !(c == '_' || c.is_ascii_digit() || c.is_ascii_lowercase() || c.is_ascii_uppercase()) {
            return false;
        }
    }
    true
}

/// Port of `indentOf` — spaces count 1, tabs count 4.
fn indent_of(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.chars() {
        if c == ' ' {
            n += 1;
        } else if c == '\t' {
            n += 4;
        } else {
            break;
        }
    }
    n
}

/// Port of `parseContextValue`.
fn parse_context_value(toks: &[String]) -> Result<Val, String> {
    if toks.len() == 1 {
        let t = toks[0].as_str();
        if t == "[]" {
            return Ok(Val::List(Vec::new()));
        }
        if t == "{}" {
            return Ok(Val::obj());
        }
        if t == "true" {
            return Ok(Val::Bool(true));
        }
        if t == "false" {
            return Ok(Val::Bool(false));
        }
        // numeric?
        if let Ok(n) = t.parse::<i64>() {
            return Ok(Val::Int(n));
        }
        if let Ok(f) = parse_float_go(t) {
            return Ok(Val::Float(f));
        }
        // fallback: string
        return Ok(Val::Str(t.to_string()));
    }
    if toks[0] == "[" && toks[toks.len() - 1] == "]" {
        // inline list of scalars: [ a b c ]
        let mut out: Vec<Val> = Vec::with_capacity(toks.len() - 2);
        for t in &toks[1..toks.len() - 1] {
            out.push(parse_context_value(std::slice::from_ref(t))?);
        }
        return Ok(Val::List(out));
    }
    Ok(Val::Str(toks.join(" ")))
}

/// Go's `strconv.ParseFloat` accepts forms Rust's `f64::from_str` rejects or
/// spells differently; restrict to what Go would accept here.
fn parse_float_go(s: &str) -> Result<f64, ()> {
    if s.is_empty() {
        return Err(());
    }
    // Rust accepts "inf"/"NaN" in mixed case just as Go does, but also accepts a
    // trailing/leading "+"; Go does too. The meaningful difference is that Rust
    // rejects nothing Go accepts for plain decimal forms, so a direct parse is
    // faithful for every value the lib parser can see.
    s.parse::<f64>().map_err(|_| ())
}

/// Port of `parseCloseSeqSegs`.
///
/// Parses the arguments of a `block_close_seq` directive line into segments,
/// preserving which were QUOTED (→ literal) vs BARE (→ capture reference). It
/// must read the raw line because [`tokenize_lib_line`] de-quotes, erasing that
/// distinction. The leading `block_close_seq` keyword is skipped.
fn parse_close_seq_segs(line: &str) -> Result<Vec<RawCloseSeg>, String> {
    let s = line.trim_end_matches([' ', '\t']);
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    let mut segs: Vec<RawCloseSeg> = Vec::new();
    let mut first = true;
    while i < b.len() {
        let c = b[i];
        if c == b' ' || c == b'\t' {
            i += 1;
        } else if c == b'#' {
            i = b.len();
        } else if c == b'"' {
            let mut j = i + 1;
            let mut out: Vec<u8> = Vec::new();
            while j < b.len() && b[j] != b'"' {
                if b[j] == b'\\' && j + 1 < b.len() {
                    match b[j + 1] {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        other => out.push(other),
                    }
                    j += 2;
                    continue;
                }
                out.push(b[j]);
                j += 1;
            }
            if j >= b.len() {
                return Err("block_close_seq: unterminated string literal".to_string());
            }
            if !first {
                // Skip the directive keyword (always bare, position 0).
                segs.push(RawCloseSeg {
                    text: String::from_utf8_lossy(&out).into_owned(),
                    is_ref: false,
                });
            }
            first = false;
            i = j + 1;
        } else {
            let mut j = i;
            while j < b.len() && b[j] != b' ' && b[j] != b'\t' && b[j] != b'#' {
                j += 1;
            }
            let word = &s[i..j];
            if first {
                // This is the `block_close_seq` keyword itself.
                first = false;
            } else {
                segs.push(RawCloseSeg { text: word.to_string(), is_ref: true });
            }
            i = j;
        }
    }
    Ok(segs)
}

/// Port of `tokenizeLibLine`.
///
/// Splits a line into tokens, respecting double-quoted strings with Go-style
/// escapes. The result is the list of token *contents* (quotes removed, escapes
/// decoded). Comments after `#` (outside strings) are dropped.
pub fn tokenize_lib_line(line: &str) -> Result<Vec<String>, String> {
    let mut toks: Vec<String> = Vec::new();
    let s = line.trim_end_matches([' ', '\t']);
    let b = s.as_bytes();
    let mut i = 0usize;
    // skip leading indent
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    while i < b.len() {
        let c = b[i];
        if c == b' ' || c == b'\t' {
            i += 1;
        } else if c == b'#' {
            return Ok(toks);
        } else if c == b'"' {
            let mut j = i + 1;
            let mut out: Vec<u8> = Vec::new();
            while j < b.len() && b[j] != b'"' {
                if b[j] == b'\\' && j + 1 < b.len() {
                    match b[j + 1] {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        other => out.push(other),
                    }
                    j += 2;
                    continue;
                }
                out.push(b[j]);
                j += 1;
            }
            if j >= b.len() {
                return Err("unterminated string literal".to_string());
            }
            toks.push(String::from_utf8_lossy(&out).into_owned());
            i = j + 1;
        } else {
            let mut j = i;
            while j < b.len() && b[j] != b' ' && b[j] != b'\t' && b[j] != b'#' {
                j += 1;
            }
            toks.push(s[i..j].to_string());
            i = j;
        }
    }
    Ok(toks)
}
