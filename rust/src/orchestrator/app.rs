//! Port of `orchestrator/app.go`.
//!
//! NOTE: nothing in the Go tree calls `AppOrchestrator.RunCLI` — the real CLI
//! goes through `orchestrator.RunMulti`. Ported for completeness, and because its
//! pipeline differs from `run_multi`'s in ways worth keeping visible (see
//! `make_run_script`).

use super::features::{make_evaluator, make_lexer, make_library_loader, make_parser};
use super::usecases::make_run_script::make_run_script_multi;
use super::views::make_cli_view;
use crate::infra::file_reader::FileReader;
use crate::io::cli::CliView;

/// Port of `AppOrchestrator`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppOrchestrator;

/// Port of the `files.Read` method value Go passes in.
fn read_file(path: &str) -> Result<String, String> {
    FileReader.read(path)
}

/// Port of the `files.Write` method value Go passes in.
fn write_file(path: &str, content: &str) -> Result<(), String> {
    FileReader.write(path, content)
}

impl AppOrchestrator {
    /// Port of `(AppOrchestrator).RunCLI`.
    pub fn run_cli(&self, script_path: &str, library_path: &str) -> i32 {
        let rs = make_run_script_multi(
            read_file,
            make_lexer::tokenize,
            make_parser::parse,
            make_evaluator::run,
            make_evaluator::run_noop_host_multi,
            make_library_loader::load_library,
            write_file,
        );
        let vm = make_cli_view(rs.as_cli_use_case(), script_path, library_path);
        CliView { vm: &vm }.render_to_stdio()
    }
}
