//! Port of `cmd/capy/cmd_lib.go`.

use super::flags::reorder_flags_first;
use super::impl_resolve::resolve_lib_with_impl;
use super::lib_path::{lib_search_path, list_installed_libs, resolve_lib, try_resolve};
use capy_core::gopath;
use std::process::Command;

/// Port of `cmdLib`.
pub fn cmd_lib(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return cmd_lib_list();
    }
    match args[0].as_str() {
        "list" => cmd_lib_list(),
        "which" => cmd_lib_which(&args[1..]),
        "new" => cmd_lib_new(&args[1..]),
        "path" => cmd_lib_path(),
        "add" => cmd_lib_add(&args[1..]),
        "remove" | "rm" => cmd_lib_remove(&args[1..]),
        "impl" => cmd_lib_impl(&args[1..]),
        "resolve" => cmd_lib_resolve(&args[1..]),
        other => Err(format!(
            "unknown lib subcommand {:?} (try: list / which / new / add / remove / impl / resolve / path)",
            other
        )),
    }
}

/// Port of `cmdLibImpl` — prints the impls a library declares, marking the
/// default with `*`.
fn cmd_lib_impl(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: capy lib impl <library>".to_string());
    }
    // Don't error on "multiple impls + no default" here; we just want to list.
    let resolved = match resolve_lib_with_impl(&args[0], "") {
        Err(e) if !e.contains("impl") => return Err(e),
        Err(_) => {
            // Re-resolve without impl selection so the listing still works.
            let manifest_path = resolve_lib(&args[0])?;
            let lib = capy_core::orchestrator::features::make_library_loader::load_library(
                &manifest_path,
                capy_core::orchestrator::features::make_lexer::tokenize,
            )
            .map_err(|e| e.to_string())?;
            super::impl_resolve::Resolved {
                impl_path: manifest_path.clone(),
                manifest_path,
                lib,
            }
        }
        Ok(r) => r,
    };
    let lib = &resolved.lib;
    let mut header = lib.lib_name.clone();
    if !lib.lib_version.is_empty() {
        header = format!("{} {}", header, lib.lib_version);
    }
    if !header.is_empty() {
        println!("{}", header);
    }
    println!("manifest: {}", resolved.manifest_path);
    if lib.impls.is_empty() {
        println!("    (no impl declarations — the manifest file is the library)");
        return Ok(());
    }
    println!("impls:");
    for (n, im) in &lib.impls {
        let mark = if *n == lib.default_impl { "* " } else { "  " };
        let mut extra = im.description.clone();
        if !im.version.is_empty() {
            extra = format!("{} (v{})", extra, im.version);
        }
        println!("    {}{:<12}  {}", mark, n, extra.trim());
    }
    if !lib.default_impl.is_empty() {
        println!("\nDefault: {}", lib.default_impl);
    }
    Ok(())
}

/// Port of `cmdLibResolve` — shows which impl + path the current environment
/// would select.
fn cmd_lib_resolve(args: &[String]) -> Result<(), String> {
    let args = reorder_flags_first(args);
    let mut impl_flag = String::new();
    let mut pos: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        if a == "--impl" && i + 1 < args.len() {
            impl_flag = args[i + 1].clone();
            i += 2;
            continue;
        }
        if let Some(v) = a.strip_prefix("--impl=") {
            impl_flag = v.to_string();
            i += 1;
            continue;
        }
        pos.push(a.clone());
        i += 1;
    }
    if pos.len() != 1 {
        return Err("usage: capy lib resolve <library> [--impl <name>]".to_string());
    }
    let r = resolve_lib_with_impl(&pos[0], &impl_flag)?;
    print!("library:       {}", r.lib.lib_name);
    if !r.lib.lib_version.is_empty() {
        print!(" v{}", r.lib.lib_version);
    }
    println!();
    println!("manifest:      {}", r.manifest_path);
    if !r.lib.selected_impl.is_empty() {
        println!("impl:          {}", r.lib.selected_impl);
        println!("impl file:     {}", r.impl_path);
    } else {
        println!("impl:          (no impls declared)");
    }
    Ok(())
}

/// Port of `cmdLibList`.
fn cmd_lib_list() -> Result<(), String> {
    let libs = list_installed_libs();
    if libs.is_empty() {
        println!("no libraries found on CAPY_LIBS");
        println!("search path:");
        for p in lib_search_path() {
            println!("   {}", p);
        }
        return Ok(());
    }
    for (n, path) in &libs {
        println!("{:<20}  {}", n, path);
    }
    Ok(())
}

