# Capy for LLMs — single-page brief

Paste this into a model's context window when you want it to author a
Capy library. Covers the `.capy` schema, the inner DSL, and common
pitfalls. About 500 lines of prose; designed to be self-contained.

---

## What Capy is

A transpiler engine. You define a source-language grammar + transformation
in a **`.capy` library file**. Capy reads source code, matches each
statement against the library's function shapes, and for each match runs
the function's body — a sequence of inner-DSL statements that may
emit output (`write \`...\``) and/or mutate an accumulated `context`
(`set` / `append` / …). A top-level `file_template` block assembles
`body` + `context` into the final output.

There are NO built-in user-facing keywords. Every shape is
library-defined.

**Format note:** Capy libraries are `.capy` files. The previous YAML
library format has been removed.

---

## The library schema

```
extension <str>              # informational; suggests output file extension
output_file <str>            # optional; write output here instead of stdout

context                      # initial accumulated state
    <name> []                # empty list
    <name> {}                # empty map
    <name> 0                 # numeric default
    <name> "default"         # string default
end

type <TypeName>              # library-defined argument type. EITHER
                             # a constraint type (base/pattern/options)
                             # OR a group type (group_open/close) —
                             # never both.
    base <kind>              # optional: any|string|int|float|bool
    pattern "<regex>"        # optional: regex on the value's string form
    options "v1" "v2" "v3"   # optional: enum membership
    group_open  "X"          # delimited capture: walk tokens between
    group_close "Y"          # `X` and `Y` (balanced nesting, multi-
                             # line OK) and return the joined source
                             # text. Use for Markdown-style inline
                             # syntax like `[label](url)` or `**bold**`.
end

function <NAME>              # one DSL statement shape
    priority <int>           # optional; higher wins ambiguous matches
    bare                     # optional; opt out of auto-name-prepend so
                             # captures-only functions match without a
                             # leading keyword (e.g. a bare `"1" "2" "3"`
                             # row of three string captures)
    arg literal "TEXT"       # match a literal token
    arg capture <NAME> <TYPE> # capture a typed named variable
    arg capture <NAME> <TYPE> default "V"  # optional trailing arg;
                             # binds "V" when omitted. Must be trailing.
    arg capture <NAME> <FUNC>   # function-as-type: <TYPE> names another
                             # library function (named nonterminal); the
                             # capture matches that function's shape and
                             # renders its template.
    arg capture <NAME> <FUNC>*  # repeated nonterminal: * (zero+) / + (one+)
    arg capture <NAME> <FUNC>+ sep "," # repeated, "," consumed between
                             # repetitions on INPUT while parsing
    arg capture <NAME> <FUNC>* sep "," join ", " # sep=input separator,
                             # join=OUTPUT separator inserted between
                             # rendered sub-results (independent of sep)
    block_closer <NAME>      # block opener: body runs until <NAME> appears
    block_open "OPEN" close "CLOSE"   # alternative: explicit delimiters
    block_dedent             # alternative: body ends at first DEDENT,
                             # no closer keyword (CSS-style selectors,
                             # YAML-style sections)
    block_verbatim <NAME>    # alternative: body captured as raw source
                             # bytes (no nested parsing) until <NAME>.
                             # For code blocks, embedded HTML/SVG.
    block_sections rescue finally closer end  # alternative: multi-section
                             # block. Main body + each section render to
                             # ${body} / ${rescue} / ${finally}. (try/rescue)
    block_close_seq "</" name ">"  # alternative: multi-token sequence
                             # closer. Segments are quoted literals or bare
                             # capture-name refs bound by the opener. Body
                             # is a free-flowing statement sequence ended by
                             # the exact token run. Enables matched-pair HTML
                             # (<div>…</div>); mismatched nesting is an error.
    when_followed_by indent      # gate: match only if an indented block follows
    when_not_followed_by indent  # gate: match only if one does NOT follow
                             # (context-sensitive keyword reuse — a flat and a
                             # block function can share one leading keyword)

    # Function body — sequence of inner-DSL statements:
    write `Hello, ${name}!\n`        # emit literal text + interpolations
    template                         # sugar for multi-line backtick `write`:
        <div>${name}</div>           #   body captured verbatim, dedented,
    end                              #   ${…} interpolation active.
    append context.greetings name    # mutate state
    # if / for / set / prepend / merge / delete also available
    # render-time locals always available inside the body:
    #   `body`      — rendered inner-block output (block functions)
    #   `top_level` — true when this call is at the file root
    #   `depth`     — integer AST depth (0 at root)
    #   `line` / `col` — 1-indexed source position of this statement
    #                    (e.g. data-capy-line="${line}" for editors)
end

file_template                # whole-file assembler
    # inner-DSL body (write calls + for/if + state reads)
    write body
end
```

