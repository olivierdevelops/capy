//! Port of the Go `domain` package — pure data model, no I/O.

pub mod ast;
pub mod ast_json;
pub mod command;
pub mod docs;
pub mod errors;
pub mod host;
pub mod impl_def;
pub mod library;
pub mod token;
pub mod val;
pub mod value;

pub use ast::*;
pub use command::{CommandArg, CommandDef, CommandFlag};
pub use docs::render_library_docs;
pub use errors::{format_with_source, suggest_closest, suggest_closest_sorted, CapyError};
pub use host::{Host, NoOpHost};
pub use impl_def::ImplDef;
pub use library::{
    ArgEntry, BlockSpec, CloseSegment, FuncDef, Library, Lookahead, PatternElement, TypeDef,
};
pub use token::{Token, TokenKind};
pub use val::Val;
