//! Port of `io/cli/view.go`.

use super::view_model::{CliState, CliViewModel};

/// Port of `CLIView` — the dumb view: it observes a [`CliViewModel`] and
/// renders, mapping state enums to user-visible text. No business logic.
pub struct CliView<'a> {
    pub vm: &'a CliViewModel,
}

impl<'a> CliView<'a> {
    /// Port of `(CLIView).Render`.
    ///
    /// Go writes to `Stdout`/`Stderr` writers and returns the process exit code.
    /// This returns `(stdout, stderr, exit_code)` so the text is testable, and
    /// [`CliView::render_to_stdio`] does the actual printing.
    pub fn render(&self) -> (String, String, i32) {
        match self.vm.state {
            CliState::Idle => (String::new(), "capy: nothing to do\n".to_string(), 1),
            CliState::Running => (String::new(), "capy: still running\n".to_string(), 1),
            CliState::Success => {
                let mut err = String::new();
                if !self.vm.file.is_empty() {
                    err.push_str(&format!("wrote {}\n", self.vm.file));
                }
                (self.vm.output.clone(), err, 0)
            }
            CliState::Error => {
                (String::new(), format!("capy: {}\n", self.vm.err_msg), 1)
            }
        }
    }

    /// Prints the rendered text to the real stdio and returns the exit code.
    pub fn render_to_stdio(&self) -> i32 {
        let (out, err, code) = self.render();
        eprint!("{}", err);
        print!("{}", out);
        code
    }
}
