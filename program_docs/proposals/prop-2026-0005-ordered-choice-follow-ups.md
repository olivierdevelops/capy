---
document_id: PROP-2026-0005
title: Ordered-Choice Follow-Ups — Name the Nesting Bound, Expose `alts` to the Browser, Identify the Build
document_type: proposal
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
  - Release Management

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
  - architects

scope: Closes three limitations recorded in REL-0.23.0 — the nesting-depth error that surfaces as a generic message, the browser introspection JSON that omits `alts`, and a `capy version` that prints `dev` for every local build. Additive and small; no new grammar.

reason: REL-0.23.0 shipped with one PARTIAL result (R4, the depth message) and two documented limitations. The owner asked for them to be fixed on 2026-10-07.

related_documents:
  - REL-0.23.0
  - RPT-2026-0003
  - PROP-2026-0004
  - ADR-0003
  - TRBL-2026-0001
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - diagnostics
  - wasm
  - cli
  - follow-up

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Ordered-Choice Follow-Ups — Name the Nesting Bound, Expose `alts` to the Browser, Identify the Build

> **Status:** Approved (2026-10-07, Olivier — by explicit instruction)
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

```text
  FOLLOW-UP (from REL-0.23.0)                         FIX                      REQ
  ──────────────────────────────────────────────────  ───────────────────────  ────
  nesting bound surfaces as generic `expected …`      name it, code E0003      R1
  wasm capyIntrospect JSON omits `alts`               add `alts` array         R2
  `capy version` prints `capy dev` for every build    print the crate version  R3
```

## Decision Requested

Approve all three as one additive release, `0.24.0`. Approval does **not** authorize
changing the bound (64 captures), changing any existing diagnostic code's meaning, or
adding any new grammar.

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "is it done? is there more to do?" → "do it" → "fix" | User, 2026-10-07 | The assistant listed optional follow-ups; "fix" was answered "Code follow-ups as 0.24.0" |
| UQ-02 | "Code follow-ups as 0.24.0 (Recommended)" — surface the depth message, add `alts` to the wasm JSON, stamp the real version into `capy --version` | User, 2026-10-07 | Exactly the three items in `REL-0.23.0` Follow-up Work that are code changes |

## Problem and Evidence

| Problem | Evidence (verified 2026-10-07 on the v0.23.0 build) | Consequence | UQ |
|---|---|---|---|
| P-01 | 32-deep call nest: ``error: expected `)`, found "1" in `call` ``. The bound's own message exists (`make_parser.rs`, "nesting too deep (limit 64)") but `match_one` turns every sub-failure into a rewind, and inside `operand*` the bound reads as "the list ended" | A reader is told the source is malformed when it is only too deep; `E0003` is reserved and never emitted (`docs/diagnostics.md`) | UQ-02 |
| P-02 | `capy_introspect` JSON args hold `kind, value, name, type, description` and no `alts` (`rust/wasm/src/lib.rs`) | A browser tool shows only alternative 1 of `call \| name \| num` | UQ-02 |
| P-03 | `capy version` → `capy dev` for every unstamped build; a May build on PATH printed the same and rejected shipped samples (`TRBL-2026-0001`) | A stale binary is indistinguishable from a current one | UQ-02 |

```text
  P-01 today                              P-01 after
  ──────────────────────────────          ──────────────────────────────────────────
  nest 32 ─► [depth bound hit]            nest 32 ─► [depth bound hit] ─► remembered
          ─► swallowed by rewind                  ─► statement fails ─► reports the bound
          ─► `expected `)`, found "1"`            ─► `nesting too deep (limit 64) …`  E0003
```

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems | Success Signal |
|---|---|---|---|
| G-01 | A statement that fails because of the nesting bound says so | P-01 | message contains `nesting too deep`; `capy ast` code is `E0003` |
| G-02 | The browser introspection JSON carries the whole choice | P-02 | `"type":"call","alts":["name","num"]` |
| G-03 | `capy version` identifies the build | P-03 | prints `capy <crate version>` unstamped; a stamped `CAPY_VERSION` still wins |

