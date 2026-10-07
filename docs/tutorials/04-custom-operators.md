# Tutorial 4: Custom Operators and Block Contexts

How to define patterns that DON'T start with a function-name literal —
operator-style syntax like `x = 1`, `a + b`, `x := y`. Estimated time: 10
minutes.

## The auto-name-prepend rule recap

If your `args:` has **zero** `kind: literal` entries, Capy auto-prepends
the function's key as a leading literal. That's how `greet <any>` works
without you writing `{kind: literal, value: "greet"}`.

The moment you add any literal, the rule turns OFF. Now the args list IS
the entire shape — and the function key is just a reference name.

## Pattern A: assignment

```
function assign
    arg capture var ident
    arg literal "="
    arg capture value any
    write `${var} = ${value}
`
end
```

Matches `x = 1`. The function name `assign` does not appear in source.

## Pattern B: walrus-operator style

```
function walrus
    arg capture var ident
    arg literal ":="
    arg capture value any
    write `${var} := ${value}
`
end
```

The lexer reads `:=` as one `PUNCT` token because both characters are in
the punctuation set — your literal must match the full text.

## Pattern C: arithmetic-flavored assignment

```
function assign_add
    arg capture var ident
    arg literal "="
    arg capture a any
    arg literal "+"
    arg capture b any
    write `${var} = ${a} + ${b}
`
end
```

Matches `x = 4 + 5`. Five tokens, three captures, two literals.

## Pattern overlap and priority

`x = 4 + 5` could match BOTH `assign` (with `value` = `4`, then leftover
`+ 5` — no, actually that wouldn't match because the trailing tokens
don't fit the pattern → match fails) and `assign_add` (full match).

What about `x = 1`? Only `assign` matches because there's no `+` to anchor
the `assign_add` pattern.

When two patterns DO genuinely overlap, Capy picks the one that:

1. Has higher `priority:` (default 0).
2. Has more leading literal tokens (so longer, more specific patterns
   win).

You can usually rely on (2). Set `priority:` explicitly only when (2)
doesn't pick the right one.

## Block constructs with custom delimiters

A `for x in xs { ... }` C-style block:

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

This is Mode B (delimiter blocks). The opener consumes everything up to
`{`, then the body is parsed until `}`. Newlines inside the body are
statement boundaries; the `}` ends the block.

Contrast with Mode A (indent + named closer) from Tutorial 3.

## Combining both block modes

You can declare two functions with the same surface keyword, one Mode A
and one Mode B:

```
function for_indent
    arg literal "for"
    arg capture v ident
    arg literal "in"
    arg capture i any
    arg literal "do"
    block_closer end
    write `for ${v} in ${i}:
${indent 4 body}`
end

function for_brace
    arg literal "for"
    arg capture v ident
    arg literal "in"
    arg capture i any
    block_open "{"
    block_close "}"
    write `for ${v} in ${i} { ${body} }
`
end
```

The matcher picks based on which delimiter shows up in source.

## Object literals with unquoted keys

For configuration DSLs, unquoted keys read better:

```
set config {host: "localhost", port: 5432, ssl: true}
```

Object literals accept either quoted strings OR bare identifiers as keys
— no library setting needed.

## Try it

- Add a `pipe` operator: `x |> f` that emits `f(x)` in Python.
- Build a tiny expression language with `+`, `-`, `*`, `/`. Since 0.22.0 you
  usually should **not** do this with multi-token patterns: a single
  `arg capture value any` takes the whole expression and the engine builds the
  tree, with conventional precedence and left associativity. See
  [operator precedence](../language-reference.md#operator-precedence) and
  [`samples/operator-precedence/`](https://github.com/olivierdevelops/capy/tree/main/samples/operator-precedence).
  Flat multi-token patterns are still the right tool for operators the engine
  does not know, like the `|>` above.
- Add a `where` clause to a `find` block: `find x in xs where x > 0
  { ... }`.

## What you've learned across the tutorials

| Tutorial | Concept |
|---|---|
| 1 | Single function + template |
| 2 | Types, context, file template |
| 3 | Block functions (Mode A), body composition |
| 4 | Operator patterns, delimiter blocks (Mode B), priority |

You now have the full toolkit. Look at the [cookbook](../cookbook.md)
for more recipes, or build something real and share it via
[Discussions](https://github.com/olivierdevelops/capy/discussions).