Strings use double quotes (with C-style escapes `\n` `\t` `\"`
`\\`) or backticks (multi-line, with `${EXPR}` interpolation).
Bare words are accepted for `extension`, type names, and capture
names. Indentation delimits the function body and `file_template`.

---

## Args

Each function's `arg` lines take one of two forms:

- `arg literal "TEXT"` — match this exact token in source.
- `arg capture NAME TYPE` — bind a captured value to NAME.

Both forms accept an optional trailing description string for
auto-generated docs.

## Auto-name-prepend rule

If a function declares ZERO `arg literal` lines, the engine
auto-prepends a literal of the function's name. So:

```
function greet
    arg capture name any
end
```

…matches `greet <any>` in source. As soon as you write any literal,
you own the entire shape (function name NOT auto-prepended). That's
how you define operator-style functions like:

```
function assign
    arg capture var ident
    arg literal "="
    arg capture value any
    write `${var} = ${value}
`
end
```

…which matches `<ident> = <any>` — no leading `assign` token.

## Built-in capture types

| Type     | Captures                                              |
|----------|-------------------------------------------------------|
| `any`    | Any value expression (number, string, ident, list, object, dotted path, paren-sub-call, comparison). |
| `ident`  | A single identifier token.                            |
| `raw`    | Identifier OR string.                                 |
| `word`   | Shell-style bare word — a maximal run of adjacent tokens with no source whitespace. Captures `--oneline`, `k8s/deploy.yaml`, `name=^web$`, `restart-api` as ONE value. |
| `dotted_ident` | `IDENT(.IDENT)*` captured as one string, e.g. `err.kind`. |
| `tail`   | Every remaining token on the statement, joined with original column spacing. For free-form trailing values like `20px` or `1px solid red`. Quoted tokens keep their quotes — `-m "fix the bug"` stays one slot, not `-m fix the bug`. |
| `string` | A quoted string — OR a bare identifier.               |
| `int`    | An integer literal — OR a bare identifier.            |
| `float`  | A float literal — OR a bare identifier.               |
| `bool`   | `true`/`false` — OR a bare identifier.                |

Bare identifiers pass primitive type checks because they could refer to
target-language variables.

## Library-defined types

```
type Email
    base string                           # built-in kind check first
    pattern "^[^@]+@[^@]+\\.[^@]+$"       # regex
end

type Status
    options ["todo", "done"]              # enum
end
```

Applied in order: `base` → `pattern` → `options`. All three are optional.

---

## The inner DSL (the function body)

A small fixed language. Updates `context` only — does NOT execute user
code.

### Statements

```
set <path> <value>             # bind a field
append <path> <value>          # push to a list
prepend <path> <value>         # push to front
merge <path> <map-value>       # shallow-merge into a map
delete <path>                  # remove field/key
if <expr>                      # library-side conditional
    ...
end
loop <var> in <expr>           # library-side iteration
    ...
end
error <message>                # abort with a message
```

### Paths

Rooted at `context` (or at a `loop` local). Examples:

```
context.imports
context.config.api.url
context.scripts[name]          # `name` is a capture/local; evaluated to a key
context.buf[i]                 # list element by integer index (negative = from end)
context.grid[i][j]             # nested; index then field also works: rows[i].name
```

`[<expr>]` index steps work in **read** position too (inner-DSL value
position and `${…}` templates), not just as a `set …[key]` write target —
same semantics both ways. Map parents key on the index's string form,
list parents on an integer. A missing key / out-of-range index is `nil`
(falsy) / empty, so `if context.seen[k]` guards without a `for`-scan.

### Expressions

