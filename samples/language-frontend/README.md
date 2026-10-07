# language-frontend

**What it teaches:** Capy as the **front end of a real language** — not a config
DSL, not a template. A grammar with nested scopes, typed parameter lists,
recursive expressions, spans on every node, and statement-level error recovery.

```sh
capy run samples/language-frontend/lib.capy samples/language-frontend/main.capy   # lower to Python
capy ast samples/language-frontend/lib.capy samples/language-frontend/main.capy   # the tree
capy ast samples/language-frontend/lib.capy samples/language-frontend/broken.capy # recovery
```

## The language

```text
fn NAME(P: T, …) { … }     function, typed parameter list, nested scope
let NAME = EXPR            a whole expression in one `any` capture
if EXPR { … }              nested scope
while EXPR { … }           nested scope
return EXPR
```

Roughly 70 lines of `lib.capy`, six functions, no engine support for any of it —
Capy ships zero grammar, so every keyword here is declared by the library.

## What makes it a front end

```text
   main.capy                  capy ast                        capy run
   ─────────                  ────────                        ────────
   fn gcd(a: int, b: int) {   fn 1:1-1:23                     def gcd(a: int, b: int):
       while b != 0 {           name = "gcd" 1:4                  while b != 0:
           let t = b            params:                               t = b
           let b = a % b          param 1:8-1:14                      b = a % b
           let a = t                pname = "a" 1:8                   a = t
       }                            ptype = "int" 1:11            return a
       return a                   param 1:16-1:22
   }                                pname = "b" 1:16
                                    ptype = "int" 1:19
                                 while 2:5-2:17
                                   cond = "b != 0" 2:11
                                   let 3:9-3:18 …
                                 return 7:5-7:13
```

Three things to notice in the tree:

1. **Scopes nest.** `while` is a child of `fn`, `let` a child of `while`. Block
   structure is real, not flattened.
2. **`params` is a sub-tree**, not a string. It is a *named nonterminal* —
   `arg capture params param* sep "," join ", "` — so each parameter is its own
   node with its own spans. That is what lets a consumer walk arguments.
3. **Every node carries a span**, `start_line:start_col-end_line:end_col`, down
   to individual captures.

The Python lowering is there to show the front end is attached to something. A
consumer that only wants the tree never calls `capy run` at all.

## Recovery: what a language's users need

`broken.capy` drops one `)` from a function header:

```text
$ capy run … broken.capy
error: expected `)`, found "{" in `fn`
  1 │ fn add(x: int, y: int {
    │                       ^
```

The message names the construct that got furthest (`fn`) and what it wanted
there (`)`) — the difference between a language people can use and one they
cannot. `capy ast` goes further and keeps parsing:

```text
<error> 1:1-1:24  12 token(s) skipped
<error> 3:1-3:2  1 token(s) skipped
error[E0001] 1:1: expected `)`, found "{" in `fn`
error[E0001] 3:1: no library function matches token "}"
```

The second `fn` and the trailing `while` parse normally — one bad header does
not cost you the rest of the file.

**Note the cascade.** One real mistake produced two diagnostics: the broken
header, and then the orphaned `}` on line 3, which belongs to a block that never
opened. The cascade constants are defaults and have not been tuned against a
large corpus (`REL-0.22.0` Known Limitation 4), so expect some follow-on noise
in a real grammar.

## Getting the tree out

| Route | What you get |
|---|---|
| `Library::parse` in Rust | `ParseResult { tree, diagnostics }`, and `CaptureValue.expr` — the **parsed expression tree** for `any` captures |
| `capy ast --json` | the statement tree, spans and diagnostics; expression captures arrive as `is_expr: true` plus their **source text**, not a tree |

If your analysis walks expression trees, embed in Rust today. See
[embedding](../../docs/embedding.md) and
[host your language's frontend](../../docs/language-frontend.md).

## Files and goldens

| File | Golden | Asserts |
|---|---|---|
| `main.capy` | `main.expected.txt` | the Python lowering |
| `main.capy` | `main.expected-ast.txt` | the full parse tree, spans included |
| `broken.capy` | `broken.expected-error.txt` | what `capy run` refuses with |
| `broken.capy` | `broken.expected-ast.txt` | the recovered tree and its diagnostics |
