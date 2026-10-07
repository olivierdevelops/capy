---
document_id: TEST-2026-0013
title: Test — Measured Results for 0.24.0
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
  from: "0.24.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the predeclared measurements M-01 to M-04 of PLAN-2026-0004 against baselines taken on the clean v0.23.0 tree before the 0.24.0 code was re-applied.

reason: DOCUMENTATION.md section 27.1 requires every measurable claim to execute its predeclared test with baseline, environment, command, threshold, actual values and limitations.

related_documents:
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - TEST-2026-0010
  - TEST-2026-0008
  - TEST-2026-0012
  - RPT-2026-0004

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

# Test — Measured Results for 0.24.0

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi

## Purpose

Execute the measurements predeclared in `PLAN-2026-0004` "Measurable Claims". The
release adds a field to the parser state, a JSON array to the wasm export and a
version fallback, so time and size were expected to move very little; the 5 % wasm
allowance shared with earlier releases was the tightest constraint.

```text
   WHAT WAS PREDECLARED (frozen in PLAN-2026-0004 before any "after" measurement)
   ┌──────┬───────────────────────────┬───────────────────────┬──────────────────────┐
   │ ID   │ metric                    │ baseline              │ threshold            │
   ├──────┼───────────────────────────┼───────────────────────┼──────────────────────┤
   │ M-01 │ transpile time, best of 7 │ retained nativebench  │ <= +10 %             │
   │      │ interleaved               │ of the v0.23.0 tree   │                      │
   │ M-02 │ wasm size                 │ 1 389 633 bytes       │ <= +3 000 bytes;     │
   │      │                           │                       │ ceiling 1 410 035    │
   │ M-03 │ capy-core direct deps     │ 1                     │ exactly 1            │
   │ M-04 │ golden suite wall time,   │ retained golden test  │ <= 1.15x             │
   │      │ best of 7 interleaved     │ binary of v0.23.0     │                      │
   └──────┴───────────────────────────┴───────────────────────┴──────────────────────┘
```

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0004 M-01 | Transpile time within +10 % of the retained v0.23.0 `nativebench` |
| PLAN-2026-0004 M-02 | wasm size within +3 000 bytes of 1 389 633 and under the 1 410 035 cumulative ceiling |
| PLAN-2026-0004 M-03 | `capy-core` keeps exactly one direct dependency |
| PLAN-2026-0004 M-04 | Golden suite wall time <= 1.15x the retained v0.23.0 test binary |

## Preconditions

- The 0.24.0 code was drafted before the proposal, ADR and plan existed (a deviation
  recorded in `PLAN-2026-0004` and `RPT-2026-0004`). Before anything was measured it
  was saved as a patch and removed from the tree, so the baselines below were measured
  on the **clean** v0.23.0 tree and the thresholds were committed (`a046032`) before
  any "after" number existed.
- No other heavy process was intentionally running; the host is nevertheless noisy, so
  M-01 and M-04 are interleaved against retained baseline binaries, not compared
  against remembered numbers.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable, **release profile**. M-01 and M-04 compare
two builds on one host in one session, never absolute numbers across hosts.

## Test Data

| Measurement | Dataset |
|---|---|
| M-01 | `samples/transpile-py/lib.capy`, `samples/transpile-py/script.capy` |
| M-02 | `capy-wasm-abi` crate, `wasm32-unknown-unknown`, release |
| M-03 | `capy-core` manifest |
| M-04 | the full `samples/` golden corpus through the `golden` test binary |

## Procedure

```text
   1. save the 0.24.0 code as a patch            (git diff > patch)
   2. git checkout -- rust                        clean v0.23.0 tree
   3. build + measure baselines                   nativebench, golden test binary,
        copy both binaries ASIDE                  wasm size
   4. commit PROP-2026-0005, ADR-0004, PLAN-2026-0004     (a046032, thresholds frozen)
   5. re-apply the patch, rebuild                 (committed as 8769f51)
   6. run baseline and new binaries ALTERNATELY   best of 7
```

```sh
# M-01 — transpile time (baseline binary and new binary run alternately, 7 each)
cargo build --release -p capy-devtools --bin nativebench
nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy

# M-02 — wasm size
cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi
ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm

# M-03 — direct dependencies
cargo tree -p capy-core --depth 1

# M-04 — golden suite wall time, release test binaries, best of 7, interleaved
cargo test --release --test golden        # baseline copy vs current tree
```

## Expected Results

| Measurement | Baseline | Threshold |
|---|---|---|
| M-01 | retained `nativebench` of the v0.23.0 tree (single runs 196.9 to 218.1 µs on this host, as recorded in the plan) | <= +10 % |
| M-02 | 1 389 633 bytes | <= +3 000 bytes; ceiling 1 410 035 |
| M-03 | 1 | exactly 1 |
| M-04 | retained golden test binary of the v0.23.0 tree | <= 1.15x |

## Actual Results

### M-01 — transpile time (`nativebench`, best of 7, interleaved)

Measured 2026-10-07. All fourteen samples are reported, sorted ascending.

| Rank | Baseline v0.23.0 (µs) | New 0.24.0 (µs) |
|---:|---:|---:|
| 1 | **195.896** | **195.687** |
| 2 | 196.659 | 195.834 |
| 3 | 198.849 | 196.381 |
| 4 | 198.868 | 196.981 |
| 5 | 199.093 | 197.164 |
| 6 | 199.998 | 197.915 |
| 7 | 203.523 | 198.160 |

