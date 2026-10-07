---
document_id: PLAN-2026-0003
title: Implementation Plan — Ordered Alternation for Nonterminals (0.23.0)
document_type: plan
status: completed

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

scope: Implements PROP-2026-0004 revision 4 (ordered alternation in a capture type) as the 0.23.0 release, and carries the already-implemented, previously uncommitted PROP-2026-0002 docs-and-samples work through validation and into the same release.

reason: A capture type names exactly one function, so a recursive expression grammar cannot nest past one level and a parameter list cannot mix plain and marked parameters. Reproduced on the repo build on 2026-10-07.

related_documents:
  - PROP-2026-0004
  - PROP-2026-0002
  - ADR-0003
  - ADR-0001
  - PLAN-2026-0002
  - REL-0.22.0
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - parser
  - grammar
  - alternation

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Implementation Plan — Ordered Alternation for Nonterminals (0.23.0)

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs, samples

## Summary

```text
  P0 contracts ─► P1 data model + guard ─► P2 matcher ─► P3 sample + tests
                                                              │
        P7 release ◄─ P6 manual/system ◄─ P5 validate ◄─ P4 measure
```

One plan, one release. Alternation is a minor, additive engine change (`0.23.0`);
the carried docs/samples work (`PROP-2026-0002`) has no engine behaviour of its
own beyond the shared `ast_text` renderer already present in the tree.

## Objective, Scope and Proposal Baseline

**Baseline:** `PROP-2026-0004` revision 4; decision `ADR-0003`.
**Predecessor:** `PLAN-2026-0002` (completed, released as 0.22.0).

| Category | IDs owned |
|---|---|
| Goals | G-01, G-02, G-03, G-04 |
| Requirements | R1 … R11 |
| Use cases | UC-01 … UC-06 |
| Changes | C-01 … C-07 |
| Carried (`PROP-2026-0002`) | R1 … R19 of that proposal, validated in `RPT-2026-0003` |

**Not owned:** named unions (OQ-04), the `default`-in-repeated-nonterminal defect
(OQ-06), a leading-optional combinator (OQ-07), optional captures before a block
opener and `} else {` continuation (roadmap), `PROP-2026-0003` (still a draft,
not part of this release).

### Deviation from the proposal's increment order

`PROP-2026-0004` asked to land behind the `PROP-2026-0003` INC-1 correctness fix.
`PROP-2026-0003` is a draft with no approved plan, so it cannot gate this release.
Recorded here, repeated in `RPT-2026-0003` and `REL-0.23.0`.

## Live Status Summary

| State | Count | Notes |
|---|---:|---|
| NOT STARTED | 5 | release commit, SHA, tag, push, `REL-0.23.0` — P7, finalised in the release-finalize commit |
| IN PROGRESS | 0 | |
| BLOCKED | 0 | |
| DONE | 24 | phases P0–P6 and the version bump |
| FAILED | 0 | |
| DEFERRED | 0 | |

- **Current phase:** P7 — release. P0–P6 exited; `RPT-2026-0003` validated with one documented `PARTIAL` (R4).
- **Next action:** release commit, tag, push, `REL-0.23.0`.
- **Blockers:** none
- **Release target:** 0.23.0

## Requirements and Use Cases

