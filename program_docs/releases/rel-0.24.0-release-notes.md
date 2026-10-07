---
document_id: REL-0.24.0
title: Release 0.24.0 — The Nesting Bound Has a Name
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
  - capy-wasm-abi
  - docs

version: "0.24.0"
git_tag: v0.24.0
git_commit: 54f1a5ead499e5ad4d5317ba19cfc4abe78bee63
release_date: 2026-10-07
source_branch: main
previous_version: "0.23.0"
previous_tag: v0.23.0

affected_versions:
  from: "0.24.0"
  to: "0.24.0"

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Records what shipped in 0.24.0, how each update was verified, and the exact source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests, validation and Git tag.

related_documents:
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - RPT-2026-0004
  - DEMO-2026-0004
  - MAN-2026-0002
  - SYS-2026-0001
  - TEST-2026-0012
  - TEST-2026-0013
  - TRBL-2026-0001
  - REL-0.23.0

supersedes: null
superseded_by: null

tags:
  - release
  - diagnostics
  - wasm
  - cli

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Release 0.24.0 — The Nesting Bound Has a Name

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Release Management
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

Closes the three code follow-ups that `REL-0.23.0` left open, including its one `PARTIAL`
result.

```text
  FOLLOW-UP (REL-0.23.0)                              0.24.0
  ──────────────────────────────────────────────────  ─────────────────────────────────────
  nesting bound surfaced as generic `expected …`      `nesting too deep (limit 64) …`  E0003
  wasm capyIntrospect JSON omitted `alts`             `"type":"call","alts":["name","num"]`
  `capy version` printed `capy dev` for every build   `capy 0.24.0`
```

The bound itself did not move: 31 call levels parse, 32 are refused.

## Release Identity

| Field | Value |
|---|---|
| Release Version | 0.24.0 |
| Git Tag | `v0.24.0` (annotated) |
| Git Commit | `54f1a5ead499e5ad4d5317ba19cfc4abe78bee63` |
| Release Date | 2026-10-07 |
| Source Branch | `main` |
| Previous Version | 0.23.0 |
| Previous Tag | `v0.23.0` |
| Previous Release Commit | `8f10723e07eec625e7721a4941651b3541cd1aa8` |
| Repository | `git@github:olivierdevelops/capy.git` |
| Version source | `rust/Cargo.toml` `[workspace.package] version`, plus the four crate path-dependency pins and `Cargo.lock` (7 files) |

```text
   REL-0.24.0
      │ version = 0.24.0
      │ tag     = v0.24.0
      │ commit  = 54f1a5ead499e5ad4d5317ba19cfc4abe78bee63
      ▼
   Git Tag v0.24.0 ─► Git Commit 54f1a5e ─► exact source state
   (git rev-list -n 1 v0.24.0 == git rev-parse HEAD at tagging; checked out fresh: 135 tests, 131 goldens, `capy 0.24.0`)
```

Commits, in order: `a046032` proposal + ADR + plan (**before** the code), `8769f51` fix,
`cb23972` chore(release), `54f1a5e` docs(program) **← tagged**, then the finalize commit that
adds this document.

## Plan

`PLAN-2026-0004`. Decision in `ADR-0004`. Proposal `PROP-2026-0005` revision 2.

## Project Standards Baseline

| Standards Index | Revision | Applicable Rules | Proposal Validation |
|---|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says 2 — `PROP-2026-0004` OQ-05, still unreconciled) | GOAL-001, GOAL-002, PHIL-001, CODE-001, CODE-003, ARCH-001, ARCH-002, ARCH-003, QUAL-001, QUAL-002, QUAL-003, GATE-001, GATE-002 | PASS |

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| PROP-2026-0005 R1, R2, R4 / UC-01 | Owner: "Code follow-ups as 0.24.0" (UQ-02) — the depth message, `REL-0.23.0` limitation 1 | U-01, U-02, U-03 | PASS |
| PROP-2026-0005 R3 / UC-02 | no leak between statements | U-04 | PASS |
| PROP-2026-0005 R5 / UC-03 | `REL-0.23.0` limitation 4 | U-05 | PASS |
| PROP-2026-0005 R6 / UC-04 | `REL-0.23.0` limitation 7, `TRBL-2026-0001` | U-06 | PASS |
| PROP-2026-0005 R7 | `GOAL-002` | U-07 | PASS |
| PROP-2026-0005 R8 | docs obligations | U-08 | PASS |

