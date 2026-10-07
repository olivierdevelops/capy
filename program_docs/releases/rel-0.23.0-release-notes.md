---
document_id: REL-0.23.0
title: Release 0.23.0 — Ordered Choice for Grammars
document_type: release
status: completed

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 1

authors:
  - Olivier

owner: Release Management
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - capy-cli
  - docs
  - samples

version: "0.23.0"
git_tag: v0.23.0
git_commit: 8f10723e07eec625e7721a4941651b3541cd1aa8
release_date: 2026-10-07
source_branch: main
previous_version: "0.22.0"
previous_tag: v0.22.0

affected_versions:
  from: "0.23.0"
  to: "0.23.0"

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Records what shipped in 0.23.0, how each update was verified, and the exact source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests, validation and Git tag.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - PROP-2026-0002
  - ADR-0003
  - RPT-2026-0003
  - DEMO-2026-0003
  - MAN-2026-0002
  - SYS-2026-0001
  - TEST-2026-0009
  - TEST-2026-0010
  - TEST-2026-0011
  - INC-2026-0001
  - TRBL-2026-0001
  - REL-0.22.0

supersedes: null
superseded_by: null

tags:
  - release
  - grammar
  - alternation

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Release 0.23.0 — Ordered Choice for Grammars

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Release Management
> **Affected Components:** capy-core, capy-cli, docs, samples

## Summary

Capy had sequence, repetition and recursion but no choice. One capture type may now name several
library functions, tried left to right, first match wins:

```text
   arg capture v call | name | num

   add ( 3 , mul ( 4 , 5 ) )
         │       └──────┬──────┘
         │              operand ─► call ✅ ─► mul ( operand operand )
         └ operand ─► call ❌ ─► name ❌ ─► num ✅
```

A declared expression grammar now nests, and a parameter list can mix marked and unmarked parameters.
The release also carries the already-implemented `PROP-2026-0002` public docs and samples.

## Release Identity

| Field | Value |
|---|---|
| Release Version | 0.23.0 |
| Git Tag | `v0.23.0` (annotated) |
| Git Commit | `8f10723e07eec625e7721a4941651b3541cd1aa8` |
| Release Date | 2026-10-07 |
| Source Branch | `main` |
| Previous Version | 0.22.0 |
| Previous Tag | `v0.22.0` |
| Previous Release Commit | `f72fa3d0849d48527454b492e2ed7a4a1db76e7b` |
| Repository | `git@github:olivierdevelops/capy.git` |
| Version source | `rust/Cargo.toml` `[workspace.package] version`, plus the four crate path-dependency pins and `Cargo.lock` (7 files) |

```text
   REL-0.23.0
      │ version = 0.23.0
      │ tag     = v0.23.0
      │ commit  = 8f10723e07eec625e7721a4941651b3541cd1aa8
      ▼
   Git Tag v0.23.0 ─► Git Commit 8f10723 ─► exact source state
   (git rev-list -n 1 v0.23.0 == git rev-parse HEAD at tagging; checked out fresh: 128 tests, 131 goldens)
```

Commits, in order: `d7d99fc` feat, `5cacbe4` chore(release), `8f10723` docs(program) **← tagged**, then
the finalize commit that adds this document (the same shape as 0.22.0).

## Plan

`PLAN-2026-0003`. Decision in `ADR-0003`. Proposal `PROP-2026-0004` revision 5.

## Project Standards Baseline

| Standards Index | Revision | Applicable Rules | Proposal Validation |
|---|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says 2 — `PROP-2026-0004` OQ-05, unreconciled) | GOAL-001, GOAL-002, PHIL-001, PHIL-002, CODE-001, CODE-003, ARCH-001, ARCH-002, ARCH-003, QUAL-001, QUAL-002, QUAL-003, GATE-001, GATE-002 | PASS (ARCH-001 resolved by `ADR-0003`) |

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| PROP-2026-0004 R1, R3, R4 / UC-01, UC-02 | Ambit consumer: "still cannot nest" (UQ-01, UQ-04) | U-01 | PASS (R4 / UC-02 PARTIAL — see Known Limitations) |
| PROP-2026-0004 R1, R3 / UC-06 | Owner: "why not use mut" (UQ-05) | U-02 | PASS |
| PROP-2026-0004 R2 | determinism of choice | U-03 | PASS |
| PROP-2026-0004 R6 | diagnostic style | U-04 | PASS |
| PROP-2026-0004 R5 / UC-04 | `ARCH-003` safety | U-05 | PASS |
| PROP-2026-0004 R9 | truthful introspection | U-06 | PASS |
| PROP-2026-0004 R8 / UC-03 | stable JSON | U-07 | PASS |
| PROP-2026-0004 R1 / OQ-08 | `ADR-0003` | U-08 | PASS |
| PROP-2026-0004 R7 | `GOAL-002` | U-13 | PASS |
| PROP-2026-0004 R10, R11 | docs obligations | U-01, U-02 | PASS |
| PROP-2026-0002 R1…R19 | carried public docs and samples | U-09…U-12 | PASS (19 of 19) |

