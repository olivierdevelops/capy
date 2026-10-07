---
document_id: TEST-2026-0012
title: Test — Nesting Bound, Browser alts and Build Identity
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

scope: Records the functional, regression, documentation and gate tests T-01 to T-08 of PLAN-2026-0004 that prove requirements R1 to R8 and use cases UC-01 to UC-04 of PROP-2026-0005 (name the nesting bound with E0003, expose alts to the browser JSON, identify the build in capy version).

reason: DOCUMENTATION.md section 27 requires every plan requirement to link to an executed test, and QUAL-003 requires evidence that each new test can fail.

related_documents:
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - TEST-2026-0009
  - TEST-2026-0013
  - RPT-2026-0004
  - TRBL-2026-0001

supersedes: null
superseded_by: null

tags:
  - diagnostics
  - nesting-bound
  - wasm
  - version
  - mutation-check

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Test — Nesting Bound, Browser alts and Build Identity

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Purpose

Prove that (1) a statement that fails because the nesting bound was reached now says
so and carries `E0003`, without the bound moving and without leaking into the next
statement; (2) the browser introspection JSON carries `alts`; (3) `capy version`
identifies the build; and (4) nothing that already worked changed.

```text
   WHAT IS UNDER TEST
   ┌──────────────────────────────────────────────────────────────────────────┐
   │ C-01  make_parser.rs      depth bound hit ─► depth_err remembered        │
   │                           parse_stmt: all candidates failed              │
   │                              └─► depth_err? ─► E0003 + named bound       │
   │                                    └─ no ───► furthest failure (E0001)   │
   │ C-02  wasm/src/lib.rs     capy_introspect  "type":"call","alts":[...]    │
   │ C-03  cli/src/main.rs     VERSION = CAPY_VERSION  or  CARGO_PKG_VERSION  │
   │ C-04  docs/*, CHANGELOG   say all of the above, truthfully               │
   └──────────────────────────────────────────────────────────────────────────┘
```

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0004 R1 | A statement that fails at the nesting bound reports `nesting too deep (limit 64)` and the alternatives joined with ` \| ` |
| PLAN-2026-0004 R2 | `Library::parse` reports it with `E0003`; every other failure keeps `E0001` |
| PLAN-2026-0004 R3 | The bound error does not leak into a later statement (`[E0003, E0001]`) |
| PLAN-2026-0004 R4 | The bound did not move: 31 call levels parse, 32 are refused |
| PLAN-2026-0004 R5 | `capy_introspect` args carry `alts` (empty for a plain capture); `type` stays alternative 1 |
| PLAN-2026-0004 R6 | `capy version` and `capy --version` print `capy <CARGO_PKG_VERSION>` when unstamped |
| PLAN-2026-0004 R7 | No existing library changes behaviour (goldens unchanged) |
| PLAN-2026-0004 R8 | Documentation states the new behaviour; `mkdocs build --strict` exits 0 |
| PLAN-2026-0004 UC-01 to UC-04 | See the matrix; use case to requirement mapping is in `RPT-2026-0004` |

## Preconditions

- Tree at `cb23972` (`chore(release): 0.24.0`), clean.
- Built with `cd rust && cargo build --workspace`. A plain `cargo build` in `rust/`
  does **not** rebuild the CLI (`TRBL-2026-0001`), so it was not used. The binary under
  test is `rust/target/debug/capy`; it prints `capy 0.24.0`.
- Library under test: `CALL_LIB` in `rust/tests/alternation.rs` (`ret`, `call`,
  `operand` = `arg capture v call | name | num`, `num`, `name`), also shipped as
  `samples/expression-grammar/lib.capy`.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable, debug test profile for functional tests,
release profile for the wasm size check (`TEST-2026-0013`).

## Test Data

| Name | Content |
|---|---|
| `CALL_LIB` | recursive call grammar; `operand` is the three-way choice |
| 70-, 40-, 32- and 31-deep sources | `return f(f(...f(1)...))` with that many `f(` levels |
| `CHOICE_LIB` (wasm tests) | `call \| name \| num` choice library, inline in `rust/wasm/src/lib.rs` |
| `samples/expression-grammar/` | `lib.capy`, `script.capy`, `broken.capy` + four goldens (unchanged) |

## Procedure

```sh
cd rust
cargo build --workspace
cargo test --test alternation
cargo test -p capy-wasm-abi
cargo test -p capy-cli --test version
cargo test --test golden -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd .. && mkdocs build --strict
```

