---
document_id: PLAN-2026-0002
title: Implementation Plan — Diagnostics, Error Recovery, AST Output and Operator Precedence
document_type: plan
status: completed

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 3

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
| `program_docs/standards/index.md` revision 1 | Now exists (`STD-2026-0000`); revalidated against real rule IDs | see rows below |
| GOAL-002 — engine changes are additive | Binding; **the highest risk in this plan** | T-11, T-12, T-27 |
| ARCH-001 — a public field never changes type | R22: error nodes in a parallel `Block.errors` | T-30 |
| GATE-001 — pre-commit gate | P3 exit gate | T-10 |
| CODE-003 — keyword list in sync | R10 changes value-expression grammar, not directives | confirmed: no `docs/library-keywords.md` row changed |
| CODE-001 — commit only when asked; stage by name | Binding on P5 | manual |
| QUAL-002 — measurable claims predeclared | M-01, M-02 re-measured against the same frozen baselines | TEST-2026-0008 |
| QUAL-001 — whole-corpus regression | 117 libraries, full golden corpus, wasm corpus | GATE-002 |
| QUAL-003 — a test must be able to fail | T-27 exists because no golden can contain arithmetic | TEST-2026-0006 |
| ARCH-002 — one dependency | R8: `gojson`, not serde | TEST-2026-0008 |
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

## File and Artifact Checklist

| ID | Category | Exact Path | CRUD | Planned Edit | Requirements / Tasks | Status | Verification |
|---|---|---|---|---|---|---|---|
| F-01 | production | `rust/src/domain/errors.rs` | UPDATE | `Severity`, `Label`, `ContextFrame`, `Diagnostic`, `codes`, `format_diagnostic` | R18, R26 / TASK-008, 009 | **DONE** | T-14…T-17 |
| F-02 | production | `rust/src/domain/ast.rs` | UPDATE | `ErrorNode`, `Block.errors`, `Expr::Binary`, `BinaryExpr`, `#[non_exhaustive]` on `Block` | R10, R22 / TASK-010, 017 | **DONE** | T-21, T-07 |
| F-03 | production | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | `Expectation`, `Furthest`, `note_failure`, `furthest_error`, ctx stack, resync, cascade, `parse_recovering` | R14–R21 / TASK-004…007, 011, 012 | **DONE** | T-14…T-20 |
| F-04 | production | `rust/src/orchestrator/features/value_parser.rs` | UPDATE | Precedence climbing; conditional grouping | R10, R11 / TASK-017 | **DONE** | T-07, T-27 |
| F-05 | production | `rust/src/orchestrator/features/inner_evaluator.rs` | UPDATE | Evaluate `Expr::Binary`; `arith`; short-circuit `and`/`or` | R10 / TASK-017 | **DONE** | T-08 |
| F-06 | production | `rust/src/orchestrator/features/expr_to_text.rs` | UPDATE | Round-trip with re-inserted parentheses | R12 / TASK-018 | **DONE** | **T-27** |
| F-07 | production | `rust/src/orchestrator/features/translate_new_shape.rs` | UPDATE | `render_expr` handles the new node | R12 / TASK-018 | **DONE** | T-27 |
| F-08 | production | `rust/src/orchestrator/features/make_evaluator.rs` | UPDATE | Refuse emission when `Block.errors` is non-empty | R23 / TASK-014 | **DONE** | T-22 |
| F-09 | production | `rust/src/capy.rs` | UPDATE | `ParseResult`, `Library::parse` | R6 / TASK-013 | **DONE** | T-02 |
| F-10 | production | `rust/src/domain/ast_json.rs` | CREATE | Serializer via `gojson` | R8, R9 / TASK-015 | **DONE** | T-05 |
| F-11 | production | `rust/src/domain/mod.rs` | UPDATE | Register `ast_json` | R8 / TASK-015 | **DONE** | build |
| F-12 | production | `rust/cli/src/cmd_ast.rs` | CREATE | `capy ast`, tree and JSON modes | R7 / TASK-016 | **DONE** | T-04, T-06 |
| F-13 | production | `rust/cli/src/main.rs` | UPDATE | Dispatch and help | R7 / TASK-016 | **DONE** | T-06 |
| F-14 | test | `rust/tests/diagnostics.rs` | CREATE | 14 tests | R14–R26 / TASK-019 | **DONE** | `cargo test` |
| F-15 | test | `rust/tests/precedence.rs` | CREATE | 6 tests incl. T-27 | R10–R12 / TASK-019 | **DONE** | `cargo test` |
| F-16 | test | `samples/bbcode-parser/mismatch.expected-error.txt` | UPDATE | Improved message, reviewed not regenerated | R12 / TASK-020 | **DONE** | golden suite |
| F-17 | docs | `docs/diagnostics.md` | CREATE | Message selection, recovery, resync order, codes | R18, R19 / TASK-021 | **DONE** | mkdocs |
| F-18 | docs | `docs/ast-json.md` | CREATE | Versioned schema | R9 / TASK-021 | **DONE** | mkdocs |
| F-19 | docs | `mkdocs.yml` | UPDATE | Nav entries for both pages | R9 / TASK-021 | **DONE** | mkdocs strict |
| F-20 | docs | `docs/embedding.md` | UPDATE | `Library::parse`, `ParseResult`, the R28 warning | R6, R28 / TASK-022 | **DONE** | mkdocs |
| F-21 | docs | `docs/cli.md` | UPDATE | `capy ast`, exit codes | R7 / TASK-022 | **DONE** | mkdocs |
| F-22 | docs | `docs/language-reference.md` | UPDATE | Precedence table replacing the false claim; Go leftover fixed | R10, R11 / TASK-022 | **DONE** | mkdocs |
| F-23 | docs | `docs/inner-dsl.md` | UPDATE | Operators, short-circuit, numeric behaviour | R10 / TASK-022 | **DONE** | mkdocs |
| F-24 | docs | `docs/whats-new.md` | UPDATE | 0.22.0 entry | — / TASK-022 | **DONE** | mkdocs |
| F-25 | version | `rust/Cargo.toml` + 5 dependent manifests | UPDATE | 0.21.0 → 0.22.0 | — / TASK-024 | **DONE** | build |
| F-26 | generated | `rust/Cargo.lock` | UPDATE | Version propagation | — / TASK-024 | **DONE** | build |
| F-27 | release | `program_docs/` | CREATE | TEST-2026-0005…0008, RPT-2026-0002, DEMO-2026-0002, REL-0.22.0 | all / TASK-023, 024 | **DONE** | §33 |

