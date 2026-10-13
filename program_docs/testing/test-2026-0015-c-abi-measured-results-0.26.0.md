---
document_id: TEST-2026-0015
title: Test — C ABI, Expression JSON and Nesting Bound, with Measured Results (0.26.0)
document_type: test
status: completed

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 1

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

scope: Defines and records every test and measurement of PLAN-2026-0005, including the predeclared measurable claims M-01 to M-05.

reason: DOCUMENTATION.md sections 27 and 27.1 require each test and measurable claim to be recorded with baseline, environment, command, threshold and actual values.

related_documents:
  - PLAN-2026-0005
  - PROP-2026-0008
  - ADR-0005
  - RPT-2026-0005
  - INC-2026-0002
  - INC-2026-0003
  - TEST-2026-0013
  - TEST-2026-0014
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - test
  - ffi
  - c-abi
  - measurement

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-13
---

# Test — C ABI, Expression JSON and Nesting Bound, with Measured Results (0.26.0)

> **Status:** Completed
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Purpose

Executes the test and validation checklist of `PLAN-2026-0005` and the measurable claims `M-01`…`M-05`
that were frozen in that plan **before** any measurement. Every sample is reported (`QUAL-002`).

```text
  PLAN-2026-0005 R1…R17 ──► implementation ──► this document ──► PASS / PARTIAL / FAIL ──► RPT-2026-0005
```

## Validated Requirements

`PLAN-2026-0005` R1–R17 and UC-01–UC-07 (traceability in the table under *Actual Results*).

## Preconditions

- Tree: `d1cf161` (`chore(release): 0.26.0`) plus the `docs(program)` commit that carries the tag: it adds `program_docs/`, two demo programs (`examples/c/diagnose.c`, `exprs.c`), one test (`the_bound_is_exactly_63_pairs_of_parentheses`) and doc wording. No other source changed.
- Baseline for comparisons: commit `69fb4ac98033af4bb2a8ea8afdc4c5977898a29e`, built from `git archive` into a scratch
  directory (release `capy`, 1,751,744 bytes) and, for the wasm check, a clean `git worktree`.

## Test Environment

macOS 26.4.1 arm64 (Darwin 25.4.0), rustc/cargo 1.90.0, Apple clang 21.0.0, zig 0.16.0, deno 2.5.6, cbindgen 0.29.4,
mkdocs. Laptop host, other work running; **timings are noisy** (see M-02 spread). No Linux host was available.

## Test Data

`samples/*/` (123 libraries, 133 scripts), hand-built `ParseResult`s for expression kinds no grammar produces,
a seeded xorshift generator (seed `0x9E3779B97F4A7C15`, 3 000 trees of depth ≤ 4), and generated hostile inputs.

## Procedure

```sh
cd rust
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --profile ffi -p capy-ffi --all-targets -- -D warnings
cargo test --workspace                      # T-01..T-11 Rust side; T-13
cargo test --profile ffi -p capy-ffi        # the same tests under the ffi profile
cargo test --test golden                    # T-14
cd .. && examples/c/run.sh all              # T-01, T-05, T-07 (ASan + UBSan)
rust/ffi/check_header.sh                    # T-09
rust/devtools/ast_json_compat.sh <baseline-capy>     # T-10
python3 rust/devtools/check_publishable.py  # M-05 / R1
mkdocs build --strict                       # R16
cargo rustc --profile ffi --target aarch64-unknown-linux-gnu -p capy-ffi --lib --crate-type staticlib   # T-12
rust/devtools/wasm_check.sh                 # T-14 (and on the baseline worktree)
cargo run --profile ffi -p capy-ffi --example bench   # M-02, M-03 (three sessions)
```

## Expected Results

As frozen in `PLAN-2026-0005` §Measurable Claims and §Test and Validation Checklist; repeated beside each actual result.

## Actual Results

### Functional tests

