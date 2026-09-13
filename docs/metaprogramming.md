---
title: Metaprogramming
hide:
  - toc
---

<div class="capy-hero" markdown>

<span class="capy-eyebrow">METAPROGRAMMING</span>

# Source files can extend their own grammar

A Capy source file isn't limited to the functions its library
declares. With `define NAME ... end` blocks, the **source itself**
can introduce new patterns mid-file. The rest of the source — and
any `@import`ed file — can then use them.

This is "macros, but typed" — every `define` is a real Capy function
with the same template, type-validation, and run-block machinery
as a library-defined one.

</div>

---

## A worked example

The library here is **deliberately minimal** — one function, `print`.
Everything else is declared inline by the source:

<div class="split" markdown>

<div markdown>

`lib.capy` (5 lines of useful content):

```
extension md

function print
    arg literal "print"
    arg capture text string
    write `${unquote text}

`
end
```

`script.capy` (extends the grammar with three new patterns):

```
define heading
    arg literal "heading"
    arg capture text string
    write `# ${unquote text}

`
end

define quote
    arg literal "quote"
    arg capture text string
    arg capture who string
    write `> ${unquote text}
>
> — *${unquote who}*

`
end

define checklist_item
    arg literal "todo"
    arg capture done ident
    arg capture text string
    if eq done "yes"
        write `- [x] ${unquote text}
`
    else
        write `- [ ] ${unquote text}
`
    end
end

heading "Today's todos"
todo yes "Ship metaprogramming"
todo no  "Update the docs"
quote "Description over implementation." "Capy"
```

</div>

<div class="visual" markdown>

Output:

<div class="browser-frame">
  <div class="chrome">
    <span class="lights"><span class="r"></span><span class="y"></span><span class="g"></span></span>
    <span class="url">output.md (rendered)</span>
  </div>
<pre style="background:white;color:#1f2328;padding:18px;line-height:1.6;font-family:'-apple-system','Segoe UI',Helvetica,Arial,sans-serif;font-size:13px;height:auto;"><strong style="font-size:18px;">Today's todos</strong>

- [x] Ship metaprogramming
- [ ] Update the docs

<em style="color:#57606a;border-left:3px solid #d0d7de;padding-left:10px;display:block;margin-top:8px;">Description over implementation.<br>— <strong>Capy</strong></em>
</pre>
</div>

</div>
</div>

[Full sample → `samples/metaprogramming/`](https://github.com/olivierdevelops/capy/tree/main/samples/metaprogramming)

---

## The `define` block

`define NAME ... end` has the **same body shape** as a function
declaration in a `.capy` library file:

| Inside a define block | Meaning |
|---|---|
| `arg literal "TEXT"` | Match a literal token in the source. |
| `arg capture NAME TYPE` | Capture one typed value. |
| `write \`...\`` | Emit text (multi-line backticks, `${EXPR}` interpolation). |
| `set` / `append` / `prepend` / `merge` / `delete` | Mutate `context.*`. |
| `if` / `else` / `for` | Control flow inside the body. |
| `block_closer NAME` | This function opens a block, closed by `NAME`. |
| `block_open "{" close "}"` | Or: explicit delimiter-pair blocks. |
| `priority N` | Disambiguation when two functions overlap. |

The full reference is in [library authoring](library-authoring.md);
everything that works in a library function works in a `define`.

## Rules

- **Top-level only.** `define` must be at column 0; matching `end`
  also at column 0. Indented `define` is treated as regular content
  (won't be picked up as a meta-block).
- **Defined before use.** The pre-pass scans the WHOLE file before
  parsing, so a `define` at the bottom of the file is still visible
  at the top. But for readability, conventionally put defines at
  the top.
- **Source wins on conflict.** If the library declares `function foo`
  and the source declares `define foo`, the source version wins.
  Use this to specialize without forking the library.
- **Identifier names only.** `define greet` works; `define "weird name"`
  doesn't — function names must be valid identifiers because the
  source-side parser couldn't tokenize them anyway.
- **`@import` composes.** Defines from imported files are visible
  in the importing file. Put shared metaprogramming in a `common/`
  directory and `@import` it.

## When to use it

| Pattern | Right tool |
|---|---|
| Repeating boilerplate in one source file | `define` |
| Used across many sources, same project | shared `.capy` file + `@import` |
| Used across many projects | library-level function in `lib.capy` |
| Truly project-specific UI / behavior | `define` (don't pollute the shared library) |

The progression is natural: prototype with `define`, promote to a
shared file via `@import` when reused, promote to the library when
the whole team should have it.

## How it composes with everything else

- **Types.** A `define` can use `arg capture name Email` where
  `Email` is a library-defined type — the validation kicks in just
  like for library functions.
- **State mutation.** A `define` body can `set` / `append` / etc.
  the same `context.*` as library functions. Use it for source-side
  state that the rest of the file consumes.
- **Block functions.** `define` blocks can have `block_closer end`
  to introduce indented body blocks. Inline a whole DSL extension
  if you need one.
- **Source `@import`.** Define a primitive in `shared/macros.capy`,
  `@import` from many scripts. Same primitive, defined once.
- **Errors.** Type violations, missing closers, regex-pattern
  mismatches all surface the same caret-pointed hints as library-
  function errors.

## Implementation notes

The pre-pass is a tiny string-level scanner in `rust/src/infra/define_extractor.rs`.
It:

1. Walks the source line-by-line.
2. At each column-0 `define NAME` line, collects until the
   matching column-0 `end`.
3. Rewrites each block as `function NAME ... end` and gathers them
   into a synthetic `.capy` library.
4. Runs the synthetic library through the normal library loader.
5. Returns the cleaned source (defines removed) plus the augment.

The orchestrator merges the augment into the loaded library before
lexing. From the parser's perspective there's only one library — it
can't tell which functions came from the library file and which
came from the source.

## Caveats

- **Defines are local to one Capy invocation.** They don't persist
  across runs; they're not stored in the library.
- **No CLI flag to list them yet.** `capy check` shows library
  functions; source-defined ones are visible only when running the
  source. We may add a `--print-defines` flag later.
- **Defines run in the WASM playground too.** Try the
  [metaprogramming sample in the playground](playground.md) — paste
  your own `define` blocks and watch them work in the browser.
