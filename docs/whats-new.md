---
title: What's new — engine primitives shipped in this release
---

# What's new

## 0.21.0 — parser foundations

**A left-recursive library no longer crashes the process.** A rule whose first
element captured a function that led back to it passed `capy check` and then died
with a stack overflow — which a program embedding Capy could not catch. It is now
refused when the library loads, with the cycle named. Nonterminal descent is also
bounded at 64 levels, so deeply nested *input* can no longer abort either.

**Every AST node carries a source span.** `FuncCall` and `CaptureValue` gained a
`Span` with start and end line/column. Nodes from function-as-type captures used
to report `line: 0, col: 0`; they now report where they actually are, so tooling
can point at the argument a problem is about rather than the whole statement.
`${line}` / `${col}` are unchanged.

**Comments are retained.** A script's comments survive lexing, and those
immediately above a statement are attached to it. A node's own span excludes its
comments, so a formatter can have both. Retention changes nothing about what
parses or what is rendered.

Nothing existing changes: all 117 sample libraries load, and the golden corpus is
byte-identical.


## Inner DSL — list & map elements are readable by index

A `[<expr>]` index may now appear in a **read** position, mirroring the
write target that already existed. Inside both inner-DSL value position
and `${…}` templates:

```
if context.readsince[loc] == 0      # map read by captured key
    set context.buf[context.laststore[loc]] `… eliminated`
end
write `${context.buf[i]}`           # list read by loop index
${context.grid[i][j]}               # nested
${context.rows[i].name}             # index then field
${context.buf[(sub n 1)]}           # computed index expression
${context.buf[-1]}                  # negative → last
```

