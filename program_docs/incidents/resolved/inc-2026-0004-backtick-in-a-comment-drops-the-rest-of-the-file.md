---
document_id: INC-2026-0004
title: Incident — A Backtick in a Comment Drops the Rest of the File
document_type: incident
status: resolved

created_date: 2026-11-17
last_updated: 2026-11-17
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
  - capy-core

severity: high
start_time: 2026-11-17T00:00:00+08:00
end_time: 2026-11-17T00:00:00+08:00
root_cause_status: identified

affected_versions:
  from: "0.1.0"
  to: "0.27.0"

applicable_environments:
  - development

audience:
  - engineers

scope: Records the lexer pre-pass that let one unpaired backtick in a comment or string silently discard the rest of a script, and its fix.

reason: DOCUMENTATION.md section 25 requires a defect to be recorded, not buried in a commit message. Found by the downstream Capyler compiler (its INC-2026-0001).

related_documents:
  - PLAN-2026-0007
  - REL-0.27.1

supersedes: null
superseded_by: null

tags:
  - incident
  - lexer
  - backtick
  - silent-failure

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-05-17
---

# Incident — A Backtick in a Comment Drops the Rest of the File

> **Status:** Resolved
> **Created:** 2026-11-17
> **Last Updated:** 2026-11-17
> **Affected Versions:** 0.27.0 and earlier
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Incident Summary

`merge_backtick_lines` (`rust/src/orchestrator/features/make_lexer.rs`) joins a multi-line backtick
literal into one logical line before tokenizing. It toggled "inside a literal" on every backtick,
including backticks inside a comment or a string. One unpaired backtick in a `//` comment made every
later line part of the same logical line, which then tokenized as a single comment. The parse reported no
error; everything after the comment was ignored.

```text
   // see the ` operator        <- opens a "literal" that never closes
   note a                       <- merged into the comment line
   note b                       <- merged into the comment line

   capy run  ->  exit 0, no output for a and b          (a clean parse of nothing)
```

## Severity

High: silent data loss with a success exit code. A string with a backtick (`say "a ` b"`) failed with a
confusing `no library function matches token` instead.

## Status

Resolved in 0.27.1.

## Discovery Context

The downstream Capyler compiler lost the rest of a file to a backtick in a comment (its INC-2026-0001).

## Start Time

Present since the backtick merge was written; not bisected.

## End Time

2026-11-17.

## Affected Systems

Every path that tokenizes a user script: `capy run`, `capy ast`, `Library::parse`, the wasm module, the C ABI.

## Affected Versions

0.27.0 and earlier.

## Affected Components

`capy-core`.

## Customer Impact

Scripts silently lost their tail. Fixed by recognising comments and strings before counting backticks.

## Detection

Reported by a downstream project.

## Reproduction Steps

```sh
capy run samples/backtick-in-comment/lib.capy samples/backtick-in-comment/script.capy
# 0.27.0: prints nothing.  0.27.1: four lines.
```

## Timeline

| Time | Event |
|---|---|
| 2026-11-17 | Reported with the downstream incident |
| 2026-11-17 | Reproduced on 0.27.0: empty output, exit 0 |
| 2026-11-17 | Fixed, four samples added |

## Logs and Evidence

Old `samples/backtick-in-comment` output: empty. Old `samples/backtick-in-string`: `no library function matches token "say"`. Old unclosed literal: `line 2: unterminated string`.

## Source Files

`rust/src/orchestrator/features/make_lexer.rs` (`merge_backtick_lines`); `samples/backtick-*`.

## Root Cause

An ordering error. The merge runs before tokenizing, so it cannot ask the tokenizer what is a comment. It
counted raw backticks instead of recognising comments and strings first.

## Contributing Factors

- A clean parse of a swallowed tail looks like success.
- No sample put a backtick in a comment or string.

## Resolution

The pass scans in the lexer's order. Outside a literal: a comment marker (the library's `comments`)
copies the rest of the line verbatim; a `"` or `'` string is copied whole; only then is a backtick a
literal opener. A literal still open at end of input is `line N: backtick literal opened here is never
closed`, naming the opening line. Inside a literal nothing changes. No existing golden changed.

## Verifying Tests

Goldens `backtick-in-comment`, `backtick-in-string`, `backtick-multiline`, `backtick-unclosed`.

## Corrective Actions

Done: the fix and the four samples.

## Preventive Actions

Any new lexical construct that can contain a backtick must be handled before the merge counts it.

## Owners

Capy Engine.

## Remaining Risks

The library-file parser has its own backtick handling for function bodies; it was not changed.

## Lessons Learned

A pre-pass that runs before the tokenizer must not guess at what the tokenizer would decide; scan in the same order.

## Related Documents

- `PLAN-2026-0007`, `REL-0.27.1`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-11-17 | Olivier, Claude | Initial record |
