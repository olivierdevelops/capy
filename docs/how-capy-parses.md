# How Capy parses and extracts content

This page explains, end to end, **how Capy turns a line of source into
output** — and walks 40+ runnable examples from the simplest possible
grammar up to matched-pair tag parsing, nonterminals, and custom types.

Every example here is a real, self-contained library + script. The
outputs shown are produced by the engine, not hand-written. Run any of
them yourself:

```sh
cargo build --release --manifest-path rust/Cargo.toml -p capy-cli
./capy run lib.capy script.capy
```

> **The one thing to internalise first:** Capy has **zero default
> grammar**. The lexer knows no keywords. There is no built-in `if`, no
> built-in `function`, no built-in anything. *Your library defines the
> entire source language.* Without a library, every script is rejected.

---

## The pipeline

A source file flows through four stages:

```
source text
   │
   ▼  ① LEX      split into tokens (words, numbers, strings, punctuation,
   │            indentation, newlines) — no keywords, purely lexical
   ▼  ② MATCH    for each logical line, find the library function whose
   │            pattern (literals + captures) matches the tokens
   ▼  ③ CAPTURE  pull the variable pieces out of the matched tokens into
   │            named values (a capture per `arg capture`)
   ▼  ④ RENDER   run the function's `write` / `template`, substituting
                captures and render-locals (${body}, ${line}, …)
```

The rest of this page follows that order: first what the lexer
produces, then how a statement is matched, then every way to capture a
value, then disambiguation, blocks, repetition, and types.

---

## Stage ① — Tokenization

The lexer is **purely lexical**. It never decides "this word is a
keyword" — that's the matcher's job in stage ②. It only classifies
characters into tokens.

### 1. Words become idents

Any run of letters/digits/underscores (and any non-ASCII rune — Capy is
UTF-8 aware) is one **ident** token. `set color blue` → three idents:
`set`, `color`, `blue`. `café` and `δelta` are valid single idents too.

### 2. Numbers and strings are their own tokens

`42` is a number token, `3.14` a float, `"hi"` and `'hi'` are string
tokens (the surrounding quotes are stripped from the stored text;
escape sequences are kept verbatim for later decoding), and a
` ``backtick`` ` string is a *template* token that may span multiple
lines.

### 3. Punctuation merges greedily

Adjacent punctuation characters fuse into a single token. This is why
markup parses cleanly:

| Source | Tokens |
|--------|--------|
| `</div>` | `</`  `div`  `>` |
| `><`     | `><` (one token) |
| `x > 0`  | `x`  `>`  `0` |
| `a.b.c`  | `a`  `.`  `b`  `.`  `c` |

The merge set is `=<>!+-*/%&|^~?:,.;@$#\`. Letters/digits never merge
into punctuation, so `</div>` splits exactly at the `div`.

### 4. Indentation is tracked as a width stack

The lexer emits an **INDENT** token when a line is more indented than
the previous one and a **DEDENT** when it dedents. Any *consistent*
indent works — 2 spaces, 4 spaces, a tab — as long as you're
consistent within a block. Blocks (stage ⑤) use these tokens to find
their bodies.

### 5. Each logical line ends in a NEWLINE; the file ends in EOF

Multi-line backtick strings are merged into one logical line before
this happens, so a backtick template never gets split by a newline.

You rarely look at tokens directly — but every later stage operates on
this stream, so the merge rules above explain a lot of "why did my
pattern match like that?" surprises.

---

## Stage ② — Matching a statement

For each logical line, Capy tries the library's functions and uses the
**first complete match** whose tokens end at a valid statement
terminator (newline, EOF, dedent, or a block closer). A function's
pattern is its ordered list of `arg literal "…"` (fixed text) and
`arg capture NAME TYPE` (variable pieces).

### 6. A literal-only function

The minimal grammar: one function, no captures.

```capy
extension txt
function hr
    arg literal "hr"
    write `----------`
end
```

```
# script               # output
hr                      ----------
hr                      ----------
```

The token `hr` matches the literal `"hr"`; the line is complete; the
template fires.

### 7. The auto-name-prepend rule

A function with **no `arg literal`** and that is **not `bare`** gets its
own *name* prepended as a leading literal. So this function:

```capy
extension txt
function greet
    arg capture who ident
    write `Hello, ${who}!`
