# Library-authoring keyword cookbook

These are the keywords you write **inside a `.capy` library** to define a
source language — `function`, `arg literal`, `arg capture`, the
`block_*` openers, `write`, `type`, and the file-level directives. They
are the vocabulary of the *author*.

> Looking for the `${ … }` helpers you call **inside a template**
> (`pascalCase`, `escapeHtml`, `indent`, …)? Those are a different set —
> see the [Built-in function cookbook](function-cookbook.md).

Every keyword below is dispatched by the library parser in
`rust/src/infra/capy_lib_parser.rs`; the canonical `if … end` example is mirrored
by the CI-checked sample
[`samples/library-keywords/`](https://github.com/olivierdevelops/capy/tree/main/samples/library-keywords).

> **Maintenance rule.** This list must stay in lock-step with the
> directive switches in `rust/src/infra/capy_lib_parser.rs`. Add/rename/remove a
> directive → update this page in the same change. See `CLAUDE.md`.

## The shape of a library

```
extension py            # ← file-level directives

function if             # ← define a source keyword
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}`
end

function end            # ← a silent block closer
end
```

A `.capy` file is a flat list of **top-level directives**. The two that
do the heavy lifting are `function` (define a keyword users can write) and
`type` (define a custom capture type). Everything is opt-in: with an empty
library, every script is rejected — Capy ships *zero* grammar.

---

## File-level directives

Written at column 0.

| Keyword | Form | What it does |
|---------|------|--------------|
| `extension` | `extension py` | Output file extension (sets the default output name / fenced-block hint). |
| `output_file` | `output_file "dist/app.js"` | Fixed output path instead of stdout. |
| `description` | `description "…"` | Library summary; shown by `capy docs`. |
| `name` / `version` | `name "mylib"` / `version "1.0.0"` | Manifest fields. |
| `function` | `function NAME … end` | Define a source keyword. See below. |
| `type` | `type NAME … end` | Define a custom capture type. See [Types](#defining-a-type). |
| `context` | `context … end` | Seed initial context values (state the inner DSL reads/writes). |
| `comments` | `comments … end` | Declare line-comment markers (`# …`). With none, scripts have no comment syntax. |
| `import` | `import "other.capy"` | Merge another library's functions/types. |
| `preprocess` | `preprocess` + `include "@import"` … `end` | Opt into text-level include directives (off by default). |
| `command` | `command "NAME" … end` | Define a `capy <lib> NAME` CLI subcommand. |
| `file` / `file_template` | `file "PATH" … end` | Multi-file output. See [Multi-file output](multi-file-and-imports.md). |
| `impl` / `default_impl` | `impl "d3" "impl/d3.capy" … end` | Swappable interface implementations. |

---

## Defining a function

A `function NAME … end` block defines one source keyword. Inside it:

### Matching the source — `arg`

The `arg` lines declare, **in order**, what the keyword matches on the
source line.

#### `arg literal`

```
arg literal "if"            # match the exact token `if`
arg literal "->" "arrow"    # optional 2nd token = a description
```

Matches a fixed token. A function with **no** `arg literal` (and not
declared `bare`) gets its own name auto-prepended as a leading literal —
so `function say` implicitly requires the token `say` first.

#### `arg capture`

```
arg capture NAME [TYPE] [sep "X"] [join "Y"] [default "V"] [DESCRIPTION]
```

Binds a named value you can interpolate in the template as `${NAME}`.

```
arg capture cond any              # capture an expression as `cond`
arg capture name ident            # capture a single identifier
arg capture attrs attribute*      # repeat a function-typed capture (0+)
arg capture params param+ sep "," join ", "   # 1+, comma in, ", " out
arg capture label string default "Submit"     # optional, trailing
```

The optional modifiers (all independent):

| Modifier | Role |
|----------|------|
| `TYPE` | the capture type — a [built-in](#built-in-capture-types) or another **function/type name** (a *nonterminal*). |
| `*` / `+` suffix | repetition: `*` = zero-or-more, `+` = one-or-more (function-typed captures only). |
| `sep "X"` | **input** separator consumed between repetitions while parsing. |
| `join "Y"` | **output** separator inserted between rendered sub-results. |
| `default "V"` | makes a *trailing* capture optional; binds `V` when omitted. |

### Opening a body — the `block_*` directives

**This is how you know a function "can have a body":** it declares
**exactly one** of the directives below. A function with none of them is a
single-line statement and takes no body. The loader rejects a function
that sets more than one.

When a function opens a body, the rendered inner statements are exposed to
its template as the automatic local `${body}` (see
[block functions](block-functions.md)).

| Directive | Body delimited by | Use it for |
|-----------|-------------------|------------|
| `block_closer end` | indentation; closed by the named closer function (`end`) | Python/YAML-style indented blocks |
| `block_open "{" close "}"` | explicit open/close delimiters | `{ … }` brace blocks |
| `block_dedent` | the first DEDENT (no closer keyword) | CSS selectors, YAML sections |
| `block_verbatim end` | raw source bytes until the closer (no nested parsing) | code blocks, embedded HTML/SVG |
| `block_close_seq "</" name ">"` | an exact multi-token sequence (literals + capture refs) | matched-pair HTML/XML tags |
| `block_sections rescue finally closer end` | a main body plus named sub-sections | try/rescue/finally |

```
function if
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}`
end
```

```
if x > 0      →    if x > 0:
    say "ok"           print("ok")
end
```

The closer (`end` here) is itself a function — often silent
(`function end … end`), but it may emit text (e.g. a `}`).

### Emitting output — `write` and `template`

The function body is **inner-DSL** statements. The two you'll use most:

| Statement | What it does |
|-----------|--------------|
| `` write `…${x}…` `` | Append a template string to this function's output. Backtick strings are multi-line; `${…}` interpolates captures, `${body}`, and [helpers](function-cookbook.md). |
| `template … end` | Multi-line template sugar — same `${…}` rules, no backtick bookkeeping. Desugars to `write`. |

Other inner-DSL statements (for accumulating state across calls) include
`set`, `append`, `prepend`, `merge`, `delete`, `if`, `for`/`loop`, `let`,
and `exec`. See the [Inner DSL reference](inner-dsl.md).

### Function modifiers

| Keyword | Effect |
|---------|--------|
| `description "…"` | Per-function doc, surfaced by `capy docs` and `Introspect()`. |
| `priority N` | Disambiguates overlapping matches — higher wins. |
| `bare` | Opt out of the auto-prepended name keyword (for pure-capture nonterminals like `param NAME TYPE`). |
| `when_followed_by indent` / `when_not_followed_by indent` | Context-sensitive matching: pick this function only when the next line is / isn't indented. |

---

## Built-in capture types

The `TYPE` in `arg capture NAME TYPE`. Any **function or type name** also
works as a type (a *nonterminal* — the capture matches that construct and
renders its template).

| Type | Matches |
|------|---------|
| `any` | one expression (default if no type given) |
| `ident` | a single identifier |
| `dotted_ident` | a dotted path (`a.b.c`) |
| `raw` | exactly one token, verbatim |
| `word` | adjacent tokens with no source whitespace joined (`--oneline`, `k8s/deploy.yaml`) |
| `tail` | every remaining token on the line, original spacing preserved |
| `string` | a quoted string (runs through the expression parser) |
| `int` / `float` / `bool` | a numeric / boolean literal |

> **Trailing-capture tip:** a nonterminal's *last* capture has no
> following literal to stop on, so prefer single-token types (`raw`,
> `ident`, `word`) there — a `string` can swallow a following delimiter
> like `>` as a comparison operator.

---

## Defining a type

`type NAME … end` defines a reusable, validated capture type.

| Keyword | Form | What it does |
|---------|------|--------------|
| `base` | `base ident` | Build on a built-in type. |
| `pattern` | `pattern "^[A-Z][A-Za-z0-9]*$"` | Regex the captured token must match. |
| `options` | `options GET POST PUT` | Restrict to an enumerated set. |
| `group_open` / `group_close` | `group_open "["` / `group_close "]"` | Make the type a delimited inline group (`[label]`). |
| `description` | `description "…"` | Doc string. |

```
type HttpMethod
    base ident
    options GET POST PUT DELETE
end
```

See [Types](types.md) for the full treatment.

---

## See also

- [.capy library syntax](capy-libraries.md) — the formal grammar of a library file.
- [Library authoring](library-authoring.md) — a guided walkthrough.
- [Block functions](block-functions.md) — every body mode in depth.
- [Built-in function cookbook](function-cookbook.md) — the `${ … }` template helpers.
- [`samples/library-keywords/`](https://github.com/olivierdevelops/capy/tree/main/samples/library-keywords) — the runnable source for the `if … end` example.