| Req / UC | Why it exists | Planned outcome | Acceptance criterion | Phase | Status |
|---|---|---|---|---|---|
| R1 / UC-01 | Choice is the missing combinator | `arg capture v call \| atom` loads | Library loads; unknown name in any position → load error naming it | P1 | **DONE** |
| R2 / UC-01 | Determinism | Left to right, first match wins | Reordering alternatives changes the match, **in a test that fails if order is ignored** | P2 | **DONE** |
| R3 / UC-01 | Argument lists | `(call \| atom)* sep ","` | Repeated alternation parses with `sep` / `join` | P2 | **DONE** |
| R4 / UC-02 | Nesting | Arbitrary depth to the existing bound | 5-deep nest parses; 70-deep hits the existing depth error, no panic | P2 | **DONE** |
| R5 / UC-04 | `ARCH-003` safety | Guard walks every alternative edge | `expr = expr \| atom` refused at load with the cycle trace | P1 | **DONE** |
| R6 / UC-01 | Diagnostic style | Union expectation | ``expected `call`, `num` or `name`, found …`` | P2 | **DONE** |
| R7 | `GOAL-002` | No existing library changes | Golden corpus byte-identical | P3, P4 | **DONE** |
| R8 / UC-03, UC-06 | API stability | `sub[].func` names the matched alternative; `schema_version` stays 1 | JSON test asserts discriminator | P3 | **DONE** |
| R9 | Truthful introspection | Union printed | `capy docs` Type column prints `a \| b`; `ArgInfo.alts` populated | P2 | **DONE** |
| R10 / UC-05 | `bare` was hidden (P-04) | Worked `bare` example + sample | Section in `library-authoring.md`; `samples/expression-grammar/` with goldens | P3, P6 | **DONE** |
| R11 | `CODE-003` | `\|` documented | `docs/library-keywords.md` row + ordered-choice section | P6 | **DONE** |
| UC-06 | Mixed parameter markers | `mut_param \| plain_param` parses | T-10 passes; same input rejected on 0.22.0 | P3 | **DONE** |

## Applicable Project Standards

Standards index validated at the revision recorded in `PROP-2026-0004` (body
counter 1, front matter 2 — OQ-05 there). Rules: `GOAL-001`, `GOAL-002`,
`PHIL-001`, `PHIL-002`, `CODE-001` (commit only when asked — the owner asked for a
release commit on 2026-10-07), `CODE-003` (keyword list in sync), `ARCH-001`
(resolved by `ADR-0003`), `ARCH-002`, `ARCH-003`, `QUAL-001`, `QUAL-002`,
`QUAL-003`, `GATE-001`, `GATE-002`.

## Measurable Claims

Predeclared **before implementation**. Baselines were measured on 2026-10-07 on
the working tree immediately before any `PROP-2026-0004` change.

| ID | Metric | Baseline (method) | Environment | Command | Threshold | Report |
|---|---|---|---|---|---|---|
| M-01 | Transpile time, best of 5 | **190.903 µs** (`nativebench`, `samples/transpile-py`, release) | macOS arm64, Darwin 25.4.0, Rust stable, release profile | `./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` ×5 | ≤ **+10 %** (≤ 210.0 µs) | `TEST-2026-0010` |
| M-02 | wasm size | **1 386 116 bytes** now; **1 342 891** at 0.21.0 | same | `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`, `ls -l` | the 5 % allowance is **cumulative since 0.21.0**: ≤ **1 410 035 bytes** total (≤ +23 919 bytes this release) | `TEST-2026-0010` |
| M-03 | `capy-core` direct dependencies | 1 (`regex`) | same | `cargo tree -p capy-core --depth 1` | exactly 1 | `TEST-2026-0010` |
| M-04 | Golden suite wall time | measured in P3 before the matcher change is merged | same | `cargo test --test golden` | ≤ **1.15×** baseline | `TEST-2026-0010` |

## Prerequisites

`ADR-0003` approved; baseline tree builds and passes (`cargo test --workspace`,
2026-10-07: all green).

## Phases, Entry Gates and Exit Gates

| Phase | Content | Entry | Exit |
|---|---|---|---|
| P0 | Contracts | proposal approved | `ADR-0003` approved; OQ-02, OQ-08 settled |
| P1 | `alts` data model; loader parse; resolve; **guard first** | P0 | T-04 and T-06 pass; library with `\|` loads, cyclic one refused |
| P2 | Matcher: ordered choice, repetition, union diagnostic, introspection | P1 | T-01, T-03, T-05 pass |
| P3 | `samples/expression-grammar/`, tests, goldens | P2 | T-02, T-07, T-10, T-11 pass; corpus unchanged |
| P4 | Measurements | P3 | M-01 … M-04 recorded |
| P5 | Validation report | P4 | `RPT-2026-0003`: result per requirement |
| P6 | Demo, manual, README / system / architecture assessment | P5 | `DEMO-2026-0003`, manual chapter, §31 decisions recorded |
| P7 | Version, commit, tag, push, release document | P6 | §34 gate passes |

