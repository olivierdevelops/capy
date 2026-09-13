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
