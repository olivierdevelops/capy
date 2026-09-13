# AGENTS.md — working notes for this repo

Capy is a **target-agnostic transpiler**: it ships *zero* source-language
grammar. A `.capy` library defines the source language; one source can
render to many text targets. Keep engine changes additive — no existing
library should break.

## Built-in functions list — KEEP IT IN SYNC

The authoritative list of Capy's built-in **template helper functions**
(the ones callable inside `${ … }` and inner-DSL expressions) lives in:

- **Source of truth:** the `FUNCS` table in `rust/src/infra/helpers.rs`.
- **Cookbook (user-facing):** `docs/function-cookbook.md` — documents and
  shows an example of every helper. Linked in `mkdocs.yml` nav under
  Reference → "Built-in function cookbook", so it publishes to gh-pages.
- **Verified examples:** `samples/builtin-functions/` — a runnable sample
  whose golden output (`script.expected.txt`) is checked in CI.

**RULE (do this every time):** whenever you add, rename, or remove a
built-in helper in `infra/helpers.go`, you MUST in the same change:

1. Update `docs/function-cookbook.md` — the quick-reference table **and** a
   per-function section with a worked example.
2. Update `samples/builtin-functions/` if the helper is demonstrable there,
   and regenerate the golden:
   `cargo run --manifest-path rust/Cargo.toml -p capy-cli --bin capy -- run samples/builtin-functions/lib.capy samples/builtin-functions/script.capy > samples/builtin-functions/script.expected.txt`
3. Mention the change in `docs/whats-new.md`.

## Library-authoring keyword list — KEEP IT IN SYNC

The keywords authors write *inside* a `.capy` library (`function`,
`arg literal`, `arg capture`, the `block_*` openers, `write`, `type`, the
file-level directives, capture types) are documented in
**`docs/library-keywords.md`** (nav: Reference → "Library keyword
cookbook"). Their source of truth is the directive switches in
`rust/src/infra/capy_lib_parser.rs` (top-level, function-body, type-body) plus the
capture-type list in `rust/src/orchestrator/features/make_library_loader.rs`
(`case "any", "ident", …`) and `rust/src/orchestrator/features/make_parser.rs`.

**RULE:** whenever you add/rename/remove a library directive or capture
type, update `docs/library-keywords.md` (and `docs/features.md`,
`docs/CAPY_FOR_LLMS.md`) in the same change. The canonical `if … end`
example is verified by `samples/library-keywords/`.

## Repo conventions

- **gh-pages** deploys from `docs/` on push to `main`; verify locally with
  `mkdocs build --strict` (must exit 0).
- **Playground** samples come from the curated list in
  `rust/playground/src/curated.rs`. It writes JSON to STDOUT — regenerate
  with `cargo run --manifest-path rust/Cargo.toml -p capy-playground-bundle > docs/assets/playground/samples.json`
  (that file is gitignored; CI rebuilds it).
- **Golden tests** (`rust/tests/golden.rs`) discover `samples/*/` and
  pair each `*.capy` with `<base>.expected.txt` (success) or
  `<base>.expected-error.txt` (error, format `LINE:COL: message`).
  `cargo test --manifest-path rust/Cargo.toml --test golden`. Refresh goldens
  with `CAPY_UPDATE_GOLDENS=1` (an env var rather than a flag, since `cargo test`
  can't pass custom flags through). It only refreshes EXISTING golden files —
  pre-create an empty placeholder for a brand-new sample first.
- **Before committing:** `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, and `mkdocs build --strict` all green.
- **Git:** commit only when asked; stage files by name (never `git add -A`);
  never commit `.ignore/` files; never force-push `main`; never `--no-verify`.
