---
document_id: PLAN-2026-0002
title: Implementation Plan — Diagnostics, Error Recovery, AST Output and Operator Precedence
document_type: plan
status: completed

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 2

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
  - docs

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Implements PLAN-B, PLAN-C, PLAN-D and PLAN-E of PROP-2026-0001 as one releasable increment — actionable parse diagnostics, statement-level error recovery, a structured AST output surface, and infix operator precedence.

reason: A language cannot be hosted on Capy while a failed parse reports only "no function matched" and stops at the first error, the parse result is unreachable as data, and infix arithmetic never becomes a tree.

related_documents:
  - PROP-2026-0001
  - PLAN-2026-0001
  - ADR-0001
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - parser
  - diagnostics
  - recovery
  - ast
  - precedence

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Implementation Plan — Diagnostics, Error Recovery, AST Output and Operator Precedence

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, docs

## Summary

The remaining four increments of `PROP-2026-0001`, implemented as one plan and
released together as 0.22.0. PLAN-2026-0001 delivered the spans these depend on.

## Objective, Scope and Proposal Baseline

**Baseline:** `PROP-2026-0001` revision 4; decision recorded in `ADR-0001`.
**Predecessor:** `PLAN-2026-0001` (completed, released as 0.21.0) — spans are a
hard prerequisite for every requirement here.

### Deviation from the proposal's plan map

`PROP-2026-0001` maps these requirements to four separate plans (PLAN-B…PLAN-E).
This plan owns all four. The reason is in the proposal's own PLAN-D rationale:

> "Emits tree **and** diagnostics; shipping it earlier would freeze a schema that
> recovery then changes."

The same argument runs backwards through the whole chain. `ParseResult` is
introduced by C, consumed by D and shaped by B's `Diagnostic`; releasing B alone
would publish a `Diagnostic` type that C immediately extends, and releasing C
alone would publish a `ParseResult` whose JSON form D then fixes. Four releases
would mean three public shapes that exist for one version each.

`DOCUMENTATION.md` §4.5 permits several plans "when work has independently
approvable or releasable increments". These are not independently releasable, so
one plan and one release is the conformant choice. PLAN-E (precedence) *is*
independent, but it is small and shares the release.

### Owned by this plan

| Category | IDs |
|---|---|
| Goals | G-02 (actionable diagnostics), G-03 (recovery), G-04 (structured output), G-05 (precedence) |
| Requirements | R6, R7, R8, R9, R10, R11, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R28, and R12 in part |
| Use cases | UC-01…UC-04, UC-06, UC-07, UC-08 |
| Changes | C-03, C-04, C-06, C-08, C-09, C-10, C-11, C-12 |
| Discovery | D-01 (`gojson` nesting depth) |

### Not owned

Byte offsets on `Span` (deferred from PLAN-2026-0001, tracked in RPT-2026-0001);
UQ-01's lattice, graphs and monomorphization (Non-Goals 1–3); an `error`
statement in the inner DSL (Non-Goal 4).

## Live Status Summary

| State | Count | Notes |
|---|---:|---|
| NOT STARTED | 0 | |
| IN PROGRESS | 0 | |
| BLOCKED | 0 | |
| DONE | 24 | all phases P1–P5 |
| FAILED | 0 | |
| DEFERRED | 0 | |

- **Current phase:** **COMPLETE.** All five phases exited; 0.22.0 released.
- **Next action:** none. D-01 resolved (gojson handles nested structures); contracts frozen and shipped.
- **Blockers:** none
- **Last updated:** 2026-09-16T00:00:00+08:00
- **Release target:** 0.22.0 — **released**

## Requirements and Use Cases