| ID | Validates | Result | Actual |
|---|---|---|---|
| T-01 | R1–R4, UC-01–03 | **PASS** | `examples/c/run.sh walk`: `walk: PASS (greet(who=world))`, built with `zig cc -O2` |
| T-02 | R4, R6, R10, R13 | **PASS** | `differential`: **132 scripts**, 10 with diagnostics, 110 with expressions; C-walked document == Rust JSON byte for byte, with and without `expr`; `capy_parse_to_json` equal too. A capture-order assertion was **added after** mutation M1 survived (below) |
| T-03 | R5 | **PASS** | 4 tests: all 12 kinds by hand, an index step with no expression, wrong-kind/foreign ids, 3 000 seeded random trees (generator asserted to reach every kind), `i64::MIN/MAX` |
| T-04 | R2 | **PASS** | 4 threads × 500 parses of one library, each equal to the Rust document |
| T-05 | R3 | **PASS** | `hostile`: 30 checks — empty, newlines, NUL bytes, invalid UTF-8, lone quote, 1 MiB line, 10 000 `(` and `{`, 50 000 statements, 64 KiB random bytes; handle + JSON dump for each, handle never poisoned |
| T-06 | R7 | **PASS** | `load_errors_match` (msg, hint, file, line, col equal to the Rust `CapyError`); `rejected_sample_libraries_are_rejected_over_c_too`; C `misuse` |
| T-07 | R8, R14 | **PASS** | `misuse`: **76 checks** under `-fsanitize=address,undefined -fno-sanitize-recover=undefined`, 0 failed, sanitizers silent |
| T-08 | R9 | **PASS** | `arena::tests::panic_is_caught_and_poisons_the_handle` and `tests::panics_in_load_and_parse_become_status_codes` (forced panic in `capy_library_new` and `capy_parse` → `CAPY_E_PANIC`, no unwind, next call unaffected) |
| T-09 | R11 | **PASS** | `rust/ffi/check_header.sh`: `capy.h is up to date` (75 functions) |
| T-10 | R12, R15 | **PASS** | `ast_json_compat.sh`: **133 scripts, 0 differences** against the `69fb4ac` binary; 3 CLI tests (`--with-expr` adds the member and only that; refused without `--json`) |
| T-11 | R9, `ARCH-003` | **PASS** | 4 new `recursion_guard` tests (including one that pins the boundary: 63 pairs of parentheses parse, the 64th is refused); 6 nesting shapes × 200 / 5 000 / 200 000 levels on a 2 MiB stack, parse and run |
| T-12 | R1, R15, R17 | **PARTIAL** | Cross-build of the static library for `aarch64-unknown-linux-gnu`: builds (9,374,018 B); not linked or run. Wasm size: **not equal** (M-04 FAIL) |
| T-13 | `GATE-001` | **PASS** | build exit 0; clippy clean (workspace and `--profile ffi`); `cargo test --workspace` **174 passed, 0 failed** (baseline 153); `mkdocs build --strict` exit 0 |
| T-14 | `GATE-002` | **PARTIAL** | `capy check`: 122 of 123 libraries ok — the one rejection is `left-recursion-rejected`, a sample that exists to demonstrate a rejection; golden: ok; `wasm_check.sh`: **FAIL**, one case, **identical on the baseline** (`INC-2026-0003`) |

Test counts: workspace 153 → 174 (+21): `capy-ffi` 5 unit + 4 differential + 4 expression + 1 thread, 3 CLI, 4 recursion.

### Mutation checks (QUAL-003 — each test must be able to fail)

| Mutation | Caught by | Outcome |
|---|---|---|
| M1 capture enumeration reversed | initially **not caught** (the walker collected into a `BTreeMap`, hiding order); after adding an ascending-order assertion: `capture names out of order` | caught after the test was strengthened |
| M2 foreign-tag check removed | `id_zero_foreign_tag_and_out_of_range_are_rejected` | caught |
| M3 statement accessor accepts a block | `kind_mismatch_is_rejected` | caught |
| M3b statement accessor accepts a capture | initially **not caught** (no capture id was passed to a statement accessor); after adding wrong-kind checks: `misuse.c` FAIL | caught after the test was strengthened |
| M4 `expr` flag ignored in the writer | `differential` (2 failures) | caught |
| M5 `Binary` children swapped | `expr` tests (2 failures) | caught |
| M6 `catch_unwind` removed from `capy_parse` | `panics_in_load_and_parse_become_status_codes` aborts | caught |
| M7 expression depth limit removed | `recursion_guard` aborts with a stack overflow | caught |
| M8 `publish = true` on `capy-ffi` | `check_publishable.py` exit 1 | caught |

### Measurements

