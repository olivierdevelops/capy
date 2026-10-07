---
document_id: RPT-2026-0003
title: Validation of PLAN-2026-0003 — Ordered Alternation for Nonterminals (0.23.0)
document_type: report
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

scope: Validates the implemented result of PLAN-2026-0003 against every requirement R1 to R11 and use case UC-01 to UC-06 of PROP-2026-0004, the four predeclared measurements, and the carried PROP-2026-0002 requirements R1 to R19, and records deviations, limitations and required follow-up.

reason: DOCUMENTATION.md section 28 requires an explicit validation of the implementation against the plan before the release proceeds.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - PROP-2026-0002
  - ADR-0003
  - TEST-2026-0009
  - TEST-2026-0010
  - TEST-2026-0011
  - REL-0.22.0

supersedes: null
superseded_by: null

tags:
  - validation
  - alternation
  - parser

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Validation of PLAN-2026-0003 — Ordered Alternation for Nonterminals (0.23.0)

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs, samples

## Summary

Ordered alternation (`arg capture v call | name | num`) is implemented and
evidenced. Of the eleven requirements, **ten are `PASS` and one is `PARTIAL`
(R4)**; all six use cases are `PASS` except UC-02, which inherits R4's `PARTIAL`.
All four measurements pass. The carried `PROP-2026-0002` work has all nineteen
requirements `PASS` (its R4 failed on first run and was fixed). No result is `FAIL`.

```text
   RESULT BOARD
   ┌─────────────────────────────┬──────┬─────────┬──────┬────────┐
   │                             │ PASS │ PARTIAL │ FAIL │ N/A    │
   ├─────────────────────────────┼──────┼─────────┼──────┼────────┤
   │ PROP-2026-0004 R1..R11      │  10  │    1    │  0   │   0    │
   │ PROP-2026-0004 UC-01..UC-06 │   5  │    1    │  0   │   0    │
   │ Measurements M-01..M-04     │   4  │    0    │  0   │   0    │
   │ Carried PROP-2026-0002 R1-19│  19  │    0    │  0   │   0    │
   └─────────────────────────────┴──────┴─────────┴──────┴────────┘

   the one PARTIAL that matters:
     R4 (alternation)  error is refused, but its message is generic
```

## Plan Under Validation

`PLAN-2026-0003` revision 1 (approved 2026-10-07), implementing `PROP-2026-0004`
revision 4 under `ADR-0003`, and carrying `PROP-2026-0002` into the same release.

```text
   P0 contracts ─► P1 data model + guard ─► P2 matcher ─► P3 sample + tests
                                                                │
        P7 release ◄─ P6 manual/system ◄─ P5 validate ◄─ P4 measure
                                              ▲
                                         this report
```

## Method

Each requirement was checked against its acceptance criterion as written, using
the three test records below, re-executed on 2026-10-07:

```text
   RPT-2026-0003
        │
        ├── TEST-2026-0009  functional: rust/tests/alternation.rs (18 tests),
        │                   goldens 131/8/0, gate, two mutation checks
        ├── TEST-2026-0010  measured: M-01 .. M-04
        └── TEST-2026-0011  carried docs and samples: PROP-2026-0002 R1..R19
```

`cd rust && cargo test --test alternation` was re-run for this report:

