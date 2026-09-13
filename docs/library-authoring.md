# Library Authoring Guide

A Capy library is the entire grammar of one source language, plus the
recipe for generating output from it. This doc is the reference
walkthrough.

Libraries are written in **`.capy`** — Capy's native syntax. Multi-line
`write` blocks read natively, same indentation and string rules as the
source files the library will parse. Every example below is `.capy`.

## File shape

A complete Capy library has these top-level sections (all optional
except at least one `function`):

```
extension py                          # informational; suggests target extension
output_file ""                        # if set, capy writes here instead of stdout

comments                              # OPTIONAL — opt user scripts into a
    line "#"                          # comment syntax. With no `comments`
end                                   # block, user scripts have NO comments
                                      # (the engine ships zero defaults).

context                               # initial schema for the accumulated context
    imports []
    classes []
end

type Email                            # library-defined argument types
    pattern "^[^@]+@[^@]+\\.[^@]+$"
end

function greet                        # one DSL statement shape
    arg literal "greet"
    arg capture name string
    write `Hello, ${name}!
`
end

file_template                         # final-output assembler
    write body
end
```

## Comments — opt-in, not automatic

Capy ships **zero predefined grammar**, and that includes comment
syntax. Out of the box, a user script that starts a line with `#`
or `//` will fail to lex with `unexpected character '#'`. To allow
comments in scripts written against your library, declare them:

```
comments
    line "#"
    line "//"
end
```

Each `line "MARKER"` entry adds one line-comment prefix. The marker
matches at the start of a line (after any indent) and at any point
on a line; everything from the marker to the end of the line is
discarded by the lexer.

This declaration only affects **user scripts**. The library
manifest itself, and inner-DSL / command bodies, always use
`#` for comments — that's Capy's own config syntax.

Why opt-in: a library targeting a host language that uses `#` as
a meaningful character (e.g. CSS selectors, Markdown headings,
Mermaid flowcharts) might want to forbid `#` comments to avoid
ambiguity. The grammar belongs to the library, end-to-end.

## Functions

Each `function NAME … end` block defines one DSL statement shape.
`NAME` is the function's reference name (used for the auto-name-prepend
rule and to name a block's closer); it's not necessarily what appears
in source.

The function body is a sequence of inner-DSL statements. Two
common forms:

- **`write \`...\``** appends to the output body. Backtick literals
  can span multiple lines and accept `${EXPR}` interpolation.
- **`set` / `append` / `prepend` / `merge` / `delete`** mutate the
  accumulated context.

```
function greet
    arg literal "greet"
    arg capture name Email             # built-in OR library-declared type
    priority 0                         # higher wins; default 0 — must come
                                       # BEFORE the output statements
    write `Hello, ${name}!
`
    append context.greetings name
end
```

The body is the *only* place statements go. Renders (`write`) and
state mutations (`set` / `append` / …) interleave freely; the engine
walks the AST twice per function call — once to produce output, once
to update `context`.

## `arg` — the match shape

Each `arg` line takes one of two forms:

| Form                                | Meaning                                            |
|-------------------------------------|----------------------------------------------------|
| `arg literal "TEXT"`                | A literal token to match exactly in source.        |
| `arg capture NAME TYPE`             | Bind a captured value of type TYPE to NAME.        |

Both forms accept an optional trailing description string for
[auto-generated docs](library-documentation.md):

```
arg literal "recipe" "Open a new recipe."
arg capture title string "Display name, shown as the H1."
```

### Optional args with defaults

A trailing capture can declare a `default`, making it omittable at
the call site. The full `arg capture` shape is:

```
arg capture NAME [TYPE] [default "VALUE"] [DESCRIPTION]
```

When the call site ends before reaching the optional arg, the
engine binds the default instead of consuming a token. This
collapses families of near-duplicate components into one function:

```
function button
    arg literal "button"
    arg capture label   string
    arg capture variant string default "primary"
    arg capture kind    string default "button"
    template
        <button type="${decoded kind}" class="btn btn-${decoded variant}">${escapeHtml (decoded label)}</button>
    end
end
```

