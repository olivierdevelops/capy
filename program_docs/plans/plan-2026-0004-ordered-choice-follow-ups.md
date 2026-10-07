---
document_id: PLAN-2026-0004
title: Implementation Plan — Ordered-Choice Follow-Ups (0.24.0)
document_type: plan
status: completed

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 2

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
  - capy-cli
  - capy-wasm-abi
  - docs

affected_versions:
  from: "0.24.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Implements PROP-2026-0005 revision 1 as release 0.24.0.

reason: REL-0.23.0 left one PARTIAL result and two limitations; the owner asked for them to be fixed.

related_documents:
  - PROP-2026-0005
  - ADR-0004
  - REL-0.23.0
  - PLAN-2026-0003
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - diagnostics
  - wasm
  - cli

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Implementation Plan — Ordered-Choice Follow-Ups (0.24.0)

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

```text
  P0 contracts ─► P1 depth bound (C-01) ─► P2 wasm alts + version (C-02, C-03)
                                                   │
        P6 release ◄─ P5 manual/system ◄─ P4 validate ◄─ P3 measure + docs
```

## Objective, Scope and Proposal Baseline

**Baseline:** `PROP-2026-0005` revision 1; decision `ADR-0004`. **Predecessor:** `PLAN-2026-0003`.

| Category | IDs owned |
|---|---|
| Goals | G-01, G-02, G-03 |
| Requirements | R1 … R8 |
| Use cases | UC-01 … UC-04 |
| Changes | C-01 … C-04 |

**Not owned:** the bound's value, `optional` / `default` in the browser JSON, OQ-06, `PROP-2026-0003`.

### Deviation: implementation drafted before this plan

The three fixes were first written, tested and mutation-checked **before** `PROP-2026-0005`,
`ADR-0004` and this plan existed — the wrong order under §21.3 and §24. It was caught before
anything was committed or measured. The code was saved as a patch and removed from the tree; the
baselines were measured on the clean tree; this plan and its thresholds were written and
committed; then the patch is re-applied. Thresholds were therefore frozen before any "after"
measurement. The deviation is repeated in `RPT-2026-0004` and `REL-0.24.0`.

## Live Status Summary

| State | Count | Notes |
|---|---:|---|
| NOT STARTED | 0 | |
| IN PROGRESS | 0 | |
| BLOCKED | 0 | |
| DONE | 13 | `W-01`…`W-08` and all release rows; tests T-01…T-08, requirements R1…R8 and measurements M-01…M-04 all DONE / PASS |
| FAILED | 0 | |
| DEFERRED | 0 | |

- **Current phase:** **COMPLETE.** P0–P6 exited; `RPT-2026-0004` validated with no PARTIAL; 0.24.0 released.
- **Next action:** none.
- **Blockers:** none
- **Release target:** 0.24.0 — **released** (`v0.24.0`)

## Requirements and Use Cases

| Req / UC | Planned Outcome | Acceptance Criterion | Phase | Status |
|---|---|---|---|---|
| R1 / UC-01 | bound named | message has `nesting too deep (limit 64)` + the alternatives | P1 | **DONE** |
| R2 / UC-01 | `E0003` | `parse` diagnostic code `E0003`, others `E0001` | P1 | **DONE** |
| R3 / UC-02 | no leak | `[E0003, E0001]` | P1 | **DONE** |
| R4 / UC-01 | bound unchanged | 31 parse, 32 refused | P1 | **DONE** |
| R5 / UC-03 | `alts` in browser JSON | `"type":"call","alts":["name","num"]`, `[]` for plain | P2 | **DONE** |
| R6 / UC-04 | version identifies build | `capy <crate version>`, not `dev` | P2 | **DONE** |
| R7 | nothing else changes | goldens byte-identical | P3 | **DONE** |
| R8 | docs true | `mkdocs --strict` exit 0 | P3 | **DONE** |

## Applicable Project Standards

`GOAL-001`, `GOAL-002`, `PHIL-001`, `CODE-001` (the owner asked for a release), `CODE-003`,
`ARCH-001`, `ARCH-002`, `ARCH-003`, `QUAL-001`, `QUAL-002`, `QUAL-003`, `GATE-001`, `GATE-002`.

## Measurable Claims

Frozen **before** any "after" measurement. Baseline = the v0.23.0 tree, measured on the clean
tree 2026-10-07 (macOS arm64, Darwin 25.4.0, Rust stable, release profile). The host is noisy, so
M-01 and M-04 compare retained baseline binaries against new ones **interleaved in one session**.