```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Requirement Results

### PROP-2026-0004 requirements

| Requirement | Expected | Observed | Test | Result |
|---|---|---|---|---|
| R1 | A capture type may list several library functions separated by `\|`; unknown name in any position is a load error naming it | `call \| name \| num` loads; `alts == ["name","num"]`; glued `a\|b` equivalent; `nope` in first, middle or last position fails naming `nope`; dangling or empty `\|` rejected. **Narrowed to library functions only** (deviation 2) | TEST-2026-0009 T-01a to T-01e, T-11 | PASS |
| R2 | Left to right, first match wins, order observable | swapping `first_id \| second_id` swaps output `use first:x` / `use second:x`; failed alternative is rewound. Reversing iteration order made 4 tests fail (mutation b) | TEST-2026-0009 T-01 | PASS |
| R3 | `*` / `+` with `sep` / `join` on an alternation | `operand*` with `sep "," join ", "` parses `f(g(h(i(j(1)))), k(2, 3))` and mixed parameter lists | TEST-2026-0009 T-02, T-10 | PASS |
| R4 | Composes with recursion to arbitrary depth; depth 70 reports the existing depth error | 5-deep nest parses; 31 call levels parse and 32 are refused in `samples/expression-grammar/`; depth 70 is **refused with an `Err`, never a panic**, but the "nesting too deep" message is **not** what surfaces — it is consumed by capture-local backtracking and the generic `expected` error shows (deviation 4) | TEST-2026-0009 T-02, T-03 | PARTIAL |
| R5 | Guard detects a cycle through any alternative | `expr \| atom` and `atom \| expr` both refused with `left recursion`, naming `expr`; right recursion still loads. Guard restricted to first alternative made the second-alternative test fail (mutation a) | TEST-2026-0009 T-04 | PASS |
| R6 | Failure reports the union of alternatives | `` expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand` ``; union semantics met, wording differs from the proposal's illustration (deviation 3) | TEST-2026-0009 T-05 | PASS |
| R7 | No existing library changes behaviour | `goldens: 131 passed, 8 skipped (no golden file), 0 failed`; baseline 125 / 8 / 0; the +6 are new goldens, every pre-existing golden byte-identical | TEST-2026-0009 T-06 | PASS |
| R8 | `sub[].func` names the matched alternative; `schema_version` 1 | walk gives `ret, call, operand, num, operand, call, operand, num, operand, name`; JSON has `"schema_version": 1`; no serializer change | TEST-2026-0009 T-07 | PASS |
| R9 | Introspection and `capy docs` print the union | `ArgInfo.type_ == "call"`, `alts == ["name","num"]`; docs Type column shows `` `call \| name \| num` `` | TEST-2026-0009 T-01a, T-12 | PASS |
| R10 | Worked `bare` example in the authoring guide; sample with goldens | `docs/library-authoring.md` gains "Ordered choice" (line 578), "A flat alternative is a `bare` function" (624), order and left-recursion subsections; `samples/expression-grammar/` with four goldens, `samples/mixed-parameters/` with two | TEST-2026-0009 T-02, T-08; TEST-2026-0011 | PASS |
| R11 | `\|` documented as ordered choice with its termination rule | `docs/library-keywords.md` row for `A \| B \| C` plus example line; `docs/syntax-cheat-sheet.md` and `docs/features.md` rows; "Left recursion through any alternative is refused" section | TEST-2026-0009 T-08 | PASS |

### PROP-2026-0004 use cases

| Use case | Expected | Observed | Test | Result |
|---|---|---|---|---|
| UC-01 | `add(3, mul(4, 5))` parses; no alternative matching gives the union diagnostic | parses and round-trips through `run`; `+` argument gives the union diagnostic | TEST-2026-0009 T-02, T-05 | PASS |
| UC-02 | `f(g(h(1)))` parses; depth above the bound hits the existing bound error | nests parse to 31 levels; deeper is refused with an `Err`; the dedicated depth message is not shown (R4) | TEST-2026-0009 T-02, T-03 | PARTIAL |
| UC-03 | A consumer reads which shape matched from `sub[0].func` | discriminator test and JSON assertions | TEST-2026-0009 T-07 | PASS |
| UC-04 | A left-recursive alternative is caught at load | refused with `left recursion` and the function name | TEST-2026-0009 T-04 | PASS |
| UC-05 | An author finds `bare` | worked section and sample in the authoring guide | TEST-2026-0009 T-08; TEST-2026-0011 | PASS |
| UC-06 | `def f(mut c: Counter, n: int)` parses; a parameter matching neither gives the union diagnostic | both orders parse; `sub` funcs `["mut_param","plain_param"]`; same input is rejected on 0.22.0 | TEST-2026-0009 T-10 | PASS |

### Measurements

