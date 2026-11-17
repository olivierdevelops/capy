---
document_id: REL-0.27.1
title: Release 0.27.1 — a backtick in a comment or string is text
document_type: release
status: completed

created_date: 2026-11-17
last_updated: 2026-11-17
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

version: "0.27.1"
git_tag: v0.27.1
code_revision: 46dcf9037e568e9f53a17f5c71ba9656b4d74867
git_commit: 46dcf9037e568e9f53a17f5c71ba9656b4d74867
release_date: 2026-11-17
source_branch: main
previous_version: "0.27.0"
previous_tag: v0.27.0

affected_versions:
  from: "0.27.1"
  to: "0.27.1"

applicable_environments:
  - development

audience:
  - engineers
  - release-managers

scope: Records what shipped in 0.27.1, how it was verified, and the exact source state it was built from.

reason: DOCUMENTATION.md section 33 requires a release document tying the release to its plan, tests and Git tag.

related_documents:
  - PLAN-2026-0007
  - INC-2026-0004
  - REL-0.27.0

supersedes: null
superseded_by: null

tags:
  - release
  - lexer
  - bugfix

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-02-17
---

# Release 0.27.1 — a backtick in a comment or string is text

> **Status:** Completed
> **Created:** 2026-11-17
> **Last Updated:** 2026-11-17
> **Affected Versions:** 0.27.1
> **Owner:** Release Management
> **Affected Components:** capy-core, docs

## Summary

A patch release. One unpaired backtick in a comment (or a string) no longer swallows the rest of the file.

```text
  0.27.0                                    0.27.1
  ────────────────────────────────────────  ─────────────────────────────────────────
  // use ` here   -> rest of file dropped   a comment is a comment; code after it parses
  say "a ` b"     -> no function matches   a string is a string
  say `never closed -> unterminated string  line N: backtick literal opened here is never closed
```

## Release Identity

| Field | Value |
|---|---|
| Release Version | 0.27.1 |
| Git Tag | `v0.27.1` (to be created on the commit that carries this document) |
| Code Revision | `46dcf9037e568e9f53a17f5c71ba9656b4d74867` (commit B: version bump and changelog) |
| Release Date | 2026-11-17 |
| Source Branch | `main` |
| Previous Version / Tag | 0.27.0 / `v0.27.0` |
| Repository | not pushed |
| Version source | `rust/Cargo.toml` `[workspace.package] version`, six member `capy-core` pins, `rust/Cargo.lock`; `capy version` prints `capy 0.27.1` |

A tag cannot name a commit that contains its own hash, so this document records B and the tag points at the
document commit R above it.

```text
   da31b25  PLAN-2026-0007  ──┐         B = code_revision (this document)
   71b7d0d  fix(lexer)        ├──►  46dcf90  chore(release): 0.27.1
                              ┘                    │
                                                   ▼
                                       R  docs(release): REL-0.27.1   ◄── tag v0.27.1
```

## Plan

`PLAN-2026-0007`.

## Project Standards Baseline

`program_docs/standards/index.md` revision 1. No helper or capture type changed.

## User Requirements

| Requirement | Source | Released Update | Result |
|---|---|---|---|
| A backtick in a comment or string must not drop the rest of the file; additive | Owner, 2026-11-17 (downstream Capyler incident) | U-01 to U-04 | PASS |

## Added

- `samples/backtick-in-comment/`, `backtick-in-string/`, `backtick-multiline/`, `backtick-unclosed/`.
- Error `line N: backtick literal opened here is never closed`.

## Changed

- Workspace version 0.27.0 to 0.27.1.

## Fixed

- `merge_backtick_lines` counts only backticks that are code; comments use the library's `comments` markers, strings the lexer's `"` and `'`. See `INC-2026-0004`.

## Removed

Nothing.

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | comment | backtick in comment is text | `capy run samples/backtick-in-comment/lib.capy samples/backtick-in-comment/script.capy` | four `note:`/`say:` lines | golden |
| U-02 | string | backtick in string is text | same, `backtick-in-string` | four lines | golden |
| U-03 | additive | real multi-line literal | same, `backtick-multiline` | literal keeps its newlines | golden |
| U-04 | positioned error | unclosed literal | same, `backtick-unclosed` | `line 2: backtick literal opened here is never closed` | golden |

## Tests

`cargo test --workspace`: all pass, 0 failed; every pre-existing golden is unchanged.
Gates on B: `cargo build --workspace` ok; `cargo clippy --workspace --all-targets -- -D warnings` clean; `mkdocs build --strict` exit 0.

## Validation

No separate validation report (patch release). R1 to R5 of `PLAN-2026-0007` are met by the goldens.

## Measured Results

No performance claim. Lines with no backtick still take the borrowed fast path. NOT MEASURED.

## Incidents

`INC-2026-0004` (resolved here).

## Troubleshooting

No new document.

## Release Verification Guide and Demo

No separate demo (patch release); the four rows above are the guide.

## Manual

None.

## System

No change to `SYS-2026-0001` or the architecture documents.

## Documentation Impact

| Artifact | Decision | Updated Document or Reason |
|---|---|---|
| Release verification guide / demo | NOT APPLICABLE | patch release; rows above |
| README.md | NOT APPLICABLE | no headline capability |
| System documentation | NOT APPLICABLE | no new component |
| Architecture documentation | NOT APPLICABLE | no new component |
| API and CLI reference | UPDATED | `docs/whats-new.md`, `samples/README.md`; `library-keywords.md` NOT APPLICABLE (no directive changed) |
| Manual | NOT APPLICABLE | none |

## Source Changes

| Source | Change | Reason |
|---|---|---|
| `rust/src/orchestrator/features/make_lexer.rs` | Modified | scan comments and strings before counting backticks |
| `samples/backtick-*` | Created | goldens |
| `rust/**/Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` | Modified | 0.27.1 |

## Known Issues

The library-file parser's own function-body backtick handling was not changed. `INC-2026-0003` was not re-run.

## Limitations

String delimiters are the lexer's `"` and `'`, not library-declared.

## Follow-up Work

| Item | Destination |
|---|---|
| Tag and push | orchestrating session |

## Deviations from the Plan and Proposal

None.

## Upgrade Notes

Nothing to do. A script that relied on a backtick inside a string or comment toggling a literal (none known) changes.

## Related Documents

- `PLAN-2026-0007`, `INC-2026-0004`, `REL-0.27.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-11-17 | Olivier, Claude | Initial release document |