| Requirement / UC | Planned Outcome | Acceptance Criteria | Owning Phase | Status |
|---|---|---|---|---|
| R14 / UC-06 | Furthest-progressing attempt is reported | Three shapes dying at one token produce a message listing all three | P2 | **DONE** |
| R15 / UC-06 | Typed expectation vocabulary | Each rejection site contributes its typed expectation | P2 | **DONE** |
| R16 / UC-06 | `OneOf` reuses the did-you-mean hints | A typo'd enum value produces `did you mean` | P2 | **DONE** |
| R17 / UC-06 | `CloseDelim` carries the opening span | The unclosed-delimiter case labels the opening token | P2 | **DONE** |
| R18 / UC-06 | Diagnostic with severity, code, primary span, labels | `Diagnostic` public; renderer draws primary plus label | P2 | **DONE** |
| R26 / UC-06 | Context frame in the diagnostic | Message names the shape and argument being matched | P2 | **DONE** |
| R19 / UC-07 | Cascade suppression, configurable | One broken construct → one diagnostic; a 60-error file caps | P2 | **DONE** |
| R20 / UC-07 | Recovery emits a diagnostic, an error node, then resyncs | Three broken statements → exactly three diagnostics | P2 | **DONE** |
| R21 / UC-07 | Resync honours delimiter balance first | A missing `)` on line 2 of 40 does not eat the file | P2 | **DONE** |
| R22 / UC-08 | Partial tree plus diagnostics; `Block.stmts` type unchanged | Statements after a broken one are intact in `stmts` | P2 | **DONE** |
| R23 / UC-07 | Emission refuses output when an error node is present | `capy run` writes nothing and exits non-zero | P2 | **DONE** |
| R24 | `Library::run` signature and behaviour unchanged | Error goldens unchanged where the message is unchanged | P3 | **DONE** |
| R6 / UC-01 | `Library::parse -> ParseResult` | An external crate reads both fields | P2 | **DONE** |
| R7 / UC-02, UC-03 | `capy ast [--json]` | `capy ast … --json \| jq .` succeeds, exit 0 | P2 | NOT STARTED |
| R8 | JSON via `gojson`, one dependency | `cargo tree -p capy-core --depth 1` lists one | P3 | **DONE** |
| R9 / UC-03 | Documented schema with `schema_version` | `docs/ast-json.md` in the nav | P4 | **DONE** |
| R28 | Embedding guide warns `stmts` alone is not success | The line is adjacent to the `parse` example | P4 | NOT STARTED |
| R10 / UC-04 | Infix operators with documented precedence | `a*b+c` → `(a*b)+c`; `a+b*c` → `a+(b*c)` | P2 | **DONE** |
| R11 / UC-04 | Comparison precedence-ordered | `a+1==b*2` → `(a+1)==(b*2)` | P2 | **DONE** |
| R13 | wasm builds, size not materially worse | Within M-02 | P3 | **DONE** |
| R25 | Furthest tracking does not slow parsing | Within M-01 | P3 | **DONE** |
| R12 (partial) | Nothing that parses today changes | 117 libraries; goldens; wasm; **T-27 round-trip** | P3 | NOT STARTED |

## Applicable Project Standards

| Rule | Plan Impact | Validation |
|---|---|---|
| `program_docs/standards/index.md` | Still does not exist | `NEEDS HUMAN REVIEW`, as in PLAN-2026-0001 |
| `CLAUDE.md` — additive; no existing library breaks | Binding; **the highest risk in this plan** | T-11, T-12, T-27 |
| `CLAUDE.md` — build, clippy, test, mkdocs green | P3 exit gate | T-10 |
| `CLAUDE.md` — library keyword list in sync | R10 changes value-expression grammar, not directives | confirm no `docs/library-keywords.md` row changes |
| `CLAUDE.md` — commit by name, no force-push | Binding on P5 | manual |
| `DOCUMENTATION.md` §21.3 | M-01, M-02 re-measured against the same frozen baselines | TEST |
| `DOCUMENTATION.md` §31 | Decision recorded per row at P4 | P4 exit gate |

## Measurable Claims

Baselines stay those frozen in `PROP-2026-0001`; 0.21.0's measured values are the
new comparison point, since the 5 % wasm allowance is shared across the release.

| Measurement | Baseline | Threshold | Report |
|---|---|---|---|
| M-01 transpile time | 187–219 µs (0.21.0 measured 188.185 µs) | ≤ 10 % regression | TEST |
| M-02 wasm size | 1 342 891 bytes (0.21.0 measured 1 367 674, +1.85 %) | ≤ 5 % **total** growth from baseline | TEST |
| M-03 direct dependencies | 1 (`regex`) | exactly 1 | TEST |
| M-04 failing-parse cost | — | ≤ 2× a successful parse of the same input | TEST |

## Prerequisites

