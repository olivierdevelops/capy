---
document_id: INC-2026-0002
title: Incident — Deeply Nested Expressions Overflow the Stack
document_type: incident
status: resolved

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
  - capy-core

severity: high
start_time: 2026-10-13T00:00:00+08:00
end_time: 2026-10-13T00:00:00+08:00
root_cause_status: identified

affected_versions:
  from: "0.25.0"
  to: "0.25.0"

applicable_environments:
  - development

audience:
  - engineers

scope: Records the unbounded recursion in the value-expression parser, found while writing the C ABI, and its fix.

reason: DOCUMENTATION.md section 25 requires a defect found during implementation to be recorded, not buried in a commit message.

related_documents:
  - PROP-2026-0008
  - PLAN-2026-0005
  - ARCH-003 (STD-2026-0004)
  - REL-0.26.0
  - INC-2026-0003

supersedes: null
superseded_by: null

tags:
  - incident
  - recursion
  - stack-overflow
  - parser

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Incident — Deeply Nested Expressions Overflow the Stack

> **Status:** Resolved
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.25.0 (earlier versions not tested)
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Incident Summary

The value-expression parser (`rust/src/orchestrator/features/value_parser.rs`) is recursive descent and
had **no depth bound**. A statement with enough nested parentheses overflowed the stack and killed the
process. A stack overflow cannot be caught as `Err`, so an embedder cannot survive it; `ARCH-003` ("the
engine never aborts its host") forbids exactly this, and `PROP-2026-0008` had recorded `ARCH-003` as
`PASS` on the belief that `MAX_PARSE_DEPTH = 64` covered it. It covered capture nesting only.

```text
  let x = (((((( … 20 000 levels … 1 … ))))))        capy ast lib.capy deep.capy
  thread 'main' has overflowed its stack
  fatal runtime error: stack overflow, aborting        rc=134 (SIGABRT)
```

## Severity

High for the new C ABI (a hostile or buggy source file aborts the embedding compiler, defeating the
point of `G-03`); low for the CLI (the process is already the one parsing the file).

## Status

Resolved in 0.26.0.

## Discovery Context

Found while testing `capy-ffi` against hostile input (`PLAN-2026-0005` P4): a 10 000-opener C test passed
only because its grammar used an `ident` capture. A probe through an `any` capture with the sample
`operator-precedence` library crashed.

## Start Time

Not bisected. Reproduced on 0.25.0 (`69fb4ac`); earlier versions were not tested.

## End Time

2026-10-13 (this release).

## Affected Systems

`capy run`, `capy ast`, `Library::parse`, `Library::run`, the wasm module, the MCP server — anything that
parses an expression-typed capture.

## Affected Versions

0.25.0, reproduced identically on `69fb4ac` in a release build. Earlier versions were not tested.

## Affected Components

`capy-core`.

## Customer Impact

A script with thousands of nested brackets aborts `capy` instead of reporting an error. No report of this
in the wild; the input is unnatural.

## Detection

Manual probe during C-ABI hostile-input testing.

## Reproduction Steps

```sh
python3 -c "n=20000; print('let x = '+'('*n+'1'+')'*n)" > deep.capy
capy ast samples/operator-precedence/lib.capy deep.capy        # SIGABRT on the baseline binary
```

Thresholds, same code: about 1 000 levels in a debug build on the 8 MiB main thread; between 5 000 and
20 000 in a release build; far fewer on a thread with a smaller stack.

## Timeline

| Time | Event |
|---|---|
| 2026-10-13 | 10 000-opener hostile test passes (wrong grammar); probe with an `any` capture overflows |
| 2026-10-13 | Confirmed identical on baseline `69fb4ac`; not caused by the FFI work |
| 2026-10-13 | Bound added, tests written, mutation-checked |

## Logs and Evidence

Baseline release binary: 5 000 levels `rc=0`, 20 000 `rc=-6`. New binary before the fix: same. After the
fix: every level count returns a diagnostic. See `TEST-2026-0015` T-11.

## Source Files

`rust/src/orchestrator/features/value_parser.rs` (`parse_binary`, `parse_unary`, `parse_primary`);
`rust/tests/recursion_guard.rs`.

## Root Cause

Three mutually recursive functions with no counter. The paths that recurse without passing through the
others: a `**` chain (`parse_binary` calls `parse_binary` for its right operand), `not` chains
(`parse_unary` → `parse_unary`), and prefix-call arguments (`parse_primary` → `parse_primary`), so a
guard in only one function would have left holes.

## Contributing Factors

- `ARCH-003` was satisfied for captures (`MAX_PARSE_DEPTH`) and assumed to cover expressions.
- No test drove nesting through an `any` capture at depth.
- The recursion guard tests exercised `ident`-captured nesting only.

## Resolution

A thread-local counter with an RAII guard (`DepthGuard`) entered at the top of all three functions; limit
`3 * 64` entries: an expression may nest 64 levels counting its own outermost one, so 63 nested pairs of
parentheses parse and the 64th is refused (measured: 63 → exit 0, 64 → exit 1). Beyond it the function returns a `CapyError`, and the
statement is reported as unparseable. The grouping rule for `(` discards the inner error, so the guard also
records its error in a thread-local that the statement matcher moves into `OuterP.depth_err`; the statement is
then reported as `E0003` with a message that names the limit and a caret at the column where the parser stopped. Behaviour for any input nested less deeply is unchanged (all 131
goldens and the 133-script default-JSON comparison are byte-identical).

## Verifying Tests

`rust/tests/recursion_guard.rs`: `expression_nesting_inside_the_bound_still_parses` (60 levels of three
shapes), `deeply_nested_expressions_error_instead_of_aborting` (six shapes × 200 / 5 000 / 200 000, parse
and run, on a 2 MiB stack), `the_depth_counter_is_released_between_statements`. Mutation: setting the
limit to `u32::MAX` makes the binary abort with a stack overflow.

## Corrective Actions

Done: the bound and the tests. `PROP-2026-0008`'s `ARCH-003` row is corrected in `RPT-2026-0005`.

## Preventive Actions

`recursion_guard.rs` now covers expression nesting in six shapes. Any new recursive parse path must add a
shape to `deeply_nested_expressions_error_instead_of_aborting`.

## Owners

Capy Engine.

## Remaining Risks

- Other recursive consumers of `Expr` (the evaluator, `expr_to_text`, the JSON writer) are bounded by the
  parser's limit but have no independent test at the limit beyond the shapes above.
- 64 levels is a new limit: source nested between 64 and about 5 000 pairs of parentheses parsed before and is now refused.
  Nothing in `samples/` or the documentation approaches it.

## Lessons Learned

A `PASS` against a blocking standard must name the code that enforces it; "the bound exists" was true of the
wrong recursion.

## Related Documents

- `PROP-2026-0008`, `PLAN-2026-0005`, `TEST-2026-0015`, `RPT-2026-0005`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial record |
