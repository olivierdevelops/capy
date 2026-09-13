# Common pitfalls

Read this BEFORE authoring a library.

## 1. Forgetting `kind:` on args

WRONG:

```yaml
args:
  - { literal: "if" }
  - { name: cond, type: any }
```

RIGHT:

```yaml
args:
  - { kind: literal, value: "if" }
  - { kind: capture, name: cond, type: any }
```

The loader rejects entries without an explicit `kind:`.

## 2. Mixing capture and literal fields

WRONG:

```yaml
- { kind: literal, value: "if", name: cond }
- { kind: capture, name: cond, type: any, value: "default" }
```

RIGHT: each `kind` has its own required field set. Don't mix.

## 3. Capy DOES NOT execute user source

You can't write `if x ... end` in a Capy source expecting Capy to
conditionally skip rendering. `if x` is just a pattern that emits an
`if` block in the target language. If you need transpile-time
conditional logic, write it in the library's `run:` snippet against
`context`.

## 4. Confusing source text with evaluated value

In **templates**, captures appear as source text — including quotes for
string literals. So `say "hello"` exposes `.msg = "\"hello\""` (with
quotes). `print({{ .msg }})` correctly emits `print("hello")` in Python.

In **`run:` snippets**, captures appear as evaluated values — strings
without quotes, numbers as int64, lists as []any. `append context.x msg`
stores the unquoted string `"hello"`.

If you want the SAME treatment in both, the template can use `toQuoted`
to re-quote a string value, or the run snippet would need a parse-back
primitive (currently none).

## 5. Indentation: 4 spaces or 1 tab per level

Both in user source AND inside `run:` snippets. 2-space indent breaks
the lexer with "indentation must be 4 spaces or 1 tab per level".

## 6. YAML block-scalar indentation vs inner-DSL indentation

These are two independent layers. YAML strips the common leading indent
of a block scalar (`|`) — what's left is what the inner-DSL parser sees,
and that has its OWN 4-space rule.

```yaml
functions:
  if:
    run: |
      if cond            # this is at column 0 of the inner-DSL source
          body           # this is at column 4 — one level deeper
      end
```

## 7. Auto-name-prepend silently turns off

If `args` has ANY `kind: literal` entry, the function key is NOT
auto-prepended. So:

```yaml
greet:
  args:
    - { kind: literal, value: "hi" }     # auto-prepend disabled
    - { kind: capture, name: name, type: any }
```

…matches `hi <any>`, NOT `greet hi <any>`. Add `{kind: literal, value: "greet"}`
explicitly if you want the function name in source.

## 8. `{}` ambiguity

A `{...}` in source is an object literal by default. For a brace-delimited
block, the opener function must declare `block: { open: "{", close: "}" }`
explicitly.

If you want both forms in the same library, make sure the contexts
don't overlap (e.g. an `any` capture won't accidentally swallow the `{`
that should start a block).

## 9. Closer mismatch

For Mode A blocks (`block: { closer: end }`):
- `end` must be defined as a function (`end: {}` is fine).
- The body's DEDENT must match the function's indent baseline.

## 10. Forgetting `priority:` for overlapping patterns

If two functions could match the same source prefix:

```yaml
assign:
  args:
    - { kind: capture, name: var, type: ident }
    - { kind: literal, value: "=" }
    - { kind: capture, name: value, type: any }

assign_add:
  args:
    - { kind: capture, name: var, type: ident }
    - { kind: literal, value: "=" }
    - { kind: capture, name: a, type: any }
    - { kind: literal, value: "+" }
    - { kind: capture, name: b, type: any }
```

For `x = 4 + 5`, `assign_add` wins because it consumes more literal
tokens. For `x = 1`, only `assign` matches. Usually fine. When in doubt,
set `priority: 100` on the more specific pattern.

## 11. No `else` branch

Inner `if` has no `else` arm. Workaround:

```
if cond
    ...
end
if not cond
    ...
end
```

## 12. Forgetting to capture goldens

After every behavior change:

```sh
CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden
```

…to refresh `*.expected.txt` files. Then `go test ./...` to confirm.

## 13. Output that's empty / unexpected

Common causes:
- Function matched but `template:` is empty (intentional context-only).
- Function matched but the template doesn't reference `.<capture>`.
- A block opener has `block:` set but no template / body reference.

Quick check: `capy run lib.yaml script.capy` shows what's emitted at
each level.
