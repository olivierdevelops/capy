---
document_id: REL-0.26.0
title: Release 0.26.0 — Capy from C
document_type: release
status: completed

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 1

authors:
  - Olivier
  - Claude (Sonnet 5.5)

owner: Release Management
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-ffi
  - capy-core
  - capy-cli
  - docs

version: "0.26.0"
git_tag: v0.26.0
git_commit: 05ae277798183abf79b038c3716c817ed485c1a8
release_date: 2026-10-13
source_branch: main
previous_version: "0.25.0"
previous_tag: v0.25.0

affected_versions:
  from: "0.26.0"
  to: "0.26.0"

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Records what shipped in 0.26.0, how each update was verified, and the exact source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests, validation and Git tag.

related_documents:
  - PLAN-2026-0005
  - PROP-2026-0008
  - ADR-0005
  - TEST-2026-0015
  - RPT-2026-0005
  - DEMO-2026-0005
  - MAN-2026-0003
  - SYS-2026-0001
  - ARCH-2026-0002
  - INC-2026-0002
  - INC-2026-0003
  - REL-0.24.0

supersedes: null
superseded_by: null

tags:
  - release
  - ffi
  - c-abi

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-13
---

# Release 0.26.0 — Capy from C

> **Status:** Completed
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Release Management
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Summary

Capy can now be called from C. A new `capy-ffi` crate exposes the parse tree — statements, captures, typed
expressions, spans, error regions and diagnostics — through opaque handles and `u32` ids, so a compiler written in
another language can use Capy as its parser. The release also carries the performance work of `PROP-2026-0007`
that had been committed under *Unreleased*, and fixes a crash found along the way.

```text
  0.25.0                                          0.26.0
  ──────────────────────────────────────────────  ───────────────────────────────────────────────────
  no C interface                                  libcapy_ffi.{a,dylib,so} + capy.h, 75 functions, ABI 1
  AST JSON has no typed expressions               capy ast --json --with-expr  (opt-in; default unchanged)
  ~20 000 nested ( ) abort the process            63 pairs parse; the 64th gets a diagnostic
  run renders every statement via a tree          run: 18-24x faster on flat statements (PROP-2026-0007)
```

## Release Identity

| Field | Value |
|---|---|
| Release Version | 0.26.0 |
| Git Tag | `v0.26.0` (annotated) |
| Git Commit | `05ae277798183abf79b038c3716c817ed485c1a8` |
| Release Date | 2026-10-13 |
| Source Branch | `main` |
| Previous Version | 0.25.0 |
| Previous Tag | `v0.25.0` |
| Previous Release Commit | `75a9f684935c22cd2f0e67f9c67cad8549d2d4e1` |
| Repository | `git@github:olivierdevelops/capy.git` (**not pushed**) |
| Version source | `rust/Cargo.toml` `[workspace.package] version`, six member manifests' `capy-core` pins, `Cargo.lock` (7 files); `capy version` prints `capy 0.26.0` |

```text
   REL-0.26.0
      │ version = 0.26.0
      │ tag     = v0.26.0
      │ commit  = 05ae277798183abf79b038c3716c817ed485c1a8
      ▼
   Git Tag v0.26.0 ─► Git Commit 05ae277 ─► exact source state
   (git rev-list -n 1 v0.26.0 == git rev-parse HEAD at tagging; a fresh worktree of the tag: 174 tests pass, `capy 0.26.0`,
    `examples/c/run.sh walk` PASS, `capy.h` up to date)
```

Commits, in order: `79dae85` proposal + ADR + plan (**before** the code), `114525b` feature, `d1cf161` chore(release),
`bb7c951` demo programs and the exact bound, `8e81cdd` docs(program) and `85b708b` first release document (both
**superseded**: the tag was moved before anything was pushed), `e096ba2` fix: over-deep expressions are reported by name
(`E0003`), `05ae277` docs(program) **← tagged**, then the finalize commit that updates this document.

## Plan

`PLAN-2026-0005`. Decision in `ADR-0005`. Proposal `PROP-2026-0008` revision 2.

## Project Standards Baseline

