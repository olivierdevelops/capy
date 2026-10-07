---
document_id: PROP-2026-0003
title: Expression Trees Over JSON, and the Parenthesis Round-Trip Fix
document_type: proposal
status: draft

created_date: 2026-09-16
last_updated: 2026-09-17
document_revision: 3

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine
  - Release Management

systems:
  - Capy

components:
  - capy-core
  - docs

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Two engine changes that make Capy's language-front-end story true without an asterisk — serializing the parsed expression tree in `capy ast --json`, and restoring the parenthesis round-trip for `not` and nested comparison in `expr_to_text`.

reason: PROP-2026-0002's implementation established that Capy is a usable language front end, and the documentation now says so. Two things qualify that claim, and they are different in kind. C-02 is a DEFECT: PLAN-2026-0002 R12 required `expr_to_text` to round-trip the new operator nodes and named T-27 as the gate, but the precedence guard was applied to `Expr::Binary` only, and T-27's corpus covered neither `Not` nor nested `Compare` — so captured source text for those forms is silently wrong. C-01 is a GAP, not a defect: no requirement ever specified serializing the expression tree, UC-01 scoped analyzer access to the Rust API, and `Expr` is the engine's own value grammar rather than the library's — so exposing it in a source-language AST is a design decision to take, not a regression to repair.

related_documents:
  - PROP-2026-0002
  - PROP-2026-0001
  - REL-0.22.0
  - ADR-0001
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - parser
  - ast
  - json
  - expressions
  - correctness

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Expression Trees Over JSON, and the Parenthesis Round-Trip Fix

> **Status:** Draft
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.23.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs

## Summary

Two changes, one theme — **the expression tree should be reachable, and its text
rendering should be trustworthy** — but they are **not the same kind of change**,
and a reviewer should treat them differently:

| | C-02 — round-trip | C-01 — serialize `expr` |
|---|---|---|
| Kind | **Defect** | **Gap / new capability** |
| Against what | `PLAN-2026-0002` R12 + TASK-018 required the round-trip; T-27 was the named gate and its corpus had the hole | Nothing. No requirement in `PROP-2026-0001` R6–R9 mentions expression trees |
| Consequence | Silently wrong target code today | A non-Rust consumer re-implements a parser |
| Decision | **Has to be fixed** | **A judgement call** — see the layering argument in P-01 |

INC-1 (C-02) therefore stands alone and should ship regardless of what is decided
about INC-2.

```text
                         ┌──────────────────────────────────────────┐
   in-memory AST         │ CaptureValue { is_expr, expr, text, … }  │
                         └──────────┬─────────────────┬─────────────┘
                                    │                 │
              Library::parse ───────┘                 └─────── capy ast --json
              (Rust embedding)                                 (any language)
                                    │                                │
                         expr: Some(Expr)  ✅              "is_expr": true   ❌
                         the real tree                     "text": "a * b"
                                                           tree DROPPED
                                                                 │
                                                                 ▼
                                            re-parse the text?  ❌ C-02: `not (a==b)`
                                                                   renders `not a==b`
```

A consumer on the JSON boundary has no route to the expression tree: it is not
serialized, and reconstructing it from `text` is unsafe because of the second
defect. Fixing both makes one sentence true — *"declare a grammar, get a parse
tree"* — regardless of which language the consumer is written in.

## Decision Requested

Approve two changes to `capy-core`:

1. **C-01** — serialize `CaptureValue.expr` in `domain/ast_json.rs` as a new
   `expr` field on a capture. Additive; the schema already permits added fields
   without a version bump.
2. **C-02** — apply the existing `wrap_if_looser` precedence guard to
   `Expr::Not` and `Expr::Compare` in `orchestrator/features/expr_to_text.rs`,
   and extend the T-27 round-trip corpus to cover both.

Approval does **not** authorize: exposing the AST over the wasm ABI; adding byte
offsets to `Span`; tuning the recovery cascade; or any change to `Library::run`.

## Priority — revised 2026-09-17

Evidence arrived after revision 2 that **separates the two changes further**.

