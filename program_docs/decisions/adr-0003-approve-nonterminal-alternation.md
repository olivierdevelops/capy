---
document_id: ADR-0003
title: Approve PROP-2026-0004 — Ordered Alternation for Nonterminals
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
  - docs
  - samples

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Records the decision to approve PROP-2026-0004 revision 4, the two open questions it settles (OQ-02, OQ-08) and the order in which the increments are built.

reason: DOCUMENTATION.md section 3.2 places a Decision between a Proposal and its Plan, and section 24 requires an approved plan before implementation. PROP-2026-0004 named OQ-02 as blocking.

related_documents:
  - PROP-2026-0004
  - PLAN-2026-0003
  - ADR-0001
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - decision
  - grammar
  - alternation

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Approve PROP-2026-0004 — Ordered Alternation for Nonterminals

> **Status:** Approved
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs, samples

## Context

Capy has three of the four grammar combinators (sequence, repetition, recursion)
and no choice. A capture's type names exactly one function, so a rule cannot say
"an argument is a nested call OR an atom", and a parameter list cannot mix plain
and marked parameters. Both were reproduced on the repo build on 2026-10-07
(`PROP-2026-0004` P-01, P-05).

```text
  sequence    ✅     repetition  ✅     recursion  ✅     CHOICE  ❌  ← this decision
```

## Decision

Approve `PROP-2026-0004` revision 4, Option A: a capture type may name several
library functions separated by `|`, tried left to right, first match wins.

| Open question | Decision |
|---|---|
| **OQ-02** — representation without breaking `ARCH-001` | The existing single-name field `cap_type` / `type_` **stays and holds alternative 1**. An ordered `alts: Vec<String>` is added beside it and holds alternatives 2…n. No existing public field changes type |
| **OQ-08** — may an alternative be a built-in or declared type | **No.** Alternatives name **library functions** only. A flat alternative is written as a `bare` one-capture function, the pattern `PROP-2026-0004` P-04 already documents. `PROP-2026-0004` R1 is narrowed accordingly |
| **OQ-01** — `\|` vs the source token `\|>` | The type position is **library** syntax, not source. Confirmed by test, not assumed: the plan carries a lexing test (T-11) |
| **OQ-03** — cap on alternatives | No hard cap |
| **OQ-04**, **OQ-06**, **OQ-07** | Deferred. Not in this release's scope |

## Rationale

- Additive representation keeps `ARCH-001` satisfied and leaves every existing
  library byte-identical (R7).
- Restricting alternatives to functions reuses the one matcher path
  (`capture_func_type`) that already carries the depth bound, failure
  expectations and repetition. Mixing in flat types would add a second path with
  stop-literal handling and a second place for the left-recursion guard to miss.
- The left-recursion guard is built **first** (plan phase P1): the guard must
  exist before the feature that can produce the cycle (`RK-03`).

## Alternatives Rejected

- **Named unions (`PROP-2026-0004` Alternative B)** — a new top-level directive
  and namespace for the same power. Revisit under OQ-04.
- **Changing `cap_type` to a list** — source-breaking for every reader of
  `PatternElement` and `ArgInfo` (`ARCH-001`).
- **Allowing flat types as alternatives** — see Rationale.

## Consequences

- `ArgInfo` (public introspection) gains an `alts` field; `type_` keeps meaning
  alternative 1. `PatternElement` and `ArgEntry` gain the same.
- `capy docs` prints the union (`a | b | c`) in the Type column.
- This release is a **minor** bump, `0.22.0` → `0.23.0`.
- The release also carries `PROP-2026-0002` (public docs and samples, already
  `implemented`, previously uncommitted) by the owner's instruction on
  2026-10-07. `PLAN-2026-0003` records it as carried work and `RPT-2026-0003`
  validates it.

## Status

Approved for implementation. Approval authorizes the behaviour above. It does not
approve a built-in expression grammar (`PROP-2026-0001` Non-Goal 5), named unions,
or any change to the `any` value grammar.

Approval was given by the owner's explicit instruction to plan, implement,
validate, test and release. Agent validation did not approve the proposal
(`DOCUMENTATION.md` §21.2).

## Related Documents

- `PROP-2026-0004` — the proposal this approves
- `PLAN-2026-0003` — the plan that implements it
- `ADR-0001` — precedent for the Decision stage

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial document |
