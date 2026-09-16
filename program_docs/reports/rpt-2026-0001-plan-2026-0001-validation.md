---
document_id: RPT-2026-0001
title: Validation of PLAN-2026-0001 — Recursion Guard, Spans and Comment Retention
document_type: report
status: completed

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 1

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine
  - Glang (consumer)

systems:
  - Capy

components:
  - capy-core
  - capy-cli

affected_versions:
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Validates the implemented result of PLAN-2026-0001 against every requirement the plan owns, and records deviations, limitations and required follow-up.

reason: DOCUMENTATION.md section 28 requires an explicit validation of the implementation against the plan before the release proceeds.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001
  - TEST-2026-0001
  - TEST-2026-0002
  - TEST-2026-0003
  - TEST-2026-0004

supersedes: null
superseded_by: null

tags:
  - validation
  - parser
  - spans

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Validation of PLAN-2026-0001 — Recursion Guard, Spans and Comment Retention

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Summary

Every requirement PLAN-2026-0001 owns is implemented and evidenced. **One result
is `PARTIAL`** — R1, because byte offsets were deferred with a recorded reason.
Nothing is `FAIL`. Two findings changed the plan during execution and are
recorded below rather than folded away.

## Plan Under Validation

`PLAN-2026-0001`, revision 3, implementing PLAN-A of `PROP-2026-0001` revision 4.

## Method

Each requirement was checked against its acceptance criteria as written in the
plan, using the tests recorded in TEST-2026-0001…0004. Regression was validated
across all 117 checked-in libraries and the full golden corpus rather than a
sample. Measurements used baselines frozen before implementation.

## Requirement Results

| Requirement | Expected | Observed | Test | Result |
|---|---|---|---|---|
| R0 | The P-07 reproduction exits 1; rc never 134 | `capy check` and `capy run` both exit 1 naming the cycle. 20 000-level nested input also exits 1 instead of aborting | TEST-2026-0002 | PASS |
| R0b | `capy check` fails on the P-07 library | It does — detection is at library-load time, so the author learns at check time | TEST-2026-0002 | PASS |
| R1 | `Span` public, `Copy`, six fields incl. byte offsets | `Span` is public and `Copy` with **four** fields: start/end line and column. **Byte offsets deferred** | TEST-2026-0001 | **PARTIAL** |
| R2 | Block statement's `span.end` ≥ closer's last token | Verified: a `wrap … end` statement spans lines 1–3 | TEST-2026-0001 | PASS |
| R3 | Two captures differ and do not overlap | `world` 7..12 and `now` 13..16 | TEST-2026-0001 | PASS |
| R4 | No reachable node unset; child ⊆ parent | Three-level tree walked; no `line == 0`; containment holds | TEST-2026-0001 | PASS |
| R5 | Golden corpus byte-identical | 116 pass / 0 fail / 8 skip, unchanged | TEST-2026-0003 | PASS |
| R27 | Comment span on the following node; node span starts at the first code token | Both hold; rendered output identical with and without comments | TEST-2026-0003 | PASS |
| R12 (partial) | 117 libraries ok; goldens unchanged; wasm unchanged | 117/117, goldens unchanged, wasm 113 pass / 0 fail | TEST-2026-0003 | PASS |
| M-01 | ≤ 10 % regression vs 187–219 µs | 188.185 µs best of 5 — inside the baseline band | TEST-2026-0004 | PASS |
| M-02 | ≤ 5 % growth vs 1 342 891 bytes | 1 367 674 bytes, +1.85 % | TEST-2026-0004 | PASS |
| M-03 | exactly 1 direct dependency | 1 (`regex`) | TEST-2026-0004 | PASS |

## Deviations From the Plan

1. **Byte offsets deferred (R1 → `PARTIAL`).** `merge_backtick_lines` rewrites
   source bytes — it inserts a literal newline escape and reassembles via
   `unsafe` — so a merged line's column does not map onto a source byte offset.
   Deriving offsets needs a separate mapping threaded through byte-faithful code.
   `Span` is `#[non_exhaustive]`, applied in the same change as the fields, so the
   offsets can be added later without breaking any consumer.

2. **R0 required two mechanisms, not one.** The plan's discovery note treated the
   depth bound as a backstop for the case where left recursion proved statically
   undecidable. It is not: a *valid* right-recursive library fed 20 000 nested
   parentheses still aborted with rc=134. The load-time guard and the depth bound
   are both load-bearing.

3. **Depth limit 64, not 256.** A Rust test thread gets a 2 MiB stack against the
   main thread's 8 MiB. A limit tuned to the main thread passed via the CLI and
   still aborted under `cargo test` — the CI signal and the manual signal
   disagreed, and the CI signal was right.

## Unintended Behaviour

None observed. Specifically checked, because each was a named risk:

- The recursion guard rejects **zero** of the 117 existing libraries.
- Comment trivia does not reach the matcher: it is emitted only by
  `tokenize_with_trivia` and stripped in `parse` before matching.
- The `line`/`col` render locals are unchanged, evidenced by byte-identical
  golden output.

## Unresolved Incidents

None. No incident occurred during implementation, so no `INC` document was
created — recorded here rather than left blank.

## Remaining Limitations

1. **No byte offsets** on `Span` (deviation 1).
2. **Nesting deeper than 64 nonterminal levels is refused**, and the message
   falls back to the generic "no library function matches" rather than naming the
   depth limit. Surfacing the real reason is PLAN-B's furthest-failure work.
3. **No public entry point yet.** `Library::parse` arrives in PLAN-D; a consumer
   still reaches the AST through `orchestrator::features::make_parser::parse`,
   which is documented as unsupported in the interim.
4. **Leading comments only.** Trailing and interior comments are retained as
   trivia but unattached, per the answered OQ-12.

## Required Follow-Up

| Item | Owner | Destination |
|---|---|---|
| Byte offsets on `Span` | Capy Engine | a follow-up task in a PLAN-A revision, or folded into PLAN-D where the JSON schema needs them |
| Depth-limit diagnostic wording | Capy Engine | PLAN-B (furthest-failure) |
| Restate the comment-span rule in `docs/ast-json.md` | Capy Engine | PLAN-D — carry-forward CF-01 |

## Conclusion

PLAN-A's implementation is **validated**. No requirement failed. The single
`PARTIAL` is a recorded, reasoned deferral with a migration-safe representation,
not an unmet requirement. The release may proceed on this evidence.

The two findings are worth carrying into the remaining plans: a discovery answer
that looks favourable can still be incomplete (deviation 2), and a limit verified
only through the CLI can still be wrong under `cargo test` (deviation 3).

## Related Documents

- PLAN-2026-0001 — the plan validated here
- PROP-2026-0001 — P-01…P-03, P-07 (the reported defects)
- TEST-2026-0001…0004 — the evidence

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial validation |
