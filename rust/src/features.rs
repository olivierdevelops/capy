//! Port of the Go `features` package (`lexer.go`, `parser.go`, `evaluator.go`,
//! `library_loader.go`).
//!
//! Go declares each engine capability as a struct of function pointers so the
//! orchestrator can inject implementations. The structs hold no state and no
//! logic — they exist purely to invert the dependency. Rust expresses the same
//! seam with function-pointer type aliases, which is what the injected fields
//! were in all but name.

use crate::domain::ast::Block;
use crate::domain::errors::CapyError;
use crate::domain::library::Library;
use crate::domain::token::Token;
use std::collections::BTreeMap;

/// Port of `features.Lexer.Tokenize` — lexes using the engine-default comment
/// markers (`#`), for the manifest format and inner-DSL bodies.
pub type TokenizeFn = fn(&str) -> Result<Vec<Token>, CapyError>;

/// Port of `features.Lexer.TokenizeWith` — lexes USER SCRIPTS with the library's
/// declared markers. An empty list means no comment syntax at all.
pub type TokenizeWithFn = fn(&str, &[String]) -> Result<Vec<Token>, CapyError>;

/// Port of `features.Parser.Parse`. `src` is the original source the tokens came
/// from — needed so `block_verbatim` bodies can capture the raw byte range.
pub type ParseFn = fn(Vec<Token>, &str, &Library) -> Result<Block, CapyError>;

/// Port of `features.Evaluator.Run`.
pub type EvaluateFn = fn(&Block, &Library) -> Result<String, CapyError>;

/// Port of `features.Evaluator.RunMulti` — returns the file_template-rendered
/// output AND every `file "path"` template rendered against the same context.
pub type EvaluateMultiFn =
    fn(&Block, &Library) -> Result<(String, BTreeMap<String, String>), CapyError>;

/// Port of `features.LibraryLoader.Load`.
pub type LoadLibFn = fn(&str, TokenizeFn) -> Result<Library, CapyError>;

/// Port of `usecases.ReadFileFn`.
pub type ReadFileFn = fn(&str) -> Result<String, String>;

/// Port of the `writeOut` field threaded through `MakeRunScript`.
pub type WriteOutFn = fn(&str, &str) -> Result<(), String>;
