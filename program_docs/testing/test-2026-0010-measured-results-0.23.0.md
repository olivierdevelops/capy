---
document_id: TEST-2026-0010
title: Test — Measured Results for 0.23.0
document_type: test
status: completed

created_date: 2026-10-07
last_updated: 2026-10-07
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
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the predeclared measurements M-01 to M-04 of PLAN-2026-0003 against baselines taken on the working tree immediately before the ordered-alternation change.

reason: DOCUMENTATION.md section 27.1 requires every measurable claim to execute its predeclared test with baseline, environment, command, threshold, actual values and limitations.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - ADR-0003
  - TEST-2026-0008
  - TEST-2026-0004
  - TEST-2026-0009
  - RPT-2026-0003

supersedes: null
superseded_by: null

tags:
  - performance
  - wasm
  - dependencies
  - measurement

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Test — Measured Results for 0.23.0

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi

## Purpose

Execute the measurements predeclared in `PLAN-2026-0003` "Measurable Claims".
The matcher now tries several functions per capture and the loader walks more
guard edges, so time and size were expected to move; the 5 % wasm allowance is
shared with earlier releases and was the tightest constraint.

```text
   WHAT WAS PREDECLARED (before any engine edit)
   ┌──────┬──────────────────────────┬───────────────────┬────────────────────┐
   │ ID   │ metric                   │ baseline          │ threshold          │
   ├──────┼──────────────────────────┼───────────────────┼────────────────────┤
   │ M-01 │ transpile time, best of 5│ 190.903 µs        │ <= +10 % (210.0 µs)│
   │ M-02 │ wasm size                │ 1 386 116 B       │ <= 1 410 035 B     │
   │      │                          │ (1 342 891 @0.21) │  (cumulative 5 %)  │
   │ M-03 │ capy-core direct deps    │ 1 (regex)         │ exactly 1          │
   │ M-04 │ golden suite wall time   │ measured pre-merge│ <= 1.15x baseline  │
   └──────┴──────────────────────────┴───────────────────┴────────────────────┘
```

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0003 M-01 | Transpile time within +10 % of the 190.903 µs baseline |
| PLAN-2026-0003 M-02 | wasm size within the cumulative 5 % allowance (<= 1 410 035 bytes) |
| PLAN-2026-0003 M-03 | `capy-core` keeps exactly one direct dependency |
| PLAN-2026-0003 M-04 | Golden suite wall time <= 1.15x baseline (RK-02: capture-local rewind must not make the suite slow) |

## Preconditions

- Baseline figures were measured on this tree **before** any `PROP-2026-0004`
  change, on the same machine.
- No other heavy process running during timing runs.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable (`rustc 1.90.0`), **release profile**,
same host as the baseline. M-01 and M-04 are host-sensitive; they compare two
builds on one host, never absolute numbers across hosts.

## Test Data

| Measurement | Dataset |
|---|---|
| M-01 | `samples/transpile-py/lib.capy`, `samples/transpile-py/script.capy` |
| M-02 | `capy-wasm-abi` crate, `wasm32-unknown-unknown`, release |
| M-03 | `capy-core` manifest |
| M-04 | the full `samples/` golden corpus through the `golden` test binary |

## Procedure

```sh
# M-01 — transpile time
cargo build --release -p capy-devtools --bin nativebench
./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy   # x5, best of

# M-02 — wasm size
cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi
ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm

# M-03 — direct dependencies
cargo tree -p capy-core --depth 1

# M-04 — golden suite wall time, release test binaries, best of 7
#   run the same command against the baseline copy and the current tree
cargo test --release --test golden
```

The **baseline copy** for M-04 is this tree with the 7 engine files restored from
`HEAD` and the new tests and samples removed, built in a scratch directory, so
both sides run on one machine in one session.

## Expected Results

| Measurement | Baseline | Threshold |
|---|---|---|
| M-01 | 190.903 µs (best of 5) | <= +10 % i.e. <= 210.0 µs |
| M-02 | 1 386 116 bytes (this tree before the change); 1 342 891 at 0.21.0 | <= 1 410 035 bytes cumulative (5 % over 0.21.0); <= +23 919 bytes this release |
| M-03 | 1 | exactly 1 |
| M-04 | baseline copy wall time | <= 1.15x |

## Actual Results

### M-01 — transpile time (`nativebench`, best of 5)

| Run | Baseline (µs) | After (µs) |
|---:|---:|---:|
| 1 | 191.18 | 195.635 |
| 2 | 190.903 | 194.935 |
| 3 | 192.296 | 195.533 |
| 4 | 192.4 | 195.363 |
| 5 | 192.762 | 195.901 |
| **best** | **190.903** | **194.935** |

```text
   best-of-5 change:  (194.935 - 190.903) / 190.903  =  +2.11 %
   threshold       :  <= +10 %   (<= 210.0 µs)

   190.9 ──┤███████████████████▌                        baseline
   194.9 ──┤███████████████████▉                        after  (+2.11 %)
   210.0 ──┤█████████████████████                       ceiling
```

All ten samples are reported, not only the best. Spread inside each series is
under 2 µs, and every "after" run is above every "baseline" run, so the +2 %
shift is real and small, not noise.

### M-02 — wasm size

