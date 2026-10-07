---
document_id: PROP-2026-0004
title: Ordered Alternation for Nonterminals
document_type: proposal
status: implemented

created_date: 2026-09-17
last_updated: 2026-10-07
document_revision: 5

approval:
  approved_by:
    - Olivier
  approved_date: 2026-10-07

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine
  - Ambit (consumer)

systems:
  - Capy

components:
  - capy-core
  - docs
  - samples

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Adds ordered choice to the library-authoring grammar — a capture whose type names several functions, tried in declaration order — so a nonterminal can be "either this shape or that one". Completes the four grammar combinators and is the single change that makes recursive expression grammars expressible.

reason: Capy has sequence, repetition and recursion but no choice. A capture type names exactly one function, so a rule cannot say "an argument is a nested call OR an atom". Verified against 0.22.0: a declared call grammar parses `add(3, 5)` and `add(a + 1, b * 2)` but fails on `add(3, mul(4, 5))`. Every nested expression grammar hits this, and the front-end positioning published in PROP-2026-0002 depends on it.

related_documents:
  - PROP-2026-0002
  - PROP-2026-0003
  - PROP-2026-0001
  - REL-0.22.0
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - grammar
  - parser
  - nonterminals
  - alternation

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-17
---

# Ordered Alternation for Nonterminals

> **Status:** Implemented in 0.23.0 (approved 2026-10-07, Olivier)
> **Created:** 2026-09-17
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, docs, samples

## Summary

A grammar system needs four combinators. Capy has three.

```text
  ┌──────────────┬─────────────────────────────────────────┬────────┐
  │ sequence     │ arg literal / arg capture, in order      │  ✅    │
  │ repetition   │ param*   param+   sep ","   join ", "    │  ✅    │
  │ recursion    │ a capture whose type names a function    │  ✅    │
  │ CHOICE       │ operand = call | atom                    │  ❌    │
  └──────────────┴─────────────────────────────────────────┴────────┘
```

Without choice you can descend, but every level must have exactly one shape —
so recursion is one level deep in practice. This proposal adds ordered choice
and nothing else.

## Decision Requested

Approve **Option A** in *Alternatives Considered*: a capture type may name
several functions separated by `|`, tried left to right, first match wins.

```
arg capture args operand* sep ","
function operand
    bare
    arg capture v call | number | ident      # <- the change
end
```

Approval does **not** authorize: a built-in source-language expression grammar
(`PROP-2026-0001` Non-Goal 5); backtracking across statement boundaries;
C-style call syntax in `any`; or any change to the `any` value grammar.

## Priority

**Not urgent, and that is now established rather than assumed.** The consumer
who raised it confirmed on 2026-09-17 that they are unblocked, took the interim
(flat calls for M1, nesting deferred to M2), and found only **two** examples in
their corpus that need rewriting.

```text
  urgency   LOW      no consumer is blocked; the interim is verified and accepted
  value     HIGH     completes the four grammar combinators; every nested
                     expression grammar hits this, not just this consumer
  risk      MEDIUM   touches the matcher and the left-recursion guard (RK-03)
```

That combination argues for scheduling it deliberately — behind the
`PROP-2026-0003` INC-1 correctness fix, and with OQ-02 settled before coding —
rather than rushing it to unblock someone who is not blocked.

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "B1 — Capy cannot parse `f(x, y)`. … the workaround is exactly the thing they forbid: declare a per-shape statement function for each call position … and it still cannot nest." | `.ignore/capy_missing.md`, Ambit consumer, 2026-09-17 | The headline is wrong — `bare` makes `f(x, y)` declarable, verified. The **"still cannot nest"** clause is correct, and its cause is the absence of choice, not per-position functions. |
| UQ-02 | "so what to do?? code change for capy or them" | User, 2026-09-17 | Both: the consumer adapts for M1, Capy closes the gap. This document is the Capy half; `.ignore/capy-missing-reply.md` is the consumer half. |
| UQ-03 | "is that a bug or them misunderstanding how to use capy" | User, 2026-09-17 | Neither cleanly. A missed directive (`bare`) plus a real expressiveness gap. This proposal addresses the gap. |
| UQ-04 | "No blocking change needed. … Nested calls — their option 1. M1 ships flat calls; nesting is an M2 item gated on PROP-2026-0004. **Only two examples in the corpus need rewriting** (`divide(10, read_int())`, `sqrt(a) / (b * c)`)." | Ambit consumer, rev-5 reply, 2026-09-17 | **Confirms the shape** — alternation is what they need, and they accepted the interim. Also **lowers the urgency**: the requesting consumer is unblocked and only two of their examples are affected. See *Priority* below. |

