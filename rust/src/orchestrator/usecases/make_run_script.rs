//! Port of `orchestrator/usecases/make_run_script.go`.
//!
//! NOTE: this pipeline is NOT the one `orchestrator::run` implements, and the
//! differences are real. `Execute` below:
//!
//!   * tokenizes with the DEFAULT comment markers, not `lib.comments`
//!   * does NOT run the preprocessor or `define` extraction
//!   * WRITES `lib.output_file` when the library declares one
//!   * tolerates an empty library path, yielding an empty library
//!
//! In the Go tree this path is reachable only through `AppOrchestrator.RunCLI`,
//! which nothing calls — the real CLI goes through `orchestrator.RunMulti`. It is
//! ported faithfully anyway so the two implementations stay comparable.

use crate::domain::library::Library;
use crate::features::{
    EvaluateFn, EvaluateMultiFn, LoadLibFn, ParseFn, ReadFileFn, TokenizeFn, WriteOutFn,
};
use crate::io::cli::RunOutcome;
use crate::usecases::RunResult;

/// Port of `RunScriptImpl`.
pub struct RunScriptImpl {
    read: ReadFileFn,
    tokenize: TokenizeFn,
    parse: ParseFn,
    evaluate: EvaluateFn,
    eval_multi: Option<EvaluateMultiFn>,
    load_lib: LoadLibFn,
    write_out: WriteOutFn,
}

/// Port of `MakeRunScript` — single-output wiring.
#[allow(clippy::too_many_arguments)]
pub fn make_run_script(
    read: ReadFileFn,
    tokenize: TokenizeFn,
    parse: ParseFn,
    evaluate: EvaluateFn,
    load_lib: LoadLibFn,
    write_out: WriteOutFn,
) -> RunScriptImpl {
    RunScriptImpl { read, tokenize, parse, evaluate, eval_multi: None, load_lib, write_out }
}

/// Port of `MakeRunScriptMulti` — wires both the single-file evaluator (for
/// backwards compatibility) and the multi-file one. The latter is used when the
/// library declared any `file "path"` blocks.
#[allow(clippy::too_many_arguments)]
pub fn make_run_script_multi(
    read: ReadFileFn,
    tokenize: TokenizeFn,
    parse: ParseFn,
    evaluate: EvaluateFn,
    eval_multi: EvaluateMultiFn,
    load_lib: LoadLibFn,
    write_out: WriteOutFn,
) -> RunScriptImpl {
    RunScriptImpl {
        read,
        tokenize,
        parse,
        evaluate,
        eval_multi: Some(eval_multi),
        load_lib,
        write_out,
    }
}

impl RunScriptImpl {
    /// Port of `(*RunScriptImpl).Execute`.
    pub fn execute(&self, script_path: &str, library_path: &str) -> Result<RunResult, String> {
        // Go starts from a zero-valued Library with initialised maps, so an empty
        // library path yields a library with no functions rather than an error.
        let mut lib = Library::default();
        if !library_path.is_empty() {
            lib = (self.load_lib)(library_path, self.tokenize).map_err(|e| e.to_string())?;
        }
        let src = (self.read)(script_path)?;
        let toks = (self.tokenize)(&src).map_err(|e| e.to_string())?;
        let prog = (self.parse)(toks, &src, &lib).map_err(|e| e.to_string())?;

        let (out, files) = match self.eval_multi {
            Some(f) => f(&prog, &lib).map_err(|e| e.to_string())?,
            None => {
                let o = (self.evaluate)(&prog, &lib).map_err(|e| e.to_string())?;
                (o, Default::default())
            }
        };
        if !lib.output_file.is_empty() {
            (self.write_out)(&lib.output_file, &out)?;
        }
        Ok(RunResult { output: out, output_file: lib.output_file.clone(), files })
    }

    /// Port of `cliAdapter.Execute` + `AsCLIUseCase` — adapts the use case to the
    /// shape the CLI view model consumes.
    pub fn as_cli_use_case(&self) -> impl Fn(&str, &str) -> Result<RunOutcome, String> + '_ {
        move |script, lib| {
            let res = self.execute(script, lib)?;
            Ok(RunOutcome {
                output: res.output,
                output_file: res.output_file,
                files: res.files,
            })
        }
    }
}
