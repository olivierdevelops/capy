---
document_id: TEST-2026-0005
title: Test — Diagnostics and Error Recovery
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

scope: Records the tests proving parse errors name what was expected, and that parsing recovers from a failed statement.

reason: A language cannot be hosted on an engine whose parse errors say only that nothing matched and which stops at the first mistake.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - diagnostics
  - recovery

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — Diagnostics and Error Recovery

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Purpose

Prove R14–R26: furthest-failure reporting, the expectation vocabulary, context
frames, resync, error nodes, cascade control and the unchanged `run`.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0002 R14 | The furthest attempt is reported, expectations unioned |
| PLAN-2026-0002 R15, R16 | Typed expectations; did-you-mean reuse |
| PLAN-2026-0002 R18, R26 | Diagnostic shape; context frame |
| PLAN-2026-0002 R19–R23 | Cascade control, resync, error nodes, refusal to emit |
| PLAN-2026-0002 R24 | `Library::run` unchanged |

## Procedure

```sh
cargo test --manifest-path rust/Cargo.toml --test diagnostics
```

## Expected Results

14 tests pass, including the delimiter-fence case, which must leave the file
intact.

## Actual Results

14 passed, 0 failed.

| Test | Outcome |
|---|---|
| `furthest_attempt_is_reported` | PASS — names `` `)` ``, not the generic message |
| `diagnostic_carries_the_context_frame` | PASS — message contains ``in `fn` `` |
| `expectations_union_at_a_tie` | PASS — all three alternatives listed |
| `recovery_reports_every_broken_statement` | PASS — 3 statements, 2 diagnostics |
| `statements_after_a_failure_still_parse` | PASS |
| `an_old_walker_still_sees_only_statements` | PASS |
| `a_missing_delimiter_does_not_eat_the_file` | PASS |
| `cascade_is_suppressed` | PASS |
| `diagnostics_are_capped` | PASS |
| `emission_refuses_a_partial_parse` | PASS |
| `run_is_unchanged_by_recovery` | PASS |
| `a_valid_program_is_clean` | PASS |
| `recovery_always_terminates` | PASS |

## Result

PASS

## Evidence

Before and after on the motivating case:

```text
before:  error: no library function matches token "fn"
after:   error: expected `)`, found end of statement in `fn`
```

## Evidence Sources

- `rust/tests/diagnostics.rs` (committed)
- `cargo test --manifest-path rust/Cargo.toml --test diagnostics`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

**One real defect, caught by the test rather than in review.**
`a_missing_delimiter_does_not_eat_the_file` initially failed with **0 statements
surviving** — the resync scan tracked delimiter balance, but an unclosed `(`
holds the depth above zero to EOF, so it consumed the whole file. That is the
precise failure R21 exists to prevent. Fixed by treating a line break inside an
unclosed delimiter as evidence the delimiter is missing, and accepting a
statement-start token from that point.

## Related Documents

- PLAN-2026-0002
- PROP-2026-0001 UQ-06 (the error-recovery design note; tests A–K)

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