Map parents key on the index's string form; list parents key on an
integer (negative from the end) — read and write semantics are
identical, so a value written by `set context.buf[i] …` reads back by the
same index. Missing keys / out-of-range indices are `nil` (falsy) in
value position and render empty in a template, so `if context.seen[k]`
guards without a `for`-scan. This retires the
`for k,v in … / if k == name` linear-lookup idiom: the new
[`samples/value-index-read/`](https://github.com/olivierdevelops/capy/tree/main/samples/value-index-read)
rewrites the dead-store eliminator's nested scans as two direct indexed
reads and produces byte-identical output. Design notes:
[value-position indexing](design/value-position-indexing.md).

Ten new samples show the idiom across map-by-key, list-by-index,
computed and negative indices — all in the playground under the
**Indexed reads** category:
[`word-frequency`](https://github.com/olivierdevelops/capy/tree/main/samples/word-frequency)
(map increment),
[`memo-fib`](https://github.com/olivierdevelops/capy/tree/main/samples/memo-fib)
(computed index),
[`stack-top`](https://github.com/olivierdevelops/capy/tree/main/samples/stack-top)
(negative index),
[`enum-lookup`](https://github.com/olivierdevelops/capy/tree/main/samples/enum-lookup)
(list by index),
[`leaderboard`](https://github.com/olivierdevelops/capy/tree/main/samples/leaderboard)
(positional reads),
[`color-palette`](https://github.com/olivierdevelops/capy/tree/main/samples/color-palette)
(map by name),
[`symbol-types`](https://github.com/olivierdevelops/capy/tree/main/samples/symbol-types),
[`register-allocator`](https://github.com/olivierdevelops/capy/tree/main/samples/register-allocator),
[`grid-game`](https://github.com/olivierdevelops/capy/tree/main/samples/grid-game)
and
[`emoji-react`](https://github.com/olivierdevelops/capy/tree/main/samples/emoji-react).
See the [🔢 Indexed reads showcase](showcase.md#indexed-reads-lists-maps-read-back-by-index-or-key)
for the full gallery.

## Inner DSL — list elements are writable by index

`set context.buf[i] value` now overwrites a **list** element in place
(previously only map keys accepted a `[…]` write target; a list parent
errored with "target parent is not a map"). `append`/`prepend` against an
indexed target reach the nested list at that element, and negative
indices count from the end (`-1` = last).

This unlocks **retroactive rewrites of buffered output** — the pattern a
code generator uses when a decision can't be made until later. Buffer the
instruction stream in a `context` list, keep side-tables as you go, and
then edit a past slot once the future is known: null out a dead store,
back-patch a jump offset, or drop a redundant reload. The new
[`samples/list-index-assign/`](https://github.com/olivierdevelops/capy/tree/main/samples/list-index-assign)
is a working **dead-store eliminator** built entirely in-library this
way, and the [cross-arch assembler notes](design/asm-backend-remaining.md)
use it to show that "looking back at already-emitted code" is
bookkeeping, not whole-program reasoning.

## Helpers — integer `div` / `mod` / `align`

Three new numeric template helpers join `add`/`sub`/`mul`:

- `div a b` — integer quotient `a / b` (returns `0` when `b == 0`).
- `mod a b` — remainder `a % b` (returns `0` when `b == 0`).
- `align n a` — round `n` **up** to the next multiple of `a`
  (`(n + a - 1) / a * a`).

`align` is the op a code generator needs for ABI-correct layout: struct
field offsets, struct total size, and 16-byte stack-frame rounding all
reduce to it. Previously the only arithmetic helpers were
`add`/`sub`/`mul`/`percent`, so alignment rounding (which needs division
or bitwise AND) couldn't be expressed in-library — `align` closes that
gap. See the [function cookbook](function-cookbook.md#div-mod-align) and
the [cross-arch assembler design notes](design/asm-backend-remaining.md),
where this unblocks Problems 6 (struct layout) and 7 (stack frames).

## Docs — how Capy parses & extracts content

A new end-to-end walkthrough,
[How Capy parses & extracts content](how-capy-parses.md), explains the
four-stage pipeline (lex → match → capture → render) and showcases it
with 40+ runnable examples, building from a one-line literal grammar up
to matched-pair tag parsing, nonterminal repetition with independent
`sep`/`join`, custom types, and group-type inline syntax. Every output
shown is produced by the engine, and the trickier behaviors (greedy
`tail`, the auto-name-prepend rule, `when_followed_by indent`,
capture-bound `block_close_seq`) are grounded in CI-checked samples.

## Docs — library keyword cookbook

A companion to the function cookbook: the
[Library keyword cookbook](library-keywords.md) documents every keyword
you write *inside* a `.capy` library — `function`, `arg literal`,
`arg capture` (with `sep`/`join`/`default`), all six `block_*` body
openers, `write`/`template`, the file-level directives (`extension`,
`context`, `comments`, `command`, `import`, `file`, …), the built-in
capture types, and `type` definitions. It also answers "how do I know a
function can have a body?" — it declares exactly one `block_*` directive.
The canonical `if … end` example is verified by
[`samples/library-keywords/`](https://github.com/olivierdevelops/capy/tree/main/samples/library-keywords).

## Docs — built-in function cookbook

Every built-in template helper (the functions you call inside `${ … }`)
now has a single reference page:
[Built-in function cookbook](function-cookbook.md). All 29 helpers —
`indent`, `pascalCase`/`camelCase`/`snakeCase`, `dasherize`, `unquote`,
`decoded`, `escapeHtml`, `toQuoted`, `asString`, `unescape`,
`trimSuffix`/`trimPrefix`, `join`/`split`/`nonEmpty`, `toJSON`/
`toJSONIndent`/`toPyLit`, `add`/`sub`/`mul`/`div`/`mod`/`align`,
`percent`, `stars`, `lower`/`upper` — with a worked example each. The examples are mirrored by
the CI-checked [`samples/builtin-functions/`](https://github.com/olivierdevelops/capy/tree/main/samples/builtin-functions)
sample, also runnable in the playground.

## Round 6 — matched-pair HTML: sequence closers & named nonterminals

Two composing grammar primitives that together make well-formed,
matched-pair markup (HTML/XML) authorable from a single library function.

**`block_close_seq` — multi-token sequence closers.** A block can now
close on an exact *run* of tokens rather than a single keyword or
delimiter. The closer is declared as a list of segments — each a quoted
literal or a bare **capture-name ref** bound by the opener:

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

The ref makes the closer depend on the opener, so one `element` function
covers every paired tag: `<div>` closes only on `</div>`, `<p>` only on
`</p>`, and mismatched nesting (`<div><p></div>`) is a hard parse error.
Inside the body, layout is insignificant — structure comes from the tags.

**Function-as-type captures (named nonterminals).** A capture's type may
name another library function. The capture matches that function's shape
and renders its template. Add `*` / `+` to repeat, and `sep "X"` for a
separator literal:

```
arg capture attrs attribute*          # zero or more attributes
arg capture items cell+ sep ","       # one or more, comma-separated input
arg capture params param* sep "," join ", "   # comma input, ", " output
```

`sep` is the **input** separator (consumed between repetitions while
parsing); `join` is the **output** separator (inserted between the rendered
sub-results). They're independent — `sep "," join ", "` parses
comma-separated input and renders a comma-**space**-separated list. A
pure-capture nonterminal (one with no leading keyword, like `param NAME
TYPE`) must be declared `bare`, otherwise Capy auto-prepends the function
name as a required keyword.

Both primitives are additive — no existing library changes behaviour.

Four runnable demos, all in the [playground](playground.md) under
**✨ Features**:

| Demo | Shows |
|---|---|
| [`html-xml-parser`](https://github.com/olivierdevelops/capy/tree/main/samples/html-xml-parser) | One generic `element` parses any `<tag>` — capture-bound `</NAME>` closer + `attribute*` |
| [`bbcode-parser`](https://github.com/olivierdevelops/capy/tree/main/samples/bbcode-parser) | Same primitive, **bracket** delimiters: `[b]…[/b]` with literal close-seqs |
| [`markdown-from-tags`](https://github.com/olivierdevelops/capy/tree/main/samples/markdown-from-tags) | The same tag markup rewritten to **Markdown** — target-agnostic |
| [`signature-parser`](https://github.com/olivierdevelops/capy/tree/main/samples/signature-parser) | Param lists via `param* sep "," join ", "` |

Tests: [`functype_test.go`](https://github.com/olivierdevelops/capy/blob/main/functype_test.go),
[`multitoken_closer_test.go`](https://github.com/olivierdevelops/capy/blob/main/multitoken_closer_test.go).

---

## Round 5 — `tail` preserves quoted slots

A follow-up to the `tail` capture (Round 3, §6). Previously `tail`
rebuilt its value from de-quoted tokens, so a spaced, quoted argument
lost its slot boundary: `exec git commit -m "fix the bug"` collapsed to
`commit -m fix the bug`, indistinguishable from four separate tokens.

Now quoted tokens are re-emitted **with** their quotes, and inter-token
spacing is computed from each token's true source width (which counts
the quotes). So the boundary survives:

```
exec git commit -m "fix the bug"   →  argv = git commit -m "fix the bug"
```

A quote-aware split on the value now recovers `"fix the bug"` as one
argument. Bare flags/paths/globs are unchanged, and no existing library
breaks (only quoted tokens inside a `tail` change, and none of the
samples relied on the old quote-stripping). This lets a single `tail`
function replace a hand-written `word`-ladder for shell-style argv.

Tests: [`missing_features_test.go`](https://github.com/olivierdevelops/capy/blob/main/missing_features_test.go) (`TestTailPreservesQuotedSlots`).

---

## Round 4 — context-sensitive grammar & multi-section blocks

The last two open items from the automation-DSL audit — both grammar
features rather than tokenizer fixes. Additive and opt-in:

| Feature | What it gives you |
|---|---|
| `when_followed_by indent` / `when_not_followed_by indent` | **Context-sensitive keyword reuse.** A function matches only when an indented block does (or doesn't) follow, so a flat `os "linux"` allowlist entry and an `os "…" … end` conditional block can share the one `os` keyword and disambiguate purely by position — no rename needed. |
| `block_sections <S>... closer <NAME>` | **Multi-section blocks** — `try … rescue … finally … end`. The main body and each interior section render independently and land in the template as `${body}`, `${rescue}`, `${finally}`. Omitted sections render empty; order and subset are free. |

```
function try
    arg literal "try"
    block_sections rescue finally closer end
    write `try { ${body} } rescue { ${rescue} } finally { ${finally} }`
end
```

Tests: [`context_blocks_test.go`](https://github.com/olivierdevelops/capy/blob/main/context_blocks_test.go).
Sample: [`samples/error-handling/`](https://github.com/olivierdevelops/capy/tree/main/samples/error-handling).

---

## Round 3 — automation-DSL parser hardening

A third wave, driven by building a shell-/automation-style DSL on Capy
(command runners, `exec` surfaces, config languages with member access).
These close concrete parser and tokenizer gaps. All additive, all opt-in:

| Feature | What it gives you |
|---|---|
| **Block backtracking** | When a function matches a block opener's header but no body follows, the parser now backtracks and tries the next candidate — so a flat function and a block function can safely share a leading keyword (e.g. `os "linux"` as a flat entry vs `os "…" … end` as a conditional block). Previously it committed to the block and errored. |
| **Deterministic candidate ordering** | The function-match order is now total (priority → literal-start → literal-length → **name**). Functions that tie on all other keys no longer inherit Go's randomized map order — eliminating run-to-run "parses 50% of the time" heisenbugs on keyword collisions. |
| `word` capture type | A shell-style bare word: `--oneline`, `-f`, `k8s/deploy.yaml`, `name=^web$`, `restart-api` capture as ONE value despite the lexer splitting on `-`/`/`/`=`/`.`. |
| `dotted_ident` capture type | `match err.kind` works bare — captures the dotted path as one string instead of needing the `"${err.kind}"` workaround. |
| `${asString x}` helper | Emits exactly one valid JSON string, quoting iff the capture isn't already a string. `exec echo foo` and `exec echo "foo"` both interpolate correctly — no more `${toJSON}` double-quoting or bare-ident invalid JSON. |
| **Source-level metaprogramming** (`define … end`) | A script can declare its own DSL functions inline — same grammar as a library `function` — then use them in the same file. The library stays minimal; power users extend the vocabulary without forking it. Embedded callers get the same behaviour through `capy.Library.Run` / `RunMulti`. |

Tests: [`missing_features_test.go`](https://github.com/olivierdevelops/capy/blob/main/missing_features_test.go).

**Metaprogramming in one breath.** The source below ships with a library
that defines *only* `print` — `heading`, `todo`, and `quote` are declared
inline with `define … end` and then used immediately:

```
define heading
    arg literal "heading"
    arg capture text string
    write `# ${unquote text}
`
end

heading "Capy metaprogramming"
todo yes "Ship metaprogramming feature"
```

Full worked example: [`samples/metaprogramming/`](https://github.com/olivierdevelops/capy/tree/main/samples/metaprogramming)
· pattern docs: [metaprogramming.md](metaprogramming.md)
· runnable in the [playground](playground.md) under **🧬 Metaprogramming**.

---

## Round 2 — editor integration & authoring ergonomics

A second wave, driven by building a real product on Capy (a
46-component library + a live editor with highlight / autocomplete /
hover-docs / scroll-sync). All additive, all opt-in:

| Feature | What it gives you |
|---|---|
| `${line}` / `${col}` render locals | Source↔output mapping. Stamp `data-capy-line="${line}"` and the host editor does `querySelector('[data-capy-line="N"]')` for scroll-sync / inline errors — no source mutation. |
| [`Library.Introspect()`](embedding.md#introspection-the-library-describes-itself) + `capyIntrospect()` / `pagesIntrospect()` | Declared functions, args, capture types, optional/default flags, and `description` doc strings — so an editor derives its autocomplete / hover / highlight metadata instead of hand-maintaining a parallel catalogue. |
| `${decoded x}` handles embedded quotes | `<div class="x">\nmore` round-trips — quotes preserved, newlines real. No host-side decoder needed. |
| Column-0 lines inside `template … end` | A flush-left `${indent 2 body}` no longer truncates the function body. |
| Verbatim raw-byte fidelity | `block_verbatim` preserves blank lines and `#`-comment lines exactly — unblocks a real `markdown … end` block. |
| Backtick code spans in captures | `` markdown `inline \`code\` here` `` keeps the span. |
| Optional args with defaults | `arg capture variant string default "primary"` — collapses `button` / `button_link` / `submit` families into one function. |

Samples: [`line-mapping`](https://github.com/olivierdevelops/capy/tree/main/samples/line-mapping),
[`backtick-codespan`](https://github.com/olivierdevelops/capy/tree/main/samples/backtick-codespan),
[`optional-args`](https://github.com/olivierdevelops/capy/tree/main/samples/optional-args),
plus extended [`string-decoded`](https://github.com/olivierdevelops/capy/tree/main/samples/string-decoded),
[`template-sugar`](https://github.com/olivierdevelops/capy/tree/main/samples/template-sugar),
[`verbatim-pre`](https://github.com/olivierdevelops/capy/tree/main/samples/verbatim-pre).

---

## Round 1 — prose-heavy DSL substrate

This release closes a six-stage roadmap focused on making Capy a
viable substrate for **prose-heavy DSLs** (markdown-like authoring
tools, blog/notes engines, technical docs, AI-emitted documents).
Every feature here is opt-in — existing libraries keep working
unchanged; nothing is deprecated.

The lineup, ordered by impact-per-line-of-engine-change:

| Stage | Feature | Solves |
|---|---|---|
| 1 | UTF-8 in bare prose | Em-dashes, accents, CJK, emoji no longer crash the lexer |
| 2 | `block_verbatim` directive | Code blocks, embedded HTML, anywhere a body is data not grammar |
| 3 | `${decoded x}` helper | Quotes-in-prose round-trip — `"He said \"hi\""` → `He said "hi"` |
| 4 | `${escapeHtml x}` helper | XSS surface closed for HTML-emitting libraries |
| 5 | Multi-line backticks in scripts | Heredoc-style prose, multi-line paragraphs |
| 6 | `group_open` / `group_close` types | First-class inline syntax — `[label](url)`, `**bold**`, `~~strike~~` |
| 7 | `template … end` write-literal sugar | Multi-line HTML/text templates without backtick bookkeeping |

Plus a few smaller correctness fixes (source-absolute column tracking,
`#` and `\` accepted as punctuation tokens, the renamed
`${escapeHtml x}` helper) that all the above build on.

---

## A flagship demo: math equation plots

Every primitive composes in [`samples/math-plots/`](https://github.com/olivierdevelops/capy/tree/main/samples/math-plots).
The library takes a one-line DSL —

```
title "A few familiar shapes"

plot "sin(x)"
    domain -6.28 6.28
    color "#4ef"
end

plot "exp(-x*x)"
    domain -3 3
    color "#bf4"
    samples 400
end
```

— and produces a self-contained HTML page with canvas plots and a
small inline plotter. **Live preview** (rendered by the actual
library committed to this repo):

<iframe src="../assets/demos/math-plots.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; height: 540px; border: 0; border-radius: 12px; box-shadow: 0 12px 40px rgba(0,0,0,0.18); display: block; margin: 18px 0 24px;" title="Math plots rendered live from a Capy source"></iframe>

The library uses every new primitive in concert:

- **`template … end` sugar** for the multi-line HTML literals (the
  page template AND the per-plot `<figure>` block);
- **`${escapeHtml (decoded expr)}`** to render the expression
  safely in the figure caption — `sin(x*<thing>)` won't inject
  markup;
- **`${decoded expr}`** to recover the user-intended expression
  for JS evaluation;
- **context accumulation** (`append context.plots …`) tracks every
  plot for an end-of-page summary;
- **UTF-8 in prose** for the page title and captions (works with
  Greek letters, em-dashes, accented descriptions out of the box);
- A `command "run"` block compiles the script, writes the HTML
  next to it, and opens the default browser — `capy math-plots run
  page.plots` is the entire workflow.

If you have the repo:

```sh
cd samples/math-plots
capy run lib.capy script.capy > plots.html && open plots.html
```

---

## Copy-paste snippets for each new primitive

Want to see each feature in isolation? The snippets below all
compile against today's `capy` and produce the output shown.

### `template … end` sugar

```
function card
    arg literal "card"
    arg capture title string
    block_closer end
    template
        <div class="card">
          <h3>${escapeHtml (decoded title)}</h3>
          <div>${indent 2 body}</div>
        </div>
    end
end

function p
    arg literal "p"
    arg capture text string
    template
        <p>${escapeHtml (decoded text)}</p>
    end
end

function end
end
```

Input:

```
card "Hello"
    p "First line of body."
    p "Second line, with <markup> & quotes."
end
```

Output:

```html
<div class="card">
  <h3>Hello</h3>
  <div>  <p>First line of body.</p>
  <p>Second line, with &lt;markup&gt; &amp; quotes.</p>
</div>
</div>
```

### `block_verbatim` — raw code blocks

```
function pre
    arg capture lang ident
    block_verbatim end
    template
        <pre><code class="language-${lang}">${escapeHtml body}</code></pre>
    end
end

function end
end
```

Input:

```
pre go
    func main() {
        fmt.Println("hello & world")
    }
end
```

Output:

```html
<pre><code class="language-go">func main() {
    fmt.Println(&quot;hello &amp; world&quot;)
}
</code></pre>
```

### Group types — `[label](url)`, `**bold**`

```
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
    template
        <a href="${escapeHtml url}">${escapeHtml text}</a>
    end
end

function bold
    arg literal "bold"
    arg capture text Bold
    template
        <strong>${escapeHtml text}</strong>
    end
end
```

Input:

```
link [Al the Alien](https://example.com/alien)
bold **important text**
```

Output:

```html
<a href="https://example.com/alien">Al the Alien</a>
<strong>important text</strong>
```

### Multi-line user-script backticks + `${decoded …}`

```
function p
    arg literal "p"
    arg capture text string
    template
        <p>${escapeHtml (decoded text)}</p>
    end
end
```

Input — a single `p` call spans three source lines:

```
p `This is
a multi-line paragraph
written with backticks.`
```

Output:

```html
<p>This is
a multi-line paragraph
written with backticks.</p>
```

### UTF-8 in bare prose

```
function prose_line
    bare
    arg capture content tail
    template
        <p>${content}</p>
    end
end
```

Input:

```
Each line — yes, with em-dashes — becomes a paragraph.
Café au lait with naïve crème brûlée
北京 上海 东京
🎉 emoji works too 🚀
```

Output:

```html
<p>Each line — yes, with em-dashes — becomes a paragraph.</p>
<p>Café au lait with naïve crème brûlée</p>
<p>北京 上海 东京</p>
<p>🎉 emoji works too 🚀</p>
```

### Source↔output mapping with `${line}` / `${col}`

Every statement knows its 1-indexed source position. Stamp it onto
emitted elements and a host editor can scroll-sync the preview to the
cursor, or underline the exact line that failed — with a single
`querySelector`, no source mutation, working inside every region.

```
function p
    arg literal "p"
    arg capture text string
    template
        <p data-capy-line="${line}">${escapeHtml (decoded text)}</p>
    end
end
```

Input:

```
p "First paragraph."
p "Second paragraph."
p "Third paragraph."
```

Output — each `<p>` carries the line it came from:

```html
<p data-capy-line="1">First paragraph.</p>
<p data-capy-line="2">Second paragraph.</p>
<p data-capy-line="3">Third paragraph.</p>
```

The editor then does `querySelector('[data-capy-line="2"]')`. A user
capture named `line`/`col` still wins — these are render locals, same
precedence as `body` / `depth` / `top_level`.

### Optional args with defaults — one function, many call shapes

A trailing capture can declare a `default`, so the call site may omit
it. One `button` collapses what used to be `button` / `button_link` /
`submit`:

```
function button
    arg literal "button"
    arg capture label   string
    arg capture variant string default "primary"
    arg capture kind    string default "button"
    template
        <button type="${decoded kind}" class="btn-${decoded variant}">${escapeHtml (decoded label)}</button>
    end
end
```

Input:

```
button "Save"
button "Delete" "danger"
button "Submit" "primary" "submit"
```

Output:

```html
<button type="button" class="btn-primary">Save</button>
<button type="button" class="btn-danger">Delete</button>
<button type="submit" class="btn-primary">Submit</button>
```

A required capture after an optional one is a load-time error
(optional args must be trailing).

### The library describes itself — `Introspect()`

Point a tool at a library and it reports every function's shape —
including which args are optional and their defaults — so an editor
builds autocomplete / hover-docs / highlighting from one source of
truth:

```go
for _, fn := range lib.Introspect() {
    fmt.Println(fn.Name, "-", fn.Description)
    for _, a := range fn.Args {
        if a.Kind == "capture" {
            fmt.Printf("  %s: %s (optional=%v default=%q)\n",
                a.Name, a.Type, a.Optional, a.Default)
        }
    }
}
```

The identical JSON shape is available in the browser via
`capyIntrospect(librarySrc)` / `pagesIntrospect()`. Full struct
reference and a runnable example: [Introspection in
embedding.md](embedding.md#introspection-the-library-describes-itself).

---

## 1. UTF-8 in bare prose

**Before:** the lexer walked the source byte-by-byte, so any
non-ASCII character (em-dash, accented Latin, CJK, emoji) crashed
with `unexpected character`. To get prose with Unicode through, you
had to wrap every line in `"…"` — which then needed escape
unwrapping at render time. The Pages preprocessor was doing exactly
this.

**Now:** the lexer decodes UTF-8 properly. Any rune that's a Letter
(or any non-ASCII rune at all) is accepted as part of an identifier
token. A `bare + tail` catch-all function happily reassembles
arbitrary prose:

```
function prose_line
    bare
    arg capture content tail
    write `<p>${content}</p>
`
end
```

Source — straight from `samples/utf8-prose/script.capy`:

```
Each line — yes, with em-dashes — becomes a paragraph.
Café au lait with naïve crème brûlée
北京 上海 东京
🎉 emoji works too 🚀
```

Output:

```html
<p>Each line — yes, with em-dashes — becomes a paragraph.</p>
<p>Café au lait with naïve crème brûlée</p>
<p>北京 上海 东京</p>
<p>🎉 emoji works too 🚀</p>
```

ASCII-only sources tokenise identically to before; this is a strict
superset.

---

## 2. `block_verbatim` — raw bodies, no nested parsing

**Before:** every block mode (`block_closer`, `block_open … close …`,
`block_dedent`) re-parsed the body as nested Capy statements. A code
block like `pre go { func main() { fmt.Println("hi") } }` would
fail because `func main() {` doesn't match any library function.
Workaround: every line had to be wrapped in `raw "…"` calls.

**Now:** declare `block_verbatim NAME` and the body is captured as
**raw source bytes** until the named closer keyword appears. The
captured text reaches your template as `${body}`.

```
function pre
    arg capture lang ident
    block_verbatim end
    write `<pre><code class="language-${lang}">${escapeHtml body}</code></pre>
`
end

function end
end
```

Source — from `samples/verbatim-pre/script.capy`:

```
pre go
    func main() {
        fmt.Println("hello & welcome <world>")
    }
end
```

Output (indent preserved, HTML-escaped for safety):

```html
<pre><code class="language-go">func main() {
    fmt.Println(&quot;hello &amp; welcome &lt;world&gt;&quot;)
}
</code></pre>
```

`block_verbatim` is the **fourth** block mode; the loader validates
that exactly one of the four is set per function:

| Directive | Body delimited by | Body parsed? |
|---|---|---|
| `block_closer NAME` | Indented body, ends at `NAME` keyword | Yes |
| `block_open "X" close "Y"` | Delimiter pair | Yes |
| `block_dedent` | First DEDENT (no closer keyword) | Yes |
| `block_verbatim NAME` | Indented body, ends at `NAME` keyword | **No — raw source bytes** |

---

## 3. `${decoded x}` — full escape round-trip in one helper

**Before:** capturing a `string` like `"He said \"hi\""` left two
levels of escaping in the captured text. `${x}` showed
`"He said \\\"hi\\\""`, `${unquote x}` stripped one level to
`He said \\\"hi\\\"`, and `${unescape x}` only ran one Unquote
pass — `He said \"hi\"`. No composition recovered the user-intended
form. Quotes inside prose simply didn't work.

**Now:** the `decoded` helper does the full round-trip:

```
function p
    arg literal "p"
    arg capture text string
    write `<p>${decoded text}</p>
`
end
```

Source:

```
p "He said \"hi\""
p "line1\nline2"
p "tab\there"
```

Output:

```html
<p>He said "hi"</p>
<p>line1
line2</p>
<p>tab	here</p>
```

`decoded` strips outer quotes (if present), then resolves Go-style
escape sequences (`\"`, `\n`, `\t`, `\\`, plus `\xNN` and `\uNNNN`).
If the source has doubled escapes (which happens when Capy stored
them as byte-preserved literals), a second decode pass catches them.

Compose with `escapeHtml` for safe-by-construction HTML:
`${escapeHtml (decoded text)}`.

This is **additive**: existing libraries that want the source-text
quoted form (YAML output, TypeScript string literals, Markdown
frontmatter) keep using `${x}` directly. `decoded` is opt-in.

---

## 4. `${escapeHtml x}` — close the XSS surface

**Before:** Capy shipped helpers for case conversion (`upper`,
`lower`, `pascalCase`), indentation (`indent N`), JSON (`toJSON`,
`toJSONIndent`), and quoting (`unquote`, `toQuoted`, `unescape`) —
but **no HTML escape**. Any `${user_value}` in an HTML-emitting
library was an XSS hole.

**Now:** the `escapeHtml` helper replaces the five characters every
HTML emitter has to neutralise — `& < > " '` — with their entity
references:

```
function p
    arg literal "p"
    arg capture text string
    write `<p>${escapeHtml text}</p>
`
end
```

Source:

```
p "Look at <script>alert('xss')</script>"
```

Output:

```html
<p>Look at &lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;</p>
```

The verbose name is deliberate: `${html x}` would read as "convert
this to HTML" instead of "escape this FOR HTML". `${escapeHtml x}`
makes the intent obvious at the call site.

Composes with `decoded` for the common pattern "decode user-string
escapes, then HTML-escape the result":

```
write `<p>${escapeHtml (decoded text)}</p>
`
```

---

## 5. Multi-line backticks in user scripts

**Before:** the docs said backtick strings span lines — and they did,
inside library `write` blocks (which run a separate body parser).
Inside **user scripts** the tokenizer treated backticks the same as
double quotes, closing them at end-of-line:

```
p `This is
a multi-line
paragraph.`
```

→ `line 1: unterminated string`. Workaround: type each line as a
separate `p "…"` call.

**Now:** the user-script lexer merges any line opening an unclosed
backtick with subsequent lines (using `\n` escapes for the
intervening newlines) until the closing backtick. Combine with
`${decoded text}` to recover the real newlines at render time:

```
function p
    arg literal "p"
    arg capture text string
    write `${decoded text}

`
end
```

Source — from `samples/multiline-strings/script.capy`:

```
p `This is
a multi-line
paragraph.`

p `Empty lines preserved:

middle blank.`
```

Output:

```
This is
a multi-line
paragraph.

Empty lines preserved:

middle blank.
```

The docs' multi-line-backtick promise now holds in user scripts
too.

---

## 6. `group_open` / `group_close` — inline syntax as a type

**Before:** Capy could parse statement-shaped DSLs cleanly, but it
had no way to express Markdown-style inline syntax. A line like
`link [Al the Alien](https://alien.com/1)` had no path through the
parser — `[` and `]` weren't delimiters that any function could
own.

**Now:** types can declare `group_open` / `group_close` directives.
A capture of that type consumes the open delimiter, walks tokens
(with balanced nesting and multi-line support) until the matching
close delimiter, and returns the joined source text:

```
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

Source — from `samples/inline-markdown/script.capy`:

```
link [Al the Alien](https://alien.com/1)
link [Click & view](https://example.com/<safe>)
link [nested [inner] brackets](https://example.com/path)
bold **important text**
```

Output:

```html
<a href="https://alien.com/1">Al the Alien</a>
<a href="https://example.com/&lt;safe&gt;">Click &amp; view</a>
<a href="https://example.com/path">nested [inner] brackets</a>
<strong>important text</strong>
```

Key properties:

- **Balanced nesting** — `[nested [inner] brackets]` captures the
  whole text including the inner brackets, because the parser
  tracks depth.
- **Multi-char delimiters work** — `**` is a single token thanks
  to the lexer's punct-greedy rule. Same for `~~`, `$$`, etc.
- **Multi-line groups** — a group can span newlines; the captured
  text contains the real newlines.
- **Mutually exclusive with constraint fields** — a type is either
  a group type OR a constraint type (`base`/`pattern`/`options`),
  never both.

Limitations called out in [types.md](types.md#group-types):

- The **backtick** (`` ` ``) collides with Capy's string-literal
  lexing; it can't currently be used as a group delimiter. Pick
  another delimiter (e.g. `~~code~~`).
- A **prose run that embeds inline calls** (`This is **important**
  text.`) needs a separate prose-scanner that's not part of the
  group-types primitive. For now, write inline calls on their own
  lines.

---

## 7. `template … end` — sugar for multi-line `write ` ... ` `

**Before**: long HTML / text templates carried three pieces of
ceremony per `write` — the opening backtick after `write`, the
trailing newline before the closer, the closing backtick on its own
line at arbitrary indent. Editors couldn't syntax-highlight the body
as HTML because of the wrapping backtick. Auto-indent fought the
closing backtick:

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

**Now**: the same function, no backtick bookkeeping:

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

`template … end` is **pure sugar** — the lib parser rewrites it into
the synthesised backtick `write` before anything downstream sees the
function body. Identical AST, identical render path, identical
output bytes. `${…}` interpolation is active (the one thing that
distinguishes this from `block_verbatim`, which is interpolation-OFF).

Properties:

- **Where it works** — anywhere a `write` is valid: function bodies,
  `file_template`, and `file "X" … end` blocks.
- **Composes with state mutations** — `template … end` is one
  statement among many in the function body. Pair it with `append
  context.x …` / `set` / `if` / `for` exactly the same way you would
  pair a `write`.
- **Auto-dedented** — body lines retain their relative indentation
  but the common leading whitespace is stripped, so the captured
  text starts flush-left.
- **Nested templates** at the same indent are balanced via depth
  tracking, so a `template … end` inside another `template … end`
  works.
- **Backticks and backslashes** inside the body are escaped
  automatically — paste HTML containing `` ` `` without escaping
  manually.

Adds one new keyword (`template`) at statement position. Any
existing library function literally named `template` would conflict
at call site — we grepped the in-tree samples and found none.

See [`samples/template-sugar/`](https://github.com/olivierdevelops/capy/tree/main/samples/template-sugar)
for a complete worked example with `card`, `p`, `file_template`, and
state accumulation in `context.cards`.

---

## Bonus: `#!/usr/bin/env -S capy --lib …` shebang scripts

Capy strips a leading `#!` line before lexing, so any `.capy` /
`.recipe` / `.greet` / `.<your-ext>` file with a shebang is
directly executable on Linux and macOS:

```
#!/usr/bin/env -S capy --lib greet
greet "world"
greet "from a shebang script"
```

```sh
chmod +x hello.greet
./hello.greet
# → Hello from greet, world!
#   Hello from greet, from a shebang script!
```

Works with the system `capy` binary or a standalone `capy build`
output. Four shebang forms with trade-offs (`env -S`, plain `env`,
absolute path, built-binary path) are documented in the
[compile cookbook §12](cookbook-compile.md#recipe-12-shebang-scripts-greet-files-that-run-themselves).
Built standalone binaries automatically suppress the
"library not on CAPY_LIBS" trust warning since the library is
embedded in the binary — `chmod +x script.greet` and ship it.

---

## Smaller correctness fixes that ship with the above

### Source-absolute column tracking

The lexer previously used two different column conventions: lines
outside brackets were tokenised after the indent was stripped (so
`fmt` on a `    func main()` line reported col 1), but lines inside
brackets kept their raw source column. `block_verbatim`
reconstruction needed consistent columns, so the lexer now passes a
`startCol` argument to `tokenizeLine` equal to `1 + stripped_count`.
Tokens carry source-absolute columns regardless of bracket state —
better error messages too.

### `#` and `\` accepted as punct tokens

These weren't in `punctChars` before, so a Python comment inside a
`block_verbatim` body or a LaTeX command (`\href{x}{y}`) would error
with `unexpected character`. Now they're tokens — neutral by
default; libraries that want them to mean something (e.g.
`comments line "#"`) opt in.

### `${html x}` renamed to `${escapeHtml x}`

The helper shipped briefly as `${html x}` but the name was
ambiguous ("make this HTML"? "treat this as HTML"?). Renamed to
`${escapeHtml x}` before reaching anyone outside this repo.

---

## Where to go from here

| You want to… | Read |
|---|---|
| Build a Markdown-like authoring DSL | [Group types in types.md](types.md#group-types) |
| Ship a code-block library | [Block functions in library-authoring.md](library-authoring.md#block-functions) |
| Safely emit HTML | [`escapeHtml` in templates.md](templates.md#escapehtml-string) |
| Handle quoted user strings cleanly | [`decoded` in templates.md](templates.md#decoded-string) |
| Build editor tooling from the library | [Introspection in embedding.md](embedding.md#introspection-the-library-describes-itself) |
| Map output back to source lines | [`${line}` / `${col}` in inner-dsl.md](inner-dsl.md) |
| See the full grammar | [language-reference.md](language-reference.md) |
| Drop into the LLM-facing brief | [CAPY_FOR_LLMS.md](CAPY_FOR_LLMS.md) |

Every new feature has a regression sample under `samples/`:

- [`samples/utf8-prose/`](https://github.com/olivierdevelops/capy/tree/main/samples/utf8-prose)
- [`samples/verbatim-pre/`](https://github.com/olivierdevelops/capy/tree/main/samples/verbatim-pre)
- [`samples/string-decoded/`](https://github.com/olivierdevelops/capy/tree/main/samples/string-decoded)
- [`samples/multiline-strings/`](https://github.com/olivierdevelops/capy/tree/main/samples/multiline-strings)
- [`samples/inline-markdown/`](https://github.com/olivierdevelops/capy/tree/main/samples/inline-markdown)
- [`samples/template-sugar/`](https://github.com/olivierdevelops/capy/tree/main/samples/template-sugar)