| UQ-05 | "why not use mut" / "is this true??" — a consumer design note claimed `mut d: Door` cannot replace a trailing `writes d` because the parameter list is one rule and the grammar has no alternation. | User, 2026-10-07 | The grammar claim is **correct** and is a second, independent motivation for this proposal (P-05). Verified, not assumed — see *Problem and Evidence* and `.ignore/mut-vs-writes-grammar-limits.md`. |

## Problem and Evidence

### Current User Journey

```text
[language author] -> declares `call` as a grammar rule            ✅ works
                  -> `add(3, 5)`                                  ✅ parses
                  -> `add(a + 1, b * 2)`                          ✅ expression args
                  -> `add(3, mul(4, 5))`                          ❌
                        |
                        +-> needs: an argument is a CALL or an ATOM
                        +-> Capy: a capture type names exactly ONE function
                        +-> exits: prefix calls, or bounded hand-rolled depth,
                                   or drop nesting from the language
```

| Problem | Affected Users | Evidence and Inline Source | Consequence | UQ IDs |
|---|---|---|---|---|
| P-01 | anyone declaring a recursive grammar | Verified 0.22.0. With `operand` declared `bare` over `any`: `return add(3, 5)` → `return add(3, 5);`, `return add(a + 1, b * 2)` → renders; `return add(3, mul(4, 5))` → ``error: expected `)`, found "(" in `call` `` | Nesting stops at one level, in any grammar whose sub-structure has more than one shape | UQ-01 |
| P-02 | same | `docs/library-keywords.md:178` — "The `TYPE` in `arg capture NAME TYPE`. Any **function or type name** also works as a type". Singular by construction; there is no union form. `grep -i alternation docs/library-authoring.md docs/library-keywords.md docs/how-capy-parses.md` → **0 hits** | The gap is structural, not a missing sugar | UQ-01 |
| P-03 | the published positioning | `docs/language-frontend.md` (shipped by `PROP-2026-0002`) invites readers to declare their own expression syntax with function-as-type captures when `any` does not fit. That advice tops out at one nesting level | The front-end claim is weaker than the page implies | UQ-02, UQ-03 |
| P-04 | new library authors | `bare` is the key to a pure-capture nonterminal and appears once in `docs/library-keywords.md:171` as a table row, with no worked example. A careful external reviewer read the whole reference and missed it | Authors conclude a shape is impossible when it is merely undocumented | UQ-01 |
| P-05 | anyone declaring a signature where **some** parameters carry a marker (`mut`, `ref`, `out`) and others do not | Verified 2026-10-07 on the repo debug build. A `param` rule requiring a leading `mut` accepts `def f(mut c: Counter, mut n: int)` but rejects `def f(mut c: Counter, n: int)` → ``error: expected `mut`, found "n" in `param` ``. `arg capture ps mut_param \| plain_param* sep ","` → ``unexpected extra tokens after NAME [TYPE] …`` | A per-parameter prefix works only when **every** parameter has it, so authors fall back to a trailing clause on the whole signature (e.g. `writes d`), which cannot say which parameter it applies to by position | UQ-05 |

### Same gap, second shape — the parameter marker

```text
  today                                    with PROP-2026-0004

  def f(mut c: Counter, n: int)            param_list = mut_param | plain_param
        └─ ONE rule `param`                    ├─ mut_param    "mut" NAME ":" TYPE
           must say "mut" for ALL              └─ plain_param  NAME ":" TYPE
           params or for NONE                  mixed signature parses            ✅
        ❌ mixed signature rejected
```