## Implementation Approach

1. **Contract (P0, R1, OQ-02, OQ-08).** `cap_type` stays and holds alternative 1;
   `alts: Vec<String>` holds 2…n. Alternatives are library functions only.
2. **Parse the type position (P1, R1).** In `capy_lib_parser.rs`, the `arg capture
   NAME TYPE` tokens are split on whitespace today, so `a | b*` arrives as three
   tokens. Accept `TYPE ( "|" TYPE )*` with the repetition suffix on the **last**
   name, applying to the whole alternation. `a|b` without spaces is accepted too
   if the tokenizer yields it as one token (T-11 decides).
3. **Validate and resolve (P1, R1, R5).** In `make_library_loader.rs`: every
   alternative must be a library function, else an error naming it; set
   `is_func` when `cap_type` is a function; copy `alts` in `compile_elements`.
4. **Guard before matcher (P1, R5).** `reject_left_recursion` adds an edge to
   **every** alternative, not only `cap_type`. Done before P2 so a cyclic library
   can never reach a matcher that cannot yet terminate on it.
5. **Matcher (P2, R2, R3, R4, R6).** `capture_func_type_inner` resolves the target
   list `[cap_type, alts…]`; `match_one` becomes "try each in order, first success
   wins", rewinding locally on failure. A failure notes each alternative's
   `Expectation::Nonterminal`, so the existing merge yields the union (R6). The
   depth bound is unchanged.
6. **Introspection (P2, R9).** `ArgInfo.alts`, and `capy docs` renders the union.
7. **Sample and tests (P3).** A recursive expression grammar; goldens including
   `.expected-ast.txt`; unit tests T-01 … T-11.

### Risks

| Risk | Mitigation |
|---|---|
| `RK-01` public field type changes | additive `alts`; `type_` unchanged |
| `RK-03` guard misses a cycle through one alternative | guard lands first in P1 with its own test |
| `RK-02` pathological backtracking | rewind is capture-local; M-04 bounds the suite time |
| First-match-wins surprises authors | R11 docs; `capy docs` prints order |
| wasm allowance is shared | M-02 ceiling is cumulative, stated above |

## Live Work Checklist

| ID | Task | Phase | Req / UC / Change | Files | Depends | Status | Evidence |
|---|---|---|---|---|---|---|---|
| W-01 | `alts` on `RawArg`, `ArgEntry`, `PatternElement`, `ArgInfo` | P1 | R1, R9 / C-01 | `rust/src/infra/raw_library.rs`, `rust/src/domain/library.rs`, `rust/src/capy.rs` | — | DONE | `cargo build` ok; `ArgInfo.alts` asserted by `alternation_loads_and_is_introspectable` |
| W-02 | Parse `A \| B \| C` in the type position | P1 | R1 / C-01 | `rust/src/infra/capy_lib_parser.rs` | W-01 | DONE | `glued_pipe_is_the_same_choice`, `malformed_alternation_is_rejected` |
| W-03 | Validate each alternative; resolve `is_func`; copy `alts` | P1 | R1 / C-01 | `rust/src/orchestrator/features/make_library_loader.rs` | W-01 | DONE | `unknown_alternative_is_a_load_error_naming_it`, `builtin_type_is_not_an_alternative` |
| W-04 | Guard walks every alternative edge | P1 | R5 / C-04 | `rust/src/orchestrator/features/make_library_loader.rs` | W-03 | DONE | `left_recursion_through_second_alternative_is_rejected` — **fails when the guard is reverted** (mutation check) |
| W-05 | Ordered choice in the matcher | P2 | R2, R4, R6 / C-02 | `rust/src/orchestrator/features/make_parser.rs` | W-04 | DONE | `first_matching_alternative_wins` — **fails when order is reversed** (mutation check) |
| W-06 | Alternation inside `*` / `+` with `sep` / `join` | P2 | R3 / C-03 | `rust/src/orchestrator/features/make_parser.rs` | W-05 | DONE | `five_deep_nest_parses`, `mixed_parameter_markers_parse` |
| W-07 | `capy docs` prints the union | P2 | R9 / C-01 | `rust/src/domain/docs.rs` | W-01 | DONE | `docs_print_the_union` |
| W-08 | Discriminator verified; no serializer change | P3 | R8 / C-05 | `rust/tests/alternation.rs` | W-05 | DONE | `discriminator_names_the_matched_alternative` |
| W-09 | Sample `samples/expression-grammar/` | P3 | R10 / C-07 | `samples/expression-grammar/*` | W-06 | DONE | `samples/expression-grammar/` — 4 goldens, `goldens: 131 passed, 8 skipped, 0 failed` |
| W-10 | Mixed-parameter sample (UC-06) | P3 | UC-06 | `samples/mixed-parameters/*` | W-06 | DONE | `samples/mixed-parameters/` — 2 goldens |
| W-11 | Docs: keywords, authoring guide, front-end guide, cheat sheet, features | P6 | R10, R11 / C-06 | `docs/library-keywords.md`, `docs/library-authoring.md`, `docs/language-frontend.md`, `docs/syntax-cheat-sheet.md`, `docs/features.md` | W-09 | DONE | `mkdocs build --strict` rc=0; transcripts re-run against `rust/target/debug/capy` |

