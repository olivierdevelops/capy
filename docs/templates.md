# Templates

Capy emits text via `write` calls inside function bodies and
`file_template`. The string body of a `write` call is a backtick
literal with `${EXPR}` interpolation — that's the primary surface.

The renderer walks the parsed template AST directly — there is no
intermediate template language. The helpers below are called Capy-style via
`${func arg arg}`.

## Per-function values in scope

Inside a function body, these are visible to `${EXPR}` interpolation:

| Reference          | Source                                                          |
|--------------------|-----------------------------------------------------------------|
| `${<capture>}`     | One entry per capture; the captured source text as a string.    |
| `${body}`          | The inner block's rendered output (block functions only).       |
| `${context.X}`     | Read-only snapshot of the current accumulated context.          |
| `${top_level}` / `${depth}` | Whether this call renders at file scope; its AST depth. |
| `${line}` / `${col}` | The 1-indexed source position of the statement. Stamp `data-capy-line="${line}"` for editor source↔output mapping. |
| `${func arg arg}`  | Call a helper inline (see Helpers below).                       |

```
function greet
    arg capture name any
    write `Hello, ${name}!
`
end
```

`${name}` is the source text of the captured value (`"Alice"` with
quotes, or `Alice` if the user passed a bare identifier).

## file_template values in scope

In `file_template`:

| Reference        | Source                                                              |
|------------------|---------------------------------------------------------------------|
| `${body}`        | Concatenation of all top-level statements' written output.          |
| `${context.X}`   | The final accumulated context.                                      |

```
file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

## Helpers

> For the **complete** list of every built-in helper — all 26, each with
> a worked example — see the [Built-in function cookbook](function-cookbook.md).
> The highlights below cover the most common ones.

Capy provides these helpers:

### `indent N`

Indents every line of a string by N spaces. Most useful for block bodies.

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

### `toQuoted`

Wraps a string in JSON-style double quotes (with proper escaping).
Useful for emitting string literals in target languages.

```
function say
    arg capture msg any
    write `print(${toQuoted msg})
`
end
```

### `toPyLit`

Formats a value as a Python literal (`True`/`False`, `None`,
quoted strings, list/dict syntax). Useful when accumulated `context`
carries real values you want to splat into Python.

```
file_template
    write `CONFIG = ${toPyLit context.config}
`
end
```

### `toJSON` / `toJSONIndent`

Marshal any value to JSON. Compact and pretty respectively.
Excellent for config-file targets.

```
file_template
    write (toJSONIndent context)
end
```

### `asString <capture>`

Normalises a capture to exactly **one valid JSON string**, quoting *iff*
it isn't already a string. This is the verb to reach for when a single
capture might hold either a bare token (`foo`, `42`, `true`) or a quoted
string (`"foo"`) and you want correct JSON for both:

- `${x}` alone emits the source text — a bare ident `foo` is invalid JSON.
- `${toJSON x}` re-quotes an already-quoted string, leaking literal `"`.
- `${asString x}` resolves the captured text to its user-intended form
  (peeling one quote layer and decoding escapes, like `decoded`) and
  re-encodes it as a single JSON string.

```
function exec
    arg literal "exec"
    arg capture bin word
    write `{"run": ${asString bin}}
`
end
```

`exec git` and `exec "git"` both emit `{"run": "git"}`. Pairs naturally
with the `word` / `tail` capture types for shell-like surfaces.

### `lower` / `upper`

Case helpers.

### `join SEP <list>`

Join a list of strings (or any-types coerced to strings).

```
file_template
    write `scripts: ${join ", " context.script_names}
`
end
```

### `escapeHtml <string>`

HTML-entity-escape the five characters every HTML emitter has to
neutralise: `&`, `<`, `>`, `"`, `'`. Wrap any free-form capture that
flows into HTML so the output stays safe even if the capture
contains markup. The verbose name is deliberate: `${html x}` would
read as "make this HTML"; `${escapeHtml x}` makes the intent
("escape this FOR HTML") obvious at the call site.

```
function p
    arg literal "p"
    arg capture text string
    write `<p>${escapeHtml text}</p>
`
end
```

Source `p "Look at <script>"` then emits `<p>Look at &lt;script&gt;</p>`
— no markup injection. Composes with `unquote` / `decoded` for
captures that need their quotes stripped first:
`${escapeHtml (decoded text)}`.

### `decoded <string>`

Strip outer quotes (if present) AND fully resolve C-style escape
sequences (`\"` → `"`, `\n` → newline, `\t` → tab, `\\` → `\`,
plus `\xNN` / `\uNNNN`). Use this when a `string`-typed capture
contains escaped quotes or whitespace that the user wrote
literally — `p "He said \"hi\""` becomes `He said "hi"`, not
`He said \"hi\"` like `${unquote x}` would produce.

```
function p
    arg literal "p"
    arg capture text string
    write `<p>${escapeHtml (decoded text)}</p>
`
end
```

Existing libraries that want the source-text quoted form (YAML
output, TypeScript string literals, markdown frontmatter) keep
using `${x}` directly — `decoded` is opt-in.

## Common patterns

### Imports at top, body below

```
file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

### A block function emitting an indented body

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

### Pure-context output (no body emission)

```
function set_name
    arg capture n any
    set context.name n
end

file_template
    write (toJSONIndent context)
end
```

### Mixing output and state in one function

```
function section
    arg literal "section"
    arg capture title string
    block_closer end
    # Record this section in the TOC AND emit its heading.
    append context.toc title
    write `## ${title}

${body}
`
end
```

## Whitespace

There is no `{{- -}}` trimming sigil in the unified shape. You
control whitespace by where `write` is called and what bytes are
inside the backtick literal. To avoid blank lines between iterations,
make sure each `write` ends with the exact newline you want and no
more:

```
for imp in context.imports
    write `import ${imp}
`
end
```