The fix is the same ordered choice. No second mechanism is needed.

### What does NOT need this proposal — verified 2026-10-07

Several forms a consumer might assume need alternation do not. Statement-level
shapes are already a choice among functions; only a choice **inside one capture**
is missing.

```text
  ┌─────────────────────────────────────────────┬──────────┬───────────────────────┐
  │ form                                        │ today    │ how                   │
  ├─────────────────────────────────────────────┼──────────┼───────────────────────┤
  │ lifecycle T { … } head + block              │   ✅     │ block_open "{" close  │
  │ state s { update: replace }  (one line)     │   ✅     │ literal "{" / "}"     │
  │ state s {⏎ k: v ⏎}  (nested attribute block)│   ✅     │ nested block_open     │
  │ a -> b by f                                 │   ✅     │ arg literal "->"      │
  │ abi_type(T, C) inside a block               │   ✅     │ generic call rule     │
  │ lifecycle.exit(T, f)                        │   ✅*    │ needs its own rule —  │
  │                                             │          │ `ident` has no "."    │
  │ mixed plain / `mut` parameters              │   ❌     │ THIS PROPOSAL         │
  │ nested calls  add(3, mul(4, 5))             │   ❌     │ THIS PROPOSAL         │
  └─────────────────────────────────────────────┴──────────┴───────────────────────┘
   * a plain call rule fails on the dot: ``expected `(`, found "."``; a separate
     `ns "." fname "(" … ")"` rule parses, in either declaration order.
```

Two authoring hazards surfaced and belong in the docs (R10), not in the engine:

- A `bare` rule whose first arg is an `ident` capture can swallow a keyword rule
  that follows the same ident shape (``expected `->` or `{` ``); the keyword rule
  must also be `bare` with an explicit `arg literal "state"`.
- A statement declared with `block_open` takes its attributes on their own lines;
  the one-line `{ … }` form needs literal braces instead.

### What already works — do not re-solve this

```text
$ capy run lib.capy s.capy     # `operand` declared `bare`, args `operand* sep ","`
return add(3, 5)             ->  return add(3, 5);
return add(a + 1, b * 2)     ->  return add(a + 1, b * 2);
```

Flat calls and expression-valued arguments are already expressible. The proposal
is scoped to the one thing that is not.

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-01 | A nonterminal can have more than one shape | P-01, P-02 | Ordered choice in a capture type | `add(3, mul(4, 5))` parses from a declared grammar |
| G-02 | Recursion nests to arbitrary depth, bounded only by the existing depth limit | P-01 | Choice terminates the recursion at an atom alternative | A 5-deep nest parses; a 70-deep nest hits the existing 64 bound with its normal error |
| G-03 | The tree tells you which alternative matched | P-01 | `sub[].func` already names the matched function — no schema change | `jq '.tree…sub[0].func'` returns `call` or `number` |
| G-04 | `bare` stops being a hidden feature | P-04 | Worked example in the authoring guide and a sample | A reader reaches a pure-capture nonterminal without reverse-engineering it |

**Non-goals**

- **The consumer's two other asks are out of scope here**, and deliberately so —
  they are unrelated mechanisms that happen to have arrived in the same message:
  *optional captures before a block opener* (`default` applies only to a
  genuinely trailing capture, and `capy check` accepts the library anyway), and
  *statement continuation after a block close* (`} else {`). Both are verified,
  both are on the [roadmap](../../docs/roadmap.md), and neither shares any code
  path with alternation. Folding them in would make one proposal that has to be
  approved or rejected as a bundle.
- **No built-in source expression grammar.** `PROP-2026-0001` Non-Goal 5 stands;
  `GOAL-001` (zero default grammar) is why this is a *combinator* for library
  authors rather than syntax the engine ships.
- **No change to the `any` value grammar** — no C-style calls, no `&&`/`||`/`!`.
- **No unordered/ambiguous choice.** First match wins, PEG-style.
- **No backtracking across a statement boundary.** See RK-02.
- **No new JSON schema fields** — `sub[].func` already carries the answer (G-03).

## Proposed User Journey

```text
[language author] -> function operand
                         bare
                         arg capture v call | number | ident
                     end
                  -> `add(3, mul(4, 5))`
                        |
                        +-> arg 1: try `call` → no; try `number` → 3          ✅
                        +-> arg 2: try `call` → mul(4, 5) → recurse           ✅
                        +-> tree: operand -> call -> operand -> number
```

## Requirements

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal IDs |
|---|---|---|---|---|---|
| R1 | A capture type may list several **library function** names separated by `\|` (built-in and declared types are not alternatives; see OQ-08) | functional | P-01 | `arg capture v call \| number` loads and matches either | G-01 |
| R2 | Alternatives are tried **left to right**; the first that matches wins | functional | determinism; mirrors the existing candidate ordering | Reordering alternatives changes which matches, provably, in a test | G-01 |
| R3 | An alternation capture may carry a repetition suffix (`operand*`, `operand+`) and `sep` / `join` | functional | argument lists are the motivating case | `args (call \| number)* sep ","` parses | G-01 |
| R4 | Alternation composes with recursion to arbitrary depth | functional | P-01 | A 5-deep nest parses; depth 70 reports the existing depth error | G-02 |
| R5 | The left-recursion guard detects a cycle reachable through **any** alternative | safety | `ARCH-003`; today's guard walks a single type edge per capture | A left-recursive alternative is refused at load with the existing cycle message | G-02 |
| R6 | A failed alternation reports what was expected as the **union** of the alternatives | UX | matches the 0.22.0 "expected X, Y or Z" diagnostic style | ``expected `call`, `number` or `ident`, found …`` | G-01 |
| R7 | No existing library changes behaviour | compatibility | `GOAL-002` | 125 goldens byte-identical | G-01 |
| R8 | `sub[].func` identifies the matched alternative; no schema change, `schema_version` stays 1 | API | G-03 | a JSON test asserts the discriminator | G-03 |
| R9 | `capy check` lists an alternation capture's type readably | UX | introspection stays truthful | the union is printed, not the first alternative | G-01 |
| R10 | `bare` gains a worked example in the authoring guide, and a sample demonstrates a recursive expression grammar | docs | P-04 | new section + `samples/` directory with goldens | G-04 |
| R11 | Alternation is documented as ordered choice with its termination rule | docs | P-02 | `library-authoring.md` and `library-keywords.md` updated | G-01 |

## Use Cases

| ID | User Outcome | Actor | Surface | Inputs | Outputs | Negative Paths | Requirement IDs |
|---|---|---|---|---|---|---|---|
| UC-01 | Nested calls parse | language author | declared grammar | `add(3, mul(4, 5))` | tree, 3 levels | none of the alternatives match → union diagnostic (R6) | R1, R2, R3, R4, R6 |
| UC-02 | An expression grammar terminates | language author | declared grammar | `f(g(h(1)))` | tree, 4 levels | depth > 64 → existing bound error | R4 |
| UC-03 | A consumer knows which shape matched | analyzer author | `capy ast --json` | as UC-01 | `sub[0].func == "call"` | — | R8 |
| UC-04 | A left-recursive alternative is caught at load | library author | `capy check` | `expr = expr \| atom` | refused with the cycle trace | — | R5 |
| UC-05 | An author finds `bare` | new library author | docs | — | a worked pure-capture nonterminal | — | R10 |
| UC-06 | A signature mixes marked and unmarked parameters | language author | declared grammar | `def f(mut c: Counter, n: int)` | tree with `sub[0].func == "mut_param"`, `sub[1].func == "plain_param"` | a parameter matching neither → union diagnostic (R6) | R1, R2, R3, R6, R8 |

### UC-01 — Contract (illustrative; settled in the plan)

```text
$ capy ast lib.capy s.capy
return 1:1-1:25
  value:
    call 1:8-1:25
      fname = "add" 1:8
      args:
        operand 1:12-1:13
          v = "3" 1:12
        operand 1:15-1:24
          v:
            call 1:15-1:24
              fname = "mul" 1:15
              args: …
$ echo $?
0
```

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says `document_revision: 2` — see OQ-05) | 2026-09-17 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 (zero source-language grammar) | Applies | Alternation is a **library-authoring combinator**. The engine still ships no source grammar; the author declares every alternative | PASS | — |
| GOAL-002 (engine changes additive) | Applies | New syntax in a position that is currently a parse error (`\|` in a capture type). No existing library can contain it | PASS | R7 verifies |
| PHIL-001 (verify before recording) | Applies | Every claim in *Problem and Evidence* was run against 0.22.0, including the ones that disprove the consumer's headline | PASS | — |
| PHIL-002 (record deviations) | Applies | The consumer's stated cause was wrong and is corrected in P-01 rather than repeated | PASS | — |
| CODE-001 (commit only when asked) | Applies | Plan stages by name | PASS | — |
| CODE-002 (helper list in sync) | Not applicable | No helper added | NOT APPLICABLE | — |
| CODE-003 (keyword list in sync) | **Applies** | `\|` is new library-authoring syntax → `docs/library-keywords.md` **must** be updated in the same change, per `AGENTS.md` | PASS | R11 is the binding obligation |
| ARCH-001 (a public field never changes type) | **At risk** | The capture's declared type is currently a single name. Widening it to a list must not change an existing public field's type | NEEDS HUMAN REVIEW | See RK-01 and OQ-02 — decide additive representation before coding |
| ARCH-002 (`capy-core` keeps one dependency) | Applies | No dependency | PASS | — |
| ARCH-003 (engine never aborts its host) | Applies | R5 extends the left-recursion guard across alternatives; the depth bound already exists | PASS | R5 is the gate |
| QUAL-001 (regression across the corpus) | Applies | R7 — 125 goldens byte-identical | PASS | — |
| QUAL-002 (measurable claim predeclared) | Applies | Parse-time budget predeclared below | PASS | — |
| QUAL-003 (a test that cannot fail is not a test) | Applies | R2's test must fail when alternatives are reordered, or it proves nothing | PASS | — |
| GATE-001 / GATE-002 | Applies | Full gate plus corpus | PASS | — |

