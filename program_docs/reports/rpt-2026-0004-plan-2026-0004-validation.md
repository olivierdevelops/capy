---
document_id: RPT-2026-0004
title: Validation of PLAN-2026-0004 — Ordered-Choice Follow-Ups (0.24.0)
document_type: report
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

scope: Validates the implemented result of PLAN-2026-0004 against every requirement R1 to R8 and use case UC-01 to UC-04 of PROP-2026-0005 and the four predeclared measurements, and records deviations, limitations and required follow-up.

reason: DOCUMENTATION.md section 28 requires an explicit validation of the implementation against the plan before the release proceeds.

related_documents:
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - TEST-2026-0012
  - TEST-2026-0013
  - RPT-2026-0003
  - REL-0.23.0
  - TRBL-2026-0001

supersedes: null
superseded_by: null

tags:
  - validation
  - diagnostics
  - wasm
  - cli

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Validation of PLAN-2026-0004 — Ordered-Choice Follow-Ups (0.24.0)

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

The three follow-ups left by `REL-0.23.0` are implemented and evidenced: the nesting
bound is named and carries `E0003`, the browser introspection JSON carries `alts`, and
`capy version` identifies the build. **All eight requirements and all four use cases are
`PASS`; all four measurements `PASS`. No result is `PARTIAL` or `FAIL`.** The `PARTIAL`
that `RPT-2026-0003` recorded for R4 (depth message) is resolved. One process deviation
is recorded: the code was written before the proposal, ADR and plan.

```text
   RESULT BOARD
   ┌─────────────────────────────┬──────┬─────────┬──────┬────────┐
   │                             │ PASS │ PARTIAL │ FAIL │ N/A    │
   ├─────────────────────────────┼──────┼─────────┼──────┼────────┤
   │ PROP-2026-0005 R1..R8       │  8   │    0    │  0   │   0    │
   │ PROP-2026-0005 UC-01..UC-04 │  4   │    0    │  0   │   0    │
   │ Measurements M-01..M-04     │  4   │    0    │  0   │   0    │
   └─────────────────────────────┴──────┴─────────┴──────┴────────┘

   REL-0.23.0 follow-ups
     depth message (R4, PARTIAL)    ──►  RESOLVED   (R1..R4 here)
     wasm JSON omitted alts         ──►  RESOLVED   (R5)
     `capy version` printed `dev`   ──►  RESOLVED   (R6)
```

## Plan Under Validation

`PLAN-2026-0004` revision 1 (approved 2026-10-07), implementing `PROP-2026-0005`
revision 1 under `ADR-0004`.

```text
   P0 contracts ─► P1 depth bound (C-01) ─► P2 wasm alts + version (C-02, C-03)
                                                   │
        P6 release ◄─ P5 manual/system ◄─ P4 validate ◄─ P3 measure + docs
                                              ▲
                                         this report
```

## Method

Each requirement was checked against its acceptance criterion as written, by reading
the tests and the code and re-running the tests on 2026-10-07 on a binary built with
`cd rust && cargo build --workspace` (a plain `cargo build` in `rust/` does not rebuild
the CLI, `TRBL-2026-0001`).

```text
   RPT-2026-0004
        │
        ├── TEST-2026-0012  functional: alternation.rs (21), wasm-abi (2),
        │                   cli version (2), goldens, gate, three mutation checks
        └── TEST-2026-0013  measured: M-01 .. M-04
```

Code read for this report: `rust/src/orchestrator/features/make_parser.rs`
(`depth_err` set where the bound is hit; reset on `parse_stmt` entry; returned with
`fail_code = NESTING_TOO_DEEP` when every candidate failed), `rust/wasm/src/lib.rs`
(`capy_introspect` and its `#[cfg(test)]` module), `rust/cli/src/main.rs` (`VERSION`
falls back to `env!("CARGO_PKG_VERSION")`), `rust/cli/tests/version.rs`,
`rust/tests/alternation.rs`.

## Requirement Results

