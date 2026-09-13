//! Port of `infra/os_host.go`.
//!
//! The [`Host`] implementation backed by real OS primitives. Used by the CLI so
//! libraries can read env vars, positional CLI args, and sibling files. NOT used
//! by the wasm playground or default embedded callers — exposing the embedder's
//! filesystem from a library would be a security hazard.

use crate::domain::errors::CapyError;
use crate::domain::host::Host;
use crate::gopath;
use std::fs;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Default)]
pub struct OsHost {
    /// Positional CLI args AFTER the library + script paths have been stripped,
    /// so `arg(0)` is the first user arg, not the binary name.
    pub user_args: Vec<String>,
    /// The script's directory; `read_file` resolves relative paths against it so
    /// libraries can ask for `read_file "config.toml"` and get the file next to
    /// the script, not next to the binary.
    pub base_dir: String,
}

impl Host for OsHost {
    fn env(&self, name: &str) -> String {
        std::env::var(name).unwrap_or_default()
    }

    fn arg(&self, i: usize) -> String {
        // Go takes a signed int and guards `i < 0`; the caller maps a negative
        // index to usize::MAX, which fails this bound the same way.
        self.user_args.get(i).cloned().unwrap_or_default()
    }

    fn arg_count(&self) -> usize {
        self.user_args.len()
    }

    fn args(&self) -> Vec<String> {
        self.user_args.clone()
    }

    fn os(&self) -> String {
        // Matches Go's runtime.GOOS spelling.
        if cfg!(target_os = "macos") {
            "darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            std::env::consts::OS.to_string()
        }
    }

    fn arch(&self) -> String {
        // Matches Go's runtime.GOARCH spelling.
        match std::env::consts::ARCH {
            "x86_64" => "amd64".to_string(),
            "aarch64" => "arm64".to_string(),
            "x86" => "386".to_string(),
            other => other.to_string(),
        }
    }

    fn cwd(&self) -> Result<String, CapyError> {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|e| CapyError::structured(gopath::io_error("getwd", ".", &e)))
    }

    fn home_dir(&self) -> Result<String, CapyError> {
        match std::env::var("HOME") {
            Ok(h) if !h.is_empty() => Ok(h),
            _ => match std::env::var("USERPROFILE") {
                Ok(h) if !h.is_empty() => Ok(h),
                _ => Err(CapyError::structured("$HOME is not defined")),
            },
        }
    }

    fn write_file(&self, path: &str, contents: &str) -> Result<(), CapyError> {
        let dir = gopath::dir(path);
        if !dir.is_empty() {
            if let Err(e) = fs::create_dir_all(&dir) {
                return Err(CapyError::structured(format!(
                    "write_file {}: mkdir parent: {}",
                    crate::gofmt::quote(path),
                    gopath::io_error("mkdir", &dir, &e)
                )));
            }
        }
        fs::write(path, contents.as_bytes()).map_err(|e| {
            CapyError::structured(format!(
                "write_file {}: {}",
                crate::gofmt::quote(path),
                gopath::io_error("open", path, &e)
            ))
        })
    }

    fn mkdir(&self, path: &str) -> Result<(), CapyError> {
        fs::create_dir_all(path).map_err(|e| {
            CapyError::structured(format!(
                "mkdir {}: {}",
                crate::gofmt::quote(path),
                gopath::io_error("mkdir", path, &e)
            ))
        })
    }

    fn mk_temp(&self, suffix: &str) -> Result<String, CapyError> {
        let dir = std::env::temp_dir();
        for _ in 0..1000 {
            let name = dir.join(format!("capy-{}{}", random_token(), suffix));
            match fs::OpenOptions::new().write(true).create_new(true).open(&name) {
                Ok(_) => return Ok(name.to_string_lossy().into_owned()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => {
                    return Err(CapyError::structured(format!(
                        "mktemp: {}",
                        gopath::io_error("open", &name.to_string_lossy(), &e)
                    )))
                }
            }
        }
        Err(CapyError::structured("mktemp: exhausted attempts"))
    }

    fn mk_temp_dir(&self) -> Result<String, CapyError> {
        let dir = std::env::temp_dir();
        for _ in 0..1000 {
            let name = dir.join(format!("capy-{}", random_token()));
            match fs::create_dir(&name) {
                Ok(_) => return Ok(name.to_string_lossy().into_owned()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => {
                    return Err(CapyError::structured(format!(
                        "mktemp_dir: {}",
                        gopath::io_error("mkdir", &name.to_string_lossy(), &e)
                    )))
                }
            }
        }
        Err(CapyError::structured("mktemp_dir: exhausted attempts"))
    }

    fn exec(&self, name: &str, args: &[String]) -> Result<(), CapyError> {
        let status = Command::new(name)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status();
        match status {
            Ok(st) if st.success() => Ok(()),
            Ok(st) => Err(CapyError::structured(format!(
                "exec {}: {}",
                name,
                go_exit_error(st.code())
            ))),
            Err(e) => Err(CapyError::structured(format!("exec {}: {}", name, e))),
        }
    }

    fn exec_capture(&self, name: &str, args: &[String]) -> Result<String, CapyError> {
        match Command::new(name).args(args).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
                if out.status.success() {
                    Ok(stdout)
                } else {
                    Err(CapyError::structured(format!(
                        "exec {}: {}\n{}",
                        name,
                        go_exit_error(out.status.code()),
                        stderr
                    )))
                }
            }
            Err(e) => Err(CapyError::structured(format!("exec {}: {}", name, e))),
        }
    }

    fn read_file(&self, path: &str) -> Result<String, CapyError> {
        let mut resolved = path.to_string();
        if !gopath::is_abs(&resolved) && !self.base_dir.is_empty() {
            resolved = gopath::join(&[&self.base_dir, path]);
        }
        fs::read_to_string(&resolved).map_err(|e| {
            CapyError::structured(format!(
                "read_file {}: {}",
                crate::gofmt::quote(path),
                gopath::io_error("open", &resolved, &e)
            ))
            .with_hint("path is resolved relative to the script's directory")
        })
    }
}

/// Go's `*exec.ExitError` renders as `exit status N`.
fn go_exit_error(code: Option<i32>) -> String {
    match code {
        Some(c) => format!("exit status {}", c),
        None => "signal: killed".to_string(),
    }
}

/// A short random token for temp names, mirroring `os.CreateTemp`'s random
/// suffix (Go substitutes the `*` in the pattern).
fn random_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ d.as_secs())
        .unwrap_or(0);
    let pid = std::process::id() as u64;
    format!("{}", nanos.wrapping_mul(2654435761).wrapping_add(pid) % 1_000_000_000)
}
