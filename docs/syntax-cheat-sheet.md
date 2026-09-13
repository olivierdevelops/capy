# Syntax cheat sheet

One page. Every `.capy` **library** directive, every capture and block mode,
the inner-DSL statements, and the built-in template helpers — in lookup form.
For prose explanations follow the links into the reference docs.

> A `.capy` library file *is* the grammar. The engine ships **zero** built-in
> keywords; everything below is a directive the engine's library parser
> understands (`rust/src/infra/capy_lib_parser.rs`), not a keyword in your source
> language.

---

## Library-level directives

Top-level directives in a `.capy` library file:

| Directive | Form | Purpose |
|-----------|------|---------|
| `name` | `name "my-lib"` | Library identity (used by `capy lib`, `import`). |
| `version` | `version "1.0.0"` | Library version string. |
| `description` | `description "…"` | Shown by `capy docs`. |
| `extension` | `extension html` | Default source file extension this library binds to. |
| `output_file` | `output_file index.html` | Default single-file output name. |
| `function` | `function NAME … end` | Declare a source construct. See below. |
| `type` | `type NAME … end` | Declare a capture type / validator. See below. |
| `context` | `context … end` | Initial mutable state (scalars, lists, objects). |
| `file_template` | `file_template … end` | Wrapper rendered around the whole body; `write body` emits the accumulated output. |
| `file` | `file "path": … end` | Declare an additional output file (path may interpolate `${…}`). See [multi-file](multi-file-and-imports.md). |
| `command` | `command "name" … end` | A CLI subcommand (`capy <lib> name …`). See [library-commands](library-commands.md). |
| `impl` / `default_impl` | `impl NAME file "other.capy"` | Alternate backend selectable with `--impl`. |
| `import` | `import "other.capy"` | Merge another library (functions, types, context). |
| `preprocess` | `preprocess … end` | Opt-in file-inclusion directives (e.g. `include "@import"`). |
| `comments` | `comments … end` | Declare line-comment markers for the *source* language (default: none). |

## `function` body directives

```capy
function button
    arg literal "button"
    arg capture label string
    arg capture color string default "#007bff"
    priority 10
    block_closer end_button
    template
        <button style="background:${color}">${label}
    end
end
```

| Directive | Purpose |
|-----------|---------|
| `arg literal "x"` | Match an exact token in source. |
| `arg capture NAME TYPE [default "…"]` | Bind a value of `TYPE` to `NAME`. |
| `arg capture NAME TYPE optional` / `required` / `bare` | Capture modifiers (optional value, required, bare/unquoted). |
| `priority N` | Higher wins when multiple functions could match (default 0). |
| `` write `…` `` / `template … end` | Emit output. Backtick strings are multi-line; `${expr}` interpolates. `template … end` is block sugar for the same thing. |
| `set` / `append` / `prepend` / `merge` / `delete` | Mutate `context` when this function matches. Written directly in the body, interleaved with `write`. |
| `block_closer NAME` | Body is INDENT/DEDENT-delimited; `NAME` runs after the body. |
| `block_open "{" … block_close "}"` | Body delimited by literal tokens. |
| `block_close_seq "</" cap ">"` | Matched-pair closer (HTML tags); closer depends on a capture. |
| `block_dedent` | Dedent-only body (no closer function). |
| `block_verbatim NAME` | Capture the body as raw bytes (no nested parsing). |
| `block_sections [a, b]` | Multi-section block (`try … rescue … finally … end`); each section renders as `${a}`, `${b}`. |
| `when_followed_by TOKEN` / `when_not_followed_by TOKEN` | Lookahead so one keyword can mean two things (e.g. flat vs. block form). |

### Capture types (built-in)

`string` · `int` · `float` · `bool` · `ident` · `raw` · `expr` — plus any
custom `type` you declare. See [types.md](types.md).

## `type` body directives

```capy
type color
    base string
    pattern "^#[0-9a-fA-F]{6}$"
    description "6-digit hex colour"
end
```

| Directive | Purpose |
|-----------|---------|
| `base TYPE` | Underlying built-in type. |
| `pattern "regex"` | Validation regex. |
| `options [a, b, c]` | Enum of allowed values. |
| `group_open "x"` / `group_close "y"` | Inline-syntax grouping delimiters. |

