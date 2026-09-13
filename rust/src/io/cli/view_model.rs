//! Port of `io/cli/view_model.go`.

use super::RunOutcome;

/// Port of `CLIState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliState {
    #[default]
    Idle,
    Running,
    Success,
    Error,
}

/// Port of `CLIViewModel`.
#[derive(Debug, Clone, Default)]
pub struct CliViewModel {
    pub state: CliState,
    pub output: String,
    pub file: String,
    pub err_msg: String,
}

impl CliViewModel {
    /// Port of `NewCLIViewModel`.
    pub fn new() -> CliViewModel {
        CliViewModel { state: CliState::Idle, ..Default::default() }
    }

    /// Port of `(*CLIViewModel).Run`.
    ///
    /// Go holds the use case as a field; here the caller supplies it so the view
    /// model stays a plain data holder.
    pub fn run<F>(&mut self, script_path: &str, library_path: &str, execute: F)
    where
        F: FnOnce(&str, &str) -> Result<RunOutcome, String>,
    {
        self.state = CliState::Running;
        match execute(script_path, library_path) {
            Err(e) => {
                self.state = CliState::Error;
                self.err_msg = e;
            }
            Ok(res) => {
                self.state = CliState::Success;
                self.output = res.output;
                self.file = res.output_file;
            }
        }
    }
}
