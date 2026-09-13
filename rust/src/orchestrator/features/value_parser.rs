//! Port of `orchestrator/features/value_parser.go`.

use crate::domain::ast::{CallExpr, CompareExpr, Expr, NumberLit, ObjLit, PathStep};
use crate::domain::errors::CapyError;
use crate::domain::token::{Token, TokenKind};
use crate::gofmt;

/// Port of the `tokReader` interface — the read-only token cursor used by the
/// value-expression parser. Both the outer pattern matcher and the inner DSL
/// parser implement it.
pub trait TokReader {
    fn peek(&self) -> Token;
    fn advance(&mut self) -> Token;
    fn save(&self) -> usize;
    fn restore(&mut self, p: usize);
}

/// Port of `contains`.
pub fn contains(s: &[String], x: &str) -> bool {
    s.iter().any(|v| v == x)
}

/// Port of `parseValue`.
///
/// Parses one value expression with an optional comparison tail. It stops at any
/// token in the `stop` set (literal stop tokens) or at statement-boundary
/// tokens (NEWLINE, EOF, INDENT, DEDENT).
pub fn parse_value<R: TokReader + ?Sized>(
    r: &mut R,
    stop: &[String],
) -> Result<Expr, CapyError> {
    let left = parse_unary(r, stop)?;
    let t = r.peek();
    if t.kind == TokenKind::Punct {
        match t.text.as_str() {
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                if !contains(stop, &t.text) {
                    r.advance();
                    let right = parse_unary(r, stop)?;
                    return Ok(Expr::Compare(Box::new(CompareExpr {
                        op: t.text.clone(),
                        left,
                        right,
                    })));
                }
            }
            _ => {}
        }
    }
    Ok(left)
}

/// Port of `parseUnary`.
pub fn parse_unary<R: TokReader + ?Sized>(
    r: &mut R,
    stop: &[String],
) -> Result<Expr, CapyError> {
    let t = r.peek();
    if t.kind == TokenKind::Ident && t.text == "not" && !contains(stop, "not") {
        r.advance();
        let x = parse_unary(r, stop)?;
        return Ok(Expr::Not(Box::new(x)));
    }
    parse_primary(r, stop)
}

/// Port of `parsePrimary`.
pub fn parse_primary<R: TokReader + ?Sized>(
    r: &mut R,
    stop: &[String],
) -> Result<Expr, CapyError> {
    let t = r.peek();
    match t.kind {
        TokenKind::Number => {
            r.advance();
            // Go tries ParseInt first; ANY error (syntax or range) falls through
            // to ParseFloat, so an i64-overflowing literal becomes a float.
            if let Ok(i) = t.text.parse::<i64>() {
                return Ok(Expr::Number(NumberLit { is_int: true, i, f: 0.0 }));
            }
            match t.text.parse::<f64>() {
                Ok(f) => Ok(Expr::Number(NumberLit { is_int: false, i: 0, f })),
                Err(_) => {
                    Err(CapyError::msg(format!("bad number {}", gofmt::quote(&t.text))))
                }
            }
        }
        TokenKind::Str | TokenKind::Template => {
            r.advance();
            Ok(Expr::Str(t.text.clone()))
        }
        TokenKind::Ident => {
            match t.text.as_str() {
                "true" => {
                    r.advance();
                    return Ok(Expr::Bool(true));
                }
                "false" => {
                    r.advance();
                    return Ok(Expr::Bool(false));
                }
                "null" => {
                    r.advance();
                    return Ok(Expr::Null);
                }
                _ => {}
            }
            r.advance();
            // steps[0] is the root name; subsequent `.field` and `[expr]` steps
            // alternate freely so `context.rows[i].name[j]` parses.
            let mut steps: Vec<PathStep> = vec![PathStep::field(t.text.clone())];
            loop {
                let n = r.peek();
                if n.kind == TokenKind::Punct && n.text == "." {
                    r.advance();
                    let name = r.peek();
                    if name.kind != TokenKind::Ident {
                        return Err(CapyError::msg("expected identifier after ."));
                    }
                    r.advance();
                    steps.push(PathStep::field(name.text.clone()));
                    continue;
                }
                if n.kind == TokenKind::LBrack {
                    // Postfix index: `[expr]`. `[` is only a list literal when
                    // it's the FIRST token of a primary; here it follows an
                    // identifier path, so the two never collide.
                    r.advance();
                    let idx = parse_value(r, &[])?;
                    if r.peek().kind != TokenKind::RBrack {
                        return Err(CapyError::msg("expected ]"));
                    }
                    r.advance();
                    steps.push(PathStep::index(idx));
                    continue;
                }
                break;
            }
            Ok(Expr::Var(steps))
        }
        TokenKind::LParen => {
            r.advance();
            skip_newlines(r);
            let name_tok = r.peek();
            if name_tok.kind != TokenKind::Ident {
                return Err(CapyError::msg("expected identifier inside ( )"));
            }
            r.advance();
            let mut name = vec![name_tok.text.clone()];
            while r.peek().kind == TokenKind::Punct && r.peek().text == "." {
                r.advance();
                let n = r.peek();
                if n.kind != TokenKind::Ident {
                    return Err(CapyError::msg("expected identifier after ."));
                }
                r.advance();
                name.push(n.text.clone());
            }
            let mut args: Vec<Expr> = Vec::new();
            while r.peek().kind != TokenKind::RParen {
                let p = r.peek();
                if (p.kind == TokenKind::Punct && p.text == ",") || p.kind == TokenKind::Newline {
                    r.advance();
                    continue;
                }
                args.push(parse_primary(r, &[])?);
            }
            r.advance();
            Ok(Expr::Call(CallExpr { name, args }))
        }
        TokenKind::LBrack => parse_list_lit(r),
        TokenKind::LBrace => parse_obj_lit(r),
        _ => Err(CapyError::msg(format!(
            "line {}: unexpected token {} in value",
            t.line,
            gofmt::quote(&t.text)
        ))),
    }
    .map_err(|e| {
        // `stop` is unused past this point in Go too; silence the parameter
        // without changing behaviour.
        let _ = stop;
        e
    })
}