## Added

- **`E0003` is emitted.** A statement that fails because the nesting bound was reached reports
  ``nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow``;
  `capy ast` / `Library::parse` give it code `E0003`. Internally `OuterP.depth_err` and
  `fail_code` carry the refusal past `match_one`'s rewind.
- **`alts` in the browser introspection JSON** (`capy_introspect`): `"type":"call","alts":["name","num"]`,
  `"alts":[]` for a plain capture.
- Tests: three depth tests, two wasm unit tests, two CLI version tests (`rust/cli/tests/version.rs`).
- Program documents: `PROP-2026-0005`, `ADR-0004`, `PLAN-2026-0004`, `TEST-2026-0012`, `TEST-2026-0013`,
  `RPT-2026-0004`, `DEMO-2026-0004`.

## Changed

- `capy version` / `capy --version` print `capy <crate version>` for an unstamped build (was `capy dev`).
  A stamped `CAPY_VERSION` still wins.
- One error message changes for over-deep input (generic `expected …` → the named bound). No golden
  depended on it.
- `MAN-2026-0002`, `SYS-2026-0001`, `TRBL-2026-0001` and the user docs describe the new behaviour.
- Workspace version 0.23.0 → 0.24.0.

## Fixed

`REL-0.23.0` Known Limitations 1 (depth message, the `PARTIAL` R4 / UC-02), 4 (wasm JSON `alts`) and 7
(`capy --version` printed `dev`).

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP-2026-0005 R1 | The bound is named | `capy run samples/expression-grammar/lib.capy` on a 70-deep `return f(f(…1…))` | ``error: nesting too deep (limit 64) while matching "call \| name \| num" …`` | TEST-2026-0012 T-01, DEMO-2026-0004 U-01 |
| U-02 | PROP-2026-0005 R2 | `capy ast` gives `E0003` | `capy ast …` on the same file | `error[E0003] 1:1: nesting too deep …`, exit 1 | TEST-2026-0012 T-02, DEMO-2026-0004 U-02 |
| U-03 | PROP-2026-0005 R4 | The bound did not move | 31-deep, then 32-deep | exit 0, then the named error and exit 1 | TEST-2026-0012 T-01, DEMO-2026-0004 U-03 |
| U-04 | PROP-2026-0005 R3 | No leak between statements | `return add(1, 2)`, a 40-deep line, `return add(3, +)` through `capy ast` | `E0003` on line 2, `E0001` on line 3 | TEST-2026-0012 T-03, DEMO-2026-0004 U-04 |
| U-05 | PROP-2026-0005 R5 | Browser JSON carries `alts` | `cd rust && cargo test -p capy-wasm-abi` | 2 passed | TEST-2026-0012 T-04, DEMO-2026-0004 U-05 |
| U-06 | PROP-2026-0005 R6 | `capy version` identifies the build | `capy version`; `capy --version` | both `capy 0.24.0` | TEST-2026-0012 T-05, DEMO-2026-0004 U-06 |
| U-07 | PROP-2026-0005 R7 | Nothing existing changed | `cd rust && cargo test --workspace`; `cargo test --test golden -- --nocapture` | 135 passed, 0 failed; `goldens: 131 passed, 8 skipped (no golden file), 0 failed` | TEST-2026-0012 T-06, T-08, DEMO-2026-0004 U-07 |
| U-08 | PROP-2026-0005 R8 | Docs true and building | `mkdocs build --strict` | exit 0 | TEST-2026-0012 T-07, DEMO-2026-0004 U-08 |

## Tests

| Test | Requirement | Result |
|---|---|---|
| TEST-2026-0012 | PLAN-2026-0004 R1…R8, T-01…T-08 | PASS |
| TEST-2026-0013 | M-01…M-04 | PASS |

