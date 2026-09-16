# Inner DSL Reference

The inner DSL is the language inside a library function's body — the
sequence of statements that runs every time the function matches a
source statement. It combines two concerns into one block:

- **Output** via `write \`...\``, which appends a (possibly
  interpolated) string to the function's body contribution.
- **State** via `set` / `append` / `prepend` / `merge` / `delete`,
  which mutate the accumulated `context` map.

Plus control flow (`if` / `else` / `for` / `loop`) and primitive
host calls (`env`, `arg`, `read_file`, `os`, `arch`, …). It **does
not execute user-script code** — library authors compose primitives
to describe what each match contributes to the final output.

> Removed form: older libraries wrapped these statements in a `run:`
> block, separate from the output template. That shape no longer parses —
> `capy check` reports `unexpected token ":" in value`. Inner-DSL
> statements now sit directly in the function body, interleaved with
> `write` calls.

> `${line}` and `${col}` are the statement's **start** position and are
> unchanged. They are the same position as the AST node's `span.start`; the span
> additionally carries where the statement *ends*. See
> [embedding](embedding.md#source-positions).

## Operators

Value expressions support infix `*` `/` `%` `+` `-`, the comparisons, and
`and` / `or`, with conventional precedence and left associativity. See
[the precedence table](language-reference.md#operator-precedence).

```
if count * 2 > limit
    set context.over true
end
```

`and` and `or` short-circuit: the right operand is evaluated only when it can
change the answer.

Integer arithmetic stays integer (`1 + 1` is `2`, not `2.0`); a float operand
promotes the result, and `7 / 2` is `3.5` rather than truncating. `+` on two
strings concatenates. Anything else is an error rather than a silent coercion.

## Tokens & expressions

The inner DSL reuses the engine's lexer and value-expression parser. Values
may be:

- Numbers, strings, templates, `true`, `false`, `null`.
- Bare or dotted identifier paths — these resolve, in order, against:
  inner-DSL locals (loop variables), captures from the matched function,
  the root `context` map.
- **Indexed reads:** a path may carry `[<expr>]` index steps —
  `context.buf[i]`, `context.known[name]`, `context.grid[i][j]`,
  `context.rows[i].name`, `context.buf[(sub n 1)]`, `context.buf[-1]`.
  A **map** parent is keyed by the index's string form; a **list**
  parent is keyed by an integer (negative counts from the end). This is
  the read-side twin of the `set context.buf[i] …` write target, so a
  value written by index reads back by the same index. A missing key or
  out-of-range index is `nil` (falsy) in value position and renders empty
  inside a `${…}` template — so `if context.seen[k]` guards cleanly
  without a `for`-scan. See
  [`samples/value-index-read/`](https://github.com/olivierdevelops/capy/tree/main/samples/value-index-read).
- Lists `[...]` and objects `{...}`.
- Comparison expressions: `a == b`, `a != b`, `a < b`, `a <= b`, `a > b`,
  `a >= b`.
- Unary `not expr`.
- Parenthesized sub-calls: `(regex_match name "^[a-z]+$")`.

## Statements

### `write <expr>`

Appends EXPR (coerced to string) to the function's output buffer.
EXPR is most commonly a backtick literal with `${EXPR}` interpolations:

```
write `Hello, ${name}!
`
write `${indent 4 body}`
write body
```

Backtick literals are multi-line; the bytes inside (including
newlines, tabs, leading whitespace) are emitted verbatim. `${EXPR}`
holes accept any value expression: paths (`name`, `context.foo`,
`body`), helper calls (`indent 4 body`, `pascalCase name`), or
literals.

`write` has no effect on context — pair it with `set`/`append`/etc.
in the same function body when both output and state mutation are
needed.

### `set <path> <value>`

Assigns a value to a field path on the context (or to a local).

```
set context.name "Alice"
set context.config.api.url "https://example.com"
set context.scripts[key] cmd
set context.buf[i] "    ; (rewritten in place)"   # list element by index
```

Paths use:
- `.<field>` for map field access.
- `[<expr>]` for dynamic indexing. When the parent is a **map**, the
  expression's string form becomes the key. When the parent is a
  **list**, the expression must evaluate to an integer index and the
  element is overwritten **in place** (negative indices count from the
  end, so `-1` is the last element). Out-of-range indices error.

Overwriting a list element by index is what enables *retroactive*
rewrites of buffered output — e.g. an optimizer that buffers its
instruction stream in `context.buf` and later nulls out a dead store, or
back-patches a jump offset once the target is known. See
[`samples/list-index-assign/`](https://github.com/olivierdevelops/capy/tree/main/samples/list-index-assign)
for a dead-store eliminator built this way.

### `append <list-path> <value>`

Appends to a list field. Creates the list if it doesn't exist yet. With
an index target (`append context.rows[i] value`) it appends to the
*nested* list stored at element `i`.

```
append context.imports name
append context.errors {kind: "warn", msg: msg}
```

### `prepend <list-path> <value>`

Like `append` but inserts at the front (also works on an indexed nested
list, `prepend context.rows[i] value`).

```
prepend context.docstrings line
```

### `merge <map-path> <map-value>`

Shallow-merges a map into a map field. The value must be a map expression.

```
merge context.headers {"X-Built-With": "capy"}
```

### `delete <path>`

Removes a field or list index.

```
delete context.scripts[old_name]
```

### `if <expr>` … (`else` …) `end`

Library-side conditional. The expression is evaluated; if truthy,
the body runs. An optional `else` arm handles the falsy case.
`else if cond` chains naturally.

```
if (regex_match name "^_")
    set context.private true
end

if optional
    write `${name}?: any;
`
else
    write `${name}: any;
`
end
```

### `for <var> in <expr>` … `end`  (alias: `loop`)

Iterates a list, binding the variable in each iteration's local
scope. `for` and `loop` are synonyms.

```
for tag in tags
    append context.tags tag
end

for imp in context.imports
    write `import ${imp}
`
end
```

Note: this iterates within a single function body. It does NOT
iterate user-script code — that's what `block:` functions are for.

### Plain calls

A line that starts with an identifier that isn't a statement keyword is
treated as a primitive call:

```
error "expected a positive integer"
```

The only primitive call defined is `error <message>` (abort transpilation).
`regex_match` is also callable but is most useful in expression position
(`(regex_match v "...")`).

## Truthiness

| Value | Truthy? |
|---|---|
| `nil` / `null` | no |
| `false` | no |
| `""` (empty string) | no |
| `0`, `0.0` | no |
| empty list `[]` | no |
| empty map `{}` | no |
| anything else | yes |

## Captures inside the function body

When you reference a capture by name, you get the **evaluated** value:

- A string literal `"foo"` becomes the string `foo` (no surrounding quotes).
- A number becomes `int64` or `float64`.
- A list becomes `[]any` of evaluated items.
- An object becomes `map[string]any`.
- A bare identifier (when the source has e.g. `import json`) becomes its
  literal name as a string (`"json"`).

This means `append context.imports name` correctly stores `"json"` without
extra quoting — useful for the file template's `for ... in ... end` loop.

## Engine-injected locals

In addition to your function's captures, these locals are always
available inside the function body:

| Local | Available where | What it is |
|---|---|---|
| `body` | `write` literals AND state-mutation statements | The rendered output of the function's inner block (block functions only). In a state mutation like `append context.styles {name: name, body: body}`, this lets you stash rendered text back into context. |
| `top_level` | `write` literals AND state-mutation statements | Boolean. `true` when the function call is being rendered at the file's outermost program block; `false` once we're inside any block's body. |
| `depth` | `write` literals AND state-mutation statements | Integer. `0` at the top level, `1` inside one nested block, `2` inside two, etc. |
| `line` | `write` literals AND state-mutation statements | Integer. The 1-indexed source line of the statement's first token. Stamp it onto emitted output for source↔output mapping: `<p data-capy-line="${line}">…</p>`. |
| `col` | `write` literals AND state-mutation statements | Integer. The 1-indexed source column of the statement's first token. |

`top_level` is the convenience boolean — most uses only need
`if top_level … else … end` to branch between "this call appears at
file scope" and "this call appears inside a block body". Concrete
win: a single `NAME = VALUE` syntax can be a *declaration* at file
scope (wrap in `<script>` for an HTML target, emit a `var` line for
a JS target, etc.) and a *bare reassignment* inside a handler body,
with no extra keyword required from the user.

`line` / `col` give a host editor source↔output mapping for free: a
library that writes `data-capy-line="${line}"` lets the editor do
`querySelector('[data-capy-line="N"]')` for scroll-sync or inline
error underlines — no source mutation, works inside every region.

If a user-defined capture happens to be named `body`, `top_level`,
`depth`, `line`, or `col`, the capture **wins**: the engine local
only gets injected when there is no capture of the same name.

## Context paths

Paths must be rooted at `context` (or at a local introduced by a `loop`).
You cannot directly mutate captures.

```
set context.imports.json true             # ok
set context.imports[name] true            # ok — `name` is a capture
set name "json"                            # ERROR — captures are read-only
```

## Putting it together

```
# transpile-py-style import handling
function import
    arg literal "import"
    arg capture name ident
    if (regex_match name "^[a-z][a-z_]*$")
        append context.imports name
    else
        append context.rejected name
    end
end
```

To *reject* invalid input rather than route it, declare a `type` with a
`pattern` and capture against it — validation is the type system's job, and it
produces a proper `line:col` error. There is no `error` statement in the inner
DSL.

## What's NOT here (and why)

The inner DSL is intentionally small. It does not have:

- User-defined inner functions. Compose with multiple library functions or with `loop`.
- An `error` statement. Reject invalid input with a `type` `pattern` / `options` instead.
- Arithmetic operators (`+`, `-`, …). Compute at template time with helpers, or accumulate into a count.

These omissions keep the runtime tiny and predictable. If you find yourself
wanting them often, open a [feature request](https://github.com/olivierdevelops/capy/issues).