```
button "Save"                       → btn-primary, type=button
button "Delete" "danger"            → btn-danger,  type=button
button "Submit" "primary" "submit"  → btn-primary, type=submit
```

Optional args must be **trailing** — a required capture (or a
literal) after an optional one is a load-time error, since the
matcher couldn't tell which slot a supplied value fills. A `string`
default is delivered in the same quoted form a real string capture
carries, so `${x}` and `${decoded x}` behave identically whether the
arg was supplied or defaulted.

### Auto-name-prepend

If a function declares **zero** `arg literal` lines, the engine
prepends a literal of the function's name. So:

```
function greet
    arg capture name any
    write `Hello, ${name}!
`
end
```

…matches `greet <any>` — the function name becomes the leading token.

As soon as you add any `arg literal`, you own the entire shape. This
is how you build operator-style functions:

```
function assign
    arg capture var ident
    arg literal "="
    arg capture value any
    write `${var} = ${value}
`
end
```

…matches `<ident> = <any>`. No leading `assign` token in source.

### `bare` — opt out of auto-name-prepend

If you want a function whose args are all captures AND you DON'T want
a leading keyword, declare it `bare`. The function then matches
purely by shape — useful for grammars whose data lines have no
syntactic anchor:

```
function row
    bare
    arg capture a string
    arg capture b string
    arg capture c string
    write `<button>${unquote a}</button><button>${unquote b}</button><button>${unquote c}</button>
`
end
```

Source: `"1" "2" "3"` parses as one call to `row` with three
captures — no `row` keyword needed.

Use sparingly: a function declared `bare` will try to match every
statement that starts with a capture-compatible token, so prioritise
specific shapes ahead of it if there's ambiguity.

## Built-in types

`any`, `ident`, `raw`, `string`, `int`, `float`, `bool`.

| Type   | What it captures                                                                                                   |
|--------|--------------------------------------------------------------------------------------------------------------------|
| `any`  | Any single value expression (number, string, ident, list, object, bool, null, dotted ident, parenthesized sub-call). |
| `ident`| A single identifier token; bound as a string.                                                                       |
| `raw`  | One identifier OR string token; bound as a string.                                                                   |
| `tail` | Every remaining token on the statement, reconstructed with original column-position spacing. Use for free-form trailing values (e.g. CSS `20px`, `1px solid red`) that don't fit `any`'s single-value-expression grammar. |
| `string` | A quoted string literal — OR a bare identifier (transpile-mode permissive).                                       |
| `int`/`float`/`bool` | The respective literal — OR a bare identifier.                                                          |

Bare identifiers always pass primitive type checks because at the
target language's runtime they could refer to a value of any type.

See [types.md](types.md) for library-defined types with
`pattern`/`options`.

## `write` — what goes into the body

`write EXPR` appends EXPR to the function's output. EXPR is most
commonly a backtick literal with `${EXPR}` interpolations:

```
function greet
    arg capture name any
    write `Hello, ${name}!
`
end
```

Inside a backtick string you can use:

- `${name}` — a capture by name.
- `${body}` — the inner block's rendered output (block functions only).
- `${context.X}` — the read-only accumulated context.
- `${func arg arg}` — call a template helper inline. The same
  helpers that work in `text/template` (`indent`, `pascalCase`,
  `toQuoted`, `toJSON`, `lower`, `upper`, `join`, `split`,
  `unescape`, …) are all available — see [templates.md](templates.md).

The bytes inside the backticks are emitted verbatim. There is no
whitespace-trimming sigil — if you don't want a trailing newline,
don't put one in.

## `template … end` — write-literal sugar

