---
document_id: TEST-2026-0006
title: Test — Operator Precedence and Expression Round-Trip
document_type: test
status: completed

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 1

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

scope: Records the precedence tests and the round-trip property that guards against silent output corruption.

reason: A renderer that drops grouping parentheses changes the emitted code silently, and no golden can catch it because arithmetic was a parse error before this change.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - precedence
  - round-trip

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — Operator Precedence and Expression Round-Trip

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Purpose

Prove R10 and R11, and the round-trip property (T-27) that no other test covers.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0002 R10 | Infix operators with documented precedence, left-associative |
| PLAN-2026-0002 R11 | Comparison precedence-ordered |
| PLAN-2026-0002 R12 | `expr_to_text` round-trips without changing the tree |

## Procedure

```sh
cargo test --manifest-path rust/Cargo.toml --test precedence
```

## Expected Results

`a * b + c` parses `(a*b)+c`; `a + b * c` parses `a+(b*c)`; and for every
expression in the corpus, `parse(render(parse(E)))` is structurally equal to
`parse(E)`.

## Actual Results

6 passed, 0 failed.

Evaluation was also verified end-to-end through the CLI:

| Expression | Result |
|---|---|
| `1 + 2 * 3` | 7 |
| `(1 + 2) * 3` | 9 |
| `10 - 2 - 3` | 5 |
| `2 * 3 % 4` | 2 |
| `7 / 2` | 3.5 — no truncation |
| `1 + 1` | 2 — integers stay integers |

## Result

PASS

## Evidence

The round-trip test uses **structural** equality, not string equality: the
assertion is about the tree, so a formatting change cannot mask a structural one.
`a - (b - c)` is in the corpus specifically because a renderer that only
parenthesises looser children gets it wrong.

## Evidence Sources

- `rust/tests/precedence.rs` (committed)
- `cargo test --manifest-path rust/Cargo.toml --test precedence`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None in the implementation. The Rust compiler's exhaustiveness check named every
site that had to learn the new node, including `expr_to_text` — the one whose
omission would have corrupted output silently.

An initial version of the test corpus assumed parenthesised grouping already
existed; it did not (`(` is the prefix-call form), which is what prompted adding
grouping in a way that leaves `(upper n)` and `(foo)` untouched.

## Related Documents

- PLAN-2026-0002
- PROP-2026-0001 review finding 1 (T-27 was added in review, before implementation)

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