/// Port of `cmdLibWhich`.
fn cmd_lib_which(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: capy lib which <name>".to_string());
    }
    println!("{}", resolve_lib(&args[0])?);
    Ok(())
}

/// Port of `cmdLibPath`.
fn cmd_lib_path() -> Result<(), String> {
    for p in lib_search_path() {
        println!("{}", p);
    }
    Ok(())
}

/// Port of `cmdLibNew` — scaffolds a starter library at the first writable
/// directory on CAPY_LIBS.
fn cmd_lib_new(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: capy lib new <name>".to_string());
    }
    let name = &args[0];
    let target_dir = first_writable_search_dir()
        .ok_or_else(|| "no writable directory on CAPY_LIBS".to_string())?;
    let lib_dir = gopath::join(&[&target_dir, name]);
    if std::fs::metadata(&lib_dir).is_ok() {
        return Err(format!("library {:?} already exists at {}", name, lib_dir));
    }
    std::fs::create_dir_all(&lib_dir)
        .map_err(|e| gopath::io_error("mkdir", &lib_dir, &e))?;

    let manifest = format!(
        r#"# Library manifest. See https://olivierdevelops.github.io/capy/
name        "{name}"
version     "0.1.0"
description "A new Capy library."

extension   "txt"

function greet
    arg literal "greet"
    arg capture who string
    write `Hello from {name}, ${{unquote who}}!
`
end

command "run"
    description "Compile and print to stdout."
    let out = (compile context.arg0)
    print out
end

command "compile"
    description "Compile and write to a .txt file."
    let out    = (compile context.arg0)
    let target = "${{context.arg0}}.txt"
    write_file target out
    print "wrote ${{target}}"
end
"#
    );
    let readme = format!(
        r#"# {name}

A Capy library generated by `capy lib new {name}`.

## Try it

```sh
echo 'greet "world"' > hello.{name}
capy {name} run hello.{name}
```

## Customise

Edit `{name}.capy`. Add functions, commands, types. Re-run
`capy lib list` to confirm it's picked up.
"#
    );
    let example = "greet \"world\"\n";

    let p = gopath::join(&[&lib_dir, &format!("{}.capy", name)]);
    std::fs::write(&p, manifest.as_bytes())
        .map_err(|e| gopath::io_error("open", &p, &e))?;
    let p = gopath::join(&[&lib_dir, "README.md"]);
    std::fs::write(&p, readme.as_bytes())
        .map_err(|e| gopath::io_error("open", &p, &e))?;
    let ex_dir = gopath::join(&[&lib_dir, "examples"]);
    std::fs::create_dir_all(&ex_dir).map_err(|e| gopath::io_error("mkdir", &ex_dir, &e))?;
    let p = gopath::join(&[&ex_dir, &format!("hello.{}", name)]);
    std::fs::write(&p, example.as_bytes())
        .map_err(|e| gopath::io_error("open", &p, &e))?;

    println!("✓ created library {:?} at {}", name, lib_dir);
    println!("  capy {} run {}/examples/hello.{}", name, lib_dir, name);
    Ok(())
}

/// Port of `cmdLibAdd` — clones a library from a git URL (or copies a local
/// path) into the first writable directory on CAPY_LIBS.
fn cmd_lib_add(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: capy lib add <git-url-or-path> [--as <name>]".to_string());
    }
    let source = &args[0];
    let mut as_name = String::new();
    let mut i = 1usize;
    while i < args.len() {
        if args[i] == "--as" && i + 1 < args.len() {
            as_name = args[i + 1].clone();
            i += 2;
            continue;
        }
        i += 1;
    }

    let target_parent = first_writable_search_dir()
        .ok_or_else(|| "no writable directory on CAPY_LIBS".to_string())?;

    let lib_name = if as_name.is_empty() { infer_lib_name(source) } else { as_name };
    if lib_name.is_empty() {
        return Err(format!(
            "could not infer library name from {:?}; use --as <name>",
            source
        ));
    }
    let target = gopath::join(&[&target_parent, &lib_name]);
    if std::fs::metadata(&target).is_ok() {
        return Err(format!(
            "library {:?} already exists at {} (remove first or pass --as <other-name>)",
            lib_name, target
        ));
    }

    // Local path? Just copy.
    if std::fs::metadata(source).is_ok() {
        copy_tree(source, &target).map_err(|e| format!("copy {}: {}", source, e))?;
        println!("✓ added local library {:?} at {}", lib_name, target);
        return Ok(());
    }

    // Otherwise: git clone. Normalise common shorthand.
    let mut url = source.clone();
    if !url.contains("://") && !url.starts_with("git@") {
        url = format!("https://{}", url);
    }
    println!("Cloning {} → {}", url, target);
    let status = Command::new("git")
        .args(["clone", "--depth=1", "--quiet", &url, &target])
        .status()
        .map_err(|e| format!("git clone failed: {}", e))?;
    if !status.success() {
        return Err(format!(
            "git clone failed: exit status {}",
            status.code().unwrap_or(-1)
        ));
    }
    // Sanity check: ensure there's a parseable entry point inside.
    if try_resolve(&target_parent, &lib_name).is_none()
        && std::fs::metadata(gopath::join(&[&target, "lib.capy"])).is_err()
    {
        eprintln!(
            "warning: cloned {:?} but found no <name>.capy / lib.capy entry point",
            lib_name
        );
    }
    println!("✓ added library {:?}", lib_name);
    Ok(())
}