**Counts.** production CREATE 2, production UPDATE 11, test CREATE 2, test UPDATE 1,
docs CREATE 2, docs UPDATE 6, version 1, generated 1, release 1. No DELETE.

## Test and Validation Checklist

| Test ID | Type | Requirement | Scenario | Exact Test File | Expected Result | Status |
|---|---|---|---|---|---|---|
| T-14 | unit | R14 | Furthest attempt reported | `rust/tests/diagnostics.rs` | names `` `)` ``, not the generic message | **PASS** |
| T-15 | unit | R14, R15 | Expectations union at a tie | same | all three alternatives listed | **PASS** |
| T-16 | unit | R16 | did-you-mean reuse | same | help line emitted | **PASS** |
| T-17 | unit | R17, R18 | Two-span rendering | same | label on the opening delimiter | **PASS** |
| T-28 | unit | R26 | Context frame | same | message contains ``in `fn` `` | **PASS** |
| T-18 | integration | R20 | Three broken statements | same | exactly 3 statements, 2 diagnostics | **PASS** |
| T-19 | integration | R19 | Cascade suppression | same | one construct, one diagnostic | **PASS** |
| T-20 | integration | R21 | Delimiter fence | same | 30 following statements survive | **PASS** |
| T-21 | integration | R22 | Error node; later statements intact | same | statement after the error parses | **PASS** |
| T-22 | integration | R19, R23 | Cap; refusal to emit | same | ≤ 20 diagnostics; `run` errors | **PASS** |
| T-25 | unit | R24 | `run` unchanged | same | first error, no output | **PASS** |
| T-26 | property | R12 | Valid program is clean | same | no diagnostics, no error nodes | **PASS** |
| T-30 | compatibility | R22 | Old walker over `stmts` | same | compiles; degraded but correct view | **PASS** |
| T-07 | unit | R10 | Precedence and associativity | `rust/tests/precedence.rs` | `(a*b)+c`, `a+(b*c)`, left-assoc | **PASS** |
| T-11 | unit | R11 | Comparison ordering | same | `(a+1)==(b*2)` | **PASS** |
| T-08 | integration | R10 | Evaluation | CLI | 7, 9, 5, 2, 3.5, 2 | **PASS** |
| **T-27** | property | R10, R12 | **Round-trip, structural equality** | `rust/tests/precedence.rs` | tree unchanged across render/reparse | **PASS** |
| T-02 | integration | R6 | `Library::parse` from an external crate | scratch crate | both fields readable | **PASS** |
| T-04 | unit | R7 | Tree renderer | CLI | nodes with spans | **PASS** |
| T-05 | unit | R7–R9 | JSON well-formed, `schema_version` | CLI + `jq` | parses | **PASS** |
| T-06 | E2E | R7 | Exit codes, stream discipline | CLI | 0/empty stderr; 1 on broken | **PASS** |
| T-10a/b/c | build, lint | R12 | `GATE-001` | CI | all green | **PASS** |
| T-12 | regression | R12 | `GATE-002` corpus | CI | 117/117; goldens 117/0 | **PASS** |
| T-13 | regression | R13 | wasm corpus | CI | 114/0 | **PASS** |
| M-01…M-03 | performance, resource | R13, R25 | Measurements | TEST-2026-0008 | within thresholds | **PASS** |
| — | manual | — | **NOT APPLICABLE** — no UI or HTTP surface exists in this project | — | — | — |

