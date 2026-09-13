# Capy

> A transpiler engine with zero default grammar. You define a tiny source
> language in a `.capy` library file, and Capy generates the target output.

[![CI](https://github.com/olivierdevelops/capy/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierdevelops/capy/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/olivierdevelops/capy?include_prereleases)](https://github.com/olivierdevelops/capy/releases)
[![License: Source-Available](https://img.shields.io/badge/License-Source--Available-orange.svg)](LICENSE)

Capy reads source code, matches each statement against library-defined
function shapes, and for each match (a) renders a template fragment and
(b) updates an accumulated **context**. A top-level `file_template`
assembles `body` + `context` into the final output file.

There are **no built-in keywords**. `if`, `loop`, `=`, blocks, comments —
all defined by the library, or not at all if your DSL doesn't need them.

---

## 30-second teaser

**`lib.capy`**

```
extension py

context
    imports []
end

function import
    arg literal "import"
    arg capture name ident
    append context.imports name
end

function say
    arg literal "say"
    arg capture msg any
    write `print(${msg})
`
end

file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

**`script.capy`**

```
import json
import os
say "hello, world"
```

**Output (Python)**

```python
import json
import os
print("hello, world")
```

The library is the entire grammar. Swap it and the same engine produces
HTML, SQL, JSON, Makefiles — anything you can describe with patterns,
templates, and run blocks.

Libraries are always written in Capy's own `.capy` syntax. An earlier
version accepted YAML with Go templates for output bodies; that format was
removed, because it meant juggling three languages at every edit. See
[`docs/library-authoring.md`](docs/library-authoring.md).

---

## Why Capy is genuinely useful for AI agents

Two properties most people miss:

1. **Token compression** — agents emit short structured Capy; the engine
   deterministically expands it into long boilerplate-heavy target code.
   12 lines of game-DSL → 67-line runnable canvas game (**5.5×**). 9 lines
   of landing-page DSL → 54 lines of responsive HTML+CSS (**6.0×**). In
   an agent loop the gap compounds — the library is reusable across
   hundreds of invocations.

2. **Sandboxing for free** — the library is the complete grammar. A SQL
   DSL whose `TableName` is an enum **cannot** emit `DROP TABLE`. A
   shell DSL whose `Command` whitelists `ls`/`cat`/`grep` **cannot**
   invoke `rm`. No prompt injection, no post-hoc filtering. The grammar
   is the boundary.

See [docs/ai-agents.md](docs/ai-agents.md) for the token-cost math,
sandboxing patterns, and integrations with Claude Code, Cursor,
Continue, and Aider.

> **50 worked demos** live under [`samples/`](samples/). Compact DSLs
> producing substantial useful targets: a full HTML5 canvas game (12
> lines → 67 lines of working HTML+CSS+JS), responsive landing pages,
> React TSX components with hooks, complete Express/Flask/FastAPI
> servers, production code generators (Python, TypeScript, Go, NASM
> x86-64, Bash, Cobra CLIs), infrastructure (Terraform, Kubernetes,
> Dockerfile, nginx, systemd, GitHub Actions, Prometheus alerts, Chrome
> extensions), schemas (PostgreSQL DDL, Prisma, Zod, XState, GraphQL,
> Protobuf, OpenAPI), and docs (CV, changelog, invoice, blog, Slack
> Block Kit, Mermaid diagrams).

---

## Install

```sh
# Rust users — CLI
cargo install --git https://github.com/olivierdevelops/capy capy-cli

# Rust users — embed Capy as a library
cargo add --git https://github.com/olivierdevelops/capy capy-core

# MCP server for AI agents (Claude Desktop, Claude Code, Cursor, Zed)
cargo install --git https://github.com/olivierdevelops/capy capy-mcp

# Or: try it in your browser, no install — playground runs Capy compiled
# to WebAssembly with six curated samples (recipe / invite / meal plan /
# reading log / Breakout / Snake). Live editor, preview, download:
# https://olivierdevelops.github.io/capy/playground/

# macOS / Linux (binary, no Rust toolchain required)
curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh

# Homebrew
brew install olivierdevelops/tap/capy
```

Or download a binary from the [releases page](https://github.com/olivierdevelops/capy/releases).

### Embed in your Rust program

No `capy` binary required at runtime. Your program defines its own grammar inline:

```rust
use capy_core::capy::Library;

let lib = Library::new(r#"
extension html

function button
    arg literal "button"
    arg capture label string
    write `<button>${label}</button>
`
end
"#)?;

let out = lib.run(r#"button "Click me""#)?;
// → <button>"Click me"</button>
```

See the [embedding guide](docs/embedding.md) and the runnable
[`examples/embed-html-dsl/`](examples/embed-html-dsl) for full
patterns (loading from disk, multiple grammars per process, hot-swap).

---

## Quick try

```sh
git clone https://github.com/olivierdevelops/capy
cd capy
cargo build --release --manifest-path rust/Cargo.toml -p capy-cli
./rust/target/release/capy run samples/recipe-card/lib.capy samples/recipe-card/script.capy
```

---

## Documentation

| Reading order | What it covers |
|---|---|
| [docs/getting-started.md](docs/getting-started.md) | Five-minute tour |
| [docs/embedding.md](docs/embedding.md) | Embedding Capy as a Rust library in your own program |
| [docs/mcp.md](docs/mcp.md) | MCP server setup for Claude Desktop / Claude Code / Cursor / Zed |
| [docs/cookbook-ai.md](docs/cookbook-ai.md) | AI integration cookbook (10 recipes) |
| [docs/library-authoring.md](docs/library-authoring.md) | Writing your own `lib.capy` |
| [docs/capy-libraries.md](docs/capy-libraries.md) | `.capy` syntax reference (vs. YAML) |
| [docs/language-reference.md](docs/language-reference.md) | Surface grammar + lexer behavior |
| [docs/inner-dsl.md](docs/inner-dsl.md) | The inner-DSL operations |
| [docs/types.md](docs/types.md) | `base` / `pattern` / `options` |
| [docs/templates.md](docs/templates.md) | Template helpers |
| [docs/cookbook.md](docs/cookbook.md) | Recipes for common patterns |
| [docs/CAPY_FOR_LLMS.md](docs/CAPY_FOR_LLMS.md) | One-page brief for AI agents |
| [docs/roadmap.md](docs/roadmap.md) | What's planned |

Six worked examples live under [samples/](samples/) — each is a complete
library + script + expected output + README.

---

## Why Capy?

Capy is not a templating engine (it has a parser) and not a parser
generator (it has a runtime). It's something in between: **a
configurable transpiler**, with the configuration written as data.

Compared to alternatives:

| Tool                    | What it does                       | What Capy adds |
|-------------------------|------------------------------------|----------------|
| Jinja, Go templates     | Substitute values into text        | A real parser + accumulated context + types |
| ANTLR, lark, tree-sitter| Parse a language you defined       | Targeted at code generation; ships with a runtime; no Java/Python required |
| Hand-written transpilers| Full control                       | A `.capy` library replaces hundreds of lines of code per project |
| gomplate, ytt           | Powerful templating with data      | A source language with custom syntax, not just template inputs |

Use Capy when you'd otherwise hand-roll a tiny parser to drive
code-generation: configuration languages, scaffolding tools, DSLs for
domain experts, source-to-source rewrites.

---

## Status

**Pre-1.0.** The library schema may change between minor versions. See
[CHANGELOG.md](CHANGELOG.md) for what's stable; [docs/roadmap.md](docs/roadmap.md)
for what's planned.

---

## Contributing

**Contributions are closed.** No pull requests, no patches. Bug
reports via issues are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md)
for the full statement.

## License

[Capy Source-Available License](LICENSE) — © 2026 Capy authors.

Source-visible for reading, building, and personal evaluation.
Commercial use, redistribution, and derivative works require prior
written permission.