Gate summary on the tagged tree: `cargo test --workspace` 135 passed, 0 failed · `cargo clippy --workspace
--all-targets -- -D warnings` clean · goldens **131 passed, 8 skipped, 0 failed** (unchanged: no golden added or
edited) · `mkdocs build --strict` exit 0 · a fresh `git worktree` checkout of the tagged commit reproduces 135 / 131 and
`capy 0.24.0`. Three mutation checks (depth report disabled; version fallback reverted to `dev`; no `alts` emitted)
each turn the named tests red.

## Validation

`RPT-2026-0004`: all of R1…R8 and UC-01…UC-04 `PASS`; no `PARTIAL`, no `FAIL`. The `PARTIAL` recorded for R4 in
`RPT-2026-0003` is resolved.

## Measured Results

| Update | Requirement | Metric | Baseline | Expected Threshold | Actual | Result | Test Report and Source |
|---|---|---|---|---|---|---|---|
| U-01…U-06 | PLAN-2026-0004 M-01 | transpile time, best of 7, interleaved | retained v0.23.0 `nativebench`, best 195.896 µs | ≤ +10 % | best 195.687 µs, ratio 0.9989 | PASS | TEST-2026-0013, `nativebench` |
| U-05 | M-02 | wasm size | 1 389 633 B | ≤ +3 000 B; ceiling 1 410 035 | 1 390 420 B (+787) | PASS | TEST-2026-0013, `ls -l` |
| U-07 | M-03 | `capy-core` direct dependencies | 1 | exactly 1 | 1 (`regex`) | PASS | TEST-2026-0013, `cargo tree` |
| U-07 | M-04 | golden suite wall time, best of 7, interleaved | retained v0.23.0 test binary, 0.0637 s | ≤ 1.15× | 0.0641 s (1.005×) | PASS | TEST-2026-0013 |

The shared 5 % wasm allowance: **3.54 % spent, 19 615 bytes left** (ceiling 1 410 035).

## Incidents

None. No unexpected bug was found during this release.

## Troubleshooting

`TRBL-2026-0001` updated: a second cause of a stale CLI was found — a plain `cargo build` in `rust/` builds only
`capy-core`, not the CLI (use `cargo build --workspace`). Since 0.24.0 `capy version` identifies the build.

## Release Verification Guide and Demo

`DEMO-2026-0004` — 8 `U-NN` rows, every command executed; result 8 PASS.

## Manual

`MAN-2026-0002` updated (revision 3): the depth bound, the browser JSON, `capy version`; two limitations that became
false were removed.

## System

`SYS-2026-0001` updated (revision 4): the nesting bound as implemented (`depth_err`, `fail_code`, reporting order),
the wasm `alts` field, and the version fallback.

## Documentation Impact

| Artifact | Decision | Updated Document or Reason |
|---|---|---|
| Release verification guide / demo | UPDATED | DEMO-2026-0004 |
| README.md | NOT APPLICABLE | the top-level README has no mention of `capy dev`, `E0003`, nesting depth or `alts`, and no version output |
| System documentation | UPDATED | SYS-2026-0001 |
| Architecture documentation | NOT APPLICABLE | `docs/architecture.md` and `docs/how-capy-parses.md` do not describe the depth error, the version string or the wasm JSON; `ARCH-2026-0001` (node shape) is unchanged |
| API and CLI reference | UPDATED | `docs/cli.md` (`capy version`), `docs/embedding.md` (browser JSON `alts`), `docs/diagnostics.md` (`E0003` emitted) |
| Manual | UPDATED | MAN-2026-0002 |

## Source Changes

| Source | Change | Reason |
|---|---|---|
| `rust/src/orchestrator/features/make_parser.rs` | Modified | `depth_err`, `fail_code`; report the bound as `E0003` |
| `rust/wasm/src/lib.rs` | Modified | `alts` in `capy_introspect`; two unit tests |
| `rust/cli/src/main.rs` | Modified | `VERSION` falls back to the crate version |
| `rust/cli/tests/version.rs` | Created | two CLI tests |
| `rust/tests/alternation.rs` | Modified | depth tests: three added, one rewritten (18 → 21 tests) |
| `rust/Cargo.toml`, `rust/Cargo.lock`, `rust/{cli,devtools,mcp,playground,wasm}/Cargo.toml` | Modified | version 0.24.0 |
| `docs/diagnostics.md`, `errors-and-debugging.md`, `library-authoring.md`, `cli.md`, `embedding.md`, `roadmap.md`, `language-frontend.md`, `whats-new.md`, `CHANGELOG.md` | Modified | user documentation |