| ID | Metric | Baseline | Command | Threshold | Report |
|---|---|---|---|---|---|
| M-01 | transpile time, best of 7, interleaved | retained `nativebench` of the v0.23.0 tree (single runs 196.9–218.1 µs on this host) | `nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` | ≤ +10 % | `TEST-2026-0013` |
| M-02 | wasm size | **1 389 633 bytes** | `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`; `ls -l` | ≤ +3 000 bytes; ceiling 1 410 035 | `TEST-2026-0013` |
| M-03 | direct dependencies | 1 | `cargo tree -p capy-core --depth 1` | exactly 1 | `TEST-2026-0013` |
| M-04 | golden suite wall time, best of 7, interleaved | retained golden test binary of the v0.23.0 tree | `cargo test --release --test golden` binaries | ≤ 1.15× | `TEST-2026-0013` |

## Phases, Entry Gates and Exit Gates

| Phase | Content | Exit |
|---|---|---|
| P0 | contracts | `ADR-0004`; this plan committed **before** code |
| P1 | C-01 depth bound | T-01…T-03 pass; each fails when C-01 is reverted |
| P2 | C-02, C-03 | T-04, T-05 pass; each fails when reverted |
| P3 | measure + docs (C-04) | M-01…M-04 recorded; T-06, T-07 pass |
| P4 | validation | `RPT-2026-0004` |
| P5 | demo, manual, system, impact decisions | `DEMO-2026-0004`, `MAN-2026-0002` update, `SYS-2026-0001` update |
| P6 | version, commits, tag, push, `REL-0.24.0` | §34 gate |

## Implementation Approach

1. **C-01 (R1–R4).** `OuterP` gains `depth_err: Option<CapyError>` and `fail_code`. `capture_func_type`
   stores the bound error when the limit is hit. `parse_stmt` clears both on entry; when every
   candidate has failed and a bound error is held, it returns that error with `E0003`; the recovery
   branch builds its `Diagnostic` from `fail_code`.
2. **C-02 (R5).** `capy_introspect` writes `,"alts":[…]` after `type`.
3. **C-03 (R6).** `VERSION` falls back to `env!("CARGO_PKG_VERSION")`.
4. **C-04 (R8).** Docs per the proposal.

| Risk | Mitigation |
|---|---|
| stale `depth_err` | reset on entry; T-03 |
| a test that cannot fail | mutation check each of C-01, C-02, C-03 |
| stale `capy` binary during verification | build with `cargo build --workspace` (`TRBL-2026-0001`) |

## Live Work Checklist

| ID | Task | Phase | Req / Change | Files | Status | Evidence |
|---|---|---|---|---|---|---|
| W-01 | `depth_err`, `fail_code`; report `E0003` | P1 | R1–R4 / C-01 | `rust/src/orchestrator/features/make_parser.rs` | DONE | `depth_err` / `fail_code` in `make_parser.rs`; mutation (a) turns 3 tests red (TEST-2026-0012 T-01…T-03) |
| W-02 | depth tests | P1 | R1–R4 | `rust/tests/alternation.rs` | DONE | `seventy_deep_nest_hits_the_existing_bound`, `depth_bound_is_reported_as_e0003`, `depth_error_does_not_leak_into_the_next_statement`, `thirty_one_levels_still_parse` — 21 pass |
| W-03 | `alts` in `capy_introspect` | P2 | R5 / C-02 | `rust/wasm/src/lib.rs` | DONE | `capy_introspect` emits `"alts"`; mutation (c) turns `introspect_json_carries_alts` red |
| W-04 | wasm tests | P2 | R5 | `rust/wasm/src/lib.rs` (`#[cfg(test)]`) | DONE | 2 wasm unit tests pass |
| W-05 | version fallback | P2 | R6 / C-03 | `rust/cli/src/main.rs` | DONE | `capy version` prints `capy 0.24.0`; mutation (b) turns the version test red |
| W-06 | version tests | P2 | R6 | `rust/cli/tests/version.rs` | DONE | `rust/cli/tests/version.rs` — 2 pass |
| W-07 | docs | P3 | R8 / C-04 | see File Checklist | DONE | `mkdocs build --strict` rc=0; messages re-run against `rust/target/debug/capy` |
| W-08 | version bump, commits, tag, push, `REL-0.24.0` | P6 | — | see Version checklist | DONE | `v0.24.0` → `54f1a5ead499e5ad4d5317ba19cfc4abe78bee63`; `REL-0.24.0` |

