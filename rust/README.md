# capy-core

The Capy transpiler engine: a **target-agnostic transpiler** that ships *zero*
source-language grammar. A `.capy` library defines the source language; one
source can render to many text targets.

This crate is a port of the Go reference implementation. It is verified against
that implementation by a differential test suite — the same `samples/` golden
corpus plus per-layer output comparisons.

## Use as a library

Not on crates.io yet, so depend on it from git — cargo locates the crate inside
this repo's `rust/` subdirectory on its own:

```toml
[dependencies]
capy-core = { git = "https://github.com/olivierdevelops/capy" }
```

That tracks the default branch. Pin a `rev` for a reproducible build:

```toml
capy-core = { git = "https://github.com/olivierdevelops/capy", rev = "<commit>" }
```

Don't pin `tag = "v0.12.0"` — the release tags up to and including v0.12.0
predate the Rust port, so `rust/` doesn't exist at those commits and cargo
won't find this crate. The first tag usable here is whichever release follows
the port's merge.

Or, once published:

```toml
[dependencies]
capy-core = "0.12"
```

Minimum supported Rust version: **1.74** (verified against 1.74.1). Note that
*contributing* to this workspace needs 1.78+, because its `Cargo.lock` is
version 4 — that affects building the repo, never consuming the crate.

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
use std::sync::Arc;

let mut lib = Library::new(src)?;
lib.set_host(Some(Arc::new(OsHost {
    user_args: vec![],
    base_dir: "/path/to/scripts".into(),
})));
```

### Threads

`Library` is `Send + Sync`. Compile once, share it, transpile concurrently —
`run` takes `&self` and builds a fresh accumulating context per call, so no lock
is needed:

```rust
use std::sync::{Arc, OnceLock};

static LIB: OnceLock<Arc<Library>> = OnceLock::new();

let lib = Arc::clone(LIB.get_or_init(|| Arc::new(Library::new(src).unwrap())));
std::thread::spawn(move || lib.run(script));
```

The engine holds `Arc` rather than `Rc` throughout to keep this true, and a
compile-time assertion in `tests/embed.rs` stops it regressing. A custom `Host`
must therefore be `Send + Sync`: use `Mutex`/`RwLock` for interior mutability,
not `RefCell`.

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

## Crates in this workspace

| Crate | Published | What it is |
|---|---|---|
| `capy-core` | yes | the engine — this is what you depend on from Rust |
| `capy-cli` | yes | the `capy` binary |
| `capy-mcp` | yes | MCP server exposing Capy to AI agents |
| `capy-wasm-abi` | no | `cdylib` producing the `.wasm` module for wazero / browser hosts |
| `capy-playground-bundle` | no | repo tooling: regenerates the playground's `samples.json` |
| `capy-devtools` | no | differential-test harnesses used by `devtools/conformance.sh` |

`capy-core` is a plain `rlib` with one dependency (`regex`). It declares no
features and exports no `extern "C"` symbols — the linear-memory wasm ABI lives
in the separate `capy-wasm-abi` crate precisely so a Rust consumer inherits
nothing from it.

## Building for wasm

```sh
cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi
```

Produces a 1.28 MB module exporting `capy_alloc` / `capy_free` / `capy_run` /
`capy_introspect` / `capy_docs` / `capy_version`. Every result is a
length-prefixed JSON buffer; see `wasm/src/lib.rs` for the contract.

## License

MIT
