# mixed-parameters

**What it teaches:** a parameter list where some parameters carry a marker
(`mut`) and others do not. Shipped in 0.23.0 (`PROP-2026-0004`, UC-06).

```sh
capy run samples/mixed-parameters/lib.capy samples/mixed-parameters/script.capy
```

```text
  before (0.22.0)                          now
  param = "mut" NAME ":" TYPE              ps = mut_param | plain_param*
        └ one shape: `mut` on ALL or NONE          └ two shapes, tried in order

  def f(mut c: Counter, n: int)
        └ mut_param ┘  └ plain_param ┘      ✅ parses; was: expected `mut`, found "n"
```

Put the marked shape first. If `plain_param` began with a bare `ident` it would
swallow `mut` as a parameter name.

| File | Checks |
|---|---|
| `script.expected.txt` | output for marked, unmarked, mixed and empty parameter lists |
| `script.expected-ast.txt` | `sub[].func` is `mut_param` or `plain_param` per parameter |

See [docs/library-authoring.md](../../docs/library-authoring.md).