**Non-goals**

- Raising or lowering the 64-capture bound.
- Changing `E0001` / `E0002`, or emitting `E0002` (still reserved).
- Adding `optional` / `default` to the browser JSON (a pre-existing omission, separate).
- The `default`-in-repeated-nonterminal defect (`PROP-2026-0004` OQ-06).
- `PROP-2026-0003` (expression trees over JSON).

## Proposed User Journey

```text
  author writes a 40-level nested source
        │  capy run lib script
        ▼
  error: nesting too deep (limit 64) while matching "call | name | num" — the source
         nests further than the parser will follow
        │  capy ast lib script
        ▼
  <error> …   error[E0003] 1:1: nesting too deep (limit 64) …   exit 1
```

## Requirements

| ID | Requirement | Type | Source | Acceptance Criteria | Goal |
|---|---|---|---|---|---|
| R1 | When a statement fails because the nesting bound was reached, the error names the bound and what was being matched | functional | P-01 | message contains `nesting too deep (limit 64)` and the alternatives joined with ` \| ` | G-01 |
| R2 | `Library::parse` reports that failure with code `E0003`; every other failure keeps `E0001` | API | P-01 | `diagnostics[0].code == "E0003"`; a later ordinary failure is `E0001` | G-01 |
| R3 | The bound does not leak: a bound error from an earlier statement does not colour a later one | safety | P-01 | sequence deep-then-ordinary yields `[E0003, E0001]` | G-01 |
| R4 | The bound itself does not move: 31 call levels parse, 32 are refused | compatibility | `GOAL-002` | both asserted | G-01 |
| R5 | `capy_introspect` args carry `alts` (array; empty for a plain capture); `type` keeps meaning alternative 1 | API | P-02 | JSON contains `"type":"call","alts":["name","num"]` and `"alts":[]` | G-02 |
| R6 | `capy version` and `capy --version` print `capy <CARGO_PKG_VERSION>` when `CAPY_VERSION` was not stamped; a stamped value wins | functional | P-03 | the two commands agree and never print `dev` | G-03 |
| R7 | No existing library changes behaviour | compatibility | `GOAL-002` | all pre-existing goldens byte-identical | G-01…03 |
| R8 | Documentation states the new behaviour and no longer calls `E0003` reserved or the browser JSON incomplete | docs | `CODE-003` | `docs/diagnostics.md`, `docs/errors-and-debugging.md`, `docs/roadmap.md`, `docs/language-frontend.md`, `docs/embedding.md`, `docs/library-authoring.md` updated; `mkdocs --strict` exit 0 | all |

## Use Cases

| ID | User Outcome | Actor | Surface | Inputs | Outputs | Negative Paths | Req |
|---|---|---|---|---|---|---|---|
| UC-01 | Learn the source is too deep, not malformed | language author | `capy run`, `capy ast` | a 40- or 70-deep nest of `f( … )` | `nesting too deep (limit 64) while matching "call \| name \| num"`; `ast` → `E0003`, exit 1 | exactly 31 levels → parses, exit 0 | R1, R2, R4 |
| UC-02 | An error after a deep one is still reported as itself | language author | `capy ast` | deep statement, then `return add(3, +)` | `[E0003, E0001]` | — | R3 |
| UC-03 | A browser tool shows every alternative | playground / editor | `capyIntrospect` | a library with `call \| name \| num` | `"type":"call","alts":["name","num"]` | plain capture → `"alts":[]` | R5 |
| UC-04 | Tell which build is on PATH | developer | `capy version`, `capy --version` | a local build | `capy 0.24.0` | a CI build stamps `CAPY_VERSION` → the stamp is printed | R6 |

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says 2 — `PROP-2026-0004` OQ-05, still unreconciled) | 2026-10-07 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Follow-Up |
|---|---|---|---|---|
| GOAL-001 | Applies | no grammar added | PASS | — |
| GOAL-002 | Applies | additive; R4, R7 verify | PASS | — |
| PHIL-001 | Applies | every problem reproduced on the v0.23.0 build before writing | PASS | — |
| CODE-001 | Applies | commits only on the owner's instruction (given) | PASS | — |
| CODE-003 | Applies | R8 | PASS | — |
| ARCH-001 | Applies | `ArgInfo` unchanged; wasm JSON gains a field, no field changes type | PASS | — |
| ARCH-002 | Applies | no dependency | PASS | — |
| ARCH-003 | Applies | the bound is the safeguard; this only reports it | PASS | — |
| QUAL-001 / QUAL-003 | Applies | R7; each new test is mutation-checked | PASS | — |
| QUAL-002 | **Applies** | wasm size may move: thresholds below, frozen before measurement | PASS | M-01…M-04 |
| GATE-001 / GATE-002 | Applies | full gate | PASS | — |