| Measurement | Baseline | Observed | Threshold | Test | Result |
|---|---|---|---|---|---|
| M-01 transpile time, best of 5 | 190.903 µs | 194.935 µs (+2.11 %) | <= +10 % (<= 210.0 µs) | TEST-2026-0010 | PASS |
| M-02 wasm size | 1 386 116 B (1 342 891 at 0.21.0) | 1 389 633 B on the final tree (+3 517, +0.25 %; +3.48 % cumulative); 1 389 616 B before the version bump and the `capy docs` escape fix | <= 1 410 035 B (cumulative 5 %) | TEST-2026-0010 | PASS |
| M-03 `capy-core` direct dependencies | 1 | 1 (`regex`) | exactly 1 | TEST-2026-0010 | PASS |
| M-04 golden suite wall time | baseline copy 0.0611 s | 0.0612 s, ratio 1.002 | <= 1.15x | TEST-2026-0010 | PASS |

The 5 % wasm allowance is shared across releases and **20 402 bytes** of headroom
remain (20 419 before the final-tree re-measurement) (see `TEST-2026-0008`, `TEST-2026-0010`).

### Carried PROP-2026-0002 requirements (R1 to R19)

Evidence and per-requirement commands are in `TEST-2026-0011`.

| Requirement | Result | Note |
|---|---|---|
| R1, R2, R3 | PASS | 18 pages mention `capy ast` |
| R4 | PASS | `docs/CAPY_FOR_LLMS.md` lacked the `language-reference.md#operator-precedence` link on first run; added in this release, `grep -c` is now `1` on all four pages, `mkdocs build --strict` exits 0 |
| R5 | PASS | `parse-recovery`, `language-frontend` and showcase transcripts byte-identical apart from `<-` annotations, `...` elisions and a blank separator line |
| R6, R7 | PASS | roadmap rows present; all five 0.22.0 Known Limitations published |
| R8, R9, R10, R11 | PASS | `operator-precedence` and `parse-recovery` samples with goldens, `lib.capy`, no golden-less script (skip count 8 unchanged) |
| R12 | PASS | `golden.rs` compares `<base>.expected-ast.txt` through `ast_text::render` |
| R13, R14 | PASS | README counts re-derived: 130 directories, 131 golden cases; "Parser surface" section present |
| R15, R16, R17 | PASS | recovery tab, Tutorial 5 in nav, Pattern E in `ai-agents.md` |
| R18 | PASS | `mkdocs build --strict` exit 0; docs entry in `whats-new.md` |
| R19 | PASS | `parse-recovery` absent from `curated.rs` (doc comment only); README explains why |

## Deviations From the Plan

```text
   #   deviation                                   disposition
   ─   ─────────────────────────────────────────   ──────────────────────────
   1   not gated behind PROP-2026-0003 INC-1       recorded in the plan
   2   R1 narrowed to library functions            ADR-0003 OQ-08
   3   R6 wording differs                          union semantics met
   4   R4 depth message not what surfaces          PARTIAL, limitation
   5   ARCH-001 via additive `alts` field          ADR-0003 OQ-02
   6   T-11 and T-12 added beyond the proposal     plan scope growth
   7   PROP-2026-0002 carried in                   owner instruction
```

1. **Not gated behind `PROP-2026-0003` INC-1.** `PROP-2026-0004` asked to land
   behind that correctness fix. `PROP-2026-0003` is a draft with no approved
   plan, so it cannot gate a release. Recorded in `PLAN-2026-0003`.
2. **R1 narrowed from "function or type names" to library functions only**
   (`ADR-0003` OQ-08). Matching a flat type uses a different path
   (`capture_value` with stop-literals). A flat alternative is a `bare`
   one-capture function. Load error: `…alternation names "int", which is not a
   library function`.
