---
document_id: REL-0.21.0
title: Release 0.21.0 — Parser Foundations
document_type: release
status: draft

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 1

authors:
  - Olivier

owner: Release Management
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - capy-cli

affected_versions:
  from: "0.21.0"
  to: "0.21.0"

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Records what shipped in 0.21.0, how each update was verified, and the source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests, validation and Git tag.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001
  - ADR-0001
  - RPT-2026-0001
  - DEMO-2026-0001
  - TEST-2026-0001
  - TEST-2026-0002
  - TEST-2026-0003
  - TEST-2026-0004

supersedes: null
superseded_by: null

tags:
  - release
  - parser
  - spans

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Release 0.21.0 — Parser Foundations

> **Status:** Draft
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Release Management
> **Affected Components:** capy-core, capy-cli

## Summary

The first of five increments from `PROP-2026-0001`. It removes a defect that
could kill a program embedding Capy, and gives every AST node a real source
position — the prerequisite for the diagnostics, recovery and AST output that
follow in PLAN-B through PLAN-E.

## Release Identity

| Field | Value |
|---|---|
| Version | 0.21.0 |
| Previous version | 0.12.0 |
| Tag | `v0.21.0` |
| Commit | *pending — recorded at tagging* |
| Version source | `rust/Cargo.toml` `[workspace.package] version` |

## Plan

`PLAN-2026-0001` (PLAN-A of `PROP-2026-0001`). Decision recorded in `ADR-0001`.

## Project Standards Baseline

| Standards Index | Revision | Applicable Rules | Proposal Validation |
|---|---|---|---|
| `program_docs/standards/index.md` | **does not exist** | none defined | de facto rules from `CLAUDE.md` assessed in their place |

No formal standards set has been authored; `PROP-2026-0001` OQ-10 asks whether to
create one. Recorded rather than left blank.

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| PROP-2026-0001 R0 | `capy_error.md` step 0 — "left-recursion guard; correctness bug" | U-01, U-02 | PASS |
| PROP-2026-0001 R0b | as above | U-01 | PASS |
| PROP-2026-0001 R1 | `needs2.md` §2 — "no byte offsets, no end positions, no per-capture spans" | U-03 | **PARTIAL** — byte offsets deferred |
| PROP-2026-0001 R2–R4 | as above | U-03 | PASS |
| PROP-2026-0001 R5 | internal — additive-change rule | — | PASS |
| PROP-2026-0001 R27 | review finding 3 — comment retention unstated | U-04 | PASS |

## Added

- `Span` on `FuncCall` and `CaptureValue` — start and end line/column, exclusive
  end. Both structs are `#[non_exhaustive]`, so future fields (byte offsets) will
  not break consumers.
- `FuncCall.leading_comments` — spans of the comments immediately above a
  statement.
- `TokenKind::Comment` and `make_lexer::tokenize_with_trivia`.
- `samples/left-recursion-rejected/` — a sample whose golden is the error.

## Changed

- Nested nodes from function-as-type captures now report their real position
  instead of `line: 0, col: 0`.
- A statement's span extends over its block body and closer.
- Workspace version 0.12.0 → 0.21.0.

## Fixed

- **A left-recursive library no longer aborts the process.** It previously passed
  `capy check` and then exited 134 with a stack overflow, which an embedding
  program could not catch. It is now refused at load with the cycle named.
- **Deeply nested input no longer aborts.** Nonterminal descent is bounded at 64
  levels. This was a *separate* defect from the one above: a valid right-recursive
  library was enough to trigger it.

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP-2026-0001 R0b | Left-recursive library refused at load | `capy check bad.capy` | Exit 1, cycle named | TEST-2026-0002, DEMO-2026-0001 |
| U-02 | PROP-2026-0001 R0 | Deep input refused, not fatal | `capy run ok.capy deep.capy` | Exit 1, never 134 | TEST-2026-0002, DEMO-2026-0001 |
| U-03 | PROP-2026-0001 R4 | Nested nodes carry real spans | `cargo test --test ast_spans` | 13 pass | TEST-2026-0001 |
| U-04 | PROP-2026-0001 R27 | Comments retained, output unchanged | `capy run c.capy s1.capy` | `hi world`, same as without | TEST-2026-0003 |

## Tests

| Test | Requirement | Result |
|---|---|---|
| TEST-2026-0001 | PLAN-2026-0001 R1–R5 | PASS |
| TEST-2026-0002 | PLAN-2026-0001 R0, R0b | PASS |
| TEST-2026-0003 | PLAN-2026-0001 R12, R27 | PASS |
| TEST-2026-0004 | PLAN-2026-0001 M-01, M-02, M-03 | PASS |

Gate summary: 90 tests pass · 0 clippy findings at `--all-targets` · `capy check`
117/117 · goldens 117 pass / 0 fail · wasm 113 pass / 0 fail · `mkdocs --strict`
exit 0.

## Validation

`RPT-2026-0001` — all requirements `PASS` except R1, which is `PARTIAL`. No
`FAIL`. The `PARTIAL` is a recorded deferral with a migration-safe
representation, not an unmet requirement.

## Measured Results

| Measurement | Baseline | Threshold | Observed | Result |
|---|---|---|---|---|
| Transpile time | 187–219 µs | ≤ 10 % regression | 188.185 µs best of 5 | PASS |
| wasm module size | 1 342 891 bytes | ≤ 5 % growth | 1 367 674 bytes (+1.85 %) | PASS |
| `capy-core` direct deps | 1 | exactly 1 | 1 (`regex`) | PASS |

Baselines were frozen before implementation per §21.3 and are not editable
retroactively.

## Known Limitations

1. No byte offsets on `Span`; line/column only.
2. The AST is not yet a public API — `Library::parse` ships in PLAN-D.
3. The depth-limit error reports the generic "no library function matches".
4. Only leading comments are attached.

## Upgrade Notes

Additive for `.capy` libraries — every existing library loads unchanged, and
golden output is byte-identical. The only libraries affected are ones that were
already broken: a left-recursive grammar now fails to load instead of crashing.

For Rust consumers: `FuncCall` and `CaptureValue` are now `#[non_exhaustive]`.
Exhaustive struct literals and patterns over them must use the constructors.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial release document; commit and tag pending |
