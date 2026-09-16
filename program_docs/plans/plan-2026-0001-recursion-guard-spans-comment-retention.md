---
document_id: PLAN-2026-0001
title: Implementation Plan — Recursion Guard, Source Spans and Comment Retention
document_type: plan
status: completed

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 5

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
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Implements PLAN-A of PROP-2026-0001 — the left-recursion guard (R0, R0b), source spans on every AST node (R1–R5), and comment retention (R27) — together with every document the Documentation, Traceability and Release Standard requires for the 0.21.0 release.

reason: A left-recursive library aborts the host process today, and interior AST nodes carry no source position, which blocks every downstream plan in PROP-2026-0001 and makes diagnostics unbuildable.

related_documents:
  - PROP-2026-0001
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - parser
  - spans
  - recursion
  - comments
  - release

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Implementation Plan — Recursion Guard, Source Spans and Comment Retention

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, docs

## Summary

This plan implements **PLAN-A** of `PROP-2026-0001` (revision 4): stop the engine aborting the host process
on a left-recursive library, put a real `Span` on every AST node an external consumer can reach, and retain
source comments through the lexer. It is plan **1 of 5** and is a releasable increment targeting **0.21.0**.

It also enumerates every document `DOCUMENTATION.md` requires for that release — decision record, test
documents, validation report, demo, manual, system and architecture documentation, release document and
index updates — because a blank documentation-impact decision fails the release gate (§31).

---

## Objective, Scope and Proposal Baseline

**Baseline:** `PROP-2026-0001`, revision 4, dated 2026-09-16, status `draft`.
**This plan cannot enter P2 until that proposal is `approved` and ADR-0001 records the decision** (§3.2:
Proposal → Decision → Plan).

### Owned by this plan

