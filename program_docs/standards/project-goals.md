---
document_id: STD-2026-0001
title: Project Goals
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

scope: The goals every proposal is measured against.

reason: PROP-2026-0001 could not be validated against project goals because none were recorded.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - goals
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Project Goals

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

Two goals. Both are already stated in `CLAUDE.md`; this records them as rules
with IDs so a proposal can be validated against them.

---

## GOAL-001 — Capy ships zero source-language grammar

**Intent.** A `.capy` library defines the source language. The engine supplies
lexing, matching and rendering, and no opinions about what a statement looks
like.

**Rationale.** It is the property that distinguishes Capy from a templating
engine and from a parser generator. One source can render to many targets
precisely because the engine holds no grammar of its own.

**Scope.** The engine, the CLI, and every proposal that adds surface syntax.

**Required.** New surface syntax is expressed as library directives or capture
types. Engine-level grammar is confined to the fixed structures the engine
already owns: value expressions, the inner DSL, and the manifest format.

**Prohibited.** Adding a built-in keyword, statement shape or expression grammar
for *source* languages.

**Example.** PROP-2026-0001 R10 added infix operators to **value expressions**
— the engine's own fixed grammar — and explicitly declined to add a built-in
expression grammar for source languages (Non-Goal 5).

**Validation.** Review of any proposal that touches parsing; `docs/index.md`'s
"no built-in keywords" claim must stay true.

**Enforcement.** Blocking. **Owner.** Capy Engine.
**Exception process.** An `EXC-NNN` record naming approver, reason, scope, expiry.

---

## GOAL-002 — Engine changes are additive

**Intent.** No library that works today stops working.

**Rationale.** Libraries live in users' repositories, not this one. A breaking
engine change is a breaking change to every downstream project at once, and they
have no way to detect it before it bites.

**Scope.** Every change to `capy-core`.

**Required.** New behaviour is reachable only through new syntax or a new API.
Regression is demonstrated across **all** checked-in libraries and the full
golden corpus, not a sample.

**Prohibited.** Changing what existing syntax means. Changing the type of an
existing public field.

**Example.** Comment retention (R27) emits comment tokens only from a new lexer
entry point, and the parser strips them before matching — so no matcher sees a
token kind it was not written for.

**Validation.** `GATE-002`.

**Enforcement.** Blocking. **Owner.** Capy Engine.
**Exception process.** As GOAL-001; a major version bump is the usual answer.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
