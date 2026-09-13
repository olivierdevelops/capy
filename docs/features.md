# Feature reference

A flat list of everything Capy ships with. For deep dives follow the
links; this page is a quick reference for "is X supported?".

---

## Engine model

| Feature | Status | Where |
|---------|--------|-------|
| Zero default grammar — every shape is library-defined | ✅ | [language-reference](language-reference.md) |
| Single-binary Rust engine, no runtime deps | ✅ | `cargo install --git https://github.com/olivierdevelops/capy capy-cli` |
| Programmatic Rust API | ✅ | `Library::new(src)` → `lib.run(script)` / `lib.run_multi(script)`. See [embedding](embedding.md). |
| Library introspection (names, args, docstrings, optional/default flags) | ✅ | `lib.Introspect()` / `lib.CommentMarkers()` — see [embedding](embedding.md#introspection-the-library-describes-itself) |
| Browser playground — same engine compiled to WebAssembly | ✅ | [playground](playground.md) |
| Caret-pointed error messages with line:col + did-you-mean | ✅ | [`domain::errors::CapyError`](https://github.com/olivierdevelops/capy/blob/main/rust/src/domain/errors.rs), [errors-and-debugging](errors-and-debugging.md) |

## Library schema

| Section | Required? | Purpose |
|---------|-----------|---------|
| `extension` | optional | Suggested output file extension (informational). |
| `output_file` | optional | If set, capy writes here instead of stdout. |
| `types` | optional | Library-defined argument types with `base`, `pattern`, `options`. |
| `context` | optional | Initial schema for the accumulated context (lists/maps/scalars). |
| `functions` | required | Every recognised source-language construct. |
| `file_template` | optional | Top-level assembler block. Defaults to emitting `body` verbatim. |

## Function definitions

| Field | Purpose |
|-------|---------|
| `args` | Ordered list of `{kind: literal, value}` and `{kind: capture, name, type}` entries. |
| `template` | A literal block emitted into the body per match (no interpolation; prefer `write`). |
| `run` | Inner-DSL snippet that mutates `context`. Does NOT execute user code. |
| `block.closer` | Mode-A block opener (INDENT/DEDENT body + named closer function). |
| `block.open` + `block.close` | Mode-B block opener (explicit delimiter pair). |
| `block_close_seq` | Mode-E block opener — multi-token sequence closer (matched-pair HTML/XML). |
| `arg capture x FUNC` / `FUNC*` / `FUNC+` | Function-as-type capture (named nonterminal), optionally repeated with `sep`. |
| `priority` | Higher wins on ambiguous matches; default 0. |

### Auto-name-prepend rule

If `args:` contains zero `kind: literal` entries, the engine prepends a
literal of the function's key. Add any literal and the function name
disappears from the surface — that's how operator-style patterns
(`<var> = <value>`, `<a> -> <b>`) work. Use `bare` to opt out and keep
the function callable as plain prose.

## Authoring ergonomics

| Feature | What it gives you |
|---------|-------------------|
| `template … end` | Block sugar for a multi-line backtick `write` literal — same `${…}` interpolation, no backtick bookkeeping. |
| Optional args with `default` | A trailing capture can declare `default "value"`, making it omittable. One `button` function serves `button "Save"` and `button "Save" "danger" "submit"`. Optional args must be trailing (checked at load time). |
| Group types | `type X { group_open "[" group_close "]" }` lets a capture consume a delimited span, so `link [text](url)`, `bold **x**`, `~~strike~~` map onto one source line. See [types](types.md). |
| Multi-line backtick captures | A `` `…` `` string capture in a **user script** spans newlines — wrap a paragraph across several lines. |
| Escapable backticks | `` \` `` inside a backtick capture is a literal backtick, so a Markdown code span survives without closing the capture. |
| UTF-8 prose | Accented Latin, CJK and emoji tokenise as ordinary idents — bare prose needs no quoting. |
| `${line}` / `${col}` source mapping | Stamp the current statement's source position onto output for editor scroll-sync / inline errors. |

These ship with focused, golden-tested examples — open the
**✨ Features** category in the [playground](playground.md), or browse
the [new-feature showcase](showcase.md#new-feature-showcase-22-examples-in-the-playground).

## Built-in capture types

| Type | What it captures |
|------|------------------|
| `any` | Any single value expression (numbers, strings, lists, objects, dotted paths, paren sub-calls, comparisons). |
| `ident` | A single identifier token; bound as a string. |
| `raw` | One identifier OR string token. |
| `word` | A shell-style bare word — adjacent tokens with no source whitespace joined into one value (`--oneline`, `k8s/deploy.yaml`, `restart-api`). |
| `dotted_ident` | A dotted path `IDENT(.IDENT)*` captured as one string (`err.kind`). |
| `string` | A quoted string literal — OR a bare identifier. |
| `int` | An integer literal — OR a bare identifier. |
| `float` | A float literal — OR a bare identifier. |
| `bool` | `true`/`false` — OR a bare identifier. |
| `tail` | Every remaining token on the line, joined as one source-text string (preserves source spacing). Great for free-form trailing values. |
| `<library-type>` | Any name defined under `types:`, including [group types](types.md) (`group_open`/`group_close`) for inline syntax like `[text](url)`. |

Bare identifiers always pass primitive type checks because the target
language could resolve them as variables of any type at its own
runtime.

## Library-defined types

Declared under `types:` with three optional fields **applied in order**:

| Field | What it does |
|-------|--------------|
| `base` | Built-in kind check first (`string`/`int`/`float`/`bool`/`any`). |
| `pattern` | Regex applied to the value's string form. |
| `options` | Enum membership. |

Common patterns shipped with samples: `Email`, `Slug`, `EnvName`,
`SemVer`, `Status`, `Identifier`.

## Lexical features

| Feature | Notes |
|---------|-------|
| Identifiers | `[A-Za-z_][A-Za-z0-9_]*` |
| Numbers | Integers and floats, signed. |
| Strings | `"..."`, `'...'`, and `` `...` `` (template), all with `${expr}` interpolation |
| Multi-character punctuation | Greedy run: `==`, `!=`, `<=`, `>=`, `:=`, `->`, `=>`, `|>`, etc. each lex as one token. |
| Comments | `#` to end of line. |
| Indentation | 4 spaces or 1 tab per level. Mixed/odd indent is a parse error. |
| Multi-line literals | `{...}`, `[...]`, `(...)` allow newlines inside; value parsers skip them. |
| Object literal keys | Quoted strings OR bare identifiers (`{name: "x", "id": 1}`). |

## Inner DSL (function body)

All operations available in a function body, interleaved freely with
`write` / `template` output:

| Statement | Purpose |
|-----------|---------|
| `set <path> <value>` | Bind a field at a dotted/indexed path on `context`. |
| `append <path> <value>` | Push to a list. Creates the list if it doesn't exist. |
| `prepend <path> <value>` | Prepend to a list. |
| `merge <path> <map>` | Shallow-merge into a map. |
| `delete <path>` | Remove a field/key. |
| `if <expr>` ... `end` | Library-side conditional update. |
| `loop <var> in <expr>` ... `end` | Library-side iteration (NOT user-script iteration). |
| `regex_match value pattern` | Boolean expression value. |
| `error <message>` | Abort transpilation with a clear message. |

Paths root at `context` (or at a `loop` local). Bracket indexing
supported: `context.scripts[name]` evaluates `name` at runtime.

Expressions inside the function body support comparisons (`==`, `!=`,
`<`, `<=`, `>`, `>=`), unary `not`, lists `[...]`, objects `{...}`,
dotted paths.

## Interpolation helpers

Available inside any `write \`...\`` literal via `${expr | helper}`
or `${helper arg expr}`, in both per-function bodies and the top-level
`file_template`.

| Helper | Effect |
|--------|--------|
| `indent N` | Indent every line of a string by N spaces. |
| `decoded` | Resolve escape sequences (`\n` `\t` `\"` `` \` `` `\xNN` `\uNNNN`) in a captured string — round-trips even when the text contains bare quotes. The inverse of source-text capture. |
| `escapeHtml` | Neutralise HTML-special chars (`& < > " '`) — the safe way to interpolate user prose into HTML. |
| `lower` / `upper` | Case conversion. |
| `pascalCase` / `camelCase` / `snakeCase` | Identifier case conversion. |
| `dasherize` | Replace underscores with hyphens (for CSS, etc.). |
| `join SEP <list>` | Join a list of strings. |
| `split SEP <string>` | Split a string into a list. |
| `unquote` | Strip one layer of surrounding `"..."`, `'...'`, or `` `...` ``. |
| `unescape` | Resolve standard C-style escape sequences (legacy; prefer `decoded`). |
| `toQuoted` | Wrap a string in JSON-style double quotes. |
| `asString` | Normalise a capture to ONE valid JSON string, quoting iff not already a string — correct for both a bare ident and a quoted string. |
| `toPyLit` | Format a value as a Python literal (`True`/`False`/`None`/lists/dicts). |
| `toJSON` / `toJSONIndent` | Marshal any value to compact / pretty JSON. |
| `trimPrefix P` / `trimSuffix S` | Strip a leading / trailing substring. |
| `nonEmpty` | True when the string is non-empty (handy in `if`). |
| `add` / `sub` / `mul` | Integer arithmetic. |
| `percent N D` | Format `N/D` as a percentage. |
| `stars N` | Render `N` filled stars (★) — used by the reading-log demo. |

Control flow (`if`, `for`) lives in the function body itself —
not inside `${...}` interpolations.

### Render locals — available inside every body / `template`

| Local | What it is |
|-------|-----------|
| `${body}` | Inner block's rendered output (block-opener functions). |
| `${line}` / `${col}` | Source line / column of the current statement — stamp onto output for source↔output mapping (editor scroll-sync, inline errors). A capture of the same name wins. |
| `${depth}` / `${top_level}` | Nesting depth, and whether the statement is at the top level. |

## Block functions

Two modes; declare exactly one per opener:

### Mode A — indent + named closer

```
function if
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}
`
end
```

Body delimited by INDENT/DEDENT. Closer is itself a library function
(may have a template, or be silent).

### Mode B — explicit delimiters

```
block_open "{"
block_close "}"
```

Body delimited by the named tokens. No closer function needed.

### Mode C — verbatim (raw bytes)

```
block_verbatim end
```

The body is captured as **raw source bytes** — not re-parsed as Capy.
Blank lines and `#`-prefixed lines survive byte-for-byte. Ideal for
code blocks, inline SVG, or embedded HTML. Pair with `${escapeHtml body}`
to emit it safely. See [`samples/verbatim-pre`](showcase.md#new-feature-showcase-22-examples-in-the-playground).

### Mode D — dedent

```
block_dedent
```

Body runs until the indentation returns to the opener's level — no
explicit closer keyword.

### Mode E — multi-token sequence closer (matched-pair HTML/XML)

```
function element
    arg literal "<"
    arg capture name ident
    arg capture attrs attribute*
    arg literal ">"
    block_close_seq "</" name ">"
    write `<${name}${attrs}>${body}</${name}>`
end
```

The body closes on an exact **run of tokens**, not a single keyword.
Segments are quoted literals or bare **capture-name refs** bound by the
opener, so a capture-bound closer (`</NAME>`) lets ONE function parse any
well-formed `<tag>…</tag>` — HTML or XML. Each open tag demands its own
matching close tag, so mismatched nesting (`<div><p></div>`) is a hard
parse error. See [`samples/html-xml-parser`](showcase.md) and
[block-functions](block-functions.md#mode-c-multi-token-sequence-closer-block_close_seq).

Nesting works freely across all modes, including mixed (Mode-A inside
Mode-B and vice versa).

### Function-as-type captures (named nonterminals)

A capture's type may name **another library function** instead of a
built-in type. The capture then matches that function's shape and renders
its template. Add a quantifier to repeat it, plus two **independent**
separators: `sep "X"` (consumed between repetitions on INPUT while
parsing) and `join "Y"` (inserted between rendered sub-results on
OUTPUT):

| Form | Meaning |
|------|---------|
| `arg capture x SomeFunc` | exactly one `SomeFunc` (mandatory) |
| `arg capture xs SomeFunc*` | zero or more |
| `arg capture xs SomeFunc+` | one or more |
| `arg capture xs SomeFunc+ sep ","` | one or more, comma-separated input |
| `arg capture xs SomeFunc* sep "," join ", "` | comma in, `", "` out |

A pure-capture nonterminal (no `arg literal`) must be declared `bare`
to opt out of the auto-prepended name keyword. See
[block-functions](block-functions.md#function-as-type-captures-named-nonterminals).

## CLI

| Subcommand | Effect |
|------------|--------|
| `capy run <lib.capy> <script.capy>` | Transpile a script. Output to stdout unless `--out` or library `output_file`. |
| `capy check <lib.capy>` | Validate a library; report functions and types. Exit 0 if valid. |
| `capy docs <lib.capy>` | Render a Markdown reference doc from the library's `description` annotations. |
| `capy build <lib.capy> [-o <out>]` | Compile a library into a standalone single-purpose CLI binary. See [compiling-libraries](compiling-libraries.md). |
| `capy lib new <name>` | Scaffold a new library. |
| `capy lib list` / `which` / `path` | Manage installed libraries on the `CAPY_LIBS` search path. See [library-commands](library-commands.md). |
| `capy new <dir> --using <library>` | Scaffold a new project from a library. |
| `capy <library> <command> [args]` | Dispatch a library-defined command (`command "run" … end`). |
| `capy version` | Print build version. |
| `capy help [<command>]` | Inline help. |

Flags for `run`:

| Flag | Effect |
|------|--------|
| `--out <path>` | Override `output_file`. |
| `--no-color` | Disable ANSI escape codes (reserved). |
| `--debug` | Verbose engine tracing (reserved). |
| `-lib <path>` | Legacy library path (use the positional form instead). |

## Output assembly

Each library function's body `write`s text into a per-block **body**
string. Block functions reference `${body}` (or `${indent N body}`)
to get the rendered output of their children. The top-level
program's body, plus the accumulated `context`, are passed to
`file_template` for the final output.

Inside any function body / `write` literal:

| Reference | What it is |
|-----------|------------|
| `${<capture>}` | Source-text form of a capture (with quotes for string literals). |
| `${body}` | Inner block's rendered output (only inside block-opener functions). |
| `${context.X}` | Read-only snapshot of the accumulated context at this point. |
| `${func arg arg}` | Call a helper inline (`indent`, `pascalCase`, `toQuoted`, …). |

Inside `file_template`:

| Reference | What it is |
|-----------|------------|
| `${body}` / `write body` | Concatenation of all top-level statements' rendered output. |
| `${context.X}` | Final accumulated context. |

## Source vs evaluated captures

A capture has two faces:

- **Inside `write \`...\`` interpolation** — captures resolve to
  **source text** (with quotes for string literals). So `if x > 0`
  exposes `cond` as the literal text `x > 0` and a Python emitter
  can write `if ${cond}:` unchanged.
- **In `set` / `append` / `if` expressions** — captures resolve to
  **evaluated values**. Strings lose their quotes,
  numbers become int64/float64, lists become `[]any`, objects become
  `map[string]any`. Unresolved bare identifiers fall back to their
  literal name.

This dual model means one capture serves both render-by-text (templates)
and structured accumulation (context) without any explicit conversion.

## AI / agent ecosystem

| Asset | Purpose |
|-------|---------|
| [Claude Code skill](https://github.com/olivierdevelops/capy/tree/main/skills/capy-author) | Full skill with `SKILL.md` + instructions + 5 reference docs |
| [Slash commands](https://github.com/olivierdevelops/capy/tree/main/commands/capy) | `/capy-new`, `/capy-add-function`, `/capy-add-type`, `/capy-explain`, `/capy-debug` |
| [`CAPY_FOR_LLMS.md`](CAPY_FOR_LLMS.md) | Single-page brief paste-able into any model |
| [Cursor rule](https://github.com/olivierdevelops/capy/tree/main/editors/cursor) | Drop-in `.cursor/rules/capy.md` |
| [Continue config](https://github.com/olivierdevelops/capy/tree/main/editors/continue) | Adds LLM brief to context |
| [Aider read](https://github.com/olivierdevelops/capy/tree/main/editors/aider) | `aider --read docs/CAPY_FOR_LLMS.md` |
| [Agent system prompt](https://github.com/olivierdevelops/capy/blob/main/agents/capy-system-prompt.md) | Drop-in for any tool |

See [Capy for AI agents](ai-agents.md) for token-savings math and
sandboxing patterns.

## Editor support

| Editor | Where |
|--------|-------|
| VS Code | [`editors/vscode/capy/`](https://github.com/olivierdevelops/capy/tree/main/editors/vscode/capy) — syntax highlighting for `.capy` source and libraries |

## Distribution

| Method | Command |
|--------|---------|
| Rust | `cargo install --git https://github.com/olivierdevelops/capy capy-cli` |
| Binary release | [Releases page](https://github.com/olivierdevelops/capy/releases) |
| Install script | `curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh \| sh` |
| Docker | `docker build -t capy .` (Dockerfile in repo root) |
| Homebrew | Coming when the `olivierdevelops/homebrew-tap` repo exists |

## Pre-1.0 status

The library schema may evolve between minor versions before 1.0.
Each breaking change appears in [CHANGELOG](https://github.com/olivierdevelops/capy/blob/main/CHANGELOG.md)
with a migration note. The engine itself is stable enough to use in
production for code generation; just pin a specific version.

Planned features: see [roadmap](roadmap.md).
