---
title: Library commands + library path
hide:
  - toc
---

<div class="capy-hero" markdown>

<span class="capy-eyebrow">v0.19 — LIBRARIES AS CLIs</span>

# Install libraries. Run them by name. Ship commands.

Libraries live on a search path. Reference them by name from any
directory. Declare your own commands (`run`, `build`, `serve`, …)
that can shell out, write files, and produce real artifacts —
not just stdout.

</div>

## The big picture

Three new things in v0.19, each useful on its own and great
together:

1. **`CAPY_LIBS` search path.** Install a library once; reference
   it by name from anywhere.
2. **Library commands.** A library can declare custom verbs in
   its `capy.capy` manifest. Each command body runs against a
   small shell-flavoured inner DSL — `let`, `exec`, `write_file`,
   `mktemp`, `print`, `cd`, plus the `compile` primitive that
   runs the library on a script.
3. **`capy new`.** Scaffold a project from a library. If the
   library declares a `new` command, it gets called with the
   project directory; otherwise a tiny default scaffold lands.

## A 60-second tour

```sh
# 1. Set the search path (default: ~/.config/capy/libs/ + ~/.capy/libs/).
export CAPY_LIBS=~/.capy/libs

# 2. Scaffold a new library.
capy lib new recipe
# ✓ created library "recipe" at ~/.capy/libs/recipe
#   capy recipe run ~/.capy/libs/recipe/examples/hello.recipe

# 3. Use it.
capy recipe run ~/.capy/libs/recipe/examples/hello.recipe
# Hello from recipe, world!

# 4. Build (uses the library's `compile` command).
capy recipe compile ~/.capy/libs/recipe/examples/hello.recipe
# wrote ~/.capy/libs/recipe/examples/hello.recipe.txt

# 5. Auto-detect by file extension.
echo 'greet "ext"' > /tmp/cake.recipe
capy /tmp/cake.recipe
# Hello from recipe, ext!

# 6. Scaffold a project that uses the library.
capy new my-app --using recipe
# ✓ created project "my-app" using library "recipe"
```

## `CAPY_LIBS` search path

`capy` resolves library names against a colon-separated
(semicolon on Windows) path list. Defaults (when `CAPY_LIBS` is
unset):

| Platform | Default |
|---|---|
| Any | The current working directory (so a project-local `<name>.capy` is always discoverable) |
| Linux | `$XDG_CONFIG_HOME/capy/libs/` (and `~/.capy/libs/` as a fallback) |
| macOS | `~/Library/Application Support/Capy/libs/` |
| Windows | `%APPDATA%\Capy\libs\` |

The CWD entry is what makes `capy main.<ext>` work out of the box for
projects whose library file sits next to the script — no install
step required. Libraries resolved via the CWD entry are also
trusted (no "not on CAPY_LIBS" warning when their commands shell out).

Override entirely with `CAPY_LIBS` — it's a full list, not a
supplement:

```sh
export CAPY_LIBS="$HOME/.capy/libs:/usr/local/share/capy/libs"
```

Resolution looks for:

1. `<dir>/<name>.capy` (bare file)
2. `<dir>/<name>/<name>.capy` (library directory, file matches name)
3. `<dir>/<name>/lib.capy` (library directory, generic name)

First match wins.

### `capy lib` subcommands

```sh
capy lib list                  # list every library on CAPY_LIBS
capy lib which recipe          # show the resolved path
capy lib new my-recipe         # scaffold a new library
capy lib path                  # print the search path entries
```

## Library commands

A library declares commands in its manifest (also a `.capy` file
— no separate config format). Example from the
`capy lib new` scaffold:

```
name        "recipe"
version     "0.1.0"
description "A recipe DSL."

extension   "txt"

function greet
    arg literal "greet"
    arg capture who string
    write `Hello from recipe, ${unquote who}!
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
    let target = "${context.arg0}.txt"
    write_file target out
    print "wrote ${target}"
end
```

### Command body — the shell-flavoured inner DSL

Inside a `command` body you can use everything the regular inner
DSL gives you (`set`, `append`, `if`, `for`, …) plus:

| Form | Effect |
|---|---|
| `let X = (EXPR)` | Bind a local. |
| `(compile script_path)` | Run the library on a script. Returns the output. |
| `print EXPR` | Print to stdout. |
| `write_file PATH CONTENTS` | Create or overwrite the file (parent dirs made). |
| `mkdir PATH` | Create a directory (with parents). |
| `(mktemp ".ext")` | Fresh temp file path. |
| `(mktemp_dir)` | Fresh temp directory path. |
| `exec CMD ARGS...` | Run a subprocess; stream stdout/stderr to the user. |
| `(exec_capture CMD ARGS...)` | Run a subprocess; capture combined output. |
| `cd PATH` | Change working directory. |