- Numbers, strings (including ${interp}), `true`, `false`, `null`.
- Identifier paths resolve in order: locals (loop vars), captures, context.
- Lists `[1, 2, 3]`, objects `{"k": "v", name: "Alice"}` (keys can be unquoted idents).
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`. Unary `not expr`.
- `(regex_match value pattern)` returns a boolean — useful in `if` conditions.

### Captures inside the body

When you reference a capture, you get the **evaluated** value:

- String literal `"foo"` → `Val::Str("foo")` (no quotes).
- Number `42` → `Val::Int(42)`.
- List `[1, 2]` → `Val::List([Val::Int(1), Val::Int(2)])`.
- Object `{a: 1}` → `map[string]any{"a": int64(1)}`.
- Bare identifier `x` → string `"x"`.

So `append context.imports name` for source `import json` correctly stores
`"json"` (without quotes).

---

## Interpolation and helpers

Inside a `write \`...\`` literal you can interpolate with `${expr}`
and pipe through helpers: `${expr | helper}` or `${helper arg expr}`.

Available bindings inside a function body:

- captures by name — e.g. `${name}` (string captures keep their source quotes).
- `${body}` — the rendered inner block (block functions only).
- `context.<field>` — read-only access to accumulated state.

Inside `file_template`:

- `${body}` — concatenation of all top-level statements' output.
- `context.<field>` — final accumulated context.

### Helpers

- `indent N` — pad every line with N spaces. Use for block bodies.
- `lower`, `upper` — case.
- `join SEP` — joiner over a list.
- `toQuoted` — wrap a string in `"…"`.
- `toPyLit` — Python literal formatting (True/False/None, lists, dicts).
- `toJSON`, `toJSONIndent` — JSON marshal.
- `asString` — normalise a capture to ONE valid JSON string, quoting iff not already a string (handles bare ident OR quoted string uniformly).
- `unquote` — strip surrounding quotes from a captured string.

---

## Two block modes

### Mode A: named closer + indentation

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

Body is delimited by INDENT/DEDENT (4 spaces or 1 tab per level). The
named closer must match after the body.

### Mode B: explicit delimiters

```
function for
    arg literal "for"
    arg capture v ident
    arg literal "in"
    arg capture i any
    block_open "{"
    block_close "}"
    write `for ${v} in ${i} {
${indent 2 body}
}
`
end
```

Body delimited by the literal `{` and `}` tokens. No closer function.

---

## Worked example (full Python transpiler library)

```
extension py

context
    imports []
end

type Identifier
    pattern "^[A-Za-z_][A-Za-z0-9_]*$"
end

function import
    arg literal "import"
    arg capture name Identifier
    append context.imports name
end

function say
    arg capture msg any
    write `print(${msg})
`
end

function assign
    arg capture name Identifier
    arg literal "="
    arg capture value any
    write `${name} = ${value}
`
end

function if
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}
`
end

function loop
    arg literal "loop"
    arg capture var ident
    arg literal "in"
    arg capture iter any
    block_closer end
    write `for ${var} in ${iter}:
${indent 4 body}
`
end

function end
end

file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

Source:

```
import json
say "hello"
x = 42
if x
    say x
end
```

Output:

```python
import json
print("hello")
x = 42
if x:
    print(x)
```

---

## Common pitfalls

1. **Putting business logic in user-script** — Capy transpiles, it
   doesn't execute. `if x ... end` emits an `if`; it doesn't
   conditionally render.
2. **Quoting confusion** — string captures expose source text *with*
   quotes in templates. Use the value as-is for Python (which uses
   the same quoting) or strip with the `unquote` helper if needed.
3. **`{}` ambiguity** — `{...}` is an object literal by default. For
   `{...}` blocks, the opener function must declare `block_open "{"`
   and `block_close "}"` explicitly.
4. **Indentation must be 4 spaces or 1 tab per level.** 2-space
   indent breaks the lexer.
5. **No `else`** — use two `if` blocks (one with `not`) for now.
6. **Auto-name-prepend silently disables** the moment you add any
   `arg literal`. So an `arg literal "in"` line somewhere removes
   the automatic function-name leading literal.

## CLI quick reference

```sh
capy run <lib.capy> <script.capy>     # transpile
capy check <lib.capy>                 # validate library
capy docs <lib.capy>                  # auto-generate reference docs
capy init [<dir>]                     # scaffold
capy version
capy help [<command>]
```

## When in doubt

Run `capy check lib.capy` after every edit. If it loads cleanly, run
`capy run lib.capy script.capy` against a minimal script. Errors are
caret-pointed at line:col.

---

## Format note

Libraries are always `.capy` files. The embedded Rust API uses
`Library::new` to load a `.capy` source.
