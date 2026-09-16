---
document_id: MAN-2026-0001
title: Manual — Source Positions, Comments and Recursion Limits
document_type: manual
status: active

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 2

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - technical-writers

scope: The reader-facing book for what 0.21.0 adds — what a span is, what comment retention does and does not promise, and how recursion is bounded.

reason: A reader must be able to use these features without reading the source or the plan.

related_documents:
  - PLAN-2026-0001
  - SYS-2026-0001

supersedes: null
superseded_by: null

tags:
  - manual
  - spans
  - comments

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Manual — Source Positions, Comments and Recursion Limits

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Summary

0.21.0 makes it possible to say *where* something in a source file is, keeps the
comments a source file contains, and stops a recursive grammar from killing the
process. This chapter is what a reader needs to use those three things.

## 1. What a span is, and what it is for

Capy has always told you which line a statement was on. It could not tell you
which *argument* of that statement a problem was about, because arguments had no
position at all, and anything nested inside a statement reported line 0.

A `Span` is a range: a start line and column, and an end line and column.

```text
    greet world now
    ^^^^^^^^^^^^^^^     statement span      1:1 – 1:16
          ^^^^^         capture `who`       1:7 – 1:12
                ^^^     capture `when`      1:13 – 1:16
```

Two rules are worth committing to memory:

- **The end column is exclusive.** It is one past the last byte, so
  `end_col - start_col` is a width.
- **An unset span means nothing was consumed.** An optional argument that fell
  back to its default has no source range, and says so, rather than claiming to
  be at line 0. Check `is_unset()` before trusting a span.

A statement's span covers its whole block, including the closer:

```text
    wrap            ◄─┐
        item          │  span: 1:1 – 3:4
    end             ◄─┘
```

## 2. Comments

Comments in a script survive parsing. Those written immediately above a statement
are attached to it.

```text
    # who to greet          ◄── leading_comments[0]  1:1 – 1:15
    greet world             ◄── span                 2:1 – 2:12
```

**What is promised:** comments above a statement, in source order, with their own
spans, and a statement span that *excludes* them.

**What is not promised:** trailing comments (`greet world # here`) and comments
inside an empty block are retained as trivia but are **not attached to
anything**. That is deliberate — see "Why leading only" below.

**What never changes:** comment retention does not affect what parses or what is
rendered. The same script with and without comments produces identical output.

A library that declares no comment markers has no comment syntax at all; `#` is
then ordinary source. Retention does not invent a syntax.

### Why leading only

Scoped by what the consuming tool needs:

| Tool | Needs |
|---|---|
| compiler / analyzer | leading only — doc comments for hover and generated docs |
| LSP hover | leading only |
| formatter | leading **and** trailing **and** interior |

A formatter that drops a trailing `# why` is unusable, so the day Capy emits a
formatter — or hosts an annotation system whose trailing comments carry meaning —
this expands. Until then, leading is what is supported and what is tested.

## 3. Recursion limits

### A rule cannot begin with itself

```
function expr
    arg capture lhs expr      ← rejected
```

The matcher would descend into `expr` before consuming anything, forever. The
library is refused when it loads:

```text
function "expr": left recursion — it can match itself without consuming a token
(cycle: expr -> expr)
```

Consume something first:

```
function expr
    arg literal "("           ← consumed before the descent
    arg capture inner expr
    arg literal ")"
end
```

**A surprise worth knowing:** a function that declares no `arg literal` gets its
own name prepended as one. `function term / arg capture inner expr` is really
`term <inner>`, so it consumes a token and is never the offending hop. Two
libraries that look alike in source can differ here — `capy docs <lib>` prints
the compiled pattern.

### Input cannot nest forever either

Nonterminal descent stops at 64 levels. That is far past anything hand-written;
before the limit existed, deeply nested input aborted the process. The message
you get today is the generic "no library function matches"; naming the limit is
future work.

## 4. Reading errors (0.22.0)

When a statement does not match, Capy reports the shape that got **furthest**
into it and what that shape wanted next — not merely that nothing matched.

```text
error: expected `)`, found end of statement in `fn`
```

The trailing clause is the context frame: which shape, and which of its
arguments. When several shapes stop at the same token you see all of them:
`expected \`p\`, \`q\`, or \`r\``.

## 5. Recovery (0.22.0)

`Library::parse` does not stop at the first mistake. Every broken region is
reported and skipped, and the statements around it still parse.

```rust
let r = lib.parse(src);
r.tree.stmts     // what parsed
r.tree.errors    // what did not
r.diagnostics    // why
r.is_clean()     // neither of the last two
```

`capy run` still refuses to emit anything when a region failed — a partial parse
would produce target code missing whatever broke.

## 6. Arithmetic (0.22.0)

Infix `*` `/` `%` `+` `-`, comparisons and `and` / `or`, with conventional
precedence, left-associative. Integers stay integers; `7 / 2` is `3.5`. `and`
and `or` short-circuit.

Grouping parentheses work where they do not collide with the prefix-call form
`(upper n)` — which means `(a + b) * c` groups, and `(foo)` is still a call.

## 7. Limitations at 0.22.0

| Limitation | Consequence |
|---|---|
| No byte offsets on `Span` | You cannot slice the original source by offset; use line/column |
| Depth-limit wording | Reports "no library function matches" rather than naming the limit |
| Leading comments only | Trailing and interior are retained but unattached |
| Grouping is conditional | `(foo)` is a zero-argument call, not a grouped identifier |
| Cascade constants untuned | Suppression and cap use defaults, not corpus-derived values |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial manual chapter |
| 2 | 2026-09-16 | Olivier | Added reading errors, recovery and arithmetic for 0.22.0; the AST is now a public API |
