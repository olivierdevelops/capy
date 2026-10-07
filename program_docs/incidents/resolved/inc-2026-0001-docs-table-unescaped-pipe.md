---
document_id: INC-2026-0001
title: Incident — `capy docs` Printed Unescaped Pipes in a Markdown Table Cell
document_type: incident
status: resolved

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 1

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core

severity: low
start_time: 2026-10-07T00:00:00+08:00
end_time: 2026-10-07T00:00:00+08:00
root_cause_status: identified

affected_versions:
  from: "0.23.0"
  to: "0.23.0"

applicable_environments:
  - development

audience:
  - engineers

scope: A defect in the new ordered-choice change, found during release verification and fixed before the release commit. It never shipped.

reason: DOCUMENTATION.md section 25 requires unexpected bugs found during implementation to be recorded in incidents/ rather than hidden in implementation notes or commit messages.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - DEMO-2026-0003
  - TEST-2026-0009
  - RPT-2026-0003
  - REL-0.23.0

supersedes: null
superseded_by: null

tags:
  - incident
  - docs
  - alternation

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Incident — `capy docs` Printed Unescaped Pipes in a Markdown Table Cell

> **Status:** Resolved
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0 (pre-release working tree only)
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Incident Summary

`capy docs` renders each function's captures as a Markdown table. The ordered-choice change printed
the alternatives joined with a bare ` | `, so the Type column read `` `call | name | num` ``. In GitHub
Flavored Markdown an unescaped `|` ends a table cell **even inside a code span**, so a strict renderer
would split the one cell into three.

```text
   | Argument | Type                     | Description |      what was printed
   | `v`      | `call | name | num`      | ...         |   ─►  5 cells instead of 3

   | `v`      | `call \| name \| num`    | ...         |      what is printed now
```

## Severity

Low. Documentation output only; no parse behaviour changed and nothing shipped.

## Status

Resolved before the release commit.

## Discovery Context

`DEMO-2026-0003` U-06 (verify `capy docs` prints the union). The verifier scored the row `PARTIAL`
because the output was "correct text, wrong for a Markdown table". The original test
`docs_print_the_union` asserted the unescaped string, so it passed and could not have caught this.

## Start Time

2026-10-07, when the change to `rust/src/domain/docs.rs` was made.

## End Time

2026-10-07, when the fix was verified.

## Affected Systems

Capy, `capy docs` output.

## Affected Versions

Pre-release working tree only. `v0.22.0` is unaffected (it has no alternation); `v0.23.0` contains the fix.

## Affected Components

`capy-core` — `rust/src/domain/docs.rs`.

## Customer Impact

None. No release contained the defect.

## Detection

Release verification (`DEMO-2026-0003` U-06), by reading the rendered table rather than only checking that
the text was present.

## Reproduction Steps

```sh
git stash   # or check out the working tree before the fix
rust/target/debug/capy docs samples/expression-grammar/lib.capy | grep '`v`'
# | `v` | `call | name | num` | *(no description)* |
```

## Timeline

```text
   change made ─► tests green ─► DEMO U-06 verified ─► scored PARTIAL
        ─► fix: escape pipes, update the test ─► re-verified ─► release commit
```

## Logs and Evidence

After the fix:

```text
$ rust/target/debug/capy docs samples/expression-grammar/lib.capy | grep '`v`'
| `v` | `call \| name \| num` | *(no description)* |
```

## Source Files

`rust/src/domain/docs.rs`, `rust/tests/alternation.rs` (`docs_print_the_union`).

## Root Cause

The union was built with `.join(" | ")`. The existing single-type path never needed escaping because a
type name cannot contain `|`; the new join introduced the first pipe into a table cell.

## Contributing Factors

The first test asserted the string the code produced, not the property the output is for (that it
renders as one cell).

## Resolution

`.join(" \\| ")` in `rust/src/domain/docs.rs`; the test now asserts the escaped form.

## Verifying Tests

`docs_print_the_union` (`TEST-2026-0009` T-12), `DEMO-2026-0003` U-06.

## Corrective Actions

Done: escape the pipes; update the test.

## Preventive Actions

When output is rendered into a format with metacharacters, assert the rendered structure, not only the
text. Recorded here; no tooling added.

## Owners

Capy Engine

## Remaining Risks

None known. The wasm `capyIntrospect` JSON is unaffected (it is not a Markdown table) and does not carry
`alts` (see `REL-0.23.0` Limitations).

## Lessons Learned

A passing test that restates the implementation proves nothing about the property that matters.

## Related Documents

- `PLAN-2026-0003`, `PROP-2026-0004`, `DEMO-2026-0003`, `TEST-2026-0009`, `RPT-2026-0003`, `REL-0.23.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