end
```

…actually matches `greet <ident>`, not a bare ident:

```
# script               # output
greet world             Hello, world!
```

That's why most "keyword + args" functions need no explicit
`arg literal` — the function name *is* the keyword.

### 8. `bare` opts out of the name prefix

When you want a function to match without its name appearing in the
source (a catch-all, an inline node, a nonterminal), declare `bare`:

```capy
function line
    bare
    arg capture txt tail
    write ` ${txt}`
end
```

Now `line` matches any leftover line of text rather than the literal
word `line`. (Used heavily for prose catch-alls and for nonterminals,
below.)

---

## Stage ③ — Capturing values

A capture's **type** decides *how many tokens it eats and how*. These
are the built-in capture types. Pick the one whose appetite matches
what you want.

For the next several examples the function is:

```capy
extension txt
function show
    arg literal "show"
    arg capture x <TYPE>
    write `[${x}]`
end
```

### 9. `ident` — exactly one word

```
show hello              [hello]
```

Eats a single ident token. `show two words` would fail (`words` is
left over).

### 10. `raw` — exactly one token (ident *or* string)

```
show "a quoted string"  [a quoted string]
```

One token, quotes stripped. Perfect for attribute values where a `>`
must stay a tag, not be read as a comparison.

### 11. `word` — a maximal no-whitespace run

```
show k8s/deploy.yaml    [k8s/deploy.yaml]
```

Grabs everything up to the **first whitespace gap**. `/`, `.`, `-`
stay attached. Stops dead at a space — `show a b` leaves `b` over and
fails to match unless something else consumes it.

### 12. `tail` — ALL remaining tokens (greedy)

```
show git commit -m "fix the bug"   [git commit -m "fix the bug"]
```

Consumes the rest of the line, preserving original column spacing and
re-quoting string tokens. **`tail` is greedy** — it eats everything,
including text you may have wanted a later literal to match. (See
example 22 for what to use instead when you need a stop word.)

### 13. `dotted_ident` — a dotted path, no surrounding spaces

```
show err.kind.code      [err.kind.code]
```

Matches `a.b.c` as one value. Whitespace ends it.

### 14. `string` — a quoted literal

A capture typed `string` expects a `"…"` / `'…'` / backtick token. The
stored text is the source-escaped form; use the `decoded`/`unquote`
helpers when rendering (see the [function cookbook](function-cookbook.md)).

### 15–17. `int`, `float`, `bool` — typed scalars

```capy
function set
    arg literal "set"
    arg capture key ident
    arg capture val any
    write `${key} = ${val}`
end
```

```
set n 42                n = 42
set pi 3.14             pi = 3.14
set on true             on = true
```

`int`/`float`/`bool` validate the token shape; `any` (below) accepts
any of them plus expressions.

### 18. `any` — an expression, including a comparison tail

`any` parses a value *expression*. Crucially it absorbs a trailing
comparison, so a condition comes through as one capture:

```
set cond x > 0          cond = x > 0
```

This is exactly how an `if`-style keyword captures `x > 0` as its
condition in one `arg capture cond any`.

### 19. Commas between captures are optional

When two captures sit next to each other, a separating comma is
**auto-skipped** — both spellings work identically:

```capy
function pair
    arg literal "pair"
    arg capture a ident
    arg capture b ident
    write `(${a},${b})`
end
```

```
pair x y                (x,y)
pair x, y               (x,y)
```

---

## Stage ④ — Disambiguation: which function wins?

When several functions could match the same opening tokens, Capy
orders candidates by **(priority desc, then literal-start / literal
length)** and takes the first that yields a complete statement,
backtracking on failure.

### 20. `priority` forces a winner

```capy
extension txt
function generic
    arg literal "set"
    arg capture rest tail
    write `generic: ${rest}`
end
function special
    priority 10
    arg literal "set"
    arg literal "mode"
    arg capture v ident
    write `special mode=${v}`