| Standards Index | Revision | Applicable Rules | Proposal Validation |
|---|---|---|---|
| `program_docs/standards/index.md` | 1 | GOAL-001, GOAL-002, PHIL-001, PHIL-002, CODE-001, ARCH-001, ARCH-002, ARCH-003, QUAL-001, QUAL-002, QUAL-003, GATE-001, GATE-002 | PASS at review — **with one correction found in execution**: the proposal's `ARCH-003` row was wrong (`INC-2026-0002`); see `RPT-2026-0005` D-3 |

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| PROP-2026-0008 R1, R2, R3 / UC-01, UC-02 | Owner: "i have a project here which want to use capy ffi … write a proposal to add missing features" (UQ-01, UQ-02); Capylar `TASK.md` line 16 | U-01, U-02 | PASS |
| R4, R5, R6, R7 / UC-03, UC-04, UC-05 | the same | U-03, U-04 | PASS |
| R8, R14 | misuse and hostile input must not corrupt the host | U-07 | PASS |
| R9 | no panic or stack overflow crosses the boundary | U-06 | PASS |
| R10, R11, R13 | deterministic, versioned, equal to the Rust view | U-08, U-09 | PASS |
| R12 / UC-06 | the typed expression must be reachable | U-05 | PASS |
| R15 | nothing existing changes | U-08 | **PARTIAL** (wasm +1,950 B; `INC-2026-0003`) |
| R16 | docs true | U-10 | PASS |
| R17 | predeclared measurements | `TEST-2026-0015` | PASS (two predictions missed) |
| "plan implement validate and release, follow documentation.md" | Owner, 2026-10-13 | all | PASS |

## Added

- **`capy-ffi`**: `rust/ffi/` — `libcapy_ffi.a`, `libcapy_ffi.dylib`/`.so`, `rust/ffi/include/capy.h` (generated, checked in).
  75 functions: library and load errors, parse, block, statement, capture, expression, diagnostic, error node, JSON.
  Parsing only. `docs/c-abi.md`.
- **`[profile.ffi]`**: `release` with `panic = "unwind"`; `capy_panics_are_caught()` reports whether a build can catch a panic.
- **`capy ast --json --with-expr`** and `ast_json::{expr_val, to_json_with, to_json_pretty_with, parse_result_val_with}`.
- `examples/c/` (`walk`, `diagnose`, `exprs`, `hostile`, `misuse`, `run.sh`), `rust/ffi/check_header.sh`, `rust/devtools/ast_json_compat.sh`.
- 21 tests (workspace 153 → 174).
- Program documents: `PROP-2026-0008`, `ADR-0005`, `PLAN-2026-0005`, `TEST-2026-0015`, `RPT-2026-0005`, `DEMO-2026-0005`,
  `MAN-2026-0003`, `ARCH-2026-0002`, `INC-2026-0002`, `INC-2026-0003`.

## Changed

- `Library::run` speed and memory work (`PROP-2026-0007`, `TEST-2026-0014`) — committed before this release under *Unreleased*, shipped here.
- `rust/devtools/check_publishable.py` also enforces `ARCH-002` (`capy-core` one dependency) and that `capy-ffi` is never published.
- `docs/embedding.md`, `docs/ast-json.md`, `docs/cli.md`, `docs/whats-new.md`, `README.md`, `CHANGELOG.md`, `SYS-2026-0001` (rev 5).
- Workspace version 0.25.0 → 0.26.0.

## Fixed

- **Deeply nested expressions aborted the process** (`INC-2026-0002`): bounded at 64 levels; 63 nested pairs of parentheses
  parse, the 64th is refused with `E0003` and a message that names the limit. This is a **behaviour change in `capy-core`** that the plan did not list: source nested
  between 64 and about 5 000 levels parsed before and is now refused.
- The punctuation-split token corruption (`PROP-2026-0007` E8), carried from *Unreleased*.

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP-2026-0008 R1 | C library and header exist | `cargo build --profile ffi -p capy-ffi` | `libcapy_ffi.a`/`.dylib`, `capy.h` | TEST-2026-0015 T-13, DEMO-2026-0005 U-01 |
| U-02 | R2, R3, R4 | complete C program | `examples/c/run.sh walk` | `walk: PASS (greet(who=world))` | T-01, DEMO U-02 |
| U-03 | R3, R6, R7, R8 | errors and diagnostics from C | `examples/c/run.sh diagnose` | status 7 + grammar message; `E0001 2:1`; NULL for a bad id | T-05, T-06, DEMO U-03 |
| U-04 | R5 | typed expression from C | `examples/c/run.sh exprs` | indented tree with a `var` path and an index | T-03, DEMO U-04 |
| U-05 | R12 | `--with-expr` | `capy ast … --json --with-expr` | `expr` present only with the flag | T-10, DEMO U-05 |
| U-06 | R9 / INC-2026-0002 | deep nesting survives | 20 000 `(` through `capy ast` | 0.25.0 rc=134; 0.26.0 rc=1 | T-11, DEMO U-06 |
| U-07 | R8, R14 | misuse and hostile input | `examples/c/run.sh misuse && … hostile` | 76 and 30 checks, 0 failed | T-05, T-07, DEMO U-07 |
| U-08 | R15 | default JSON unchanged | `rust/devtools/ast_json_compat.sh <0.25.0 capy>` | 133 scripts, 0 differences | T-10, DEMO U-08 |
| U-09 | R11 | header cannot drift | `rust/ffi/check_header.sh` | `capy.h is up to date` | T-09, DEMO U-09 |
| U-10 | R16 | docs build | `mkdocs build --strict` | exit 0 | T-13, DEMO U-10 |
| U-11 | PROP-2026-0007 | faster `run` | — | `NOT DIRECTLY USER-TESTABLE` here | TEST-2026-0014 |

