//! Port of `cmd/capy/cmd_build.go`.
//!
//! Produces a standalone executable with a library baked in. Run the executable
//! against any script and it dispatches to the library's commands — no `capy`
//! install required on the target host.
//!
//! IMPORTANT, and unchanged from the Go original by design: this writes a small
//! Go `main.go` that embeds the library source and imports
//! `github.com/olivierdevelops/capy/orchestrator`, then shells out to
//! `go mod tidy` + `go build`. So:
//!
//!   * running `capy build` needs a Go toolchain (as it always did)
//!   * the OUTPUT binary is powered by the GO engine, not this Rust one
//!
//! Porting it faithfully preserves the feature exactly. Retargeting the generated
//! program at the Rust engine would be a design change, not a port — it needs a
//! Rust template plus a `cargo build` path, and a decision about how the library
//! source gets embedded.

use super::flags::{self, reorder_flags_first, Spec};
use super::lib_path::resolve_lib;
use capy_core::gofmt;
use capy_core::gopath;
use std::process::Command;

const BUILD_SPEC: Spec = Spec { bools: &["keep-temp"], strings: &["o"] };

/// Port of `cmdBuild`.
pub fn cmd_build(args: &[String]) -> Result<(), String> {
    // Go's flag package stops at the first positional, so flags move to the front.
    let args = reorder_flags_first(args);
    let fs = flags::parse(&BUILD_SPEC, &args)?;
    let pos = &fs.positionals;
    if pos.len() != 1 {
        return Err("usage: capy build <library> [-o <output>]".to_string());
    }
    let lib_name = &pos[0];

    // Resolve the library — by name on CAPY_LIBS, or by direct path.
    let lib_path = match resolve_lib(lib_name) {
        Ok(p) => p,
        Err(_) => {
            if std::fs::metadata(lib_name).is_ok() {
                lib_name.clone()
            } else {
                return Err(format!("library {:?} not found", lib_name));
            }
        }
    };

    let lib_src = std::fs::read_to_string(&lib_path)
        .map_err(|e| format!("read library: {}", gopath::io_error("open", &lib_path, &e)))?;

    // Default output path.
    let base = gopath::base(&lib_path);
    let ext = gopath::ext(&lib_path);
    let resolved_name = base.strip_suffix(&ext).unwrap_or(&base).to_string();
    let mut out = fs.str("o").to_string();
    if out.is_empty() {
        out = format!("./{}", resolved_name);
    }

    // Materialise a temp Go module with the library source embedded.
    let tmp_dir = make_temp_dir("capy-build-")?;
    let keep = fs.bool("keep-temp");

    // Locate the local capy module root so we can use a `replace` directive.
    let capy_root = find_capy_module_root();

    let main_go_path = gopath::join(&[&tmp_dir, "main.go"]);
    std::fs::write(&main_go_path, build_main_go(&resolved_name, &lib_src).as_bytes())
        .map_err(|e| format!("write main.go: {}", gopath::io_error("open", &main_go_path, &e)))?;

    let gomod = if !capy_root.is_empty() {
        format!(
            "module capybuild\n\ngo 1.22\n\nrequire github.com/olivierdevelops/capy v0.0.0\nreplace github.com/olivierdevelops/capy => {}\n",
            capy_root
        )
    } else {
        "module capybuild\n\ngo 1.22\n\nrequire github.com/olivierdevelops/capy latest\n".to_string()
    };
    let gomod_path = gopath::join(&[&tmp_dir, "go.mod"]);
    std::fs::write(&gomod_path, gomod.as_bytes())
        .map_err(|e| format!("write go.mod: {}", gopath::io_error("open", &gomod_path, &e)))?;
    // With a local checkout, copy its go.sum so deps don't re-resolve.
    if !capy_root.is_empty() {
        if let Ok(sum) = std::fs::read(gopath::join(&[&capy_root, "go.sum"])) {
            let _ = std::fs::write(gopath::join(&[&tmp_dir, "go.sum"]), sum);
        }
    }

    let cleanup = |dir: &str, keep: bool| {
        if !keep {
            let _ = std::fs::remove_dir_all(dir);
        }
    };

    // `go mod tidy` to settle deps for the new module.
    let tidy = Command::new("go").arg("mod").arg("tidy").current_dir(&tmp_dir).status();
    match tidy {
        Ok(st) if st.success() => {}
        Ok(st) => {
            cleanup(&tmp_dir, keep);
            return Err(format!(
                "go mod tidy failed: exit status {}",
                st.code().unwrap_or(-1)
            ));
        }
        Err(e) => {
            cleanup(&tmp_dir, keep);
            return Err(format!("go mod tidy failed: {}", e));
        }
    }

    // `go build`.
    let abs_out = if gopath::is_abs(&out) {
        gopath::clean(&out)
    } else {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        gopath::join(&[&cwd, &out])
    };
    eprintln!("building {} (this needs the Go toolchain)…", resolved_name);
    let build = Command::new("go")
        .args(["build", "-o", &abs_out, "./..."])
        .current_dir(&tmp_dir)
        .status();
    match build {
        Ok(st) if st.success() => {}
        Ok(st) => {
            cleanup(&tmp_dir, keep);
            return Err(format!("go build failed: exit status {}", st.code().unwrap_or(-1)));
        }
        Err(e) => {
            cleanup(&tmp_dir, keep);
            return Err(format!("go build failed: {}", e));
        }
    }
    let size = std::fs::metadata(&abs_out).map(|m| m.len()).unwrap_or(0);
    eprintln!("✓ wrote {} ({:.1} MB)", out, size as f64 / 1024.0 / 1024.0);
    eprintln!("  try:  {} --help", out);
    cleanup(&tmp_dir, keep);
    Ok(())
}