```text
   RUN ORDER                                         TEST ROWS
   build --workspace ─► alternation ─► wasm-abi ─►   T-01 T-02 T-03 │ T-04
                        version ─► golden ─► docs    T-05 │ T-06 │ T-07
                        ─► clippy ─► workspace       T-08
```

## Test Matrix

Legend: `T-nn` is the plan test id; `fn` is the test function.

| ID | What | Why | Procedure | Expected | Actual | Result | Requirement validated | Files |
|---|---|---|---|---|---|---|---|---|
| T-01 | `seventy_deep_nest_hits_the_existing_bound` — 70 nested `f(` through `run` | The bound's message must reach the reader instead of a generic `expected` error | `lib.run("return f(f(...1...))")` with 70 levels | `Err` whose text contains `nesting too deep` and `call \| name \| num` | as expected; through the CLI `capy run` printed `error: nesting too deep (limit 64) while matching "call \| name \| num" — the source nests further than the parser will follow`, exit 1 | PASS | PLAN-2026-0004 R1 | `rust/tests/alternation.rs`, `rust/src/orchestrator/features/make_parser.rs` |
| T-01 | `thirty_one_levels_still_parse` — 31 nested `f(` through `run` | The bound itself must not move | `lib.run` with 31 levels | `Ok` | as expected; through the CLI `capy run` exit 0 for 31 levels, exit 1 for 32 levels | PASS | PLAN-2026-0004 R4 | `rust/tests/alternation.rs` |
| T-02 | `depth_bound_is_reported_as_e0003` — 40 nested `f(` through `parse` | A tool must tell "too deep" from "unknown function" | `lib.parse(src)`; read `diagnostics` | not clean; exactly one diagnostic; code `E0003`; message contains `nesting too deep` | as expected; `capy ast` on a 32-deep source printed `error[E0003] 1:1: nesting too deep (limit 64) while matching "call \| name \| num" — the source nests further than the parser will follow`, exit 1 | PASS | PLAN-2026-0004 R2 | `rust/tests/alternation.rs`, `rust/src/orchestrator/features/make_parser.rs` |
| T-03 | `depth_error_does_not_leak_into_the_next_statement` — `return add(1, 2)`, a 40-deep statement, `return add(3, +)` | A stale `depth_err` would colour the third statement | `lib.parse(src)`; collect diagnostic codes | `["E0003", "E0001"]` | as expected | PASS | PLAN-2026-0004 R3 | `rust/tests/alternation.rs` |
| T-04 | `introspect_json_carries_alts` — `capy_introspect` on `CHOICE_LIB` | A browser tool must show every alternative | call `capy_introspect`, read the JSON back | contains `"type":"call","alts":["name","num"]` | as expected | PASS | PLAN-2026-0004 R5 | `rust/wasm/src/lib.rs` |
| T-04 | `introspect_json_alts_is_empty_for_a_plain_capture` — `function say / arg capture w ident` | A consumer must be able to read `alts` unconditionally | same, plain library | contains `"type":"ident","alts":[]` | as expected | PASS | PLAN-2026-0004 R5 | `rust/wasm/src/lib.rs` |
| T-05 | `version_reports_the_crate_version_when_not_stamped` | A stale binary must be distinguishable from a current one (`TRBL-2026-0001`) | run the built `capy version` | `capy 0.24.0` (`CARGO_PKG_VERSION`), not `capy dev`; the test returns early if the build was stamped with `CAPY_VERSION` | as expected; `rust/target/debug/capy version` printed `capy 0.24.0` | PASS | PLAN-2026-0004 R6 | `rust/cli/tests/version.rs`, `rust/cli/src/main.rs` |
| T-05 | `dash_dash_version_agrees_with_version` | The two spellings must not drift apart | run `capy --version` and `capy version` | identical output | as expected; both printed `capy 0.24.0` | PASS | PLAN-2026-0004 R6 | `rust/cli/tests/version.rs` |
| T-06 | `cargo test --test golden -- --nocapture` | `GOAL-002`: nothing that worked may change | run the golden harness | `goldens: 131 passed, 8 skipped (no golden file), 0 failed` (unchanged: no golden was added or edited by this release) | `goldens: 131 passed, 8 skipped (no golden file), 0 failed` | PASS | PLAN-2026-0004 R7 | `rust/tests/golden.rs`, `samples/**` |
| T-07 | Every command quoted in the changed pages, then `mkdocs build --strict` | `PHIL-001`: docs must be true | re-ran the commands behind the quoted messages (see *Documentation check*) and `mkdocs build --strict` | quoted text matches real output; exit 0 | matches; `mkdocs build --strict` exit 0 | PASS | PLAN-2026-0004 R8 | `docs/diagnostics.md`, `docs/errors-and-debugging.md`, `docs/library-authoring.md`, `docs/cli.md`, `docs/embedding.md`, `docs/roadmap.md`, `docs/whats-new.md` |
| T-08 | Full gate | `GATE-001` | `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` | clippy clean; all tests pass | clippy finished with no warnings; `cargo test --workspace`: 135 passed, 0 failed | PASS | PLAN-2026-0004 GATE-001 | whole tree |