| Point | Bytes | Delta vs previous | Cumulative vs 0.21.0 |
|---|---:|---:|---:|
| 0.21.0 | 1 342 891 | — | 0 % |
| this tree before the change (0.22.0 + carried work) | 1 386 116 | +43 225 | +3.22 % |
| **after the change** | **1 389 616** | **+3 500 (+0.25 %)** | **+3.48 %** |
| predeclared ceiling | 1 410 035 | — | +5.00 % |
| **remaining headroom** | **20 419** | | |

```text
   shared 5 % allowance, cumulative since 0.21.0  (bar = 1 410 035 B ceiling)

   0.21.0    ██████████████████████████████████████████████░░░░░░░░░░   0.00 %
   0.22.x    ████████████████████████████████████████████████▌░░░░░░   3.22 %
   0.23.0    █████████████████████████████████████████████████▎░░░░░   3.48 %
                                                                 ▲
                                                           ceiling 5.00 %
   headroom left: 20 419 bytes (1.52 % of the 0.21.0 size)
```

Confirmed on the final tree on 2026-10-07:

```text
$ ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm
-rwxr-xr-x  1 oliverlaleau  staff  1389616 Oct  7 14:16 rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm
```

### M-03 — direct dependencies

```text
$ cargo tree -p capy-core --depth 1
capy-core v0.22.0 (/Users/oliverlaleau/Documents/projects/capylang-claude/rust)
└── regex v1.13.1
```

Exactly one direct dependency. (`capy-core` still prints `v0.22.0` because the
version bump is a release step that follows validation, per `PLAN-2026-0003`.)

### M-04 — golden suite wall time (release test binaries, best of 7)

| Build | Best of 7 (s) |
|---|---:|
| baseline copy | 0.0611 |
| current tree | 0.0612 |
| **ratio** | **1.002** |

```text
   current / baseline = 0.0612 / 0.0611 = 1.002x       threshold <= 1.15x

   1.00x ─┤██████████████████████████                  baseline
   1.002x─┤██████████████████████████                  current
   1.15x ─┤██████████████████████████████              ceiling
```

The current tree runs six more goldens than the baseline copy (131 vs 125), so a
ratio of 1.002 is a slight overstatement of per-case cost, not an understatement.

## Result

PASS

| Measurement | Observed | Against threshold | Result |
|---|---|---|---|
| M-01 | 194.935 µs best of 5 (+2.11 %) | <= +10 % (<= 210.0 µs) | PASS |
| M-02 | 1 389 616 bytes (+3 500, +0.25 % this release; +3.48 % cumulative) | <= 1 410 035 bytes | PASS |
| M-03 | 1 (`regex`) | exactly 1 | PASS |
| M-04 | ratio 1.002 | <= 1.15x | PASS |

## Evidence

The 5 % wasm allowance is **shared across releases**, not per change (see
`TEST-2026-0008`, where 0.22.0 consumed 3.22 %). Ordered alternation consumed a
further 0.25 %. Only **20 419 bytes** of headroom remain; the next increment that
touches the wasm build must either fit inside that or ship with a re-baselined
allowance approved by an ADR. This is the most useful number in this document.

Limitations that may invalidate comparisons:

- M-01 and M-04 are single-host, single-session measurements. A different CPU or
  background load can move them; the 10 % and 15 % thresholds are set well above
  the observed 2 % and 0.2 %.
- M-04 times the golden suite, which exercises few alternation captures
  (`samples/expression-grammar/`, `samples/mixed-parameters/`). It bounds
  regression on existing libraries (R7), not the cost of deeply nested
  alternation, which is bounded separately by the 64-capture depth limit.
- M-02 baseline of 1 386 116 bytes is the tree before this change, not the
  `v0.22.0` tag, because the carried docs-and-samples work (`PROP-2026-0002`) and
  the `ast_text` module were already in the working tree.
- The wasm figure differed by one byte from the 0.22.0 figure quoted in
  `TEST-2026-0008` (1 386 117 vs 1 386 116); recorded as measured.

## Evidence Sources

- `./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` (x5)
- `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`; `ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm`
- `cargo tree -p capy-core --depth 1`
- `cargo test --release --test golden` (baseline copy and current tree)

## Executed By

Capy Engine (Olivier, with an AI agent re-confirming M-02 and M-03)

## Executed At

2026-10-07

## Defects Raised

None.

## Final-tree re-measurement

After the version bump and the `capy docs` pipe-escaping fix, the wasm build was re-measured on the
release tree: **1 389 633 bytes** (+17 bytes against the figure above), still below the 1 410 035
ceiling. `cargo tree -p capy-core --depth 1` reports `capy-core v0.23.0` with exactly one dependency
(`regex`). `cargo test --workspace`: 128 passed, 0 failed; golden suite 131 passed, 8 skipped, 0 failed.

## Related Documents

- PLAN-2026-0003 — "Measurable Claims" M-01 to M-04
- PROP-2026-0004
- ADR-0003
- TEST-2026-0008 — the 0.22.0 measurements and the shared 5 % allowance
- TEST-2026-0004 — the 0.21.0 baseline of 1 342 891 bytes
- TEST-2026-0009 — functional tests
- RPT-2026-0003

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