1. PLAN-2026-0001 released (done — 0.21.0, tag `cf5f2f1`).
2. **D-01** — does `gojson` serialize arbitrarily nested maps and lists?
3. Contracts frozen for `Diagnostic`, `ParseResult` and the error node before P2.

## Phases, Entry Gates and Exit Gates

| Phase | Purpose | Entry Criteria | Exit Criteria | Depends On | Status |
|---|---|---|---|---|---|
| P1 | Contract / discovery | Predecessor released | D-01 answered; contracts frozen | — | NOT STARTED |
| P2 | Implementation | P1 exit | Every owned requirement implemented; workspace builds | P1 | NOT STARTED |
| P3 | Tests / validation | P2 complete | All gates pass; measurements within threshold | P2 | NOT STARTED |
| P4 | Docs / demos | Behaviour stable | §31 decision complete; demo executed | P3 | NOT STARTED |
| P5 | Release / rollout | Docs and demo verified | 0.22.0 tagged and verified | P4 | NOT STARTED |

## Implementation Approach

**Order is forced by the types.** B defines `Diagnostic`; C puts it in
`ParseResult` alongside the tree; D serializes both. E is independent and goes
last so its regression risk is isolated.

**Step 1 — furthest-failure (B).** The candidate loop currently discards the
error from every failed `try_match`. Thread a `Furthest { token_index, expected,
context }` through the matcher that is *not* restored on backtrack: strictly
further replaces, equal-distance unions, nearer is discarded. The union is what
produces "expected an expression, `(`, or an identifier".

**Step 2 — expectation vocabulary (B).** Each rejection site contributes a typed
`Expectation`. `OneOf` reuses `suggest_closest`, already used for type `options`.
`CloseDelim` carries the opening delimiter's span, which the block parsers know
and currently discard.

**Step 3 — recovery (C).** On statement failure: emit a diagnostic, push an
`ErrorNode` covering the skipped span, resync, continue. Resync checks delimiter
balance **first** — that rule is what stops a missing `)` eating the file. Error
nodes go in a parallel `Block.errors`, never by changing `Block.stmts`' type.

**Step 4 — `ParseResult` and output (C, D).** `Library::parse` returns
`{ tree, diagnostics }`; `Library::run` becomes a thin wrapper that returns the
first error, keeping its signature. `capy ast [--json]` serializes through
`gojson`.

**Step 5 — precedence (E).** A precedence-climbing layer between `parse_value`
and `parse_unary`. `expr_to_text` **must** learn the new node, or captured text
round-trips wrong and corrupts output silently — which no golden can catch,
because arithmetic is a parse error today. That is what T-27 exists for.

### Risks

| Risk | Detection | Mitigation |
|---|---|---|
| Recovery changes what parses | T-11, T-12 | Error nodes only at statement level, never inside a successful shape match |
| Runaway resync | T-20 | Delimiter balance before statement boundary |
| Cascade noise | T-19, T-22 | Spacing window + cap |
| Precedence silently corrupts output | **T-27 round-trip** | `expr_to_text` updated in the same change; structural equality, not string |
| Furthest tracking costs time | M-01, M-04 | One integer compare per rejection |
| `Diagnostic` churn across the four increments | — | One plan, one release; contracts frozen at P1 |

## Live Work Checklist

