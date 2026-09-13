# Troubleshooting

A decision tree for when a transpilation goes wrong. Each branch points at the
underlying cause and the fix. For the anatomy of an error message and worked
examples, read [errors-and-debugging.md](errors-and-debugging.md) first; this
page is the "my thing is broken, where do I look" companion.

> First move, always: re-run with `--debug`. It prints the token stream, the
> matched functions, and source-line attribution — most mysteries resolve here.
> ```bash
> capy run my-lib script.capy --debug
> ```

---

## My library won't load (`capy check` fails)

The error happens before any script runs — it's in the `.capy` library itself.

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| `unknown directive "…"` | Typo in a directive, or a function-body directive used at top level | Check it against the [cheat sheet](syntax-cheat-sheet.md). |
| `unknown type "…"` | A `capture` references a `type` you didn't declare (or declared after use) | Declare the `type` block; built-in types are `string int float bool ident raw expr`. |
| `duplicate function`/literal | Two functions share the same literal and priority | Give one a higher `priority`, or differentiate the literal. |
| Template never closes | A `template:` block missing its `end`, or an unbalanced backtick | Backticks inside templates are significant — escape or rebalance them. |

---

## My script fails to parse

The library loads, but a script line doesn't match any function.

1. **No function matched this line.** The first token doesn't begin any
   function's pattern. Confirm the literal spelling and that the function is in
   the loaded library (`capy check` lists them).
2. **The wrong function matched.** Two functions start with the same literal.
   Parser candidates are ordered by `(priority desc, literal-start,
   literal-length desc, name asc)` — raise `priority` on the one you want, or
   make literals more specific. See [transpiler-patterns.md](transpiler-patterns.md).
3. **Indentation surprise.** Capy tracks an indent *stack* (not fixed 4-space
   levels). Mixed tabs/spaces or an unexpected dedent can close a block early.
   Run `capy fmt` to normalise leading whitespace to 4 spaces.
4. **A keyword means two things.** If you reused a keyword for both a flat and
   a block form, you need `when_followed_by` / `when_not_followed_by` lookahead
   so the parser can disambiguate.

---

## My output is wrong (parses fine, renders badly)

| Symptom | Cause | Fix |
|---------|-------|-----|
| `${name}` appears literally in output | Reference resolved to nothing, or wrong path | Check the capture name and `context.` path; `--debug` shows captures. |
| A helper isn't applied | Misspelled helper, or wrong arity | Built-ins are listed in [function-cookbook.md](function-cookbook.md) / the [cheat sheet](syntax-cheat-sheet.md). |
| Wrong whitespace / indentation | Template whitespace is literal | Use the `indent` / `align` helpers; remember backtick contents are verbatim. |
| Block closer text missing or doubled | Wrong block mode | Confirm you used `block_closer` (runs a closer) vs `block_dedent` (no closer). See [block-functions.md](block-functions.md). |

---

## My `run:` mutations don't stick

- **Context isn't accumulating.** `set` replaces; use `append`/`prepend` to
  build lists. Confirm you're writing to `context.x` (persists) not a `let`
  local (scoped to that block).
- **`if` has no `else`-of-`else`.** Inner `if` supports a single `else`. For
  more arms, nest or use a negated second `if`.
- **A value reads as a string when you wanted a number.** Captures are typed by
  their `type`; cast or use math helpers (`add`, `mul`) which coerce.

---

## Multi-file output is missing files

- Each extra file needs a `file "path": … end` declaration. Confirm the path
  and that `${…}` interpolations resolve (an empty interpolation yields an empty
  path, which is dropped).
- When embedding, remember `Run` returns only the primary output — use
  `RunMulti` to get the file map. See [multi-file-and-imports.md](multi-file-and-imports.md).

---

## `command` / host calls behave oddly

- **`env`/`read_file` return empty.** When embedding, the default host is a
  sandbox (`domain.NoOpHost`) — install `infra.OSHost` (or your own) via
  `SetHost`. The CLI wires the OS host automatically.
- **`exec`/`write_file` do nothing.** Side-effecting primitives only run inside
  a `command` body, never in a `template:`/`run:` render path. See
  [host-capabilities.md](host-capabilities.md).

---

## `import` surprises

- **Later import wins.** Imports merge last-write-wins for functions, types, and
  context defaults. Order matters.
- **Cycle detected.** The loader refuses import cycles. Break the cycle or
  extract the shared piece into a third file both import.

---

## Still stuck?

1. Minimise: shrink the script to the smallest line that reproduces it.
2. `capy check <lib>` to isolate library vs script problems.
3. `--debug` to see tokens, matches, and captures.
4. Compare against a working [sample](showcase.md) that uses the same feature.
5. Open an issue with the minimal library + script + the `--debug` output.