### Declarative args + flags (v0.19.1)

A command can declare its own positional args + flags. The CLI
parses them, surfaces them via the context, and generates a
`--help` screen from the declarations.

```
command "build"
    description "Compile a script and write to a file."
    arg "script" required "Path to the input script."
    arg "out"    optional "Output path. Defaults to <script>.txt."
    flag "--minify" bool "Strip whitespace from the output."
    flag "--prefix"      "Optional prefix line." default ""

    let rendered = (compile context.script)
    let target   = "${context.script}.txt"
    if context.out
        let target = context.out
    end
    if context.flags.prefix
        let rendered = "${context.flags.prefix}\\n${rendered}"
    end
    write_file target rendered
    print "wrote ${target} (minify=${context.flags.minify})"
end
```

In the body, declared positional args appear under their declared
name (`context.script`, `context.out`); flags appear under
`context.flags.NAME` with the leading dashes trimmed.

### Generated help

`capy <lib> --help` lists all commands. `capy <lib> <cmd> --help`
shows that command's args + flags:

```
$ capy myapp --help
myapp 1.0.0
A demo library.

COMMANDS
    build         Compile a script and write to a file.

$ capy myapp build --help
build — Compile a script and write to a file.

USAGE
    capy myapp build [--minify] [--prefix VALUE] <script> [out]

ARGUMENTS
    script        Path to the input script. (required)
    out           Output path. Defaults to <script>.txt. (optional)

FLAGS
    --minify      Strip whitespace from the output.
    --prefix      Optional prefix line.
```

### Cross-command composition (`call`)

Inside one command body, run another command of the same library:

```
command "build"
    arg "script" required
    let out    = (compile context.script)
    let target = "${context.script}.txt"
    write_file target out
end

command "release"
    description "Build then announce."
    arg "script" required
    call "build" context.script
    print "🚀 released!"
end
```

`call` accepts a command name and any number of positional args;
it runs the named command end-to-end (including its own arg /
flag parsing) and returns `""` so it can also be used in value
position.

### Built-in context paths

Inside every command body:

| Path | What it holds |
|---|---|
| `context.<declared-arg-name>` | The value of a declared positional. |
| `context.flags.<name>` | The value of a declared flag (string or bool). |
| `context.args` | The full positional args list. |
| `context.extra` | Positional args beyond what the command declared. |
| `context.lib_path` | Path to the resolved library. |
| `context.lib_dir` | Directory containing the library. |
| `context.lib_name` | Name from the manifest (or filename if unset). |
| `context.lib_version` | Version from the manifest. |
| `context.argN` | `arg0`/`arg1`/… legacy numeric aliases (when no args declared). |

### Shebang scripts (v0.19.1)

A `#!/usr/bin/env capy --lib <name>` line at the top of a script
is stripped before lexing. With `chmod +x`, the file is directly
executable:

```sh
$ cat cake.recipe
#!/usr/bin/env capy --lib recipe
greet "world"

$ chmod +x cake.recipe
$ ./cake.recipe
Hello from recipe, world!
```

Works on POSIX systems where `/usr/bin/env` honours subsequent
flags. Windows needs explicit `.capy` file associations.

### Built-in `run`

If your library doesn't declare a `command "run"`, calling
`capy <lib> run <script>` falls back to the legacy "render to
stdout" behaviour — no commands declared = no behaviour change.

## `capy new`

Scaffold a project that USES a library:

```sh
capy new my-app --using recipe
```

If the library declares a `command "new"`, it's called with the
project directory as `context.arg0`. That lets a library author
ship rich scaffolding (a whole Vite app, an Android Studio
project, etc.).

Otherwise, the CLI drops a minimal `hello.<lib>` script + README
into the new directory.

## File-extension convention

A script named `X.<libname>` is auto-resolved when you pass it
alone:

```sh
echo 'greet "world"' > cake.recipe
capy cake.recipe         # resolves library `recipe`, runs `run`
```

The library must be on `CAPY_LIBS` for this to work.

## Walkthrough: a Python library with multi-step build

```
# ~/.capy/libs/python/python.capy
name "python"
version "0.18.0"
extension "py"

function say
    arg literal "say"
    arg capture msg any
    write `print(${msg})
`
end

command "run"
    description "Generate the Python and run python3 on it."
    let out = (compile context.arg0)
    let tmp = (mktemp ".py")
    write_file tmp out
    exec "python3" tmp
end

command "build"
    description "Generate the Python file next to the script."
    let out    = (compile context.arg0)
    let target = "${context.arg0}.py"
    write_file target out
    print "wrote ${target}"
end
```

Use:

```sh
$ echo 'say "hi"' > hello.py.capy
$ capy python run hello.py.capy
hi
$ capy python build hello.py.capy
wrote hello.py.capy.py
```