| Requirement | Expected | Observed | Test | Result |
|---|---|---|---|---|
| R1 | A statement failing at the bound names it and the alternatives | `capy run` on a 70-deep source printed `nesting too deep (limit 64) while matching "call \| name \| num" — the source nests further than the parser will follow`, exit 1 | TEST-2026-0012 T-01 | PASS |
| R2 | `Library::parse` code `E0003`; every other failure `E0001` | one diagnostic with code `E0003` for a 40-deep source; `capy ast` printed `error[E0003] 1:1: nesting too deep (limit 64) …`; the `+` source in `broken.capy` still gives `E0001`. Implemented as the proposal's C-01 says: reported when the statement would otherwise fail generically | TEST-2026-0012 T-02 | PASS |
| R3 | The bound does not leak into a later statement | `return add(1, 2)`, a 40-deep statement, `return add(3, +)` gives `["E0003", "E0001"]`; `depth_err` is cleared on `parse_stmt` entry | TEST-2026-0012 T-03 | PASS |
| R4 | The bound did not move: 31 parse, 32 refused | `thirty_one_levels_still_parse` passes; through the CLI, 31 levels exit 0 and 32 levels exit 1 | TEST-2026-0012 T-01 | PASS |
| R5 | `capy_introspect` args carry `alts`; empty for plain; `type` = alternative 1 | JSON contains `"type":"call","alts":["name","num"]` and, for a plain capture, `"type":"ident","alts":[]` | TEST-2026-0012 T-04 | PASS |
| R6 | `capy version` / `--version` print `capy <CARGO_PKG_VERSION>` unstamped; stamp wins | both print `capy 0.24.0` on the built binary and agree; the test returns early for a stamped build, and `VERSION` matches `CAPY_VERSION` first. The stamped path itself was **not** exercised by a test | TEST-2026-0012 T-05 | PASS |
| R7 | No existing library changes behaviour | `goldens: 131 passed, 8 skipped (no golden file), 0 failed`, same as 0.23.0; no golden file in `a046032..cb23972` | TEST-2026-0012 T-06 | PASS |
| R8 | Docs state the new behaviour; `mkdocs --strict` exit 0 | `docs/diagnostics.md` lists `E0003` as emitted since 0.24.0; `errors-and-debugging.md`, `library-authoring.md`, `embedding.md`, `cli.md`, `roadmap.md`, `language-frontend.md`, `whats-new.md` updated; quoted messages match real output; `mkdocs build --strict` exit 0 | TEST-2026-0012 T-07 | PASS |

### Use cases

| Use case | Expected | Observed | Test | Result |
|---|---|---|---|---|
| UC-01 | Learn the source is too deep, not malformed: 40/70-deep names the bound, `ast` gives `E0003`, exit 1; exactly 31 levels parse | all observed through the CLI as listed under R1, R2, R4 | TEST-2026-0012 T-01, T-02 | PASS |
| UC-02 | An error after a deep one is still reported as itself: `[E0003, E0001]` | asserted by `depth_error_does_not_leak_into_the_next_statement` | TEST-2026-0012 T-03 | PASS |
| UC-03 | A browser tool shows every alternative; plain capture gives `[]` | the two wasm unit tests assert both | TEST-2026-0012 T-04 | PASS |
| UC-04 | Tell which build is on PATH: `capy 0.24.0` for a local build | observed on the freshly built binary | TEST-2026-0012 T-05 | PASS |

Note: `PLAN-2026-0004`'s requirements table pairs R3 with UC-01 and R1 to R4 with UC-01;
the proposal's use case table (used here) pairs the leak case with UC-02. The results do
not depend on which pairing is read.

```text
   TRACEABILITY
   R1  R2  R4 ─► UC-01 ─► T-01 T-02
   R3         ─► UC-02 ─► T-03
   R5         ─► UC-03 ─► T-04
   R6         ─► UC-04 ─► T-05
   R7  R8     ─► (all) ─► T-06 T-07      GATE-001 ─► T-08
```

### Measurements

Full evidence in `TEST-2026-0013`.