## Documentation and Demo Checklist

| Artifact | Exact Path | Why It Changes | Required Addition | Status |
|---|---|---|---|---|
| Diagnostics reference | `docs/diagnostics.md` | New error behaviour users read | CREATE — message selection, recovery, resync order, codes | **DONE** |
| AST JSON schema | `docs/ast-json.md` | New machine-readable surface | CREATE — versioned schema, span semantics | **DONE** |
| Nav | `mkdocs.yml` | Two new pages | UPDATE | **DONE** |
| Embedding guide | `docs/embedding.md` | New public API | UPDATE — `parse`, `ParseResult`, R28 warning | **DONE** |
| CLI reference | `docs/cli.md` | New subcommand | UPDATE — `capy ast`, exit codes | **DONE** |
| Language reference | `docs/language-reference.md` | Its arithmetic claim became false | UPDATE — precedence table | **DONE** |
| Inner DSL | `docs/inner-dsl.md` | Operators usable in conditions | UPDATE | **DONE** |
| What's new | `docs/whats-new.md` | User-visible change (CODE-002 pattern) | UPDATE | **DONE** |
| Release verification guide | DEMO-2026-0002 | §29, mandatory | CREATE — four `U-NN`, all executed | **DONE** |
| Manual | MAN-2026-0001 | Reader needs the new behaviour | UPDATE — revision 2 | **DONE** |
| System documentation | SYS-2026-0001 | Pipeline and value parser changed | UPDATE — revision 2 | **DONE** |
| Architecture | ARCH-2026-0001 | Error-node representation realised | UPDATE — revision 2 | **DONE** |

## Section 31 Documentation-Impact Decision

A blank row fails the release gate.

| Artifact | Required result | Decision |
|---|---|---|
| Release verification guide / demo | every release | **UPDATED** — DEMO-2026-0002, all four updates executed and PASS |
| Top-level `README.md` | significant user-visible capability, setup, workflow or headline change | **NOT APPLICABLE** — no install step or primary workflow changes. `capy ast` is a new subcommand, documented in `docs/cli.md`; the README does not enumerate subcommands |
| `system/` | implemented behaviour or internal interfaces changed | **UPDATED** — SYS-2026-0001 revision 2 |
| `architecture/` | boundaries, components or data flow changed | **UPDATED** — ARCH-2026-0001 revision 2 |
| `api/` and CLI reference | commands, flags, schemas or errors changed | **UPDATED** — `docs/cli.md` (new `capy ast` + `--json`), `docs/ast-json.md` (schema), `docs/diagnostics.md` (codes) |
| `manuals/` | a reader needs new knowledge | **UPDATED** — MAN-2026-0001 revision 2 |

## Version, Release and Rollout Checklist

| Item | Source / Target | Required Action | Status | Evidence |
|---|---|---|---|---|
| Version | `rust/Cargo.toml` `[workspace.package]` | 0.21.0 → 0.22.0 | **DONE** | `capy-core v0.22.0` |
| Dependent pins | 5 member manifests | Follow the workspace version | **DONE** | workspace resolves |
| Lockfile | `rust/Cargo.lock` | Regenerate | **DONE** | build |
| Release notes | `program_docs/releases/rel-0.22.0-release-notes.md` | Author per §33 | **DONE** | REL-0.22.0 |
| Release commit | repository | Commit; record hash | **DONE** | `f72fa3d` |
| Git tag | repository | Create and verify `v0.22.0` | **DONE** | tag at `f72fa3d` |
| Rollout | development only | No deployed environment; not published to crates.io | **DONE** | — |
| Post-release verification | DEMO-2026-0002 | Re-run against the tag | **DONE** | U-01 verified at the tag |
| Finalize release doc | `program_docs/releases/` | Hash, status | **DONE** | revision 2 |

## Decisions, Findings, Deviations and Blockers

