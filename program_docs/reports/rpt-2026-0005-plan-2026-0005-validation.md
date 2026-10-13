---
document_id: RPT-2026-0005
title: Validation of PLAN-2026-0005 — A C ABI for Capy (0.26.0)
document_type: report
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

scope: Validates the 0.26.0 implementation against PLAN-2026-0005 requirement by requirement.

reason: DOCUMENTATION.md section 28 requires an explicit validation of the implementation against the plan before the release.

related_documents:
  - PLAN-2026-0005
  - PROP-2026-0008
  - ADR-0005
  - TEST-2026-0015
  - INC-2026-0002
  - INC-2026-0003
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - validation
  - ffi
  - c-abi

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-13
---

# Validation of PLAN-2026-0005 — A C ABI for Capy (0.26.0)

> **Status:** Completed
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Summary

```text
  R1 … R17    16 PASS   1 PARTIAL (R15)   0 FAIL          UC-01 … UC-07   7 PASS
  Measured claims (TEST-2026-0015):   M-02 PASS · M-03 recorded · M-05 PASS · M-01 FAIL · M-04 FAIL
  Unresolved incidents: INC-2026-0003 (pre-existing, open)      Resolved: INC-2026-0002
```

The C ABI is implemented, differentially verified against the Rust writer over the whole sample corpus, and
exercised under ASan and UBSan. Two frozen measurement predictions were wrong (archive size, wasm size) and one
defect in `capy-core` (`INC-2026-0002`) was found and fixed in scope because the ABI's "no input aborts the host"
claim could not otherwise be made. That fix is a behaviour change to `capy-core` that the plan did not list; it is
called out under *Deviations*.

## Plan Under Validation

`PLAN-2026-0005` (approved 2026-10-13), decision `ADR-0005`, proposal `PROP-2026-0008` revision 1. Implementation
commits: `114525b` (feature), `d1cf161` (version).

## Method

Executed every row of the plan's test checklist (`TEST-2026-0015`), compared the working tree with the baseline
`69fb4ac` (JSON byte comparison over 133 scripts; wasm gate on a clean worktree), and mutation-checked each test.
Observations are from the commands in `TEST-2026-0015` §Procedure.

## Requirement Results

| Req | Result | Evidence |
|---|---|---|
| R1 crate, one dependency | **PASS** | `cargo tree -p capy-core --depth 1` = `regex`; `capy-ffi` = `capy-core`; `check_publishable.py` enforces both |
| R2 library handle, shared read-only | **PASS** | T-04 (4 threads × 500 parses); `capy_library_new` / `_from_file` / `_free` |
| R3 parse never tree-less | **PASS** | T-05 (30 hostile checks); `capy_parse` returns a handle or a non-OK status |
| R4 full statement/capture/error-node access | **PASS** | T-02: C-walked document == Rust JSON for 132 scripts |
| R5 all expression kinds | **PASS** | T-03: 12 kinds, 3 000 random trees, integer edges |
| R6 diagnostics incl. labels, context | **PASS** | T-02 (10 scripts with diagnostics; the walker composes `message` from `msg` + last context frame) |
| R7 load error object | **PASS** | T-06 |
| R8 ids and handles validated | **PASS** | T-07: 76 checks under ASan + UBSan; limitation: the foreign-id tag is best effort (`ADR-0005` deviation 4) |
| R9 no panic crosses | **PASS** | T-08 (accessors, load, parse; mutation M6); T-11 (stack-overflow class closed for expressions) |
| R10 deterministic order | **PASS** | T-02 asserts ascending capture/section names (added after mutation M1 survived) |
| R11 ABI version, header drift | **PASS** | T-09; `CAPY_ABI_VERSION` = 1 |
| R12 opt-in `expr` JSON | **PASS** | T-10: 133 scripts byte-identical by default |
| R13 differential equivalence | **PASS** | T-02 |
| R14 memory safety verified | **PASS** | T-07; each unsafe dereference carries a SAFETY note. Limitation: the **Rust** side is not run under a sanitizer or Miri (nightly), only the C side |
| R15 nothing existing changes | **PARTIAL** | `GATE-001` PASS; golden PASS; default JSON identical; **but** wasm +1,216 B (M-04) and `wasm_check.sh` FAIL — the latter identical on the baseline (`INC-2026-0003`) |
| R16 docs true | **PASS** | `mkdocs build --strict` exit 0; every export appears in `docs/c-abi.md` |
| R17 measurements predeclared and run | **PASS** | M-01…M-05 recorded; two predictions missed and reported as such |

### Use cases

| UC | Result | Evidence |
|---|---|---|
| UC-01 compile once, keep it | PASS | T-01, T-04, T-06 |
| UC-02 parse and learn if clean | PASS | T-01, T-05, T-08 |
| UC-03 walk statements and captures | PASS | T-02 |
| UC-04 read an expression | PASS | T-03 |
| UC-05 report diagnostics | PASS | T-02 |
| UC-06 JSON dump | PASS | T-02 (`capy_parse_to_json`), T-10 (CLI) |
| UC-07 detect a mismatched library | PASS | T-09; `misuse.c` checks `capy_abi_version()` and `capy_panics_are_caught()` |

### Measurements

| ID | Result | Note |
|---|---|---|
| M-01 archive ≤ 5.5 MB | **FAIL** | 31,225,392 B. The prediction used a no-export baseline. A linked program is 1.3 MB with dead-stripping; the docs say so |
| M-02 overhead ≤ +25 % | PASS | +7.1 … +8.1 % median; one session just below the predicted range |
| M-03 accessor cost | recorded | 4.63 – 4.68 ns |
| M-04 wasm equal | **FAIL** | +1,216 B, all from the depth guard (`INC-2026-0002`) |
| M-05 one dependency | PASS | |