The one known consumer building a language on Capy (Ambit) embeds in **Rust** —
their crate takes a path dependency on `capy-core` and walks the tree in their
own `translate.rs`. Their rev-5 probe records `add(a + 1, b * 2)` returning
"full expr trees", which is true on the route they are using: `CaptureValue.expr`.

```text
  C-02  round-trip defect   ── affects anyone emitting captured source text,
                               INCLUDING a Rust embedder using `${capture}`
                               in a write template.   urgency: SHIP IT
  C-01  serialize `expr`    ── affects consumers on the JSON boundary.
                               No such consumer exists today.
                               urgency: schedule, do not rush
```

This does not change the recommendation — C-01 is still the right end state, and
`docs/language-frontend.md` still carries the caveat — but it does mean C-01 is
for the *next* consumer rather than a waiting one. INC-1 remains the increment
that should ship regardless.

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "given that now it can be used as a grammar frontend for a real language shouldn't we edit the docs/ even more to highlight this features as many people could be interested in" | User, conversation, 2026-09-16 | Fact: the front-end positioning should be documented. Fact: the documentation half was done directly. The two defects below were surfaced while verifying the claim. |
| UQ-02 | "documentation first then write proposal if needed" / "if code change write proposal else fix doc" | User, conversation, 2026-09-16 | A standing rule: documentation edits proceed directly; anything requiring a code change becomes a proposal. This document exists because of that rule. |
| UQ-03 | "NO OPERATOR PRECEDENCE … every Glang analysis operates on expression trees" | `PROP-2026-0001` UQ-02, from `.ignore/needs2.md` §1, 2026-09-16 | The original consumer requirement. 0.22.0 satisfied it for Rust embedders; C-01 satisfies it for everyone else. |

## Problem and Evidence

### Current User Journey

```text
[language author] -> declares a grammar -> capy ast --json -> walks the tree
                  -> reaches an expression capture
                  -> gets {"is_expr": true, "text": "a * b + c[i]"}
                  -> writes their own expression parser  <- the thing Capy just did
                        |
                        +-> or re-parses `text` -> silently wrong tree for `not (…)`
```

| Problem | Affected Users | Evidence and Inline Source | Consequence | UQ IDs |
|---|---|---|---|---|
| P-01 | every non-Rust consumer | `rust/src/domain/ast_json.rs:48` serializes `is_expr` but never `expr`, though `CaptureValue.expr: Option<Expr>` exists at `rust/src/domain/ast.rs:156`. Verified: `let x = a * b + c[i]` → `{"is_expr": true, "text": "a * b + c[i]", "sub": []}`. **This is a gap, not a defect** — see the counter-argument below | A consumer must re-implement the expression parser Capy already contains, or embed in Rust | UQ-01, UQ-03 |
| P-02 | every consumer that emits captured source text | **Defect against `PLAN-2026-0002` R12 / TASK-018.** `rust/src/orchestrator/features/expr_to_text.rs:45` renders `Expr::Not` as `format!("not {}", …)` with no precedence guard; `:25-27` renders `Expr::Compare` children bare. `Expr::Binary` at `:34-44` **does** apply `wrap_if_looser` / `wrap_if_looser_or_equal` | `not (1 == 2)` renders as `not 1 == 2`, which re-parses as `(not 1) == 2` — a different tree and, here, the opposite value. A transpiler emitting `${cond}` emits wrong target code, silently | UQ-01 |
| P-03 | maintainers | `rust/tests/precedence.rs` `round_trip_preserves_structure` (T-27) exists precisely to catch P-02's class, but its corpus contains no `not` and no nested comparison | The guard that was supposed to prevent this had a hole in its test data, not in its intent | UQ-01 |

### The counter-argument to P-01 — why this may be correct as it stands

A reviewer should weigh this before approving C-01. Three points say the current
behaviour was a decision rather than an oversight:

1. **Nothing required it.** `PROP-2026-0001` R6–R9 cover `ParseResult`, the
   `capy ast` command, the no-serde constraint and the documented schema. None
   mentions expressions. R9's acceptance criterion — "documents every field" —
   is met by `docs/ast-json.md` as shipped.