## Added

- **Ordered alternation in a capture type**: `arg capture v A | B | C`, glued `A|B` accepted. Alternatives
  are library functions; a repetition suffix after the last name applies to the whole choice and composes
  with `sep` / `join`.
- `ArgInfo.alts` (public introspection) — alternatives 2…n; `type_` keeps meaning alternative 1.
- `capy docs` prints the whole choice in the Type column, pipes escaped for the Markdown table.
- Samples: `samples/expression-grammar/`, `samples/mixed-parameters/`.
- Carried from `PROP-2026-0002`: `docs/language-frontend.md`, `docs/tutorials/05-reading-diagnostics.md`,
  `samples/language-frontend/`, `samples/operator-precedence/`, `samples/parse-recovery/`, the shared
  `capy_core::domain::ast_text` renderer, and the `<base>.expected-ast.txt` golden kind.
- Program documents: `ADR-0003`, `PLAN-2026-0003`, `TEST-2026-0009…0011`, `RPT-2026-0003`,
  `DEMO-2026-0003`, `MAN-2026-0002`, `INC-2026-0001`, `TRBL-2026-0001`.

## Changed

- The left-recursion guard walks **every** alternative, so a cycle reachable only through a later
  alternative is refused at load with the existing cycle trace.
- `capy ast` prints through `ast_text` (one renderer shared with the golden runner); output is unchanged.
- `docs/language-frontend.md` no longer says a nested call has no route; it points at ordered choice.
- Workspace version 0.22.0 → 0.23.0.

## Fixed

Nothing was broken in a released version. One defect in the new change was found and fixed before the
release commit — see Incidents.

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP-2026-0004 R1, R4 | Nested calls parse | `capy run samples/expression-grammar/lib.capy samples/expression-grammar/script.capy` | `return add(3, mul(4, 5));` … | TEST-2026-0009 T-02, DEMO-2026-0003 U-01 |
| U-02 | PROP-2026-0004 UC-06 | Mixed marked / unmarked parameters | `capy run samples/mixed-parameters/lib.capy samples/mixed-parameters/script.capy` | `update(&c: Counter, n: int)` | TEST-2026-0009 T-10, DEMO-2026-0003 U-02 |
| U-03 | PROP-2026-0004 R2 | First match wins, order observable | reorder `name` / `num` in a library | the matched `func` swaps | TEST-2026-0009 T-01, DEMO-2026-0003 U-03 |
| U-04 | PROP-2026-0004 R6 | Failure names every alternative | `capy run … broken.capy` | ``expected a `call`, a `name`, or a `num` `` | TEST-2026-0009 T-05, DEMO-2026-0003 U-04 |
| U-05 | PROP-2026-0004 R5 | Cycle through a later alternative refused | `capy check` on `expr = atom \| expr` | `left recursion … (cycle: expr -> expr)`, exit 1 | TEST-2026-0009 T-04, DEMO-2026-0003 U-05 |
| U-06 | PROP-2026-0004 R9 | `capy docs` prints the choice | `capy docs samples/expression-grammar/lib.capy` | `` `call \| name \| num` `` | TEST-2026-0009 T-12, DEMO-2026-0003 U-06 |
| U-07 | PROP-2026-0004 R8 | `sub[].func` names the alternative | `capy ast --json …` | `"func": "call"`, `schema_version` 1 | TEST-2026-0009 T-07, DEMO-2026-0003 U-07 |
| U-08 | PROP-2026-0004 OQ-08 | A built-in type is refused as an alternative | library with `call \| int` | `…names "int", which is not a library function` | TEST-2026-0009, DEMO-2026-0003 U-08 |
| U-09…U-12 | PROP-2026-0002 | carried docs and samples | see `DEMO-2026-0003` | outputs match the pages | TEST-2026-0011, DEMO-2026-0003 |
| U-13 | PROP-2026-0004 R7 | No existing library changes | `cd rust && cargo test --workspace` | 128 passed, 0 failed | TEST-2026-0009 T-06, DEMO-2026-0003 U-13 |

## Tests

| Test | Requirement | Result |
|---|---|---|
| TEST-2026-0009 | PLAN-2026-0003 R1…R11, T-01…T-12 | PARTIAL (T-03 / R4: refused with an `Err`, depth message generic) |
| TEST-2026-0010 | M-01…M-04 | PASS |
| TEST-2026-0011 | carried PROP-2026-0002 R1…R19 | PASS |

