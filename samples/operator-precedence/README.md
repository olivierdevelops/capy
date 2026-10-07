# operator-precedence

**What it teaches:** a single `any` capture takes a whole infix expression, and
the engine builds the tree — with conventional precedence and left
associativity. Shipped in 0.22.0.

```sh
capy run samples/operator-precedence/lib.capy samples/operator-precedence/script.capy
```

## The ladder

```text
  loosest   or
            and
            ==  !=  <  >  <=  >=
            +  -
  tightest  *  /  %

            not   <- prefix, binds tighter than all of them
```

All infix operators are **left-associative**: `a - b - c` is `(a - b) - c`.
The canonical table is
[docs/language-reference.md#operator-precedence](../../docs/language-reference.md).

## What the golden shows

Each `let` line prints twice, and the two lines are the point:

```text
let a = 1 + 2 * 3      <- ${value}              the SOURCE TEXT you wrote
      a == 7           <- ${context.vars[a]}    the EVALUATED result
```

A capture has two faces. In a `write` literal it resolves to source text, so a
Python emitter can pass `1 + 2 * 3` straight through. In a state mutation it
resolves to the evaluated value, so `set` stores `7`. Precedence is what
connects the two columns.

| Line | Source | Value | Rule |
|---|---|---|---|
| a | `1 + 2 * 3` | 7 | `*` binds tighter than `+` |
| b | `(1 + 2) * 3` | 9 | parentheses override |
| c | `10 - 2 - 3` | 5 | left-associative, not 11 |
| d | `7 % 4 + 1` | 4 | `%` with `*` and `/` |
| e | `100 / 5 / 2` | 10 | left-associative, not 40 |
| f | `1 + 1 == 2 * 1` | true | comparison looser than arithmetic |
| h | `1 == 1 and 2 == 3` | false | `and` looser than comparison |
| i | `… and … or …` | true | `and` binds tighter than `or` |
| j | `not 1 == 2` | false | **`not` binds tightest** — this is `(not 1) == 2` |

Line `j` is the one that surprises people. `not` is a prefix operator that binds
tighter than every infix operator, so it takes `1`, not the comparison.

## Two caveats worth knowing

**Grouping is conditional.** `(` is primarily the *prefix-call* form — `(upper name)`
calls a helper. Parentheses group only where that is unambiguous: when the
contents parse as a complete expression that is not a bare identifier. So
`(a + b) * c` groups, `(upper name)` is still a call, and `(foo)` is still a
zero-argument call rather than a grouped `foo`.

**`not` does not round-trip through source text.** `not (1 == 2)` evaluates
correctly, but renders back as the text `not 1 == 2`, which means something
else. Until that is fixed, do not write `not (…)` in a capture whose source text
you emit into a target language. Arithmetic and `and`/`or` grouping round-trip
correctly.

## Files

| File | Role |
|---|---|
| `lib.capy` | three functions: `let`, `show`, `note` |
| `script.capy` | the expressions under test |
| `script.expected.txt` | golden, compared by `cargo test --test golden` |
