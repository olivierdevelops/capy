---
title: Diagnostics
---

# Diagnostics

What Capy tells a user when their source doesn't parse, and what a tool can read
programmatically.

## What changed

A failed parse used to report only that nothing matched:

```
error: no library function matches token "fn"
```

That names what failed and never what was wanted, points at the statement rather
than the token, and stops at the first mistake. Now:

```
error: expected `)`, found end of statement in `fn`
  1 │ fn add(x
```

## How the message is chosen

Every library shape is tried. Rather than report the aggregate failure, Capy
remembers the attempt that got **furthest** through the token stream and reports
where *that* one stopped.

```text
   source:   fn add(x: int, y: int {
   tokens:   fn  add  (  x  :  int  ,  y  :  int  {
   index:    0   1    2  3  4  5    6  7  8  9    10

   shape `fn`      consumed through 9, then wanted `)` at 10   ← furthest
   shape `call`    failed at index 0
   shape `assign`  failed at index 1
```

When several shapes stop at the **same** token, their expectations are unioned,
so you see every alternative:

```
error: expected `p`, `q`, or `r`, found "zzz" in `x`
```

The trailing `in \`fn\`` is the **context frame** — which shape, and which
argument of it, the matcher was working on. Without it, "expected `)`" cannot
say *which* `)`, and the shape that got furthest is not always the one you meant.

## Diagnostic codes

Stable and machine-readable. A code's meaning does not change once published;
new situations get new codes.

| Code | Meaning |
|---|---|
| `E0001` | No library function matches the statement |
| `E0002` | A delimiter was opened and never closed |
| `E0003` | Source nests deeper than the parser will follow |

## Error recovery

`Library::parse` does not stop at the first mistake. It reports the failure,
skips the unparseable region, resynchronises, and carries on — so a user fixes
every mistake in one run, and an editor gets a usable tree from a buffer that is
mid-edit.

```text
greet a        ← parsed
bogus one      ← error 1, skipped
greet b        ← parsed
also bad       ← error 2, skipped
greet c        ← parsed
```

```
statements parsed: 3
error regions:     2
diagnostics:       2
```

### Where it resumes

Resync points, in priority order:

1. **Delimiter balance first.** Never resume inside an unclosed `(`, `[` or `{`.
   This is the rule that stops a single missing `)` from swallowing the rest of
   the file. If the delimiter is never closed, a line break is taken as evidence
   that it is *missing* rather than spanning lines, and the scan gives up on
   balance rather than losing the file.
2. **Statement boundary.** A newline, or a token that begins some known shape —
   and because the library enumerates every shape, that set is exact rather than
   guessed.
3. **Block boundary** — a dedent.
4. **End of file.**

### Cascade control

One broken construct should produce one message, not forty:

- a new diagnostic within a few tokens of the previous one is suppressed;
- output is capped, and the remainder reported as a count.

Both are tuning values, not contracts.

## Output is refused for a partial parse

`capy run` writes nothing when any region failed to parse. Rendering a partial
parse would emit target code that silently omits whatever broke, which is worse
than emitting nothing.

`capy ast` still prints the tree, because inspecting a broken file is the point.

## Reading diagnostics programmatically

```rust
let result = lib.parse(source);
for d in &result.diagnostics {
    println!("{}[{}] {}:{} {}",
        match d.severity { Severity::Error => "error", _ => "warning" },
        d.code, d.primary.start_line, d.primary.start_col, d.full_message());
    for label in &d.labels {
        println!("  {} at {}:{}", label.text, label.span.start_line, label.span.start_col);
    }
}
```

`result.tree.errors` holds the same regions as AST nodes, each pointing back at
its diagnostic by index. See [the AST JSON schema](ast-json.md) for the
machine-readable form.

> **`tree.stmts` alone does not tell you the parse succeeded.** Check
> `diagnostics` (or `tree.errors`) first — a partial parse looks exactly like a
> complete one if you only read `stmts`.