## Implementation Design

| Change | UC | Method | Req |
|---|---|---|---|
| C-01 | UC-01, UC-02 | `OuterP.depth_err` records the bound error when `capture_func_type` hits the limit; `parse_stmt` resets it on entry and, when a statement would otherwise fail generically, returns it and sets `fail_code = E0003`; the recovery diagnostic uses `fail_code` | R1–R4 |
| C-02 | UC-03 | `capy_introspect` writes `,"alts":[…]` after `type` | R5 |
| C-03 | UC-04 | `VERSION` falls back to `CARGO_PKG_VERSION`, as `capy_version` in the wasm ABI already does | R6 |
| C-04 | all | docs per R8; `samples/` unchanged | R8 |

```text
  match_one ──Err──► None (rewind)        depth_err (survives the rewind)
                                              │
  parse_stmt: all candidates failed ──► depth_err? ──yes──► E0003, named bound
                                              └──no───► furthest-failure (E0001)
```

## Alternatives Considered

| Alternative | Why Rejected |
|---|---|
| Propagate the bound as a hard error through `match_one` | changes the rewind contract every existing grammar relies on (R7) |
| Raise the bound | moves the cliff without removing it (`ADR-0001`) |
| Stamp the version at build time with a build script | a dependency and a build step for what `env!` already does |

## Risks and Rollback

| Risk | Mitigation | Rollback |
|---|---|---|
| A stale `depth_err` colours an unrelated failure | reset on `parse_stmt` entry; R3 test | revert C-01 |
| wasm size grows | M-02 predeclared | drop C-02 |
| A script parses `capy dev` | none known; CI stamps release builds | revert C-03 |

## Security Impact

None.

## Operational Impact

`capy version` output changes for local builds (`dev` → crate version).

## Compatibility Impact

Additive. One error *message* changes from a generic `expected` to the bound's own; no
existing golden depends on it (R7 verifies). `E0003` was reserved and never emitted.

## Test and Validation Design

| ID | Type | Req | Procedure | Expected |
|---|---|---|---|---|
| T-01 | unit | R1, R4 | 70-deep nest through `run`; 31-deep | names the bound; 31 parses |
| T-02 | unit | R2 | 40-deep nest through `parse` | one diagnostic, `E0003` |
| T-03 | unit | R3 | `return add(1, 2)`, a deep statement, `return add(3, +)` | `[E0003, E0001]` |
| T-04 | unit | R5 | `capy_introspect` on a choice library, and on a plain one | `alts` populated; `[]` |
| T-05 | integration | R6 | run the built `capy version` and `--version` | crate version, equal, not `dev` |
| T-06 | integration | R7 | `cargo test --test golden` | 131 passed, 8 skipped, 0 failed |
| T-07 | manual | R8 | run every command in changed pages; `mkdocs --strict` | exit 0 |
| T-08 | regression | GATE-001 | build, clippy `-D warnings`, `cargo test --workspace` | all exit 0 |

Every new test must fail when its fix is reverted (`QUAL-003`).

### Measurement and Validation (QUAL-002) — frozen before measurement

