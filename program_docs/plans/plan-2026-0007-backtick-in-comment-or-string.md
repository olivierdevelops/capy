---
document_id: PLAN-2026-0007
title: Implementation Plan — A backtick in a comment or string no longer opens a literal (0.27.1)
document_type: plan
status: completed

created_date: 2026-11-17
last_updated: 2026-11-17
document_revision: 1

approval:
  approved_by:
    - Olivier
  approved_date: 2026-11-17

authors:
  - Olivier
  - Claude (Sonnet 5.5)

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - docs

affected_versions:
  from: "0.27.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Fix the lexer pre-pass that merges multi-line backtick literals so only a backtick that is code counts; report an unclosed literal with a position.

reason: The downstream Capyler compiler lost the rest of a file to one unpaired backtick in a comment (its INC-2026-0001). The owner authorized the fix on 2026-11-17.

related_documents:
  - INC-2026-0004
  - PLAN-2026-0006
  - REL-0.27.0

supersedes: null
superseded_by: null

tags:
  - lexer
  - bugfix
  - release

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-05-17
---

# Implementation Plan — A backtick in a comment or string no longer opens a literal (0.27.1)

> **Status:** Completed
> **Created:** 2026-11-17
> **Last Updated:** 2026-11-17
> **Affected Versions:** 0.27.1
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs

## Summary

```text
  P0 plan ─► P1 reorder the merge scan ─► P2 four samples ─► P3 docs + incident ─► P4 gates ─► P5 release B, R
                                                                                                 (no tag, no push)
```

## Objective, Scope and Baseline

**Problem.** `merge_backtick_lines` (`rust/src/orchestrator/features/make_lexer.rs`) joins a multi-line
backtick literal into one logical line BEFORE tokenizing, and flips "inside a literal" on EVERY backtick,
including those in a comment or a string. One unpaired backtick in a comment swallows the rest of the file
into one comment token. The parse is clean; everything after it is silently ignored.

```text
   source                                    old pre-pass                   tokenizer then sees
   ──────                                    ────────────                   ───────────────────
   // use ` for code          ── ` opens ──► literal never closes    ──►   ONE line: "// use ` for code\nnote a\n…"
   note a                                    every later line merged        = one comment token
   note b                                                                   notes a and b never parsed
```

**Goal.** Recognise comments and strings first, so only a code backtick counts.
**Additive:** for every input where each backtick was already code, the output is byte-identical.

**Not owned:** the library-file parser's own function-body backtick handling (`capy_lib_parser.rs`).

## Design

The pass scans the line left to right in the lexer's own order. The comment markers are the library's
(`lib.comments`, already passed to the tokenizer); the string delimiters are the ones `tokenize_line`
recognises (`"` and `'`). Nothing is hard-coded to a target.

```text
  outside a literal, at each byte:
        comment marker?  ──yes──► copy the rest of the line verbatim, next line
              │ no
        quote " or ' ?   ──yes──► copy to the matching quote (\ escapes), carry on
              │ no
        backtick?        ──yes──► open literal, remember the line   (code backtick)
              │ no
        copy byte
  inside a literal: unchanged (a backslash escapes the next byte; a backtick closes)

  end of input still inside a literal ──► Err "line N: backtick literal opened here is never closed"
```

| Option | Verdict | Why |
|---|---|---|
| scan order: comment, string, then backtick | **chosen** | an ordering fix; no special case |
| tell comments apart after tokenizing | rejected | the merge exists to run BEFORE tokenizing |
| ignore backticks in comments only | rejected | `"a ` b"` is the same defect |

## Requirements and Acceptance

| Req | Outcome | Acceptance | Status |
|---|---|---|---|
| R1 | a backtick in a comment is text | `samples/backtick-in-comment`: code after it still parses | DONE |
| R2 | a backtick in a string is text | `samples/backtick-in-string` | DONE |
| R3 | real multi-line literals unchanged | `samples/backtick-multiline` | DONE |
| R4 | unclosed literal is a positioned error | `samples/backtick-unclosed`: `line 2: backtick literal opened here is never closed` | DONE |
| R5 | additive | every existing golden and `cargo test --workspace` unchanged | DONE |

## Work Checklist

| ID | Task | Files | Status |
|---|---|---|---|
| W-01 | comment/string-aware merge, unclosed error | `rust/src/orchestrator/features/make_lexer.rs` | DONE |
| W-02 | four samples | `samples/backtick-*` | DONE |
| W-03 | docs, incident | `docs/whats-new.md`, `samples/README.md`, `CHANGELOG.md`, `INC-2026-0004` | DONE |
| W-04 | gates | — | DONE |
| W-05 | version 0.27.1 (commit B), release document (commit R) | `rust/**/Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `program_docs/**` | see REL-0.27.1 |

## Project Facts

No `vhco-contract.json` here. No library directive or capture type changes, so `docs/library-keywords.md`
is not touched.

## Version and Release

Patch, 0.27.0 to 0.27.1 (a fix). Commits: plan, fix, B (version + changelog), R (release document naming B).
Tag and push are not performed here.

## Risks

| Risk | Mitigation |
|---|---|
| a script that relied on a backtick in a string toggling a literal | none known; every golden is unchanged |
| byte scan slicing inside a multi-byte character | the comment test uses `str::get`, which is `None` off a boundary |

## Rollback

Revert the fix commit.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-11-17 | Olivier, Claude | Plan committed before the fix commit |
