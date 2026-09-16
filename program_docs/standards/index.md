---
document_id: STD-2026-0000
title: Project Standards Index
document_type: standard
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
  - All

affected_versions: not-applicable

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: The canonical entry point and current-state policy index for Capy's project standards — active documents, stable rule-ID ranges, approved exceptions and open conflicts.

reason: DOCUMENTATION.md section 4.26 makes this file mandatory, and section 21.2 requires a proposal to record the standards index revision it validated against. Both PROP-2026-0001 validations had to record NEEDS HUMAN REVIEW because no index existed.

related_documents:
  - REF-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - standards
  - index
  - governance

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Project Standards Index

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

The policy a proposal is validated against, per `DOCUMENTATION.md` §21.

**Index revision: 1** — cite this number in a proposal's *Project Standards
Baseline* table.

## Provenance

These rules **codify practice that was already binding**, not new policy. Every
one restates something already enforced by `AGENTS.md`, by the verification gates
run on every change, or by a decision recorded in `ADR-0001`. Nothing here was
invented for this index.

That matters for §21.1's requirement that draft rules do not govern proposals
until approved: these are marked `active` because they describe rules already in
force. A genuinely *new* rule would start as `draft` and would not govern
anything until an approval is recorded.

## Reading order

1. [Project goals](project-goals.md) — what Capy is for
2. [Engineering philosophy](engineering-philosophy.md) — how work is done
3. [Architecture rules](architecture-rules.md) — invariants of the engine
4. [Codebase rules](codebase-rules.md) — rules for changing this repository
5. [Quality expectations](quality-expectations.md) — required evidence and gates
6. [Exceptions](exceptions.md) — the register

## Active standards

| Document | ID | Revision | Status | Rules |
|---|---|---|---|---|
| [Project goals](project-goals.md) | STD-2026-0001 | 1 | active | GOAL-001, GOAL-002 |
| [Engineering philosophy](engineering-philosophy.md) | STD-2026-0002 | 1 | active | PHIL-001, PHIL-002 |
| [Codebase rules](codebase-rules.md) | STD-2026-0003 | 1 | active | CODE-001…003 |
| [Architecture rules](architecture-rules.md) | STD-2026-0004 | 1 | active | ARCH-001…003 |
| [Quality expectations](quality-expectations.md) | STD-2026-0005 | 1 | active | QUAL-001…003, GATE-001, GATE-002 |
| [Exceptions](exceptions.md) | STD-2026-0006 | 1 | active | — |

## Rule index

| ID | Rule | Enforcement |
|---|---|---|
| GOAL-001 | Capy ships zero source-language grammar | blocking |
| GOAL-002 | Engine changes are additive | blocking |
| PHIL-001 | Verify a claim before recording it | blocking for TEST/RPT/REL |
| PHIL-002 | Record deviations rather than smoothing them over | blocking |
| CODE-001 | Commit only when asked; stage by name | blocking |
| CODE-002 | The built-in helper list stays in sync | blocking |
| CODE-003 | The library keyword list stays in sync | blocking |
| ARCH-001 | A public field never changes type | blocking |
| ARCH-002 | `capy-core` keeps exactly one dependency | blocking |
| ARCH-003 | The engine never aborts its host | blocking |
| QUAL-001 | Regression is demonstrated across the whole corpus | blocking |
| QUAL-002 | A measurable claim is predeclared | blocking |
| QUAL-003 | A test that cannot fail for the right reason is not a test | advisory |
| GATE-001 | Pre-commit gate | blocking |
| GATE-002 | Regression gate | blocking |

## Stable ID ranges

| Prefix | Range | Owner |
|---|---|---|
| `GOAL` | 001–099 | Capy Engine |
| `PHIL` | 001–099 | Capy Engine |
| `CODE` | 001–199 | Capy Engine |
| `ARCH` | 001–199 | Capy Engine |
| `QUAL` | 001–199 | Capy Engine |
| `GATE` | 001–099 | Capy Engine |
| `EXC` | 001–999 | Capy Engine |

IDs are permanent and never reused. A withdrawn rule is marked superseded, not
deleted.

## Approved exceptions

None. See the [register](exceptions.md).

## Superseded rules

None.

## Unresolved conflicts

None between rules.

One conflict between a rule and reality is worth naming: **GOAL-002 (additive)
versus `#[non_exhaustive]`**. Marking an existing public struct `#[non_exhaustive]`
is itself source-breaking for a consumer who already wrote an exhaustive literal.
It was applied in 0.21.0 in the same change as the first new fields, while
`capy-core` was unpublished and had no external consumers, which is the cheapest
moment such a change ever gets. Doing it now means every later field addition is
additive. Recorded here rather than as an exception because it was a one-time
cost paid to make the rule holdable, not a departure from it.

## Recent changes

| Date | Change |
|---|---|
| 2026-09-16 | Index created; STD-2026-0001…0006 authored at revision 1, codifying existing practice |
| 2026-09-16 | References retargeted from `CLAUDE.md` to `AGENTS.md`, which is now the authoritative instruction file. The rules are unchanged — only where a contributor reads them moved. Historical documents (proposals, plans, releases) keep their original citation, because they record what was validated at the time |

## Retroactive validation

`PROP-2026-0001` and both its plans were validated before this index existed, and
recorded `NEEDS HUMAN REVIEW` for the missing standards set. They have since been
revalidated against revision 1 — see each document's *Project Validation* table.
No result changed from `PASS`, which is expected: the rules describe what those
changes were already held to.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial index |
| 2 | 2026-09-16 | Olivier | Retargeted rule provenance from `CLAUDE.md` to `AGENTS.md` |