## Implementation Design

### Methods by Use Case

| Change ID | UC IDs | Method | Positive Path | Negative Path | Requirements | Feasibility |
|---|---|---|---|---|---|---|
| C-01 | UC-01 | Library loader: parse `A \| B \| C` in a capture's type position into an ordered list | loads | unknown name in any position → load error naming it | R1, R9 | proven — the loader already resolves one name |
| C-02 | UC-01, UC-02 | Matcher: try each alternative in order at the capture position, first success wins; on total failure emit the union expectation | nests via existing recursion | all fail → R6 diagnostic | R2, R4, R6 | proven — candidate ordering already exists at statement level |
| C-03 | UC-01 | Repetition: alternation inside `*` / `+` with `sep` / `join` | `(call \| number)* sep ","` | — | R3 | proven |
| C-04 | UC-04 | Left-recursion guard: walk **every** alternative edge when detecting cycles | refuses at load | — | R5 | needs care — see RK-03 |
| C-05 | UC-03 | Verify `sub[].func` discriminates; add a test. No serializer change expected | — | — | R8 | proven |
| C-06 | UC-05 | Docs: ordered-choice section, `bare` worked example, keyword table rows | — | — | R10, R11 | proven |
| C-07 | UC-01 | `samples/expression-grammar/` — a recursive expression grammar with nested calls, goldens incl. `.expected-ast.txt` | — | — | R10 | proven |