/// Port of `parseListLit`.
fn parse_list_lit<R: TokReader + ?Sized>(r: &mut R) -> Result<Expr, CapyError> {
    r.advance();
    let mut items: Vec<Expr> = Vec::new();
    while r.peek().kind != TokenKind::RBrack {
        let p = r.peek();
        if (p.kind == TokenKind::Punct && p.text == ",") || p.kind == TokenKind::Newline {
            r.advance();
            continue;
        }
        items.push(parse_primary(r, &[])?);
    }
    r.advance();
    Ok(Expr::List(items))
}

/// Port of `parseObjLit`.
fn parse_obj_lit<R: TokReader + ?Sized>(r: &mut R) -> Result<Expr, CapyError> {
    r.advance();
    let mut keys: Vec<String> = Vec::new();
    let mut vals: Vec<Expr> = Vec::new();
    while r.peek().kind != TokenKind::RBrace {
        let p = r.peek();
        if (p.kind == TokenKind::Punct && p.text == ",") || p.kind == TokenKind::Newline {
            r.advance();
            continue;
        }
        let k_tok = r.peek();
        // Accept either a quoted string OR a bare identifier as a key.
        // `{name: "p", "age": 34}` is valid — same as `{"name": "p", …}`.
        if k_tok.kind != TokenKind::Str && k_tok.kind != TokenKind::Ident {
            return Err(CapyError::msg(format!(
                "line {}: object keys must be strings or identifiers",
                k_tok.line
            )));
        }
        r.advance();
        let nx = r.peek();
        if !(nx.kind == TokenKind::Punct && nx.text == ":") {
            return Err(CapyError::msg(format!("line {}: expected ':'", nx.line)));
        }
        r.advance();
        skip_newlines(r);
        let v = parse_primary(r, &[])?;
        keys.push(k_tok.text.clone());
        vals.push(v);
    }
    r.advance();
    Ok(Expr::Obj(ObjLit { keys, vals }))
}

/// Port of `skipNewlines`.
pub fn skip_newlines<R: TokReader + ?Sized>(r: &mut R) {
    while r.peek().kind == TokenKind::Newline {
        r.advance();
    }
}

/// A concrete [`TokReader`] over a token slice — used by the inner-DSL parser
/// and by tests.
pub struct SliceReader {
    pub toks: Vec<Token>,
    pub pos: usize,
}

impl SliceReader {
    pub fn new(toks: Vec<Token>) -> SliceReader {
        SliceReader { toks, pos: 0 }
    }

    fn eof_token(&self) -> Token {
        Token {
            kind: TokenKind::Eof,
            text: String::new(),
            line: 0,
            col: 0,
            width: 0,
        }
    }
}

impl TokReader for SliceReader {
    fn peek(&self) -> Token {
        self.toks.get(self.pos).cloned().unwrap_or_else(|| self.eof_token())
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
    fn restore(&mut self, p: usize) {
        self.pos = p;
    }
}