2. **The analyzer journey was scoped to Rust on purpose.** `PROP-2026-0001`
   `UC-01` ("locate a nested operand precisely", actor: analyzer author) names
   its surface as **Rust API; `Library::parse`**. `UC-03` is the separate
   "feed the AST to an external tool" case.
3. **Layering.** `Expr` is the **engine's own** value-expression grammar.
   `GOAL-001` is zero source-language grammar, and a built-in source expression
   grammar is Non-Goal 5 of `PROP-2026-0001`. Putting `Expr` inside a
   source-language AST publishes the engine's grammar as part of the library
   author's tree.

And the structure a library author *declares* already serializes. Verified on
`samples/language-frontend/`:

```text
$ capy ast … --json | jq '.tree.stmts[0].captures.params'
  is_expr : false
  sub     : 2 nodes -> ["param", "param"]
  sub[0].captures.pname : text "a", span 1:8-1:9
```

A language whose expression syntax is declared with function-as-type captures
gets full trees over JSON **today**. Only the engine's built-in value grammar
flattens to text.

**The question C-01 actually asks** is therefore not "should we fix this" but
"should the engine's value-expression grammar be part of the published
source-language AST?" A reasonable *no* exists: it commits the engine's grammar
to the schema. The recommendation here is **yes** — `is_expr: true` already
announces the expression's existence, `text` already leaks its content
unstructured, and `PROP-2026-0001` UQ-03 recorded a consumer requirement for
expression trees that today only Rust embedders can satisfy.

### Reproduction — P-02

Verified against `rust/target/release/capy` at `v0.22.0`, using
`samples/operator-precedence/lib.capy`, whose `${value}` prints the source text
and `${context.vars[name]}` the evaluated result:

```text
source written          text rendered back     re-parses as      true value
──────────────────      ──────────────────     ─────────────     ──────────
not (1 == 2)            not 1 == 2             (not 1) == 2      true   ✗ text says false
1 == (2 == 2)           1 == 2 == 2            (1 == 2) == 2     false  ✗ different tree
(1 + 2) * 3             (1 + 2) * 3            (1 + 2) * 3       9      ✓ Binary is guarded
1 == 1 and (2 == 3 or 4 == 4)   …preserved…                      true   ✓ Binary is guarded
```

The two correct rows are what the `Expr::Binary` arm already does. C-02 extends
the same treatment to the two arms that lack it.

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-01 | An expression capture in `capy ast --json` carries its tree | P-01 | New `expr` field on a capture | `jq '.tree.stmts[0].captures.value.expr'` returns a node, not null |
| G-02 | `parse(expr_to_text(e))` is structurally equal to `e`, for every expression form | P-02, P-03 | Precedence guard on `Not` and `Compare`; corpus extended | T-27 passes over a corpus including both |
| G-03 | The front-end documentation loses its asterisk | P-01 | `docs/language-frontend.md`, `ast-json.md`, `roadmap.md` updated once C-01 lands | No "embed in Rust for expression trees" caveat remains |

**Non-goals**

- **No `Expr` evaluation semantics change.** C-02 changes only rendering; every
  value the engine computes today is already correct.
- **No schema version bump.** `docs/ast-json.md` already states fields may be
  added without one. Consumers ignoring unknown keys are unaffected.
- **No wasm ABI change** (`REL-0.22.0` Known Limitation 5 stays open).
- **No byte offsets, no cascade tuning.**
- **FU-02 (bare-hyphen lexing) is not fixed here.** See *Open Questions* OQ-03.

## Proposed User Journey

```text
[language author] -> capy ast --json -> walks the tree
                  -> reaches an expression capture
                  -> reads capture.expr  -> a real node: {"kind":"binary","op":"+", …}
                  -> walks it with the same visitor as every other node
                        |
                        +-> or reads capture.text, and the text now round-trips
```