---

## Block modes at a glance

| Mode | Declared with | Body delimited by | Closer |
|------|---------------|-------------------|--------|
| Indent + named closer | `block_closer end_x` | INDENT / DEDENT | runs `end_x` |
| Delimiter | `block_open "{"` + `block_close "}"` | literal tokens | none |
| Sequence-close | `block_close_seq "</" tag ">"` | matched pair | capture-bound |
| Dedent-only | `block_dedent` | INDENT / DEDENT | none |
| Verbatim | `block_verbatim end_x` | INDENT / DEDENT | raw, unparsed |
| Multi-section | `block_sections [...]` | INDENT / DEDENT | per-section |

Full detail: [block-functions.md](block-functions.md).

---

## Inner DSL (`run:` blocks, command bodies, type validators)

Statements (`rust/src/orchestrator/features/inner_parser.rs`):

| Statement | Example | Effect |
|-----------|---------|--------|
| `set` | `set context.count 0` | Assign a value. |
| `let` | `let total = sum context.items` | Bind a local. |
| `append` | `append context.items value` | Push onto a list. |
| `prepend` | `prepend context.items value` | Unshift onto a list. |
| `merge` | `merge context.meta {"k": "v"}` | Shallow-merge into an object. |
| `delete` | `delete context.temp` | Remove a key. |
| `if … else … end` | `if x == 1` … `else` … `end` | Conditional (single `else`). |
| `loop` / `for` | `for item in context.items` … `end` | Iterate. |
| `write` | `write context.accumulated` | Emit text into the template stream. |

**Expressions:** literals, variable refs (`context.field`, `items[0]`),
comparisons (`== != < > <= >= !`), function calls (`upper name`), lists `[…]`,
objects `{…}`. See [inner-dsl.md](inner-dsl.md).

### Host capabilities (read-only anywhere; write only in `command` bodies)

Read-only: `env "NAME"` · `arg N` · `read_file "path"` · `os` · `arch` · `cwd`.
Command-only (side-effecting): `write_file` · `exec` · `exec_capture` ·
`mktemp` · `mktemp_dir` · `cd` · `print` · `compile`. See
[host-capabilities.md](host-capabilities.md).

---

## Built-in template helpers

Callable inside `${…}` interpolation. The authoritative list lives in
`rust/src/infra/helpers.rs`; keep it and [function-cookbook.md](function-cookbook.md)
in sync (see `AGENTS.md`).

**Case / strings:** `upper` · `lower` · `camelCase` · `pascalCase` ·
`snakeCase` · `dasherize` · `trimPrefix` · `trimSuffix` · `split` · `join`

**Quoting / encoding:** `toQuoted` · `unquote` · `escapeHtml` · `unescape` ·
`decoded` · `toJSON` · `toJSONIndent` · `toPyLit` · `asString`

**Numbers / math:** `add` · `sub` · `mul` · `div` · `mod` · `percent`

**Formatting / layout:** `indent` · `align` · `stars` · `nonEmpty`

> Tip: `capy docs <lib>` auto-generates a Markdown reference from your
> function signatures and `description` annotations.

---

## CLI quick reference

| Command | Purpose |
|---------|---------|
| `capy run <lib> <script> [args]` | Transpile a script. Flags: `--out`, `--out-dir`, `--zip`, `--debug`. |
| `capy <lib> <command> [args]` | Run a library-defined command. |
| `capy check <lib>` | Validate a library; list functions/types. |
| `capy docs <lib>` | Render auto-generated reference. |
| `capy watch <lib> [args]` | Re-run on file change. |
| `capy fmt <files…>` | Format `.capy` files (`--check`, `--diff`, `--stdout`). |
| `capy build <lib> [-o out]` | Compile to a standalone binary. |
| `capy lib list \| which \| add \| remove \| path` | Manage installed libraries on `CAPY_LIBS`. |
| `capy new <dir> --using <lib>` | Scaffold a project. |

Full flags and behaviour: [cli.md](cli.md).
