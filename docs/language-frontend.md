---
title: Host your language's frontend
---

# Host your language's frontend

Most of these docs describe Capy as a transpiler: a small source, a library, a
generated artifact. That is one use. This page is about the other one.

Since 0.22.0 Capy has the pieces a **language front end** needs — a declared
grammar with nested scopes, recursive expressions with precedence, a span on
every node, error recovery, and actionable diagnostics. If you are building a
language and do not want to write a lexer, a parser, an error-recovery scheme
and a diagnostic renderer before you can start on the part you actually care
about, you can declare the grammar instead and get all of that.

!!! note "Front end, not compiler"
    Capy gives you **tokens → tree → diagnostics**. Type checking, name
    resolution, data-flow analysis, optimisation and code generation are yours.
    Capy deliberately has no opinion about any of them — see
    [what Capy does not do](#what-capy-does-not-do).

---

## A real language, in about 70 lines

This is a complete front end for a small imperative language: functions with
typed parameter lists, `let`, `if`, `while`, `return`, nested braces, and
expressions with real precedence. It ships as
[`samples/language-frontend/`](https://github.com/olivierdevelops/capy/tree/main/samples/language-frontend),
and everything below is golden-checked in CI.

```
function param
    arg capture pname ident
    arg literal ":"
    arg capture ptype ident
    write `${pname}: ${ptype}`
end

function fn
    arg literal "fn"
    arg capture name ident
    arg literal "("
    arg capture params param* sep "," join ", "
    arg literal ")"
    block_open "{" close "}"
    write `def ${name}(${params}):
${indent 4 body}
`
end

function let
    arg literal "let"
    arg capture name ident
    arg literal "="
    arg capture value any
    write `${name} = ${value}
`
end
```

`while`, `if` and `return` follow the same shape. That is the whole grammar
declaration — no lexer, no parser generator, no build step.

Source in the language:

```text
fn gcd(a: int, b: int) {
    while b != 0 {
        let t = b
        let b = a % b
        let a = t
    }
    return a
}
```

---

## What you get

### 1. The tree

```sh
capy ast samples/language-frontend/lib.capy samples/language-frontend/main.capy
```

```text
fn 1:1-1:23
  name = "gcd" 1:4
  params:
    param 1:8-1:14
      pname = "a" 1:8
      ptype = "int" 1:11
    param 1:16-1:22
      pname = "b" 1:16
      ptype = "int" 1:19
  while 2:5-2:17
    cond = "b != 0" 2:11
    let 3:9-3:18
      name = "t" 3:13
      value = "b" 3:17
    ...
  return 7:5-7:13
    value = "a" 7:12
```

Three properties a front end needs, all present:

| Property | Why it matters | Where you see it |
|---|---|---|
| **Scopes nest** | `while` is a child of `fn`; block structure is real | the indentation above |
| **Parameter lists are sub-trees** | each parameter is a node with its own spans, walkable | `params:` → two `param` nodes |
| **Everything is spanned** | `start_line:start_col-end_line:end_col`, down to captures | every line |

The parameter list is a **named nonterminal** —
`arg capture params param* sep "," join ", "` names another library function as
the element type. That is the mechanism for any repeated structure: argument
lists, field lists, enum variants, import lists.

### 2. Errors your users can read

A front end is judged on its error messages. Drop one `)`:

```text
$ capy run samples/language-frontend/lib.capy samples/language-frontend/broken.capy
error: expected `)`, found "{" in `fn`
  1 │ fn add(x: int, y: int {
    │                       ^
```

The message names the construct that got furthest (`fn`) and what it wanted
there (`)`). Before 0.22.0 this said "no library function matches" — true, and
useless to whoever is writing the code.

### 3. Recovery, so one mistake does not cost the file

```text
$ capy ast samples/language-frontend/lib.capy samples/language-frontend/broken.capy
return 2:5-2:17
  value = "x + y" 2:12
fn 5:1-5:24                      <- the NEXT function parsed fine
  name = "area" 5:4
  ...
while 10:1-10:17                 <- and so did this
  ...
<error> 1:1-1:24  12 token(s) skipped
<error> 3:1-3:2  1 token(s) skipped
error[E0001] 1:1: expected `)`, found "{" in `fn`
error[E0001] 3:1: no library function matches token "}"
```

One broken header, and the rest of the file still parses. That is what an editor
needs from a buffer mid-edit, and what a build needs to report more than one
error per run.

!!! warning "Expect some cascade"
    One real mistake produced two diagnostics above: the header, and then the
    orphaned `}` whose block never opened. The cascade-suppression constants are
    defaults that have not been tuned against a large corpus
    (`REL-0.22.0` Known Limitation 4). Budget for some follow-on noise.

### 4. Expressions with precedence

```text
let score = n * 2 + 1
if score > limit and n != 0 {
```

`* / %` bind tighter than `+ -`, comparison looser than both, `and` tighter than
`or`, prefix `not` tightest, everything left-associative. A single
`arg capture value any` takes the whole expression — you do not declare an
operator ladder.