```text
   RESULT COUNT FOR THIS MATRIX
   ┌───────────────┬───────┐
   │ PASS          │  11   │   11 rows (T-01..T-08, several with 2 functions)
   │ PARTIAL       │   0   │
   │ FAIL          │   0   │
   │ NOT APPLICABLE│   0   │
   └───────────────┴───────┘
```

### Mapping of the new test functions

```text
   file                              function                                      plan id  req
   ────────────────────────────────  ────────────────────────────────────────────  ───────  ──────
   rust/tests/alternation.rs         seventy_deep_nest_hits_the_existing_bound     T-01     R1
   rust/tests/alternation.rs         thirty_one_levels_still_parse                 T-01     R4
   rust/tests/alternation.rs         depth_bound_is_reported_as_e0003              T-02     R2
   rust/tests/alternation.rs         depth_error_does_not_leak_into_the_next_...   T-03     R3
   rust/wasm/src/lib.rs (cfg test)   introspect_json_carries_alts                  T-04     R5
   rust/wasm/src/lib.rs (cfg test)   introspect_json_alts_is_empty_for_a_plain_... T-04     R5
   rust/cli/tests/version.rs         version_reports_the_crate_version_when_not_.. T-05     R6
   rust/cli/tests/version.rs         dash_dash_version_agrees_with_version         T-05     R6
```

`rust/tests/alternation.rs` holds 21 functions in total (it held 18 at 0.23.0: the
old `seventy_deep_nest_hits_the_existing_bound` was rewritten, and three were added).

## Expected Results

Every function above passes; the golden line is unchanged; the gate is green; each
mutation below turns the named tests red.

## Actual Results

Re-run on 2026-10-07 for this document, on the binary built by
`cargo build --workspace`:

```text
$ cd rust && cargo test --test alternation
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cd rust && cargo test -p capy-wasm-abi
test tests::introspect_json_alts_is_empty_for_a_plain_capture ... ok
test tests::introspect_json_carries_alts ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cd rust && cargo test -p capy-cli --test version
test version_reports_the_crate_version_when_not_stamped ... ok
test dash_dash_version_agrees_with_version ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

$ cd rust && cargo test --test golden -- --nocapture
goldens: 131 passed, 8 skipped (no golden file), 0 failed

$ cd rust && cargo test --workspace      (sum of every "test result" line)
passed 135, failed 0
```

### CLI observations (through the built binary)

```text
$ rust/target/debug/capy version
capy 0.24.0
$ rust/target/debug/capy --version
capy 0.24.0

$ cd samples/expression-grammar
$ capy run lib.capy <70-deep source>          exit 1
error: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow

$ capy ast lib.capy <32-deep source>          exit 1
<error> 1:1-1:105  99 token(s) skipped
error[E0003] 1:1: nesting too deep (limit 64) while matching "call | name | num" — ...

$ capy run lib.capy <31-deep source>          exit 0
$ capy run lib.capy <32-deep source>          exit 1

$ capy ast lib.capy broken.capy               (ordinary failure keeps E0001)
error[E0001] 1:1: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
```

```text
   THE BOUND, SEEN FROM THE CLI                 code     exit
   ───────────────────────────────────────────  ───────  ────
   31 call levels   ██████████████████████████  parses     0
   32 call levels   ██████████████████████████  refused    1   E0003  (limit 64 captures)
   70 call levels   ██████████████████████████  refused    1   E0003
   `+` argument     (ordinary mismatch)         refused    1   E0001  (unchanged)
```

### Documentation check (T-07)

The error texts quoted in `docs/errors-and-debugging.md` and `docs/library-authoring.md`
are byte-identical to the `capy run` output above. `docs/cli.md` and `docs/whats-new.md`
quote `capy 0.24.0`, which the built binary prints. `docs/embedding.md` describes the
`alts` field asserted by T-04. `docs/diagnostics.md` lists `E0003` as emitted since
0.24.0 and keeps `E0002` reserved; the 40- and 32-deep sources produce `E0003`, the `+`
source produces `E0001`. `mkdocs build --strict` exit code: 0.