## Requirements

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal IDs |
|---|---|---|---|---|---|
| R1 | A capture with `is_expr: true` serializes an `expr` object | API | P-01 | `expr` non-null for every expression capture | G-01 |
| R2 | A capture with `is_expr: false` serializes `expr: null` | API | consistency with `span` | null, not an empty object | G-01 |
| R3 | The serialized shape covers every `Expr` variant | API | P-01 | Each variant has a documented `kind`; a round-trip test walks all of them | G-01 |
| R4 | `schema_version` stays **1** | compatibility | additive change | unchanged; `docs/ast-json.md` says why | G-01 |
| R5 | `expr_to_text(Expr::Not(x))` parenthesizes `x` when `x` binds looser than prefix `not` | functional | P-02 | `not (1 == 2)` renders `not (1 == 2)` | G-02 |
| R6 | `expr_to_text(Expr::Compare(c))` parenthesizes children that bind looser, and an equal-precedence right child | functional | P-02 | `1 == (2 == 2)` renders `1 == (2 == 2)` | G-02 |
| R7 | The T-27 corpus includes `not` and nested-comparison forms | test | P-03 | Corpus extended; test fails without R5/R6 | G-02 |
| R8 | No existing golden changes | compatibility | `GATE-002` | 125/125 byte-identical | G-02 |
| R9 | Docs updated once C-01 lands | docs | G-03 | `ast-json.md` documents `expr`; `language-frontend.md` and `roadmap.md` drop the caveat; `samples/language-frontend/README.md` route table updated | G-03 |
| R10 | `samples/operator-precedence/` regains the `not (…)` case | samples | D-05 of PROP-2026-0002 | Case restored with a correct golden | G-02 |

## Use Cases

| ID | User Outcome | Actor | Surface and Trigger | Inputs | Outputs / Visible Result | Negative Paths | Requirement IDs |
|---|---|---|---|---|---|---|---|
| UC-01 | Walks an expression tree from Python/TS/Go | language author | `capy ast --json` | source with `let x = a * b + c` | `captures.value.expr` is a `binary` node | non-expression capture → `expr: null` | R1, R2, R3 |
| UC-02 | Emits captured source into a target language safely | library author | `${cond}` in a `write` template | `if not (a == b) {` | target receives `not (a == b)` | — | R5, R6 |
| UC-03 | Upgrades without touching anything | existing consumer | any | any | unchanged behaviour; one new key they may ignore | — | R4, R8 |

### UC-01 — CLI Contract (illustrative shape, settled in the plan)

```text
$ capy ast lib.capy main.capy --json | jq '.tree.stmts[0].captures.value'
{
  "is_expr": true,
  "text": "a * b + c",
  "span": { "start_line": 1, "start_col": 9, "end_line": 1, "end_col": 18 },
  "sub": [],
  "expr": {
    "kind": "binary",
    "op": "+",
    "left":  { "kind": "binary", "op": "*",
               "left":  { "kind": "var", "path": ["a"] },
               "right": { "kind": "var", "path": ["b"] } },
    "right": { "kind": "var", "path": ["c"] }
  }
}
```

`text` is retained — a transpiler that emits source still wants it, and after
C-02 it is trustworthy.

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 | 2026-09-16 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 (zero default grammar) | Applies | Neither change adds source-language grammar; C-01 serializes what is already parsed | PASS | — |
| GOAL-002 (engine changes additive) | Applies | C-01 adds a JSON field. C-02 changes rendered text for two expression forms that are **currently wrong** — additive in capability, corrective in output | PASS | See RK-01: C-02 is the one behaviour change; no golden contains the affected forms (R8) |
| PHIL-001 (verify before recording) | Applies | Every claim above reproduced against the tagged binary; commands recorded | PASS | — |
| PHIL-002 (record deviations) | Applies | This proposal exists because PROP-2026-0002 recorded FU-01 rather than silently fixing it | PASS | — |
| CODE-001 (commit only when asked) | Applies | Plan stages by name | PASS | — |
| CODE-002 / CODE-003 (helper and keyword lists) | Not applicable | No helper, directive or capture type added | NOT APPLICABLE | — |
| ARCH-001 (a public field never changes type) | Applies | `CaptureValue.expr` already exists and is untouched; only its serialization is added. `Expr` is read, not altered | PASS | — |
| ARCH-002 (`capy-core` keeps one dependency) | Applies | JSON is written with the crate's own `gojson` writer, as `ast_json.rs` already does | PASS | — |
| ARCH-003 (engine never aborts its host) | Applies | Serialization is total; deeply nested expressions recurse — see RK-03 | PASS | Bound the recursion as `ast_json` already does for nodes |
| QUAL-001 (regression across the corpus) | Applies | R8: all 125 goldens byte-identical | PASS | — |
| QUAL-002 (measurable claim predeclared) | Applies | JSON size growth predeclared below | PASS | — |
| QUAL-003 (a test that cannot fail is not a test) | Applies | R7 exists because T-27 could not fail for the right reason — its corpus omitted the broken forms | PASS | — |
| GATE-001 / GATE-002 | Applies | Full gate plus golden corpus | PASS | — |