### Backtracking boundary

```text
  statement                    ← never backtrack across this
    └── capture position       ← alternation tries alternatives HERE
          ├── alt 1: call      ← may consume, fail, and be rewound
          ├── alt 2: number
          └── alt 3: ident
```

Rewind is local to the capture. A statement that begins matching is not
re-selected on an inner alternation failure — that is today's behaviour and R7
depends on it staying.

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected |
|---|---|---|---|
| **A. `\|` in the capture type (recommended)** | Reads like every grammar notation; local to the capture; no new top-level directive; composes with `*` / `sep` / `join` | Adds a metacharacter to the type position | **Selected** — smallest surface, familiar |
| B. A named union declared once — `nonterminal operand = call \| number` | Reusable; one place to edit | A new top-level directive and a new namespace; more machinery for the same power | Rejected for M1; revisit if unions repeat (OQ-04) |
| C. Variant functions — `function operand_call` + `variant_of operand` | No new metacharacter | Two concepts (function, variant) where one would do; scatters a rule across the file | Rejected |
| D. Fallback type — `arg capture v call default_type any` | Tiny change | Two alternatives only, and "default" misdescribes it | Rejected — solves the example, not the class |
| E. Multi-pattern function with `alt … alt … end` | Keeps everything inside one `function` | Largest syntax addition; nests badly inside an arg list | Rejected |
| F. Do nothing; consumers use prefix calls | Zero cost | Leaves three of four combinators and contradicts the published front-end positioning | Rejected as an end state; **it is the correct interim**, and is what the consumer reply recommends |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback |
|---|---|---|---|---|
| RK-01 | Widening the capture's declared type breaks `ARCH-001` | A public field changes type | Represent alternatives additively — keep the existing single-name field populated with the first alternative, add a list alongside; decide in the plan (OQ-02) | Revert |
| RK-02 | Backtracking makes pathological grammars slow | Parse time blows up | Rewind is capture-local; alternatives are tried once each, no memoisation needed at this depth. Budget predeclared below | Cap alternatives per capture |
| RK-03 | The left-recursion guard misses a cycle that only exists through one alternative | Stack overflow at parse time — the exact failure `PROP-2026-0001` R0 removed | R5 is a blocking requirement with its own test; treat a cycle through *any* alternative as left-recursive | Revert C-01 |
| RK-04 | First-match-wins surprises an author whose alternatives overlap | Wrong alternative silently matches | Document ordered choice prominently (R11); `capy check` prints the order (R9) | — |
| RK-05 | The union diagnostic gets long for many alternatives | Noisy errors | Cap the listed names the way the existing expectation-union does | — |