/// Port of `buildMainGo` — the source of the standalone wrapper binary. Embeds
/// the library source as a string constant and dispatches every invocation to
/// `orchestrator.RunCommand`.
fn build_main_go(lib_name: &str, lib_src: &str) -> String {
    format!(
        concat!(
            "// Generated by capy build. Do not edit.\n",
            "package main\n",
            "\n",
            "import (\n",
            "\t\"fmt\"\n",
            "\t\"os\"\n",
            "\n",
            "\t\"github.com/olivierdevelops/capy/orchestrator\"\n",
            ")\n",
            "\n",
            "const libName = {name}\n",
            "\n",
            "const libSource = {src}\n",
            "\n",
            "func main() {{\n",
            "\tif len(os.Args) < 2 || os.Args[1] == \"--help\" || os.Args[1] == \"-h\" {{\n",
            "\t\tprintUsage()\n",
            "\t\treturn\n",
            "\t}}\n",
            "\t// The library is EMBEDDED in this binary; the user already\n",
            "\t// trusted it by running the binary. Suppress the\n",
            "\t// \"not on CAPY_LIBS\" warning that would otherwise fire\n",
            "\t// for the temp-file path.\n",
            "\tos.Setenv(\"CAPY_TRUST\", \"1\")\n",
            "\tf, err := os.CreateTemp(\"\", \"capy-lib-*.capy\")\n",
            "\tif err != nil {{ fmt.Fprintln(os.Stderr, err); os.Exit(1) }}\n",
            "\tdefer os.Remove(f.Name())\n",
            "\tif _, err := f.WriteString(libSource); err != nil {{ fmt.Fprintln(os.Stderr, err); os.Exit(1) }}\n",
            "\tf.Close()\n",
            "\n",
            "\tcmd := os.Args[1]\n",
            "\targs := os.Args[2:]\n",
            "\tif err := orchestrator.RunCommand(f.Name(), cmd, args); err != nil {{ fmt.Fprintln(os.Stderr, err); os.Exit(1) }}\n",
            "}}\n",
            "\n",
            "func printUsage() {{\n",
            "\tfmt.Printf(\"%s \\u2014 bundled Capy library (built with capy build)\\n\\n\", libName)\n",
            "\tfmt.Println(\"USAGE\")\n",
            "\tfmt.Printf(\"    %s <command> [args...]\\n\\n\", libName)\n",
            "\tfmt.Println(\"Try --help for command-specific help.\")\n",
            "}}\n",
        ),
        name = gofmt::quote(lib_name),
        src = go_raw_string_literal(lib_src)
    )
}

/// Port of `goRawStringLiteral` — encodes `s` as a Go raw-string literal, falling
/// back to an interpreted string with escaping when `s` contains a backtick.
fn go_raw_string_literal(s: &str) -> String {
    if !s.contains('`') {
        return format!("`{}`", s);
    }
    let mut b = String::from("\"");
    for c in s.bytes() {
        match c {
            b'\\' => b.push_str("\\\\"),
            b'"' => b.push_str("\\\""),
            b'\n' => b.push_str("\\n"),
            b'\t' => b.push_str("\\t"),
            b'\r' => b.push_str("\\r"),
            other => b.push(other as char),
        }
    }
    b.push('"');
    b
}

/// Port of `findCapyModuleRoot` — looks up from the cwd (and from this binary's
/// location) for a go.mod declaring `module github.com/olivierdevelops/capy`.
fn find_capy_module_root() -> String {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.to_string_lossy().into_owned());
    }
    if let Ok(self_exe) = std::env::current_exe() {
        candidates.push(gopath::dir(&self_exe.to_string_lossy()));
    }
    for start in candidates {
        let mut dir = start;
        for _ in 0..12 {
            let gomod = gopath::join(&[&dir, "go.mod"]);
            if let Ok(data) = std::fs::read_to_string(&gomod) {
                if data.contains("module github.com/olivierdevelops/capy") {
                    return dir;
                }
            }
            let parent = gopath::dir(&dir);
            if parent == dir {
                break;
            }
            dir = parent;
        }
    }
    String::new()
}

/// Stand-in for `os.MkdirTemp`.
fn make_temp_dir(prefix: &str) -> Result<String, String> {
    let base = std::env::temp_dir();
    for _ in 0..1000 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = base.join(format!("{}{}{}", prefix, std::process::id(), nanos));
        match std::fs::create_dir(&p) {
            Ok(_) => return Ok(p.to_string_lossy().into_owned()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(gopath::io_error("mkdir", &p.to_string_lossy(), &e)),
        }
    }
    Err("mkdirtemp: exhausted attempts".to_string())
}