end
```

```
set mode fast           special mode=fast
set color blue          generic: color blue
```

`set mode fast` matches *both*, but `special` has higher priority and
wins; `set color blue` only fits `generic`.

### 21. Longer/literal patterns beat catch-alls

Even without `priority`, a function that begins with a literal is tried
before a leading-capture catch-all, and a longer literal run is
preferred over a shorter one — so specific grammar reliably shadows a
generic `bare` fallback.

### 22. Multiple literals with a stop word — and the greedy-`tail` trap

You can interleave literals and captures (`select … from …`). But
**don't use `tail` for the middle capture** — it's greedy and will
swallow the stop word. Use a non-greedy type like `word`:

```capy
extension sql
function select
    arg literal "select"
    arg capture cols word
    arg literal "from"
    arg capture tbl ident
    write `SELECT ${cols} FROM ${tbl};`
end
```

```
select id,name from users    SELECT id,name FROM users;
```

`word` stops at the first space, so the `from` literal still has a
token to match. With `tail` here, `cols` would eat `id,name from users`
and the `from` literal would never match.

### 23. Context-sensitive keywords: `when_followed_by` / `when_not_followed_by`

The same keyword can be a flat line *or* a block opener depending on
whether an indented body follows. (`when_followed_by` currently
supports `indent`.)

```capy
function block
    arg literal "do"
    when_followed_by indent
    block_dedent
    write `DO{${body}}`
end
function once
    arg literal "do"
    arg capture n int
    write `once x${n}`
end
```

```
# script         # output
do 3              once x3
do                DO{ a b c d}
  a b
  c d
```

`do 3` has no indented body → `once`; `do` + indented lines → `block`.

---

## Stage ⑤ — Blocks: capturing a body

A function may declare **exactly one** block mode. The block's inner
output is rendered and handed to the template as the `${body}`
render-local. There are five modes.

### 24. `block_closer NAME` — body until a closing keyword

The body runs from the opener to a line containing the closer keyword.
**The closer must itself be a defined function** (usually an empty
one):

```capy
extension txt
function group
    arg literal "group"
    arg capture name ident
    block_closer end
    write `[${name}: ${body}]`
end
function item
    arg literal "item"
    arg capture v ident
    write `(${v})`
end
function end
end
```

```
# script         # output
group box         [box: (a)(b)]
  item a
  item b
end
```

### 25. `block_dedent` — body until the indent drops

No explicit closer — the body is everything more-indented than the
opener (used in example 23). Great for Python-like off-side rule
blocks.

### 26. `block_open "X" close "Y"` — delimiter-bounded body

```capy
extension txt
function obj
    arg literal "obj"
    arg capture name ident
    block_open "{" close "}"
    write `${name}={${body}}`
end
function kv
    bare
    arg capture k ident
    arg capture v any
    write `${k}:${v};`
end
```

```
# script         # output
obj point {       point={x:1;y:2;}
  x 1
  y 2
}
```

### 27. `block_verbatim NAME` — body as raw bytes (no parsing)

The body is **not parsed as Capy** — it's captured as raw source text,
exactly as written, including blank lines and lines that look like
comments. Ideal for code blocks:

```capy
extension html
function pre
    arg capture lang ident
    block_verbatim end
    write `<pre><code class="language-${lang}">${escapeHtml body}</code></pre>
`
end
function end
end
```

```
# script                          # output
pre go                             <pre><code class="language-go">func main() {
    func main() {                      fmt.Println(&quot;hello &amp; welcome &lt;world&gt;&quot;)
        fmt.Println("hello & welcome <world>")  }
    }                              </code></pre>
