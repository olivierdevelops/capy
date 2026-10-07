---
document_id: ADR-0004
title: Approve PROP-2026-0005 — Ordered-Choice Follow-Ups
document_type: decision
status: approved

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 1

approval:
  approved_by:
    - Olivier
  approved_date: 2026-10-07

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - capy-cli
  - capy-wasm-abi
  - docs

affected_versions:
  from: "0.24.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Records the decision to approve PROP-2026-0005 and the contracts it freezes.

reason: DOCUMENTATION.md section 3.2 places a Decision between a Proposal and its Plan.

related_documents:
  - PROP-2026-0005
  - PLAN-2026-0004
  - ADR-0003
  - REL-0.23.0

supersedes: null
superseded_by: null

tags:
  - decision
  - diagnostics

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Approve PROP-2026-0005 — Ordered-Choice Follow-Ups

> **Status:** Approved
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Context

`REL-0.23.0` shipped with one `PARTIAL` result (the nesting-bound message) and two
documented limitations (the browser JSON omits `alts`; `capy version` prints `dev`).

## Decision

Approve `PROP-2026-0005` revision 1 and release it as `0.24.0`.

| Contract | Decision |
|---|---|
| Bound error | remembered in `OuterP.depth_err`, reported only when the statement would otherwise fail; the rewind contract of `match_one` is **unchanged** |
| Code | `E0003` (`NESTING_TOO_DEEP`), reserved since 0.22.0, is now emitted; `E0001` keeps its meaning; `E0002` stays reserved |
| Leakage | `depth_err` and the failure code are reset at every `parse_stmt` entry |
| The bound | stays 64 captures (31 call levels in the sample grammar) |
| Browser JSON | `alts` added after `type`; `type` keeps meaning alternative 1; `optional` / `default` stay out of scope |
| Version string | unstamped builds print the crate version; a stamped `CAPY_VERSION` wins |
| Process | the implementation was drafted before this document existed, then **set aside and re-applied after the plan was committed** so that the thresholds were frozen before any measurement (DOCUMENTATION.md §21.3) |

## Alternatives Rejected

- Hard-propagating the bound through `match_one` — changes the rewind contract (R7).
- Raising the bound — moves the cliff.
- A build script to stamp the version — a step and a dependency for what `env!` does.

## Consequences

- One error message changes for over-deep input; no golden depended on it.
- `docs/diagnostics.md` stops calling `E0003` reserved.
- Minor bump `0.23.0` → `0.24.0`.

## Status

Approved for implementation. Approval was given by the owner's explicit instruction.
Agent validation did not approve the proposal (§21.2).

## Related Documents

- `PROP-2026-0005`, `PLAN-2026-0004`, `ADR-0003`, `REL-0.23.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial document |
