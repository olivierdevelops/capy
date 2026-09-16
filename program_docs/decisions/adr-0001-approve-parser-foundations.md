---
document_id: ADR-0001
title: Approve PROP-2026-0001 and its Frozen Contracts
document_type: decision
status: approved

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
  - architects

scope: Records the decision to approve PROP-2026-0001 and the contracts frozen before implementation began.

reason: DOCUMENTATION.md section 3.2 places a Decision between a Proposal and its Plan; PLAN-2026-0001 gates P2 entry on it.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - decision
  - parser

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Approve PROP-2026-0001 and its Frozen Contracts

> **Status:** Approved
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Context

`PROP-2026-0001` proposes five additive parser changes. Three consumer reports
(`needs.md`, `needs2.md`, `capy_error.md`) identified the blockers; one of them —
a left-recursive library aborting the host process with rc=134 — was reproduced
during assessment.

## Decision

Approve `PROP-2026-0001` revision 4 and proceed with PLAN-A. The following
contracts are frozen:

| Contract | Decision |
|---|---|
| `Span` indexing | 1-indexed, source-absolute, matching `Token.line`/`col` |
| `Span.end_col` | **Exclusive** — one past the last byte, so `end - start` is a width |
| Byte offsets | Deferred; `Span` is `#[non_exhaustive]` so they can be added without breaking consumers |
| Comment attachment | **Leading only**, per the consumer's answer to OQ-12 |
| Node span vs comments | The node's span **excludes** its attached comments |
| Error-node representation | A parallel `Block.errors`, not a change to `Block.stmts`' type (PLAN-C) |
| Left recursion | Rejected at **library-load** time, so `capy check` is the gate |

## Alternatives Rejected

- **Error nodes as `Block.stmts: Vec<Node>`** — source-breaking for every walker.
- **Error nodes as a reserved `__error` function name** — an un-updated walker
  would compile and silently treat an error as real code. A loud compile error is
  better than a silent wrong answer.
- **Raising the stack limit instead of guarding recursion** — moves the cliff
  without removing it, and leaves `capy check` reporting `ok`.

## Consequences

- `capy-core`'s public AST types gain fields; `#[non_exhaustive]` must be applied
  in the *same change*, or the attribute is useless to anyone who already wrote an
  exhaustive literal.
- The five plans are sequenced; PLAN-A blocks B, C and D.
- Dataflow analysis (lattice, graphs, monomorphization) is explicitly **not**
  Capy's responsibility and stays with the consumer.

## Status

Approved for implementation. Approval authorizes the behaviour and contracts
above; it does not approve the JSON schema as stable, nor the cascade constants,
both of which belong to later plans.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document |
