//! Port of the Go `io/cli` package.

pub mod view;
pub mod view_model;

pub use view::CliView;
pub use view_model::{CliState, CliViewModel};

use std::collections::BTreeMap;

/// Port of `RunOutcome` in `io/cli/cli_usecases.go`.
///
/// Go also declares a `RunScriptUseCase` interface here (consumer-owned, next to
/// the view model); Rust passes the closure directly instead.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunOutcome {
    pub output: String,
    pub output_file: String,
    /// Populated when the library declared `file "path"` blocks.
    pub files: BTreeMap<String, String>,
}
