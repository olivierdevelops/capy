---
document_id: PLAN-2026-0005
title: Implementation Plan — A C ABI for Capy (0.26.0)
document_type: plan
status: completed

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 2

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
  from: "0.25.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Implements PROP-2026-0008 revision 1 as release 0.26.0.

reason: The owner asked for the C ABI proposal to be planned, implemented, validated and released.

related_documents:
  - PROP-2026-0008
  - ADR-0005
  - PROP-2026-0007
  - PLAN-2026-0004
  - REL-0.24.0
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - ffi
  - c-abi
  - release

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Implementation Plan — A C ABI for Capy (0.26.0)

> **Status:** Completed
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Summary

```text
  P0 contracts ─► P1 crate+handles+tree ─► P2 expressions ─► P3 JSON expr + CLI flag
                                                                    │
  P6 release ◄─ P5 docs+measure ◄─ P4 header, C tests, sanitizers, panic, threads
        ▲
        └─ P4b validation (RPT-2026-0005) ─► demo ─► manual ─► version ─► commit ─► tag ─► REL-0.26.0
```

## Objective, Scope and Proposal Baseline

**Baseline:** `PROP-2026-0008` revision 1; decision `ADR-0005`. **Predecessor:** `PLAN-2026-0004`.
Baseline commit: `69fb4ac98033af4bb2a8ea8afdc4c5977898a29e`.

| Category | IDs owned |
|---|---|
| Goals | G-01 … G-05 |
| Requirements | R1 … R17 |
| Use cases | UC-01 … UC-07 |
| Changes | C-01 … C-10 |
| Files | F-01 … F-12 |

**Not owned:** byte offsets (D5), exposing `run`/`set_host`/`introspect` (D6), spans on `Expr`, incremental parse.

## Live Status Summary

| State | Count | Notes |
|---|---:|---|
| NOT STARTED | 0 | |
| IN PROGRESS | 0 | |
| BLOCKED | 0 | |
| DONE | 17 | W-01 … W-17 |
| FAILED | 0 | |
| DEFERRED | 0 | |

- **Current phase:** **COMPLETE.** P0–P6 exited; `RPT-2026-0005` validated: R1–R17 16 PASS, 1 PARTIAL (R15); no FAIL.
- **Next action:** push, if the owner wants it (not authorized by this plan).
- **Blockers:** none. Open: `INC-2026-0003` (pre-existing).
- **Release target:** 0.26.0 — released locally (`v0.26.0`); see the Version table.

## Requirements and Use Cases

| Req / UC | Planned Outcome | Acceptance Criterion | Phase | Status |
|---|---|---|---|---|
| R1 / UC-01 | `capy-ffi` crate, one dependency | `cargo tree -p capy-core --depth 1` = 1; `-p capy-ffi` = `capy-core` only | P1 | **PASS** |
| R2 / UC-01 | library handle, shared read-only | two threads parse one library, TSan/run clean | P1, P4 | **PASS** |
| R3 / UC-02 | parse never fails to return a handle | hostile inputs return handle + diagnostics | P1 | **PASS** |
| R4 / UC-03 | full statement/capture/error-node access | C-view JSON = Rust JSON | P1 | **PASS** |
| R5 / UC-04 | all 12 `Expr` kinds | property test equal | P2 | **PASS** |
| R6 / UC-05 | diagnostics incl. labels and context | equal to JSON `diagnostics` | P1 | **PASS** |
| R7 / UC-01 | load error msg/hint/line/col/file | equals CLI text fields | P1 | **PASS** |
| R8 | ids and handles validated | ≥ 40 misuse cases, sanitizers silent | P4 | **PASS** |
| R9 | no panic crosses | forced panic returns `CAPY_E_PANIC` | P4 | **PASS** |
| R10 | deterministic order | equals `BTreeMap` / source order | P1 | **PASS** |
| R11 / UC-07 | ABI version + header drift test | `check_header.sh` exit 0 | P4 | **PASS** |
| R12 / UC-06 | opt-in `expr` JSON | default output byte-identical | P3 | **PASS** |
| R13 | differential equivalence | 0 differences over `samples/*` | P3 | **PASS** |
| R14 | memory safety verified | ASan+UBSan C runs | P4 | **PASS** |
| R15 | nothing existing changes | GATE-001, GATE-002, wasm size equal | P5 | **PARTIAL** |
| R16 | docs true | `mkdocs build --strict` exit 0 | P5 | **PASS** |
| R17 | measurements predeclared and run | M-01 … M-05 recorded | P5 | **PASS** |

## Applicable Project Standards

