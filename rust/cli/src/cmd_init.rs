//! Port of `cmd/capy/cmd_init.go`.

use capy_core::gopath;

/// Port of `cmdInit`.
///
/// Scaffolds a starter library + script in the target dir (default `.`). Refuses
/// to overwrite existing files.
pub fn cmd_init(args: &[String]) -> Result<(), String> {
    let dir = if args.is_empty() { "." } else { args[0].as_str() };
    std::fs::create_dir_all(dir).map_err(|e| gopath::io_error("mkdir", dir, &e))?;
    // Go iterates a map here, so its creation order varies; the ordering is not
    // observable beyond the "created …" lines, which this emits alphabetically.
    let files: [(&str, &str); 3] = [
        ("README.md", STARTER_README),
        ("lib.yaml", STARTER_LIB),
        ("script.capy", STARTER_SCRIPT),
    ];
    for (name, content) in files {
        let p = gopath::join(&[dir, name]);
        if std::fs::metadata(&p).is_ok() {
            return Err(format!("refusing to overwrite existing file: {}", p));
        }
        std::fs::write(&p, content.as_bytes())
            .map_err(|e| gopath::io_error("open", &p, &e))?;
        println!("created {}", p);
    }
    println!();
    println!("Next:");
    println!(
        "  capy run {} {}",
        gopath::join(&[dir, "lib.yaml"]),
        gopath::join(&[dir, "script.capy"])
    );
    Ok(())
}

const STARTER_LIB: &str = r#"# A starter Capy library. Edit to define your own source language.
extension: txt

context:
  lines: []

functions:
  say:
    args:
      - { kind: capture, name: msg, type: any }
    template: "say {{ .msg }}\n"
    run: |
      append context.lines msg

file_template: |
  {{- .body -}}
  --- captured {{ len .context.lines }} line(s) ---
"#;

const STARTER_SCRIPT: &str = r#"say "hello, world"
say "this is a starter script"
"#;

const STARTER_README: &str = r#"# My Capy project

Run:

    capy run lib.yaml script.capy

Edit `lib.yaml` to define your own functions, types, and file template.
See https://github.com/olivierdevelops/capy for documentation.
"#;