Baselines were taken on the clean tree at `69fb4ac` before implementation (wasm, test count) or are stated in
the plan (archive size, from the consumer's measurement of `capy-core`).

| ID | Metric | Baseline | Threshold (frozen) | Prediction (frozen) | Actual | Result |
|---|---|---|---|---|---|---|
| M-01 | static archive size | 5,154,280 B (`capy-core` alone, no exports) | ≤ 5.5 MB | 5.2 – 5.4 MB | **31,225,392 B** (`--profile ffi`) | **FAIL** |
| M-02 | arena overhead over `Library::parse` | `Library::parse` over 132 scripts × 20 passes, 15 interleaved samples | ≤ +25 % | +8 % … +18 % | median **+7.1 / +7.8 / +8.1 %** (and +8.0 % in an earlier session) | **PASS** (one session just below the predicted range) |
| M-03 | `capy_node_func` per call | none | recorded only | 2 – 6 ns | **4.63 – 4.68 ns** (best of 5, 10^7 calls, four sessions) | recorded; inside prediction |
| M-04 | wasm module size | **1,434,097 B** (`69fb4ac`, rebuilt from a clean worktree, reproducible) | exactly equal | 0 bytes | **1,435,313 B (+1,216)** | **FAIL** |
| M-05 | direct dependencies of `capy-core` | 1 | exactly 1 | 1 | 1 (`regex`) | **PASS** |

#### M-01 — the prediction was wrong, and why

The prediction extrapolated from a static library of `capy-core` with **no exports**, which LTO reduces to almost
nothing. A real C ABI keeps everything reachable from its exports, and the archive also carries the Rust standard
library's objects.

| Variant | Size |
|---|---|
| `libcapy_ffi.a`, macOS arm64, `--profile ffi` (unwind) | 31,225,392 B |
| same with `panic = "abort"` (information only) | 29,817,904 B |
| `libcapy_ffi.a`, `aarch64-unknown-linux-gnu`, cross-built | 9,374,018 B |
| `libcapy_ffi.dylib` | 806,608 B |
| `walk` linked with `clang -O2 -Wl,-dead_strip` | 1,319,016 B (1,081,856 B after `strip -x`) |
| `walk` linked with `clang -O2` (no dead-strip) | 4,413,992 B |
| `walk` linked with `zig cc` defaults (no `-O`, no dead-strip) | 21,616,040 B |
| `walk` linked with `zig cc -O2 -Wl,-dead_strip` (what `run.sh` does) | 1,332,872 B |

Why the macOS and Linux archives differ by 3x was **not investigated**. The claim "an archive of at most 5.5 MB" is
**not supported**; the user documentation states the measured sizes and the dead-strip flag instead.

#### M-02 — samples (ms for 20 passes over 132 scripts; 15 interleaved samples per session)

| Session | `Library::parse` min / median / max | `capy_parse` + free min / median / max | Median overhead | Min-to-min |
|---|---|---|---|---|
| earlier | 59.70 / 61.24 / 73.85 (spread 23.1 %) | 63.76 / 66.12 / 86.56 (34.5 %) | +8.0 % | +6.8 % |
| 1 | 59.45 / 60.63 / 61.46 (3.3 %) | 64.09 / 65.38 / 151.50 (133.7 %) | +7.8 % | +7.8 % |
| 2 | 59.13 / 59.98 / 80.83 (36.2 %) | 63.89 / 64.87 / 90.62 (41.2 %) | +8.1 % | +8.0 % |
| 3 | 60.08 / 62.93 / 68.16 (12.8 %) | 63.99 / 67.39 / 77.75 (20.4 %) | +7.1 % | +6.5 % |

The host is noisy (spread up to 134 % from single outliers), but the overhead is stable at 6.5 – 8.1 % on both median
and minimum, against a threshold of 25 %. Limitation: the corpus is the shipped samples — small scripts; a very
large file was not measured.

#### M-04 — attribution

The +1,216 bytes were isolated by experiment: with the three `DepthGuard::enter()` calls removed from
`value_parser.rs`, the wasm module is **exactly 1,434,097 bytes**. The `expr` writer in `ast_json.rs` costs nothing
in wasm (dead code); `capy-ffi` is not part of the wasm build. The growth is the price of `INC-2026-0002`.

### Raw evidence (abridged outputs)

```text
$ cargo test --workspace            -> passed 174 failed 0 ignored 0
$ examples/c/run.sh all             -> walk: PASS (greet(who=world)) | hostile: 30 checks, 0 failed | misuse: 76 checks, 0 failed
$ rust/ffi/check_header.sh          -> capy.h is up to date
$ rust/devtools/ast_json_compat.sh  -> compared 133 scripts, 0 differences
$ differential (--nocapture)        -> differential: 132 scripts (10 with diagnostics, 110 with expressions)
$ python3 devtools/check_publishable.py -> PUBLISHABILITY PASS — 3 crate(s) publish-ready
$ wasm_check.sh (baseline worktree) -> 1 ✗: parse-recovery/broken.capy: error mismatch   (same ✗ list on the new tree)
```

## Result

**PARTIAL.** 12 of 14 checklist rows PASS; T-12 and T-14 are PARTIAL; M-01 and M-04 FAIL their frozen thresholds.
Neither failure affects correctness: M-01 is a size claim that was wrong, M-04 is the cost of a defect fix.

## Evidence

Commands above, output reproduced in this document, plus the unit and integration tests committed in
`rust/ffi/tests/`, `rust/ffi/src/`, `rust/cli/tests/ast_with_expr.rs`, `rust/tests/recursion_guard.rs`, `examples/c/`.

## Evidence Sources

`rust/ffi/examples/bench.rs` (M-02, M-03), `rust/devtools/ast_json_compat.sh` (T-10), `examples/c/run.sh`, the commands in *Procedure*.

## Executed By

Claude (Sonnet 5.5) for Olivier.

## Executed At

2026-10-13.

## Defects Raised

`INC-2026-0002` (deep expression nesting; fixed), `INC-2026-0003` (`wasm_check.sh` mismatch; pre-existing, open).

## Related Documents

- `PLAN-2026-0005`, `PROP-2026-0008`, `ADR-0005`, `RPT-2026-0005`, `INC-2026-0002`, `INC-2026-0003`, `TEST-2026-0013`, `TEST-2026-0014`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial record |