```text
   best 195.687 / best 195.896 = 0.9989        threshold <= 1.10

   195.9 ──┤█████████████████████████████████▌            baseline best
   195.7 ──┤█████████████████████████████████▍            new best   (0.9989)
   215.5 ──┤████████████████████████████████████▌         ceiling (+10 %)
```

Every new run is at or below the baseline's median, so no regression is visible;
the difference between the two bests (0.2 µs) is inside the host's noise and is not
claimed as an improvement.

### M-02 — wasm size

| Point | Bytes | Delta vs previous | Cumulative vs 0.21.0 |
|---|---:|---:|---:|
| 0.21.0 | 1 342 891 | n/a | 0 % |
| 0.23.0 (baseline of this plan) | 1 389 633 | n/a | +3.48 % |
| **0.24.0** | **1 390 420** | **+787 (+0.06 %)** | **+3.54 %** |
| predeclared per-release threshold | 1 392 633 | +3 000 | n/a |
| cumulative ceiling | 1 410 035 | n/a | +5.00 % |
| **remaining headroom** | **19 615** | | |

```text
   shared 5 % allowance, cumulative since 0.21.0  (bar = 1 410 035 B ceiling)

   0.21.0    ██████████████████████████████████████████████░░░░░░░░░░   0.00 %
   0.23.0    █████████████████████████████████████████████████▎░░░░░   3.48 %
   0.24.0    █████████████████████████████████████████████████▍░░░░░   3.54 %
                                                                 ▲
                                                           ceiling 5.00 %
   headroom left: 19 615 bytes
```

Re-confirmed for this document on the `cb23972` tree:

```text
$ cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi
$ ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm
-rwxr-xr-x  1 oliverlaleau  staff  1390420 Oct  7 15:11 rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm
```

(The build reported the artifact already up to date, so the file is the one produced
from the committed tree; its size equals the figure measured earlier.)

### M-03 — direct dependencies

```text
$ cargo tree -p capy-core --depth 1
capy-core v0.24.0 (/Users/oliverlaleau/Documents/projects/capylang-claude/rust)
└── regex v1.13.1
```

Exactly one direct dependency.

### M-04 — golden suite wall time (release test binaries, best of 7, interleaved)

| Build | Best of 7 (s) |
|---|---:|
| baseline (v0.23.0 test binary) | 0.0637 |
| new (0.24.0) | 0.0641 |
| **ratio** | **1.005** |

```text
   new / baseline = 1.005x                      threshold <= 1.15x

   1.00x ─┤██████████████████████████                  baseline
   1.005x─┤██████████████████████████▏                 new
   1.15x ─┤██████████████████████████████              ceiling
```

The ratio 1.005 was computed from the unrounded timings; the four-digit figures
above give 1.006. Either is far below the threshold. Both builds run the same 131
goldens (no golden was added in this release).

## Result

PASS

| Measurement | Observed | Against threshold | Result |
|---|---|---|---|
| M-01 | best 195.687 µs vs 195.896 µs, ratio 0.9989 | <= +10 % | PASS |
| M-02 | 1 390 420 bytes (+787, +0.06 %; +3.54 % cumulative) | <= +3 000 bytes; <= 1 410 035 | PASS |
| M-03 | 1 (`regex`) | exactly 1 | PASS |
| M-04 | ratio 1.005 | <= 1.15x | PASS |

## Evidence

The 5 % wasm allowance is **shared across releases** (see `TEST-2026-0008` and
`TEST-2026-0010`, where 0.22.0 consumed 3.22 % and 0.23.0 a further 0.25 %). This
release consumed 0.06 % more. Only **19 615 bytes** remain; the next change that
touches the wasm build must fit inside that or ship with a re-baselined allowance
approved by an ADR.

Limitations that may invalidate comparisons:

- M-01 and M-04 are single-host, single-session measurements on a noisy host. Their
  thresholds (10 %, 15 %) are set far above the observed differences; the observed
  differences themselves are not claimed to be real.
- The M-01 and M-04 baseline binaries were produced from the clean v0.23.0 tree and
  retained by copying them aside; this document cannot re-run them because they are
  not committed. The baseline columns are therefore as recorded on 2026-10-07, not
  re-measured while writing this document. Only M-02 and M-03 were re-confirmed.
- M-04 times the golden suite, which does not exercise the nesting bound. It bounds
  regression on existing libraries (R7), not the cost of the depth report.
- The M-02 baseline (1 389 633) is the `v0.23.0` release-tree figure from `TEST-2026-0010`.

## Evidence Sources

- `nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` (7 baseline + 7 new, interleaved)
- `cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi`; `ls -l rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm`
- `cargo tree -p capy-core --depth 1`
- `cargo test --release --test golden` (retained v0.23.0 binary and current tree)

## Executed By

Capy Engine (Olivier, with an AI agent measuring and re-confirming M-02 and M-03)

## Executed At

2026-10-07

## Defects Raised

None.

## Related Documents

- PLAN-2026-0004 — "Measurable Claims" M-01 to M-04
- PROP-2026-0005
- ADR-0004
- TEST-2026-0010 — the 0.23.0 measurements
- TEST-2026-0008 — the 0.22.0 measurements and the shared 5 % allowance
- TEST-2026-0012 — functional tests
- RPT-2026-0004

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