## Security Impact

None. No new execution surface, no new dependency, no I/O.

## Operational Impact

None beyond the added tests and one sample.

## Compatibility Impact

`|` in a capture type position is currently a load error, so no existing library
can contain it. R7 verifies the corpus is byte-identical.

## Test and Validation Design

| ID | Type | Requirement IDs | Scenario | Procedure | Expected Result |
|---|---|---|---|---|---|
| T-01 | unit | R1, R2 | Ordered choice selects the first match | reorder alternatives; assert the match changes | **Must fail when the order is ignored** (QUAL-003) |
| T-02 | integration | R1, R3, R4 | Nested calls | `samples/expression-grammar/` golden | `add(3, mul(4, 5))` parses; 5-deep nest parses |
| T-03 | unit | R4 | Depth bound still applies | 70-deep nest | existing depth diagnostic, no panic |
| T-04 | unit | R5 | Left recursion through an alternative | `expr = expr \| atom` | refused at load with the cycle trace |
| T-05 | unit | R6 | Union expectation | input matching no alternative | ``expected `call`, `number` or `ident`, found …`` |
| T-06 | integration | R7 | Corpus unchanged | `cargo test --test golden` | 125 passed, 8 skipped, 0 failed |
| T-07 | unit | R8 | Discriminator | `--json` on UC-01 input | `sub[0].func` names the matched alternative; `schema_version` 1 |
| T-08 | manual | R9, R10, R11 | Docs are true | run every command quoted in the changed pages | byte-identical |
| T-09 | regression | GATE-001 | Full gate | build, clippy `-D warnings`, test, `mkdocs --strict` | all exit 0 |
| T-10 | integration | R1, R2, R3, R8 | Mixed marked / unmarked parameters (UC-06) | `samples/expression-grammar/` or a sibling sample: `param_list = mut_param \| plain_param`; input `def f(mut c: Counter, n: int)` | parses; `sub[0].func` is `mut_param`, `sub[1].func` is `plain_param`; the same input is **rejected today** (P-05 evidence) |

