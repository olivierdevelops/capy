---
document_id: REL-0.22.0
title: Release 0.22.0 — Diagnostics, Recovery, AST Output and Precedence
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
  from: "0.22.0"
  to: "0.22.0"

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Records what shipped in 0.21.0, how each update was verified, and the source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests, validation and Git tag.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001
  - ADR-0001
  - RPT-2026-0002
  - DEMO-2026-0002
  - TEST-2026-0005
  - TEST-2026-0006
  - TEST-2026-0007
  - TEST-2026-0008

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

# Release 0.22.0 — Diagnostics, Recovery, AST Output and Precedence

> **Status:** Draft
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Release Management
> **Affected Components:** capy-core, capy-cli

## Summary

The remaining four increments of `PROP-2026-0001`, shipped together. With 0.21.0
this completes the proposal: all three blockers a downstream consumer reported
are addressed.

## Release Identity

| Field | Value |
|---|---|
| Version | 0.22.0 |
| Previous version | 0.21.0 |
| Tag | `v0.22.0` |
| Commit | *pending — recorded at tagging* |
| Version source | `rust/Cargo.toml` `[workspace.package] version` |

## Plan

`PLAN-2026-0002`, consolidating PLAN-B…PLAN-E. Decision in `ADR-0001`.

## Project Standards Baseline

| Standards Index | Revision | Applicable Rules | Proposal Validation |
|---|---|---|---|
| `program_docs/standards/index.md` | **does not exist** | none defined | de facto rules from `CLAUDE.md` assessed in their place |

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| PROP-2026-0001 R14–R18, R26 | `capy_error.md` Mechanisms 1–2 | U-01 | PASS |
| PROP-2026-0001 R19–R24 | `capy_error.md` Mechanisms 3–4 | U-02 | PASS |
| PROP-2026-0001 R6–R9, R28 | `needs2.md` §3 — no structured output | U-03 | PASS |
| PROP-2026-0001 R10, R11 | `needs2.md` §1 — no operator precedence | U-04 | PASS |

## Added

- `Diagnostic`, `Label`, `Severity`, `ContextFrame` and stable diagnostic codes.
- `ErrorNode` and `Block.errors` — a **parallel** field, so `Block.stmts` keeps
  its type and every existing walker still compiles.
- `ParseResult` and `Library::parse`.
- `capy ast <library> <script> [--json]`.
- Infix `*` `/` `%` `+` `-`, comparisons, `and` / `or`, with precedence and
  left associativity; plus grouping parentheses where unambiguous.
- `docs/diagnostics.md` and `docs/ast-json.md`.

## Changed

- A failed parse reports the shape that got furthest and what it wanted there,
  instead of "no library function matches".
- Comparison is precedence-ordered rather than a single non-associative step.
- One error golden (`samples/bbcode-parser/mismatch`) now carries a more precise
  message. Reviewed deliberately, not regenerated.
- Workspace version 0.21.0 → 0.22.0.

## Fixed

Nothing was broken; this release adds capability.

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP R14 | Errors name what was expected | `capy run lib.capy a.capy` | ``expected `)`, found …`` | TEST-2026-0005, DEMO-2026-0002 |
| U-02 | PROP R20 | Parsing recovers | `capy ast lib.capy b.capy` | 3 statements, 2 errors | TEST-2026-0005, DEMO-2026-0002 |
| U-03 | PROP R7 | Tree as JSON | `capy ast … --json` | one JSON document | TEST-2026-0007 |
| U-04 | PROP R10 | Precedence | `capy run p.capy q.capy` | 7, then 9 | TEST-2026-0006 |

## Tests

| Test | Requirement | Result |
|---|---|---|
| TEST-2026-0005 | R14–R26 | PASS |
| TEST-2026-0006 | R10–R12 | PASS |
| TEST-2026-0007 | R6–R9, R28 | PASS |
| TEST-2026-0008 | M-01…M-03, R13, R25 | PASS |

Gate summary: 110 tests · 0 clippy findings at `--all-targets` · `capy check`
117/117 · goldens 117/0 · wasm 114/0 · `mkdocs --strict` exit 0.

## Validation

`RPT-2026-0002` — every requirement `PASS`. No `FAIL`, no `PARTIAL`.

## Measured Results

| Measurement | Baseline | Threshold | Observed | Result |
|---|---|---|---|---|
| Transpile time | 187–219 µs | ≤ 10 % regression | 192.036 µs best of 5 | PASS |
| wasm module size | 1 342 891 bytes | ≤ 5 % total | 1 386 117 bytes (+3.22 %) | PASS |
| `capy-core` direct deps | 1 | exactly 1 | 1 (`regex`) | PASS |

The 5 % wasm allowance is shared across the release series; 3.22 % is now spent.

## Known Limitations

1. Byte offsets still absent from `Span`.
2. The nesting-depth error still reports the generic message.
3. Grouping parentheses are conditional — `(foo)` remains a zero-argument call.
4. Cascade constants are defaults, untuned against a real corpus.
5. The wasm ABI does not expose the AST or diagnostics.

## Upgrade Notes

Additive for `.capy` libraries: all 117 sample libraries load unchanged and
golden output is byte-identical, apart from one error message that improved.

For Rust consumers:

- `Block` gains `errors` and is `#[non_exhaustive]`; use `..Default::default()`
  in struct literals.
- `Severity` and `Diagnostic` are `#[non_exhaustive]` — a `match` on `Severity`
  in an external crate needs a wildcard arm.
- `Library::run` is unchanged.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial release document; commit and tag pending |
