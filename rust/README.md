# capy-core

The Capy transpiler engine: a **target-agnostic transpiler** that ships *zero*
source-language grammar. A `.capy` library defines the source language; one
source can render to many text targets.

This crate is a port of the Go reference implementation. It is verified against
that implementation by a differential test suite — the same `samples/` golden
corpus plus per-layer output comparisons.

## Use as a library

```toml
[dependencies]
capy-core = "0.12"
```

```rust
use capy_core::capy::Library;

let lib = Library::new(r#"
extension html

function button
    arg literal "button"
    arg capture label string
    write `<button>${decoded label}</button>
`
end
"#)?;

let out = lib.run(r#"button "Click me""#)?;
assert_eq!(out, "<button>Click me</button>\n");
```

Libraries are sandboxed by default (`NoOpHost`): the `env` / `read_file` /
`exec` inner-DSL primitives return empty values or error. Opt in explicitly:

```rust
use capy_core::infra::os_host::OsHost;
use std::rc::Rc;

let mut lib = Library::new(src)?;
lib.set_host(Some(Rc::new(OsHost {
    user_args: vec![],
    base_dir: "/path/to/scripts".into(),
})));
```

Other entry points:

| API | Purpose |
|---|---|
| `Library::run_multi` | multi-file output for libraries declaring `file "path"` blocks |
| `Library::introspect` | function/arg metadata for editor tooling |
| `capy::render_library_docs` | Markdown reference docs |
| `orchestrator::run::run` | file-based transpile (library + script paths) |
| `orchestrator::commands::run_command` | dispatch a library-declared command |
| `domain::errors::format_with_source` | render an error with a source caret |

## Use as a CLI

The `capy` binary lives in the `capy-cli` crate:

```sh
cargo install capy-cli
capy run lib.capy script.capy
```

## Features

| Feature | Default | Effect |
|---|---|---|
| `wasm-abi` | off | exposes the `#[no_mangle] extern "C"` linear-memory ABI (`capy_alloc`, `capy_run`, …) for wasm hosts such as wazero. Off by default so a normal Rust consumer inherits no C symbols. |

## Building for wasm

```sh
cargo build --release --target wasm32-unknown-unknown --features wasm-abi
```

Produces a ~1.4 MB module exporting `capy_alloc` / `capy_free` / `capy_run` /
`capy_introspect` / `capy_docs` / `capy_version`. Every result is a
length-prefixed JSON buffer; see `src/wasm_abi.rs` for the contract.

## License

MIT
