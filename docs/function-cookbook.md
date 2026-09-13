# Built-in function cookbook

Capy has **zero source-language grammar** of its own — every keyword in a
script comes from a `.capy` library. But the *template* side ships a fixed
set of **built-in helper functions** you call inside `${ … }` (and inside
`run`/`set` inner-DSL expressions). They are the only functions Capy
itself defines; everything else you build.

> **Looking for the library-authoring keywords** — `function`,
> `arg literal`, `arg capture`, `block_closer`, `write`, `type`, … (the
> vocabulary you write *inside a `.capy` library*)? Those are documented
> in the [Library keyword cookbook](library-keywords.md). This page is
> only the `${ … }` template helpers.

This page documents **every** built-in helper, what it does, and a worked
example. The examples are mirrored by a runnable sample —
[`samples/builtin-functions/`](https://github.com/olivierdevelops/capy/tree/main/samples/builtin-functions)
— whose golden output is checked in CI, so what you read here is what the
engine actually produces.

> **Maintenance rule.** This list must stay in lock-step with the helper
> table in `rust/src/infra/helpers.rs`. Whenever a built-in function is added,
> renamed, or removed, update this page (and `samples/builtin-functions/`)
> in the same change. See `CLAUDE.md`.

## How to call a helper

Inside a `write` / `template` body, a helper is called **prefix-style** —
name first, arguments after, separated by spaces, no commas:

```
${pascalCase name}            # one argument
${add count 1}                # two arguments
${indent 4 body}              # literal + the body local
```

Wrap a sub-call in parentheses to nest:

```
${escapeHtml (decoded text)}  # decode escapes, THEN HTML-escape
${dasherize (snakeCase name)} # "Order Item" → order-item
${toJSON (split argv " ")}    # split a string, then JSON-encode the list
```

The same helpers are available in inner-DSL expressions, so
`set context.total (add context.total n)` works identically.

## Full signature reference

Every helper, its **parameters with types**, **arity**, and **return
type** — taken directly from the `funcs` map and the `ApplyHelper`
dispatch in `rust/src/infra/helpers.rs`. Call order matches the parameter order
(prefix style, space-separated). See the [type glossary](#parameter-return-types)
below the table.

| Function | Parameters (in order) | Args | Returns | Summary |
|----------|-----------------------|:----:|---------|---------|
| [`indent`](#indent) | `n int`, `s string` | 2 | `string` | Indent every non-blank line of `s` by `n` spaces |
| [`lower`](#lower-upper) | `s string` | 1 | `string` | Lowercase |
| [`upper`](#lower-upper) | `s string` | 1 | `string` | Uppercase |
| [`pascalCase`](#pascalcase-camelcase-snakecase) | `s any` | 1 | `string` | `display name` → `DisplayName` |
| [`camelCase`](#pascalcase-camelcase-snakecase) | `s any` | 1 | `string` | `display name` → `displayName` |
| [`snakeCase`](#pascalcase-camelcase-snakecase) | `s any` | 1 | `string` | `display name` → `display_name` |
| [`dasherize`](#dasherize) | `s any` | 1 | `string` | `_` → `-` (snake → kebab) |
| [`unquote`](#unquote) | `s any` | 1 | `string` | Strip one layer of surrounding quotes |
| [`unescape`](#unescape) | `s any` | 1 | `string` | Reverse string escaping (Go `strconv.Unquote` semantics) |
| [`decoded`](#decoded) | `s any` | 1 | `string` | User-intended string: unquote + resolve escapes |
| [`escapeHtml`](#escapehtml) | `s any` | 1 | `string` | Neutralise `& < > " '` for HTML |
| [`toQuoted`](#toquoted) | `s any` | 1 | `string` | Wrap in JSON-style double quotes |
| [`asString`](#asstring) | `s any` | 1 | `string` | Normalise any capture to exactly one JSON string |
| [`toPyLit`](#topylit) | `v any` | 1 | `string` | Format a value as a Python literal |
| [`toJSON`](#tojson-tojsonindent) | `v any` | 1 | `string` | Compact JSON |
| [`toJSONIndent`](#tojson-tojsonindent) | `v any` | 1 | `string` | Pretty-printed JSON (2-space indent) |
| [`trimSuffix`](#trimsuffix-trimprefix) | `suffix string`, `s any` | 2 | `string` | Drop a trailing substring from `s` |
| [`trimPrefix`](#trimsuffix-trimprefix) | `prefix string`, `s any` | 2 | `string` | Drop a leading substring from `s` |
| [`join`](#join) | `sep string`, `items []any` | 2 | `string` | Join a list with `sep` between items |
| [`split`](#split) | `s any`, `sep string` | 2 | `[]string` | Split `s` into a list at each `sep` |
| [`nonEmpty`](#nonempty) | `items []string` | 1 | `[]string` | Drop blank entries from a string list |
| [`add`](#add-sub-mul) | `a any`, `b any` | 2 | `int64` | Integer `a + b` |
| [`sub`](#add-sub-mul) | `a any`, `b any` | 2 | `int64` | Integer `a - b` |
| [`mul`](#add-sub-mul) | `a any`, `b any` | 2 | `int64` | Integer `a * b` |
| [`div`](#div-mod-align) | `a any`, `b any` | 2 | `int64` | Integer quotient `a / b` (0 if `b == 0`) |
| [`mod`](#div-mod-align) | `a any`, `b any` | 2 | `int64` | Remainder `a % b` (0 if `b == 0`) |
| [`align`](#div-mod-align) | `n any`, `a any` | 2 | `int64` | Round `n` up to the next multiple of `a` |
| [`percent`](#percent) | `n any`, `d any` | 2 | `int64` | `n / d * 100`, clamped to 0–100 |
| [`stars`](#stars) | `n any` | 1 | `string` | `n` filled stars + the rest outlined, out of 5 |

### Parameter & return types

The types above are the value kinds each helper accepts; here's what
each means at a Capy call site:

| Type | At the call site | Notes |
|------|------------------|-------|
| `any` | any captured value — a string capture, a number, a context value | String helpers stringify it (and most strip one surrounding quote layer themselves); numeric helpers coerce it to an integer (`int`/`int64`/`float`, or a digit-string like `"42"`). |
| `string` | a value used as a fixed string — usually a literal you type (`","`, `".go"`) | A capture passed here is stringified first. |
| `int` | a whole number — only `indent`'s `n` | Coerced from `int`/`int64`/`float`/digit-string. |
| `int64` | **return** type of the arithmetic helpers | Renders as a plain integer. |
| `[]any` / `[]string` | a **list** value | Produced by a context list you built with `set`/`append`/`range`, or by [`split`](#split). Reference it by name (`context.models`); you can't write a list literal inline. |

**Arity is enforced.** Passing the wrong number of arguments is an error
(`helper "join": expected 2 args, got 1`), so the *Args* column is exact.

> **Tip — composing list helpers.** `split` returns `[]string` while
> `join` expects `[]any`, so they don't chain directly. Use `join` on a
> context list you built (`${join ", " context.models}`) and use `split`
> to *drive* a loop (`for x in (split s ",")`), not to feed `join`.

---

## Layout

### `indent`

`indent N str` — prefix every **non-blank** line of `str` with `N`
spaces. Blank lines stay empty (no trailing whitespace). The classic use
is indenting a block function's `body` to the target language's nesting:

```
function block
    arg literal "block"
    block_closer end
    write `block:
${indent 2 body}`
end
```

```
block:
  one
  two
```

---

## Case & identifiers

These turn a friendly display name into an identifier. Each one strips a
single layer of surrounding quotes first, then splits on space, `-`, `_`,
and `.`.

### `pascalCase` / `camelCase` / `snakeCase`

| Call | `"user profile"` → |
|------|--------------------|
| `${pascalCase s}` | `UserProfile` |
| `${camelCase s}`  | `userProfile` |
| `${snakeCase s}`  | `user_profile` |

`pascalCase` upper-cases the first letter of every segment; `camelCase` is
the same with a lowercased first letter; `snakeCase` lowercases and joins
with `_`.

> **Watch the input.** `snakeCase` *also* inserts `_` before an interior
> capital, so `"User Profile"` (space **and** capital `P`) becomes
> `user__profile` (double underscore). Feed it an already-lowercase
> display name, or run identifiers through it, to avoid surprises.

### `lower` / `upper`

`lower str` / `upper str` — straight case folding (no quote stripping; pair
with `unquote` if the capture is quoted):

```
${upper (unquote s)}   # "user profile" → USER PROFILE
${lower (unquote s)}   # "user profile" → user profile
```

### `dasherize`

`dasherize str` — replace every `_` with `-`. Built for CSS property names
and kebab-case identifiers, where the lexer won't allow a `-` inside an
identifier. Compose with `snakeCase`:

```
${dasherize (snakeCase s)}   # "user profile" → user-profile
```

---

## Strings & escaping

A `string` capture surfaces in templates as the **source-quoted** form —
`p "He said \"hi\""` gives `text` the value `"He said \\\"hi\\\""`. These
helpers convert it to whatever the target wants.

### `unquote`

`unquote str` — strip exactly one layer of matched surrounding quotes
(`"…"`, `'…'`, or `` `…` ``) if present; otherwise return the input
unchanged. Escape sequences inside are **not** resolved — use
[`decoded`](#decoded) for that. Good for Markdown headings and other
spots that just need the quotes gone.

### `decoded`

`decoded str` — deliver the **user-intended** string: strip the outer
quotes *and* fully resolve C-style escapes (`\"`, `\n`, `\t`, `\\`,
`\xNN`, `\uNNNN`). This is what you want when a capture may contain
escaped quotes or whitespace:

```
text "He said \"hi\" <b>"
```

| Call | Output |
|------|--------|
| `${unquote s}` | `He said \\\"hi\\\" <b>` |
| `${decoded s}` | `He said "hi" <b>` |

Unlike `unquote`, `decoded` tolerates a bare `"` inside the string (common
in captured HTML like `class="card"`), so newlines and tabs always resolve.

### `escapeHtml`

`escapeHtml str` — replace the five characters every HTML emitter must
neutralise (`& < > " '`) with their entities, ampersand first. Your XSS
guard for any free-form capture dropped into `<…>${…}</…>`. Compose it
**after** `decoded`:

```
write `<p>${escapeHtml (decoded text)}</p>`
```

```
He said "hi" <b>   →   He said &quot;hi&quot; &lt;b&gt;
```

> The verbose name is deliberate: `${html x}` would read as "make this
> HTML"; `escapeHtml` reads as "escape this *for* HTML" — what it does.

### `unescape`

`unescape str` — reverse string escaping (Go `strconv.Unquote` semantics)
(wrapping in `"…"` first if unquoted). Use it when the **target** wants
the literal escape sequence resolved — assembler `.asciz`, C string
literals, JSON on the wire. For HTML/template work prefer `decoded`, which
is lenient about bare quotes; `unescape` falls back to the raw input if
the value isn't valid escaped-string syntax.

### `toQuoted`

`toQuoted str` — wrap a string in JSON-style double quotes (also valid for
Python). Uses `json.Marshal`, so `<`, `>`, and `&` come out as `<`,
`>`, `&`:

```
${toQuoted (decoded text)}   # He said "hi" <b>  →  "He said \"hi\" <b>"
```

### `asString`

`asString str` — normalise **any** capture to exactly one valid JSON
string, quoting only if it isn't already a string literal. Solves the case
where a `raw`/`any` capture holds either a bare token (`foo`, `42`) or a
quoted string (`"foo"`): both interpolate to `"foo"`. It peels one quote
layer and decodes escapes (like `decoded`) then re-encodes as JSON.

```
write `{"op":"exec","bin":${asString bin},"argv":${toJSON (split argv " ")}}`
```

### `trimSuffix` / `trimPrefix`

`trimSuffix suffix str` / `trimPrefix prefix str` — drop a trailing /
leading substring if present. Note the **fixed string comes first**, the
value second (reads naturally as a pipeline). The canonical use is
shaving the dangling comma off a comma-suffixed list body:

```
${trimSuffix ",\n" body}     # drop the trailing comma+newline
```

```
trimSuffix ".go" "src/main.go"   →  src/main
trimPrefix "src/" "src/main.go"  →  main.go
```

---

## Lists

List helpers work on the `[]`-valued context entries you build with the
inner DSL (`set context.items …`, `range`) or on the result of `split`.

### `join`

`join sep list` — join a list of values with `sep` between them. Real
example from the CSV transpiler, joining an accumulated row:

```
write `${join "," context.header}`
write `${join ", " context.models}`
```

### `split`

`split str sep` — split a string into a list at each `sep` (argument order
matches `strings.Split`: value first, separator second, so it reads well
inline). Drive a loop or feed another helper:

```
for line in (split context.api_keys "\n")   # inner-DSL loop
${toJSON (split argv " ")}                   # split then JSON-encode
```

### `nonEmpty`

`nonEmpty list` — filter a string list down to entries that aren't blank
after trimming. Handy after splitting `read_file` output so a trailing
newline doesn't produce an empty final row:

```
range (nonEmpty (split (read_file "items.txt") "\n"))
```

---

## Serialisation

### `toJSON` / `toJSONIndent`

`toJSON value` marshals any value to **compact** JSON; `toJSONIndent value`
pretty-prints with two-space indentation. Great for config-file targets:

```
${toJSON context.settings}
${toJSONIndent context.settings}
```

### `toPyLit`

`toPyLit value` formats a value as a **Python literal**: `nil` → `None`,
`true`/`false` → `True`/`False`, strings get quoted, lists render as
`[…]`, maps as `{…}`. For emitting Python source from accumulated context.

---

## Numbers & display

### `add` / `sub` / `mul`

`add a b`, `sub a b`, `mul a b` — integer arithmetic; both arguments are
coerced to `int64` (a digit-string from a `string` capture works too).
Common in running totals inside `range`:

```
set context.total (add context.total n)
${add a b}   # 3 4 → 7
${sub a b}   # 3 4 → -1
${mul a b}   # 3 4 → 12
```

### `div` / `mod` / `align`

`div a b`, `mod a b`, `align n a` — the integer ops that running-total
bookkeeping (`add` alone) can't express. Both `div` and `mod` return `0`
when the divisor is `0` so a template never panics on bad input.

`align n a` rounds `n` **up** to the next multiple of `a` — the classic
`(n + a - 1) / a * a`. It's the one op that ABI code generators need:
struct-field offsets, struct total size, and 16-byte stack-frame
alignment all reduce to it. Because it's a named helper, a library emits
correct layout without open-coding the formula (and without bitwise ops,
which Capy doesn't have).

```
${div 17 4}     # → 4   (quotient)
${mod 17 4}     # → 1   (remainder)
${align 9 4}    # → 12  (next multiple of 4 at or above 9)
${align 13 8}   # → 16  (16-byte stack frame for 13 bytes of locals)
```

A struct-layout function placing each field at its aligned offset:

```
function field
    arg literal "field"
    arg capture name ident
    arg capture size int
    # align INLINE in the template — a same-function `set` wouldn't be
    # visible to its own `write` (the one-call lag).
    write `; ${name} @ offset ${align context.off size} (size ${size})
`
    set context.off (add (align context.off size) size)
end
```

For `id:8`, `flag:1`, `count:4` this emits `count @ offset 12` (padded
9 → 12) — the ABI-correct layout, not the naive `offset 9`.

### `percent`

`percent n d` — `n / d * 100` as an integer, clamped to `[0, 100]`
(returns 0 when `d == 0`). Built for HTML progress bars:

```
<div class="bar" style="width:${percent done total}%"></div>
```

```
percent 3 4   →  75
```

### `stars`

`stars n` — render `n` filled stars (`★`) followed by the remainder out of
five as outlined stars (`☆`), clamped to 0–5. For ratings in
non-programmer DSLs (reading logs, restaurant lists):

```
stars 3   →  ★★★☆☆
```

---

## See also

- [Templates](templates.md) — the `${ … }` interpolation syntax and the
  per-function / file-template values in scope.
- [Inner DSL](inner-dsl.md) — `run`/`set`/`range`/`if`, where these same
  helpers are available in expression position.
- [`samples/builtin-functions/`](https://github.com/olivierdevelops/capy/tree/main/samples/builtin-functions)
  — the runnable, CI-checked source for every example above.
