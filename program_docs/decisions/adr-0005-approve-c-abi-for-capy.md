---
document_id: ADR-0005
title: Approve PROP-2026-0008 — A C ABI for Capy
document_type: decision
status: approved

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 1

approval:
  approved_by:
    - Olivier
  approved_date: 2026-10-13

authors:
  - Olivier
  - Claude (Sonnet 5.5)

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-ffi
  - capy-core
  - capy-cli
  - docs

affected_versions:
  from: "0.26.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Records the decision to approve PROP-2026-0008 and the contracts it freezes.

reason: DOCUMENTATION.md section 3.2 places a Decision between a Proposal and its Plan.

related_documents:
  - PROP-2026-0008
  - PLAN-2026-0005
  - ADR-0004

supersedes: null
superseded_by: null

tags:
  - decision
  - ffi
  - c-abi

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Approve PROP-2026-0008 — A C ABI for Capy

> **Status:** Approved
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Context

The Capylar project needs to call Capy from a C-ABI host. Capy has no C artifact, no handle API, no
tree or expression access outside Rust, and no ABI contract (`PROP-2026-0008` P-01…P-08). The owner
asked, on 2026-10-13, for the proposal to be planned, implemented, validated and released:
"plan implement validate and release, follow documentation.md".

## Decision

Approve `PROP-2026-0008` revision 1 with every recommended answer, and release it as `0.26.0`.

| Decision | Outcome |
|---|---|
| D1 crate location | new workspace member `capy-ffi` in this repository |
| D2 interface | opaque handles, `u32` ids, borrowed UTF-8 `(ptr, len)` strings; JSON as an optional second layer |
| D3 panic policy | dedicated `[profile.ffi]` with `panic = "unwind"`; every export under `catch_unwind`; `capy_panics_are_caught()` reports whether the build can catch (a build under `panic = "abort"` still works but cannot convert a panic) |
| D4 JSON | `expr` field is opt-in (`--with-expr`, `CAPY_JSON_EXPR`); default output byte-identical |
| D5 byte offsets | deferred to a separate proposal; `CapySpan` stays four `u32` line/column values |
| D6 scope | parsing only; `run`, `introspect`, docs rendering and `set_host` are not exposed |
| D7 release | minor version `0.26.0`, ABI version 1 |

## Deviations frozen at approval

Recorded now so they are decisions, not discoveries:

1. **`CAPY_E_LOAD` is added** for a grammar that fails to compile (the proposal listed no status for it).
2. **`CAPY_E_OOM` is replaced by `CAPY_E_LIMIT`**: Rust aborts on allocation failure, so the code could never be returned; the real failure is a parse whose tree exceeds the 2^24 − 2 id space.
3. **`rlib` is added to `crate-type`** so integration tests can import the crate; the C artifacts are unchanged.
4. **Foreign-id detection is best effort**: an 8-bit per-parse tag in the id catches a mix-up with high probability, not with certainty. Out-of-range, wrong-kind and stale-after-free use are the guaranteed cases; a freed handle is a caller contract violation.
5. **T-12 platform**: only macOS arm64 can be run here. The second platform is a cross-build of the static library for `aarch64-unknown-linux-gnu`, not a run.
6. **Diagnostic message**: `capy_diag_message` returns `msg`; `capy_diag_context` returns the frames separately, and the differential test composes `full_message` itself, so the C view and the JSON `message` field are compared on equal terms.

## Alternatives Rejected

As in `PROP-2026-0008` §Alternatives Considered (B to H).

## Consequences

- A new public ABI that the project must keep additive within version 1.
- `[profile.ffi]` exists at the workspace root; `capy-cli` and the wasm build keep `panic = "abort"`.
- Minor bump `0.25.0` → `0.26.0`. The release also carries the performance work already recorded under `[Unreleased]` in `CHANGELOG.md` (`PROP-2026-0007`).

## Status

Approved for implementation by the owner's explicit instruction. Agent validation did not approve the
proposal (§21.2).

## Related Documents

- `PROP-2026-0008`, `PLAN-2026-0005`, `ADR-0004`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial document |
