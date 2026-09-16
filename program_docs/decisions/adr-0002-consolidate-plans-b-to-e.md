---
document_id: ADR-0002
title: Consolidate PLAN-B through PLAN-E into One Plan and One Release
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
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Records the decision to implement the last four increments of PROP-2026-0001 as a single plan and a single release, departing from the proposal's plan map.

reason: The proposal's plan map assigns PLAN-B through PLAN-E to four plans; departing from it is a decision, and section 3.2 places a Decision before the Plan that acts on it.

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

# Consolidate PLAN-B through PLAN-E into One Plan and One Release

> **Status:** Approved
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0 and later
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Context

`PROP-2026-0001` maps its requirements to five plans. PLAN-A shipped as 0.21.0.
The remaining four — diagnostics, recovery, structured output and operator
precedence — were to be four plans and, by implication, four releases.

`DOCUMENTATION.md` §4.5 permits several plans "when work has independently
approvable or **releasable** increments". That condition is the question.

## Decision

Implement PLAN-B through PLAN-E as **one plan** (`PLAN-2026-0002`) and **one
release** (0.22.0).

## Rationale

Their types layer rather than stand alone:

```text
  B defines  Diagnostic
  C puts it in ParseResult, beside the tree, and adds ErrorNode
  D serializes both, and freezes a JSON schema over them
  E is independent, but small
```

Releasing B alone publishes a `Diagnostic` that C immediately extends with
labels from recovery. Releasing C alone publishes a `ParseResult` whose JSON
form D then fixes. Four releases would mean **three public shapes that each
exist for one version** — and for `capy-core`, whose consumers pin a git rev,
that is three unnecessary migrations.

The proposal makes this argument itself, about PLAN-D:

> "Emits tree **and** diagnostics; shipping it earlier would freeze a schema that
> recovery then changes."

The decision extends that reasoning backwards through the chain. B, C and D are
therefore **not independently releasable**, so §4.5's condition for splitting is
not met.

## Alternatives Rejected

| Alternative | Why rejected |
|---|---|
| Four plans, four releases | Publishes three shapes that live one version each |
| Four plans, one release | Four ledgers for one release gate; the traceability §4.5 asks for would be spread across four Decisions tables with no single status summary |
| Three plans (B+C+D together, E separate) | E is genuinely independent, but it is small and shares the release; a separate plan for one requirement pair is ceremony without a gate |

## Consequences

- One status summary covers four phases' worth of findings, which is the cost.
  Recorded in `PLAN-2026-0002`'s Post-Implementation Review: this is correct here
  and should not become a habit. Where increments *are* independently releasable,
  four ledgers give four honest status summaries.
- The proposal's plan map is now inaccurate. It is left as written and this
  decision is cited from the plan, rather than editing the approved proposal to
  match what was built — see `PHIL-002`.
- 0.22.0 carries a larger upgrade note than four smaller releases would have.

## Status

Approved. Implemented as `PLAN-2026-0002`, released as 0.22.0.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial decision, recorded retrospectively after the consolidation was carried out |