end
```

(The `&`, `<`, `>`, `"` are HTML-escaped by `escapeHtml`; indentation
and the literal source survive untouched.) See
[`samples/verbatim-pre/`](https://github.com/olivierdevelops/capy/tree/main/samples/verbatim-pre).

### 28. `block_sections A B … closer NAME` — multi-part block

One block with named sections; each section's rendered output is a
separate render-local. A try/rescue/finally:

```capy
function try
    arg literal "try"
    block_sections rescue finally closer end
    write `try:
${body}rescue:
${rescue}finally:
${finally}`
end
function end
end
```

```
# script              # output
try                    try:
    step "open file"     - open file
rescue                 rescue:
    step "log error"     - log error
finally                finally:
    step "close file"    - close file
end
```

The lead section is `${body}`; each named section (`rescue`,
`finally`) is its own local. See
[`samples/error-handling/`](https://github.com/olivierdevelops/capy/tree/main/samples/error-handling).

---

## Stage ⑥ — Matched-pair close sequences (`block_close_seq`)

`block_close_seq` lets a block close on a **multi-token sequence**,
either fixed literals or — the powerful part — a sequence that
*references one of the opener's captures*. This is how Capy parses real
tag trees with one function.

### 29. Literal close sequence

`block_close_seq "[" "/b" "]"` closes a BBCode `[b]…[/b]` on the exact
token run `[`, `/b`, `]`.

### 30. Capture-bound close sequence — the matched pair

```capy
extension html
function element
    arg literal "<"
    arg capture name ident
    arg capture attrs attribute*
    arg literal ">"
    block_close_seq "</" name ">"
    write `<${name}${attrs}>${body}</${name}>`
end
function attribute
    arg capture key ident
    arg literal "="
    arg capture val raw
    write ` ${key}="${val}"`
end
function text
    bare
    arg capture s raw
    write `${escapeHtml s}`
end
```

The closer `</` `name` `>` reuses the captured tag `name`, so `<div>`
closes **only** on `</div>`. One generic function parses *any*
well-formed tag tree:

```
# script
<article class="post" id="p1"><h1>"Capy parses HTML"</h1><p>"One generic function, "<b>"any"</b>" tag."</p></article>

# output
<article class="post" id="p1"><h1>Capy parses HTML</h1><p>One generic function, <b>any</b> tag.</p></article>
```

### 31. Mismatched nesting is a hard error

Because each opener demands *its own* close tag, bad nesting can't
silently pass:

```
# script                                      # error
<div class="card"><p>"never closed"</div>     1:36: no library function matches token "</"
```

This is genuine structural parsing — not a regex that happens to match
angle brackets. See
[`samples/html-xml-parser/`](https://github.com/olivierdevelops/capy/tree/main/samples/html-xml-parser).

---

## Stage ⑦ — Repetition and nonterminals

A capture's type can be **another function's name** — a *nonterminal*.
Add a repetition marker (`*` zero-or-more, `+` one-or-more) to match a
list, and `sep` / `join` to control separators.

### 32. Function-typed repetition (`attribute*`)

In example 30, `arg capture attrs attribute*` matched zero or more
`attribute` nonterminals (`class="post" id="p1"`), each rendered by the
`attribute` function and concatenated into `${attrs}`.

### 33–34. `sep` (input) and `join` (output) are independent

`sep` is the separator **consumed while parsing** the input; `join` is
the separator **inserted between rendered results**. They need not be
the same character:

```capy
extension txt
function params
    bare
    arg capture ps param* sep "," join ", "
    write `(${ps})`
end
function param
    bare
    arg capture name ident
    write `${name}`
end
```

```
# script         # output
a,b,c             (a, b, c)
```

Input is comma-separated (`sep ","`); output joins with comma-space
(`join ", "`). A signature DSL (`param* sep "," join ", "`) reads
`a,b,c` and emits `a, b, c`. See
[`samples/signature-parser/`](https://github.com/olivierdevelops/capy/tree/main/samples/signature-parser).

---

## Stage ⑧ — Custom types

A `type` block defines a **validated** capture type via `pattern`
(regex), `options` (enum), or `base` (delegate to a built-in). Captures
typed with it are checked at parse time.

```capy
extension txt
type Email
    pattern "^[^@]+@[^@]+\\.[^@]+$"
end
type Status
    options "todo" "in-progress" "done"
end

function set_email
    arg capture e Email
    write `email = ${e}
`
end
function set_status
    arg capture s Status
    write `status = ${s}
`
end
```

### 35. `pattern` — regex validation

```
set_email "alice@example.com"   email = alice@example.com
```

### 36. `options` — enum validation

```
set_status "todo"               status = todo
```

### 37. Validation failure is a clear error

```
# script                        # error
set_email "not-an-email"        function "set_email" arg "e": value "not-an-email" does not match pattern for type "Email"
```

### 38. `base` — narrow a built-in type

`base ident` / `base string` etc. inherits the built-in's matching, so
a type can add a `pattern` *on top of* a base capture shape. See
[`samples/types/`](https://github.com/olivierdevelops/capy/tree/main/samples/types)
and
[`samples/typed-config-dsl/`](https://github.com/olivierdevelops/capy/tree/main/samples/typed-config-dsl).

---

## Stage ⑨ — Group types: inline syntax

A type with `group_open` / `group_close` captures everything between a
pair of delimiters — including nested pairs — as one value. This gives
Markdown/LaTeX-style inline syntax. The greedy punct-merge (stage ①)
means symmetric multi-char delimiters like `**` Just Work.

```capy
extension html
type Bracketed
    group_open  "["
    group_close "]"
end
type Parens
    group_open  "("
    group_close ")"
end
type Bold
    group_open  "**"
    group_close "**"
end

function link
    arg literal "link"
    arg capture text Bracketed
    arg capture url  Parens
    write `<a href="${escapeHtml url}">${escapeHtml text}</a>
`
end
function bold
    arg literal "bold"
    arg capture text Bold
    write `<strong>${escapeHtml text}</strong>
`
end
```

### 39. Two groups in a row, with nesting and escaping

```
# script                                       # output
link [Al the Alien](https://alien.com/1)        <a href="https://alien.com/1">Al the Alien</a>
link [nested [inner] brackets](https://e.com/p) <a href="https://e.com/p">nested [inner] brackets</a>
```

Nested `[inner]` brackets are balanced inside the group; the URL group
follows immediately.

### 40. Symmetric multi-char delimiters

```
bold **important text**       <strong>important text</strong>
```

`**` lexes as a single token (punct-merge), so `group_open "**"` /
`group_close "**"` bound the bold span. See
[`samples/inline-markdown/`](https://github.com/olivierdevelops/capy/tree/main/samples/inline-markdown).

---

## Render-locals: values you didn't capture

During stage ④ the template can read **render-locals** the engine
injects automatically (a same-named capture shadows them):

| Local | Meaning |
|-------|---------|
| `${body}` | rendered inner output of a block (stages ⑤/⑥) |
| `${rescue}`, `${finally}`, … | named sections of a `block_sections` block |
| `${depth}` | block nesting depth (0 at top level) |
| `${top_level}` | `true` when not inside any block |
| `${line}` | source line number of this statement |
| `${col}` | source column of this statement |

So a library can stamp positions for a source↔output map:

```capy
function p
    arg capture text tail
    write `<p data-line="${line}">${escapeHtml text}</p>`
end
```

…emits `<p data-line="7">…</p>` for the statement on source line 7 —
useful for editor scroll-sync without the engine knowing it's HTML.

---

## Putting it together

Every Capy library is just these pieces composed:

1. **Lex** the source into tokens — no keywords, greedy punctuation,
   indentation as a width stack.
2. **Match** each line to a function by priority + literal specificity,
   the function name auto-prepended unless `bare`.
3. **Capture** the variable pieces with the type whose appetite fits —
   one token (`ident`/`raw`), a run (`word`), the rest (`tail`), an
   expression (`any`), a delimited group (group type), a list
   (nonterminal `*`/`+`), or a validated custom type.
4. **Render** through `write`/`template`, substituting captures and
   render-locals; blocks recurse and feed `${body}`.

Because *none* of this is hard-coded to a target language, the same
mechanics emit Python, SQL, HTML, YAML, assembly, or anything else —
the template alone decides the output. Browse
[the samples](https://github.com/olivierdevelops/capy/tree/main/samples)
to see all 40+ behaviors above in production-sized libraries.

### See also

- [Built-in function cookbook](function-cookbook.md) — every template
  helper (`escapeHtml`, `decoded`, `indent`, `join`, …) with signatures.
- [Library keyword cookbook](library-keywords.md) — every authoring
  directive (`arg`, `block_*`, `type`, `when_*`, …) in reference form.
- [Language reference](language-reference.md) and
  [Library authoring](library-authoring.md) — the full specification.