Gate summary on the tagged tree: `cargo test --workspace` 128 passed, 0 failed · `cargo clippy --workspace
--all-targets -- -D warnings` clean · goldens **131 passed, 8 skipped, 0 failed** (125 before, +6 new) ·
`mkdocs build --strict` exit 0 · a fresh `git worktree` checkout of the tagged commit reproduces 128 / 131.
Two mutation checks confirm the key tests can fail (guard reverted; alternative order reversed).

## Validation

`RPT-2026-0003`. Of PROP-2026-0004 R1…R11: ten `PASS`, one `PARTIAL` (R4). UC-02 inherits R4's `PARTIAL`.
Carried PROP-2026-0002 R1…R19: all `PASS` (R4 failed on first run, link added). No `FAIL`.

## Measured Results

| Update | Requirement | Metric | Baseline | Expected Threshold | Actual | Result | Test Report and Source |
|---|---|---|---|---|---|---|---|
| U-01…U-08 | PLAN-2026-0003 M-01 | transpile time, best of 5 | 190.903 µs | ≤ +10 % (≤ 210.0 µs) | 194.935 µs (+2.11 %) | PASS | TEST-2026-0010, `nativebench` |
| U-01…U-08 | M-02 | wasm size (cumulative 5 % allowance since 0.21.0) | 1 386 116 B now; 1 342 891 B at 0.21.0 | ≤ 1 410 035 B | 1 389 633 B (+3.48 % vs 0.21.0) | PASS | TEST-2026-0010, `ls -l` |
| U-13 | M-03 | `capy-core` direct dependencies | 1 | exactly 1 | 1 (`regex`) | PASS | TEST-2026-0010, `cargo tree` |
| U-13 | M-04 | golden suite wall time | baseline copy 0.0611 s | ≤ 1.15× | 0.0612 s (1.002×) | PASS | TEST-2026-0010 |

The shared 5 % wasm allowance: **3.48 % spent, 20 402 bytes left** (ceiling 1 410 035).

## Incidents

`INC-2026-0001` (resolved): `capy docs` printed unescaped pipes in a Markdown table cell. Found by
`DEMO-2026-0003` U-06 before the release commit; fixed and re-verified. Never shipped.

## Troubleshooting

`TRBL-2026-0001`: a stale `capy` on PATH rejects shipped samples; run `rust/target/debug/capy` explicitly.

## Release Verification Guide and Demo

`DEMO-2026-0003` — 13 `U-NN` rows, every command executed; result 13 PASS after `INC-2026-0001`.

## Manual

`MAN-2026-0002` (new chapter, feature catalogue entry); `MAN-2026-0001` cross-linked.
User docs: `docs/library-authoring.md` ("Ordered choice"), `docs/library-keywords.md`, `docs/features.md`,
`docs/syntax-cheat-sheet.md`, `docs/roadmap.md`, `docs/whats-new.md`, `CHANGELOG.md`.

## System

`SYS-2026-0001` updated: the matcher, loader, lib-parser and public surfaces as implemented.

## Documentation Impact

| Artifact | Decision | Updated Document or Reason |
|---|---|---|
| Release verification guide / demo | UPDATED | DEMO-2026-0003 |
| README.md | NOT APPLICABLE | the top-level README has no grammar-combinator or capability list, and does not mention the 0.22.0 diagnostics either; not a headline change |
| System documentation | UPDATED | SYS-2026-0001 |
| Architecture documentation | NOT APPLICABLE | `ARCH-2026-0001` describes the AST node shape (`func`, `captures`), which is unchanged; `docs/architecture.md` states only the 64-level bound and the left-recursion rejection, both still true |
| API and CLI reference | UPDATED | `docs/embedding.md` (`ArgInfo.alts`); `docs/cli.md` NOT APPLICABLE — no command or flag changed |
| Manual | UPDATED | MAN-2026-0002; MAN-2026-0001 cross-link |

## Source Changes

| Source | Change | Reason |
|---|---|---|
| `rust/src/infra/raw_library.rs` | Modified | `RawArg.alts` |
| `rust/src/domain/library.rs` | Modified | `ArgEntry.alts`, `PatternElement.alts`, `alternatives()` |
| `rust/src/infra/capy_lib_parser.rs` | Modified | parse `A \| B \| C`, glued `A\|B`, malformed forms rejected |
| `rust/src/orchestrator/features/make_library_loader.rs` | Modified | validate alternatives as functions; guard walks every alternative |
| `rust/src/orchestrator/features/make_parser.rs` | Modified | `match_alt` ordered choice; union of expectations |
| `rust/src/capy.rs` | Modified | `ArgInfo.alts` |
| `rust/src/domain/docs.rs` | Modified | print the union, pipes escaped |
| `rust/src/domain/ast_text.rs`, `rust/src/domain/mod.rs`, `rust/cli/src/cmd_ast.rs` | Created / Modified | shared AST text renderer (carried PROP-2026-0002 R12) |
| `rust/tests/alternation.rs` | Created | 18 tests |
| `rust/tests/golden.rs` | Modified | `<base>.expected-ast.txt` golden kind (carried) |
| `rust/playground/src/curated.rs` | Modified | two curated samples; `parse-recovery` deliberately absent (carried R19) |
| `rust/Cargo.toml`, `rust/Cargo.lock`, `rust/{cli,devtools,mcp,playground,wasm}/Cargo.toml` | Modified | version 0.23.0 |
| `samples/*` | Created / Modified | five samples, six new goldens in two of them, `README.md` |
| `docs/*`, `CHANGELOG.md`, `mkdocs.yml` | Created / Modified | user documentation |