`GOAL-001`, `GOAL-002`, `PHIL-001`, `PHIL-002`, `CODE-001` (the owner asked for a release), `ARCH-001`,
`ARCH-002`, `ARCH-003`, `QUAL-001`, `QUAL-002`, `QUAL-003`, `GATE-001`, `GATE-002`.
`CODE-002`/`CODE-003` do not apply (no helper, directive or capture type).

## Measurable Claims

Frozen **before** any measurement. Baselines taken on the clean tree at `69fb4ac`, macOS arm64,
Darwin 25.4.0, rustc 1.90.0.

| ID | Metric | Baseline | Command | Threshold | Prediction (frozen) | Report |
|---|---|---|---|---|---|---|
| M-01 | static library size | `capy-core` alone as a staticlib, 5,154,280 bytes (measured 2026-10-13 by the consumer; re-measured by W-15 on the same tree) | `cargo build --profile ffi -p capy-ffi`; `stat -f %z target/ffi/libcapy_ffi.a` | ≤ 5.5 MB | 5.2 – 5.4 MB | `TEST-2026-0015` |
| M-02 | arena overhead | `Library::parse` time on each sample script, 15 interleaved samples | `cargo run --release -p capy-ffi --example bench` | ≤ +25 % | +8 % to +18 % | `TEST-2026-0015` |
| M-03 | per-accessor cost | none | 10^7 calls to `capy_node_func` | recorded only | 2 – 6 ns | `TEST-2026-0015` |
| M-04 | wasm size | **1,434,097 bytes** (`capy_wasm_abi.wasm` at `69fb4ac`) | `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`; `ls -l` | exactly equal | 0 bytes | `TEST-2026-0015` |
| M-05 | direct dependencies of `capy-core` | 1 | `cargo tree -p capy-core --depth 1` | exactly 1 | 1 | `TEST-2026-0015` |

Other baselines: `cargo test --workspace` = 153 passed, 0 failed, 0 ignored at `69fb4ac`.
A result far better than predicted is investigated before it is believed.

## Phases, Entry Gates and Exit Gates

| Phase | Content | Exit |
|---|---|---|
| P0 | contracts | `ADR-0005`; this plan committed **before** code |
| P1 | C-01, C-02, C-03, C-05 | crate builds; library/parse/tree/diag/error accessors; T-01, T-05, T-06 |
| P2 | C-04 | expression accessors; T-03 |
| P3 | C-06 | `expr` JSON, `--with-expr`, `capy_parse_to_json`; T-02, T-10 |
| P4 | C-07, C-08 | header, drift check, C smoke/misuse, sanitizers, panic, threads; T-04, T-07, T-08, T-09, T-11 |
| P5 | C-09, C-10 | docs, `TEST-2026-0015`; GATE-001/002; T-12 |
| P6 | validation, demo, manual, system, impact decisions, version, commit, tag, `REL-0.26.0` | §34 gate |

## Implementation Approach

1. **C-01.** Add member `ffi` and `[profile.ffi]` (`inherits = "release"`, `panic = "unwind"`) to `rust/Cargo.toml`;
   `rust/ffi/Cargo.toml` with `crate-type = ["staticlib","cdylib","rlib"]`, `publish = false`, dependency `capy-core` only.
2. **C-02/C-03/C-05.** `arena.rs`: `CapyParse { result: Box<ParseResult>, entries, kids, tag, poisoned }`. Flatten
   iteratively (explicit stack, no recursion) so deep input cannot overflow. Ids are `tag<<24 | (index+1)`.
   Every export: null-check, `catch_unwind`, id validation (tag, range, kind). Scalar accessors return 0, string
   accessors return `{NULL,0}`.
3. **C-04.** `expr.rs`: kinds 1…12; children through the `kids` pool; `Var` has one slot per step (0 for fields).
4. **C-06.** `ast_json.rs`: `to_json_with(r, include_expr)`; default `to_json*` call it with `false`. CLI `--with-expr`.
5. **C-07/C-08.** `cbindgen` header checked in; `ffi/check_header.sh`; `examples/c/{walk,misuse}.c`, `run.sh`
   with ASan/UBSan via `clang`; differential test walking only the exported functions.
6. **C-09/C-10.** Docs and measurement per the proposal.

| Risk | Mitigation |
|---|---|
| dangling arena pointers | `Box<ParseResult>` never moved or mutated; one SAFETY comment per dereference; sanitizers |
| a test that cannot fail | mutation check each of: tag check, kind check, `catch_unwind`, `expr` flag |
| default JSON drifts | golden corpus + explicit byte comparison against the baseline binary (T-10) |
| stale `capy` binary | build with `cargo build --workspace` (`TRBL-2026-0001`) |
| `panic = "abort"` profile used for capy-ffi by mistake | `capy_panics_are_caught()` and a test asserting it under `--profile ffi` |

