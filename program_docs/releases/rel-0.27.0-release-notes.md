---
document_id: REL-0.27.0
title: Release 0.27.0 — } else { on one line
document_type: release
status: completed

created_date: 2026-11-16
last_updated: 2026-11-16
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
  - capy-core
  - docs

version: "0.27.0"
git_tag: v0.27.0
code_revision: 8b351afcefdb18b942c30b4b0c8c1daa6e8676ee
git_commit: 8b351afcefdb18b942c30b4b0c8c1daa6e8676ee
release_date: 2026-11-16
source_branch: main
previous_version: "0.26.0"
previous_tag: v0.26.0

affected_versions:
  from: "0.27.0"
  to: "0.27.0"

applicable_environments:
  - development

audience:
  - engineers
  - release-managers

scope: Records what shipped in 0.27.0, how it was verified, and the exact source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests and Git tag.

related_documents:
  - PLAN-2026-0006
  - PLAN-2026-0005
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - release
  - parser
  - block

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-02-16
---

# Release 0.27.0 — } else { on one line

> **Status:** Completed
> **Created:** 2026-11-16
> **Last Updated:** 2026-11-16
> **Affected Versions:** 0.27.0
> **Owner:** Release Management
> **Affected Components:** capy-core, docs

## Summary

A library can now let a delimiter block be followed, on the same line, by a statement that starts with a
literal it names. The release also carries two parser fixes committed after 0.26.0.

```text
  0.26.0                                   0.27.0
  ───────────────────────────────────────  ──────────────────────────────────────────────
  } else {   -> expected newline after     block_open "{" close "}" continue "else"
               block close                   } else {  parses like else on its own line
  a-1        -> syntax error               a-1 subtracts ("a -1" is still a sign)
  n * f(n-1) -> flat chain                 f(args) is an expression operand
```

## Release Identity

| Field | Value |
|---|---|
| Release Version | 0.27.0 |
| Git Tag | `v0.27.0` (to be created on the commit that carries this document) |
| Code Revision | `8b351afcefdb18b942c30b4b0c8c1daa6e8676ee` (commit B: version bump and changelog) |
| Release Date | 2026-11-16 |
| Source Branch | `main` |
| Previous Version / Tag | 0.26.0 / `v0.26.0` |
| Repository | not pushed |
| Version source | `rust/Cargo.toml` `[workspace.package] version`, six member `capy-core` pins, `rust/Cargo.lock`; `capy version` prints `capy 0.27.0` |

A tag cannot name a commit that contains its own hash, so this document records B and the tag points at the
document commit R above it.

```text
   461d5fd  a-1 / f(args)  ──┐
   3654c8e  PLAN-2026-0006   │            B = code_revision (this document)
   4bc0b7f  feature          ├──►  8b351af  chore(release): 0.27.0
   e448c68  chore            ┘                    │
                                                  ▼
                                      R  docs(release): REL-0.27.0   ◄── tag v0.27.0
```

## Plan

`PLAN-2026-0006`.

## Project Standards Baseline

`program_docs/standards/index.md` revision 1; the applicable rules are those of `PLAN-2026-0005` (no new helper or capture type).

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| A library can write `} else {` on one line; additive | Owner, 2026-11-16 (downstream library needs it) | U-01, U-02 | PASS |
| Plan, change and a quick release | Owner, 2026-11-16 | all | PASS |

## Added

- `block_open "{" close "}" continue "else" ...`: literals that may start the next statement on the closer's line.
- `samples/block-continue/` (two scripts, one expected output) and `samples/block-continue-rejected/` (the old error).
- From `461d5fd`: `f(args)` is an expression operand (`rust/tests/call_operand.rs`).

## Changed

- Workspace version 0.26.0 to 0.27.0. `docs/roadmap.md` marks the continuation item done.
- A `collapsible_match` in error recovery (`make_parser.rs`) was folded into the match guard so clippy 1.99 passes; behaviour is identical.

## Fixed

- From `461d5fd`: `a-1`, `f(x)-1`, `xs[0]-1` and `"s"-1` parse as subtraction (`rust/tests/minus_sign.rs`).

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | same-line `else` | opt-in directive | `capy run samples/block-continue/lib.capy samples/block-continue/same_line.capy` | output equal to `own_line.expected.txt` | golden `block-continue/same_line` |
| U-02 | additive | no opt-in, no change | `capy run samples/block-continue-rejected/lib.capy samples/block-continue-rejected/same_line.capy` | `line 3: expected newline after block close` | golden `block-continue-rejected/same_line` |
| U-03 | same tree | `capy ast` on both forms | `capy ast samples/block-continue/lib.capy samples/block-continue/own_line.capy` (and `same_line`) | `if` then `else`, same children; only the `else` column differs | manual run at B |

## Tests

| Test | Requirement | Result |
|---|---|---|
| `cargo test --workspace` | all | 186 passed, 0 failed |
| golden `block-continue/*`, `block-continue-rejected/*` | U-01, U-02 | PASS |

Gates on B: `cargo build --workspace` ok; `cargo clippy --workspace --all-targets -- -D warnings` clean;
`mkdocs build --strict` exit 0.

## Validation

No separate validation report (quick release). Plan requirements R1 to R5 of `PLAN-2026-0006` are met by the tests above.

## Measured Results

No performance claim is made. The new branch runs only after a delimiter block has already closed and only when the library set `continue`. NOT MEASURED.

## Incidents

None.

## Troubleshooting

No new document.

## Release Verification Guide and Demo

No separate demo document (quick release); the three rows under Released Updates are the guide.

## Manual

`docs/library-keywords.md` and `docs/features.md` describe the directive.

## System

No change to `SYS-2026-0001` or the architecture documents (no component added).

## Documentation Impact

| Artifact | Decision | Updated Document or Reason |
|---|---|---|
| Release verification guide / demo | NOT APPLICABLE | quick release; rows above |
| README.md | NOT APPLICABLE | no headline capability |
| System documentation | NOT APPLICABLE | no new component |
| Architecture documentation | NOT APPLICABLE | no new component |
| API and CLI reference | UPDATED | `docs/library-keywords.md`, `features.md`, `CAPY_FOR_LLMS.md`, `library-authoring.md`, `whats-new.md`, `roadmap.md` |
| Manual | NOT APPLICABLE | covered by the reference pages |

## Source Changes

| Source | Change | Reason |
|---|---|---|
| `rust/src/infra/capy_lib_parser.rs`, `raw_library.rs` | Modified | parse `continue` |
| `rust/src/domain/library.rs`, `orchestrator/features/make_library_loader.rs` | Modified | carry `continue_with` |
| `rust/src/orchestrator/features/make_parser.rs` | Modified | accept the continuation; clippy fold |
| `samples/block-continue*` | Created | goldens |
| `rust/**/Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` | Modified | 0.27.0 |

## Known Issues

None new. `INC-2026-0003` (`wasm_check.sh`) from 0.26.0 was not re-run.

## Limitations

- Each continuation literal is one token.
- Only delimiter-mode blocks; a statement after `}` that is not in the list still errors.
- `else` is not checked to follow an `if`; the library's own rules decide that.

## Follow-up Work

| Item | Destination |
|---|---|
| `else` arm on inner `if` (roadmap) | separate proposal |
| Tag and push | orchestrating session |

## Deviations from the Plan and Proposal

The plan commit `3654c8e` was made after the code was drafted but before it was committed (recorded in `PLAN-2026-0006`).

## Upgrade Notes

Nothing to do. Libraries without `continue` behave exactly as before.

## Related Documents

- `PLAN-2026-0006`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-11-16 | Olivier, Claude | Initial release document |