## Implementation Design

### Methods by Use Case

| Change ID | UC IDs | Method and Execution Order | Positive Path | Negative / Failure Path | Requirement Fulfilment | Feasibility |
|---|---|---|---|---|---|---|
| C-01 | UC-01, UC-03 | Add `expr_val(&Expr) -> Val` to `domain/ast_json.rs`, mirroring the existing `span_val` / node serializers; emit `("expr", …)` on the capture object | Every `Expr` variant maps to a tagged object | Unknown variant → must not silently drop; `Expr` is in-crate so the match is exhaustive | R1, R2, R3, R4 | proven — the writer and the data are both present |
| C-02 | UC-02 | In `expr_to_text.rs`, wrap `Expr::Not`'s operand with the existing `wrap_if_looser` at prefix-`not` precedence; apply `wrap_if_looser` / `wrap_if_looser_or_equal` to `Expr::Compare`'s children as `Expr::Binary` already does | `not (1 == 2)` and `1 == (2 == 2)` round-trip | Over-parenthesising is safe but ugly — guard on precedence, do not wrap unconditionally | R5, R6 | proven — the helpers exist and are used two arms away |
| C-03 | — | Extend the T-27 corpus in `rust/tests/precedence.rs` with `not (…)`, `not a == b`, `1 == (2 == 2)`, `(a and b) or c`, and mixed forms | Fails before C-02, passes after | — | R7 | proven |
| C-04 | — | Restore the `not (1 == 2)` case in `samples/operator-precedence/`, regenerate that one golden, update its README caveat | Golden shows correct text and value | — | R10 | proven |
| C-05 | — | Documentation: `ast-json.md` schema row and an `Expr` shape section; drop the caveat in `language-frontend.md`, `roadmap.md`, `samples/language-frontend/README.md` | One claim, no asterisk | — | R9 | proven |

### Added, Changed and Removed Contracts

| Item | CRUD | Kind | Name | Type / Schema | Consumers | Requirements |
|---|---|---|---|---|---|---|
| Capture field | CREATE | JSON field | `expr` | tagged object, or null | every `--json` consumer | R1, R2 |
| Expression node | CREATE | JSON shape | `{kind, …}` per `Expr` variant | documented in `ast-json.md` | same | R3 |
| Text rendering | UPDATE | behaviour | `expr_to_text` for `Not` / `Compare` | adds parentheses where required | libraries emitting `${capture}` | R5, R6 |
| `schema_version` | READ | contract | stays `1` | — | same | R4 |

### Ordering

