# Architecture

This doc describes how the engine is laid out internally. For library
authors, this isn't required reading — it's here for anyone curious
about the internals.

## Top-level layout

```
domain/         entities (Token, Library, FuncDef, FuncCall, CapyError, …)
features/       capability struct declarations (Lexer, Parser, Evaluator, …)
usecases/       contracts the orchestrator wires up (RunScript)
io/cli/         CLI view + view model + the use-case protocol it needs
infra/          adapters to external systems (file IO, YAML, text/template)
orchestrator/   the only place where things get assembled
```

The six folders are not negotiable. If you feel the urge to add `utils/` or
`shared/`, re-read the responsibilities below.

## Data flow

```
script.capy ──► Lexer ──► tokens ──► Outer Parser ──► AST (FuncCall) ──┐
                                          ▲                              │
                                          │                              ▼
                                          │                       Outer Evaluator
                                          │                              │
                                  Library (FuncDef) ◄── Library Loader   │
                                          │                              │
                                          ▼                              ▼
                                    type defs                Inner Evaluator
                                                          (mutates context via run:)
                                                                         │
                                                                         ▼
                                                               File template render
                                                                         │
                                                                         ▼
                                                                       output
```

## Module responsibilities

### `domain/`

Pure data types with no behavior beyond simple constructors. Token, AST,
Library shape, errors. Imports nothing internal.

### `features/`

Each external capability is declared as a function-pointer type alias:

```rust
pub type TokenizeFn = fn(&str) -> Result<Vec<Token>, CapyError>;
pub type ParseFn = fn(Vec<Token>, &str, &Library) -> Result<Block, CapyError>;
pub type EvaluateFn = fn(&Block, &Library) -> Result<String, CapyError>;
// ...
```

This is a deliberate VHCO move: features declare **shapes**, not
implementations. The orchestrator supplies the functions.

### `usecases/`

Higher-level user-visible operations. Currently just `RunScript`. Each
declares the function-type aliases for the capabilities it needs from
features. No implementations; only contracts.

### `io/cli/`

A dumb view (`view.rs`, renders state enums) + a view-model (`view_model.rs`,
handles flow control) + the use-case protocol the view-model needs. No business
logic in the view.

### `infra/`

External-system adapters: `file_reader.rs`, `capy_lib_parser.rs` (the `.capy`
library reader), `helpers.rs` (the built-in template helpers), `os_host.rs`,
`preprocessor.rs`. No knowledge of the domain types — the orchestrator maps
between them.

### `orchestrator/`

The **only** module that imports concrete types from other modules. Every
`make_*` factory lives here:

All paths below are under `rust/src/`.

```
orchestrator/features/
  make_lexer.rs
  make_parser.rs
  make_evaluator.rs
  make_library_loader.rs
  inner_parser.rs
  inner_evaluator.rs
  value_parser.rs
  expr_to_text.rs
  translate_new_shape.rs
orchestrator/usecases/make_run_script.rs
orchestrator/views.rs
orchestrator/app.rs
orchestrator/run.rs             # programmatic entry point
orchestrator/commands.rs        # library-declared `command` dispatch
```

## The two grammars

Capy has two grammars in the engine:

1. **Outer (zero default)** — user-facing source. Matched against
   library-defined function shapes. No hard-coded keywords.

2. **Inner (small fixed grammar)** — the language inside each library's
   `run:` field. Has a fixed parser/evaluator pair (`inner_parser.rs` /
   `inner_evaluator.rs`).

Both grammars share the same lexer (it's purely lexical and library-
agnostic) and the same value-expression parser (`value_parser.rs`).

## Captures: dual face

Each capture is parsed once but exposed two ways:

- To templates → as the source text (so `if x > 0` emits literal `if x > 0:`
  in Python).
- To the inner DSL → as the evaluated value (so `append context.x value`
  stores the evaluated value).

This is implemented in `make_evaluator.rs` (`render_template` uses `.text`)
and `inner_evaluator.rs` (`resolve_path` evaluates `.expr`).

## Error positions

`domain::errors::CapyError { line, col, msg, hint, file }` is the structured
error type. The outer parser populates it from token positions. The CLI calls
`domain::errors::format_with_source` to render the caret block.

## Testing

- Golden tests (`rust/tests/golden.rs`) walk `samples/*/` and compare
  each script's actual output to a stored `*.expected.txt` /
  `*.expected-error.txt`. Regenerate with
  `CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden`.
- Unit tests live in a `#[cfg(test)] mod tests` at the bottom of the file they
  cover; cross-module tests live in `rust/tests/`.
- `rust/devtools/wasm_check.sh` runs the same golden corpus through the wasm
  module and the browser shim, so the linear-memory ABI is covered too.

## Adding a feature

A typical change touches:

1. `domain/` — the data shape (a new field, a new struct).
2. `infra/capy_lib_parser.rs` — parsing the new directive out of a `.capy`
   library (if user-visible).
3. `orchestrator/features/make_library_loader.rs` — the mapping into `domain`.
4. The relevant feature implementation (`make_parser.rs`, `make_evaluator.rs`,
   or `inner_evaluator.rs`).
5. A new sample under `samples/` + golden.
6. Docs under `docs/` describing the new field.

`features/` and `usecases/` only change when you're adding a whole new
capability (rare).