For functions whose output is a multi-line HTML / text block, the
backtick bookkeeping (`write \` …`, closing \` on its own line, escape
hatches for embedded backticks) crowds out the actual content. The
`template … end` sugar drops the ceremony:

```
function card
    arg capture title string
    block_closer end
    template
        <div class="card">
          <h3>${escapeHtml (decoded title)}</h3>
          <div>${indent 2 body}</div>
        </div>
    end
end
```

Semantically **identical** to the hand-authored equivalent:

```
function card
    arg capture title string
    block_closer end
    write `<div class="card">
  <h3>${escapeHtml (decoded title)}</h3>
  <div>${indent 2 body}</div>
</div>
`
end
```

Properties:

- **Where it's valid:** anywhere a `write` statement is — function
  bodies, `file_template`, and `file "X" … end` blocks. Mixes
  freely with other statements (`set` / `append` / `if` / `for`).
- **Opener:** a bare `template` line, no trailing args.
- **Closer:** a bare `end` at the **same indent** as `template`.
  Lines whose `end` is at a deeper indent are body content. Nested
  `template … end` at the same indent are balanced via depth.
- **Body:** captured verbatim, then auto-dedented to flush-left (the
  smallest non-blank leading indent is stripped). Relative
  indentation between body lines is preserved.
- **`${…}` interpolation is active** — same parser as in a backtick
  `write`. This is the only thing that distinguishes the sugar from
  `block_verbatim`, which has interpolation OFF.
- **Empty body** is allowed (`template\n…blanks…\nend` produces
  empty output, no error).
- **Backticks and backslashes** inside the body are escaped
  automatically for the synthesised backtick literal — paste HTML
  with embedded `` ` `` characters worry-free.

Implementation note (for the curious): the sugar is rewritten into
a synthesised `write ` … ` ` BEFORE the inner-DSL parser runs.
There's no new AST node and the renderer doesn't need to know the
sugar exists. So everything that works with a hand-authored
multi-line `write` works with `template … end`.

## State mutation — `set`, `append`, `prepend`, `merge`, `delete`

A small inner DSL. **Does not execute user source.** It only mutates
the `context` map.

```
function import
    arg literal "import"
    arg capture name ident
    append context.imports name
end
```

Statements sit directly in the function body alongside `write` calls.
The renderer ignores state-mutation statements during the render pass;
the run pass ignores `write`. Both walk the same AST so control flow
(`if` / `for`) that contains both kinds of statement behaves
consistently.

Operations available:

| Form                                  | Effect                                       |
|---------------------------------------|----------------------------------------------|
| `set <path> <value>`                  | Set a field at a dotted/indexed path.        |
| `append <list-path> <value>`          | Append to a list.                            |
| `prepend <list-path> <value>`         | Prepend to a list.                           |
| `merge <map-path> <map-value>`        | Shallow-merge into a map.                    |
| `delete <path>`                       | Remove a field/key.                          |
| `if <expr>` ... `end`                 | Library-side conditional update.             |
| `loop <var> in <expr>` ... `end`      | Library-side iteration.                      |
| `(regex_match value pattern)`         | Boolean expression value.                    |
| `(env "X")`, `(arg N)`, `(read_file "P")` | [Host capabilities](host-capabilities.md). |
| `(os)`, `(arch)`                      | Branch on host platform.                     |
| `error "<message>"`                   | Abort transpilation with a message.          |

See [inner-dsl.md](inner-dsl.md) for full details and examples.

## Block functions

A function that opens a body block declares it in one of three modes —
exactly one of `block_closer`, `block_open`/`close`,
`block_dedent`, or `block_verbatim`:

| Directive | Body delimited by | Body parsed? | Use for |
|---|---|---|---|
| `block_closer NAME` | An indented body that ends at a `NAME` keyword | Yes — as nested statements | Most cases — `if … end`, `for … end`, `match … end`. |
| `block_open "X" close "Y"` | A `X`-prefixed, `Y`-suffixed delimited body on the same line as the opener (newlines inside OK) | Yes | Brace-style nesting, JSX-ish DSLs. |
| `block_dedent` | An indented body that ends at the first DEDENT — no closer keyword | Yes | CSS-style selector blocks, YAML-style sections. |
| `block_verbatim NAME` | An indented body that ends at a `NAME` keyword | **No** — captured as raw source bytes | Code blocks, embedded HTML / SVG, anywhere the body is data not grammar. |

Named-closer example:

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

Or with explicit delimiters:

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

Dedent-only block (no keyword needed to close):

```
function selector
    arg capture name ident
    arg literal ":"
    block_dedent
    write `${name} {
${indent 2 body}}
`
end
```

Source:

```
header:
    color: white
    padding: 8px

footer:
    color: gray
```

Each selector's body is whatever is indented under it, up to the next
peer line or the block's enclosing dedent. The parser is also lenient
about stray indents inside a body — content nested deeper than its
block's anchor (purely for visual styling) is treated as cosmetic, so
hand-written DSL sources don't have to be pedantic about consistent
indentation.

Verbatim block — captures the body as raw source bytes (NO nested
parsing). Use for code blocks, embedded HTML, or anywhere the body
is data not grammar. Pair with `${escapeHtml body}` (see
[templates.md](templates.md#escapehtml-string)) for XSS-safe emission:

```
function pre
    arg capture lang ident
    block_verbatim end
    write `<pre><code class="language-${lang}">${escapeHtml body}</code></pre>
`
end
```

Source:

```
pre go
    func main() {
        fmt.Println("hello & welcome <world>")
    }
end
```

…renders the generated Go code with its quotes and angle-brackets HTML-escaped
inside a `<pre><code>` block, indentation preserved verbatim. Every
line between `pre go` and `end` is captured byte-for-byte; the user
can paste arbitrary code without it being re-parsed as Capy.

The capture is the literal source byte range, so **blank lines and
comment-marker (`#`) lines are preserved exactly** — a `markdown …
end` block with `## headings` and blank paragraphs round-trips
untouched.

### Inline syntax via `group_open` / `group_close`

For Markdown-style or LaTeX-style inline syntax (`[label](url)`,
`**bold**`, `~~strike~~`), declare a type that opens and closes on
specific delimiters:

```
type Bracketed
    group_open  "["
    group_close "]"
end

type Parens
    group_open  "("
    group_close ")"
end

function link
    arg literal "link"
    arg capture text Bracketed
    arg capture url  Parens
    write `<a href="${escapeHtml url}">${escapeHtml text}</a>
`
end
```

Source `link [Al the Alien](https://alien.com/1)` matches `link` with
`text = "Al the Alien"` and `url = "https://alien.com/1"`. See
[types.md](types.md#group-types) for the full reference including
nested groups and multi-char delimiters.

See [block-functions.md](block-functions.md) for nesting and edge cases.

## `context` — initial schema

Whatever fields your function bodies will manipulate (via `set`/
`append`/etc.). Lists default to
`[]`, maps to `{}`. The context is rendered into the file template as
`.context`.

```
context
    imports []
    classes []
    vars {}
    total 0
end
```

## `file_template` — final assembly

Receives `body` (concatenation of all top-level statements' written
output) and `context` (final accumulated state). Common patterns:

```
# Python-style: imports at top, then body.
file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

```
# Pure JSON: ignore body entirely, render context.
file_template
    write (toJSONIndent context)
end
```

## `priority`

When two functions could match a prefix, the higher `priority` wins;
ties fall back to "more leading literals wins". You rarely need to set
this explicitly — the default (0) plus the literal-leading bias
handles most cases.

## Validation

`capy check lib.capy` parses + validates a library without running any
source. It reports load-time errors with the offending function and
arg index.

## Suggested authoring loop

```sh
capy init my-lib
cd my-lib
# edit lib.capy + script.capy in your editor
capy run lib.capy script.capy        # see output
capy check lib.capy                  # validate
capy docs lib.capy > REFERENCE.md    # regenerate reference docs
```

When the library stabilises, write a few sample scripts under
`examples/` so behaviour stays pinned as you iterate.

