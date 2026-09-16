//! Port of `domain/token.go`.
//!
//! Tokens are LEXICAL only. There are no grammar keywords (no if/loop/end/=).
//! Identifier-like words are `Ident`. Operators (=, ==, <, >, :, ., ,, !=, <=,
//! >=, +, -, ...) are `Punct` with their literal text. The library decides what
//! > any of those mean.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Number,
    /// `"..."` or `'...'` — content stored raw, supports `${}` at eval time.
    Str,
    /// `` `...` `` — same: content stored raw, supports `${}` at eval time.
    Template,
    /// `= == != < > <= >= , : . + - * /` etc.
    Punct,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    /// PLAN-2026-0001 R27 — a retained source comment. Emitted ONLY by
    /// [`tokenize_with_trivia`](crate::orchestrator::features::make_lexer::tokenize_with_trivia),
    /// and stripped from the stream by the parser before matching, so no
    /// matcher ever sees one. `text` is the comment verbatim, marker included.
    Comment,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
    /// Raw byte width of the lexeme as it appeared in source, including any
    /// surrounding quotes for string/template tokens. `text` for a string strips
    /// the quotes, so `width` (not `text.len()`) is the authoritative source
    /// span — `tail` uses it to compute inter-token spacing and to know a token
    /// was quoted. Zero means "unset"; consumers fall back to `text.len()`.
    pub width: usize,
}

impl Token {
    /// Mirrors Go's `len(tok.Text)` fallback when `Width` is unset.
    pub fn span(&self) -> usize {
        if self.width == 0 {
            self.text.len()
        } else {
            self.width
        }
    }
}
