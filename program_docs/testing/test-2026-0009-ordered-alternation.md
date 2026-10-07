---
document_id: TEST-2026-0009
title: Test — Ordered Alternation in a Capture Type
document_type: test
status: completed

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 2

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - samples
  - docs

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the functional, regression and mutation tests that prove PLAN-2026-0003 requirements R1 to R9 and use cases UC-01 to UC-06 of PROP-2026-0004 (ordered alternation in a capture type).

reason: DOCUMENTATION.md section 27 requires every plan requirement to link to an executed test, and QUAL-003 requires evidence that each test can fail.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - ADR-0003
  - TEST-2026-0010
  - TEST-2026-0011
  - RPT-2026-0003

supersedes: null
superseded_by: null

tags:
  - alternation
  - parser
  - grammar
  - mutation-check

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Test — Ordered Alternation in a Capture Type

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, samples, docs

## Purpose

Prove that `arg capture v call | name | num` loads, matches left to right with
first-match-wins, nests to the existing depth bound, is refused when left
recursive through **any** alternative, reports the union of alternatives on
failure, names the matched alternative in the tree, and changes nothing that
already worked.

```text
   WHAT IS UNDER TEST
   ┌───────────────────────────────────────────────────────────────────────┐
   │  library file            engine                       observable      │
   │  ─────────────           ──────────────────────       ──────────────  │
   │  arg capture v           parse  `A | B | C`           introspect()    │
   │      call | name | num ─►load   validate each alt  ─► docs table     │
   │                          guard  walk EVERY edge       load error      │
   │                          match  try A, then B, then C tree sub[].func │
   │                          fail   union of A, B, C      diagnostic      │
   └───────────────────────────────────────────────────────────────────────┘
```

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0003 R1 | A capture type may list several library functions separated by `\|`; an unknown name in any position is a load error naming it |
| PLAN-2026-0003 R2 | Alternatives are tried left to right; first match wins; order is observable in a test that fails if order is ignored |
| PLAN-2026-0003 R3 | An alternation capture carries `*` / `+` with `sep` / `join` |
| PLAN-2026-0003 R4 | Alternation composes with recursion to the existing depth bound |
| PLAN-2026-0003 R5 | The left-recursion guard walks every alternative edge |
| PLAN-2026-0003 R6 | A failed alternation reports the union of the alternatives |
| PLAN-2026-0003 R7 | No existing library changes behaviour (golden corpus byte-identical) |
| PLAN-2026-0003 R8 | `sub[].func` names the matched alternative; `schema_version` stays 1 |
| PLAN-2026-0003 R9 | Introspection and `capy docs` print the union; `ArgInfo.alts` is populated |
| PLAN-2026-0003 UC-06 | Mixed marked and plain parameters (`mut_param \| plain_param`) |

## Preconditions

- Working tree of 2026-10-07 containing the 0.23.0 engine change (seven engine
  files, see `RPT-2026-0003` file list) and `rust/tests/alternation.rs`.
- `cd rust && cargo build --workspace` succeeded; `rust/target/debug/capy` exists.
- Library under test: `CALL_LIB` in `rust/tests/alternation.rs` — `ret`, `call`,
  `operand` (`arg capture v call | name | num`), `num`, `name`.

## Test Environment

macOS arm64 (Darwin 25.4.0), `rustc 1.90.0 (1159e78c4 2025-09-14)`, debug test
profile for functional tests.

## Test Data

| Name | Content |
|---|---|
| `CALL_LIB` | recursive call grammar, `operand` is the alternation |
| `PARAM_LIB` | `ps mut_param \| plain_param* sep "," join ", "` |
| `samples/expression-grammar/` | `lib.capy`, `script.capy`, `broken.capy` + four goldens |
| `samples/mixed-parameters/` | `lib.capy`, `script.capy` + two goldens |

## Procedure

```sh
cd rust
cargo test --test alternation
cargo test --test golden -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd .. && mkdocs build --strict
```

Mutation checks (QUAL-003) were run once, then reverted: see *Mutation Checks*.

## Test Matrix

Legend: `T-nn` is the plan test id; `fn` is the test function in
`rust/tests/alternation.rs` (18 functions in total).

