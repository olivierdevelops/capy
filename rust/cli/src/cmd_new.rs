//! Port of `cmd/capy/cmd_new.go`.

use super::flags::{self, reorder_flags_first, Spec};
use super::lib_path::resolve_lib;
use capy_core::gopath;
use capy_core::orchestrator::commands;

const NEW_SPEC: Spec = Spec { bools: &[], strings: &["using"] };

/// Port of `cmdNew`.
///
/// Scaffolds a new project using a library. If the library declares a `new`
/// command, run it with the project directory as the first arg. Otherwise create
/// the project dir, drop a `hello.<lib>` example script in, plus a small README.
pub fn cmd_new(args: &[String]) -> Result<(), String> {
    // Go's flag package stops at the first positional, so flags move to the
    // front to make `capy new ./my-app --using recipe` work.
    let args = reorder_flags_first(args);
    let fs = flags::parse(&NEW_SPEC, &args)?;
    let pos = &fs.positionals;
    if pos.is_empty() {
        return Err("usage: capy new <project-dir> --using <library>".to_string());
    }
    let project_dir = &pos[0];
    let using = fs.str("using");
    if using.is_empty() {
        return Err("`--using <library>` is required".to_string());
    }

    let lib_path = resolve_lib(using)?;

    // If the library has a `new` command, use it.
    if has_command(&lib_path, "new") {
        let mut call_args = vec![project_dir.clone()];
        call_args.extend_from_slice(&pos[1..]);
        return commands::run_command(&lib_path, "new", &call_args).map_err(|e| e.to_string());
    }

    // Fallback: minimal default scaffolding.
    std::fs::create_dir_all(project_dir)
        .map_err(|e| gopath::io_error("mkdir", project_dir, &e))?;
    let example = format!("# Sample {} script.\n", using);
    let p = gopath::join(&[project_dir, &format!("hello.{}", using)]);
    std::fs::write(&p, example.as_bytes())
        .map_err(|e| gopath::io_error("open", &p, &e))?;
    let readme = format!(
        r#"# {base}

A Capy project using the `{using}` library.

## Run

```sh
capy {using} run hello.{using}
```
"#,
        base = gopath::base(project_dir),
        using = using
    );
    let p = gopath::join(&[project_dir, "README.md"]);
    std::fs::write(&p, readme.as_bytes())
        .map_err(|e| gopath::io_error("open", &p, &e))?;
    println!("✓ created project {:?} using library {:?}", project_dir, using);
    println!("  cd {} && capy {} run hello.{}", project_dir, using, using);
    Ok(())
}

/// Port of `hasCommand` — a text-level scan, to avoid a full library load just to
/// check (the load can fail for reasons we don't care about here).
pub fn has_command(lib_path: &str, name: &str) -> bool {
    match std::fs::read_to_string(lib_path) {
        Ok(s) => s.contains(&format!("command \"{}\"", name)),
        Err(_) => false,
    }
}
