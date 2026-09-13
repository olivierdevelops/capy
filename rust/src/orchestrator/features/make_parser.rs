//! Port of `orchestrator/features/make_parser.go`.
//!
//! The outer pattern matcher. It walks the token stream and at each statement
//! boundary tries each library function's compiled `elements` in priority order;
//! the first complete match wins.

use super::expr_to_text::expr_to_text;
use super::value_parser::{parse_value, TokReader};
use crate::domain::ast::{Block, CaptureValue, FuncCall};
use crate::domain::errors::{suggest_closest, CapyError};
use crate::domain::library::{CloseSegment, FuncDef, Library, PatternElement, TypeDef};
use crate::domain::token::{Token, TokenKind};
use crate::gofmt;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Port of `MakeParser(...).Parse`.
pub fn parse(toks: Vec<Token>, src: &str, lib: &Library) -> Result<Block, CapyError> {
    let mut fns: Vec<Rc<FuncDef>> = lib.functions.values().cloned().map(Rc::new).collect();
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
    let by_name: BTreeMap<String, Rc<FuncDef>> =
        fns.iter().map(|f| (f.name.clone(), f.clone())).collect();
    let mut pp = OuterP {
        toks,
        pos: 0,
        fns,
        by_name,
        types: lib.types.clone(),
        src_lines: split_source_lines(src),
        seq_depth: 0,
    };
    pp.parse_program(false, "")
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

struct OuterP {
    toks: Vec<Token>,
    pos: usize,
    fns: Vec<Rc<FuncDef>>,
    by_name: BTreeMap<String, Rc<FuncDef>>,
    types: BTreeMap<String, TypeDef>,
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
    /// Port of `parseProgram`.
    fn parse_program(&mut self, in_block: bool, closer_name: &str) -> Result<Block, CapyError> {
        let mut stmts: Vec<FuncCall> = Vec::new();
        // `stray_indents` counts INDENT tokens that appeared mid-body without a
        // block-opener directive in front of them — i.e. the user nested content
        // deeper than the block's anchor purely for visual styling. Each stray
        // INDENT must be paired with a matching DEDENT before the real
        // body-closing DEDENT is reached. If the matching DEDENT is immediately
        // followed by the body's closer keyword, that closer is also part of the
        // cosmetic nesting and is consumed as a no-op.
        let mut stray_indents = 0usize;
        loop {
            let k = self.peek().kind;
            if k == TokenKind::Eof {
                break;
            }
            if k == TokenKind::Newline {
                self.advance();
                continue;
            }
            if k == TokenKind::Indent {
                // A stray indent — content deeper than the surrounding block
                // without a real opener. Treat as a no-op so user-side
                // indentation is purely cosmetic.
                self.advance();
                stray_indents += 1;
                continue;
            }
            if k == TokenKind::Dedent {
                if stray_indents > 0 {
                    self.advance();
                    stray_indents -= 1;
                    // If the user mirrored their stray INDENT with a stray `end`
                    // keyword, consume it (and its newline) so the real block
                    // closer is reached.
                    if in_block && self.at_closer(closer_name) {
                        if let Some(cp) = self.by_name.get(closer_name).cloned() {
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
            stmts.push(self.parse_stmt()?);
        }
        Ok(Block { stmts, ..Default::default() })
    }

    /// Port of `atCloser`.
    fn at_closer(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let cp = match self.by_name.get(name).cloned() {
            None => return false,
            Some(c) => c,
        };
        let saved = self.save();
        let ok = self.try_match(&cp).is_ok();
        self.restore(saved);
        ok
    }

    /// Port of `parseStmt`.
    fn parse_stmt(&mut self) -> Result<FuncCall, CapyError> {
        let start_tok = self.peek();
        let start_line = start_tok.line;
        let start_col = start_tok.col;
        // `block_err` remembers the FIRST error from a candidate that matched its
        // header and opened a block but then failed to parse its body. We
        // backtrack out of such failures so a flat function can still match the
        // same leading keyword. But if NO later candidate matches either, this
        // remembered error is more informative than the generic "no library
        // function matches" — so it is surfaced at the end.
        let mut block_err: Option<CapyError> = None;
        // Index-walk with a per-candidate Rc bump instead of cloning the whole
        // candidate vector on every statement.
        for ci in 0..self.fns.len() {
            let f = &self.fns[ci].clone();
            let saved = self.save();
            let mut inst = match self.try_match(f) {
                Err(_) => {
                    self.restore(saved);
                    continue;
                }
                Ok(i) => i,
            };
            // Stamp the statement's source position so templates can read it via
            // the `line` / `col` render locals.
            inst.line = start_line;
            inst.col = start_col;

            // Delimiter-mode block: opener is followed directly by `open` (no
            // newline). Named-closer block: opener is followed by NEWLINE then
            // INDENT. Non-block: opener is followed by NEWLINE.
            if let Some(block) = &f.block {
                if !block.open.is_empty() {
                    // Expect the open delimiter on the same line.
                    if !self.matches_delim(&block.open) {
                        self.restore(saved);
                        continue;
                    }
                    let body = match self.parse_delim_block(&block.open, &block.close) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.restore(saved);
                            continue;
                        }
                        Ok(b) => b,
                    };
                    inst.body = Some(Box::new(body));
                    // Expect NEWLINE after the closing delimiter.
                    if self.peek().kind != TokenKind::Newline
                        && self.peek().kind != TokenKind::Eof
                    {
                        return Err(CapyError::msg(format!(
                            "line {}: expected newline after block close",
                            self.peek().line
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
                            self.restore(saved);
                            continue;
                        }
                        Ok(s) => s,
                    };
                    let body = match self.parse_seq_block(&seq, start_line) {
                        Err(e) => {
                            if block_err.is_none() {
                                block_err = Some(e);
                            }
                            self.restore(saved);
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
                && self.peek().kind != TokenKind::Newline
                && self.peek().kind != TokenKind::Eof
            {
                self.restore(saved);
                continue;
            }
            self.consume_newlines();

            // Lookahead: gate the candidate on whether an indented block follows.
            // Lets a flat keyword (when_not_followed_by indent) and a block
            // keyword (when_followed_by indent) share the same literal and
            // disambiguate purely by position.
            if let Some(la) = &f.lookahead {
                let is_indent = self.peek().kind == TokenKind::Indent;
                if (la.require_indent && !is_indent) || (la.forbid_indent && is_indent) {
                    self.restore(saved);
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
                            self.restore(saved);
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
                            self.restore(saved);
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
                            self.restore(saved);
                            continue;
                        }
                        Ok((body, closer)) => {
                            inst.body = Some(Box::new(body));
                            inst.closer = closer.map(Box::new);
                        }
                    }
                }
            }
            return Ok(inst);
        }

        // A candidate matched a block opener but its body failed to parse, and
        // nothing else matched either — surface that error (it points at the real
        // problem inside the body, not the generic "no match").
        if let Some(e) = block_err {
            return Err(e);
        }
        // Build a "did you mean…?" hint: the closest literal-starting function
        // name to the unrecognized token.
        let mut err = CapyError::new(
            start_line,
            start_col,
            format!("no library function matches token {}", gofmt::quote(&start_tok.text)),
        );
        let literals: Vec<String> = self
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
        if self.peek().text == d {
            self.advance();
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
            if self.peek().text == close {
                self.advance();
                return Ok(Block { stmts, ..Default::default() });
            }
            if self.peek().kind == TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "unexpected EOF inside {}...{} block",
                    gofmt::quote(open),
                    gofmt::quote(close)
                )));
            }
            if self.peek().kind == TokenKind::Newline {
                self.advance();
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
            let k = self.peek().kind;
            if k == TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "line {}: unexpected end of input: expected closing {}",
                    opener_line,
                    gofmt::quote(&closer)
                )));
            }
            if k == TokenKind::Newline || k == TokenKind::Indent || k == TokenKind::Dedent {
                self.advance();
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
            self.toks[idx] = split_tok.unwrap();
            self.pos = idx;
            return true;
        }
        self.pos = i;
        true
    }

    fn consume_newlines(&mut self) {
        while self.peek().kind == TokenKind::Newline {
            self.advance();
        }
    }

    /// Port of `tryMatch`.
    fn try_match(&mut self, f: &FuncDef) -> Result<FuncCall, CapyError> {
        let mut caps: BTreeMap<String, CaptureValue> = BTreeMap::new();
        for i in 0..f.elements.len() {
            let el = f.elements[i].clone();
            // Auto-skip an optional comma between consecutive captures.
            if i > 0
                && el.is_capture
                && f.elements[i - 1].is_capture
                && self.peek().kind == TokenKind::Punct
                && self.peek().text == ","
            {
                self.advance();
            }
            if !el.is_capture {
                if !self.match_literal(&el.literal) {
                    return Err(CapyError::msg(format!(
                        "expected {}",
                        gofmt::quote(&el.literal)
                    )));
                }
                continue;
            }
            // Function-as-type capture (named nonterminal): the capture's type
            // names another library function. Match that function's shape —
            // possibly repeated (`type*` / `type+`) with an optional separator
            // literal — and store the matched sub-FuncCall(s).
            if el.is_func {
                let val = self.capture_func_type(&el)?;
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
            let stop = next_literals(&f.elements[i + 1..]);
            let val = self.capture_value(&el.cap_type, &stop)?;
            caps.insert(el.name.clone(), val);
        }
        Ok(FuncCall {
            func: f.name.clone(),
            captures: caps,
            body: None,
            closer: None,
            sections: BTreeMap::new(),
            line: 0,
            col: 0,
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
        let target = match self.by_name.get(&el.cap_type).cloned() {
            None => {
                return Err(CapyError::msg(format!(
                    "internal: function-typed capture {} references unknown function {}",
                    gofmt::quote(&el.name),
                    gofmt::quote(&el.cap_type)
                )))
            }
            Some(t) => t,
        };

        // Exactly-one (no repetition): a single mandatory match.
        if el.repeat.is_empty() {
            match self.match_one(&target) {
                None => {
                    return Err(CapyError::msg(format!("expected {}", el.cap_type)));
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
                let sep = self.save();
                if !self.match_literal(&el.sep) {
                    self.restore(sep);
                    break;
                }
                match self.match_one(&target) {
                    None => {
                        // Separator consumed but no following item — roll back the
                        // separator so it isn't lost.
                        self.restore(sep);
                        break;
                    }
                    Some(fc) => {
                        subs.push(fc);
                        continue;
                    }
                }
            }
            match self.match_one(&target) {
                None => break,
                Some(fc) => subs.push(fc),
            }
        }
        if el.repeat == "+" && subs.is_empty() {
            return Err(CapyError::msg(format!("expected at least one {}", el.cap_type)));
        }
        Ok(CaptureValue { sub: subs, ..Default::default() })
    }

    /// The `matchOne` closure from `captureFuncType`.
    fn match_one(&mut self, target: &FuncDef) -> Option<FuncCall> {
        let saved = self.save();
        let start_pos = self.pos;
        match self.try_match(target) {
            Err(_) => {
                self.restore(saved);
                None
            }
            Ok(mut fc) => {
                if self.pos == start_pos {
                    self.restore(saved);
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
            self.peek().kind,
            TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent
        )
    }

    /// Port of `matchLiteral`.
    fn match_literal(&mut self, lit: &str) -> bool {
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
                    self.advance();
                    return true;
                }
                // The lexer greedily merges runs of punctuation into one token, so
                // an angle-bracket tag boundary like `><` arrives as a single `><`
                // punct token. When a punct literal is a strict prefix of such a
                // merged run, match the prefix and write the suffix back as the
                // next token (adjusted col/width) so it can be matched next.
                if t.kind == TokenKind::Punct
                    && t.text.len() > lit.len()
                    && t.text.starts_with(lit)
                {
                    let mut rem = t.clone();
                    rem.text = t.text[lit.len()..].to_string();
                    rem.col = t.col + lit.len();
                    rem.width = rem.text.len();
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
                self.advance();
                return Ok(CaptureValue { text: tok.text, ..Default::default() });
            }
            "raw" => {
                let tok = self.peek();
                if tok.kind == TokenKind::Ident || tok.kind == TokenKind::Str {
                    self.advance();
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
                    let k = self.peek().kind;
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
                let k = self.peek().kind;
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
                    self.advance();
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
                while self.peek().kind == TokenKind::Punct
                    && self.peek().text == "."
                    && self.peek().col == end
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
        if let Some(td) = self.types.get(t).cloned() {
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
                self.advance();
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
        let x = parse_value(self, stop)?;
        let text = expr_to_text(&x);
        Ok(CaptureValue { is_expr: true, expr: Some(x), text, sub: Vec::new() })
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
                gofmt::quote(&self.peek().text)
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
                    self.advance();
                    prev_end = -1;
                    continue;
                }
                TokenKind::Indent | TokenKind::Dedent => {
                    // These structural tokens contribute no characters to the
                    // captured text; their effect is encoded in the next token's
                    // col on the new line.
                    self.advance();
                    continue;
                }
                _ => {}
            }
            if tok.text == td.group_close {
                depth -= 1;
                if depth == 0 {
                    self.advance();
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
            self.advance();
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
                    self.advance();
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
        if self.peek().kind != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented block (verbatim, closer={})",
                self.peek().line,
                closer_name
            )));
        }
        self.advance();

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
            match self.peek().kind {
                TokenKind::Eof => {
                    return Err(CapyError::msg(format!(
                        "line {}: unterminated verbatim block, expected closer {}",
                        self.peek().line,
                        gofmt::quote(closer_name)
                    )));
                }
                TokenKind::Indent => {
                    depth += 1;
                    self.advance();
                    continue;
                }
                TokenKind::Dedent => {
                    if depth == 0 {
                        closer_line = self.peek().line as i64;
                        self.advance();
                        break;
                    }
                    depth -= 1;
                    self.advance();
                    continue;
                }
                _ => {}
            }
            self.advance();
        }

        let text = self.verbatim_slice(opener_line as i64, closer_line);

        let cp = match self.by_name.get(closer_name).cloned() {
            None => {
                return Err(CapyError::msg(format!(
                    "library function {} (verbatim closer) not found",
                    gofmt::quote(closer_name)
                )))
            }
            Some(c) => c,
        };
        let saved = self.save();
        let closer_inst = match self.try_match(&cp) {
            Err(_) => {
                self.restore(saved);
                return Err(CapyError::msg(format!(
                    "line {}: expected closer {}",
                    self.peek().line,
                    gofmt::quote(closer_name)
                )));
            }
            Ok(c) => c,
        };
        if self.peek().kind != TokenKind::Newline && self.peek().kind != TokenKind::Eof {
            return Err(CapyError::msg(format!(
                "line {}: closer must end the line",
                self.peek().line
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
        if self.peek().kind != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented block (closer={})",
                self.peek().line,
                closer_name
            )));
        }
        self.advance();

        let body = self.parse_program(true, closer_name)?;
        if self.peek().kind != TokenKind::Dedent {
            return Err(CapyError::msg(format!(
                "line {}: expected end of block",
                self.peek().line
            )));
        }
        self.advance();
        // Dedent-only block: no closer keyword to match. Useful for CSS-style
        // selectors and other DSLs where a body is delimited purely by
        // indentation.
        if closer_name.is_empty() {
            return Ok((body, None));
        }
        let cp = match self.by_name.get(closer_name).cloned() {
            None => {
                return Err(CapyError::msg(format!(
                    "library function {} (closer) not found",
                    gofmt::quote(closer_name)
                )))
            }
            Some(c) => c,
        };
        let saved = self.save();
        let closer_inst = match self.try_match(&cp) {
            Err(_) => {
                self.restore(saved);
                return Err(CapyError::msg(format!(
                    "line {}: expected closer {}",
                    self.peek().line,
                    gofmt::quote(closer_name)
                )));
            }
            Ok(c) => c,
        };
        if self.peek().kind != TokenKind::Newline && self.peek().kind != TokenKind::Eof {
            return Err(CapyError::msg(format!(
                "line {}: closer must end the line",
                self.peek().line
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
        if self.peek().kind != TokenKind::Indent {
            return Err(CapyError::msg(format!(
                "line {}: expected indented body",
                self.peek().line
            )));
        }
        self.advance();
        let body = self.parse_program(true, "")?;
        if self.peek().kind != TokenKind::Dedent {
            return Err(CapyError::msg(format!(
                "line {}: expected end of indented body",
                self.peek().line
            )));
        }
        self.advance();
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
        if self.peek().kind == TokenKind::Indent {
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
                self.advance();
                if self.peek().kind != TokenKind::Newline {
                    return Err(CapyError::msg(format!(
                        "line {}: section {} must be alone on its line",
                        tok.line,
                        gofmt::quote(&name)
                    )));
                }
                self.consume_newlines();
                if self.peek().kind == TokenKind::Indent {
                    let b = self.parse_indented_region()?;
                    sec_bodies.insert(name, b);
                } else {
                    sec_bodies.insert(name, Block::default());
                }
                continue;
            }
            // Not a section — must be the closer.
            let cp = match self.by_name.get(closer_name).cloned() {
                None => {
                    return Err(CapyError::msg(format!(
                        "library function {} (closer) not found",
                        gofmt::quote(closer_name)
                    )))
                }
                Some(c) => c,
            };
            let saved = self.save();
            let closer_inst = match self.try_match(&cp) {
                Err(_) => {
                    self.restore(saved);
                    return Err(CapyError::msg(format!(
                        "line {}: expected closer {} or one of sections [{}]",
                        tok.line,
                        gofmt::quote(closer_name),
                        sections.join(" ")
                    )));
                }
                Ok(c) => c,
            };
            if self.peek().kind != TokenKind::Newline && self.peek().kind != TokenKind::Eof {
                return Err(CapyError::msg(format!(
                    "line {}: closer must end the line",
                    self.peek().line
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
