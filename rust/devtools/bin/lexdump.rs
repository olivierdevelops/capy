//! Differential-testing aid: dumps the Rust lexer's token stream in the exact
//! format `cmd/lexdump` (Go) produces, so the two can be diffed byte-for-byte.

use capy_core::domain::token::{Token, TokenKind};
use capy_core::gofmt;
use capy_core::orchestrator::features::make_lexer;

fn kind_name(k: TokenKind) -> &'static str {
    match k {
        TokenKind::Ident => "IDENT",
        TokenKind::Number => "NUMBER",
        TokenKind::Str => "STRING",
        TokenKind::Template => "TEMPLATE",
        TokenKind::Punct => "PUNCT",
        TokenKind::LParen => "LPAREN",
        TokenKind::RParen => "RPAREN",
        TokenKind::LBrace => "LBRACE",
        TokenKind::RBrace => "RBRACE",
        TokenKind::LBrack => "LBRACK",
        TokenKind::RBrack => "RBRACK",
        TokenKind::Newline => "NEWLINE",
        TokenKind::Indent => "INDENT",
        TokenKind::Dedent => "DEDENT",
        TokenKind::Eof => "EOF",
    }
}

fn main() {
    let mut out = String::new();
    for path in std::env::args().skip(1) {
        match std::fs::read_to_string(&path) {
            Err(e) => {
                out.push_str(&format!("FILE\t{}\nREADERR\t{}\n", path, e));
                continue;
            }
            Ok(src) => {
                out.push_str(&format!("FILE\t{}\n", path));
                match make_lexer::tokenize(&src) {
                    Err(e) => out.push_str(&format!("ERR\t{}\n", e)),
                    Ok(toks) => {
                        for t in &toks {
                            let t: &Token = t;
                            out.push_str(&format!(
                                "T\t{}\t{}\t{}\t{}\t{}\n",
                                kind_name(t.kind),
                                gofmt::quote(&t.text),
                                t.line,
                                t.col,
                                t.width
                            ));
                        }
                    }
                }
            }
        }
    }
    print!("{}", out);
}