## Live Work Checklist

| ID | Task | Phase | Req / Change | Files | Status | Evidence |
|---|---|---|---|---|---|---|
| W-01 | ADR, plan, baselines | P0 | — | `program_docs/decisions/adr-0005-*.md`, this file | DONE | wasm 1,434,097 B; 153 tests pass at `69fb4ac` |
| W-02 | workspace member, profile, crate skeleton | P1 | R1 / C-01 | `rust/Cargo.toml`, `rust/ffi/Cargo.toml`, `rust/ffi/src/lib.rs` | DONE | `rust/Cargo.toml` member + `[profile.ffi]`; `rust/ffi/Cargo.toml`; `cargo tree` shows one dependency each |
| W-03 | arena, ids, guard | P1 | R3, R8, R9, R10 / C-02 | `rust/ffi/src/arena.rs` | DONE | `arena.rs`: iterative flatten, tagged ids, `with` guard; 5 unit tests |
| W-04 | library, error, parse, block, node, capture, error-node, diagnostics exports | P1 | R2, R3, R4, R6, R7 / C-03, C-05 | `rust/ffi/src/lib.rs`, `diag.rs` | DONE | `lib.rs` (75 exports with `diag.rs`, `expr.rs`, `json.rs`); T-02, T-05, T-06 |
| W-05 | expression exports | P2 | R5 / C-04 | `rust/ffi/src/expr.rs` | DONE | `expr.rs`; T-03 (4 tests, 3 000 random trees) |
| W-06 | `expr` JSON writer, CLI flag | P3 | R12 / C-06 | `rust/src/domain/ast_json.rs`, `rust/cli/src/cmd_ast.rs` | DONE | `ast_json.rs` `expr_val` + `include_expr`; `cmd_ast.rs` `--with-expr`; T-10 |
| W-07 | `capy_parse_to_json`, `capy_buf_free` | P3 | R12 / C-06 | `rust/ffi/src/json.rs` | DONE | `json.rs`; `differential` compares `capy_parse_to_json` with the Rust writer |
| W-08 | differential + expression property + thread tests | P3, P4 | R2, R5, R13 / C-08 | `rust/ffi/tests/*.rs` | DONE | `differential.rs`, `expr.rs`, `threads.rs`: 9 tests; mutations M1–M5 caught after two tests were strengthened |
| W-09 | panic test and `capy_panics_are_caught` | P4 | R9 / C-02 | `rust/ffi/src/arena.rs` (`#[cfg(test)]`) | DONE | forced-panic tests for `with`, `capy_library_new`, `capy_parse`; mutation M6 |
| W-10 | header, `check_header.sh`, `cbindgen.toml` | P4 | R11 / C-07 | `rust/ffi/cbindgen.toml`, `rust/ffi/include/capy.h`, `rust/ffi/check_header.sh` | DONE | `capy.h` (75 functions) + `check_header.sh`: `capy.h is up to date` |
| W-11 | C smoke, misuse, hostile, sanitizer script | P4 | R8, R14 / C-08 | `examples/c/walk.c`, `misuse.c`, `run.sh` | DONE | `examples/c/`: `walk`, `diagnose`, `exprs`, `hostile` (30), `misuse` (76) under ASan + UBSan |
| W-12 | recursion-guard case; `check_publishable.py` | P4 | R1, R9 / C-01 | `rust/tests/recursion_guard.rs`, `rust/devtools/check_publishable.py` | DONE | `recursion_guard.rs` +4 tests (**scope grew**: `INC-2026-0002` fix in `value_parser.rs`); `check_publishable.py` enforces both rules |
| W-13 | docs | P5 | R16 / C-09 | see File Checklist | DONE | `docs/c-abi.md` + 5 pages, `mkdocs.yml`, `CHANGELOG.md`, `README.md`; `mkdocs build --strict` rc=0 |
| W-14 | gates GATE-001, GATE-002; cross build | P5 | R15 | — | DONE | build, clippy, 174 tests, golden, header, publishable all pass; `wasm_check.sh` fails identically on the baseline (`INC-2026-0003`) |
| W-15 | measurements, `TEST-2026-0015` | P5 | R17 / C-10 | `program_docs/testing/test-2026-0015-*.md`, `rust/ffi/examples/bench.rs` | DONE | `TEST-2026-0015`: M-02, M-03, M-05 pass; **M-01 and M-04 fail their frozen thresholds**, reported |
| W-16 | validation report, demo, manual, system, architecture | P6 | — | `RPT-2026-0005`, `DEMO-2026-0005`, `MAN-2026-0003`, `SYS-2026-0001`, `ARCH-2026-0002` | DONE | `RPT-2026-0005`, `DEMO-2026-0005`, `MAN-2026-0003`, `SYS-2026-0001` rev 5, `ARCH-2026-0002` |
| W-17 | version bump, commits, tag, `REL-0.26.0` | P6 | — | see Version checklist | DONE | see the Version table |

