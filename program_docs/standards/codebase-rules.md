---
document_id: STD-2026-0003
title: Codebase Rules
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

scope: Rules governing changes to the repository itself.

reason: These are already binding via CLAUDE.md; recording them as rule IDs lets a proposal validate against them.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - codebase
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Codebase Rules

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

Verbatim restatements of `CLAUDE.md`, given IDs. `CLAUDE.md` remains the copy a
contributor reads; this is the copy a proposal validates against.

---

## CODE-001 — Commit only when asked; stage by name

**Required.** Commit only on explicit request. Stage files by name. Never
`git add -A`. Never commit `.ignore/` contents. Never force-push `main`. Never
`--no-verify`.

**Rationale.** The working tree routinely contains the user's own in-progress
work; a blanket stage sweeps it into someone else's commit.

**Example.** `AGENTS.md` and `CLAUDE.md` carried uncommitted user edits
throughout the 0.21.0 and 0.22.0 work and were left untouched in every commit.

**Validation.** Review of the staged set before each commit.
**Enforcement.** Blocking. **Owner.** Capy Engine.

---

## CODE-002 — The built-in helper list stays in sync

**Required.** Adding, renaming or removing a helper in
`rust/src/infra/helpers.rs` updates `docs/function-cookbook.md` (table **and**
worked example), `samples/builtin-functions/` with a regenerated golden, and
`docs/whats-new.md` — in the same change.

**Rationale.** The cookbook is the only place a user discovers a helper.

**Validation.** `GATE-001` plus review. **Enforcement.** Blocking.
**Owner.** Capy Engine.

---

## CODE-003 — The library keyword list stays in sync

**Required.** Adding, renaming or removing a library directive or capture type
updates `docs/library-keywords.md`, `docs/features.md` and
`docs/CAPY_FOR_LLMS.md` in the same change.

**Rationale.** Source of truth is the directive switches in
`rust/src/infra/capy_lib_parser.rs` and the capture-type list in
`make_library_loader.rs`; documentation drifts from them silently.

**Validation.** Review. **Enforcement.** Blocking. **Owner.** Capy Engine.

**Recorded non-applicability.** PLAN-2026-0001 and PLAN-2026-0002 both assessed
this `NOT APPLICABLE` — neither added a directive or capture type.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
