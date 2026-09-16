---
document_id: IDX-2026-0004
title: Decision Index
document_type: reference
status: active

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 1

authors:
  - Olivier

owner: Engineering Documentation Team
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - docs

affected_versions: not-applicable

applicable_environments:
  - development

audience:
  - engineers
  - release-managers

scope: Lookup of decisions, their status and the contracts they froze.

reason: DOCUMENTATION.md section 17 requires generated indexes allowing lookup by type, component, version, date, owner and lifecycle status.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - index
  - governance

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Decision Index

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Engineering Documentation Team

Lookup by decision, per `DOCUMENTATION.md` §17.

| ID | Decision | Status | Date | Supersedes | Related |
|---|---|---|---|---|---|
| ADR-0001 | Approve PROP-2026-0001 and freeze the `Span`, comment-attachment and error-node contracts | approved | 2026-09-16 | — | PROP-2026-0001, PLAN-2026-0001 |
| ADR-0002 | Consolidate PLAN-B through PLAN-E into one plan and one release | approved | 2026-09-16 | — | PROP-2026-0001, PLAN-2026-0002 |

## Contracts frozen by ADR-0001

| Contract | Decision |
|---|---|
| `Span` indexing | 1-indexed, source-absolute |
| `Span.end_col` | exclusive |
| Byte offsets | deferred; `#[non_exhaustive]` keeps the door open |
| Comment attachment | leading only |
| Node span vs comments | span excludes attached comments |
| Error nodes | parallel `Block.errors`, not a change to `stmts`' type |
| Left recursion | rejected at library-load time |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial index |
