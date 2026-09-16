---
document_id: STD-2026-0002
title: Engineering Philosophy
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

scope: How work is done and reported here.

reason: Two habits repeatedly changed outcomes during PROP-2026-0001 and were worth making explicit.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - philosophy
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Engineering Philosophy

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

These are descriptive, not aspirational: both were observed to change outcomes
during the 0.21.0 and 0.22.0 work.

---

## PHIL-001 — Verify a claim before recording it

**Intent.** A statement in a document, a commit message or a code comment is
checked against the system before it is written down.

**Rationale.** During PROP-2026-0001 this caught, among others: a README example
using a directive that no longer exists; a documented `--version` flag that never
worked; and a "both shapes work" claim about `run:` that was false. Each had been
in the repository for some time and each would have survived another review.

**Scope.** Documents, commit messages, comments, and any report of a result.

**Required.** Run the command, read the file, or execute the example. Cite the
evidence. Where a claim cannot be verified, say so rather than asserting it.

**Prohibited.** Reporting a result that was not observed. Describing behaviour
from memory of the design.

**Example.** `docs/library-authoring.md`'s left-recursion example was executed
before publication, and `capy docs` was run to confirm the auto-prepended literal
it describes.

**Validation.** Review. Evidence sources are mandatory in `TEST` documents (§27.1).

**Enforcement.** Blocking for `TEST`, `RPT` and `REL`; advisory elsewhere.
**Owner.** Capy Engine. **Exception process.** None; state the uncertainty instead.

---

## PHIL-002 — Record deviations rather than smoothing them over

**Intent.** When implementation diverges from plan, the divergence is recorded
with its reason, not quietly absorbed.

**Rationale.** Two of the most useful findings in this project came from
deviations: that a load-time recursion guard does **not** satisfy "never abort"
(deeply nested input still did), and that "never resync inside an unclosed
bracket" is correct only for brackets that eventually close. Both would have been
invisible had the fix simply been applied.

**Scope.** Plans, validation reports, release documents.

**Required.** A deviation names what was planned, what happened, why, and what it
changes for later work. A `PARTIAL` result is recorded as `PARTIAL`.

**Prohibited.** Editing a plan to match what was built. Reporting `PASS` for a
requirement that was descoped.

**Example.** RPT-2026-0001 records R1 as `PARTIAL` — byte offsets deferred —
rather than narrowing R1 to what shipped.

**Validation.** Review of every `RPT`.

**Enforcement.** Blocking. **Owner.** Capy Engine. **Exception process.** None.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