## Deviations From the Plan

| # | Deviation | Impact | Plan updated? |
|---|---|---|---|
| D-1 | `CAPY_E_LOAD`, `CAPY_E_LIMIT` (instead of `CAPY_E_OOM`), `rlib` crate type, best-effort foreign-id tag, cross-build as the second platform, message composition | none beyond the interface | frozen in `ADR-0005` before code |
| D-2 | **A `capy-core` behaviour change not in the plan:** expression nesting is now bounded at 64 levels (`INC-2026-0002`) | source nested 64 … ~5 000 levels parsed before and is now refused. No sample, document or test approached it. Strictly, `GOAL-002` ("engine changes are additive") is stretched: this removes acceptance of inputs that were already one step from a crash. The owner should confirm the limit | recorded here and in `CHANGELOG.md` |
| D-3 | `PROP-2026-0008`'s `ARCH-003` row said `PASS` on a false premise | corrected by this report | proposal revision 2 |
| D-4 | Names and functions the proposal's appendix did not list: `capy_parse_is_poisoned`, `capy_obj_key_count`, `capy_node_section_at`, the concrete `capy_err_*` set, the `CapyId` typedef (`capy_panics_are_caught` was added by `ADR-0005` D3) | additive; 75 functions | header is the authority |
| D-5 | `Call` expressions are never produced by an `any` capture, so T-03 builds expressions by hand | `Call` is tested at the ABI, not through a grammar | none |
| D-6 | T-12 second platform is a cross-**build**, not a run | Linux behaviour untested | recorded |
| D-7 | Two tests were too weak (M1, M3b) and were strengthened after mutation | none; recorded in `TEST-2026-0015` | none |
| D-8 | M-01 and M-04 missed their frozen thresholds | reported, not edited | none |
| D-9 | No push to a remote | the owner asked for a release, not a push | `REL-0.26.0` |
| D-10 | The previous wasm ceiling (1,410,035 B, `PLAN-2026-0004`) was already exceeded at the baseline (1,434,097 B) by the `PROP-2026-0007` work (its OQ-03, unanswered); this release adds 1,216 B more | owner decision pending | follow-up |

## Unintended Behaviour

None found beyond the two measurement misses and the generic `E0001` message for over-deep expressions (below).

## Unresolved Incidents

`INC-2026-0003` — `wasm_check.sh` mismatch, pre-existing, low severity.

## Remaining Limitations

1. The static archive is large (31 MB on macOS); link with dead-stripping.
2. Over-deep expressions are reported with the generic `E0001`, not a named bound.
3. Foreign-id detection is best effort; use after free and double free are undetectable.
4. Only macOS arm64 was run. Linux is built, not run; Windows is untested.
5. Rust-side `unsafe` is not sanitizer-instrumented.
6. No byte offsets in spans; no spans on `Expr`; parsing only (no `run`, `introspect`, `set_host`).
7. `Call` is not produced by any grammar today.
8. `ParseResult` equality of the C walker covers the AST JSON's information; `Token.kind` and per-token `width` are not exposed (JSON omits them too).

## Required Follow-Up

| Item | Destination |
|---|---|
| Name the bound in the over-deep expression message (`E0003`-style) | next release touching the parser |
| Fix or explain the `wasm_check.sh` case (`INC-2026-0003`) | a defect proposal |
| Owner decision on the wasm size ceiling (`PROP-2026-0007` OQ-03) | owner |
| Explain the 3x macOS/Linux archive size difference | optional |
| Byte offsets in `Span`; exposing `run` | separate proposals (`PROP-2026-0008` OQ-03, OQ-04) |
| Run the C suites on Linux | when a Linux host exists |

## Questions Required by DOCUMENTATION.md Section 28

| Question | Answer |
|---|---|
| Was the planned functionality actually implemented? | Yes: 75 exported functions, the crate, the profile, the header, the JSON option, the CLI flag, docs |
| Does it behave as expected? | Yes, by differential comparison over 132 scripts and 76 + 30 C checks; see limitations |
| Were all expected files and components modified? | Yes; the plan's file checklist was followed. Extra: `value_parser.rs` (D-2), `check_publishable.py` |
| Did implementation introduce unintended behaviour? | One deliberate behaviour change (D-2); nothing unintended found |
| Are there remaining limitations? | Yes, eight, listed above |
| Are there unresolved incidents? | `INC-2026-0003` (pre-existing) |
| Are follow-up changes required? | Yes, listed above |

## Files Changed

`rust/ffi/**` (new), `examples/c/**` (new), `rust/Cargo.toml` and `Cargo.lock`, `rust/cli/src/cmd_ast.rs`,
`rust/cli/tests/ast_with_expr.rs`, `rust/src/domain/ast_json.rs`, `rust/src/orchestrator/features/value_parser.rs`,
`rust/tests/recursion_guard.rs`, `rust/devtools/check_publishable.py`, `rust/devtools/ast_json_compat.sh`,
seven manifests (version), `docs/c-abi.md` and five other user pages, `mkdocs.yml`, `CHANGELOG.md`, `README.md`,
and `program_docs/**`.

## Conclusion

The release can proceed. Two things for the owner to decide, neither blocking: whether the 64-level expression
bound is acceptable (D-2), and what to do about the wasm size ceiling (D-10).

## Related Documents

- `PLAN-2026-0005`, `PROP-2026-0008`, `ADR-0005`, `TEST-2026-0015`, `INC-2026-0002`, `INC-2026-0003`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial validation |