## File and Artifact Checklist

| Path | CRUD | Why | Req |
|---|---|---|---|
| `rust/src/infra/raw_library.rs` | UPDATE | `RawArg.alts` | R1 |
| `rust/src/infra/capy_lib_parser.rs` | UPDATE | parse `A \| B` | R1 |
| `rust/src/domain/library.rs` | UPDATE | `ArgEntry.alts`, `PatternElement.alts` | R1 |
| `rust/src/orchestrator/features/make_library_loader.rs` | UPDATE | validate, resolve, guard | R1, R5 |
| `rust/src/orchestrator/features/make_parser.rs` | UPDATE | ordered choice | R2, R3, R4, R6 |
| `rust/src/capy.rs` | UPDATE | `ArgInfo.alts` | R9 |
| `rust/src/domain/docs.rs` | UPDATE | union in Type column | R9 |
| `rust/tests/alternation.rs` | CREATE | T-01 … T-05, T-07, T-10, T-11 | R1–R9 |
| `rust/tests/golden.rs` | READ | corpus runner; carried change already in tree | R7 |
| `samples/expression-grammar/` | CREATE | recursive grammar sample + goldens | R10 |
| `samples/mixed-parameters/` | CREATE | UC-06 sample + goldens | UC-06 |
| `samples/README.md` | UPDATE | rows + re-derived counts | R10 |
| `docs/library-keywords.md` | UPDATE | `\|` row (`CODE-003`) | R11 |
| `docs/library-authoring.md` | UPDATE | ordered-choice section; worked `bare` example | R10, R11 |
| `docs/language-frontend.md` | UPDATE | remove one-level-nesting caveat | R10 |
| `docs/syntax-cheat-sheet.md`, `docs/features.md` | UPDATE | one row each | R11 |
| `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | Added entry | release |
| `rust/Cargo.toml` | UPDATE | version `0.23.0` (workspace source) | release |
| `rust/Cargo.lock` | UPDATE | workspace version sync | release |
| `program_docs/**` | CREATE / UPDATE | TEST, RPT, DEMO, MAN, SYS, REL, indexes | §22–§34 |

## Test and Validation Checklist

| ID | Type | Validates | Procedure | Expected | Status |
|---|---|---|---|---|---|
| T-01 | unit | R1, R2 | reorder alternatives over input matching both; assert the match changes | **fails if order is ignored** | DONE |
| T-02 | integration | R1, R3, R4 | `samples/expression-grammar/` golden | `add(3, mul(4, 5))` and a 5-deep nest parse | DONE |
| T-03 | unit | R4 | 70-deep nest | existing depth diagnostic, no panic | DONE |
| T-04 | unit | R5 | `expr = expr \| atom`, and a cycle reachable only through the second alternative | refused at load with the cycle trace | DONE |
| T-05 | unit | R6 | input matching no alternative | union expectation | DONE |
| T-06 | integration | R7 | `cargo test --test golden` | all pre-existing goldens byte-identical | DONE |
| T-07 | unit | R8 | `capy ast --json` on UC-01 | `sub[0].func` names the alternative; `schema_version` 1 | DONE |
| T-08 | manual | R9, R10, R11 | run every command quoted in changed pages | byte-identical | DONE |
| T-09 | regression | GATE-001 | build, `clippy -D warnings`, `cargo test --workspace`, `mkdocs build --strict` | all exit 0 | DONE |
| T-10 | integration | UC-06 | `samples/mixed-parameters/` | `def f(mut c: Counter, n: int)` parses; rejected on 0.22.0 | DONE |
| T-11 | unit | OQ-01 | `\|` and `\|>` in a library file and in source | `\|` is the choice token in a capture type; source `\|>` unaffected | DONE |
| T-12 | unit | R9 | `Library::introspect` and `capy docs` on an alternation capture | `alts` populated; union printed | DONE |

## Documentation and Demo Checklist

`DEMO-2026-0003` (§29), manual chapter (§30), `SYS-2026-0001` update (§31),
`docs/*` per the file checklist, `CHANGELOG.md`, `docs/whats-new.md`.

## Section 31 Documentation-Impact Decision

| Artifact | Decision (to be confirmed in `REL-0.23.0`) | Reason |
|---|---|---|
| README.md | to assess | capability list may name combinators |
| System documentation | UPDATED | `SYS-2026-0001` matcher section gains ordered choice |
| Architecture documentation | to assess | no component boundary moves |
| API and CLI reference | UPDATED | `capy docs` output, `ArgInfo.alts` |
| Manual | UPDATED | new chapter |
| Troubleshooting | to assess | left-recursion message now covers alternatives |

## Version, Release and Rollout Checklist

| Step | Detail | Status |
|---|---|---|
| Version source | `rust/Cargo.toml` `[workspace.package] version` → `0.23.0`, the four crate path-dependency pins and `Cargo.lock` synchronised (same 7 files as `chore(release): 0.22.0`) | DONE |
| Release commit | `release: v0.23.0` after every earlier stage passes | NOT STARTED |
| Record SHA | `git rev-parse HEAD` (full) | NOT STARTED |
| Tag | `git tag -a v0.23.0`; `git rev-list -n 1 v0.23.0` equals the SHA | NOT STARTED |
| Push | `git push origin main` and `git push origin v0.23.0` — authorized by the owner on 2026-10-07 | NOT STARTED |
| Release document | `REL-0.23.0` created **after** the tag | NOT STARTED |

## Decisions, Findings, Deviations and Blockers

| Date | Kind | Entry |
|---|---|---|
| 2026-10-07 | Decision | OQ-02, OQ-08 settled in `ADR-0003` |
| 2026-10-07 | Deviation | not gated behind `PROP-2026-0003` INC-1 (draft, no plan) |
| 2026-10-07 | Decision | carried `PROP-2026-0002` work released with this version by owner instruction |

## Rollout Strategy

Single release, local build and CLI. No migration: `|` in a capture type was a
load error, so no existing library can contain it.

## Rollback Strategy

Revert the release commit. Phase P1's guard change is independently revertible
(`RK-03`).

## Completion Criteria and Final Traceability

`DOCUMENTATION.md` §34 passes in full. `Release → Validation → Tests →
Implementation → Plan` resolves for every `R` and `UC`.

## Open Questions

None blocking. OQ-04, OQ-06, OQ-07 deferred (`ADR-0003`).

## Related Documents

- `PROP-2026-0004`, `ADR-0003`, `PROP-2026-0002`, `PLAN-2026-0002`, `REL-0.22.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial approved plan |
