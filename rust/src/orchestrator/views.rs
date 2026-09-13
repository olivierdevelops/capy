//! Port of `orchestrator/views/make_cli_view.go`.

use crate::io::cli::{CliViewModel, RunOutcome};

/// Port of `MakeCLIView`.
///
/// Builds the view model, runs it, and hands it back ready to render. Go returns
/// a `CLIView` holding a pointer to the model; Rust returns the model so the
/// caller can borrow it into a `CliView` without a self-referential struct.
pub fn make_cli_view<F>(execute: F, script_path: &str, library_path: &str) -> CliViewModel
where
    F: FnOnce(&str, &str) -> Result<RunOutcome, String>,
{
    let mut vm = CliViewModel::new();
    vm.run(script_path, library_path, execute);
    vm
}