## Known Issues

None unresolved.

## Known Limitations

1. **`E0002` is still reserved and unemitted.**
2. **The browser JSON still omits `optional` and `default`** (`PROP-2026-0005` OQ-01).
3. **The bound is still 64 captures** (31 call levels in `samples/expression-grammar/`); this release only reports it.
4. **A more specific error in the same statement wins over the bound error** — a block-body error is reported first
   (`SYS-2026-0001`). `parse_stmt` also resets `depth_err` for block bodies; no test covers that interaction.
5. **The stamped `CAPY_VERSION` path has no test**; the unstamped test returns early on a stamped build.
6. **The M-01 / M-04 baselines are retained binaries that are not committed**, so those two comparisons cannot be
   repeated from the repository alone.
7. **Alternatives are library functions only; first match wins** (carried from `REL-0.23.0`).
8. **`cargo fmt --check` is not enforced**: 227 pre-existing diffs at `HEAD`.
9. **wasm headroom is 19 615 bytes** of the shared 5 % allowance.

## Follow-up Work

| Item | Destination |
|---|---|
| `default` capture inside a repeated nonterminal fails when omitted (`PROP-2026-0004` OQ-06) — not investigated | a separate defect proposal |
| Add `optional` / `default` to the wasm JSON if a browser tool needs them | additive; budget against 19 615 bytes |
| Decide whether `E0002` should be emitted or retired | a future proposal |
| Test the stamped `CAPY_VERSION` path | next release that touches the CLI |
| Named unions (`PROP-2026-0004` OQ-04) | when one union repeats three or more times |
| `PROP-2026-0003` (expression trees over JSON) — still a draft, no plan | owner decision |

## Deviations from the Plan and Proposal

1. **Order of work (wrong under §21.3 and §24).** The three fixes were drafted, tested and mutation-checked
   **before** `PROP-2026-0005`, `ADR-0004` and `PLAN-2026-0004` existed. It was caught before anything was committed or
   measured: the code was saved as a patch and removed, baselines were measured on the clean tree, the plan with its
   thresholds was committed (`a046032`), and the patch was then re-applied (`8769f51`). Thresholds were therefore frozen
   before any "after" measurement. The tests were written alongside the code, not from the approved plan.
2. `E0003` is reported only when the statement would otherwise fail generically, as `PROP-2026-0005` C-01 states.
3. `PLAN-2026-0004` pairs R3 with UC-01 while the proposal pairs the leak case with UC-02; `RPT-2026-0004` follows the
   proposal. Results do not depend on the pairing.
4. After the `v0.23.0` tag, a documentation-only correction commit (`42c3cea`) fixed stale numbers in the 0.23.0 documents.
   The `v0.23.0` tag was not moved.

## Upgrade Notes

- **Error text:** over-deep input now reports `nesting too deep (limit 64) while matching "…"` instead of a generic
  `expected …`. A tool that matched on the old text should match on `E0003` (`capy ast`, `Library::parse`) instead.
- **`capy version`:** an unstamped local build prints `capy 0.24.0`, not `capy dev`. A script that tested for `dev`
  to detect a local build must change; release builds stamp `CAPY_VERSION` and are unaffected.
- **Browser JSON:** `capyIntrospect` args gain `alts` after `type`. Additive; existing readers keep working.
- Nothing else changes: every pre-existing golden is byte-identical.

## Related Documents

- `PLAN-2026-0004`, `PROP-2026-0005`, `ADR-0004`, `RPT-2026-0004`, `DEMO-2026-0004`, `MAN-2026-0002`,
  `SYS-2026-0001`, `TEST-2026-0012`, `TEST-2026-0013`, `TRBL-2026-0001`, `REL-0.23.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial release document: tag `v0.24.0` created and verified against commit `54f1a5ead499e5ad4d5317ba19cfc4abe78bee63` before this document was written |
