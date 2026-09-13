//! Port of `domain/command.go`.

use super::ast::InnerBlock;

/// One library-declared command — a custom verb the CLI exposes as
/// `capy <lib> <name> [args]`. The command body is an extended inner-DSL
/// program with shell-like primitives available: exec, write_file, mktemp,
/// mktemp_dir, cd, print, let, etc.
///
/// The body runs against a per-invocation execution context holding `args`
/// (positional CLI args after the command name), `flags`, and locals
/// introduced via `let X = …`.
///
/// A library can override built-in commands (run, compile, check, docs) by
/// declaring its own command with the same name. If a library declares no
/// commands, the built-in default for `run` is used.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandDef {
    pub name: String,
    pub description: String,
    pub body: InnerBlock,
    /// Body raw text — kept around so tooling can re-tokenise / re-parse.
    /// Empty after the first compile.
    pub body_raw: String,
    /// Positional arguments declared via `arg "name" required "desc"`.
    /// Used to generate --help and to validate invocations.
    pub args: Vec<CommandArg>,
    /// Named flags declared via `flag "--name" "desc" default "v"`.
    pub flags: Vec<CommandFlag>,
}

/// A positional argument declaration.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandArg {
    pub name: String,
    pub required: bool,
    pub description: String,
}

/// A flag declaration. Bool flags are presence-only; string flags accept
/// `--name VALUE` or `--name=VALUE`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandFlag {
    /// Includes the leading dashes, e.g. `--port`.
    pub name: String,
    pub description: String,
    pub default: String,
    pub is_bool: bool,
}