| ID | What | Why | Procedure | Expected | Actual | Result | Requirement validated | Files |
|---|---|---|---|---|---|---|---|---|
| T-01 | `first_matching_alternative_wins` — two alternatives both match `use x`; swap their order | Determinism (R2): the order must be observable, otherwise the test cannot fail | Load `stmt` with `first_id \| second_id` and with `second_id \| first_id`; run `use x`; read `sub[0].func` | `use first:x` then `use second:x`; `sub[0].func == "first_id"` | as expected | PASS | PLAN-2026-0003 R2 | `rust/tests/alternation.rs` |
| T-01 | `failed_alternative_is_rewound` — `call` consumes `add` then fails on missing `(`, `name` must still see it | A failed alternative that leaks consumed tokens corrupts the next one | `run "return add(x, 7)"` | `return add(x, 7);` | as expected | PASS | PLAN-2026-0003 R2 | `rust/tests/alternation.rs` |
| T-01a | `alternation_loads_and_is_introspectable` | Loading is the precondition of everything else | `Library::new(CALL_LIB)`; `introspect()`; read `operand` capture | `type_ == "call"` (alternative 1, ARCH-001), `alts == ["name","num"]` | as expected | PASS | PLAN-2026-0003 R1 | `rust/tests/alternation.rs`, `rust/src/capy.rs` |
| T-01b | `glued_pipe_is_the_same_choice` — `call\|name\|num` | The type position is whitespace-split; glued form must not silently mean one odd name | Replace spaced with glued and load | same `alts` | as expected | PASS | PLAN-2026-0003 R1 | `rust/tests/alternation.rs`, `rust/src/infra/capy_lib_parser.rs` |
| T-01c | `unknown_alternative_is_a_load_error_naming_it` — `nope` in first, middle, last position | A typo must fail at load, not at first use | Three sources, one bad name each | each `Err` contains `nope` | as expected | PASS | PLAN-2026-0003 R1 | `rust/tests/alternation.rs`, `rust/src/orchestrator/features/make_library_loader.rs` |
| T-01d | `builtin_type_is_not_an_alternative` — `call \| int` | Records the OQ-08 / ADR-0003 narrowing | Load | `Err` contains `int` and `not a library function` | as expected | PASS | PLAN-2026-0003 R1 | `rust/tests/alternation.rs` |
| T-01e | `malformed_alternation_is_rejected` — `call \|`, `\| call`, `call \| \| num`, `call* \| num` | A dangling bar must not be a silent no-op | Four sources | each fails to load | as expected | PASS | PLAN-2026-0003 R1 | `rust/tests/alternation.rs`, `rust/src/infra/capy_lib_parser.rs` |
| T-02 | `nested_calls_parse` — `return add(3, mul(4, 5))` (UC-01) | The motivating case; fails on 0.22.0 | `run` | `return add(3, mul(4, 5));` | as expected | PASS | PLAN-2026-0003 R1, R3, R4 | `rust/tests/alternation.rs` |
| T-02 | `five_deep_nest_parses` — `f(g(h(i(j(1)))), k(2, 3))` (UC-02) | Depth plus repetition with `sep` / `join` | `run` | identical text with `;` | as expected | PASS | PLAN-2026-0003 R3, R4 | `rust/tests/alternation.rs` |
| T-02 | Sample goldens `samples/expression-grammar/` | The documented, user-visible form of the same grammar | `cargo test --test golden` (see T-06) | `add(3, mul(4, 5))` and a 5-deep nest match the goldens | goldens pass | PASS | PLAN-2026-0003 R1, R3, R4, R10 | `samples/expression-grammar/*` |
| T-03 | `seventy_deep_nest_hits_the_existing_bound` | Depth bound (64) must hold with alternation | 70 nested `f(` calls through `run` | an `Err` with a non-empty message, never a panic or stack overflow | `Err` returned; generic expectation message, not the depth message (see Known Limitations) | PARTIAL | PLAN-2026-0003 R4 | `rust/tests/alternation.rs` |
| T-04 | `left_recursive_first_alternative_is_rejected` — `expr \| atom` | `ARCH-003` | Load | `Err` contains `left recursion` | as expected | PASS | PLAN-2026-0003 R5 | `rust/tests/alternation.rs`, `rust/src/orchestrator/features/make_library_loader.rs` |
| T-04 | `left_recursion_through_second_alternative_is_rejected` — `atom \| expr` | RK-03: a guard that walks only the first edge misses this | Load | `Err` contains `left recursion` and `expr` | as expected; **fails under mutation (a)** | PASS | PLAN-2026-0003 R5 | `rust/tests/alternation.rs` |
| T-04 | `right_recursive_alternation_loads` | The guard must not over-reject: `call` consumes `name` and `(` first | Load `CALL_LIB` | loads | as expected | PASS | PLAN-2026-0003 R5 | `rust/tests/alternation.rs` |
| T-05 | `failure_reports_the_union_of_alternatives` | R6: the author must see every shape that would have been accepted | `parse "return add(3, +)"` | not clean; message contains `` `call` ``, `` `name` ``, `` `num` `` | as expected | PASS | PLAN-2026-0003 R6 | `rust/tests/alternation.rs` |
| T-06 | `cargo test --test golden -- --nocapture` | `GOAL-002`: nothing that worked may change | Run the golden harness | `goldens: 131 passed, 8 skipped (no golden file), 0 failed`; baseline 125 / 8 / 0 | `goldens: 131 passed, 8 skipped (no golden file), 0 failed` | PASS | PLAN-2026-0003 R7 | `rust/tests/golden.rs`, `samples/**` |
| T-07 | `discriminator_names_the_matched_alternative` | API (R8): consumers tell shapes apart by `sub[].func` with no schema change | `parse "return add(3, mul(4, x))"`; walk the tree; render JSON | funcs `ret, call, operand, num, operand, call, operand, num, operand, name`; JSON has `"schema_version": 1` and `"func": "call"` | as expected; **fails under mutation (b)** | PASS | PLAN-2026-0003 R8 | `rust/tests/alternation.rs`, `rust/src/domain/ast_json.rs` |
| T-08 | Every command quoted in the changed pages | `PHIL-001`: docs must be true | see `TEST-2026-0011` (R5 transcripts) and the `capy docs` row of T-12 | byte-identical apart from annotations | verified, see `TEST-2026-0011` | PASS | PLAN-2026-0003 R9, R10, R11 | `docs/library-authoring.md`, `docs/library-keywords.md` |
| T-09 | Full gate | `GATE-001` | `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `mkdocs build --strict` | all exit 0 | clippy clean; workspace tests all green; `mkdocs` exit 0 | PASS | PLAN-2026-0003 GATE-001 | whole tree |
| T-10 | `mixed_parameter_markers_parse` (UC-06) | P-05: marked and plain parameters in one list was rejected on 0.22.0 | `def f(mut c: Counter, n: int)`, `def g(n: int, mut c: Counter)`; read `ps` sub funcs | `f(&c: Counter, n: int)`, `g(n: int, &c: Counter)`; funcs `["mut_param","plain_param"]` | as expected | PASS | PLAN-2026-0003 UC-06, R1, R2, R3 | `rust/tests/alternation.rs`, `samples/mixed-parameters/*` |
| T-11 | `pipe_in_type_position_does_not_collide_with_pipe_arrow` | OQ-01: `\|` is the choice token only in a capture type; `"\|>"` in source is unaffected | Library with `arg literal "\|>"` and `arg capture v left \| right`; run `x \|> R y` | `x => r:y` | as expected | PASS | PLAN-2026-0003 R1 (OQ-01) | `rust/tests/alternation.rs` |
| T-12 | `docs_print_the_union` | R9: introspection must not print only the first alternative | `render_library_docs(CALL_LIB)` | contains `` `call \| name \| num` `` | as expected; `capy docs samples/expression-grammar/lib.capy` shows `` `call \| name \| num` `` in the Type column | PASS | PLAN-2026-0003 R9 | `rust/tests/alternation.rs`, `rust/src/domain/docs.rs` |

```text
   RESULT COUNT FOR THIS MATRIX
   ┌───────────────┬───────┐
   │ PASS          │  21   │   (rows above, one per test or procedure; 22 rows)
   │ PARTIAL       │   1   │   T-03: error refused, depth message not shown
   │ FAIL          │   0   │
   │ NOT APPLICABLE│   0   │
   └───────────────┴───────┘
```

### Mapping of the 18 test functions

```text
   rust/tests/alternation.rs            plan id   requirement
   ──────────────────────────────────   ───────   ───────────────────────
    1 alternation_loads_and_is_...      T-01a/12  R1, R9
    2 glued_pipe_is_the_same_choice     T-01b     R1
    3 unknown_alternative_is_a_...      T-01c     R1
    4 builtin_type_is_not_an_...        T-01d     R1 (OQ-08)
    5 malformed_alternation_is_...      T-01e     R1
    6 left_recursive_first_...          T-04      R5
    7 left_recursion_through_second_... T-04      R5
    8 right_recursive_alternation_...   T-04      R5
    9 first_matching_alternative_wins   T-01      R2
   10 failed_alternative_is_rewound     T-01      R2
   11 nested_calls_parse                T-02      R1, R3, R4
   12 five_deep_nest_parses             T-02      R3, R4
   13 seventy_deep_nest_hits_the_...    T-03      R4
   14 failure_reports_the_union_...     T-05      R6
   15 discriminator_names_the_...       T-07      R8
   16 mixed_parameter_markers_parse     T-10      UC-06
   17 pipe_in_type_position_does_...    T-11      R1 (OQ-01)
   18 docs_print_the_union              T-12      R9
```

## Expected Results

All 18 functions pass; golden suite shows 131 / 8 / 0 with every pre-existing
golden byte-identical; the gate is green; each mutation below turns specific
tests red.

## Actual Results

Re-run on 2026-10-07 for this document:

```text
$ cd rust && cargo test --test alternation
running 18 tests
...
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cd rust && cargo test --test golden -- --nocapture
goldens: 131 passed, 8 skipped (no golden file), 0 failed
```

### T-06 golden accounting

```text
   BEFORE (this tree minus the change)      AFTER
   ───────────────────────────────────      ──────────────────────────────
   125 passed                               131 passed        (+6)
     8 skipped (no golden file)               8 skipped       (unchanged)
     0 failed                                 0 failed

   the +6 are NEW goldens, nothing pre-existing was edited:

   samples/expression-grammar/script.expected.txt        +1
   samples/expression-grammar/script.expected-ast.txt    +1
   samples/expression-grammar/broken.expected-error.txt  +1
   samples/expression-grammar/broken.expected-ast.txt    +1
   samples/mixed-parameters/script.expected.txt          +1
   samples/mixed-parameters/script.expected-ast.txt      +1
                                                         ──
                                                         +6
```

Because the skip count is unchanged and the pass count rose by exactly the number
of new golden files, every pre-existing golden is byte-identical (R7).

### T-09 gate

| Command | Result |
|---|---|
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, no warnings |
| `cargo test --workspace` | every test binary `ok`, 0 failed |
| `mkdocs build --strict` | exit 0 |

## Mutation Checks

QUAL-003: a test that cannot fail is not a test. Two mutations of the
implementation were applied by hand, the tests were run, and each mutation was
reverted. Mutation (a) was run when `rust/tests/alternation.rs` held its first eight
functions (1 failed, 7 passed). Mutation (b) was run against all eighteen (4 failed,
14 passed).

```text
   MUTATION (a) — guard looks only at the FIRST alternative
   ────────────────────────────────────────────────────────
   make_library_loader.rs  reject_left_recursion
        edge per capture:  cap_type  +  every alt      ──►  cap_type only

   cargo test --test alternation
        1 failed, 7 passed
        FAILED: left_recursion_through_second_alternative_is_rejected
        ⇒ the test can fail; the guard really walks every edge (R5, RK-03)


   MUTATION (b) — alternatives iterated in REVERSE order
   ────────────────────────────────────────────────────────
   make_parser.rs  match_alt   (targets.iter().find_map  ──►  targets.iter().rev().find_map)
        try cap_type, alts[0], alts[1] ...             ──►  reversed

   cargo test --test alternation
        4 failed
        FAILED: first_matching_alternative_wins
                nested_calls_parse
                five_deep_nest_parses
                discriminator_names_the_matched_alternative
        ⇒ the order tests can fail; declaration order is what decides (R2)
```

| Mutation | Tests that went red | Count | Meaning |
|---|---|---:|---|
| (a) guard checks first alternative only | `left_recursion_through_second_alternative_is_rejected` | 1 failed, 7 passed | T-04 and R5 are asserted, not assumed |
| (b) reversed alternative order | `first_matching_alternative_wins`, `nested_calls_parse`, `five_deep_nest_parses`, `discriminator_names_the_matched_alternative` | 4 failed | T-01, T-02, T-07 and R2 are asserted, not assumed |

Both mutations were reverted; the 18-test run quoted above is on the reverted,
final code.

## Result

PARTIAL

The matrix has one `PARTIAL` row: **T-03**. The depth bound holds (the input is
refused with an `Err`, never a panic), but the dedicated "nesting too deep"
message is not what surfaces. It is documented under *Known Limitations* and in
`RPT-2026-0003` deviation 4. Every other row is `PASS`.

## Evidence

- The two mutation runs prove T-04 (second alternative) and T-01 / T-02 / T-07
  can fail. Tests T-01 (`failed_alternative_is_rewound`), T-05, T-10, T-11 and
  T-12 were not mutation-checked individually; they assert exact output strings,
  which a no-op implementation cannot produce.
- T-10 is the same input that is rejected on 0.22.0 (`` expected `mut`, found "n" ``,
  `PROP-2026-0004` P-05).
- Real diagnostic observed for T-05 through the CLI:

```text
$ rust/target/debug/capy run samples/expression-grammar/lib.capy samples/expression-grammar/broken.capy
error: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
  1 │ return add(3, +)
    │               ^
```

## Known Limitations

1. **Depth message is consumed by backtracking.** At the 64-capture depth bound
   the depth message is consumed by capture-local backtracking and surfaces as
   the generic `expected` error. Same behaviour as before for single-type
   captures. In `samples/expression-grammar/` an input of 31 call levels parses
   and 32 is refused. The `Err` contract of `recursion_guard.rs` holds.
2. **Alternatives must be library functions** (OQ-08, `ADR-0003`). A flat
   alternative is written as a `bare` one-capture function, as `num` and `name`
   are in `CALL_LIB`.
3. **`cargo fmt --check` is not enforced** in this repo: 227 pre-existing diffs
   at HEAD. The gate is `clippy -D warnings`, tests and `mkdocs --strict`.
4. **`default` capture inside a repeated nonterminal** fails when omitted (OQ-06).
   Not investigated, out of scope; alternation does not fix it.

## Evidence Sources

- `cd rust && cargo test --test alternation` (18 passed)
- `cd rust && cargo test --test golden -- --nocapture` (`goldens: 131 passed, 8 skipped (no golden file), 0 failed`)
- `cd rust && cargo clippy --workspace --all-targets -- -D warnings`
- `cd rust && cargo test --workspace`
- `mkdocs build --strict`
- `rust/tests/alternation.rs`, `rust/tests/golden.rs`
- `samples/expression-grammar/`, `samples/mixed-parameters/`

## Executed By

Capy Engine (Olivier, with an AI agent re-running the quoted commands)

## Executed At

2026-10-07

## Defects Raised

None in the implementation. Two observations, neither a defect of this change:

- T-03 message wording (known limitation 1), pre-existing for single-type captures.
- OQ-06 (`default` in a repeated nonterminal), pre-existing, tracked in
  `PROP-2026-0004`.

## Related Documents

- PLAN-2026-0003
- PROP-2026-0004
- ADR-0003
- TEST-2026-0010 — measured results for 0.23.0
- TEST-2026-0011 — carried docs and samples verification
- RPT-2026-0003 — validation report

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
| 2 | 2026-10-07 | Olivier | Corrections from a full read-through after the tag (documentation only; no code change) |