### Measurement and Validation

| Measurement | Baseline | Test Command | Threshold | Destination |
|---|---|---|---|---|
| Golden suite wall time | current `--test golden` duration | same | ≤ **1.15×** | plan evidence log |
| Transpile time, corpus median | `REL-0.22.0` recorded 192.036 µs best of 5 | same benchmark | ≤ **10 %** regression, matching the release-series budget | same |
| Golden corpus | 125 passed / 8 skipped | `cargo test --test golden` | unchanged | same |
| `capy-core` direct deps | 1 (`regex`) | `cargo tree -p capy-core --depth 1` | exactly 1 | same |

## Documentation, Demo and Release Impact

| Artifact | Path | CRUD | Required Content | Gate |
|---|---|---|---|---|
| Keyword cookbook | `docs/library-keywords.md` | UPDATE | `\|` row in the capture-type table — **required by CODE-003** | GATE-001 |
| Authoring guide | `docs/library-authoring.md` | UPDATE | Ordered-choice section; a worked `bare` pure-capture nonterminal (R10) | T-08 |
| Front-end guide | `docs/language-frontend.md` | UPDATE | Replace the one-level-nesting caveat; add the recursive grammar | T-08 |
| Cheat sheet, features | `docs/syntax-cheat-sheet.md`, `docs/features.md` | UPDATE | one row each | — |
| Sample | `samples/expression-grammar/` | CREATE | recursive grammar, nested calls, `.expected.txt` + `.expected-ast.txt` | T-02 |
| Sample index | `samples/README.md` | UPDATE | Parser-surface row; re-derive counts | — |
| Release | `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | **Added** entry | GATE-001 |
| Version source | `rust/Cargo.toml` | UPDATE | minor bump | Release Management |
| Consumer | `.ignore/capy-missing-reply.md` | READ | the interim guidance this supersedes once shipped | — |

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases | Changes | Tests |
|---|---|---|---|---|---|
| R1, R2 | UQ-01 | G-01 | UC-01 | C-01, C-02 | T-01 |
| R3 | UQ-01 | G-01 | UC-01 | C-03 | T-02 |
| R4 | UQ-01 | G-02 | UC-01, UC-02 | C-02 | T-02, T-03 |
| R5 | UQ-01 | G-02 | UC-04 | C-04 | T-04 |
| R6 | UQ-01 | G-01 | UC-01 | C-02 | T-05 |
| R7 | UQ-02 | G-01 | — | C-01…C-03 | T-06 |
| R8 | UQ-01 | G-03 | UC-03 | C-05 | T-07 |
| R9 | UQ-01 | G-01 | UC-04 | C-01 | T-08 |
| R10 | UQ-03 | G-04 | UC-05 | C-06, C-07 | T-02, T-08 |
| R11 | UQ-01 | G-01 | UC-05 | C-06 | T-08 |

## Plan Strategy and Estimated Work

One plan, three increments. C-04 is deliberately first: the guard must exist
before the feature that can produce the cycle.

| Increment | Changes | Gate |
|---|---|---|
| INC-1 | C-04 — extend the left-recursion guard across alternative edges, ahead of the feature | T-04, T-06 |
| INC-2 | C-01, C-02, C-03, C-05 — loader, matcher, repetition, discriminator | T-01, T-02, T-03, T-05, T-06, T-07 |
| INC-3 | C-06, C-07 — docs, `bare` example, sample, release notes | T-08, T-09 |

## Open Questions

| ID | Question | Why It Matters | Proposed Default |
|---|---|---|---|
| OQ-01 | Is `\|` the right metacharacter, given `\|>` is a valid multi-character punct token in source? | A lexing collision in the *library* file | Default **yes** — the type position is library syntax, not source; confirm the library lexer does not greedily take `\|>` there |
| OQ-02 | How are alternatives represented without breaking `ARCH-001`? | Blocking — see RK-01 | **RESOLVED 2026-10-07 (ADR-0003):** keep `cap_type` as alternative 1, add an ordered `alts` list holding alternatives 2…n beside it. No existing public field changes type |
| OQ-03 | Cap on alternatives per capture? | RK-02, RK-05 | Default **no hard cap**; revisit if the time budget moves |
| OQ-04 | Named unions (Alternative B) as follow-up? | Repetition across many rules | Default: defer; revisit once a real grammar shows the same union three or more times |
| OQ-05 | `program_docs/standards/index.md` says `document_revision: 2` in front matter and "Index revision: 1" in the body. Which do proposals cite? | Every proposal's baseline table cites it; the consumer hit this too | Default: the **body** counter is the one the text asks to quote — but reconcile the two, since it is a release-gate field |
| OQ-06 | A postfix optional marker inside a repeated nonterminal fails today: `param … arg capture mode ident default "in"` accepts `def f(c: Counter mut)` but rejects `def f(c: Counter)` (``expected `)`, found "c"``) and `def f(c: Counter mut, n: int)` (``expected `)`, found ","``). Is that a separate defect in `default`, or intended? | Alternation does **not** fix it; an author cannot use `default` as a workaround for choice | Default: **out of scope** here. Not investigated — raise as its own item if confirmed |
| OQ-07 | Should alternation also cover a *leading* optional token (`"mut"?`)? | A `mut_param \| plain_param` choice duplicates the shared tail (`NAME ":" TYPE`) | Default: **no** — keep to ordered choice (Alternative A). An optional-token combinator would be a fifth combinator; revisit only if duplication recurs |
| OQ-08 | May an alternative be a built-in or declared type (`call \| number` where `number` is a `type`)? | Matching a flat type uses `capture_value` with stop-literals, a different path from nonterminals | **RESOLVED 2026-10-07 (ADR-0003):** no — alternatives name library functions only. A flat alternative is written as a `bare` one-capture function, which is how P-04 already documents pure-capture nonterminals |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | **approved** | 2026-10-07 | Olivier, by explicit instruction to plan, implement and release. OQ-02 and OQ-08 settled in ADR-0003. Agent validation did not approve this proposal (§21.2) |
| Reviewer | Ambit (consumer) | **confirmed** | 2026-09-17 | rev-5 reply: alternation is the ask, gated to their M2; not a blocker for M1 (UQ-04) |

## Related Documents

- `.ignore/capy_missing.md` — the consumer review that surfaced this
- `.ignore/capy-missing-reply.md` — the reply, including the interim recommendation
- `.ignore/mut-vs-writes-grammar-limits.md` — what is and is not expressible for parameter markers (probes in `.ignore/mut-probe/`)
- `PROP-2026-0002` — published the front-end positioning this completes
- `PROP-2026-0003` — expression trees over JSON; independent, same consumer lane
- `PROP-2026-0001` Non-Goal 5 — why this is a combinator, not a built-in grammar
- `STD-2026-0000` — standards index validated against

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-17 | Olivier | Initial proposal — ordered alternation for nonterminals |
| 2 | 2026-09-17 | Olivier | Consumer confirmed the shape (UQ-04) and is unblocked; priority section added; their two other asks explicitly scoped out. No design change |
| 3 | 2026-10-07 | Olivier | Added P-05 (mixed parameter markers) as a second motivation, UQ-05, UC-06, T-10, OQ-06, OQ-07, and a verified table of forms that do **not** need alternation. No design change; Option A stands |
| 4 | 2026-10-07 | Olivier | **Approved.** OQ-02 resolved (additive `alts` list), OQ-08 added and resolved (alternatives are library functions; R1 narrowed from "function or type names"), approval recorded in front matter. Option A unchanged |
| 5 | 2026-10-07 | Olivier | **Implemented** in 0.23.0 under `PLAN-2026-0003` (`ADR-0003`). Validation: `RPT-2026-0003`. Deviations recorded there: R4 PARTIAL (depth message), R6 wording, R1 narrowed |
