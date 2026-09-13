# Editor & tooling integration

Capy ships editor assets and a self-describing introspection API so tools can
support **your** library's source language without hand-maintaining a parallel
grammar. This page covers what's in the box and how to build on it.

## VS Code extension

A bundled extension lives at `editors/vscode/capy/`:

- `package.json` — extension manifest (language id, file associations).
- `language-configuration.json` — brackets, comments, auto-closing pairs.
- `syntaxes/capy.tmLanguage.json` — TextMate grammar for `.capy` library files.
- `README.md` — install notes.

It gives you syntax highlighting and basic editing affordances for editing
`.capy` **library** files. To try it locally, open `editors/vscode/capy/` in VS
Code and run the extension (or symlink it into `~/.vscode/extensions/`).

> Scope: the bundled grammar highlights the `.capy` *library* format (directives
> like `function`, `arg`, `template:`). Highlighting a *source* file in the
> language a library defines is a per-library concern — derive it from
> introspection (below) rather than hand-writing a grammar per library.

## AI-assistant rules

Drop-in configuration for popular AI coding tools, each a short brief that
teaches the assistant Capy's schema and pitfalls:

| Tool | File |
|------|------|
| Cursor | `editors/cursor/capy.md` |
| Continue | `editors/continue/capy.md` |
| Aider | `editors/aider/capy-aider.md` |

For the canonical one-page brief, also see [CAPY_FOR_LLMS.md](CAPY_FOR_LLMS.md)
and the [MCP server](mcp.md), which lets an agent run and validate Capy directly.

## The introspection API (build your own tooling)

A compiled library can describe itself. From Rust (see [embedding.md](embedding.md)):

```rust
let lib = Library::from_file("my-lib.capy")?;
for f in lib.introspect() {
    println!("{} (priority {}) — {}", f.name, f.priority, f.description);
    for a in &f.args {
        println!("  {} {}:{}", a.kind, a.name, a.type_);
    }
    if !f.block.is_empty() {
        println!("  block: {}", f.block);
    }
}
let markers = lib.comment_markers(); // e.g. ["#", "//"] or empty
```

`Introspect()` returns `[]FunctionInfo`, sorted by name, straight from the
compiled library — so it never drifts from the real grammar:

| Field | Meaning |
|-------|---------|
| `Name`, `Description`, `Priority` | Function identity and match precedence. |
| `Args[]` | Each arg's `Kind` (`literal`/`capture`), `Value`/`Name`, `Type`, `Optional`, `Default`, `Description`. |
| `Block` | Encoded block mode: `closer:end_x`, `dedent`, `open:{ close:}`, `verbatim:end_x`, or `sections:a,b closer:end_x`. |

`CommentMarkers()` returns the library's declared line-comment markers (empty if
the library declares none).

### What you can derive from it

- **Autocomplete** — suggest function literals and their argument shapes.
- **Hover docs** — show `Description` + the arg list for the token under the cursor.
- **Syntax highlighting** — colour known literals; use `CommentMarkers()` instead
  of hardcoding `#`.
- **Diagnostics** — pair introspection with `capy check` to flag unknown
  constructs as you type.
- **An LSP server** — the planned [roadmap](roadmap.md) item; everything an LSP
  needs (symbols, signatures, comment syntax) is already exposed here.

## MCP for agents

The MCP server (`cmd/capy-mcp`, [docs](mcp.md)) exposes three tools over
JSON-RPC stdio so an AI agent can use Capy as a tool:

- `capy_run` — transpile a script through an inline library.
- `capy_run_file` — transpile using a library file on disk.
- `capy_check` — validate a library and return its function/type catalogue
  (call this before `capy_run` to give the user precise errors).

## CLI helpers for tooling

- `capy docs <lib>` — auto-generated Markdown reference (great for a docs site
  or a hover-card source).
- `capy check <lib>` — machine-checkable validation for editor diagnostics / CI.
- `capy fmt` — formatting, scriptable with `--check` and `--stdout`.
