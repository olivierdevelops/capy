---
document_id: PLAN-2026-0006
title: Implementation Plan — Statement continuation after a delimiter block close (0.27.0)
document_type: plan
status: in_progress

created_date: 2026-11-16
last_updated: 2026-11-16
document_revision: 1

approval:
  approved_by:
    - Olivier
  approved_date: 2026-11-16

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
  from: "0.26.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: A library opt-in so `} else {` parses on one line; ships with the already-committed `a-1` and `f(args)` operand work as 0.27.0.

reason: The owner asked for a plan, the code change and a quick release. A downstream language library (Capyler) needs `} else {`, which `docs/roadmap.md` lists as a known gap.

related_documents:
  - PLAN-2026-0005
  - REL-0.26.0
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - parser
  - block
  - release

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-05-16
---

# Implementation Plan — Statement continuation after a delimiter block close (0.27.0)

> **Status:** In progress
> **Created:** 2026-11-16
> **Last Updated:** 2026-11-16
> **Affected Versions:** 0.27.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs

## Summary

```text
  P0 plan ─► P1 directive + loader + parser ─► P2 samples ─► P3 docs ─► P4 gates ─► P5 release B, R
                                                                                       (no tag, no push)
```

## Objective, Scope and Baseline

**Problem.** A delimiter block (`block_open "{" close "}"`) must be followed by a newline or end of
input (`make_parser.rs`, "expected newline after block close"). A C-family `} else {` therefore fails;
only `else` on its own line parses.

```text
   today                         wanted (opt-in)
   ─────                         ───────────────
   if c {                        if c {
       a()                           a()
   }            parses           } else {        same tree as the left
   else {                            b()
       b()                       }
   }
   if c { a() } else { b() }  ->  line 1: expected newline after block close
```

**Goal.** Let a library name the literals that may begin the next statement on the closer's line.
**Additive:** without the opt-in, behaviour and the error text are byte-identical.

**Not owned:** `else` as an inner-DSL construct (roadmap "else arm on inner if"); continuation for the
other block modes (the sequence-closed mode already allows it).

## Design

Spelling: `block_open "{" close "}" continue "else" ["elif" ...]`, on the block line.

| Option | Verdict | Why |
|---|---|---|
| keyword after `close` on the `block_open` line | **chosen** | the existing directive already carries `close`; the continuation is a property of that closer and of nothing else; `block_open` is the only mode with a same-line closer that requires a newline |
| new file-level directive | rejected | one library can have brace blocks whose closer must stay strict (`fn`) beside ones that continue (`if`); a file-level switch cannot say which |
| separate `block_continue` line | rejected | a second directive that is meaningless without `block_open`; the loader would have to cross-check |
| `else`-specific engine rule | rejected | the engine ships no source grammar (AGENTS.md); the literals are the library's |

```text
  lib line:  block_open "{" close "}" continue "else"
                  │          │             │
                  │          │             └─ RawBlock.continue_with ─► BlockSpec.continue_with
                  │          └─ close   (unchanged)
                  └─ open     (unchanged)

  parse (make_parser.rs, delimiter branch), after parse_delim_block returns:

      next token ── Newline / Eof ───────────────► old path (consume newlines, done)
          │
          ├── text in continue_with ─► return the instance; do NOT consume anything;
          │                            the caller's statement loop parses `else {...}`
          │                            as an ordinary next statement   (same AST)
          └── anything else ─────────► Err "line N: expected newline after block close"  (unchanged)
```

A literal is compared with the next token's text, so each literal is one token.

## Requirements and Acceptance

| Req | Outcome | Acceptance | Status |
|---|---|---|---|
| R1 | `continue` accepted on a `block_open` line | library with it loads; malformed form is a load error naming the grammar | planned |
| R2 | `} else {` parses | `capy ast` of the same-line and own-line scripts: same tree, only the `else` column differs | planned |
| R3 | additive | with no `continue`, `} else {` gives `line 3: expected newline after block close` | planned |
| R4 | no regression | existing goldens, `cargo test --workspace` unchanged | planned |
| R5 | docs true | `library-keywords`, `features`, `CAPY_FOR_LLMS`, `library-authoring`, `whats-new`, `roadmap`; `mkdocs build --strict` | planned |

## Work Checklist

| ID | Task | Files | Status |
|---|---|---|---|
| W-01 | parse `continue` on the `block_open` line | `rust/src/infra/capy_lib_parser.rs`, `rust/src/infra/raw_library.rs` | planned |
| W-02 | carry it into `BlockSpec` | `rust/src/domain/library.rs`, `rust/src/orchestrator/features/make_library_loader.rs` | planned |
| W-03 | accept a continuation after the closer | `rust/src/orchestrator/features/make_parser.rs` | planned |
| W-04 | samples: `block-continue` (two scripts, one expected), `block-continue-rejected` (old error) | `samples/` | planned |
| W-05 | docs (AGENTS.md rule: new directive) | `docs/*.md`, `samples/README.md`, `CHANGELOG.md` | planned |
| W-06 | gates | — | planned |
| W-07 | version 0.27.0 (commit B), release document (commit R) | `rust/**/Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `program_docs/**` | planned |

## Project Facts

This repository has no `vhco-contract.json` (the VHCO pages under `docs/legacy/` are historical), so the
contract, `vhco validate` and `vhco sync` steps do not apply.

## Version and Release

| Step | Detail |
|---|---|
| Bump | minor, 0.26.0 to 0.27.0 (additive feature; pre-1.0). Source: `rust/Cargo.toml` `[workspace.package]`, six member `capy-core` pins, `rust/Cargo.lock` |
| Includes | commit `461d5fd` (`a-1` subtracts; `f(args)` is an operand), which is after `v0.26.0` |
| Commits | plan, feature, B (version + changelog), R (release document naming B) |
| Tag / push | **not performed here.** The orchestrating session verifies, tags R and pushes |

## Risks

| Risk | Mitigation |
|---|---|
| a statement starting mid-line trips column or indent logic | the sample asserts the same tree; the own-line form is the reference |
| `else` with no preceding `if` | unchanged: `else` is a normal statement rule, so it parses wherever the library allows it |
| a literal that is several tokens never matches | documented: each literal is one token |

## Rollback

Revert the feature commit. Libraries without `continue` never reached the new branch.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-11-16 | Olivier, Claude | Plan committed before the feature commit |
