# Roadmap

Direction for Capy after the current `0.20.x` line — **not commitments**.
Open an issue if any of these matter to you so we know to prioritise.

> Status legend: ✅ shipped · 🚧 in progress · 🔭 planned · 💡 idea

## Already shipped

Much of the original `v0.2`–`v0.5` roadmap has landed. Recorded here so the
"planned" list below stays honest — see [CHANGELOG.md](https://github.com/olivierdevelops/capy/blob/main/CHANGELOG.md)
for the per-release detail.

- ✅ **`capy watch`** — polling file-watcher that re-runs on change (`0.20.0`).
- ✅ **`capy fmt`** — conservative formatter (`--check`, `--diff`, `--stdout`).
  The full canonical normaliser (declaration ordering, arg alignment) is still 🚧.
- ✅ **`capy lib add` / `lib remove`** — install a library from a git URL or
  local path; remove an installed one (`0.20.0`).
- ✅ **`capy build`** — compile a library to a standalone, Capy-free binary;
  cross-compile via `--target` (`0.20.0`). See
  [compiling-libraries.md](compiling-libraries.md).
- ✅ **Argument `default` values** on captures (`arg capture x string default "…"`).
- ✅ **Library composition / `import`** — split a library across files; merge
  functions, types, and context defaults. See [multi-file-and-imports.md](multi-file-and-imports.md).
- ✅ **Multi-file output** — one source emits many files via the `file "path":`
  directive with `${…}` path interpolation. See [one-source-many-files.md](one-source-many-files.md).
- ✅ **WASM build** — Capy runs in the browser; powers the
  [Playground](playground.md) (`capy-wasm-abi`).
- ✅ **MCP server** — `cmd/capy-mcp` exposes `capy_run` / `capy_check` to AI
  agents. See [mcp.md](mcp.md).
- ✅ **Customizable comment syntax** — the `comments` directive lets a library
  declare its own line-comment markers (no longer hard-coded to `#`).
- ✅ **Source-level metaprogramming** — `define … end` blocks in a script.
  See [metaprogramming.md](metaprogramming.md).

## Near-term

- 🔭 **`else` arm on inner `if`**. Today's workaround is two `if` blocks with a
  negated condition. Tracked as the most-requested inner-DSL gap.
- 🔭 **`capy lint <lib.capy>`** — load-time-independent checks: unused
  functions, dead captures, unreachable priorities, duplicate literals.
- 🚧 **Canonical formatter** — extend `capy fmt` from whitespace hygiene to
  declaration ordering and argument alignment.
- 🔭 **Custom inner-DSL primitives** registered from the embedder
  (`MakeEvaluator(WithPrimitive(...))`) for domain-specific ops.
- 🔭 **`validate` types written in inner Capy** — the most expressive type form
  from the original spec. Requires a small bootstrap loop.

## Surface flexibility

- 🔭 **Configurable syntax** — per-library statement terminator, argument
  separator, block delimiters. Opt-in; today's defaults stay.
- 🔭 **Trailing-comma tolerance** everywhere lists/objects appear.

## Ecosystem

- 🔭 **LSP server** driven by a library's schema (completion + diagnostics for
  the *source* language a library defines). See [editor-tooling.md](editor-tooling.md)
  for what the introspection API already exposes.
- 💡 **Tree-sitter grammar** generation from a library.
- 💡 **`awesome-capy`** — curated registry of libraries (sql, graphql,
  dockerfile, terraform, …). See [the syntax cheat sheet](syntax-cheat-sheet.md)
  to start one.

## Self-hosting milestone

- 💡 **Capy's inner DSL written in Capy.** The parser/evaluator pair for
  function bodies is Rust today; rewriting it as a Capy library would be a
  satisfying self-host moment and would force the language to be expressive
  enough to describe itself.

---

If you want to drive any of these, open an issue with a sketch of the schema
change and a small example library. Contributions welcome.
