---
document_id: STD-2026-0006
title: Approved Exceptions
document_type: standard
status: active

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
  - All

affected_versions: not-applicable

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: The register of approved exceptions to active rules.

reason: Section 21.2 requires an exception to name its approver, reason, scope and expiry; a register is where those live.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - exceptions
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Approved Exceptions

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

**No exceptions are currently approved.**

## Register

| ID | Rule | Scope | Reason | Approver | Granted | Expires | Status |
|---|---|---|---|---|---|---|---|
| — | — | — | — | — | — | — | *(none)* |

## How to request one

An exception names the rule, the exact scope it covers, the reason the rule
cannot be met, the approver, and an expiry date. It is recorded here before the
work proceeds, and cited from the proposal's *Project Validation* table with the
result `EXCEPTION REQUESTED`.

An exception without an expiry is not an exception; it is an undocumented rule
change.

## Near misses

Recorded because they look like exceptions and are not:

| Item | Why it is not an exception |
|---|---|
| Byte offsets deferred from `Span` (R1 `PARTIAL`) | No rule requires them. The requirement was partially met and recorded as `PARTIAL` in RPT-2026-0001 |
| PLAN-B…E consolidated into one plan | `DOCUMENTATION.md` §4.5 permits one plan when increments are not independently releasable. Recorded as a deviation from the proposal's plan map, not from a rule |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
