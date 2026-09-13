//! Port of `usecases/run_script.go`.
//!
//! Go declares the engine's capabilities as a set of function-pointer type
//! aliases (`TokenizeFn`, `ParseFn`, …) so the orchestrator can inject them.
//! Rust calls the concrete functions directly, so only the data types carry
//! over; the aliases would be pure indirection.

use std::collections::BTreeMap;

/// Port of `RunResult`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunResult {
    pub output: String,
    pub output_file: String,
    /// Populated when the library declared `file "path"` blocks.
    pub files: BTreeMap<String, String>,
}

/// Port of the `RunScript` interface.
pub trait RunScript {
    fn execute(&self, script_path: &str, library_path: &str) -> Result<RunResult, String>;
}