3. **R6 wording.** The real diagnostic reads
   ``expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand` ``
   rather than the proposal's illustrative ``expected `call`, `number` or `ident` ``.
   Union semantics are met (every alternative is named); only the wording differs.
4. **R4: depth-70 input is refused with an `Err` (never a panic), but the
   "nesting too deep" message is not what surfaces.** The message is consumed by
   capture-local backtracking and the generic `expected` error shows. This is
   already the behaviour for single-type captures. It was a known limitation of
   0.22.0 (`RPT-2026-0002`, limitation 2), so the contract pinned by
   `recursion_guard.rs` (return an `Err`) holds. Marked `PARTIAL`.
5. **`ARCH-001` satisfied via the additive `alts` field** (`ADR-0003` OQ-02):
   `cap_type` keeps meaning alternative 1, `alts` holds alternatives 2 to n; no
   public field changes type.
6. **T-11 and T-12 were added by the plan** beyond the proposal's T-01 to T-10
   (the `|` versus `|>` collision and the `capy docs` union).
7. **`PROP-2026-0002` carried in by owner instruction.** Its engine content is
   the `ast_text` renderer already present in the tree; the rest is docs and
   samples.

## Unintended Behaviour

None found. Two guards against it were exercised:

- **Over-rejection:** `right_recursive_alternation_loads` shows the guard does not
  refuse `call`, which consumes a name and `(` before recursing.
- **Leakage between alternatives:** `failed_alternative_is_rewound` shows a failed
  earlier alternative leaves no consumed tokens behind.

## Unresolved Incidents

None; no incident occurred during implementation.

## Remaining Limitations

1. **Depth message surfaces as the generic `expected` error** at the 64-capture
   bound (R4, deviation 4). `samples/expression-grammar/`: 31 call levels parse,
   32 are refused.
2. **Alternatives must be library functions** (OQ-08).
3. **`cargo fmt --check` is not enforced** in this repo: 227 pre-existing diffs at
   HEAD; the gate is clippy, tests and `mkdocs --strict`.
4. **wasm headroom is 20 402 bytes** of the shared 5 % allowance.
5. **`capy docs` escapes the choice as `\|`** in its Markdown table. Found by `DEMO-2026-0003` U-06 (bare pipes read as column separators); fixed in `rust/src/domain/docs.rs` before release.
6. **The wasm `capyIntrospect` JSON carries `type` only, not `alts`.** It already omitted `optional` and `default`; the Rust `ArgInfo.alts` is the public surface.

## Required Follow-Up

| Item | Owner | Destination |
|---|---|---|
| Surface the "nesting too deep" message through capture-local backtracking | Capy Engine | a future plan (carries `RPT-2026-0002` limitation 2) |
| Named unions, so the same union is not repeated (OQ-04) | Capy Engine | revisit when a real grammar repeats one three or more times |
| `default` capture inside a repeated nonterminal fails when omitted (OQ-06); not investigated, out of scope | Capy Engine | a separate defect proposal |
| Leading-optional combinator (OQ-07) deferred | Capy Engine | revisit only if duplication recurs |
| ~~Add the `language-reference.md#operator-precedence` link to `docs/CAPY_FOR_LLMS.md`~~ | Capy Engine | **done** in this release |
| Remaining wasm headroom of 20 402 bytes | Capy Engine | any plan touching the wasm build must budget against it |
| Add `alts` to the wasm `capyIntrospect` JSON | Capy Engine | a future plan; additive |

## Questions Required by DOCUMENTATION.md Section 28

| Question | Answer |
|---|---|
| Was the planned functionality actually implemented? | Yes. Loader, `alts` data model, guard over every alternative, ordered-choice matcher with repetition, union diagnostic, introspection and docs table, samples and tests exist and pass. |
| Does it behave as expected? | Yes for R1 to R3, R5 to R11 and UC-01, UC-03 to UC-06. R4 and UC-02 behave as expected except the depth message wording (`PARTIAL`). |
| Were all expected files and components modified? | Yes for the implementation, tests, samples and docs (list below). Version files and the P6/P7 documents were written after this validation (`REL-0.23.0`, version bump, `DEMO-2026-0003`, `MAN-2026-0002`, `SYS-2026-0001`). |
| Did implementation introduce unintended behaviour? | No (see above). |
| Are there remaining limitations? | Yes, six, listed above. |
| Are there unresolved incidents? | No. |
| Are follow-up changes required? | Yes, listed in *Required Follow-Up*. |

## Files Changed

`git status --short` and `git diff --stat` at validation time (48 paths; 34 tracked
files modified, 1017 insertions, 129 deletions, plus untracked files).

```text
   ENGINE (7 files, the PLAN-2026-0003 change)
     M rust/src/infra/raw_library.rs                          RawArg.alts
     M rust/src/infra/capy_lib_parser.rs                      parse A | B | C
     M rust/src/domain/library.rs                             ArgEntry/PatternElement .alts
     M rust/src/orchestrator/features/make_library_loader.rs  validate, resolve, guard
     M rust/src/orchestrator/features/make_parser.rs          ordered choice
     M rust/src/capy.rs                                       ArgInfo.alts
     M rust/src/domain/docs.rs                                union in Type column

   TESTS
    ?? rust/tests/alternation.rs                              18 tests (new)
     M rust/tests/golden.rs                                   AST golden branch (carried)

   CARRIED ENGINE (PROP-2026-0002 support)
    ?? rust/src/domain/ast_text.rs                            shared renderer (new)
     M rust/src/domain/mod.rs                                 mod ast_text
     M rust/cli/src/cmd_ast.rs                                prints through ast_text
     M rust/playground/src/curated.rs                         doc comment: parse-recovery absent

   SAMPLES (new)
    ?? samples/expression-grammar/    ?? samples/mixed-parameters/
    ?? samples/language-frontend/     ?? samples/operator-precedence/
    ?? samples/parse-recovery/         M samples/README.md

   DOCS
     M CHANGELOG.md                   M docs/CAPY_FOR_LLMS.md   M docs/ai-agents.md
     M docs/ast-json.md               M docs/diagnostics.md     M docs/editor-tooling.md
     M docs/errors-and-debugging.md   M docs/faq.md             M docs/features.md
     M docs/grammar-as-contract.md    M docs/index.md           M docs/library-authoring.md
     M docs/library-keywords.md       M docs/roadmap.md         M docs/showcase.md
     M docs/syntax-cheat-sheet.md     M docs/troubleshooting.md M docs/tutorials/04-custom-operators.md
     M docs/use-cases.md              M docs/whats-new.md       M mkdocs.yml
    ?? docs/language-frontend.md     ?? docs/tutorials/05-reading-diagnostics.md

   PROGRAM DOCS
     M program_docs/index/document-index.md
    ?? program_docs/proposals/prop-2026-0002-...  prop-2026-0003-...  prop-2026-0004-...
    ?? program_docs/decisions/adr-0003-approve-nonterminal-alternation.md
    ?? program_docs/plans/plan-2026-0003-nonterminal-alternation.md
    ?? program_docs/testing/test-2026-0009 ... 0010 ... 0011 (this validation's evidence)
    ?? program_docs/reports/rpt-2026-0003-plan-2026-0003-validation.md
```

## Conclusion

`PLAN-2026-0003` is **validated, with one documented `PARTIAL`.** Ordered
alternation works as specified, its order and guard assertions were shown able to
fail by mutation, no pre-existing golden changed, and all four measurements pass.
The single engine-side shortfall (R4, depth message) is inherited unchanged from
0.22.0 and is recorded, not new. The carried documentation work is verified; the one missing link it turned up was added in the same release.

The lesson worth carrying is the mutation check: the order tests and the
second-alternative guard test each went red when the behaviour they guard was
broken, which is what turns a green run into evidence.

Release readiness: nothing found here blocks `REL-0.23.0`. Each `PARTIAL` must be
referenced from that document (`DOCUMENTATION.md` section 28).

## Related Documents

- PLAN-2026-0003
- PROP-2026-0004
- PROP-2026-0002
- ADR-0003
- TEST-2026-0009 — ordered alternation functional tests
- TEST-2026-0010 — measured results for 0.23.0
- TEST-2026-0011 — carried docs and samples verification
- TEST-2026-0008 — the shared wasm allowance
- RPT-2026-0002 — predecessor validation

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial validation |
| 2 | 2026-10-07 | Olivier | Corrections from a full read-through after the tag (documentation only; no code change) |