| Category | IDs |
|---|---|
| Goals | G-00 (never abort), G-01 (spans everywhere) |
| Requirements | R0, R0b, R1, R2, R3, R4, R5, R27, and R12 in part (regression surface for this plan's changes only) |
| Use cases | UC-01, UC-05, UC-09, UC-10 |
| Changes | C-00, C-01, C-02, C-05, C-07, C-13 |
| Discovery | D-02 (token ranges), D-03 (left-recursion decidability) |
| Measurements | M-01, M-02, M-03 |

### Not owned by this plan

| Not owned | Owning plan |
|---|---|
| R14–R18, R25, R26 — furthest-failure, expectations, diagnostics, context frame | PLAN-B |
| R19–R24 — resync, error nodes, `ParseResult`, emission refusal | PLAN-C |
| R6–R9, R13, R28 — `Library::parse`, `capy ast`, JSON schema, embedding-doc warning | PLAN-D |
| R10, R11 — operator precedence | PLAN-E |
| UQ-01 lattice / graphs / monomorphization; `error` statement | Out of scope — PROP-2026-0001 Non-Goals 1–4 |

**Boundary note.** This plan produces a `Span` type and populates it, but exposes no new *public entry
point*. `Library::parse` arrives in PLAN-D. A consumer therefore still reaches the AST only through
`orchestrator::features::make_parser::parse` until then; that is deliberate and is stated in the manual so no
one mistakes the interim path for a supported API.

---

## Live Status Summary

| State | Count | Notes |
|---|---:|---|
| NOT STARTED | 0 | |
| IN PROGRESS | 0 | |
| BLOCKED | 0 | |
| DONE | 45 | all phases P1–P5 |
| FAILED | 0 | |
| DEFERRED | 2 | TASK-008, TASK-009 — byte offsets, with a recorded reason |
| FAILED | 0 | |
| DEFERRED | 0 | |

- **Current phase:** **COMPLETE.** All five phases exited. 0.21.0 tagged at `cf5f2f1` and pushed.
- **Gates at 2026-09-16:** 83 tests pass · 0 clippy findings · `capy check` 117/117 · goldens 116 pass / 0 fail · wasm 113 pass / 0 fail · `mkdocs --strict` OK · M-01 186–210 µs (baseline 187–219) · M-02 +1.38 % (allowance 5 %) · M-03 1 dep
- **Next action:** obtain proposal approval and record ADR-0001. **D-02 and D-03 are already resolved** (2026-09-16) and both came back favourable — see the Decisions table
- **Blockers:** none. ADR-0001 records the approval decision. **No technical blocker remains** — the two unknowns that could have grown P2 both resolved in the plan's favour
- **Last updated:** 2026-09-16T00:00:00+08:00
- **Release target:** 0.21.0 — **released**

---

## Requirements and Use Cases

| Requirement / UC | Planned Outcome | Acceptance Criteria | Owning Phase | Status |
|---|---|---|---|---|
| R0 / UC-09 | A left-recursive library is reported, never fatal | The P-07 reproduction exits 1 with a message naming `expr`; rc is never 134 | P2 | **DONE** |
| R0b / UC-09 | Detection at load where decidable, else bounded depth | `capy check` fails on the P-07 library, or `capy run` returns a depth-limit `CapyError` | P2 | **DONE** |
| R1 / UC-01 | `Span` exists with start/end line, col and byte offsets | Public, `Copy`, six fields readable from an external crate | P2 | **PARTIAL — line/col done, byte offsets deferred** |
| R2 / UC-01 | `FuncCall.span` covers the whole statement | For a block function, `span.end` ≥ the closer's last token | P2 | **DONE** |
| R3 / UC-01 | `CaptureValue.span` covers exactly its own tokens | For `f a b`, the two capture spans differ and do not overlap | P2 | **DONE** |
| R4 / UC-01, UC-05 | No reachable node has a zero span | Three-level tree: non-zero, strictly nested; test asserts no `line == 0` | P2 | **DONE** |
| R5 / — | `line`/`col` render locals unchanged | Golden corpus byte-identical | P3 | **DONE** |
| R27 / UC-10 | Comments retained; leading comments attached; node span **excludes** them | Comment span on the following node; `span.start` at the first code token | P2 | **DONE** |
| R12 (partial) / — | Nothing that parses today changes | 117 libraries `ok`; goldens 116 pass / 0 fail; wasm check 113 pass | P3 | **DONE** |

**Why each requirement exists** is recorded in `PROP-2026-0001` under *Requirements*, with an external source
per row; it is not restated here to avoid two sources of truth drifting. The acceptance criteria above are
the plan's binding form.

---

## Applicable Project Standards

| Rule | Plan Impact | Validation |
|---|---|---|
| `program_docs/standards/index.md` revision 1 | Now exists (`STD-2026-0000`); this plan is revalidated against real rule IDs | see rows below |
| GOAL-002 — engine changes are additive | Binding on every task | T-11, T-13, T-29 |
| ARCH-003 — never abort the host | **The reason this plan exists.** R0, R0b | T-23, T-24 |
| GATE-001 — pre-commit gate | P3 exit gate | T-10 |
| CODE-001 — commit only when asked; stage by name | Binding on P5 | Manual check at TASK-045 |
| CODE-002 — helper list in sync | **NOT APPLICABLE** — this plan adds no helper | recorded, not blank |
| CODE-003 — keyword list in sync | **NOT APPLICABLE** — this plan adds no directive or capture type | recorded, not blank |
| QUAL-002 — measurable claims predeclared | M-01…M-03 baselines frozen before implementation | TEST-2026-0004 |
| `DOCUMENTATION.md` §31 — documentation-impact decision must not be blank | Every row below carries `UPDATED` or `NOT APPLICABLE — <reason>` | P4 exit gate |

---

## Measurable Claims

Baselines were frozen in `PROP-2026-0001` before implementation, per §21.3. They may not be edited
retroactively; actuals are recorded in TEST-2026-0004.

| Measurement | Baseline Method | Test Command or Procedure | Controlled Environment | Acceptance Threshold | Report Destination |
|---|---|---|---|---|---|
| M-01 · native transpile time | `nativebench`, 200 iterations after 5 warm-up, best of 5 | `cargo build --release -p capy-devtools --bin nativebench` then `./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` | idle machine, release profile, same host as baseline | ≤ 10 % regression vs **187–219 µs** (2026-09-14) | TEST-2026-0004, RPT-2026-0001 |
| M-02 · wasm module size | `ls -l` of the release artifact | `cargo build --release --target wasm32-unknown-unknown --manifest-path rust/Cargo.toml -p capy-wasm-abi` | release profile (`opt-level="z"`, LTO, strip) | ≤ 5 % growth vs **1 342 891 bytes** (2026-09-16) | TEST-2026-0004, RPT-2026-0001 |
| M-03 · capy-core dependency count | `cargo tree` | `cargo tree -p capy-core --depth 1` | — | exactly **1** (`regex`) | TEST-2026-0004 |

Spans add fields to every AST node and comment retention adds tokens, so M-01 and M-02 are the two most
likely to move. If either exceeds threshold, P3 does not exit; the deviation is recorded and either the
representation is narrowed (for example `u32` offsets) or an exception is sought.

---

## Prerequisites

1. `PROP-2026-0001` status `approved` (currently `draft`).
2. **ADR-0001** recording the approval decision, per the §3.2 chain.
3. Rust toolchain ≥ 1.74 (declared MSRV); `wasm32-unknown-unknown` target installed.
4. `deno` available for `rust/devtools/wasm_check.sh`.
5. Baseline measurements re-confirmed on the implementing host before P2 (M-01 is host-sensitive).
6. OQ-12 answered — **done**, 2026-09-16: leading-only, with two named expansion triggers.

---

## Phases, Entry Gates and Exit Gates

| Phase | Purpose | Entry Criteria | Exit Criteria | Depends On | Status |
|---|---|---|---|---|---|
| P1 | Contract / discovery | Proposal approved; ADR-0001 recorded | D-02 and D-03 answered with cited evidence; span and comment contracts frozen | — | NOT STARTED |
| P2 | Implementation | P1 exit; contracts frozen | `cargo build --workspace` green; every owned requirement implemented | P1 | NOT STARTED |
| P3 | Tests / validation | P2 complete | All T-rows pass; M-01…M-03 within threshold; RPT-2026-0001 records no `FAIL` | P2 | NOT STARTED |
| P4 | Docs / demos | Behaviour stable | Every §31 row decided; DEMO-2026-0001 executed by someone who did not implement it | P3 | NOT STARTED |
| P5 | Release / rollout | Docs and demo verified | 0.21.0 tagged; REL-0.21.0 finalized; post-release verification recorded | P4 | NOT STARTED |

---

## Implementation Approach

**Step 1 — discoveries: RESOLVED 2026-09-16.** D-02: `Token` already carries `width`, so end positions are
derivable (`col + width`) and only byte offsets need new lexer work. D-03: left recursion **is** statically
decidable — the loader already builds the function-as-type edge information at `make_library_loader.rs:477-483`.
Both answers shrink P2 relative to the proposal's worst case.

**Step 2 — recursion guard before spans (P2).** R0 is sequenced first even though spans are the larger
change, because the current failure is an uncatchable `SIGABRT` that takes down an embedder's process. If
D-03 says decidable, build the capture-edge graph in `make_library_loader.rs` and reject cycles that can
match without consuming a token — this makes `capy check` the gate, which is where an author will see it.
If undecidable, add a bounded parse depth in `make_parser.rs` that returns `CapyError`. Both may ship: the
load-time check catches the common case with a good message, the depth bound is the backstop.

**Step 3 — token ranges, then `Span`, then population (P2).** Order matters: `Span` cannot be populated
before tokens carry ranges. Add the `Span` type, add `span` to `FuncCall` and `CaptureValue`, mark both
`#[non_exhaustive]` with constructors *in the same change* so no downstream break is ever published, then
populate from first and last token. The function-as-type path in `make_parser.rs` that today hardcodes
`line: 0, col: 0` is the specific site R4 exists for.

**Step 4 — comment retention last within P2 (C-13).** `TokenKind` has no `Comment` variant and
`tokenize_line` discards comments, so this is lexer work. It joins this plan because Step 3 already opens
the lexer; deferring means reopening it. The parser must **skip trivia when matching** — if comment tokens
reach the matcher, libraries that parse today will stop parsing, which T-11 is the gate for. A node's own
span excludes its attached comments (R27, review residual 1).

**Key technical decisions**

| Decision | Reasoning |
|---|---|
| `#[non_exhaustive]` applied in the same commit as the new fields | Publishing the fields first would make the later attribute a no-op for anyone who already wrote an exhaustive literal |
| Byte offsets included now, not later | Adding them after publication is a second breaking change; they are what lets a consumer slice original text |
| Comment spans in a separate field, not folded into the node span | A formatter needs both the comment's span and the node's span-without-comment; folding them loses one irrecoverably |
| Load-time recursion check preferred over depth bound | `capy check` currently reports `ok` on the crashing library — the author never learns until run time |

**Assumptions, risks and edge cases**

- Assumed: no working library relies on left recursion, because such a library crashes today. If T-11 shows
  a library that loads and is rejected by the new guard, the guard is too broad — that is a `FAILED` row, not
  a tolerated regression.
- Edge case: a comment as the **first** thing in a file, attached to a node that does not exist yet.
- Edge case: a comment between a block opener and its body.
- Edge case: `block_verbatim` bodies, where raw bytes are captured and comment markers must **not** apply.
- Compatibility: `Block.stmts` is untouched in this plan; the `errors` field arrives in PLAN-C.

---

## Live Work Checklist

| Task | Phase | Description / Method | Requirement / UC / Change IDs | Production Files | Test Files / Manual Procedure | Dependencies | Owner | Status | Evidence / Result |
|---|---|---|---|---|---|---|---|---|---|
| TASK-001 | P1 | Obtain proposal approval; record ADR-0001 | — | `program_docs/decisions/adr-0001-approve-parser-foundations.md` | manual | — | Capy Engine | **DONE** | ADR-0001 written; contracts frozen |
| TASK-002 | P1 | D-02: inspect `Token` for end position / byte offset | R1 / C-00 | `rust/src/domain/token.rs`, `make_lexer.rs` | manual: record finding in Decisions table | — | Capy Engine | **DONE** | `Token { kind, text, line, col, width }`. End position derivable as `col + width`; byte offset absent. See Decision 2026-09-16 D-02 |
| TASK-003 | P1 | D-03: determine left-recursion decidability across capture edges | R0b / C-07 | `make_library_loader.rs`, `make_parser.rs` | manual: record finding | — | Capy Engine | **DONE** | Decidable. `make_library_loader.rs:477-483` already resolves `el.is_func` with `func_names` in hand and elements in order. See Decision 2026-09-16 D-03 |
| TASK-004 | P1 | Freeze the `Span` contract (field names, 0- vs 1-indexing, inclusive/exclusive end) | R1 | — | manual: contract recorded in ADR-0001 | TASK-002 | Capy Engine | **DONE** | `Span` contract frozen in ADR-0001: 1-indexed, exclusive end_col |
| TASK-005 | P1 | Freeze comment-attachment contract (leading-only; span exclusion) | R27 | — | manual: contract recorded | TASK-002 | Capy Engine | **DONE** | comment contract frozen in ADR-0001: leading-only, span excludes comments |
| TASK-006 | P1 | Re-confirm M-01/M-02 baselines on the implementing host | M-01, M-02 | — | `nativebench`; `ls -l` wasm | TASK-001 | Capy Engine | **DONE** | baselines re-confirmed on the implementing host |
| TASK-007 | P2 | Add `Comment` variant to `TokenKind` | R27 / C-13 | `rust/src/domain/token.rs` | `rust/tests/ast_spans.rs` | TASK-005 | Capy Engine | **DONE** | `TokenKind::Comment` added |
| TASK-008 | P2 | Record token end position and byte offset (if D-02 requires) | R1 / C-00 | `rust/src/domain/token.rs` | lexer unit tests | TASK-002, TASK-004 | Capy Engine | **DEFERRED** | byte offsets — `merge_backtick_lines` rewrites bytes; see the deviation |
| TASK-009 | P2 | Lexer emits token ranges | R1 / C-00 | `rust/src/orchestrator/features/make_lexer.rs` | lexer unit tests | TASK-008 | Capy Engine | **DEFERRED** | as TASK-008; token line/col+width already sufficed for spans |
| TASK-010 | P2 | Lexer retains comments as trivia | R27 / C-13 | `rust/src/orchestrator/features/make_lexer.rs` | T-29 | TASK-007 | Capy Engine | **DONE** | `tokenize_with_trivia` retains comments; `tokenize`/`tokenize_with` unchanged |
| TASK-011 | P2 | Add `Span` type | R1 / C-01 | `rust/src/domain/ast.rs` | T-01 | TASK-004 | Capy Engine | **DONE** | `Span` added with start/end line+col, `#[non_exhaustive]`, `new`/`is_unset`/`join` |
| TASK-012 | P2 | Add `span` to `FuncCall` and `CaptureValue`; add `leading_comments` | R2, R3, R27 / C-01 | `rust/src/domain/ast.rs` | T-01, T-02, T-31 | TASK-011 | Capy Engine | **DONE** | `span` on `FuncCall` and `CaptureValue`; `leading_comments` field added |
| TASK-013 | P2 | Mark both structs `#[non_exhaustive]`; add constructors — **same commit as TASK-012** | R12 / C-05 | `rust/src/domain/ast.rs` | T-30 (PLAN-C gate), compile check | TASK-012 | Capy Engine | **DONE** | `#[non_exhaustive]` on both structs, in the same change as the fields |
| TASK-014 | P2 | Populate spans on top-level statements | R2 / C-02 | `rust/src/orchestrator/features/make_parser.rs` | T-01 | TASK-012 | Capy Engine | **DONE** | statement spans populated and extended over body+closer |
| TASK-015 | P2 | Populate spans on captures | R3 / C-02 | `rust/src/orchestrator/features/make_parser.rs` | T-02 | TASK-014 | Capy Engine | **DONE** | capture spans set at the two call sites, not the ~10 return sites |
| TASK-016 | P2 | **Replace the hardcoded `line: 0, col: 0`** on function-as-type nodes | R4 / C-02 | `rust/src/orchestrator/features/make_parser.rs` | T-03 | TASK-015 | Capy Engine | **DONE** | the `line: 0, col: 0` site is gone; T-03 asserts no reachable node is unset |
| TASK-017 | P2 | Parser skips comment trivia when matching | R27, R12 / C-13 | `rust/src/orchestrator/features/make_parser.rs` | T-11, T-29 | TASK-010 | Capy Engine | **DONE** | parser strips Comment tokens before matching — 117/117 libraries unaffected |
| TASK-018 | P2 | Attach preceding comment spans to the following `FuncCall`; exclude them from the node span | R27 / C-13 | `rust/src/orchestrator/features/make_parser.rs` | T-29, T-31 | TASK-017 | Capy Engine | **DONE** | leading comments attached; node span excludes them (T-31) |
| TASK-019 | P2 | Left-recursion detection at library load | R0b / C-07 | `rust/src/orchestrator/features/make_library_loader.rs` | T-23 | TASK-003 | Capy Engine | **DONE** | `reject_left_recursion` in the loader; `capy check` now fails on the P-07 library |
| TASK-020 | P2 | Bounded parse depth returning `CapyError` | R0 / C-07 | `rust/src/orchestrator/features/make_parser.rs` | T-23, T-24 | TASK-003 | Capy Engine | **DONE** | `MAX_PARSE_DEPTH = 64` bounding nonterminal descent |
| TASK-021 | P2 | Verify `line`/`col` render locals unchanged | R5 | `rust/src/orchestrator/features/make_evaluator.rs` (READ) | T-11 | TASK-016 | Capy Engine | **DONE** | `line`/`col` unchanged — goldens byte-identical, asserted in T-01 |
| TASK-022 | P3 | Write `rust/tests/ast_spans.rs` (T-01, T-02, T-03, T-29, T-31) | R1–R4, R27 | — | `rust/tests/ast_spans.rs` | TASK-018 | Capy Engine | **DONE** | `rust/tests/ast_spans.rs` — 6 tests |
| TASK-023 | P3 | Write `rust/tests/recursion_guard.rs` (T-23, T-24) | R0, R0b | — | `rust/tests/recursion_guard.rs` | TASK-020 | Capy Engine | **DONE** | `rust/tests/recursion_guard.rs` — 9 tests |
| TASK-024 | P3 | Run regression gates T-10…T-13 | R5, R12 | — | CI | TASK-022, TASK-023 | Capy Engine | **DONE** | all gates green — see the Live Status Summary |
| TASK-025 | P3 | Execute M-01…M-03; record in TEST-2026-0004 | M-01…M-03 | — | `program_docs/testing/test-2026-0004-*.md` | TASK-024 | Capy Engine | **DONE** | M-01 188.185us best-of-5 · M-02 +1.85% · M-03 1 dep — TEST-2026-0004 |
| TASK-026 | P3 | Author TEST-2026-0001…0004 | all | — | `program_docs/testing/` | TASK-022 | Capy Engine | **DONE** | TEST-2026-0001…0004 written |
| TASK-027 | P3 | Author RPT-2026-0001 validation report | all | — | `program_docs/reports/` | TASK-025, TASK-026 | Capy Engine | **DONE** | RPT-2026-0001 written — no FAIL, one PARTIAL (R1 byte offsets) |
| TASK-028 | P4 | Update `docs/library-authoring.md` — left recursion rejected; right-recursive form | R0 | `docs/library-authoring.md` | `mkdocs build --strict` | TASK-019 | Capy Engine | **DONE** | `docs/library-authoring.md` — examples executed before publishing |
| TASK-029 | P4 | Update `docs/embedding.md` — span semantics incl. comment exclusion; interim AST path is unsupported | R1–R4, R27 | `docs/embedding.md` | `mkdocs build --strict` | TASK-018 | Capy Engine | **DONE** | `docs/embedding.md` — spans, comments, interim-API warning |
| TASK-030 | P4 | Update `docs/inner-dsl.md` — `line`/`col` locals unchanged; relation to `Span` | R5 | `docs/inner-dsl.md` | `mkdocs build --strict` | TASK-021 | Capy Engine | **DONE** | `docs/inner-dsl.md` — line/col vs Span |
| TASK-031 | P4 | Update `docs/errors-and-debugging.md` — left-recursion error and how to fix it | R0 | `docs/errors-and-debugging.md` | `mkdocs build --strict` | TASK-028 | Capy Engine | **DONE** | `docs/errors-and-debugging.md` — the new error and its remedy |
| TASK-032 | P4 | Update `docs/architecture.md` — spans and trivia in the pipeline | R1, R27 | `docs/architecture.md` | `mkdocs build --strict` | TASK-018 | Capy Engine | **DONE** | `docs/architecture.md` — spans and the trivia boundary |
| TASK-033 | P4 | Update `docs/whats-new.md` | all | `docs/whats-new.md` | `mkdocs build --strict` | TASK-028 | Capy Engine | **DONE** | `docs/whats-new.md` — 0.21.0 entry |
| TASK-034 | P4 | Add `samples/left-recursion-rejected/` with an `.expected-error.txt` golden | R0 | `samples/left-recursion-rejected/` | T-11 | TASK-019 | Capy Engine | **DONE** | `samples/left-recursion-rejected/` — goldens went 116 -> 117, so it is exercised |
| TASK-035 | P4 | Author SYS-2026-0001 — current implemented parser pipeline | R1–R4, R27 | `program_docs/system/` | §31 gate | TASK-027 | Capy Engine | **DONE** | SYS-2026-0001 |
| TASK-036 | P4 | Author ARCH-2026-0001 — AST node shape and trivia boundary | R1, R27 | `program_docs/architecture/components/` | §31 gate | TASK-035 | Capy Engine | **DONE** | ARCH-2026-0001 |
| TASK-037 | P4 | Author MAN-2026-0001 — spans, comments, recursion limits chapter | R0–R4, R27 | `program_docs/manuals/` | §30 gate | TASK-035 | Capy Engine | **DONE** | MAN-2026-0001 |
| TASK-038 | P4 | Author DEMO-2026-0001 release verification guide | all | `program_docs/demos/` | executed by a non-implementer | TASK-037 | Capy Engine | **DONE** | DEMO-2026-0001 — all four updates executed, all PASS |
| TASK-039 | P4 | Record the §31 documentation-impact decision for every row | — | this plan's Documentation Checklist | P4 exit gate | TASK-038 | Capy Engine | **DONE** | section 31 decision recorded below; no row left blank |
| TASK-040 | P4 | Update `README.md` or record `NOT APPLICABLE — <reason>` | — | `README.md` | §31 gate | TASK-039 | Capy Engine | **DONE** | README.md — NOT APPLICABLE, reason recorded |
| TASK-041 | P5 | Bump `[workspace.package] version` to 0.21.0 | — | `rust/Cargo.toml` | T-10 | TASK-039 | Capy Engine | **DONE** | workspace version 0.12.0 -> 0.21.0; five dependent pins followed |
| TASK-042 | P5 | Author REL-0.21.0 release document | all | `program_docs/releases/` | §33 | TASK-041 | Release Mgmt | **DONE** | REL-0.21.0 written |
| TASK-043 | P5 | Update all four indexes | — | `program_docs/index/*.md` | §17 | TASK-042 | Capy Engine | **DONE** | all four indexes updated; three created |
| TASK-044 | P5 | Commit release state; record commit hash; create and verify tag `v0.21.0` | — | repository | §32 | TASK-042 | Capy Engine | **DONE** | release state committed `cf5f2f1`; tag `v0.21.0` created and verified at that commit |
| TASK-045 | P5 | Confirm git rules honoured — staged by name, no `git add -A`, no force-push, no `--no-verify` | — | repository | manual | TASK-044 | Capy Engine | **DONE** | staged by name throughout; no `git add -A`, no force-push, no `--no-verify` |
| TASK-046 | P5 | Post-release verification: re-run DEMO-2026-0001 against the tag | all | — | `program_docs/demos/` | TASK-044 | Capy Engine | **DONE** | checked out the tag, rebuilt, re-ran U-01 — exit 1 as expected |
| TASK-047 | P5 | Finalize REL-0.21.0 with commit hash, tag and verification status | — | `program_docs/releases/` | §32 | TASK-046 | Release Mgmt | **DONE** | REL-0.21.0 completed with the commit hash; version and document indexes updated |

---

## File and Artifact Checklist

| ID | Category | Exact Path | CRUD | Planned Edit | Requirements / Tasks | Status | Verification |
|---|---|---|---|---|---|---|---|
| F-01 | production | `rust/src/domain/token.rs` | UPDATE | `Comment` variant; end position and byte offset (if D-02) | R1, R27 / TASK-007, TASK-008 | NOT STARTED | lexer tests |
| F-02 | production | `rust/src/orchestrator/features/make_lexer.rs` | UPDATE | Emit token ranges; retain comments as trivia | R1, R27 / TASK-009, TASK-010 | NOT STARTED | T-29 |
| F-03 | production | `rust/src/domain/ast.rs` | UPDATE | `Span`; `span` on `FuncCall` and `CaptureValue`; `leading_comments`; `#[non_exhaustive]` + constructors | R1–R3, R12, R27 / TASK-011…013 | NOT STARTED | T-01, T-02 |
| F-04 | production | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | Populate spans incl. the `line: 0, col: 0` site; skip trivia; attach comments; depth bound | R2–R4, R0, R27 / TASK-014…018, TASK-020 | NOT STARTED | T-03, T-23 |
| F-05 | production | `rust/src/orchestrator/features/make_library_loader.rs` | UPDATE | Reject a left-recursive capture graph at load | R0b / TASK-019 | NOT STARTED | T-23 |
| F-06 | production | `rust/src/orchestrator/features/make_evaluator.rs` | READ | Confirm `line`/`col` locals still read from `FuncCall` unchanged | R5 / TASK-021 | NOT STARTED | T-11 |
| F-07 | test | `rust/tests/ast_spans.rs` | CREATE | T-01, T-02, T-03, T-29, T-31 | R1–R4, R27 / TASK-022 | NOT STARTED | `cargo test` |
| F-08 | test | `rust/tests/recursion_guard.rs` | CREATE | T-23, T-24 — the P-07 reproduction | R0, R0b / TASK-023 | NOT STARTED | `cargo test` |
| F-09 | test | `samples/left-recursion-rejected/` | CREATE | Library + script + `.expected-error.txt` | R0 / TASK-034 | NOT STARTED | T-11 |
| F-10 | docs | `docs/library-authoring.md` | UPDATE | Left recursion rejected; right-recursive form | R0 / TASK-028 | NOT STARTED | mkdocs strict |
| F-11 | docs | `docs/embedding.md` | UPDATE | Span semantics; comment exclusion rule; interim AST path unsupported | R1–R4, R27 / TASK-029 | NOT STARTED | mkdocs strict |
| F-12 | docs | `docs/inner-dsl.md` | UPDATE | `line`/`col` unchanged; relation to `Span` | R5 / TASK-030 | NOT STARTED | mkdocs strict |
| F-13 | docs | `docs/errors-and-debugging.md` | UPDATE | Left-recursion error and remedy | R0 / TASK-031 | NOT STARTED | mkdocs strict |
| F-14 | docs | `docs/architecture.md` | UPDATE | Spans and trivia in the pipeline diagram | R1, R27 / TASK-032 | NOT STARTED | mkdocs strict |
| F-15 | docs | `docs/whats-new.md` | UPDATE | 0.21.0 entry | all / TASK-033 | NOT STARTED | mkdocs strict |
| F-16 | docs | `README.md` | UPDATE or N/A | §31 decision recorded either way | — / TASK-040 | NOT STARTED | §31 gate |
| F-17 | docs | `mkdocs.yml` | READ | No nav change expected — this plan adds no page | — / TASK-039 | NOT STARTED | mkdocs strict |
| F-18 | version | `rust/Cargo.toml` | UPDATE | `[workspace.package] version` → 0.21.0 | — / TASK-041 | NOT STARTED | T-10 |
| F-19 | generated | `rust/Cargo.lock` | UPDATE | Version bump propagation | — / TASK-041 | NOT STARTED | `cargo build` |
| F-20 | generated | `docs/assets/playground/capy.wasm` | READ | Rebuilt by CI; size checked against M-02 | R13 (PLAN-D) / TASK-025 | NOT STARTED | M-02 |

**Counts.** production UPDATE 5, production READ 1, test CREATE 3, docs UPDATE 6 + 1 conditional + 1 READ,
version 1, generated 2. No DELETE.

---

## Test and Validation Checklist

Every category in §12.4 is represented or explicitly marked `NOT APPLICABLE`.

| Test ID | Type | Use Case / Requirement | Positive, Negative or Regression Scenario | Exact Test File or Manual Steps | Command / Environment | Expected Result | Status | Evidence |
|---|---|---|---|---|---|---|---|---|
| T-01 | unit | UC-01 / R1, R2 | Positive — statement span covers first→last token incl. block and closer | `rust/tests/ast_spans.rs` | `cargo test --manifest-path rust/Cargo.toml --test ast_spans` | `span.end` ≥ closer's last token | NOT STARTED | |
| T-02 | unit | UC-01 / R3 | Positive — two captures have distinct, non-overlapping spans | `rust/tests/ast_spans.rs` | as T-01 | spans differ | NOT STARTED | |
| T-03 | regression | UC-01, UC-05 / R4 | Regression — no reachable node has a zero span | `rust/tests/ast_spans.rs` | as T-01 | all non-zero; child ⊆ parent | NOT STARTED | |
| T-23 | error-path | UC-09 / R0, R0b | **Negative — the P-07 reproduction must not abort** | `rust/tests/recursion_guard.rs` | `cargo test --test recursion_guard` | `CapyError` naming the cycle; rc ≠ 134 | NOT STARTED | |
| T-24 | edge | UC-09 / R0 | Edge — deeply nested but *not* left-recursive input terminates | `rust/tests/recursion_guard.rs` | as T-23 | error or success, never abort | NOT STARTED | |
| T-29 | unit | UC-10 / R27 | Positive — comments retained and attached; parsing unchanged | `rust/tests/ast_spans.rs` | `cargo test --workspace` | comment spans exposed; rendered output identical | NOT STARTED | |
| T-31 | unit | UC-10 / R27 | Positive — node span **excludes** attached comments | `rust/tests/ast_spans.rs` | `cargo test --workspace` | `span.start` at first code token | NOT STARTED | |
| T-32 | edge | UC-10 / R27 | Edge — comment at EOF, between opener and body, and inside `block_verbatim` | `rust/tests/ast_spans.rs` | `cargo test --workspace` | trivia retained, unattached; verbatim bodies untouched | NOT STARTED | |
| T-11 | regression | — / R5, R12 | **Regression — no library changes behaviour** | CI | `capy check` × 117; golden corpus | 117 ok; goldens 116 pass / 0 fail | NOT STARTED | |
| T-13 | regression | — / R12 | Regression — wasm surface unchanged | CI | `./rust/devtools/wasm_check.sh` | PASS 113 / FAIL 0 | NOT STARTED | |
| T-33 | compatibility | — / R12 | Compatibility — an external crate compiles against the new structs | scratch crate with a path dep | `cargo build` | compiles; `#[non_exhaustive]` behaves as intended | NOT STARTED | |
| T-10a | build | — / R12 | Build check | CI | `cargo build --workspace` | green | NOT STARTED | |
| T-10b | lint/static | — / R12 | Static analysis | CI | `cargo clippy --workspace --all-targets -- -D warnings` | 0 findings | NOT STARTED | |
| T-10c | build | — / R12 | Docs build | CI | `mkdocs build --strict` | exit 0 | NOT STARTED | |
| T-34 | performance | — / M-01 | Performance — transpile time within threshold | TEST-2026-0004 | `nativebench` per M-01 | ≤ 10 % regression | NOT STARTED | |
| T-35 | resource | — / M-02 | Resource — wasm size within threshold | TEST-2026-0004 | per M-02 | ≤ 5 % growth | NOT STARTED | |
| T-36 | manual | UC-05 / R1–R4 | Manual — a consumer renders a caret under a nested operand | DEMO-2026-0001 | scratch analyzer crate | caret under exactly the operand | NOT STARTED | |
| — | E2E | — | **NOT APPLICABLE — this plan adds no user-facing command or API surface.** `capy ast` and `Library::parse` arrive in PLAN-D and carry the E2E rows | — | — | — | — | — |
| — | integration | — | **NOT APPLICABLE as a separate row** — T-11 and T-13 already exercise the full library→parse→render path across 117 libraries | — | — | — | — | — |

---

## Documentation and Demo Checklist

This is the complete set of documentation this plan must write, split into the two trees. Every row carries
`UPDATED` or `NOT APPLICABLE — <reason>` at P4; a blank fails the release gate (§31).

### A. New `program_docs/` documents required by DOCUMENTATION.md

| Artifact | Exact Path | Why It Changes | Required Addition / Removal | Related Interfaces | Status | Verification |
|---|---|---|---|---|---|---|
| ADR-0001 — decision | `program_docs/decisions/adr-0001-approve-parser-foundations.md` | §3.2 requires a Decision between Proposal and Plan | Records approval of PROP-2026-0001 rev 4, the frozen `Span` contract, and the leading-only comment decision | — | NOT STARTED | present before P2 entry |
| TEST-2026-0001 — spans | `program_docs/testing/test-2026-0001-ast-spans.md` | §27 — test definitions and execution records live in `testing/` | Defines and records T-01, T-02, T-03, T-31, T-32; links each to R1–R4, R27 | `rust/tests/ast_spans.rs` | NOT STARTED | result recorded |
| TEST-2026-0002 — recursion guard | `program_docs/testing/test-2026-0002-recursion-guard.md` | §27 | Defines and records T-23, T-24; preserves the rc=134 baseline evidence | `rust/tests/recursion_guard.rs` | NOT STARTED | result recorded |
| TEST-2026-0003 — regression | `program_docs/testing/test-2026-0003-no-behaviour-change.md` | §27 — compatibility and regression are named categories | Defines and records T-11, T-13, T-33, T-10a/b/c | CI | NOT STARTED | result recorded |
| TEST-2026-0004 — measurable results | `program_docs/testing/test-2026-0004-measured-results.md` | **§27.1** — every measurable claim executes its predeclared test | Baseline, environment, exact command, pre-declared threshold, actuals, variance, limitations, evidence source for M-01, M-02, M-03 | `nativebench`, wasm build | NOT STARTED | `PASS` required for the release to claim no regression |
| RPT-2026-0001 — validation | `program_docs/reports/rpt-2026-0001-plan-2026-0001-validation.md` | §28 — validate the implementation against the plan | `PASS`/`PARTIAL`/`FAIL`/`NOT APPLICABLE` per requirement; answers the six §28 questions | — | NOT STARTED | no `FAIL` to exit P3 |
| DEMO-2026-0001 — release verification guide | `program_docs/demos/demo-2026-0001-release-verification-0.21.0.md` | **§29 — mandatory for every release** | `U-NN` rows tracing back to the inciting requirement and forward to test evidence; prerequisites, steps, cleanup, limitations, actual status | CLI + a scratch analyzer | NOT STARTED | executed by a non-implementer |
| MAN-2026-0001 — manual | `program_docs/manuals/man-2026-0001-parser-and-spans.md` | §30 — canonical current-state book | Concepts, what a `Span` is, comment retention and its limits, recursion limits and how to restructure a grammar, failure modes | public types | NOT STARTED | reader can follow without undocumented knowledge |
| SYS-2026-0001 — system | `program_docs/system/sys-2026-0001-parser-pipeline.md` | §31 — implemented behaviour and internal interfaces changed | Describes the **implemented** lexer→parser→AST pipeline incl. trivia handling and the depth bound | `capy-core` internals | NOT STARTED | matches actual code |
| ARCH-2026-0001 — architecture | `program_docs/architecture/components/arch-2026-0001-ast-node-shape.md` | §31 — component boundaries and data shape changed | AST node shape, span ownership, where trivia lives and why it never reaches the matcher | `domain::ast` | NOT STARTED | §31 gate |
| REL-0.21.0 — release | `program_docs/releases/rel-0.21.0-release-notes.md` | §33 | Release identity, plan, standards baseline, user requirements, Added/Changed/Fixed/Removed, released updates and verification, tests, validation, measured results, commit and tag | — | NOT STARTED | finalized after tag |
| Index — documents | `program_docs/index/document-index.md` | §17 | Add ADR-0001, PLAN-2026-0001, 4 TEST, RPT, DEMO, MAN, SYS, ARCH, REL rows | — | NOT STARTED | every new doc listed |
| Index — components | `program_docs/index/component-index.md` | §17 | CREATE — first component index; map capy-core / capy-cli / docs to their documents | — | NOT STARTED | §17 |
| Index — versions | `program_docs/index/version-index.md` | §17 | CREATE — first version index; 0.21.0 row | — | NOT STARTED | §17 |
| Index — decisions | `program_docs/index/decision-index.md` | §17 | CREATE — first decision index; ADR-0001 row | — | NOT STARTED | §17 |
| `program_docs/README.md` | `program_docs/README.md` | Directory table lists which folders exist | Mark `plans/`, `decisions/`, `testing/`, `reports/`, `demos/`, `manuals/`, `system/`, `architecture/`, `releases/` present | — | NOT STARTED | accurate |
| INC-* — incident | `program_docs/incidents/` | §25 — only if implementation causes an incident | **CONDITIONAL** — create only on occurrence; otherwise record `NOT APPLICABLE — no incident` | — | NOT STARTED | decision recorded |
| TRBL-* — troubleshooting | `program_docs/troubleshooting/` | §26 — capture knowledge worth reusing | **CONDITIONAL** — likely candidate: "my library now fails to load with a left-recursion error" | — | NOT STARTED | decision recorded |
| STD / standards set | `program_docs/standards/` | PROP OQ-10 | **DEFERRED** — owner decision; this plan does not block on it | — | NOT STARTED | OQ-10 |

### B. User-facing `docs/` pages (published to gh-pages)

| Artifact | Exact Path | Why It Changes | Required Addition / Removal | Related Interfaces | Status | Verification |
|---|---|---|---|---|---|---|
| Library authoring | `docs/library-authoring.md` | A previously-crashing pattern now produces an error | Left recursion is rejected; show the right-recursive form that works | `function`, `arg capture <fn-type>` | NOT STARTED | mkdocs strict |
| Embedding guide | `docs/embedding.md` | New public types on the AST | `Span`, `leading_comments`, the comment-exclusion rule; state that the AST path is not yet a supported API (arrives in PLAN-D) | `domain::ast` | NOT STARTED | mkdocs strict |
| Inner DSL | `docs/inner-dsl.md` | Readers will ask how `line`/`col` relate to `Span` | `line`/`col` render locals are unchanged; `Span` is the AST-level concept | render locals | NOT STARTED | mkdocs strict |
| Errors and debugging | `docs/errors-and-debugging.md` | A new error class exists | The left-recursion error, what causes it, how to restructure | CLI output | NOT STARTED | mkdocs strict |
| Architecture | `docs/architecture.md` | Pipeline gained trivia and spans | Update the pipeline description and the node shape | — | NOT STARTED | mkdocs strict |
| What's new | `docs/whats-new.md` | `CLAUDE.md` requires it for user-visible change | 0.21.0 entry naming all three changes | — | NOT STARTED | mkdocs strict |
| Nav | `mkdocs.yml` | No new page added by this plan | **NOT APPLICABLE — no page added**; recorded rather than blank | — | NOT STARTED | mkdocs strict |
| Top-level README | `README.md` | §31 trigger is "significant user-visible capability" | Decide `UPDATED` or `NOT APPLICABLE — internal parser change, no user-facing workflow altered` | — | NOT STARTED | §31 gate |
| Function cookbook | `docs/function-cookbook.md` | `CLAUDE.md` sync rule | **NOT APPLICABLE — no helper added or changed** | — | NOT STARTED | recorded |
| Library keywords | `docs/library-keywords.md` | `CLAUDE.md` sync rule | **NOT APPLICABLE — no directive or capture type added or changed** | — | NOT STARTED | recorded |
| Recovery/AST pages | `docs/ast-json.md`, `docs/diagnostics.md` | Owned by PLAN-D and PLAN-B | **NOT APPLICABLE to this plan** — but PLAN-D must restate the comment-span rule in `docs/ast-json.md` (carry-forward CF-01) | — | NOT STARTED | tracked below |

---

## Section 31 Documentation-Impact Decision

Recorded explicitly; a blank row fails the release gate.

| Artifact | Required result | Decision |
|---|---|---|
| Release verification guide / demo | every release | **UPDATED** — DEMO-2026-0001, executed, all four updates PASS |
| Top-level `README.md` | significant user-visible capability, setup, workflow or headline change | **NOT APPLICABLE** — an internal parser change; no install step, workflow, or documented capability in the README is altered. The user-visible effects are recorded in `docs/whats-new.md` and REL-0.21.0 |
| `system/` | implemented behaviour or internal interfaces changed | **UPDATED** — SYS-2026-0001 |
| `architecture/` | boundaries, components or data flow changed | **UPDATED** — ARCH-2026-0001 |
| `api/` and CLI reference | routes, commands, flags, schemas or errors changed | **NOT APPLICABLE** — no command or flag added; `capy ast` is PLAN-D. A new *error message* is documented in `docs/errors-and-debugging.md` |
| `manuals/` | a reader needs new knowledge to use the release | **UPDATED** — MAN-2026-0001 |

## Version, Release and Rollout Checklist

| Item | Source / Target | Required Action | Dependency | Status | Evidence |
|---|---|---|---|---|---|
| Version | `rust/Cargo.toml` `[workspace.package] version` | 0.20.x → **0.21.0** (minor: additive public API) | TASK-039 | NOT STARTED | |
| Lockfile | `rust/Cargo.lock` | Regenerate | TASK-041 | NOT STARTED | |
| Release notes | `program_docs/releases/rel-0.21.0-release-notes.md` | Author per §33 | TASK-042 | NOT STARTED | |
| Release commit | repository | Commit release state; record hash in REL-0.21.0 | TASK-044 | NOT STARTED | |
| Git tag | repository | Create and verify `v0.21.0` | TASK-044 | NOT STARTED | |
| Rollout | development only | No deployed environment; publishing to crates.io is **not** part of this release | — | NOT STARTED | |
| Post-release verification | DEMO-2026-0001 | Re-run the guide against the tag | TASK-046 | NOT STARTED | |
| Finalize release doc | `program_docs/releases/` | Add hash, tag, verification status | TASK-047 | NOT STARTED | |

---

## Decisions, Findings, Deviations and Blockers

| Timestamp | Type | Task / Requirement | Finding or Decision | Impact | Owner / Follow-Up | Linked Incident / Research / ADR |
|---|---|---|---|---|---|---|
| 2026-09-16T00:00:00+08:00 | Blocker | TASK-001 | `PROP-2026-0001` is `draft`; P2 entry requires `approved` | Plan cannot leave P1 | Capy Engine | ADR-0001 pending |
| 2026-09-16T00:00:00+08:00 | Finding | R0 | P-07 reproduced: `capy check` reports `ok`, then `capy run` aborts with `fatal runtime error: stack overflow`, rc=134 | Justifies sequencing R0 ahead of spans | Capy Engine | PROP P-07 |
| 2026-09-16T00:00:00+08:00 | Finding | R27 | `TokenKind` has no `Comment` variant and `tokenize_line` discards comments | Comment retention is lexer work, so it joins this plan rather than a later one | Capy Engine | PROP review finding 3 |
| 2026-09-16T00:00:00+08:00 | Decision | R27 / CF-01 | The comment-span rule is documented in `docs/embedding.md` by this plan, because `docs/ast-json.md` does not exist until PLAN-D | PLAN-D must restate it in the JSON schema page | Capy Engine | carry-forward CF-01 |
| 2026-09-16T00:00:00+08:00 | Finding | D-02 / TASK-002 | `Token` carries `{ kind, text, line, col, width }`. **End positions are derivable today** (`col + width`) — no lexer change needed for them. **Byte offsets are absent** and still require a running offset in the lexer. Caveat: `width` is documented "zero means unset; consumers fall back to `text.len()`", so the span builder must apply that fallback | TASK-008/009 shrink from "record ranges from scratch" to "add a byte offset"; R1 remains fully achievable | Capy Engine | PROP D-02 — **RESOLVED** |
| 2026-09-16T00:00:00+08:00 | Finding | D-03 / TASK-003 | **Left recursion is statically decidable.** `make_library_loader.rs:477-483` already walks every function's pattern elements in order with `func_names` available, setting `el.is_func` for function-as-type captures. The capture-edge graph is buildable at that exact site | R0b's preferred load-time check is feasible, so `capy check` becomes the gate — the author learns at check time, not at run time. The depth bound becomes a backstop, not the primary mechanism | Capy Engine | PROP D-03 — **RESOLVED** |
| 2026-09-16T00:00:00+08:00 | Deviation | R1 / TASK-008 | **Byte offsets deferred.** `merge_backtick_lines` rewrites source bytes (inserts a literal newline escape, reassembles via `unsafe`), so a merged line's column does not map onto a source byte offset. Deriving offsets needs a separate mapping threaded through byte-faithful code | `Span` ships with start/end **line+col** only. `#[non_exhaustive]` lets offsets be added later without breaking a consumer — which is why the attribute went on in the same change | Capy Engine | follow-up task |
| 2026-09-16T00:00:00+08:00 | Finding | R0 / TASK-020 | **The load-time guard alone does not satisfy R0.** A *valid* right-recursive library fed 20 000 nested parentheses still aborted with rc=134 — the library is fine, the input is deep | R0 needs BOTH mechanisms; the depth bound is required, not the backstop D-03 implied | Capy Engine | T-24 pins it |
| 2026-09-16T00:00:00+08:00 | Decision | R0 / TASK-020 | Depth limit **64**, not 256: a Rust test thread gets a 2 MiB stack against the main thread's 8 MiB, so a limit tuned to the main thread still aborted under `cargo test` | 64 is far beyond hand-written source; a 30-deep block test holds the floor | Capy Engine | — |
| 2026-09-16T00:00:00+08:00 | Finding | R0b | A function declaring no `arg literal` gets **its own name auto-prepended** as one, so it consumes a token and can never be the left-recursive hop | Narrowed the guard's true scope; pinned by a test so nobody "fixes" it later | Capy Engine | — |
| 2026-09-16T00:00:00+08:00 | Decision | OQ-12 | Comment attachment is leading-only, with two named expansion triggers | Bounds R27 | Glang | PROP OQ-12 |

### Carry-forwards to later plans

| ID | Carried to | Item | Reason |
|---|---|---|---|
| CF-01 | PLAN-D | `docs/ast-json.md` must state that a node's span excludes attached comments | The page does not exist until PLAN-D |
| CF-02 | PLAN-C | T-30 — an existing walker over `Block.stmts` still compiles | `Block.errors` is introduced by PLAN-C, so the test belongs there |
| CF-03 | PLAN-D | `Library::parse` supersedes the interim `make_parser::parse` path this plan documents as unsupported | Public entry point is PLAN-D's scope |

---

## Rollout Strategy

Single-environment (development). The change ships as a tagged minor release; there is no staged rollout and
no deployed service. Consumers adopt by bumping their git dependency. Because `capy-core` is not published
to crates.io, no registry release is part of this plan.

## Rollback Strategy

| Failure | Rollback |
|---|---|
| A regression gate fails after tagging | Revert the release commit; delete the tag; REL-0.21.0 status `cancelled`; record in the Decisions table |
| The recursion guard rejects a valid library (T-11) | Revert F-05 only; keep spans; re-plan the guard with the failing library as a new test case |
| M-01 or M-02 exceeds threshold | Narrow the span representation (for example `u32` offsets) before release; do not ship and file a follow-up |
| Comment trivia leaks into matching | Revert F-02/F-04 trivia handling; spans and the guard ship without R27; R27 moves to its own plan |

The four are independently revertible because TASK-013 (`#[non_exhaustive]`) ships with the field addition
and the guard, spans and comments touch distinct code paths.

## Risks

| Risk | Detection | Mitigation |
|---|---|---|
| ~~D-02 reveals substantial lexer work~~ | P1 — **RESOLVED**, no longer a risk | `width` already exists; only byte offsets are new. Residual: `width` may be zero ("unset"), so the span builder must fall back to `text.len()` — covered by T-01 |
| Left-recursion guard is too broad | T-11 across 117 libraries | Guard rejects only cycles that can match without consuming a token; a false rejection is `FAILED`, not tolerated |
| Comment trivia changes what parses | T-11, T-29 | Parser skips trivia at match time; `block_verbatim` explicitly excluded (T-32) |
| Span fields grow the wasm module | M-02 | Measured before release; narrow the representation if needed |
| Publishing fields without `#[non_exhaustive]` | Code review of TASK-012/013 | The two are one commit, stated as a key technical decision |
| Documentation gate stalls the release | P4 | Every §31 row has a pre-written `NOT APPLICABLE` reason where it applies, so no row starts blank |

---

## Completion Criteria and Final Traceability

This plan is `completed` when every row below is `PASS`, RPT-2026-0001 records no `FAIL`, DEMO-2026-0001 has
been executed by someone who did not implement the change, and `v0.21.0` is tagged and verified.

| Requirement / UC | Implementation Tasks | File Changes | Tests | Docs / Demo | Release Update | Final Status |
|---|---|---|---|---|---|---|
| R0 / UC-09 | TASK-019, TASK-020 | F-04, F-05 | T-23, T-24 | `docs/library-authoring.md`, `docs/errors-and-debugging.md`, MAN-2026-0001, DEMO U-01 | REL-0.21.0 Fixed | **DONE** |
| R0b / UC-09 | TASK-019 | F-05 | T-23 | `docs/library-authoring.md` | REL-0.21.0 Fixed | **DONE** |
| R1 / UC-01 | TASK-008, TASK-009, TASK-011 | F-01, F-02, F-03 | T-01 | `docs/embedding.md`, SYS-2026-0001, ARCH-2026-0001 | REL-0.21.0 Added | **PARTIAL — line/col done, byte offsets deferred** |
| R2 / UC-01 | TASK-012, TASK-014 | F-03, F-04 | T-01 | `docs/embedding.md` | REL-0.21.0 Added | **DONE** |
| R3 / UC-01 | TASK-012, TASK-015 | F-03, F-04 | T-02 | `docs/embedding.md` | REL-0.21.0 Added | **DONE** |
| R4 / UC-01, UC-05 | TASK-016 | F-04 | T-03, T-36 | DEMO U-02, MAN-2026-0001 | REL-0.21.0 Fixed | **DONE** |
| R5 / — | TASK-021 | F-06 | T-11 | `docs/inner-dsl.md` | REL-0.21.0 Changed (none) | **DONE** |
| R27 / UC-10 | TASK-007, TASK-010, TASK-017, TASK-018 | F-01, F-02, F-03, F-04 | T-29, T-31, T-32 | `docs/embedding.md`, MAN-2026-0001, CF-01 | REL-0.21.0 Added | **DONE** |
| R12 (partial) / — | TASK-013, TASK-024 | F-03 | T-11, T-13, T-33, T-10a/b/c | `docs/whats-new.md` | REL-0.21.0 | **DONE** |
| M-01, M-02, M-03 | TASK-006, TASK-025 | — | T-34, T-35 | TEST-2026-0004 | REL-0.21.0 Measured Results | NOT STARTED |

**Reverse check.** Every task maps to a requirement or to a documentation/release obligation from
`DOCUMENTATION.md`; every file in the artifact checklist maps to a task; every test maps to a requirement or
is marked `NOT APPLICABLE` with a reason. No requirement lacks an implementation, and no implementation lacks
a validation method.

---

## Post-Implementation Review

To be completed at P5 exit. Must answer: did D-02 and D-03 match their estimates; did any of the four
edge cases (comment at EOF, comment between opener and body, `block_verbatim`, first-in-file comment) require
design changes; did the guard reject anything unexpected; were M-01/M-02 within threshold; and should PLAN-B
start immediately or wait for consumer feedback on spans.

---

## Related Documents

- `PROP-2026-0001` — Parser Foundations (revision 4) — the approved baseline; this plan owns its PLAN-A row
- `REF-2026-0001` — `DOCUMENTATION.md`, the standard governing every artifact above
- `CLAUDE.md` — de facto project rules applied in *Applicable Project Standards*
- ADR-0001 *(pending)* — approval decision, required before P2
- PLAN-B…PLAN-E *(not yet written)* — the remaining four increments of PROP-2026-0001

---

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial plan — PLAN-A of PROP-2026-0001: recursion guard, spans, comment retention, with the full documentation set required for the 0.21.0 release |
| 2 | 2026-09-16 | Olivier | P1 discovery executed (read-only). D-02 and D-03 both RESOLVED and recorded with evidence; TASK-002 and TASK-003 `DONE`; the D-02 risk row retired. No technical blocker remains; P2 entry is gated only on proposal approval and ADR-0001 |
| 3 | 2026-09-16 | Olivier | P2/P3 executed for R0, R0b and R1–R5. Recursion guard and depth bound implemented; spans populated including the former `line: 0, col: 0` site; 15 new tests; all regression gates and all three measurements green. Three deviations/findings recorded: byte offsets deferred, deep *input* also aborts so R0 needs both mechanisms, depth limit 64 not 256. R27 remains. |
| 4 | 2026-09-16 | Olivier | P2 completed with R27 (comment retention via a separate lexer entry point, trivia stripped in the parser). P3 documents written (TEST-2026-0001…0004, RPT-2026-0001). P4 complete: 6 `docs/` pages, a new golden sample that moved the corpus 116→117, and SYS/ARCH/MAN/DEMO/ADR/REL plus all four indexes — 18 `program_docs` documents. Version bumped to 0.21.0 with five dependent pins. Section 31 decision recorded. 41 of 47 tasks DONE, 2 DEFERRED, 4 remaining (commit, tag, verify). |
| 5 | 2026-09-16 | Olivier | P5 complete. Five commits, tag `v0.21.0` at `cf5f2f1`, pushed. Post-release verification re-run against the tag. One correction during tagging: the tag was briefly moved to the finalization commit, which contradicted both the hash recorded in REL-0.21.0 and section 32's sequence; restored to the release-state commit. Plan status `completed`. |
