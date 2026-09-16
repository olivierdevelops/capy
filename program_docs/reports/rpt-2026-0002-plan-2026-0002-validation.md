---
document_id: RPT-2026-0002
title: Validation of PLAN-2026-0002 — Diagnostics, Recovery, AST Output and Precedence
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
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Validates the implemented result of PLAN-2026-0002 against every requirement the plan owns, and records deviations, limitations and required follow-up.

reason: DOCUMENTATION.md section 28 requires an explicit validation of the implementation against the plan before the release proceeds.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001
  - TEST-2026-0005
  - TEST-2026-0006
  - TEST-2026-0007
  - TEST-2026-0008

supersedes: null
superseded_by: null

tags:
  - validation
  - parser
  - spans
# Validation of PLAN-2026-0002 — Diagnostics, Recovery, AST Output and Precedence

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Summary

Every requirement PLAN-2026-0002 owns is implemented and evidenced. **No result
is `FAIL` and none is `PARTIAL`.** One defect was found during testing rather
than review, and one design constraint was discovered that shaped the
implementation; both are recorded below.

## Plan Under Validation

`PLAN-2026-0002` revision 1, implementing PLAN-B, PLAN-C, PLAN-D and PLAN-E of
`PROP-2026-0001` revision 4 as one increment.

## Method

Each requirement checked against the acceptance criteria as written, using
TEST-2026-0005…0008. Regression validated across all 117 libraries, the full
golden corpus and the wasm corpus rather than a sample.

## Requirement Results

| Requirement | Expected | Observed | Test | Result |
|---|---|---|---|---|
| R14 | Furthest attempt reported; expectations unioned | `expected \`)\`, found end of statement in \`fn\``; three tied shapes list all three | TEST-2026-0005 | PASS |
| R15 | Typed expectation vocabulary | `Literal` / `Kind` / `Nonterminal` / `OneOf` / `CloseDelim` / `BlockEnd` | TEST-2026-0005 | PASS |
| R16 | `OneOf` reuses did-you-mean | Wired to the existing `suggest_closest` | TEST-2026-0005 | PASS |
| R17 | `CloseDelim` carries the opening span | Label emitted when the opener is known | TEST-2026-0005 | PASS |
| R18 | Severity, code, primary span, labels | `Diagnostic` / `Label` / `Severity` public; renderer draws both | TEST-2026-0005 | PASS |
| R26 | Context frame present | Message contains ``in `fn` `` | TEST-2026-0005 | PASS |
| R19 | Suppression and cap | One construct → one diagnostic; 200 errors → ≤ 20 | TEST-2026-0005 | PASS |
| R20 | Diagnostic + error node + resync per failure | 3 statements, 2 diagnostics, 2 error regions | TEST-2026-0005 | PASS |
| R21 | Delimiter balance first | A missing `)` leaves 30 following statements intact | TEST-2026-0005 | PASS |
| R22 | Partial tree; `stmts` type unchanged | An unmodified walker still compiles and sees a correct partial list | TEST-2026-0005 | PASS |
| R23 | Emission refuses on error nodes | `run` errors rather than emitting | TEST-2026-0005 | PASS |
| R24 | `run` unchanged | Same signature; first error, no output | TEST-2026-0005 | PASS |
| R6 | `Library::parse -> ParseResult` | External crate reads both fields | TEST-2026-0007 | PASS |
| R7 | `capy ast [--json]` | Tree and JSON modes; exit 0/1; stderr empty on success | TEST-2026-0007 | PASS |
| R8 | JSON via `gojson`, one dependency | `cargo tree` lists 1 | TEST-2026-0007, TEST-2026-0008 | PASS |
| R9 | Documented schema with `schema_version` | `docs/ast-json.md`, in the nav | TEST-2026-0007 | PASS |
| R28 | Embedding guide warns `stmts` alone is not success | Present next to the `parse` example | TEST-2026-0007 | PASS |
| R10 | Infix operators, documented precedence | `1 + 2 * 3` = 7; `(1 + 2) * 3` = 9 | TEST-2026-0006 | PASS |
| R11 | Comparison precedence-ordered | `a + 1 == b * 2` → `(a+1)==(b*2)` | TEST-2026-0006 | PASS |
| R12 | Nothing that parses changes | 117/117; goldens 117/0; wasm 114/0; round-trip holds | TEST-2026-0005…0007 | PASS |
| R13 | wasm builds, size acceptable | +3.22 % total, 5 % allowance | TEST-2026-0008 | PASS |
| R25 | Furthest tracking not measurably slower | 192 µs best-of-5, inside the baseline band | TEST-2026-0008 | PASS |

## Deviations From the Plan

1. **Parenthesised grouping needed a narrower rule than R10 implied.** `(` is the
   prefix-call form in this grammar (`(upper n)`), so grouping could not simply be
   added. It applies only where unambiguous: the contents parse as a complete
   expression that is not a bare identifier. `(upper n)` stays a call and `(foo)`
   stays a zero-argument call, so nothing that parses today changes.

2. **Resync needed a bound beyond delimiter balance.** See below.

3. **One error golden changed.** `samples/bbcode-parser/mismatch` previously
   expected `no library function matches token "["`; it now reports
   `expected \`b\`, \`i\`, \`quote\`, or \`url\`, found "/" in \`bold\``. Reviewed
   deliberately and updated rather than regenerated, per PROP OQ-11. The other six
   error goldens are unchanged.

## Unintended Behaviour

None surviving. One was found during testing:

**The resync scan consumed entire files.** `a_missing_delimiter_does_not_eat_the_file`
failed with **0 statements surviving**. Tracking delimiter balance is correct
until the delimiter is never closed — then depth stays above zero to EOF and the
scan swallows everything, which is the precise failure R21 exists to prevent.
Fixed by treating a line break inside an unclosed delimiter as evidence that the
delimiter is missing, and accepting a statement-start token from that point.

Worth recording because the rule as written in the proposal — "never resync while
inside an unclosed bracket" — is correct only for brackets that eventually close.

## Unresolved Incidents

None; no incident occurred during implementation.

## Remaining Limitations

1. **Byte offsets** still absent from `Span` (carried from PLAN-2026-0001).
2. **The depth-limit diagnostic** still reports the generic message rather than
   naming the limit; the furthest-failure machinery now exists to improve it, but
   it was not wired for that case.
3. **Grouping is conditional**, as described in deviation 1. `(foo)` cannot be
   used to group a single identifier.
4. **Cascade constants are defaults**, not tuned against a real corpus of broken
   files. OQ-5 remains open.
5. **The wasm ABI does not expose the AST or diagnostics.** OQ-9 remains open.

## Required Follow-Up

| Item | Owner | Destination |
|---|---|---|
| Byte offsets on `Span` | Capy Engine | a future plan |
| Name the depth limit in its diagnostic | Capy Engine | a future plan |
| Tune the cascade constants | Capy Engine | OQ-5 |
| Decide wasm ABI exposure | Capy Engine + Glang | OQ-9 |

## Conclusion

PLAN-2026-0002 is **validated**. No requirement failed; none is partial. With
`PROP-2026-0001`'s five increments now all released, Glang's three reported
blockers are addressed: expression trees (via spans plus function-as-type, and
now infix precedence), positions on every node, and structured output.

The lesson worth carrying is deviation 2: a rule that is correct for well-formed
input can be exactly wrong for the malformed input it exists to handle.

## Related Documents

- PLAN-2026-0002
- PROP-2026-0001 — UQ-02…UQ-06
- TEST-2026-0005…0008

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial validation |