## File and Artifact Checklist

| Path | CRUD | Why | Req |
|---|---|---|---|
| `rust/src/orchestrator/features/make_parser.rs` | UPDATE | C-01 | R1–R4 |
| `rust/tests/alternation.rs` | UPDATE | depth tests replace the PARTIAL one | R1–R4 |
| `rust/wasm/src/lib.rs` | UPDATE | C-02 and its unit tests | R5 |
| `rust/cli/src/main.rs` | UPDATE | C-03 | R6 |
| `rust/cli/tests/version.rs` | CREATE | C-03 test | R6 |
| `docs/diagnostics.md`, `docs/errors-and-debugging.md`, `docs/roadmap.md`, `docs/language-frontend.md`, `docs/embedding.md`, `docs/library-authoring.md`, `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | R8 | R8 |
| `rust/Cargo.toml`, `rust/Cargo.lock`, `rust/{cli,devtools,mcp,playground,wasm}/Cargo.toml` | UPDATE | version `0.24.0` | release |
| `program_docs/**` | CREATE / UPDATE | §22–§34 | — |

## Test and Validation Checklist

| ID | Validates | Procedure | Expected | Status |
|---|---|---|---|---|
| T-01 | R1, R4 | 70-deep through `run`; 31-deep | names the bound; 31 parses | DONE |
| T-02 | R2 | 40-deep through `parse` | one diagnostic, `E0003` | DONE |
| T-03 | R3 | ordinary, deep, ordinary-failure | `[E0003, E0001]` | DONE |
| T-04 | R5 | `capy_introspect` choice and plain libraries | `alts` populated / `[]` | DONE |
| T-05 | R6 | `capy version`, `--version` | crate version, equal, not `dev` | DONE |
| T-06 | R7 | `cargo test --test golden` | 131 / 8 / 0 | DONE |
| T-07 | R8 | run changed pages' commands; `mkdocs --strict` | exit 0 | DONE |
| T-08 | GATE-001 | build, clippy, `cargo test --workspace` | exit 0 | DONE |

## Section 31 Documentation-Impact Decision

| Artifact | Decision | Reason |
|---|---|---|
| Release verification guide / demo | UPDATED | `DEMO-2026-0004` |
| README.md | NOT APPLICABLE | no capability or combinator list; see `REL-0.24.0` |
| System documentation | UPDATED | `SYS-2026-0001` (matcher, wasm JSON, version) |
| Architecture documentation | NOT APPLICABLE | no component boundary moves; `ARCH-2026-0001` node shape unchanged |
| API and CLI reference | UPDATED | `docs/embedding.md`, `docs/diagnostics.md` |
| Manual | UPDATED | `MAN-2026-0002` limitations, `MAN-2026-0001` unaffected |
| Troubleshooting | UPDATED | `TRBL-2026-0001` status |

## Version, Release and Rollout Checklist

| Step | Detail | Status |
|---|---|---|
| Version source | 7 files, as `chore(release): 0.23.0` — done in `cb23972` | DONE |
| Commits | `a046032` plan, `8769f51` fix, `cb23972` chore(release), `54f1a5e` docs(program) (tagged), then the finalize commit | DONE |
| Tag | `v0.24.0`; `git rev-list -n 1 v0.24.0` = `54f1a5ead499e5ad4d5317ba19cfc4abe78bee63`; `git describe --tags --exact-match HEAD` = `v0.24.0` at tagging | DONE |
| Push | `main` and `v0.24.0` to `origin` — authorized by the owner on 2026-10-07 for this release's flow | DONE — pushed together with the finalize commit |
| Release document | `REL-0.24.0`, after the tag | DONE |

## Decisions, Findings, Deviations and Blockers

| Date | Kind | Entry |
|---|---|---|
| 2026-10-07 | Deviation | implementation drafted before plan; set aside and re-applied after this plan (see above) |

## Rollback Strategy

Revert the release commit; C-01, C-02 and C-03 are independent.

## Completion Criteria and Final Traceability

`DOCUMENTATION.md` §34 passes. `Release → Validation → Tests → Implementation → Plan` resolves for every `R` and `UC`.

## Open Questions

None blocking (`PROP-2026-0005` OQ-01, OQ-02 defaulted).

## Related Documents

- `PROP-2026-0005`, `ADR-0004`, `REL-0.23.0`, `PLAN-2026-0003`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial approved plan, committed before the code was re-applied |
| 2 | 2026-10-07 | Olivier | Closed: all phases DONE; release recorded in `REL-0.24.0` |