## Security note

Library commands can do arbitrary work — `exec`, write files,
read environment. The trust model today:

- Libraries under `CAPY_LIBS` are trusted by default.
- Libraries elsewhere print a one-line stderr warning before
  every command runs:
  ```
  warning: library "/tmp/local.capy" is not on CAPY_LIBS —
  its commands can shell out / write files / read env
  ```
- Set `CAPY_TRUST=1` to suppress the warning (use when you've
  reviewed the library yourself).

A richer trust model (`~/.capy/trusted.capy`, per-library SHA
pins, `--dry-run`) is on the roadmap — see
[future-features § 6.5](https://github.com/olivierdevelops/capy/blob/main/docs/design/future-features.md).

Only put libraries on `CAPY_LIBS` that you'd trust to run as
yourself. Library files cloned from random URLs should land
elsewhere first; review them, then move into the search path.

## v0.20: tooling around libraries

A handful of dev-loop tools shipped in v0.20 to make the
"libraries as CLIs" story complete:

### `capy watch`

Polling-based file watcher; re-runs a command whenever any
watched file changes. Watches the entire library directory + any
file-path arguments you passed.

```sh
$ capy watch recipe run cake.recipe
👀 watching 2 file(s); re-runs on save (Ctrl-C to exit)
    cake.recipe
    ~/.capy/libs/recipe/recipe.capy
Hi world!

# Edit cake.recipe in another window…
--- change detected — re-running ---
Hi updated!
```

Falls back to legacy form for libraries with no declared
commands: `capy watch lib.capy script.capy`.

### `capy fmt`

Conservative formatter for `.capy` library files:
- Strips trailing whitespace.
- Converts leading tabs to 4-space indentation.
- Collapses multi-blank-line runs to one blank.
- Ensures exactly one trailing newline.

```sh
capy fmt lib.capy            # rewrite in place
capy fmt --check lib.capy    # exit 1 if not formatted
capy fmt --diff lib.capy     # print diff vs. formatted
capy fmt --stdout lib.capy   # print formatted output
```

Does NOT touch the inside of backtick literals (whitespace there
is significant for the emitted output). Future versions may add
declaration-order normalisation and arg alignment.

### `capy lib add` (git + local)

Install a library by git URL or local path:

```sh
capy lib add github.com/user/repo                  # git clone
capy lib add github.com/user/repo --as new-name    # rename on install
capy lib add /local/path/to/lib                    # copy a local directory
```

Common shorthand: `github.com/X/Y` expands to `https://github.com/X/Y`.
A `-capy` / `_capy` suffix on the repo name is stripped when
inferring the library name. The clone lands in the first writable
directory on `CAPY_LIBS`.

### `capy lib remove`

```sh
capy lib remove recipe
# ✓ removed ~/.capy/libs/recipe
```

### `capy build` — single-binary compiler

Bake a library into a standalone executable. The resulting binary
needs no `capy` install on the target host:

```sh
$ capy build recipe -o recipe-tool
building recipe (this needs the Cargo toolchain)…
✓ wrote ./recipe-tool (5.4 MB)
  try:  ./recipe-tool --help

$ ./recipe-tool run cake.recipe
Hi world!

$ ./recipe-tool build cake.recipe -o cake.txt
wrote cake.txt
```

How it works: `capy build` writes a tiny Rust wrapper `src/main.rs`
that embeds the library source as a string constant + dispatches
via `run_command`, then shells out to `cargo build`. The user needs a Cargo
toolchain installed to run `capy build`; the OUTPUT binary has no such
requirement.

Caveats:
- The build targets the host by default. Cross-compile with `--target`:
  `capy build recipe --target aarch64-unknown-linux-gnu -o recipe-linux-arm64`
  (the target needs `rustup target add` plus a linker for it first).
- The compiled binary embeds the library at build time;
  upgrade-on-the-fly isn't a thing — rebuild.
- Section 2(b) of the [LICENSE](https://github.com/olivierdevelops/capy/blob/main/LICENSE)
  applies — redistribution of compiled binaries is permitted for
  the library author (you embedded YOUR library) but the engine
  itself isn't open-redistribution.

## What's still on the roadmap

Per the [comprehensive design](https://github.com/olivierdevelops/capy/blob/main/docs/design/future-features.md):

- Argument / flag parsing with auto-generated `--help` per command.
- Trust model: `~/.capy/trusted.capy`, `--trust` / `--dry-run`.
- Cross-command composition (`call <cmd>`, `last_command.X`).
- Multiple implementations of one library.
- Library version + lockfile.
- Git-based `capy lib add github.com/...`.
- WASM packaging + single-binary compiler.

What ships in v0.19 is the substrate: search path + manifest +
commands + the project-scaffolding workflow. Everything else
builds on top.