| Task | Phase | Description / Method | Requirement IDs | Production Files | Test Files | Status |
|---|---|---|---|---|---|---|
| TASK-001 | P1 | D-01: inspect `gojson` nesting support | R8 | `rust/src/gojson.rs` | manual | **DONE** |
| TASK-002 | P1 | Freeze `Diagnostic` / `Label` / `Severity` / `ContextFrame` | R18, R26 | — | manual | **DONE** |
| TASK-003 | P1 | Freeze `ParseResult` and `ErrorNode` | R6, R22 | — | manual | **DONE** |
| TASK-004 | P2 | `Expectation` enum | R15 | `make_parser.rs` | T-15 | **DONE** |
| TASK-005 | P2 | `Furthest` threaded through the matcher | R14, R26 | `make_parser.rs` | T-14 | **DONE** |
| TASK-006 | P2 | Rejection sites contribute expectations | R15, R16 | `make_parser.rs` | T-15, T-16 | **DONE** |
| TASK-007 | P2 | Record the opening-delimiter span | R17 | `make_parser.rs` | T-17 | **DONE** |
| TASK-008 | P2 | `Diagnostic` / `Label` / `Severity` / `ContextFrame` | R18, R26 | `domain/errors.rs` | T-17, T-28 | **DONE** |
| TASK-009 | P2 | Renderer draws a secondary label | R18 | `domain/errors.rs` | T-17 | **DONE** |
| TASK-010 | P2 | `ErrorNode` + `Block.errors` | R22 | `domain/ast.rs` | T-21 | **DONE** |
| TASK-011 | P2 | Resync loop with delimiter-balance priority | R20, R21 | `make_parser.rs` | T-18, T-20 | **DONE** |
| TASK-012 | P2 | Cascade suppression and cap | R19 | `make_parser.rs` | T-19, T-22 | **DONE** |
| TASK-013 | P2 | `ParseResult`, `Library::parse`, `run` as wrapper | R6, R24 | `capy.rs` | T-02, T-25 | **DONE** |
| TASK-014 | P2 | Emission refuses on error nodes | R23 | `make_evaluator.rs` | T-22 | **DONE** |
| TASK-015 | P2 | AST + diagnostics JSON serializer | R8, R9 | `domain/ast_json.rs` | T-05 | **DONE** |
| TASK-016 | P2 | `capy ast [--json]` + dispatch | R7 | `cli/src/cmd_ast.rs`, `main.rs` | T-04, T-06 | **DONE** |
| TASK-017 | P2 | Precedence climbing + binary node + evaluation | R10, R11 | `value_parser.rs`, `ast.rs`, `inner_evaluator.rs` | T-07, T-08 | **DONE** |
| TASK-018 | P2 | `expr_to_text` renders the new node | R12 | `expr_to_text.rs` | T-27 | **DONE** |
| TASK-019 | P3 | Test files for B, C, D, E | all | `rust/tests/` | — | **DONE** |
| TASK-020 | P3 | Regression gates + measurements | R12, R13, R25 | — | T-10…T-13, M-01…M-04 | **DONE** |
| TASK-021 | P4 | `docs/diagnostics.md`, `docs/ast-json.md` + nav | R9, R18 | `docs/`, `mkdocs.yml` | mkdocs | **DONE** |
| TASK-022 | P4 | Update embedding, CLI, language-reference, inner-dsl, whats-new | R28, R10 | `docs/` | mkdocs | **DONE** |
| TASK-023 | P4 | TEST / RPT / DEMO / MAN / SYS / ARCH documents | all | `program_docs/` | — | **DONE** |
| TASK-024 | P5 | Version 0.22.0, REL, indexes, commit, tag, verify | — | — | — | **DONE** |

## Rollback Strategy

Each step is independently revertible because the types layer rather than
interleave: E touches only the value parser and its round-trip; D only adds a
surface; C can be reverted to leave B's better messages in place; B can be
reverted to leave 0.21.0's behaviour.

## Completion Criteria

Every owned requirement `PASS` in the validation report, no `FAIL`, demo executed
by a non-implementer, 0.22.0 tagged.

## Open Questions

Inherited from `PROP-2026-0001`: OQ-3 (JSON error shape), OQ-5 (cascade
constants), OQ-6 (diagnostic code namespace), OQ-7 (`and`/`or`), OQ-8 (whether
PLAN-E is needed at all), OQ-9 (wasm ABI exposure). OQ-3 and OQ-8 want Glang's
answer; the rest are decided here and recorded in the Decisions table.

## Related Documents

- `PROP-2026-0001` — the baseline
- `PLAN-2026-0001` — the predecessor, and the source of the deferred byte offsets
- `ADR-0001` — frozen contracts

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial plan — PLAN-B…PLAN-E consolidated into one releasable increment, with the deviation from the proposal's plan map recorded |
| 2 | 2026-09-16 | Olivier | All phases complete; 0.22.0 released. Two deviations recorded: grouping parentheses needed a narrower rule than R10 implied (`(` is the prefix-call form), and resync needed a bound beyond delimiter balance — an unclosed delimiter otherwise holds depth above zero to EOF and consumes the file. One error golden updated deliberately. |