Baseline = the v0.23.0 tree (`8f10723` code), measured 2026-10-07, same host, release
profile. The host is noisy (single runs of the same binary varied 197–218 µs), so M-01 and
M-04 compare the **retained baseline binary** against the new one **interleaved in one
session**, best of 7.

| ID | Metric | Baseline | Test Command | Threshold |
|---|---|---|---|---|
| M-01 | transpile time, best of 7, interleaved | retained `nativebench` of the v0.23.0 tree | `nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` | ≤ **+10 %** of the baseline binary in the same session |
| M-02 | wasm size | **1 389 633 bytes** | `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`; `ls -l` | ≤ **+3 000 bytes** (expected cost: one loop and one string); hard ceiling 1 410 035 (cumulative 5 % since 0.21.0) |
| M-03 | `capy-core` direct dependencies | 1 (`regex`) | `cargo tree -p capy-core --depth 1` | exactly 1 |
| M-04 | golden suite wall time, best of 7, interleaved | retained golden test binary of the v0.23.0 tree | `cargo test --release --test golden` binaries | ≤ **1.15×** |

## Documentation, Demo and Release Impact

| Artifact | Path | CRUD | Content | Gate |
|---|---|---|---|---|
| Diagnostics | `docs/diagnostics.md` | UPDATE | `E0003` emitted; remove the "reserved" caveats | T-07 |
| Errors guide | `docs/errors-and-debugging.md` | UPDATE | the bound message | T-07 |
| Roadmap | `docs/roadmap.md` | UPDATE | "Name the nesting limit" → shipped | T-07 |
| Front-end guide | `docs/language-frontend.md` | UPDATE | depth row now names the error | T-07 |
| Embedding | `docs/embedding.md` | UPDATE | browser introspection JSON `alts` | T-07 |
| Authoring guide | `docs/library-authoring.md` | UPDATE | "When it fails" depth paragraph | T-07 |
| Release | `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | 0.24.0 entry | GATE-001 |
| Version | `rust/Cargo.toml` + 4 pins + `Cargo.lock` | UPDATE | `0.24.0` | Release Management |
| Program docs | `PLAN`, `TEST`, `RPT`, `DEMO`, `MAN-2026-0002`, `SYS-2026-0001`, `TRBL-2026-0001`, `REL-0.24.0`, indexes | CREATE / UPDATE | §22–§34 | — |

## Requirements Alignment

| Req | UQ | Goal | UC | Changes | Tests |
|---|---|---|---|---|---|
| R1 | UQ-02 | G-01 | UC-01 | C-01 | T-01 |
| R2 | UQ-02 | G-01 | UC-01 | C-01 | T-02 |
| R3 | UQ-02 | G-01 | UC-02 | C-01 | T-03 |
| R4 | UQ-02 | G-01 | UC-01 | C-01 | T-01 |
| R5 | UQ-02 | G-02 | UC-03 | C-02 | T-04 |
| R6 | UQ-02 | G-03 | UC-04 | C-03 | T-05 |
| R7 | UQ-02 | all | — | C-01…03 | T-06 |
| R8 | UQ-02 | all | — | C-04 | T-07 |

## Plan Strategy and Estimated Work

One plan (`PLAN-2026-0004`), one release (`0.24.0`).

## Open Questions

| ID | Question | Proposed Default |
|---|---|---|
| OQ-01 | Should `optional` and `default` also join the browser JSON? | No — separate, pre-existing; revisit with a consumer |
| OQ-02 | Should the bound's message be reachable from a repetition that ends *at* the bound without failing the statement? | No — if the statement succeeds there is nothing to report |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | **approved** | 2026-10-07 | Olivier, by explicit instruction ("Code follow-ups as 0.24.0"). Agent validation did not approve this proposal (§21.2) |

## Related Documents

- `REL-0.23.0` Known Limitations 1, 4, 7 and Follow-up Work
- `RPT-2026-0003` R4 `PARTIAL`
- `TRBL-2026-0001`
- `PROP-2026-0004`, `ADR-0003`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial proposal, approved the same day |
