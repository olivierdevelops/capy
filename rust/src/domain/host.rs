//! Port of `domain/host.go`.
//!
//! This is the seam that becomes the wasm boundary: under wazero these methods
//! are the host functions exported INTO the module, because a wasm module
//! cannot read the environment or spawn a process on its own.

use super::errors::CapyError;

/// The small, intentional surface a Capy library can ask of its embedder. It
/// lets a library author pull values from outside the source file — environment
/// variables, CLI arguments, sibling files — and weave them into the
/// accumulating context.
///
/// Capy is still a transpiler: nothing here executes user-script code. `env`,
/// `arg`, `read_file` are READ-ONLY primitives that surface already-existing
/// host data so the library can incorporate it.
///
/// Implementations:
///   * `infra::OsHost` — backed by real env / args / filesystem. Used by the
///     CLI. Lets you build configs that depend on the deployment environment.
///   * [`NoOpHost`] — every method returns the zero value or an error. Default
///     for embedded callers and the wasm playground, where exposing the
///     embedder's filesystem/env would be a security hazard.
///   * tests can stub a `Host` to make builds deterministic.
pub trait Host {
    /// Returns the OS environment variable named `name`, or "" if unset.
    fn env(&self, name: &str) -> String;

    /// Returns the i-th positional CLI argument (zero-indexed), or "" if out of
    /// range. Index 0 is the first user-supplied arg AFTER the script path —
    /// `capy run lib.capy script.capy a b c` gives `arg(0) == "a"`.
    fn arg(&self, i: usize) -> String;

    /// How many positional CLI args were supplied.
    fn arg_count(&self) -> usize;

    /// A copy of the full positional CLI args slice.
    fn args(&self) -> Vec<String>;

    /// Contents of the file at `path`. Relative paths are resolved by the
    /// implementation (usually against the script's directory). An error aborts
    /// the transpilation with a clear message.
    fn read_file(&self, path: &str) -> Result<String, CapyError>;

    /// The lowercase host operating-system identifier: "linux", "darwin",
    /// "windows", "freebsd", "js" (wasm), etc. Matches Go's `runtime.GOOS` so
    /// libraries can branch on it.
    fn os(&self) -> String;

    /// The lowercase host architecture: "amd64", "arm64", "wasm", etc. Matches
    /// Go's `runtime.GOARCH`.
    fn arch(&self) -> String;

    /// The host's current working directory at the time transpilation started.
    fn cwd(&self) -> Result<String, CapyError>;

    /// The host user's home directory (`$HOME` on POSIX, `%USERPROFILE%` on
    /// Windows).
    fn home_dir(&self) -> Result<String, CapyError>;

    // --- Side-effecting primitives (library commands only) ---

    /// Creates (or overwrites) the file at `path` with `contents`. Parent
    /// directories are created as needed.
    fn write_file(&self, path: &str, contents: &str) -> Result<(), CapyError>;

    /// Creates the directory at `path` (and parents). Idempotent.
    fn mkdir(&self, path: &str) -> Result<(), CapyError>;

    /// Returns the path to a freshly-created temp file with the given suffix
    /// (e.g. ".py"). The caller is responsible for removing it.
    fn mk_temp(&self, suffix: &str) -> Result<String, CapyError>;

    /// Returns the path to a freshly-created temp directory.
    fn mk_temp_dir(&self) -> Result<String, CapyError>;

    /// Runs the named command with args, streaming its stdout and stderr to the
    /// calling process's stdout/stderr. Returns an error on non-zero exit.
    fn exec(&self, name: &str, args: &[String]) -> Result<(), CapyError>;

    /// Runs the named command and returns its combined output.
    fn exec_capture(&self, name: &str, args: &[String]) -> Result<String, CapyError>;
}

/// Port of `domain.NoOpHost`. Satisfies [`Host`] with empty/zero results
/// everywhere. The safe default for sandboxed embedders (wasm playground,
/// third-party programs that don't want their host filesystem exposed to
/// library authors).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpHost;

/// Port of the Go package-level `errNoSandboxedFS`.
fn err_no_sandboxed_fs() -> CapyError {
    CapyError::structured("filesystem / exec not available in this runtime").with_hint(
        "library commands require infra.OSHost; the WASM / embedded sandbox doesn't provide it",
    )
}

impl Host for NoOpHost {
    fn env(&self, _name: &str) -> String {
        String::new()
    }
    fn arg(&self, _i: usize) -> String {
        String::new()
    }
    fn arg_count(&self) -> usize {
        0
    }
    fn args(&self) -> Vec<String> {
        Vec::new()
    }
    fn read_file(&self, _path: &str) -> Result<String, CapyError> {
        Err(CapyError::structured("read_file: host has no filesystem access").with_hint(
            "this Capy build runs in a sandbox; pass --allow-fs on the CLI or supply an OSHost when embedding",
        ))
    }
    fn os(&self) -> String {
        String::new()
    }
    fn arch(&self) -> String {
        String::new()
    }
    fn cwd(&self) -> Result<String, CapyError> {
        Ok(String::new())
    }
    fn home_dir(&self) -> Result<String, CapyError> {
        Ok(String::new())
    }
    fn write_file(&self, _path: &str, _contents: &str) -> Result<(), CapyError> {
        Err(err_no_sandboxed_fs())
    }
    fn mkdir(&self, _path: &str) -> Result<(), CapyError> {
        Err(err_no_sandboxed_fs())
    }
    fn mk_temp(&self, _suffix: &str) -> Result<String, CapyError> {
        Err(err_no_sandboxed_fs())
    }
    fn mk_temp_dir(&self) -> Result<String, CapyError> {
        Err(err_no_sandboxed_fs())
    }
    fn exec(&self, _name: &str, _args: &[String]) -> Result<(), CapyError> {
        Err(err_no_sandboxed_fs())
    }
    fn exec_capture(&self, _name: &str, _args: &[String]) -> Result<String, CapyError> {
        Err(err_no_sandboxed_fs())
    }
}
