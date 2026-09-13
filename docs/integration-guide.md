# Integrating Capy into Your Project

> A complete, example-driven guide to adopting Capy — what it is, when it
> earns its place in a project, the three ways to wire it in (CLI, Rust
> library, WebAssembly), and five full walkthroughs from first script to
> production integration. Read top-to-bottom the first time; use it as a
> reference after that.

---

## Table of contents

1. [What Capy is (and is not)](#1-what-capy-is-and-is-not)
2. [The 60-second mental model](#2-the-60-second-mental-model)
3. [When Capy helps — concrete signals](#3-when-capy-helps-concrete-signals)
4. [Installing Capy](#4-installing-capy)
5. [The three integration modes](#5-the-three-integration-modes)
6. [Anatomy of a `.capy` library](#6-anatomy-of-a-capy-library)
7. [Capture types — the full reference](#7-capture-types-the-full-reference)
8. [Block modes — every way to nest](#8-block-modes-every-way-to-nest)
9. [The inner DSL (function bodies)](#9-the-inner-dsl-function-bodies)
10. [Interpolation and helpers](#10-interpolation-and-helpers)
11. [Multi-file output](#11-multi-file-output)
12. [Library composition with `import`](#12-library-composition-with-import)
13. [Metaprogramming with `define`](#13-metaprogramming-with-define)
14. [Host capabilities (env, args, files)](#14-host-capabilities-env-args-files)
15. [Introspection — powering editors and tools](#15-introspection-powering-editors-and-tools)
16. [Walkthrough A — a config DSL in a Rust CLI](#16-walkthrough-a-a-config-dsl-in-a-rust-cli)
17. [Walkthrough B — a SQL query DSL on the command line](#17-walkthrough-b-a-sql-query-dsl-on-the-command-line)
18. [Walkthrough C — an HTML component system with a live editor](#18-walkthrough-c-an-html-component-system-with-a-live-editor)
19. [Walkthrough D — a multi-file project scaffolder](#19-walkthrough-d-a-multi-file-project-scaffolder)
20. [Walkthrough E — running Capy in the browser (WASM)](#20-walkthrough-e-running-capy-in-the-browser-wasm)
21. [Integration patterns by ecosystem](#21-integration-patterns-by-ecosystem)
22. [Testing your library](#22-testing-your-library)
23. [Performance, concurrency, and caching](#23-performance-concurrency-and-caching)
24. [Errors and debugging](#24-errors-and-debugging)
25. [An adoption strategy that won't burn you](#25-an-adoption-strategy-that-wont-burn-you)
26. [FAQ](#26-faq)
27. [Appendix 1 — grammar cheat sheet](#appendix-1-grammar-cheat-sheet)
28. [Appendix 2 — CLI reference](#appendix-2-cli-reference)
29. [Appendix 3 — Rust API reference](#appendix-3-rust-api-reference)

---

## 1. What Capy is (and is not)

Capy is a **transpiler engine**. You hand it two things:

1. A **library** — a `.capy` file that describes a source language: which
   statements exist, what they capture, and what each one emits.
2. A **source script** — a file written in the language your library just
   described.

Capy reads the script, matches each statement against the shapes your
library declared, runs the matching function's body, and assembles the
output. The output can be anything text-shaped: HTML, SQL, Python, YAML,
JSON, Terraform, a Dockerfile, a whole directory of files — Capy does not
know or care what the target is.

**Capy ships with zero built-in user-facing keywords.** There is no
default grammar. `if`, `for`, `function`, `table`, `button` — none of
these mean anything until *your* library defines them. This is the single
most important thing to internalise: Capy is not "a language with
features you turn on." It is a machine for building the language you
want.

What Capy is **not**:

- **Not an interpreter.** It does not execute your users' code. `if x …
  end` in a user script does not branch at runtime — it *emits* an `if`
  statement in the target language (or whatever you tell it to emit).
- **Not tied to a target.** The same engine that emits Python can emit
  Kubernetes manifests. The target lives entirely in your library's
  `write` templates.
- **Not a templating language with a fixed host.** Capy runs as a CLI, as
  an embedded Rust library, and as WebAssembly in a browser — same engine,
  same `.capy` files, no per-host dialect.

---

## 2. The 60-second mental model

Three moving parts, in order:

```
   ┌─────────────┐      ┌──────────────┐      ┌────────────┐
   │  lib.capy   │      │  script.capy │      │   output   │
   │ (grammar +  │      │ (user source │      │ (any text  │
   │  templates) │      │  in your DSL)│      │  target)   │
   └──────┬──────┘      └──────┬───────┘      └─────▲──────┘
          │                    │                    │
          │   compile once     │   run many times   │
          └──────────►  Capy engine  ◄──────────────┘
                      lex → match → render
```

1. **Compile the library once.** Capy lexes and parses `lib.capy` into an
   in-memory `Library`. Do this at startup and reuse it.

2. **Run a script.** For each statement in the source:
   - The lexer turns the line into tokens.
   - The parser tries each library function in a deterministic order and
     picks the one whose shape matches (literals + captures).
   - The evaluator runs that function's body — emitting text via `write`
     and/or mutating an accumulating `context`.

3. **Assemble.** A `file_template` block (if present) wraps the
   concatenated output; `file "path":` blocks can split it into many
   files.

That's the whole engine. Everything below is detail on how to express the
grammar and templates richly.

---

## 3. When Capy helps — concrete signals

Capy pays off whenever you have a **gap between how humans want to express
something and the verbose artifact a machine needs.** Some signals that
you're in Capy territory:

### You're hand-writing repetitive structured text

If your team keeps copy-pasting near-identical YAML/JSON/SQL/HTML and
tweaking three fields, a 30-line Capy library turns that into a one-liner
DSL. Example: every microservice repo has a 120-line Kubernetes manifest
that differs only in name, image, and replica count. A `service web image
nginx replicas 3` DSL collapses it.

### You're shipping a config format and YAML hurts

YAML has no validation, no comments-as-data story, no domain vocabulary.
A Capy DSL gives you: typed captures (reject a malformed email at parse
time via a `pattern`), real comments, and keywords that read like your
domain (`route GET "/users" listUsers` instead of a nested mapping).

### You're building a code generator

Prisma turns `model User { name String }` into migrations and a client.
GraphQL SDL turns a schema into resolvers. These are transpilers. With
Capy you write the grammar declaratively instead of hand-rolling a lexer
and parser by hand.

### You want one source to feed many outputs

A single API description can emit an OpenAPI spec, a TypeScript client,
*and* server stubs. Capy's multi-file output (`file "path":` blocks) does
this from one script in one pass.

### You want users to extend the format without recompiling

Because libraries are plain `.capy` files loaded at runtime, your users
(or your ops team, or a plugin author) can add new statement shapes by
editing a text file — no rebuild, no redeploy of your binary.

### You're embedding a "formula" or "expression" surface in an app

Spreadsheet formulas, alert rules, notebook cells, low-code blocks — any
place a product exposes a small authored language. Capy compiles to WASM
and runs in the browser, so the same grammar validates in the editor and
renders on the server.

If none of these fit — if you just need to interpolate a few variables
into a string — Capy is overkill; use your language's templating. Capy
earns its keep when the **grammar itself** is worth defining.

---

## 4. Installing Capy

### As a command-line tool

```sh
cargo install --git https://github.com/olivierdevelops/capy capy-cli
capy version
```

This puts a `capy` binary on your `PATH`. You can now `capy run lib.capy
script.capy` from any shell, wire it into a Makefile, or call it from a
build step in any language.

### As a Rust library dependency

From your crate root. `capy-core` is not on crates.io yet, so depend on it from
git — cargo finds the crate inside the repo's `rust/` subdirectory on its own:

```sh
cargo add --git https://github.com/olivierdevelops/capy capy-core
```

Then `use capy_core::capy::Library;`. If you need a specific commit (for a
feature that's on `main` but not yet tagged), pin it:

```toml
[dependencies]
capy-core = { git = "https://github.com/olivierdevelops/capy", rev = "<commit-sha>" }
```

```sh
cargo tree -p capy-core   # verify what resolved
```

### As a WebAssembly bundle

Capy builds to `wasm32-unknown-unknown` with the standard Rust toolchain:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm capy.wasm
cp rust/playground/web/wasm_exec.js .
```

`wasm_exec.js` here is Capy's own loader shim, not the file Go used to ship: it
defines a `Go` class with the same surface, so a page written against the old
loader works unchanged.

Ship `capy.wasm` + `wasm_exec.js` to the browser and call the exported
globals (covered in [Walkthrough E](#20-walkthrough-e-running-capy-in-the-browser-wasm)).

### Verifying your install

```sh
capy init demo          # scaffold a starter library + script
cd demo
capy run lib.capy script.capy
```

If that prints rendered output, you're ready.

### Version & compatibility

`capy version` prints the build's version string. Capy follows
[Semantic Versioning](https://semver.org), with one **pre-1.0 caveat**:
while the major version is `0`, the library `.capy` schema may evolve
between **minor** versions — pin a version you've tested against rather
than tracking `@latest` in production:

```toml
[dependencies]
# Pin a commit you have tested. Release tags up to v0.20.0 predate the Rust
# engine, so a `tag =` older than that will not resolve this crate at all.
capy-core = { git = "https://github.com/olivierdevelops/capy", rev = "<commit-sha>" }
```

Engine changes are **additive** by policy — new syntax and helpers are
added without breaking existing libraries — but pinning still protects you
from a schema tweak landing mid-release-cycle. This guide reflects the
syntax current as of **v0.20.x** (indexed reads/writes, optional args,
`block_verbatim`/`block_sections`, group types, the full helper set, and
`${line}`/`${col}` source mapping). The [What's new](whats-new.md) page and
[CHANGELOG](https://github.com/olivierdevelops/capy/blob/main/CHANGELOG.md)
track what shipped in each release; if a feature here is missing in your
build, upgrade to the version that introduced it.

---

## 5. The three integration modes

Capy meets your project at whichever boundary is convenient. The grammar
files are identical across all three — you only choose where the engine
runs.

| Mode | You call… | Best when |
|------|-----------|-----------|
| **CLI** | `capy run lib.capy in.capy` | Build steps, codegen pipelines, any language that can shell out |
| **Rust library** | `Library::new(src)` → `lib.run(src)` | Your app is in Rust and wants in-process transpilation, hot-reload, or introspection |
| **WASM** | exported JS globals | Browser playgrounds, live editors, client-side preview |

### Mode 1 — CLI (language-agnostic)

The simplest integration: treat `capy` as a code generator in your build.

```sh
# Makefile
generate:
	capy run schema.capy models.capy > internal/models_gen.go
```

```js
// Node build script
const { execFileSync } = require("node:child_process");
const out = execFileSync("capy", ["run", "lib.capy", "config.capy"], {
  encoding: "utf8",
});
```

```python
# Python build hook
import subprocess
out = subprocess.run(
    ["capy", "run", "lib.capy", "deploy.capy"],
    capture_output=True, text=True, check=True,
).stdout
```

Any ecosystem that can run a subprocess can use Capy. No FFI, no bindings.

### Mode 2 — Rust library (in-process)

When your application is written in Rust, embed the engine directly. You
compile the library once and run many scripts against it:

```rust
use capy_core::capy::Library;

const LIBRARY_SRC: &str = r#"
extension html
function button
    arg literal "button"
    arg capture label string
    write `<button>${unquote label}</button>
`
end
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = Library::new(LIBRARY_SRC)?;
    let out = lib.run(r#"button "Click me""#)?;
    print!("{out}"); // <button>Click me</button>
    Ok(())
}
```

`lib` is safe to reuse across requests. `Run` is re-entrant on a fixed
library — each call gets a fresh accumulating context.

### Mode 3 — WebAssembly (in the browser)

The same `.capy` library, compiled into a `.wasm` blob, runs entirely
client-side. The bundle exposes JavaScript globals you call from the page.
This is what powers live playgrounds and in-editor previews — your grammar
validates and renders without a round-trip to a server.

Pick the mode per surface. A common production shape uses **all three**:
the CLI in CI for codegen, the Rust library in the backend for request-time
rendering, and WASM in the editor for instant feedback — one grammar, three
deployment targets.

---

## 6. Anatomy of a `.capy` library

A library is a sequence of top-level declarations. Here is every section,
annotated:

```
extension html              # informational: suggested output file suffix
output_file "out.html"      # optional: write here instead of stdout

description "A component DSL"  # optional: library-level doc string

comments                    # optional: opt in to user-script comments
    line "#"                #   (Capy ships NO default comment marker)
    line "//"
end

context                     # optional: initial accumulated state
    imports []              #   empty list
    config  {}              #   empty map
    count   0               #   numeric default
    title   "Untitled"      #   string default
end

type Email                  # optional: a library-defined capture type
    base string             #   built-in kind check (any|string|int|float|bool)
    pattern "^[^@]+@[^@]+$"  #   regex on the value's string form
    options "a" "b" "c"     #   enum membership (any of base/pattern/options)
end

function greet              # a statement shape
    description "Emit a greeting."   # optional doc string
    arg literal "greet"     # match this exact token
    arg capture name string # capture a typed value into `name`
    write `Hello, ${unquote name}!
`
end

file_template               # optional: whole-file assembler
    for imp in context.imports
        write `import ${imp}
`
    end
    write body              # `body` = concatenation of all statement output
end
```

### `extension` and `output_file`

`extension` is informational — it tells tooling what suffix the output
should carry. `output_file` (optional) makes the CLI write to a file
instead of stdout.

### `comments`

Capy intentionally ships **no default comment syntax**, because `#` (or
`//`, or `--`) might be meaningful *data* in your target language. Opt in
explicitly:

```
comments
    line "#"
end
```

Now `#`-prefixed lines (leading or trailing) in user scripts are skipped.
You can declare multiple markers.

### `context`

The accumulating state. Statements mutate it via the inner DSL (`set`,
`append`, …); `file_template` reads it back. Use it for things like a
running list of imports, a config map, or a collected route table.

Defaults declare the *type*: `[]` is a list, `{}` is a map, `0` a number,
`"x"` a string.

### `type`

A library-defined capture type. Three optional, ordered constraints:
`base` (built-in kind), `pattern` (regex), `options` (enum). Apply it
anywhere a built-in capture type goes:

```
type Status
    options "todo" "doing" "done"
end

function task
    arg literal "task"
    arg capture state Status   # rejects anything but todo/doing/done
    arg capture title string
    ...
end
```

### `function`

One statement shape. Covered in depth in the next sections. The key rule:

> **If a function declares zero `arg literal` lines, Capy auto-prepends a
> literal of the function's name.** The moment you add any `arg literal`,
> you own the entire shape and the name is *not* prepended.

So this:

```
function greet
    arg capture name any
end
```

matches `greet <something>`. But this:

```
function assign
    arg capture var ident
    arg literal "="
    arg capture value any
end
```

matches `<ident> = <value>` — no leading `assign` token — because you
declared a literal yourself. This is how you build operator-style syntax.

### `file_template`

The final assembler. `body` inside it is the concatenation of every
top-level statement's output. Use it to add a header/footer, emit
collected imports, or wrap everything in a document shell. If you omit it,
the output is just the concatenated statement output.

---

## 7. Capture types — the full reference

A `arg capture NAME TYPE` line binds a value. The built-in types:

| Type | Captures | Use for |
|------|----------|---------|
| `any` | Any value expression: number, string, ident, list, object, dotted path, paren-subcall, comparison | General-purpose values |
| `ident` | A single identifier token | Variable names, keywords |
| `raw` | An identifier OR a string | Loose name-or-literal slots |
| `string` | A quoted string OR a bare identifier | Labels, messages, titles |
| `int` | An integer literal OR a bare identifier | Counts, sizes |
| `float` | A float literal OR a bare identifier | Ratios, coordinates |
| `bool` | `true`/`false` OR a bare identifier | Flags |
| `word` | A shell-style bare word: a maximal run of adjacent tokens with no whitespace — `--oneline`, `k8s/deploy.yaml`, `name=^web$`, `restart-api` as ONE value | Flags, paths, globs, hyphenated names |
| `dotted_ident` | `IDENT(.IDENT)*` as one string — `err.kind`, `a.b.c` | Member paths captured bare |
| `tail` | Every remaining token on the line, joined with original column spacing; quoted tokens keep their quotes | Free-form trailing values, varargs, shell argv |

Two important notes:

1. **Bare identifiers pass the primitive checks.** `int`, `float`, `bool`,
   `string` all accept a bare identifier too, because that identifier
   might be a variable in the target language. So `count n` matches both
   `count 3` and `count limit`.

2. **`tail` returns a joined string, not an array.** It captures
   one-or-more trailing tokens. Quoted tokens are re-emitted *with* their
   quotes, so a spaced quoted argument keeps its slot boundary
   (`cmd -m "fix the bug"` rebuilds as `-m "fix the bug"`, not the
   ambiguous `-m fix the bug`) — split it with a quote-aware splitter, in
   a template (`split argv " "`) or on the host side, if you need a list.
   For "binary plus optional args," pair a required `word` with an
   optional trailing capture (see
   [optional args](#optional-trailing-arguments)).

### Library-defined types in action

```
type SemVer
    base string
    pattern "^v?[0-9]+\\.[0-9]+\\.[0-9]+$"
end

function release
    arg literal "release"
    arg capture version SemVer    # parse-time rejects "banana"
    write `Releasing ${version}
`
end
```

Constraints apply in order: `base` → `pattern` → `options`. A capture that
fails any active constraint is a parse error with a caret-pointed
location.

### Optional trailing arguments

A trailing capture with a `default` may be omitted at the call site:

```
function run
    arg literal "run"
    arg capture name word
    arg capture mode word default "fg"
    write `{"op":"run","name":${asString name},"mode":${asString mode}}
`
end
```

`run restart-api` binds `mode` to `"fg"`; `run restart-api bg` binds it to
`"bg"`. Optional args must be trailing — you cannot have a required arg
after an optional one. This collapses "with/without an extra field"
function families into one shape.

---

## 8. Block modes — every way to nest

A function becomes a *block opener* when it declares a block directive.
Capy has five block modes; pick by how the body is delimited and whether
it should be re-parsed.

### Mode A — named closer + indentation

```
function if
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}
`
end

function end
end
```

Body runs from the next indented line until a matching `end` at the
opener's indentation. `body` is the rendered inner output. Note the
separate `function end` — the closer is itself a (no-op) function.

### Mode B — explicit delimiters

```
function for
    arg literal "for"
    arg capture v ident
    arg literal "in"
    arg capture items any
    block_open "{"
    block_close "}"
    write `for (const ${v} of ${items}) {
${indent 2 body}
}
`
end
```

Body delimited by literal `{` and `}` tokens. No closer function needed.

### Mode C — dedent (no closer keyword)

```
function rule
    arg capture selector word
    block_dedent
    write `${selector} {
${indent 2 body}}
`
end
```

The body ends at the first DEDENT — CSS-style selectors, YAML-style
sections. No closing keyword at all.

### Mode D — verbatim (raw bytes, no re-parsing)

```
function pre
    arg literal "pre"
    arg capture lang ident
    block_verbatim end
    write `<pre><code class="language-${lang}">${escapeHtml body}</code></pre>
`
end
```

The body is captured as raw source bytes — blank lines, `#` lines, and
arbitrary syntax survive untouched, *not* parsed as Capy. This is how you
embed code blocks, SVG, or raw HTML. Combine with the `escapeHtml` helper to
escape it safely.

### Mode E — multi-section blocks

```
function try
    arg literal "try"
    block_sections rescue finally closer end
    write `try {
${body}} rescue {
${rescue}} finally {
${finally}}
`
end

function end
end
```

```
try
    risky_thing
rescue
    handle_it
finally
    cleanup
end
```

The main body renders into `${body}`; each declared section renders into a
local named after the section keyword (`${rescue}`, `${finally}`).
Sections are optional and may appear in any order; an omitted section
renders to the empty string. This is how you express `try/rescue/finally`,
`if/elif/else`, or any keyword-delimited multi-part construct in one
function.

### Context-sensitive block sharing (lookahead)

Two functions can share a leading keyword and disambiguate on whether an
indented block follows:

```
function os_flat               # the flat, single-line form
    arg literal "os"
    arg capture name string
    when_not_followed_by indent
    write `allow os ${unquote name}
`
end

function os_block              # the block form
    arg literal "os"
    arg capture name string
    when_followed_by indent
    block_closer end
    write `if os == ${name}:
${indent 4 body}
`
end
```

`os "linux"` with no indented body → `os_flat`; `os "linux"` followed by
an indented body → `os_block`. The decision is deterministic across runs.

> The parser also **backtracks**: if a block opener matches its header but
> its body fails to parse, Capy restores and tries the next candidate
> instead of erroring. Combined with a total, deterministic candidate
> ordering (ties broken by function name), this makes flat-vs-block
> keyword sharing safe and reproducible.

---

## 9. The inner DSL (function bodies)

A function body is a small, fixed sequence of statements. It updates
`context` and emits output — it never executes user code.

### Output

```
write `literal text with ${interpolation}
`
```

or the multi-line `template` sugar (body captured verbatim, dedented,
`${…}` active):

```
template
    <div class="card">${title}</div>
end
```

### State mutation

```
set    context.title name            # bind a field
append context.tags  tag             # push to a list
prepend context.head item            # push to front
merge  context.config {"k": "v"}     # shallow-merge a map
delete context.tmp                   # remove a field/key
```

A write target may itself be **indexed** — into a map by key, or into a
list by position (negative counts from the end):

```
set    context.counts[word] (add context.counts[word] 1)  # map slot
set    context.buf[i] `… eliminated`                       # overwrite list elem
set    context.buf[-1] last                                # last element
append context.grid[r] cell                                # nested list reached via index
```

The increment idiom above works because arithmetic helpers treat a
missing slot as `0`, so a counter needs no initialisation. Overwriting a
buffered list element in place is the primitive a code generator uses to
**back-patch already-emitted output** (null a dead store, fix a jump
offset) once a later decision is known.

### Control flow (library-side, runs at transpile time)

```
if context.author
    write `By ${context.author}
`
end

for i, t in context.tags
    if i > 0
        write `, `
    end
    write `#${t}`
end

error "duplicate id"     # abort transpilation with a message
```

> This control flow runs **inside the transpiler**, deciding what to emit.
> It is not emitted into the output. `if x … end` here means "if `x` is
> truthy *while generating*, include this text" — not "emit a runtime if."

### Paths and indexed reads

Rooted at `context` (or a `loop` local). A path is a root name followed by
any mix of `.field` steps and `[expr]` index steps — and `[expr]` works in
**read** position exactly as it does on the write side:

```
context.imports                       # field
context.config.api.url                # nested fields
context.scripts[name]                 # map read — `name` evaluated to a key
context.buf[i]                        # list read by a loop index
context.buf[(sub n 1)]                # computed index expression
context.buf[-1]                       # negative → counts from the end
context.grid[i][j]                    # nested index
context.rows[i].name                  # index, then field
```

Read and write semantics are identical: a value written with
`set context.buf[i] …` reads back by the same index. **Map** parents key
on the index's string form; **list** parents key on an integer. A missing
map key or out-of-range list index is `nil` — falsy in a value position
(`if context.seen[k]`) and the empty string in a template
(`${context.palette[name]}`). This retires the
`for k,v in … / if k == name` linear-scan idiom: read the slot directly.

These work in both inner-DSL value position (inside `if`/`for`/`set`) and
`${…}` template interpolation. See the
[Indexed reads showcase](showcase.md#indexed-reads-lists-maps-read-back-by-index-or-key)
for ten worked examples (lookup tables, memoisation, stacks, counters).

### Expressions

- Literals: numbers, strings (with `${interp}`), `true`, `false`, `null`.
- Identifier paths resolve: locals (loop vars) → captures → context.
- Index access `path[expr]` — map by key, list by integer (negative from
  the end); the index is any expression, including a paren-subcall.
- Lists `[1, 2, 3]`, objects `{"k": "v", name: "Alice"}` (unquoted keys ok).
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`; unary `not expr`.
- `(regex_match value pattern)` returns a boolean for use in `if`.

### Render-time locals always available in a body

| Local | Meaning |
|-------|---------|
| `body` | Rendered inner-block output (block functions) |
| `top_level` | `true` when the call is at the file root |
| `depth` | Integer AST depth (0 at root) |
| `line` / `col` | 1-indexed source position of this statement |
| section names | For `block_sections`, each section's rendered body |

A user capture with the same name as a local **wins** — the locals are
seeded first, then captures overlay them. `${line}` is invaluable for
source-mapping: stamp `data-line="${line}"` into generated HTML and an
editor can map output back to source.

---

## 10. Interpolation and helpers

Inside a `write` backtick literal, `${expr}` interpolates and you can pipe
through helpers two ways: `${expr | helper}` or `${helper arg expr}`.

The complete, authoritative list lives in the
[Built-in function cookbook](function-cookbook.md) (one worked example per
helper, mirrored by the CI-checked `samples/builtin-functions/`). The full
set, grouped:

**Layout & case**

| Helper | Effect |
|--------|--------|
| `indent N` | Pad every line with N spaces — use for block bodies |
| `lower` / `upper` | Lower/upper-case the whole string |
| `pascalCase` / `camelCase` / `snakeCase` | Re-case an identifier (`PascalCase`, `camelCase`, `snake_case`) |
| `dasherize` | Convert to `kebab-case` |

**Strings & escaping**

| Helper | Effect |
|--------|--------|
| `unquote` | Strip surrounding quotes from a captured string |
| `toQuoted` | Wrap a string in `"…"` |
| `decoded` | Resolve escape sequences (`\n`, `\t`, `\"`, …) without choking on embedded quotes |
| `unescape` | Single-pass `strconv` unescape of a quoted source string |
| `escapeHtml` | HTML-escape (`<`, `>`, `&`, `"`, `'`) — your XSS guard |
| `asString` | Normalise a capture to ONE valid JSON string — quotes iff not already a string. Handles bare ident OR quoted string uniformly |
| `trimPrefix S` / `trimSuffix S` | Drop a leading/trailing substring |

**Collections**

| Helper | Effect |
|--------|--------|
| `join SEP` | Join a list with a separator |
| `split SEP` | Split a string into a list (quote-aware) |
| `nonEmpty` | Drop empty entries from a list |
| `toJSON` / `toJSONIndent` | JSON-marshal a value (compact / 2-space) |
| `toPyLit` | Python literal formatting (True/False/None, lists, dicts) |

**Numbers**

| Helper | Effect |
|--------|--------|
| `add` / `sub` / `mul` | Integer arithmetic (`add a b`, …) |
| `div` / `mod` | Integer quotient / remainder (return `0` when divisor is `0`) |
| `align N A` | Round `N` **up** to the next multiple of `A` — ABI-correct layout offsets |
| `percent N D` | `N/D` as an integer percentage |
| `stars N` | A 5-slot `★`/`☆` bar for a 0–5 rating |

Pipe through a helper two ways: `${expr | helper}` or `${helper arg expr}`.
Helpers compose: `${escapeHtml (decoded text)}` decodes escapes, then
HTML-escapes the result.

### The quoting problem, solved

When a `string` capture might be a bare ident *or* a quoted literal, you
want exactly one valid JSON string out either way. `asString` does that:

| Source | `${asString bin}` |
|--------|-------------------|
| `run git` | `"git"` |
| `run "git"` | `"git"` |
| `emit "he said \"hi\""` | `"he said \"hi\""` |

No more double-quoting real strings; no `unquote`+`toJSON` dance.

### Escaping HTML

Any user value going into an HTML target should pass through `escapeHtml`:

```
write `<p>${escapeHtml (unquote text)}</p>
`
```

`<script>` becomes `&lt;script&gt;` — the XSS hole is closed at the
template boundary.

---

## 11. Multi-file output

A single source can emit a whole directory tree. Declare `file "path":`
blocks at the top level; each renders independently and reads the shared
`context`:

```
function project
    arg literal "project"
    arg capture name string
    block_closer end
    set context.name name
end

function route
    arg literal "route"
    arg capture method ident
    arg capture path string
    arg capture handler ident
    append context.routes {method: method, path: path, handler: handler}
end

function end
end

file "README.md"
    write `# ${unquote context.name}

Generated by Capy from a ${len context.routes}-route description.
`
end

file "app/routes.py"
    for r in context.routes
        write `@app.${lower r.method}("${unquote r.path}")
def ${r.handler}(): ...

`
    end
end
```

From the Rust API, `run_multi` returns the file map:

```rust
let (out, files) = lib.run_multi(script_src)?;
// files["README.md"], files["app/routes.py"], …
for (path, content) in &files {
    std::fs::write(std::path::Path::new(out_dir).join(path), content)?;
}
```

On the CLI, `capy run` writes each declared file to disk. This is the
"one description → OpenAPI + client + server" pattern: declare the API
once, fan out to as many target files as you need.

---

## 12. Library composition with `import`

Libraries scale by sharing the boring parts. A library can `import`
others; the imported types and functions merge in before it runs:

```
import "common/types.capy"      # shared Email, SemVer, Slug types
import "common/syntax.capy"     # shared `tag`, `meta` keywords

extension md

function post
    arg literal "post"
    arg capture title string
    block_closer end
    set context.title title
    ...
end
```

Define your `Email` regex once in `common/types.capy`; import it into
every library that needs it. This keeps a single source of truth for
validation rules and shared vocabulary, and lets a large grammar be split
into reviewable files.

---

## 13. Metaprogramming with `define`

A **user script** can define new statement shapes inline with `define …
end` blocks. They are extracted, merged into the library (source defines
win on conflict), and then the rest of the script runs against the
augmented grammar:

```
define shout
    arg literal "shout"
    arg capture msg string
    write `<h1>${upper (unquote msg)}</h1>
`
end

shout "hello"      # → <h1>HELLO</h1>
```

This works identically on the CLI, in the embedded Rust library (`run` /
`RunMulti` handle it), and in the WASM playground — so a user can extend
the language from inside their own document without touching the library
file. It's how you give power users an escape hatch without recompiling
anything.

---

## 14. Host capabilities (env, args, files)

Function bodies can pull values from the host environment at transpile
time via four primitives:

```
set context.environment (env "ENV")        # os.Getenv
set context.version     (arg 0)            # os.Args[n]
set context.argc        (arg_count)        # len(os.Args)
set context.secrets     (read_file "k.txt") # os.ReadFile
```

These read through a `domain.Host`. By default (after `NewLibrary`) the
host is `NoOpHost` — every primitive returns the zero value and
`read_file` errors. This is the safe default: an untrusted library cannot
read your environment or filesystem.

To opt in (only for trusted library source), install `OSHost`:

```rust
use capy_core::infra::os_host::OsHost;
use std::sync::Arc;

lib.set_host(Some(Arc::new(OsHost {
    // CLI args after the library + script paths
    user_args: std::env::args().skip(3).collect(),
    // `read_file` resolves relative paths against this directory
    base_dir: ".".into(),
})));
```

The CLI installs `OSHost` automatically (it's running trusted local
files); the WASM playground keeps `NoOpHost` (no filesystem in the
browser sandbox). This split is deliberate: the same library degrades
gracefully — `env "ENV"` is `""` in the browser and the real value on the
CLI.

---

## 15. Introspection — powering editors and tools

A compiled library can describe itself. This is the key to building
editor support (autocomplete, hover-docs, syntax highlighting) without
hand-maintaining a parallel catalogue that drifts out of sync.

```rust
for f in lib.introspect() {
    println!("{}  block={:?}  priority={}", f.name, f.block, f.priority);
    for a in &f.args {
        if a.kind == "literal" {
            println!("    literal {:?}", a.value);
        } else {
            println!("    capture {} : {}  {}", a.name, a.type_, a.description);
        }
    }
}
```

`FunctionInfo` carries the name, description, full arg shapes (literal
value / capture name + type + description + optional/default), the block
kind (`"closer:end"`, `"verbatim:end"`, `"sections:rescue,finally
closer:end"`, `"dedent"`, `"open:{ close:}"`), and priority.

Two companion methods:

```rust
lib.function_names();   // sorted Vec<String> of declared function names
lib.comment_markers();  // the library's comment markers, e.g. ["#"]
```

An editor derives its entire keyword set, argument hints, and comment
highlighting from these — change the library, and the editor follows
automatically. `capy docs lib.capy` uses the same data to generate
Markdown reference docs.

---

## 16. Walkthrough A — a config DSL in a Rust CLI

**Goal.** Replace a verbose JSON config with a friendly DSL, parsed
in-process by a Rust program.

### Step 1 — the target

Suppose your service config looks like this in JSON today:

```json
{
  "service": "checkout",
  "replicas": 3,
  "routes": [
    {"method": "GET", "path": "/health", "handler": "health"},
    {"method": "POST", "path": "/pay", "handler": "pay"}
  ]
}
```

### Step 2 — the DSL you want

```
service checkout
replicas 3
route GET  /health health
route POST /pay     pay
```

### Step 3 — the library

```
extension json

context
    service ""
    replicas 1
    routes []
end

function service
    arg literal "service"
    arg capture name ident
    set context.service name
end

function replicas
    arg literal "replicas"
    arg capture n int
    set context.replicas n
end

function route
    arg literal "route"
    arg capture method ident
    arg capture path word
    arg capture handler ident
    append context.routes {method: method, path: path, handler: handler}
end

file_template
    write `{
  "service": ${toQuoted context.service},
  "replicas": ${context.replicas},
  "routes": ${toJSONIndent context.routes}
}
`
end
```

Note `path` is captured as `word` so `/health` survives as one token.

### Step 4 — wire it into Rust

```rust
use capy_core::capy::Library;

// include_str! is the equivalent of Go's //go:embed — the library source is
// compiled into the binary, so there is no file to ship alongside it.
const LIBRARY_SRC: &str = include_str!("config.capy");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = Library::new(LIBRARY_SRC)?;
    let path = std::env::args().nth(1).ok_or("usage: tool <script.capy>")?;
    let src = std::fs::read_to_string(path)?;
    print!("{}", lib.run(&src)?);
    Ok(())
}
```

`//go:embed config.capy` bakes the library into your binary — no runtime
file dependency. Compile the library once at startup; reuse `lib` for
every config you parse.

### Step 5 — run it

```sh
cargo run -- service.conf
# {
#   "service": "checkout",
#   "replicas": 3,
#   "routes": [ ... ]
# }
```

You now have a typed config DSL with parse-time validation (`replicas`
must be an int) in ~40 lines, no hand-written parser.

---

## 17. Walkthrough B — a SQL query DSL on the command line

**Goal.** A readable query DSL that compiles to SQL, used as a build-step
code generator via the CLI.

### The library (`query.capy`)

```
# A tiny query DSL → SQL.
extension sql

comments
    line "#"
end

function select
    arg literal "select"
    arg capture cols any
    arg literal "from"
    arg capture tbl ident
    arg literal "where"
    arg capture cond any
    write `SELECT ${cols} FROM ${tbl} WHERE ${cond};
`
end

function insert
    arg literal "insert"
    arg literal "into"
    arg capture tbl ident
    arg capture vals any
    write `INSERT INTO ${tbl} VALUES ${vals};
`
end
```

### The source (`report.capy`)

```
# Monthly active users
select [id, email] from users where active == true
insert into audit ["report", now]
```

### Run it

```sh
capy run query.capy report.capy
# SELECT [id, email] FROM users WHERE active == true;
# INSERT INTO audit ["report", now];
```

### Integrate into a build

```makefile
# Makefile
queries.sql: report.capy query.capy
	capy run query.capy report.capy > $@
```

Now `make queries.sql` regenerates SQL whenever the DSL source changes.
Because the CLI is language-agnostic, the exact same step works from a
Node `package.json` script, a Python `invoke` task, a Rust `build.rs`, or
a CI job — anything that can run a binary.

---

## 18. Walkthrough C — an HTML component system with a live editor

**Goal.** A component DSL whose output is HTML, with an editor that knows
the components (autocomplete + hover-docs) — all derived from the library.

### The library (`components.capy`)

```
extension html

comments
    line "#"
end

function card
    description "A bordered content card."
    arg literal "card"
    arg capture title string  "The card heading."
    block_closer end
    template
        <section class="card" data-line="${line}">
          <h2>${escapeHtml (unquote title)}</h2>
          ${body}
        </section>
    end
end

function p
    description "A paragraph of prose."
    arg literal "p"
    arg capture text string  "The paragraph text."
    write `<p data-line="${line}">${escapeHtml (unquote text)}</p>
`
end

function end
end
```

Two things to notice: `data-line="${line}"` stamps the source line onto
each element (for scroll-sync / click-to-source in the editor), and `escapeHtml`
escapes user content (XSS-safe).

### The source

```
card "Welcome"
    p "Hello, world."
    p "This renders to safe HTML."
end
```

### Powering the editor with introspection

```rust
let lib = Library::new(components_src)?;

#[derive(serde::Serialize)]
struct EditorMeta {
    keywords: Vec<String>,
    comments: Vec<String>,
    docs: Vec<FunctionInfoDto>,
}

let meta = EditorMeta {
    keywords: lib.function_names(),  // ["card", "end", "p"]
    comments: lib.comment_markers(), // ["#"]
    docs: lib.introspect().into_iter().map(Into::into).collect(),
};
serde_json::to_writer(w, &meta)?;
```

`capy-core` has no `serde` dependency — the engine only ever *writes* JSON — so
derive `Serialize` on your own DTO and convert from `FunctionInfo` rather than
expecting it to serialize itself.

The editor consumes `meta`:

- **Syntax highlighting** colours the `Keywords` and `Comments`.
- **Autocomplete** suggests `card`, `p`, `end` with their arg hints.
- **Hover-docs** show each function's `Description` and arg descriptions.
- **Scroll-sync** maps a clicked output element back to source via the
  `data-line` attribute the library stamped.

When you add a component to the library, the editor picks it up with **zero
extra code** — the metadata is derived, not duplicated.

### Rendering on the server

```rust
match lib.run(user_source) {
    // e.line / e.col / e.hint are on the error — surface it inline in the editor
    Err(e) => return render_error(&e),
    Ok(out) => w.write_all(out.as_bytes())?,
}
```

---

## 19. Walkthrough D — a multi-file project scaffolder

**Goal.** One description → a complete project tree. This is the
"scaffolder" pattern (think `create-react-app`, but your own grammar).

### The library (`scaffold.capy`)

```
extension py

comments
    line "#"
end

context
    name ""
    description ""
    routes []
end

function project
    arg literal "project"
    arg capture name string
    block_closer end
    set context.name name
end

function describe
    arg literal "describe"
    arg capture v string
    set context.description v
end

function route
    arg literal "route"
    arg capture method ident
    arg capture path string
    arg capture handler ident
    append context.routes {method: method, path: path, handler: handler}
end

function end
end

file "README.md"
    write `# ${unquote context.name}

${unquote context.description}

Generated from a ${len context.routes}-route description.
`
end

file "pyproject.toml"
    write `[project]
name = ${toQuoted context.name}
version = "0.1.0"
`
end

file "app/__init__.py"
    write `from flask import Flask
app = Flask(__name__)

`
    for r in context.routes
        write `@app.route("${unquote r.path}", methods=["${r.method}"])
def ${r.handler}():
    return {"ok": True}

`
    end
end
```

### The source (`service.capy`)

```
project "checkout-service"
    describe "Handles payments and receipts."
    route GET  "/health" health
    route POST "/pay"     pay
end
```

### Generate the tree (Rust)

```rust
let (out, files) = lib.run_multi(&src)?;
let _ = out; // empty when everything goes to file blocks
for (path, content) in &files {
    let full = std::path::Path::new("generated").join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(full, content)?;
}
```

Result:

```
generated/
├── README.md
├── pyproject.toml
└── app/
    └── __init__.py
```

One 50-line source description produced a runnable Flask project skeleton.
Change the routes, re-run, regenerate. The same source could feed *more*
file blocks — a Dockerfile, a CI config, an OpenAPI spec — all from one
pass.

---

## 20. Walkthrough E — running Capy in the browser (WASM)

**Goal.** A live playground / in-editor preview that transpiles entirely
client-side, no server round-trip.

### Build the bundle

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm web/capy.wasm
cp rust/playground/web/wasm_exec.js web/
```

### Load it in the page

```html
<script src="wasm_exec.js"></script>
<script>
  const go = new Go();
  WebAssembly.instantiateStreaming(fetch("capy.wasm"), go.importObject)
    .then((result) => {
      go.run(result.instance);
      // The shim registers the capy* global functions on `globalThis` here.
      boot();
    });

  function boot() {
    const lib = `
extension html
function p
    arg literal "p"
    arg capture text string
    write \`<p>\${text}</p>\n\`
end`;
    const script = `p "hello from the browser"`;

    // The exact global name depends on the bundle's exports; the
    // capy-wasm bundle exposes a render entry point that takes the
    // library source and the script source and returns the output
    // (or an error string with a caret-pointed location).
    const out = capyRender(lib, script);
    document.getElementById("out").textContent = out;
  }
</script>
<pre id="out"></pre>
```

### Why this matters

The browser runs the **same engine** as your CLI and your Rust backend. So:

- The editor validates as the user types — same grammar, same errors.
- Preview is instant — no network latency, works offline.
- Your server isn't in the hot path for every keystroke.

Because libraries are just `.capy` strings, you can fetch a different
grammar at runtime and re-render — hot-swappable languages in a static
page. Introspection (`Introspect`, `CommentMarkers`) is exposed to JS too,
so the in-browser editor gets the same autocomplete/hover metadata as a
native one.

---

## 21. Integration patterns by ecosystem

### Rust services

Embed via `Library::new`. Compile the library at process start (or
lazily, once, behind a `sync.Once`), store the `*Library` on your server
struct, and call `Run`/`RunMulti` per request. The library is safe to
share across threads for reads: `Library` is `Send + Sync` and `run` takes
`&self`, so no lock is needed.

### Node / TypeScript

Two options: (1) shell out to the `capy` CLI from a build script or a
worker, or (2) load the WASM bundle in a Node process via `wasm_exec.js`.
Use the CLI for build-time codegen; use WASM if you need in-process
transpilation without a subprocess.

### Python / Ruby / Rust / anything

Shell out to the CLI:

```python
import subprocess, json
def transpile(lib_path, src_path):
    return subprocess.run(
        ["capy", "run", lib_path, src_path],
        capture_output=True, text=True, check=True,
    ).stdout
```

For multi-file output, point `capy run` at a library with `file` blocks
and it writes the tree to disk; your script reads the result.

### Build systems & CI

Capy is a deterministic, dependency-free binary. Drop `capy run` into a
Makefile target, a `build.rs`, a `package.json` script, or a CI job.
Cache the output keyed on the library + source hashes; regenerate only on
change. Because candidate ordering is total and deterministic, the same
inputs always produce the same output — safe to commit generated files and
diff them.

### Front-end / editors

Ship the WASM bundle. Drive syntax highlighting, autocomplete, and
hover-docs off `Introspect()` / `CommentMarkers()`. Stamp `${line}` into
output for scroll-sync and click-to-source.

---

## 22. Testing your library

Treat your library like code: pin its behavior with golden tests.

### Golden-file tests (CLI)

The repo's sample convention is a directory with `lib.capy`,
`script.capy`, and `script.expected.txt`. A test runs the library against
the script and diffs against the expected output:

```rust
#[test]
fn library_matches_golden() {
    let lib = Library::from_file("lib.capy").expect("library compiles");
    let src = std::fs::read_to_string("script.capy").unwrap();
    let got = lib.run(&src).expect("transpile");
    let want = std::fs::read_to_string("script.expected.txt").unwrap();
    assert_eq!(got, want);
}
```

### Unit tests with inline sources

```rust
#[test]
fn button_escapes() {
    let lib = Library::new(LIBRARY_SRC).expect("library compiles");
    let out = lib.run(r#"button "<script>""#).expect("transpile");
    assert!(!out.contains("<script>"), "XSS: unescaped output");
}
```

### What to cover

- **Happy path** per function shape.
- **Validation rejections** — a malformed `Email` capture should error.
- **Determinism** — run the same source 50× and assert identical output
  (guards against keyword-collision flakiness).
- **Edge cases** — empty bodies, omitted optional args, omitted block
  sections, Unicode prose, embedded quotes.

`cargo test --workspace` should be green before you ship a grammar.

---

## 23. Performance, concurrency, and caching

- **Compile once.** `NewLibrary` does the expensive lex+parse of the
  grammar. Do it at startup, never per-request. Store the `*Library`.
- **`Run` is re-entrant.** Each call runs a fresh accumulating context on
  the fixed, immutable compiled library. Concurrent `Run` calls on the
  same `*Library` are safe (they don't mutate shared state); don't mutate
  the `Library` itself concurrently.
- **Output is deterministic.** Total candidate ordering means identical
  inputs → identical output. Cache aggressively, keyed on
  `hash(librarySrc) + hash(scriptSrc)`.
- **WASM cost is the bundle download**, paid once. Transpilation itself is
  fast and in-process; there's no per-keystroke network cost.
- **Multi-file** rendering is a single pass over the source plus one pass
  per `file` block; it scales with output size, not file count overhead.

---

## 24. Errors and debugging

Capy errors are **caret-pointed at `line:col`**. When `Run` returns an
error, surface it directly — it tells the user exactly where the source
broke.

```rust
if let Err(e) = lib.run(src) {
    // e.g. "script.capy:3:14: no function matches `route GT /x h`"
    eprintln!("{e}");
}
```

Debugging checklist:

1. **`capy check lib.capy`** — validate the library loads cleanly. Run
   this after every grammar edit.
2. **Run against a minimal script** — isolate the failing statement.
3. **Check auto-name-prepend** — if a function unexpectedly doesn't match,
   remember that *any* `arg literal` disables the automatic name literal.
   A stray `arg literal "in"` removes the leading function-name match.
4. **Indentation** — bodies use 4 spaces or 1 tab per level. 2-space
   indent breaks the lexer.
5. **`{}` ambiguity** — `{...}` is an object literal by default. For
   `{}`-delimited *blocks*, declare `block_open "{"` / `block_close "}"`.
6. **Quoting** — `string` captures keep their source quotes in templates.
   Use `unquote` to strip, `asString` to normalise, `escapeHtml` to escape.
7. **Determinism** — if a parse seems to flip between runs, you have a
   keyword collision; disambiguate with distinct keywords or lookahead.

---

## 25. An adoption strategy that won't burn you

Capy is **additive** — adopting it never forces a rewrite. A low-risk path:

1. **Start at a build step.** Pick one painful hand-maintained artifact (a
   config, a manifest, a repetitive SQL file). Write a small library,
   generate that one file via `capy run` in your build. Commit both the
   source DSL and the generated output so reviewers see the diff.

2. **Pin your version.** Use a tagged release, or pin an exact commit if
   you need an unreleased feature:

   ```sh
   cargo add --git https://github.com/olivierdevelops/capy capy-core
   go list -m github.com/olivierdevelops/capy   # confirm what resolved
   ```

3. **Test the grammar.** Add golden tests (section 22) before anyone
   depends on the output. Lock determinism with a repeat-run test.

4. **Grow into embedding.** Once the grammar earns trust, move
   transpilation in-process (`Library::new`) for hot-reload, better
   errors, and introspection-driven tooling.

5. **Add an editor.** Derive autocomplete/hover/highlighting from
   `Introspect()`. Ship a WASM preview if the surface is user-facing.

6. **Share with `import`.** As grammars multiply, factor shared types and
   keywords into common `.capy` files.

Each step is independently valuable and reversible. You're never "all in"
until you choose to be.

---

## 26. FAQ

**Does Capy run my users' code?**
No. It transpiles. `if x … end` emits an `if`; it does not branch at
runtime. The only code that runs at transpile time is the inner DSL in
your library's function bodies.

**Can I emit a binary / non-text format?**
Capy's output is text. For binary, emit a text intermediate (assembly,
a builder script, base64) and post-process.

**What if two functions could match the same statement?**
Candidates are tried in a total, deterministic order (priority, then
literal specificity, then function name). A block opener whose body fails
to parse backtracks to the next candidate. Use `priority` to bias, or
`when_followed_by indent` / `when_not_followed_by indent` to disambiguate
flat-vs-block sharing.

**Can users extend the grammar without my library?**
Yes — `define … end` blocks in a user script add statement shapes inline,
merged before the rest of the script runs. Works on CLI, embedded, and
WASM.

**Is it safe to run an untrusted library?**
The default host is `NoOpHost` — no env, no args, no file reads. Only call
`set_host(Some(Arc::new(OsHost { .. })))` for libraries you trust. The transpiler does
not execute user code regardless.

**How do I get Unicode prose to work?**
It works out of the box — accented Latin, CJK, emoji, em-dashes all
tokenize as identifiers. Capture trailing prose with a `tail` capture (or
a `bare` catch-all) and it round-trips.

**Can I have comments in user scripts?**
Only if your library opts in with a `comments` block. Capy ships no
default marker because `#`/`//`/`--` may be data in your target.

**How do I map output back to source (for an editor)?**
Use the `${line}` / `${col}` render locals — stamp them into the output
(e.g. `data-line="${line}"`) and your editor can map clicks/scroll back to
the source statement.

---

## Appendix 1 — grammar cheat sheet

```
# ─── library-level ───────────────────────────────────────────
extension <suffix>
output_file "<path>"
description "<doc>"

comments
    line "<marker>"        # repeatable
end

context
    <name> []              # list
    <name> {}              # map
    <name> 0               # number
    <name> "default"       # string
end

type <Name>
    base <any|string|int|float|bool>
    pattern "<regex>"
    options "a" "b" "c"
end

# ─── functions ───────────────────────────────────────────────
function <NAME>
    description "<doc>"
    priority <int>
    bare                                  # opt out of auto-name-prepend

    arg literal "<TEXT>"
    arg capture <name> <TYPE>
    arg capture <name> <TYPE> default "<V>"   # optional, must be trailing

    # one block directive (optional):
    block_closer <NAME>
    block_open "<X>" close "<Y>"
    block_dedent
    block_verbatim <NAME>
    block_sections <S1> <S2> closer <CLOSER>

    # lookahead gate (optional):
    when_followed_by indent
    when_not_followed_by indent

    # body (inner DSL):
    write `text ${expr} ${helper arg}`
    template
        multi-line ${interp}
    end
    set / append / prepend / merge / delete <path> <value>
    set context.map[key] <value>          # indexed write (map key / list index)
    set context.buf[-1] <value>           # negative index = from the end
    if <expr> … end                       # reads may index: if context.seen[k]
    for <v> in <expr> … end
    for <i>, <v> in <expr> … end
    error "<message>"
end

# ─── assembly ────────────────────────────────────────────────
file_template
    write body
end

file "<path>"
    write `…`
end

# ─── imports & metaprogramming ───────────────────────────────
import "<path.capy>"           # in a library
define <NAME> … end            # in a user script
```

### Capture types

`any` `ident` `raw` `string` `int` `float` `bool` `word` `dotted_ident`
`tail` — plus any library `type`.

### Indexed access (read & write)

```
context.map[key]      # map read/write by key
context.list[i]       # list read/write by integer index
context.list[-1]      # negative = from the end
context.grid[i][j]    # nested
context.buf[(sub n 1)]# computed index expression
```

Missing map key / out-of-range list index → `nil` (falsy / empty).

### Render-time locals

`body` `top_level` `depth` `line` `col` — plus block-section names.

### Helpers

Layout/case: `indent N` `lower` `upper` `pascalCase` `camelCase`
`snakeCase` `dasherize`
Strings: `unquote` `toQuoted` `decoded` `unescape` `escapeHtml` `asString`
`trimPrefix S` `trimSuffix S`
Collections: `join SEP` `split SEP` `nonEmpty` `toJSON` `toJSONIndent`
`toPyLit`
Numbers: `add` `sub` `mul` `div` `mod` `align N A` `percent N D` `stars N`

Full reference with examples: [function cookbook](function-cookbook.md).

---

## Appendix 2 — CLI reference

```sh
capy run <lib.capy> <script.capy>     # transpile (stdout, or file blocks to disk)
capy check <lib.capy>                 # validate a library, report load errors
capy docs <lib.capy>                  # generate Markdown reference docs
capy init [<dir>]                     # scaffold a starter library + script
capy version                          # print version
capy help [<command>]                 # command help
```

Typical loop:

```sh
capy init myproj && cd myproj
capy check lib.capy        # after every grammar edit
capy run lib.capy script.capy
capy docs lib.capy > REFERENCE.md
```

---

## Appendix 3 — Rust API reference

```rust
use capy_core::capy::{render_library_docs, Library};
use capy_core::infra::os_host::OsHost;
use std::sync::Arc;

// Compile a library (do this once, reuse — Library is Send + Sync).
let lib = Library::new(library_src)?;          // from a string
let lib = Library::from_file("lib.capy")?;     // from disk

// Transpile.
let out = lib.run(script_src)?;                // single output string
let (out, files) = lib.run_multi(script_src)?; // + BTreeMap<path, content>

// Host capabilities (opt in only for trusted libraries).
let mut lib = lib;
lib.set_host(Some(Arc::new(OsHost {
    user_args: vec![],
    base_dir: ".".into(),
})));
// default after Library::new is NoOpHost (no env/args/file access)

// Metadata.
lib.extension();         // declared `extension`
lib.output_file();       // declared `output_file`
lib.function_names();    // sorted Vec<String>
lib.comment_markers();   // declared comment markers
lib.introspect();        // Vec<FunctionInfo> — name, args, block kind, priority
render_library_docs(&lib); // Markdown docs string (same as `capy docs`)
```

### `FunctionInfo` / `ArgInfo`

```rust
pub struct FunctionInfo {
    pub name: String,
    pub description: String,
    pub args: Vec<ArgInfo>,
    // "" | "closer:NAME" | "open:X close:Y" | "dedent" |
    // "verbatim:NAME" | "sections:S1,S2 closer:CLOSER"
    pub block: String,
    pub priority: i64,
}

pub struct ArgInfo {
    pub kind: String,        // "literal" | "capture"
    pub value: String,       // literal text (kind == "literal")
    pub name: String,        // capture name (kind == "capture")
    pub type_: String,       // capture type — `type` is a Rust keyword
    pub description: String, // trailing doc string on the arg line
    pub optional: bool,      // trailing capture declared with `default`
    pub default: String,     // value bound when an optional capture is omitted
}
```

### Concurrency contract

A compiled `*Library` is immutable after `NewLibrary`. `Run` / `RunMulti`
are safe to call concurrently from multiple threads on the same
`*Library`. Do not call `SetHost` concurrently with `Run`. Each `Run`
gets a fresh accumulating context — no state leaks between calls.

---

*Built with [Capy](https://github.com/olivierdevelops/capy). One grammar; CLI,
embedded, and in-browser; any text target.*