| Measurement | Baseline | Observed | Threshold | Result |
|---|---|---|---|---|
| M-01 transpile time, best of 7, interleaved | retained v0.23.0 `nativebench` (best 195.896 µs) | best 195.687 µs, ratio 0.9989 | <= +10 % | PASS |
| M-02 wasm size | 1 389 633 B | 1 390 420 B (+787, +0.06 %; +3.54 % cumulative vs 0.21.0) | <= +3 000 B; ceiling 1 410 035 | PASS |
| M-03 `capy-core` direct dependencies | 1 | 1 (`regex`) | exactly 1 | PASS |
| M-04 golden suite wall time, best of 7, interleaved | retained v0.23.0 test binary (0.0637 s) | 0.0641 s, ratio 1.005 | <= 1.15x | PASS |

The shared 5 % wasm allowance has **19 615 bytes** of headroom left (`TEST-2026-0008`,
`TEST-2026-0010`, `TEST-2026-0013`).

## Deviations From the Plan

```text
   #   deviation                                         disposition
   ─   ───────────────────────────────────────────────   ──────────────────────────────
   1   code drafted before proposal / ADR / plan         recorded; thresholds still frozen
                                                          before any "after" measurement
   2   E0003 only when the statement would otherwise     implemented as the proposal's
       fail generically                                  C-01 states
   3   optional / default still absent from wasm JSON    by design (OQ-01), non-goal
```

1. **Order of work (wrong under `DOCUMENTATION.md` sections 21.3 and 24).** The three
   fixes were first written, tested and mutation-checked before `PROP-2026-0005`,
   `ADR-0004` and `PLAN-2026-0004` existed. This was caught before anything was
   committed or measured. The code was saved as a patch and removed from the tree; the
   baselines were measured on the clean tree; the proposal, ADR and plan were committed
   (`a046032`); then the patch was re-applied (`8769f51`). Thresholds were therefore
   frozen before any "after" measurement. The deviation is also in `PLAN-2026-0004`.
   What the process cannot show is whether the design would have differed had the
   documents come first; the mutation checks and tests were written alongside the code,
   not from the approved plan.
   ```text
      WRONG ORDER (what happened)            REQUIRED ORDER
      code ─► (caught) ─► patch aside        proposal ─► ADR ─► plan ─► code
        baselines on clean tree
        plan committed (a046032)
        patch re-applied (8769f51)
   ```
2. **E0003 only when the statement would otherwise fail generically.** The proposal
   says so; the code returns a held `depth_err` after a block-body error has had its
   chance. A more specific error in the same statement therefore wins.
3. **`optional` and `default` remain absent from the wasm JSON** by design
   (`PROP-2026-0005` non-goal, OQ-01). `docs/embedding.md` says so.

## Unintended Behaviour

None found. The guard against the one identified risk, a stale `depth_err` colouring
an unrelated failure, is test T-03, which was shown able to fail (mutation (a),
`TEST-2026-0012`).

## Unresolved Incidents

None. `TRBL-2026-0001` (stale binary) is why verification used `cargo build --workspace`
and why R6 exists.

## Remaining Limitations

1. **`E0002` is still reserved and unemitted.**
2. **The wasm introspection JSON omits `optional` and `default`** (deviation 3).
3. **The bound is still 64 captures** (31 call levels in `samples/expression-grammar/`);
   this release only reports it.
4. **The stamped-version path (`CAPY_VERSION` set) has no test**; the unstamped test
   returns early on a stamped build.
5. **Mutation checks (a) to (c) were run during implementation and are quoted from those
   runs; they were not re-observed for this report.**
6. **M-01 and M-04 baselines are retained binaries that are not committed**, so the
   comparisons cannot be repeated from the repository alone.
7. **`cargo fmt --check` is not enforced** (227 pre-existing diffs recorded in `RPT-2026-0003`).
8. **wasm headroom is 19 615 bytes** of the shared 5 % allowance.

## Required Follow-Up