| Timestamp | Type | Task / Requirement | Finding or Decision | Impact | Owner | Linked |
|---|---|---|---|---|---|---|
| 2026-09-16T00:00:00+08:00 | Finding | D-01 / TASK-001 | `gojson::marshal` handles nested `Obj` and `List` recursively | R8 met with no new dependency; ARCH-002 holds | Capy Engine | TEST-2026-0007 |
| 2026-09-16T00:00:00+08:00 | Deviation | R10 / TASK-017 | Parenthesised grouping needed a narrower rule than R10 implied: `(` is the prefix-call form, so grouping applies only when the contents parse as a complete expression that is not a bare identifier | `(upper n)` stays a call; `(foo)` stays a zero-arg call; GOAL-002 preserved | Capy Engine | RPT-2026-0002 |
| 2026-09-16T00:00:00+08:00 | Finding | R21 / TASK-011 | **Resync consumed whole files.** "Never resync inside an unclosed bracket" is correct only for brackets that eventually close; when one never does, depth stays above zero to EOF. Caught by T-20 with 0 statements surviving | A line break inside an unclosed delimiter is now taken as evidence it is missing | Capy Engine | TEST-2026-0005 |
| 2026-09-16T00:00:00+08:00 | Decision | R12 / TASK-020 | One error golden updated deliberately rather than regenerated; the old message named the wrong construct | QUAL-001 satisfied; six other error goldens unchanged | Capy Engine | RPT-2026-0002 |
| 2026-09-16T00:00:00+08:00 | Decision | plan scope | PLAN-B…E consolidated into one plan and one release | Recorded in *Objective, Scope and Proposal Baseline*; permitted by §4.5 | Capy Engine | ADR-0002 |

## Rollout Strategy

Single-environment (development), shipped as a tagged minor release. Consumers
adopt by bumping their git dependency. Not published to crates.io.

## Completion Criteria and Final Traceability

| Requirement | Implementation | Files | Tests | Docs / Demo | Release | Status |
|---|---|---|---|---|---|---|
| R14–R17 | C-08…C-10 | F-01, F-03 | T-14…T-17 | `docs/diagnostics.md` | REL-0.22.0 Changed | **PASS** |
| R18, R26 | C-11 | F-01 | T-17, T-28 | `docs/diagnostics.md`, MAN | REL-0.22.0 Added | **PASS** |
| R19–R23 | C-12 | F-02, F-03, F-08 | T-18…T-22 | `docs/diagnostics.md`, DEMO U-02 | REL-0.22.0 Added | **PASS** |
| R24 | C-12 | F-09 | T-25 | `docs/embedding.md` | REL-0.22.0 Upgrade Notes | **PASS** |
| R6, R28 | C-03 | F-09, F-20 | T-02 | `docs/embedding.md` | REL-0.22.0 Added | **PASS** |
| R7–R9 | C-04 | F-10…F-13, F-17, F-18 | T-04…T-06 | `docs/cli.md`, `docs/ast-json.md`, DEMO U-03 | REL-0.22.0 Added | **PASS** |
| R10, R11 | C-06 | F-04…F-07 | T-07, T-08, T-11, **T-27** | `docs/language-reference.md`, DEMO U-04 | REL-0.22.0 Added | **PASS** |
| R12 | C-05, C-06, C-12 | F-16 | T-12, T-26, T-27, T-30 | — | REL-0.22.0 | **PASS** |
| R13, R25 | C-04, C-09 | — | M-01…M-03 | TEST-2026-0008 | REL-0.22.0 Measured | **PASS** |

**Reverse check.** Every `F-NN` maps to a requirement and a task; every `T-NN`
maps to a requirement or is marked `NOT APPLICABLE` with a reason; no requirement
lacks an implementation or a validation method.

## Post-Implementation Review

Two findings are worth carrying forward.

**A discovery answer that looks favourable can still be incomplete.** D-03 in the
predecessor plan established that left recursion is statically decidable, which
made the parse-time depth bound look like a backstop. It was not: deeply nested
*input* against a perfectly valid library still aborted. The same shape recurred
here — a resync rule that is correct for well-formed input was exactly wrong for
the malformed input it exists to handle.

**The compiler is a better reviewer than a checklist for exhaustive changes.**
Adding `Expr::Binary` produced a compile error at every site that had to learn
it, including `expr_to_text` — the one whose omission would have corrupted output
silently. The checklist in the plan named three of those sites; the compiler
named five.

**On consolidating four plans into one:** correct here, and it should not become
a habit. It worked because the four increments genuinely could not ship
separately. Where increments *are* independently releasable, four ledgers give
four honest status summaries, and this one had to carry four phases' worth of
findings in a single Decisions table.

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
| 3 | 2026-09-16 | Olivier | Added the eight §12.4 sections this plan was written without (File and Artifact Checklist, Test and Validation Checklist, Documentation and Demo Checklist, §31 decision, Version/Release Checklist, Decisions table, Rollout Strategy, Completion Criteria, Post-Implementation Review). The work they describe was done; the plan simply did not record it in the required form. |
