# FAQ

## What is Capy, in one sentence?

A configurable transpiler engine: you describe a tiny source language in
YAML, Capy reads input written in that language and produces code in any
target language.

## How is it different from a templating engine?

A templating engine substitutes values into text. Capy has a real parser:
it understands a source language *you defined* and matches statements
against library shapes before rendering. Each match can update accumulated
context and a per-line template fragment, then a file template assembles
everything.

## How is it different from a parser generator (ANTLR, lark, tree-sitter)?

Parser generators produce a parser as code in a specific host language.
Capy produces output text from data, requires no codegen step, and ships
with a runtime that drives matching and rendering. Trade-off: Capy
deliberately supports a smaller grammar surface than full PEG/CFG tools.

## Can I generate any target language?

Yes — JSON, Python, TypeScript, SQL, HTML, Makefile, .env, …. The library
decides the target via templates. See `samples/transpile-py/`,
`samples/transpile-json/`, and the cookbook for examples.

## Does Capy execute the user's source code?

No. Capy is a transpiler. `if x ... end` in source emits an `if` in the
target language; it doesn't decide at transpile time whether to skip
rendering. If you want compile-time conditional emission, do it inside
the library function's body against `context`, not against user
variables.

## What's the difference between `write` and `set` / `append`?

- `write` produces text that goes into the output body.
- `set` / `append` / `prepend` / `merge` / `delete` update the
  accumulated context (no body output).

A function can interleave both. The renderer walks `write` statements
to produce text; the run pass walks the state mutations to update
`context`.

## What's `context` for?

State accumulated across all statements. Lists, maps, scalars. Common use:
collect imports/dependencies at the top, emit them via `file_template`.

## Why do captures appear different in templates vs state mutations?

Templates see the **source text** (with quotes for string literals) so a
target like Python receives `"alice"` directly. `set` / `append`
expressions see **evaluated values** so `append context.imports name`
stores the string `json` without quotes.

## How do I define `x = 1`?

Write a function whose pattern has no leading function-name literal:

```
function assign
    arg capture var ident
    arg literal "="
    arg capture value any
end
```

The `any` capture takes the **whole** right-hand side, so `x = 1 + 2 * 3` is one
statement, not three. The engine builds the expression tree with conventional
precedence — see [operator precedence](language-reference.md#operator-precedence)
and the runnable
[`samples/operator-precedence/`](https://github.com/olivierdevelops/capy/tree/main/samples/operator-precedence).
Also see `samples/assembly/`.

## Does Capy stop at the first error?

`capy run` does, deliberately: it is a transpiler, and half-parsed source would
produce half-correct output, so it refuses rather than guess.

`capy ast` does not. Parsing **recovers** — it reports each failure, skips to the
next statement and continues, so one command shows you every error in the file
plus the tree of what did parse. Use `run` to build and `ast` to debug. See
[Diagnostics](diagnostics.md) and
[Tutorial 5](tutorials/05-reading-diagnostics.md).

## Can I get the parse tree?

Yes — `capy ast <lib> <script>` for a readable tree, `--json` for one JSON
document on stdout with a span on every node and a `diagnostics[]` array.
Embedding Capy, `Library::parse` returns the same `ParseResult`. The shape is
fixed by a schema version: see [AST JSON schema](ast-json.md). Note the wasm ABI
does not expose it, so this is a CLI and embedding feature.

## How do I make a block construct with `{ ... }`?

Use Mode B blocks:

```
function for
    arg literal "for"
    arg capture head any
    block_open "{" close "}"
end
```

See [block-functions.md](block-functions.md).

## Can two functions match the same prefix?

Yes. The engine picks the one that consumes more literal tokens, then
falls back to higher `priority:`. Test your patterns with a quick
`capy run` if you're unsure which wins.

## My JSON object keys can be unquoted — is that intentional?

Yes. Object literals in Capy accept either quoted strings or bare
identifiers as keys: `{name: "x"}` and `{"name": "x"}` are equivalent.

## Where do I put utility code?

If it's a domain concept, in `domain/`. If it's an external system, in
`infra/`. If you think you need a `utils/` folder, you probably need to
fit it into one of the six VHCO modules instead — see
[architecture.md](architecture.md).

## How stable is the YAML schema?

Pre-1.0. Breaking changes are called out in `CHANGELOG.md` under a
"Breaking" header per release. We aim for stability but reserve the right
to evolve the schema until 1.0.

## How do I report a bug?

Open a GitHub issue with the bug-report template. Minimal reproductions
go a long way — paste the smallest `lib.capy` + `script.capy` that
reproduces the issue.

## How do I add a new built-in primitive in the inner DSL?

Open an issue first to discuss the API. Implementation lives in
`rust/src/orchestrator/features/inner_evaluator.rs`. Document in `docs/inner-dsl.md`.

## Why is the project structure so opinionated (VHCO)?

Predictability across projects. A new contributor finds the layout
identical from project to project. See [architecture.md](architecture.md)
or the original VHCO writeup.

## What's planned?

See [roadmap.md](roadmap.md).

## How do I use Capy from an LLM/agent?

Either install the Claude Code skill (`skills/capy-author/`) or paste
[CAPY_FOR_LLMS.md](CAPY_FOR_LLMS.md) into your model's context. The latter
is a single-page brief covering the schema + inner DSL + common pitfalls.
