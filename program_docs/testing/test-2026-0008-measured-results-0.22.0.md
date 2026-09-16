---
document_id: TEST-2026-0008
title: Test — Measured Results for 0.22.0
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

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the predeclared measurements for 0.22.0 against the baselines frozen before implementation.

reason: DOCUMENTATION.md section 21.3 requires every measurable claim to execute a predeclared test.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - diagnostics
  - recovery

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — Measured Results for 0.22.0

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi

## Purpose

Execute the measurements predeclared in PLAN-2026-0002. Furthest-failure tracking
runs on every rejection, and four features were added, so both time and size were
expected to move.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0002 M-01 / R25 | Transpile time within 10 % of baseline |
| PLAN-2026-0002 M-02 / R13 | wasm size within 5 % **total** of baseline |
| PLAN-2026-0002 M-03 / R8 | `capy-core` keeps one direct dependency |

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable 1.90.0, release profile, same host as
the baseline. M-01 is host-sensitive.

## Procedure

```sh
cargo build --release -p capy-devtools --bin nativebench --manifest-path rust/Cargo.toml
./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy   # x5, best of
cargo build --release --target wasm32-unknown-unknown --manifest-path rust/Cargo.toml -p capy-wasm-abi
cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1
```

## Expected Results

| Measurement | Baseline | Threshold |
|---|---|---|
| M-01 | 187–219 µs | ≤ 10 % regression |
| M-02 | 1 342 891 bytes | ≤ 5 % total growth |
| M-03 | 1 | exactly 1 |

## Actual Results

| Measurement | Observed | Against threshold | Result |
|---|---|---|---|
| M-01 | 192.036 µs best of 5 (192.0, 194.3, 194.7, 197.5, 234.0) | inside the baseline band — **R25 satisfied, furthest-failure tracking is not measurable here** | PASS |
| M-02 | 1 386 117 bytes | **+3.22 %** total against a 5 % allowance (0.21.0 was +1.85 %, so this increment added ~1.4 %) | PASS |
| M-03 | 1 (`regex`) | exactly 1 | PASS |

All five M-01 samples are reported rather than only the best; the 234 µs sample
is a first-run outlier and the declared method is best-of-5.

## Result

PASS

## Evidence

The 5 % wasm allowance is **shared across the release**, not per change. Two
increments have now consumed 3.22 % of it. A third would need to fit in the
remaining 1.78 %, which is worth knowing before planning one.

## Evidence Sources

- `./rust/target/release/nativebench …`
- `ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm`
- `cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None.

## Related Documents

- PLAN-2026-0002
- TEST-2026-0004 — the 0.21.0 measurements this compares against

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