| Item | Owner | Destination |
|---|---|---|
| `default` capture inside a repeated nonterminal fails when omitted (`PROP-2026-0004` OQ-06) | Capy Engine | a separate defect proposal |
| Add `optional` / `default` to the wasm JSON if a browser tool needs them | Capy Engine | a future plan; additive; budget against 19 615 bytes |
| Decide whether `E0002` should be emitted or retired | Capy Engine | a future proposal |
| Test the stamped `CAPY_VERSION` path | Capy Engine | next release that touches the CLI |
| Any plan touching the wasm build budgets against the remaining headroom | Capy Engine | every such plan |

## Questions Required by DOCUMENTATION.md Section 28

| Question | Answer |
|---|---|
| Was the planned functionality actually implemented? | Yes. `depth_err` and `fail_code` in the parser, `E0003` from `Library::parse`, `alts` in `capy_introspect`, the `VERSION` fallback, and the docs exist and pass. |
| Does it behave as expected? | Yes for R1 to R8 and UC-01 to UC-04, as observed through tests and the built CLI. |
| Were all expected files and components modified? | Yes; see *Files Changed*. Program documents beyond this report (`REL-0.24.0`, `DEMO-2026-0004`, `MAN-2026-0002`, `SYS-2026-0001` updates) follow validation. |
| Did implementation introduce unintended behaviour? | None found (see above). |
| Are there remaining limitations? | Yes, eight, listed above. |
| Are there unresolved incidents? | No. |
| Are follow-up changes required? | Yes, listed in *Required Follow-Up*; none blocks the release. |
| Is the `REL-0.23.0` PARTIAL (R4, depth message) resolved? | Yes: the message names the bound and `capy ast` gives `E0003`. |

## Files Changed

`git diff --stat a046032 cb23972` (21 files, 246 insertions, 55 deletions). The
program documents are not in this range; `a046032` itself contains the proposal, ADR
and plan.

```text
   ENGINE
     rust/src/orchestrator/features/make_parser.rs   | 32   depth_err, fail_code, E0003
     rust/wasm/src/lib.rs                            | 48   alts in capy_introspect + 2 tests
     rust/cli/src/main.rs                            |  8   VERSION fallback

   TESTS
     rust/tests/alternation.rs                       | 47   depth tests (3 new, 1 rewritten)
     rust/cli/tests/version.rs                       | 28   new, 2 tests

   DOCS
     CHANGELOG.md                                    | 17
     docs/cli.md                                     |  6
     docs/diagnostics.md                             | 15
     docs/embedding.md                               |  6
     docs/errors-and-debugging.md                    | 25
     docs/language-frontend.md                       |  2
     docs/library-authoring.md                       | 12
     docs/roadmap.md                                 |  9
     docs/whats-new.md                               | 22

   VERSION 0.24.0
     rust/Cargo.toml, rust/Cargo.lock (12 lines), rust/cli/Cargo.toml,
     rust/devtools/Cargo.toml, rust/mcp/Cargo.toml, rust/playground/Cargo.toml,
     rust/wasm/Cargo.toml                            | 1-2 each
```

## Conclusion

`PLAN-2026-0004` is **validated with no `PARTIAL` or `FAIL`.** The depth bound is now
named and coded, without moving and without leaking between statements; the browser
JSON carries `alts`; the build identifies itself. The three behavioural tests were
shown able to fail by mutation. Measurements are well inside their thresholds, and the
wasm allowance has 19 615 bytes left. The honest caveats are the order-of-work
deviation, the unexercised stamped-version path, and that the mutation results and the
M-01 / M-04 baselines are quoted from earlier runs rather than repeated here.

Release readiness: nothing found here blocks `REL-0.24.0`.

## Related Documents

- PLAN-2026-0004
- PROP-2026-0005
- ADR-0004
- TEST-2026-0012 — functional tests
- TEST-2026-0013 — measured results for 0.24.0
- RPT-2026-0003 — predecessor validation (R4 PARTIAL now resolved)
- REL-0.23.0 — source of the three follow-ups
- TRBL-2026-0001 — stale `capy` binary

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial validation |