/// Port of `cmdLibRemove`.
fn cmd_lib_remove(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: capy lib remove <name>".to_string());
    }
    let name = &args[0];
    for dir in lib_search_path() {
        let candidate = gopath::join(&[&dir, name]);
        if let Ok(st) = std::fs::metadata(&candidate) {
            if st.is_dir() {
                std::fs::remove_dir_all(&candidate)
                    .map_err(|e| format!("remove {}: {}", candidate, e))?;
                println!("✓ removed {}", candidate);
                return Ok(());
            }
        }
        // Bare-file form.
        let file = gopath::join(&[&dir, &format!("{}.capy", name)]);
        if std::fs::metadata(&file).is_ok() {
            std::fs::remove_file(&file).map_err(|e| format!("remove {}: {}", file, e))?;
            println!("✓ removed {}", file);
            return Ok(());
        }
    }
    Err(format!("library {:?} not found on CAPY_LIBS", name))
}

/// The "first entry of CAPY_LIBS that exists or can be created" logic shared by
/// `cmd_lib_new` and `cmd_lib_add`.
fn first_writable_search_dir() -> Option<String> {
    // `create_dir_all` is the probe AND the creation step, matching Go: the first
    // entry that exists or can be created wins.
    lib_search_path().into_iter().find(|p| std::fs::create_dir_all(p).is_ok())
}

/// Port of `inferLibName`.
///
/// ```text
/// github.com/user/repo       → repo
/// github.com/user/repo-capy  → repo   (strip -capy suffix)
/// ./libs/my-recipe           → my-recipe
/// https://gh.com/u/r.git     → r
/// ```
fn infer_lib_name(source: &str) -> String {
    let mut base = source.to_string();
    if let Some(i) = base.find("://") {
        base = base[i + 3..].to_string();
    }
    base = base.trim_end_matches(".git").to_string();
    base = gopath::base(base.trim_end_matches(['/', '\\']));
    base = base.trim_end_matches("-capy").to_string();
    base = base.trim_end_matches("_capy").to_string();
    base
}

/// Port of `copyTree` — recursively copies src → dst. Files only; preserves the
/// relative tree shape but not perms / ownership.
fn copy_tree(src: &str, dst: &str) -> Result<(), String> {
    let st = std::fs::metadata(src).map_err(|e| gopath::io_error("stat", src, &e))?;
    if !st.is_dir() {
        let parent = gopath::dir(dst);
        std::fs::create_dir_all(&parent)
            .map_err(|e| gopath::io_error("mkdir", &parent, &e))?;
        let data = std::fs::read(src).map_err(|e| gopath::io_error("open", src, &e))?;
        return std::fs::write(dst, data).map_err(|e| gopath::io_error("open", dst, &e));
    }
    std::fs::create_dir_all(dst).map_err(|e| gopath::io_error("mkdir", dst, &e))?;
    let entries = std::fs::read_dir(src).map_err(|e| gopath::io_error("open", src, &e))?;
    for e in entries.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().into_owned();
        copy_tree(&gopath::join(&[src, &name]), &gopath::join(&[dst, &name]))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::infer_lib_name;

    #[test]
    fn infers_names_like_go() {
        assert_eq!(infer_lib_name("github.com/user/repo"), "repo");
        assert_eq!(infer_lib_name("github.com/user/repo-capy"), "repo");
        assert_eq!(infer_lib_name("./libs/my-recipe"), "my-recipe");
        assert_eq!(infer_lib_name("https://gh.com/u/r.git"), "r");
        // `filepath.Base` splits on the `/` in `u/thing`, so the scp-style host
        // prefix is discarded — verified against Go.
        assert_eq!(infer_lib_name("git@github.com:u/thing.git"), "thing");
    }
}
