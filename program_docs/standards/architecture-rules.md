---
document_id: STD-2026-0004
title: Architecture Rules
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

scope: Invariants of the engine's structure and public surface.

reason: Two of these were discovered the hard way during PROP-2026-0001 and are cheap to state, expensive to relearn.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - architecture
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Architecture Rules

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

---

## ARCH-001 — A public field never changes type

**Intent.** Growing a public type is allowed; reshaping one is not.

**Rationale.** `#[non_exhaustive]` protects a struct's *growth* — it does not
protect an existing field's *type*. Widening `Block.stmts` from
`Vec<FuncCall>` to `Vec<Node>` would have failed to compile for every consumer
that iterates it.

**Required.** New information goes in a new field. `#[non_exhaustive]` and a
constructor are applied **in the same change** as the first new field — applying
them afterwards is a no-op for anyone who already wrote an exhaustive literal.

**Prohibited.** Changing the type of an existing public field. Encoding new
meaning into an existing field's value space where a consumer would misread it.

**Example.** Error regions went into a parallel `Block.errors`. The rejected
alternative — an error as a `FuncCall` with a reserved `__error` name — would
have let an un-updated walker compile, run, and silently treat an error as real
code: a loud compile error traded for a silent wrong answer.

**Validation.** A compatibility test that builds a consumer written against the
previous shape. **Enforcement.** Blocking. **Owner.** Capy Engine.

---

## ARCH-002 — `capy-core` keeps exactly one dependency

**Intent.** The engine depends on `regex` and nothing else.

**Rationale.** It is embedded in other people's programs and compiled to wasm.
Every dependency is inherited by both, and the module size is a published number.

**Required.** New functionality uses the standard library or existing in-crate
modules. JSON is written with `gojson`, not serde.

**Validation.** `cargo tree -p capy-core --depth 1` reports exactly one.
Automated in `GATE-001` via `devtools/check_publishable.py`.

**Enforcement.** Blocking. **Owner.** Capy Engine.
**Exception process.** An `EXC-NNN` with a measured size and compile-time cost.

---

## ARCH-003 — The engine never aborts its host

**Intent.** A library or an input can produce an error. It cannot produce a
process abort.

**Rationale.** `capy-core` is embedded. A stack overflow cannot be caught as
`Err`; it takes down the host, and a library author has no way to know they
wrote one. This was a live defect: a left-recursive library passed `capy check`
and then exited 134.

**Required.** Recursive descent is bounded. Unbounded structures are rejected
with a `CapyError` at the earliest point they can be detected — preferably at
library load, so `capy check` is the gate.

**Prohibited.** Relying on stack size. A limit tuned to the main thread is not
sufficient: a Rust test thread gets 2 MiB against the main thread's 8 MiB, and a
limit that passes via the CLI can still abort under `cargo test`.

**Validation.** `rust/tests/recursion_guard.rs`; any new recursive path adds a
depth case. **Enforcement.** Blocking. **Owner.** Capy Engine.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