## File and Artifact Checklist

| Path | CRUD | Why | Req |
|---|---|---|---|
| `rust/Cargo.toml` | UPDATE | member `ffi`; `[profile.ffi]`; version `0.26.0` | R1, R9 |
| `rust/Cargo.lock` | UPDATE | new package; versions | R1 |
| `rust/ffi/Cargo.toml` | CREATE | crate manifest | R1 |
| `rust/ffi/src/lib.rs`, `arena.rs`, `expr.rs`, `diag.rs`, `json.rs` | CREATE | the ABI | R2–R10, R12 |
| `rust/ffi/tests/differential.rs`, `expr.rs`, `threads.rs` | CREATE | T-02, T-03, T-04 | R2, R5, R13 |
| `rust/ffi/examples/bench.rs` | CREATE | M-02, M-03 | R17 |
| `rust/ffi/cbindgen.toml`, `rust/ffi/include/capy.h`, `rust/ffi/check_header.sh` | CREATE | header and drift check | R11 |
| `examples/c/walk.c`, `misuse.c`, `hostile.c`, `run.sh` | CREATE | T-01, T-05…T-07 | R8, R14 |
| `rust/src/domain/ast_json.rs` | UPDATE | `expr` writer behind a flag | R12 |
| `rust/cli/src/cmd_ast.rs` | UPDATE | `--with-expr` | R12 |
| `rust/cli/tests/ast_with_expr.rs` | CREATE | T-10 CLI | R12 |
| `rust/tests/recursion_guard.rs` | UPDATE | expression-nesting cases (`INC-2026-0002`) | R9 |
| `rust/src/orchestrator/features/value_parser.rs`, `rust/src/orchestrator/features/make_parser.rs` | UPDATE | **added in execution:** the expression depth bound and its named `E0003` report | R9 |
| `examples/c/diagnose.c`, `examples/c/exprs.c` | CREATE | **added in execution:** demo programs for `DEMO-2026-0005` | R16 |
| `rust/devtools/check_publishable.py` | UPDATE | `capy-core` one dependency; `capy-ffi` unpublishable | R1 |
| `docs/c-abi.md`, `docs/embedding.md`, `docs/ast-json.md`, `docs/cli.md`, `docs/whats-new.md`, `mkdocs.yml`, `CHANGELOG.md` | CREATE / UPDATE | R16 | R16 |
| `rust/{cli,devtools,mcp,playground,wasm}/Cargo.toml` | UPDATE | version `0.26.0`, wasm path dependency version | release |
| `program_docs/**` | CREATE / UPDATE | §22–§34 | — |

## Test and Validation Checklist