## Known Issues

None unresolved. `INC-2026-0001` is resolved.

## Known Limitations

1. **The depth bound is not named (R4 / UC-02 `PARTIAL`).** Input nested past the 64-capture bound is refused
   with an `Err`, never a panic — 31 call levels parse and 32 are refused in `samples/expression-grammar`.
   The "nesting too deep" message is consumed by capture-local backtracking and surfaces as the generic
   `expected` error, as it already did for a single-type capture. Carried from `REL-0.22.0` limitation 2.
2. **Alternatives are library functions only** (`ADR-0003` OQ-08). Wrap a flat type in a `bare` function.
3. **The union diagnostic reads** ``expected a `call`, a `name`, or a `num` ``, not the proposal's illustrative
   wording. Union semantics are met.
4. **The wasm `capyIntrospect` JSON carries `type` only, not `alts`.** It already omitted `optional` and
   `default`; `ArgInfo.alts` is the public surface.
5. **A permissive earlier alternative shadows a later one.** `int` accepts a bare identifier at parse time, so
   list the narrower alternative first (documented in `docs/library-authoring.md`).
6. **Infix operators around a user-declared call** still have no route: the precedence ladder lives inside
   `any`.
7. **`capy --version` prints `capy dev`** and cannot reveal a stale build (`TRBL-2026-0001`).
8. **`cargo fmt --check` is not enforced**: 227 pre-existing diffs at `HEAD` before this release.

## Follow-up Work

| Item | Destination |
|---|---|
| Surface the "nesting too deep" message through capture-local backtracking | a future plan |
| Named unions (`PROP-2026-0004` Alternative B, OQ-04) | revisit when one union repeats three or more times |
| `default` capture inside a repeated nonterminal fails when omitted (OQ-06) — not investigated | a separate defect proposal |
| Leading-optional combinator (OQ-07) | revisit only if duplication recurs |
| Add `alts` to the wasm `capyIntrospect` JSON | additive; a future plan |
| Stamp the Cargo version into `capy --version` | not planned |
| `PROP-2026-0003` (expression trees over JSON) — still a draft, no plan | owner decision |

## Deviations from the Plan and Proposal

1. Not gated behind `PROP-2026-0003` INC-1, because that proposal is a draft with no approved plan.
2. R1 narrowed from "function or type names" to library functions only (`ADR-0003`).
3. R4 `PARTIAL` (Known Limitation 1); R6 wording differs (Known Limitation 3).
4. `T-11` and `T-12` were added by the plan beyond the proposal's `T-01…T-10`.
5. `PROP-2026-0002` was carried into this release by the owner's instruction; its docs and the engine change
   were committed together because several pages carry both sets of edits.
6. The release commit also contains the draft proposals `PROP-2026-0003` and the proposal documents, by the
   owner's instruction to fold the uncommitted work in.

## Upgrade Notes

Additive for `.capy` libraries: `|` in a capture type was a load error, so no existing library can contain
it; every existing golden is byte-identical.

For Rust consumers: `ArgInfo` gains a public field `alts: Vec<String>`. It is not `#[non_exhaustive]`, so an
exhaustive struct literal of `ArgInfo` in external code needs `alts` or `..Default::default()`. `PatternElement`
and `ArgEntry` gain the same field. Nothing existing changes meaning: `type_` / `cap_type` still hold
alternative 1.

## Related Documents

- `PLAN-2026-0003`, `PROP-2026-0004`, `PROP-2026-0002`, `ADR-0003`, `RPT-2026-0003`, `DEMO-2026-0003`,
  `MAN-2026-0002`, `SYS-2026-0001`, `TEST-2026-0009`, `TEST-2026-0010`, `TEST-2026-0011`, `INC-2026-0001`,
  `TRBL-2026-0001`, `REL-0.22.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial release document: tag `v0.23.0` created and verified against commit `8f10723e07eec625e7721a4941651b3541cd1aa8` before this document was written |
