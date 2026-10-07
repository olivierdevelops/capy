# expression-grammar

**What it teaches:** ordered choice in a capture type — one capture can be "a
nested call, or a name, or a number", so a grammar nests. Shipped in 0.23.0
(`PROP-2026-0004`).

```sh
capy run samples/expression-grammar/lib.capy samples/expression-grammar/script.capy
capy ast samples/expression-grammar/lib.capy samples/expression-grammar/script.capy
```

## The one line that matters

```text
  function operand
      bare
      arg capture v call | name | num       <- tried left to right, first match wins
  end

   add ( 3 , mul ( 4 , 5 ) )
         │       └──────┬──────┘
         │              operand ─► try call ✅ ─► mul ( operand operand )
         └ operand ─► try call ❌ ─► try name ❌ ─► try num ✅
```

Each alternative is a library **function**. A flat alternative (`name`, `num`
here) is a `bare` function with one capture — a built-in type name is not
allowed in the choice.

## Three things to know

```text
  ORDER     the first alternative that matches wins; `num` accepts an identifier at
            parse time, so `name` is listed first
  DEPTH     right-recursive, bounded by the engine's depth limit: 31 call levels
            parse, 32 is refused
  WHICH     `capy ast --json` -> sub[].func is "call", "name" or "num"
```

## Goldens

| File | Checks |
|---|---|
| `script.expected.txt` | the rendered output |
| `script.expected-ast.txt` | the tree, including which alternative matched |
| `broken.expected-error.txt` | `capy run` refuses with the union of all three alternatives |
| `broken.expected-ast.txt` | `capy ast` recovers and reports the same union |

See [docs/library-authoring.md](../../docs/library-authoring.md) and
[PROP-2026-0004](../../program_docs/proposals/prop-2026-0004-nonterminal-alternation.md).
