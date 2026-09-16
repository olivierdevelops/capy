---
document_id: TEST-2026-0004
title: Test — Measured Results for the 0.21.0 Parser Changes
document_type: test
status: completed

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
  - capy-core
  - capy-wasm-abi

affected_versions:
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the predeclared measurements for transpile time, wasm module size and dependency count against thresholds frozen before implementation.

reason: DOCUMENTATION.md section 21.3 requires every measurable claim to execute a predeclared test with a baseline and threshold fixed in advance.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - performance
  - size
  - measurement

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — Measured Results for the 0.21.0 Parser Changes

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi

## Purpose

Execute the measurements predeclared in PROP-2026-0001 and PLAN-2026-0001.
Spans add fields to every AST node and comment retention adds tokens, so both
transpile time and module size were expected to move.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0001 M-01 | Native transpile time within 10 % of baseline |
| PLAN-2026-0001 M-02 | wasm module size within 5 % of baseline |
| PLAN-2026-0001 M-03 | `capy-core` keeps exactly one direct dependency |

## Preconditions

Baselines were frozen **before** implementation and may not be edited
retroactively (section 21.3).

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable 1.90.0, release profile
(`opt-level = "z"`, LTO, `codegen-units = 1`, strip). Same host as the baseline.
M-01 is host-sensitive; the comparison is only valid on this host.

## Test Data

`samples/transpile-py/lib.capy` + `samples/transpile-py/script.capy`, the same
pair used for the baseline.

## Procedure

```sh
cargo build --release -p capy-devtools --bin nativebench --manifest-path rust/Cargo.toml
./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy   # x5, best of

cargo build --release --target wasm32-unknown-unknown --manifest-path rust/Cargo.toml -p capy-wasm-abi
ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm

cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1
```

## Expected Results

Thresholds frozen 2026-09-16, before any implementation:

| Measurement | Baseline | Threshold |
|---|---|---|
| M-01 transpile time | 187–219 µs (2026-09-14) | ≤ 10 % regression |
| M-02 wasm size | 1 342 891 bytes (2026-09-16) | ≤ 5 % growth |
| M-03 direct dependencies | 1 (`regex`) | exactly 1 |

## Actual Results

| Measurement | Observed | Against threshold | Result |
|---|---|---|---|
| M-01 | 188.185 µs best of 5 (188.2, 189.2, 189.7, 190.7, 222.7) | inside the 187–219 µs baseline band; **no regression** | PASS |
| M-02 | 1 367 674 bytes | **+1.85 %** against a 5 % allowance | PASS |
| M-03 | 1 (`regex`) | exactly 1 | PASS |

The 222.7 µs sample is a first-run outlier; the declared method is best-of-5,
which is 188.185 µs. All five samples are reported rather than only the best, so
the spread is visible.

## Result

PASS

## Evidence

M-02 grew 24 783 bytes. That is the cost of two `usize` pairs on every AST node
plus the trivia path, and it is well inside the allowance. It is recorded rather
than rounded away because M-02 is the measurement most likely to constrain a
later plan: PLAN-C adds error nodes and PLAN-D adds a JSON serializer, and the
5 % allowance is shared across the release, not per change.

## Evidence Sources

- `./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy`
- `ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm`
- `cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None.

## Related Documents

- PLAN-2026-0001 — Measurable Claims
- PROP-2026-0001 — Measurement and Validation

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