```text
  C-02 + C-03 ────▶ C-04 ────▶ C-01 ────▶ C-05
  (fix + test)      (sample)   (serialize)  (docs)
       │                           │
       │  fix the text BEFORE      │  serialize the tree AFTER the text is
       │  documenting that text    │  trustworthy, so the docs can describe
       │  as trustworthy           │  both routes in one pass
       ▼                           ▼
   no golden moves             one new JSON key
```

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected |
|---|---|---|---|
| **Both changes (recommended)** | One coherent claim: the tree is reachable and the text is trustworthy | Two files of engine change | **Selected** |
| C-01 only (serialize, leave the round-trip bug) | Smaller | The `text` field stays a trap, and a consumer comparing `expr` against `text` finds them disagreeing | Rejected |
| C-02 only (fix the bug, no JSON tree) | Smallest; fixes real wrongness | Leaves P-01 — non-Rust consumers still re-implement an expression parser | Rejected as the end state, acceptable as a first increment |
| Serialize `expr` only when `--expr` is passed | Smaller default payload | A schema that changes shape by flag is harder to consume than one that is always complete | Rejected — see the size measurement |
| Emit an S-expression string instead of a tree | Compact | Puts consumers back in the parsing business, which is the problem | Rejected |
| Fix `expr_to_text` by always parenthesising | Trivially correct | `a + b` renders `(a) + (b)`; unreadable target code | Rejected |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback |
|---|---|---|---|---|
| RK-01 | C-02 changes emitted target text for a library using `not (…)` or nested comparison in a capture | Output differs from 0.22.0 | The change makes it **correct**; no golden in the 125-case corpus contains these forms (R8 verifies). Call it out in the release notes as a fix | Revert `expr_to_text.rs` |
| RK-02 | A consumer has a strict JSON schema that rejects unknown keys | Their pipeline breaks | `ast-json.md` already instructs ignoring unknown keys; note it in the release notes | — |
| RK-03 | Deeply nested expression recurses during serialization | Stack growth | Parse depth is already bounded at 64; mirror whatever bound the node serializer uses | — |
| RK-04 | JSON payloads grow materially | Larger output for expression-heavy sources | Predeclared measurement below with a threshold | Gate the field if exceeded |
| RK-05 | The `kind` tag names are chosen badly and have to change later | A schema bump | Settle names in the plan against `Expr`'s variants; publish once | Bump `schema_version` |

## Security Impact

None beyond what `ast-json.md` already documents: a tree echoes source text, so
`--json` output carries whatever the source carried. The `expr` field carries
the same content in structured form and does not widen that surface.

## Operational Impact

None. No new dependency, no new command, no CI change beyond the tests added.

## Compatibility Impact

Additive for JSON consumers (one new key, `schema_version` unchanged). For
libraries emitting captured source text, C-02 changes output **only** where it
is currently wrong — verified against the whole corpus by R8.

## Test and Validation Design

| ID | Type | Requirement IDs | Scenario | Exact Procedure | Expected Result |
|---|---|---|---|---|---|
| T-01 | unit | R5, R6, R7 | Round-trip over the extended corpus | `cargo test --test precedence` | Passes; **fails before C-02**, which must be demonstrated |
| T-02 | unit | R1, R2, R3 | Every `Expr` variant serializes and is structurally recoverable | new test in `rust/tests/` | Each variant emits its `kind`; no variant unhandled |
| T-03 | integration | R8 | The corpus does not move | `cargo test --test golden` | 125 passed, 8 skipped, 0 failed — byte-identical except the one golden C-04 intentionally regenerates |
| T-04 | integration | R10 | The restored `not (…)` case | `samples/operator-precedence/` golden | Source text and evaluated value now agree |
| T-05 | manual | R4, R9 | Docs match the emitted JSON | Run each command quoted in `ast-json.md` and `language-frontend.md` | Byte-identical to the page |
| T-06 | regression | GATE-001 | Full gate | build, `clippy --all-targets -D warnings`, `test --workspace`, `mkdocs build --strict` | all exit 0 |

### Measurement and Validation

| Measurement | Baseline Method | Test Command | Acceptance Threshold | Report Destination |
|---|---|---|---|---|
| `--json` payload size, expression-heavy source | `capy ast samples/language-frontend/lib.capy samples/language-frontend/main.capy --json \| wc -c` at 0.22.0 | same after C-01 | ≤ **2.0×** baseline | plan evidence log |
| `--json` payload size, expression-free source | same, on a sample with no `any` captures | same | ≤ **1.05×** baseline | same |
| Golden corpus | 125 passed / 8 skipped at 2026-09-16 | `cargo test --test golden` | 125 / 8, one golden intentionally regenerated (C-04) | same |
| Round-trip corpus size | T-27 cases at 0.22.0 | `rust/tests/precedence.rs` | strictly greater, covering `Not` and nested `Compare` | same |

## Documentation, Demo and Release Impact

