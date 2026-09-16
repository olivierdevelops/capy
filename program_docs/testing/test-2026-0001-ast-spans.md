---
document_id: TEST-2026-0001
title: Test — AST Source Spans
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
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Defines and records the tests proving that every AST node an external consumer can reach carries a real, correctly nested source span.

reason: Interior nodes were stamped `line: 0, col: 0`, which made diagnostics and provenance chains unbuildable.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - spans
  - parser
  - ast

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — AST Source Spans

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Purpose

Prove PLAN-2026-0001 R1–R5: a `Span` exists, statements and captures carry one,
nested nodes are no longer stamped `line: 0, col: 0`, and the pre-existing
`line`/`col` render locals are unchanged.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0001 R1 | `Span` type with start/end line and column |
| PLAN-2026-0001 R2 | `FuncCall.span` covers the statement, body and closer |
| PLAN-2026-0001 R3 | `CaptureValue.span` covers exactly its own tokens |
| PLAN-2026-0001 R4 | No reachable node has an unset span |
| PLAN-2026-0001 R5 | `line`/`col` render locals unchanged |

## Preconditions

`capy-core` built from the working tree; no library or sample modified.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable 1.90.0, debug profile, `cargo test`.

## Test Data

Fixture libraries defined inline in `rust/tests/ast_spans.rs`: a flat two-capture
function, a three-level function-as-type chain (`call` -> `args` -> `pair`), a
`block_closer` function, and a function with a defaulted trailing capture.

## Procedure

```sh
cargo test --manifest-path rust/Cargo.toml --test ast_spans
```

## Expected Results

All span tests pass; in particular `no_reachable_node_has_an_unset_span` walks
every reachable node — captures' sub-matches, block bodies, closers and sections
— and finds no `line == 0` and no unset span, with every child range contained by
its parent's.

## Actual Results

13 tests passed, 0 failed (the file also carries the R27 comment tests recorded
in TEST-2026-0003).

Span-specific results:

| Test | Assertion | Outcome |
|---|---|---|
| `statement_span_covers_the_statement` | `greet world now` spans columns 1..16 | PASS |
| `capture_spans_are_distinct_and_tight` | `world` 7..12, `now` 13..16, non-overlapping | PASS |
| `no_reachable_node_has_an_unset_span` | 3 nodes walked, none unset, all nested | PASS |
| `block_span_covers_body_and_closer` | span ends on line 3 (`end`), not line 1 | PASS |
| `join_ignores_unset` | `Span::join` ignores an unset operand | PASS |
| `defaulted_capture_has_an_unset_span` | a defaulted capture consumed nothing, so stays unset | PASS |

## Result

PASS

## Evidence

```text
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

R5 is additionally evidenced by the golden corpus: 116 pass / 0 fail, byte-identical.

## Evidence Sources

- `rust/tests/ast_spans.rs` (committed)
- `cargo test --manifest-path rust/Cargo.toml --test ast_spans`
- `cargo test --manifest-path rust/Cargo.toml --test golden`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None in the implementation. Two defects in the **tests themselves** were found
and fixed during authoring: a fixture used `!` where the capture type could not
match it, and an early version asserted column arithmetic that did not account
for the exclusive end column.

## Related Documents

- PLAN-2026-0001 — the plan under test
- PROP-2026-0001 — P-01, P-02, P-03 (the reported defects)

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