## Tests

| Test | Requirement | Result |
|---|---|---|
| TEST-2026-0015 T-01 … T-11, T-13 | PLAN-2026-0005 R1 … R17 | PASS |
| TEST-2026-0015 T-12 | R1, R15, R17 | PARTIAL (Linux cross-built, not run; wasm size not equal) |
| TEST-2026-0015 T-14 | `GATE-002` | PARTIAL (`wasm_check.sh` fails on a pre-existing case, `INC-2026-0003`) |

Gate summary on the tagged tree: `cargo test --workspace` **174 passed, 0 failed** · `cargo clippy --workspace --all-targets -- -D warnings`
and `--profile ffi` clean · golden ok · `capy check`: 122 of 123 sample libraries ok (the one rejection is the sample that demonstrates a
rejection) · `mkdocs build --strict` exit 0 · C suites 76 + 30 checks under ASan + UBSan · header up to date. Ten mutation checks; two tests were
too weak and were strengthened (`TEST-2026-0015`).

## Validation

`RPT-2026-0005`: R1–R17 — 16 `PASS`, 1 `PARTIAL` (R15); UC-01–UC-07 all `PASS`. The `PARTIAL` is the wasm size and the pre-existing `wasm_check.sh`
case.

## Measured Results

| Update | Requirement | Metric | Baseline | Expected Threshold | Actual | Result | Test Report and Source |
|---|---|---|---|---|---|---|---|
| U-01 | PLAN-2026-0005 M-01 | static archive size | 5,154,280 B (`capy-core`, no exports) | ≤ 5.5 MB | **31,225,392 B** (linked program 1.3 MB with dead-strip) | **FAIL** | TEST-2026-0015, `stat` |
| U-02 | M-02 | arena overhead over `Library::parse` | 132 scripts × 20 passes, 15 interleaved | ≤ +25 % | median +7.1 … +8.1 % | PASS | TEST-2026-0015, `ffi/examples/bench.rs` |
| U-04 | M-03 | per-accessor cost | none | recorded | 4.63 – 4.68 ns | recorded | TEST-2026-0015 |
| U-06 | M-04 | wasm size | 1,434,097 B | equal | **1,436,047 B (+1,950)** — all from the expression depth bound and its error report | **FAIL** | TEST-2026-0015, `ls -l` |
| U-01 | M-05 | `capy-core` direct dependencies | 1 | exactly 1 | 1 (`regex`) | PASS | TEST-2026-0015, `cargo tree` |

The release makes **no claim** that the archive is small or that the wasm module is unchanged.

## Incidents

| Incident | Status |
|---|---|
| `INC-2026-0002` deeply nested expressions overflow the stack | resolved in this release |
| `INC-2026-0003` `wasm_check.sh` fails on `parse-recovery/broken.capy` | **open**, pre-existing (identical on `69fb4ac`), low severity |

## Troubleshooting

No new `TRBL` document. `TRBL-2026-0001` (stale `capy` on PATH) applies unchanged and is referenced by the demo.

## Release Verification Guide and Demo

`DEMO-2026-0005` — 11 `U-NN` rows; 10 executed against the release tree (all PASS); U-11 `NOT DIRECTLY USER-TESTABLE`.

## Manual

`MAN-2026-0003` (new): calling Capy from C.

## System

`SYS-2026-0001` revision 5 (expression recursion bound; the C ABI layer); `ARCH-2026-0002` (new).

## Documentation Impact

