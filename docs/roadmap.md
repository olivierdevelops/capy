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
- ✅ **Actionable parse diagnostics** — severity, a stable `E…` code, a primary
  range and secondary labels. A failed parse now names the shape that got
  furthest and what it wanted there (`0.22.0`). See
  [diagnostics.md](diagnostics.md).
- ✅ **Error recovery** — parsing no longer stops at the first mistake; each bad
  region becomes an error node and parsing resumes at the next statement
  (`0.22.0`).
- ✅ **`capy ast <lib> <script> [--json]`** — the parse tree as a readable tree
  or one JSON document, with a span on every node (`0.22.0`). See
  [ast-json.md](ast-json.md).
- ✅ **Infix operator precedence** — `* / %`, `+ -`, comparisons, `and` / `or`,
  prefix `not`, left-associative, with grouping parentheses where unambiguous
  (`0.22.0`). See
  [language reference](language-reference.md#operator-precedence).
- ✅ **Ordered alternation for nonterminals** — `arg capture v call | name | num`,
  so a rule can have more than one shape: the fourth grammar combinator, which
  lets a declared expression grammar nest and a parameter list mix marked and
  unmarked parameters (`0.23.0`). See
  [ordered choice](library-authoring.md#ordered-choice).
- ✅ **Name the nesting limit** — source nested past the parser's depth bound now
  reports `nesting too deep (limit 64) …` with code `E0003` instead of the generic
  message (`0.24.0`). See [diagnostics](diagnostics.md#diagnostic-codes).
- ✅ **`capy version` identifies the build** — an unstamped local build prints the
  crate version instead of `dev`, and the browser introspection JSON carries `alts`
  (`0.24.0`).

## Near-term

- 🔭 **`else` arm on inner `if`**. Today's workaround is two `if` blocks with a
  negated condition. Tracked as the most-requested inner-DSL gap.
- 🔭 **Statement continuation after a block close** — a Mode B block
  (`block_open "{" close "}"`) requires a newline after its closing delimiter,
  so the C-family one-liner `} else {` does not parse; `else` on its own line
  does. The sequence-closed block mode already allows continuation, so the
  asymmetry is the thing to resolve.
- 🔭 **Optional captures before a block opener** — `default` makes a capture
  omittable only when it is *trailing*, and a `block_open` after it means it is
  not. A library declaring one loads clean and then fails to match at run time;
  at minimum `capy check` should refuse it the way it refuses left recursion.
- 🔭 **Expression trees in `capy ast --json`** — `CaptureValue.expr` holds the
  parsed expression in the Rust AST, but the JSON carries only `is_expr` and the
  source text. Analysis that walks expression trees has to embed in Rust today.
  The main gap for [hosting a language](language-frontend.md) from another
  language.
- 🔭 **Byte offsets on `Span`** — spans carry line and column today; an editor
  integration has to convert. Tracked as the main gap for language-server work.
- 🔭 **AST and diagnostics over the wasm ABI** — the browser
  [playground](playground.md) can run a transpile but cannot show a tree or a
  diagnostic, so parser-surface demos are CLI-only.
- 🔭 **Tune the recovery cascade** — the constants that suppress follow-on
  errors are defaults chosen by judgement, not measured against a large corpus.
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
