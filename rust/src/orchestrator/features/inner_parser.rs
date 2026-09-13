//! Port of `orchestrator/features/inner_parser.go`.

use super::value_parser::{parse_value, TokReader};
use crate::domain::ast::{CallExpr, Expr, InnerBlock, InnerStmt, LoopStmt, Path, PathStep};
use crate::domain::errors::CapyError;
use crate::domain::token::{Token, TokenKind};
use crate::gofmt;

/// Port of `ParseInner`.
///
/// Parses an inner-DSL `run:` snippet (already lexed via the outer lexer) into
/// an `InnerBlock` AST. The inner DSL is hardcoded — it knows fixed statement
/// forms (set, append, …, if, loop, plain call) and uses the shared
/// value-expression parser.
pub fn parse_inner(toks: Vec<Token>) -> Result<InnerBlock, CapyError> {
    let mut p = InnerP { toks, pos: 0 };
    p.parse_program(false)
}

/// Port of the `innerP` cursor.
pub struct InnerP {
    pub toks: Vec<Token>,
    pub pos: usize,
}

impl TokReader for InnerP {
    /// Go indexes `p.toks[p.pos]` directly, which panics past the end. The token
    /// stream always terminates with EOF and every parser stops there, so this
    /// saturates on the final token instead of panicking — same observable
    /// behaviour for well-formed streams, no crash for malformed ones.
    fn peek(&self) -> Token {
        if self.pos < self.toks.len() {
            self.toks[self.pos].clone()
        } else {
            self.toks
                .last()
                .cloned()
                .unwrap_or(Token {
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

impl InnerP {
    fn parse_program(&mut self, in_block: bool) -> Result<InnerBlock, CapyError> {
        let mut stmts: Vec<InnerStmt> = Vec::new();
        loop {
            let k = self.peek().kind;
            if k == TokenKind::Eof {
                break;
            }
            if k == TokenKind::Newline {
                self.advance();
                continue;
            }
            if k == TokenKind::Dedent {
                break;
            }
            if in_block && (self.at_keyword("end") || self.at_keyword("else")) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(InnerBlock { stmts })
    }

    fn at_keyword(&self, w: &str) -> bool {
        let t = self.peek();
        t.kind == TokenKind::Ident && t.text == w
    }

    fn parse_stmt(&mut self) -> Result<InnerStmt, CapyError> {
        let t = self.peek();
        if t.kind != TokenKind::Ident {
            return Err(CapyError::msg(format!(
                "line {}: expected statement, got {}",
                t.line,
                gofmt::quote(&t.text)
            )));
        }
        match t.text.as_str() {
            "set" => {
                self.advance();
                let target = self.parse_path()?;
                let value = parse_value(self, &[])?;
                self.consume_newline();
                Ok(InnerStmt::Set { target, value })
            }
            "append" => {
                self.advance();
                let target = self.parse_path()?;
                let value = parse_value(self, &[])?;
                self.consume_newline();
                Ok(InnerStmt::Append { target, value })
            }
            "prepend" => {
                self.advance();
                let target = self.parse_path()?;
                let value = parse_value(self, &[])?;
                self.consume_newline();
                Ok(InnerStmt::Prepend { target, value })
            }
            "merge" => {
                self.advance();
                let target = self.parse_path()?;
                let value = parse_value(self, &[])?;
                self.consume_newline();
                Ok(InnerStmt::Merge { target, value })
            }
            "delete" => {
                self.advance();
                let target = self.parse_path()?;
                self.consume_newline();
                Ok(InnerStmt::Delete { target })
            }
            "if" => self.parse_if(),
            "loop" | "for" => self.parse_loop(),
            "write" => {
                self.advance();
                let value = parse_value(self, &[])?;
                self.consume_newline();
                Ok(InnerStmt::Write(value))
            }
            "let" => {
                // `let NAME = EXPR` — bind a local variable (in command bodies).
                self.advance();
                let name_tok = self.peek();
                if name_tok.kind != TokenKind::Ident {
                    return Err(CapyError::msg(format!(
                        "line {}: let requires an identifier name",
                        name_tok.line
                    )));
                }
                self.advance();
                let eq = self.peek();
                if !(eq.kind == TokenKind::Punct && eq.text == "=") {
                    return Err(CapyError::msg(format!(
                        "line {}: let requires `= EXPR`",
                        eq.line
                    )));
                }
                self.advance();
                let value = parse_value(self, &[])?;
                self.consume_newline();
                // Reuse Set with a synthetic path rooted at `locals`. The
                // evaluator treats `locals.X` writes as local-scope binds.
                Ok(InnerStmt::Set {
                    target: Path {
                        root: "locals".to_string(),
                        steps: vec![PathStep::field(name_tok.text.clone())],
                    },
                    value,
                })
            }
            // fallthrough: a generic call (e.g. `regex_match x y`, `error "msg"`,
            // or a recursive call to another library function — though for now
            // the inner DSL only supports primitives at this position).
            _ => self.parse_call(),
        }
    }

    fn consume_newline(&mut self) {
        while self.peek().kind == TokenKind::Newline {
            self.advance();
        }
    }

    fn parse_path(&mut self) -> Result<Path, CapyError> {
        let t = self.peek();
        if t.kind != TokenKind::Ident {
            return Err(CapyError::msg(format!(
                "line {}: expected path root identifier",
                t.line
            )));
        }
        self.advance();
        let mut path = Path { root: t.text.clone(), steps: Vec::new() };
        loop {
            let nt = self.peek();
            if nt.kind == TokenKind::Punct && nt.text == "." {
                self.advance();
                let name = self.peek();
                if name.kind != TokenKind::Ident {
                    return Err(CapyError::msg("expected identifier after ."));
                }
                self.advance();
                path.steps.push(PathStep::field(name.text.clone()));
                continue;
            }
            if nt.kind == TokenKind::LBrack {
                self.advance();
                let idx = parse_value(self, &[])?;
                if self.peek().kind != TokenKind::RBrack {
                    return Err(CapyError::msg("expected ]"));
                }
                self.advance();
                path.steps.push(PathStep::index(idx));
                continue;
            }
            break;
        }
        Ok(path)
    }

    fn parse_if(&mut self) -> Result<InnerStmt, CapyError> {
        self.advance(); // if
        let cond = parse_value(self, &[])?;
        if self.peek().kind != TokenKind::Newline {
            return Err(CapyError::msg(format!(
                "line {}: expected newline after if cond",
                self.peek().line
            )));
        }
        self.consume_newline();
        let body = self.parse_block_body()?;
        if self.at_keyword("else") {
            self.advance();
            // `else if` chains by re-entering parse_if and wrapping it in a
            // single-statement else block.
            let else_block = if self.at_keyword("if") {
                let nested = self.parse_if()?;
                InnerBlock { stmts: vec![nested] }
            } else {
                self.consume_newline();
                let eb = self.parse_block_body()?;
                if !self.at_keyword("end") {
                    return Err(CapyError::msg(format!(
                        "line {}: expected `end` to close else",
                        self.peek().line
                    )));
                }
                self.advance();
                self.consume_newline();
                eb
            };
            return Ok(InnerStmt::If { cond, body, else_: Some(Box::new(else_block)) });
        }
        if !self.at_keyword("end") {
            return Err(CapyError::msg(format!(
                "line {}: expected `end` to close if",
                self.peek().line
            )));
        }
        self.advance();
        self.consume_newline();
        Ok(InnerStmt::If { cond, body, else_: None })
    }

    fn parse_loop(&mut self) -> Result<InnerStmt, CapyError> {
        self.advance(); // for / loop
        if self.peek().kind != TokenKind::Ident {
            return Err(CapyError::msg(format!(
                "line {}: expected loop variable",
                self.peek().line
            )));
        }
        let first = self.advance().text;

        // Two-var form: `for KEY, VAL in EXPR`, detected by a `,` punct token
        // right after the first ident.
        let mut key_var = String::new();
        let mut v = first.clone();
        if self.peek().kind == TokenKind::Punct && self.peek().text == "," {
            self.advance();
            if self.peek().kind != TokenKind::Ident {
                return Err(CapyError::msg(format!(
                    "line {}: expected second loop variable after `,`",
                    self.peek().line
                )));
            }
            key_var = first;
            v = self.advance().text;
        }

        if !self.at_keyword("in") {
            return Err(CapyError::msg(format!("line {}: expected `in`", self.peek().line)));
        }
        self.advance();
        let iter = parse_value(self, &[])?;
        if self.peek().kind != TokenKind::Newline {
            return Err(CapyError::msg(format!(
                "line {}: expected newline after loop",
                self.peek().line
            )));
        }
        self.consume_newline();
        let body = self.parse_block_body()?;
        if !self.at_keyword("end") {
            return Err(CapyError::msg(format!(
                "line {}: expected `end` to close loop",
                self.peek().line
            )));
        }
        self.advance();
        self.consume_newline();
        Ok(InnerStmt::Loop(LoopStmt { var: v, key_var, iter, body }))
    }

    fn parse_block_body(&mut self) -> Result<InnerBlock, CapyError> {
        if self.peek().kind == TokenKind::Indent {
            self.advance();
        }
        let body = self.parse_program(true)?;
        if self.peek().kind == TokenKind::Dedent {
            self.advance();
        }
        Ok(body)
    }

    fn parse_call(&mut self) -> Result<InnerStmt, CapyError> {
        let t = self.advance();
        let mut name = vec![t.text.clone()];
        while self.peek().kind == TokenKind::Punct && self.peek().text == "." {
            self.advance();
            let n = self.peek();
            if n.kind != TokenKind::Ident {
                return Err(CapyError::msg("expected identifier after ."));
            }
            self.advance();
            name.push(n.text.clone());
        }
        let mut args: Vec<Expr> = Vec::new();
        while !self.at_statement_end() {
            if self.peek().kind == TokenKind::Punct && self.peek().text == "," {
                self.advance();
                continue;
            }
            args.push(parse_value(self, &[])?);
        }
        self.consume_newline();
        Ok(InnerStmt::Call(CallExpr { name, args }))
    }

    fn at_statement_end(&self) -> bool {
        let k = self.peek().kind;
        k == TokenKind::Newline || k == TokenKind::Eof || k == TokenKind::Dedent
    }
}