| Artifact | Decision | Updated Document or Reason |
|---|---|---|
| Release verification guide / demo | UPDATED | DEMO-2026-0005 |
| README.md | UPDATED | new "Embed from C" section |
| System documentation | UPDATED | SYS-2026-0001 |
| Architecture documentation | UPDATED | ARCH-2026-0002 |
| API and CLI reference | UPDATED | `docs/c-abi.md`, `docs/ast-json.md`, `docs/cli.md`, `docs/embedding.md` |
| Manual | UPDATED | MAN-2026-0003 |

## Source Changes

| Source | Change | Reason |
|---|---|---|
| `rust/ffi/` (`Cargo.toml`, `src/{lib,arena,diag,expr,json}.rs`, `include/capy.h`, `cbindgen.toml`, `check_header.sh`, `tests/`, `examples/bench.rs`) | Created | the C ABI |
| `rust/Cargo.toml`, `Cargo.lock`, six member manifests | Modified | member `ffi`, `[profile.ffi]`, version 0.26.0 |
| `rust/src/domain/ast_json.rs` | Modified | opt-in `expr` writer |
| `rust/cli/src/cmd_ast.rs`, `rust/cli/tests/ast_with_expr.rs` | Modified / Created | `--with-expr` |
| `rust/src/orchestrator/features/value_parser.rs` | Modified | expression depth bound (`INC-2026-0002`) |
| `rust/tests/recursion_guard.rs` | Modified | 4 expression-nesting tests |
| `rust/devtools/check_publishable.py`, `ast_json_compat.sh` | Modified / Created | `ARCH-002` and compatibility checks |
| `examples/c/` | Created | C programs and the test runner |
| `docs/c-abi.md` and five other pages, `mkdocs.yml`, `CHANGELOG.md`, `README.md` | Created / Modified | user documentation |

## Known Issues

1. **`INC-2026-0003`**: `rust/devtools/wasm_check.sh` fails on one case; identical on the previous release. `GATE-002` therefore does not read PASS.
2. **The static archive is 31 MB** on macOS arm64 (9.4 MB for the Linux cross-build; the difference is unexplained). Linked programs are about 1.3 MB with dead-stripping.
3. **The wasm module is 1,436,047 B**, over the 1,410,035 B ceiling recorded in `PLAN-2026-0004`; it was already over it (1,434,097 B) before this release because of the `PROP-2026-0007` work (its OQ-03, unanswered).

## Limitations

- Only macOS arm64 was run; Linux was cross-built only; Windows is untested.
- Parsing only: `run`, `introspect` and `set_host` are not exposed; no byte offsets; no spans on expressions.
- Foreign-id detection is best effort (an 8-bit tag); use after free and double free cannot be detected.
- Rust-side `unsafe` is not run under a sanitizer or Miri.
- `Call` expressions are not produced by any grammar today; they are tested through hand-built trees.

## Follow-up Work

| Item | Destination |
|---|---|
| Fix or explain `wasm_check.sh` (`INC-2026-0003`) | a defect proposal |
| Owner decision on the wasm size ceiling and on the 64-level expression bound | owner |
| Explain the macOS/Linux archive size difference | optional |
| Byte offsets in `Span`; exposing `run` | separate proposals (`PROP-2026-0008` OQ-03, OQ-04) |
| Canonical repository name (`olivierdevelops/capy` vs `luowensheng/capy`) | owner (`PROP-2026-0008` OQ-01) |
| Run the C suites on Linux | when a host exists |
| `PROP-2026-0003`, `PROP-2026-0006`, `PROP-2026-0007` remain drafts | owner |

## Deviations from the Plan and Proposal

Listed in full in `RPT-2026-0005` §Deviations (D-1 … D-10). The ones that matter:

1. **A `capy-core` behaviour change the plan did not list**: the expression depth bound (D-2).
2. **`PROP-2026-0008`'s `ARCH-003` assessment was wrong** and is corrected (D-3).
3. **Two frozen predictions were missed** (D-8).
4. **Two tests were strengthened after a mutation survived** (D-7).
5. **Nothing was pushed** (D-9).

## Upgrade Notes

Nothing to do for existing users. To use the C ABI: `cargo build --profile ffi -p capy-ffi`, include `capy.h`, link with
dead-stripping, and check `capy_abi_version()`. Source nested between 64 and about 5 000 levels of parentheses will now be
refused where it parsed before.

## Related Documents

- `PLAN-2026-0005`, `PROP-2026-0008`, `ADR-0005`, `TEST-2026-0015`, `RPT-2026-0005`, `DEMO-2026-0005`, `MAN-2026-0003`, `SYS-2026-0001`,
  `ARCH-2026-0002`, `INC-2026-0002`, `INC-2026-0003`, `REL-0.24.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial release document, created after the tag |