### T-06 golden accounting

```text
   0.23.0          0.24.0
   131 passed      131 passed      (unchanged)
     8 skipped       8 skipped     (unchanged)
     0 failed        0 failed
```

No golden file was added or edited in `a046032..cb23972` (`git diff --stat` lists none),
so the line is expected to be identical, and it is.

## Mutation Checks

QUAL-003: each new test must fail when its fix is reverted. These three mutations were
run during implementation, each reverted afterwards. They were **not** re-run while
writing this document; the results below are the ones recorded when they were run.
No other mutation check is claimed.

```text
   MUTATION (a) — depth report disabled
   ────────────────────────────────────
   make_parser.rs  parse_stmt   the depth_err report disabled (the bound error no longer returned)
   cargo test --test alternation
        3 FAILED: depth_error_does_not_leak_into_the_next_statement
                  depth_bound_is_reported_as_e0003
                  seventy_deep_nest_hits_the_existing_bound

   MUTATION (b) — version fallback reverted to "dev"
   ─────────────────────────────────────────────────
   cli/src/main.rs  VERSION   None => env!("CARGO_PKG_VERSION")  ──►  None => "dev"
   cargo test -p capy-cli --test version
        FAILED: version_reports_the_crate_version_when_not_stamped

   MUTATION (c) — no alts emitted in the wasm JSON
   ───────────────────────────────────────────────
   wasm/src/lib.rs  capy_introspect   no alts emitted in the JSON
        FAILED: introspect_json_carries_alts
```

| Mutation | Tests that went red | Meaning |
|---|---|---|
| (a) depth report disabled | 3 listed above | T-01, T-02, T-03 are asserted, not assumed |
| (b) VERSION fallback `"dev"` | `version_reports_the_crate_version_when_not_stamped` | T-05 is asserted |
| (c) no `alts` in wasm JSON | `introspect_json_carries_alts` | T-04 is asserted |

Not mutation-checked: `thirty_one_levels_still_parse` (a bound-did-not-move guard that
also passes on 0.23.0, by design), `introspect_json_alts_is_empty_for_a_plain_capture`
and `dash_dash_version_agrees_with_version`.

## Result

PASS

All eleven rows are `PASS`. The `PARTIAL` carried from `TEST-2026-0009` T-03 (depth
message not shown) is resolved by T-01, T-02 and T-03 here.

## Evidence

- The three mutation runs prove T-01 to T-05 can fail; the plain-capture, 31-level and
  `--version` agreement tests were not individually mutation-checked.
- The CLI observations above were produced on 2026-10-07 with the `cargo build
  --workspace` binary, not an older `capy` on PATH (`TRBL-2026-0001`).

## Known Limitations

1. **The bound is reported only when the statement would otherwise fail generically.**
   If another failure is more specific (for example a block body error), that one wins
   (`PROP-2026-0005` C-01). This is the implemented reading of the proposal.
2. **The wasm JSON still omits `optional` and `default`** (`PROP-2026-0005` non-goal, OQ-01).
3. **`E0002` remains reserved and unemitted.**
4. **Mutation checks (a) to (c) were not re-observed for this document**; see above.

## Evidence Sources

- `cd rust && cargo build --workspace`
- `cd rust && cargo test --test alternation` (21 passed)
- `cd rust && cargo test -p capy-wasm-abi` (2 passed)
- `cd rust && cargo test -p capy-cli --test version` (2 passed)
- `cd rust && cargo test --test golden -- --nocapture`
- `cd rust && cargo clippy --workspace --all-targets -- -D warnings`
- `cd rust && cargo test --workspace` (135 passed)
- `mkdocs build --strict`
- `rust/target/debug/capy version`, `--version`, `run`, `ast` on samples/expression-grammar
- `rust/tests/alternation.rs`, `rust/wasm/src/lib.rs`, `rust/cli/tests/version.rs`

## Executed By

Capy Engine (Olivier, with an AI agent re-running the quoted commands)

## Executed At

2026-10-07

## Defects Raised

None.

## Related Documents

- PLAN-2026-0004
- PROP-2026-0005
- ADR-0004
- TEST-2026-0009 — the 0.23.0 functional tests whose T-03 was PARTIAL
- TEST-2026-0013 — measured results for 0.24.0
- RPT-2026-0004 — validation report
- TRBL-2026-0001 — stale `capy` binary

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