| ID | Validates | Procedure | Expected | Status |
|---|---|---|---|---|
| T-01 | R1–R4, UC-01–03 | `examples/c/run.sh walk` | prints `greet(who=world)`, exit 0 | **PASS** |
| T-02 | R4, R6, R10, R13 | `cargo test -p capy-ffi --profile ffi --test differential` over every `samples/*/lib.capy` and scripts | 0 differences | **PASS** |
| T-03 | R5 | `--test expr`: all 12 kinds, int edges, nested paths | equal | **PASS** |
| T-04 | R2 | `--test threads`: 4 threads × 500 parses of one library | all equal | **PASS** |
| T-05 | R3 | `run.sh hostile` | handle + diagnostics, no crash | **PASS** |
| T-06 | R7 | broken grammar via C vs `CapyError` | five fields equal | **PASS** |
| T-07 | R8, R14 | `run.sh misuse` (ASan+UBSan) | ≥ 40 cases pass | **PASS** |
| T-08 | R9 | unit test forces a panic through `guard` | `CAPY_E_PANIC`, handle poisoned | **PASS** |
| T-09 | R11 | `rust/ffi/check_header.sh` | exit 0 | **PASS** |
| T-10 | R12, R15 | `capy ast --json` byte-identical to the `69fb4ac` binary over all samples; `--with-expr` adds `expr` | 0 diffs; field present | **PASS** |
| T-11 | R9, ARCH-003 | recursion-guard case for the JSON dump | no abort | **PASS** |
| T-12 | R1, R15, R17 | cross build of the staticlib for `aarch64-unknown-linux-gnu`; wasm size | builds; wasm 1,434,097 B | **PARTIAL** |
| T-13 | GATE-001 | `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `mkdocs build --strict` | exit 0 | **PASS** |
| T-14 | GATE-002 | `capy check` all samples; `cargo test --test golden`; `wasm_check.sh` if `deno` is present | 0 failed | **PARTIAL** |

## Section 31 Documentation-Impact Decision

| Artifact | Decision | Reason |
|---|---|---|
| Release verification guide / demo | UPDATED | `DEMO-2026-0005` |
| README.md | UPDATED | a headline capability: embedding from C (`rust/README.md` and top-level `README.md` embedding line) |
| System documentation | UPDATED | `SYS-2026-0001` gains the FFI layer |
| Architecture documentation | UPDATED | new component; `ARCH-2026-0002` (arena and ownership) |
| API and CLI reference | UPDATED | `docs/c-abi.md`, `docs/ast-json.md`, `docs/cli.md` |
| Manual | UPDATED | `MAN-2026-0003` |
| Troubleshooting | NOT APPLICABLE | unless an incident arises; decided at validation |

## Version, Release and Rollout Checklist

| Step | Detail | Status |
|---|---|---|
| Version source | `rust/Cargo.toml` `[workspace.package]`; six member manifests' `capy-core` pins; `Cargo.lock` (7 files) | DONE — `d1cf161` |
| Commits | `79dae85` plan (before code) · `114525b` feature · `d1cf161` chore(release) · `docs(program)` **tagged** · finalize | DONE (hashes in `REL-0.26.0`) |
| Tag | `v0.26.0`; `git rev-list -n 1 v0.26.0` equals the recorded SHA | DONE (see `REL-0.26.0`) |
| Push | **not authorized** — the owner asked for a release, not a push; local commit and tag only, push is offered afterwards | NOT APPLICABLE (not performed) |
| Release document | `REL-0.26.0`, after the tag | DONE |

## Decisions, Findings, Deviations and Blockers

| Date | Kind | Entry |
|---|---|---|
| 2026-10-13 | Deviation | Six deviations from the proposal are frozen in `ADR-0005` (`CAPY_E_LOAD`, `CAPY_E_LIMIT`, `rlib`, best-effort foreign-id tag, T-12 platform, diagnostic message composition) |
| 2026-10-13 | Finding | `ARCH-003` was marked `PASS` in the proposal on the belief that `MAX_PARSE_DEPTH` bounded expressions; it did not. ~20 000 nested parentheses abort the process on 0.25.0. Fixed in scope (`INC-2026-0002`); this is a `capy-core` behaviour change the plan did not list |
| 2026-10-13 | Deviation | After validation the over-deep expression message was changed from the generic `E0001` to a named `E0003`: `make_parser.rs` (value branch of capture parsing) and `value_parser.rs` carry the bound error to `OuterP.depth_err`. Message text only; boundary unchanged. Release re-validated and re-tagged locally (nothing had been pushed) |
| 2026-10-13 | Finding | `wasm_check.sh` fails on `parse-recovery/broken.capy` identically at `69fb4ac` (`INC-2026-0003`); not caused by this release |
| 2026-10-13 | Result | `M-01` (31,225,392 B vs ≤ 5.5 MB) and `M-04` (+1,950 B vs equal) missed their frozen thresholds; reported unchanged in `TEST-2026-0015` |
| 2026-10-13 | Finding | The 2026-10-08 wasm build is 1,434,097 bytes, not the 1.34 MB quoted in the consumer's research; M-04 uses the measured figure |

## Rollback Strategy

Revert the release commit. `capy-ffi` has no dependants; the only edit to shipped code is the opt-in `expr`
writer, which is independent of the default path.

## Completion Criteria and Final Traceability

`DOCUMENTATION.md` §34 passes. `Release → Validation → Tests → Implementation → Plan` resolves for every `R` and `UC`.

## Open Questions

`PROP-2026-0008` OQ-01 (canonical repository), OQ-02 (pkg-config), OQ-03 (byte offsets), OQ-04 (`run`), OQ-05 (spans on `Expr`) —
none blocks this release; all recorded as follow-up.

## Related Documents

- `PROP-2026-0008`, `ADR-0005`, `PROP-2026-0007`, `PLAN-2026-0004`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial approved plan, committed before any code (`79dae85`) |
| 2 | 2026-10-13 | Olivier, Claude | Closed: all phases DONE; two scope additions and two missed predictions recorded; release in `REL-0.26.0` |