| Artifact | Path | CRUD | Required Content | Release Gate |
|---|---|---|---|---|
| Schema | `docs/ast-json.md` | UPDATE | `expr` row, an `Expr` shape section, why `schema_version` stays 1 | T-05 |
| Front-end guide | `docs/language-frontend.md` | UPDATE | Drop the "embed in Rust for expression trees" caveat and the two-route diagram's ❌ | T-05 |
| Roadmap | `docs/roadmap.md` | UPDATE | Move "expression trees in `capy ast --json`" to Already shipped | — |
| Sample | `samples/language-frontend/README.md` | UPDATE | Route table: both routes now give trees | — |
| Sample | `samples/operator-precedence/` | UPDATE | Restore the `not (…)` case; drop the round-trip caveat from the README | T-04 |
| Release | `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | An **Added** entry for `expr`, a **Fixed** entry for the round-trip | GATE-001 |
| Version source | `rust/Cargo.toml` | UPDATE | Minor bump — C-02 is a behaviour fix | Release Management |

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases | Changes | Tests |
|---|---|---|---|---|---|
| R1, R2, R3 | UQ-01, UQ-03 | G-01 | UC-01 | C-01 | T-02 |
| R4 | UQ-01 | G-01 | UC-03 | C-01 | T-05 |
| R5, R6 | UQ-01 | G-02 | UC-02 | C-02 | T-01 |
| R7 | UQ-01 | G-02 | — | C-03 | T-01 |
| R8 | UQ-01 | G-02 | UC-03 | C-02 | T-03 |
| R9 | UQ-01 | G-03 | UC-01 | C-05 | T-05 |
| R10 | UQ-01 | G-02 | UC-02 | C-04 | T-04 |

## Plan Strategy and Estimated Work

One plan, two increments, ordered so the corrective change lands before anything
documents the text as trustworthy:

| Increment | Changes | Gate |
|---|---|---|
| INC-1 | C-02, C-03, C-04 — fix the round-trip, extend the corpus, restore the sample case | T-01, T-03, T-04; demonstrate T-01 failing before the fix |
| INC-2 | C-01, C-05 — serialize `expr`, document both routes | T-02, T-05, T-06, size measurements |

INC-1 stands alone and is worth shipping even if INC-2 is deferred: it is a
correctness fix. INC-2 without INC-1 is not recommended (see Alternatives).

## Open Questions

| ID | Question | Why It Matters | Proposed Default |
|---|---|---|---|
| OQ-01 | What are the `kind` tag names for each `Expr` variant? | Published once, then fixed (RK-05) | Lower-snake names matching the variant: `number`, `str`, `bool`, `null`, `var`, `binary`, `compare`, `not`, `list`, `obj`, `call`. Settle in the plan against the full enum |
| OQ-02 | Should `Expr::Var` serialize its path as a list of steps, including index expressions? | `c[i]` is a var with an index step; flattening to `"c"` loses the index | Default: full step list with any index expression nested as an `expr` |
| OQ-03 | Is FU-02 (bare `2026-09-16` lexes as arithmetic) in scope? | It is a real upgrade hazard, recorded in PROP-2026-0002 | Default **no** — it is intended lexing. Handle as an upgrade note plus a better error message, in its own change |
| OQ-04 | Does `text` stay once `expr` exists? | Duplication | Default **yes, keep it** — transpilers emit text and would otherwise re-render it themselves |
| OQ-05 | Minor or patch version? | C-02 changes output | Default **minor** (0.23.0): a behaviour fix plus a new field |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | pending | | |
| Reviewer | Release Management | pending | | Confirms the version bump (OQ-05) and the RK-01 release-note wording |

## Related Documents

- `PROP-2026-0002` — established the front-end documentation; recorded FU-01, which C-02 fixes
- `PROP-2026-0001` UQ-02 — the original "analysis operates on expression trees" requirement
- `REL-0.22.0` — the release whose surface this completes
- `docs/language-frontend.md` — the claim this proposal removes the asterisk from
- `STD-2026-0000` rev 1 — standards index validated against

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial proposal — expression trees over JSON, and the parenthesis round-trip fix |
| 2 | 2026-09-17 | Olivier | Classified the two changes: C-02 is a defect against R12/TASK-018, C-01 is a gap with a layering counter-argument recorded. No scope change |
| 3 | 2026-09-17 | Olivier | Priority revised: the known consumer embeds in Rust, so C-01 serves future JSON-boundary consumers rather than a waiting one. C-02 unchanged — ship it. No design change |
