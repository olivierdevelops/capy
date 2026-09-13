# CLI Reference

`capy` is the command-line front-end. All subcommands also accept
`-h`/`--help` for inline help.

## `capy run <library.capy> <script.capy>`

Transpile a script against a library. Output goes to stdout unless the
library sets `output_file:` or you pass `--out`.

```sh
capy run samples/transpile-py/lib.capy samples/transpile-py/script.capy
capy run lib.capy app.capy --out=app.py
```

Flags:

| Flag           | Effect                                                 |
|----------------|--------------------------------------------------------|
| `--out <path>` | Write output to this file (overrides library setting). |
| `--no-color`   | Disable ANSI escape codes (reserved).                  |
| `--debug`      | Verbose engine tracing (reserved).                     |
| `-lib <path>`  | (legacy) library path; equivalent to positional arg 1. |

Errors are printed with line + column + caret:

```
error: no library function matches token "x"
  1 │ x = 1
    │ ^
```

## `capy check <library.capy>`

Parse and validate a library without running any source. Reports loaded
functions and types if valid, or a structured error otherwise. Exit code
0 = valid, 1 = invalid.

```sh
capy check lib.capy
# ok — 5 function(s), 2 type(s)
#   function greet
#   function assign
#   ...
#   type     Email
#   type     Status
```

Use this in CI alongside `cargo test --workspace` to catch broken libraries early.

## `capy init [<dir>]`

Scaffold a starter project (`lib.capy`, `script.capy`, `README.md`) in the
target directory (default `.`). Refuses to overwrite existing files.

```sh
capy init my-dsl
# created my-dsl/lib.capy
# created my-dsl/script.capy
# created my-dsl/README.md
```

## `capy build <library> [-o <output>]`

Produce a **standalone binary** that has the library embedded. The
output dispatches to the library's commands at runtime — no `capy`
install needed on the target host.

```sh
capy build greet -o greet
./greet run hello.greet
```

Takes `--target <triple>` for cross-compilation (the target needs
`rustup target add` plus a linker for it first). Full walkthrough + size
benchmarks + tips:
[Compile a library to a standalone binary](compiling-libraries.md).

## `capy version`

Print the version baked in at build time. `dev` if you built from source
without `-ldflags "-X main.version=..."`.

## `capy help [<command>]`

Top-level help when called without arguments; per-command detailed help
when called with one.

## Exit codes

| Code | Meaning                                                    |
|------|------------------------------------------------------------|
| 0    | Success.                                                   |
| 1    | An error occurred (syntax, validation, I/O). See stderr.   |
| 2    | Reserved for usage errors (not currently distinguished).   |

## Embedding Capy

If you want to call Capy from Rust code, the `capy_core::capy` module
exposes a small embedding API:

```rust
use capy_core::capy::Library;

let lib = Library::from_file("lib.capy")?;
let out = lib.run(script_source)?;
```

See [embedding.md](embedding.md) for the full surface (in-memory
`NewLibrary`, host capabilities, multi-file output).

Errors are `*domain.CapyError` values; use `domain.FormatWithSource(err, src)`
to render them with a caret.