!!! warning "This is the engine's expression grammar, not yours"
    You inherit it whole: its operators, its spellings (`and` / `or` / `not`,
    **not** `&&` / `||` / `!`), and its call form (`(f x y)`, **not** `f(x, y)`).
    If your language needs different surface syntax inside expressions, declare
    it yourself with function-as-type captures and [ordered choice](library-authoring.md#ordered-choice)
    — which also gets you a real sub-tree over JSON — rather than reaching for `any`. See
    [what Capy does not do](#what-capy-does-not-do).

See [operator precedence](language-reference.md#operator-precedence) and
[`samples/operator-precedence/`](https://github.com/olivierdevelops/capy/tree/main/samples/operator-precedence).

---

## Getting the tree into your compiler

Two routes, and the difference matters:

```text
                      ┌─────────────────────────────────────────┐
                      │ ParseResult { tree, diagnostics }       │
                      │   FuncCall  → span, captures, body      │
                      │   Capture   → text, span, sub, expr     │
                      └───────────┬───────────────┬─────────────┘
                                  │               │
                Library::parse    │               │   capy ast --json
                (Rust embedding)  │               │   (any language)
                                  ▼               ▼
                     statements ✅         statements ✅
                     spans      ✅         spans      ✅
                     diagnostics✅         diagnostics✅
                     sub-trees  ✅         sub-trees  ✅
                     EXPRESSION            EXPRESSION
                     TREES      ✅         TREES      ❌ text + `is_expr`
```

### Embedding in Rust — the full tree

```rust
let result = lib.parse(source);

if !result.is_clean() {
    for d in &result.diagnostics {
        eprintln!("{} {}:{}: {}", d.code, d.primary.start_line, d.primary.start_col, d.full_message());
    }
}

for stmt in &result.tree.stmts {
    // stmt.func, stmt.span, stmt.body, stmt.captures
    if let Some(cap) = stmt.captures.get("value") {
        if let Some(expr) = &cap.expr {
            // the parsed expression tree — walk it
        }
    }
}
```

`CaptureValue.expr` is the parsed expression. This is the route to take if your
analysis operates on expression trees. See [embedding](embedding.md).

### `capy ast --json` — everything except expression trees

```sh
capy ast lib.capy main.capy --json
```

One JSON document on stdout: the statement tree, spans, sub-trees and
`diagnostics[]`, all documented in [the schema](ast-json.md). Use it from any
language.

The one thing it does **not** carry is the expression tree. An expression
capture serializes as:

```json
"value": { "is_expr": true, "text": "a * b + c[i]", "span": { … }, "sub": [] }
```

You get the source text and a flag, not the tree. Re-parsing that text is not a
safe substitute today: the text rendering drops parentheses around `not` and
nested comparisons, so `not (a == b)` comes back as `not a == b`, which parses
differently.

**So:** statement-level tooling — formatters, linters, outline views, editor
diagnostics, transpilers — works over JSON from any language. Analysis that
walks expression trees should embed in Rust for now. Closing that gap is
[tracked as a proposal](#what-capy-does-not-do).

---

## What Capy does not do

Honest boundaries, so you can tell quickly whether this fits:

| Not provided | Notes |
|---|---|
| **C-style calls inside an `any` capture** | `f(x, y)` does **not** parse in an `any` capture. `(` is the prefix-call form, so `(divide 10 5)` works and `divide(10, 5)` does not. To get C-style calls — including nested ones like `add(3, mul(4, 5))` — declare the call grammar yourself with [ordered choice](library-authoring.md#ordered-choice); see [`samples/expression-grammar/`](https://github.com/olivierdevelops/capy/tree/main/samples/expression-grammar). What is still missing is **infix operators around such a call** (`return a + divide(b, c)`): the engine's precedence ladder lives inside `any`, so a declared grammar must lower its own operators. |
| **Choosing your own operator spellings** | The ladder is fixed: `and` / `or` / `not`, not `&&` / `\|\|` / `!`. `a >= 0 && b != 0` does not parse. If your language uses C-style logical operators, the built-in expression grammar is not your path. |
| **Type checking, name resolution, data-flow analysis** | Yours. Capy's `type` declarations validate *token shape* (regex / enum), not a type system. |
| **Expression trees over JSON** | In the Rust AST, not in `capy ast --json`. Embed in Rust, or track the proposal to serialize it. |
| **Byte offsets** | Spans carry line and column. An LSP integration converts on its side. |
| **Left-recursive grammars** | Rejected at library load with a cycle trace, rather than overflowing the stack. Rewrite right-recursively; infix operators are built in, which removes the usual reason to want left recursion. |
| **Unbounded nesting** | Nonterminal descent stops at 64 captures — about 31 levels of nested calls in a choice-based grammar. Beyond it the error names the limit (`nesting too deep`, code `E0003`). |
| **AST over the wasm ABI** | The browser [playground](playground.md) can transpile but cannot show a tree or diagnostics. CLI and embedding only. |
| **Codegen beyond text** | Capy emits text. That is enough for a transpiler or a source-to-source compiler; it is not a backend. |

If you need the analysis half, Capy is the front end and you write the rest —
which is the intended division of labour.

---

## Where to go next

- [`samples/language-frontend/`](https://github.com/olivierdevelops/capy/tree/main/samples/language-frontend) — the code on this page, runnable
- [Library authoring](library-authoring.md) — the full grammar-declaration reference
- [Grammar as contract](grammar-as-contract.md) — one grammar, many consumers
- [AST JSON schema](ast-json.md) — the `--json` contract
- [Diagnostics](diagnostics.md) — codes, labels, cascade control
- [Tutorial 5 · Reading diagnostics](tutorials/05-reading-diagnostics.md) — the debug loop
- [Embed Capy in a Rust program](embedding.md) — `Library::parse` and the full tree
