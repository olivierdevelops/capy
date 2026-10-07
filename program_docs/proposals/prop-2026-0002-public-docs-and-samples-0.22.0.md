---
document_id: PROP-2026-0002
title: Public Documentation and Samples for the 0.22.0 Parser Surface
document_type: proposal
status: implemented

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 2

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine
  - Release Management

systems:
  - Capy

components:
  - docs
  - samples
  - capy-cli

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects
  - release-managers

scope: Defines exactly which files under `docs/` and `samples/` are created, updated or left alone so that the four capabilities shipped in REL-0.22.0 — diagnostics, error recovery, `capy ast`, and operator precedence — are discoverable on the published site and regression-covered by the sample corpus.

reason: REL-0.22.0 shipped four user-visible capabilities and two reference pages, but the pages a user actually reaches when a parse fails (`errors-and-debugging.md`, `troubleshooting.md`), every CLI summary except `cli.md`, the LLM brief, the roadmap and the entire `samples/` corpus still describe the pre-0.22.0 engine. Nothing in `samples/` exercises the new surface, so it cannot regress visibly.

related_documents:
  - REL-0.22.0
  - PROP-2026-0001
  - PLAN-2026-0002
  - RPT-2026-0002
  - DEMO-2026-0002
  - ADR-0001
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - documentation
  - samples
  - diagnostics
  - ast
  - precedence

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Public Documentation and Samples for the 0.22.0 Parser Surface

> **Status:** Draft
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0 →
> **Owner:** Capy Engine
> **Affected Components:** docs, samples, capy-cli (test harness only)

## Summary

`REL-0.22.0` shipped four capabilities and documented two of them well.
This proposal closes the rest of the loop:

```text
        WHAT SHIPPED                    WHERE IT IS DOCUMENTED TODAY
  ┌───────────────────────────┐   ┌──────────────────────────────────────┐
  │ Diagnostics (codes,       │──▶│ docs/diagnostics.md          ██████  │ good
  │ labels, severity)         │   │ docs/errors-and-debugging.md  ····   │ ABSENT
  │                           │   │ docs/troubleshooting.md       ····   │ ABSENT
  ├───────────────────────────┤   ├──────────────────────────────────────┤
  │ Error recovery            │──▶│ docs/diagnostics.md §64      ██████  │ good
  │ (Block.errors)            │   │ everywhere a user meets an    ····   │ ABSENT
  │                           │   │ error first                          │
  ├───────────────────────────┤   ├──────────────────────────────────────┤
  │ `capy ast [--json]`       │──▶│ docs/cli.md, docs/ast-json.md ██████ │ good
  │                           │   │ cheat sheet / features / LLM  ····   │ ABSENT
  │                           │   │ brief / editor-tooling               │
  ├───────────────────────────┤   ├──────────────────────────────────────┤
  │ Operator precedence       │──▶│ docs/language-reference.md   ██████  │ good
  │                           │   │ cheat sheet / features / FAQ  ····   │ ABSENT
  ├───────────────────────────┤   ├──────────────────────────────────────┤
  │ ALL FOUR                  │──▶│ samples/  (125 directories)   ····   │ ABSENT
  └───────────────────────────┘   └──────────────────────────────────────┘
```

Two reference pages exist and are accurate. The problem is **reach** — the
pages are only linked from the nav — and **corpus coverage**: not one of the
125 sample directories exercises diagnostics, recovery, `capy ast` or
precedence, so `cargo test --test golden` cannot notice if any of them breaks.

## Decision Requested

Approve:

1. The **file inventory** in *Expected Code and Documentation Changes* — 19
   updates and 8 new files under `docs/` and `samples/`.
2. **Option B** in *Alternatives Considered*: a test-only extension of
   `rust/tests/golden.rs` to recognise `<base>.expected-ast.txt`, so the new
   `capy ast` transcript in `samples/parse-recovery/` is verified rather than
   prose.

Approval does **not** authorize: any change to `capy-core` or `capy-cli`
shipped code; re-opening the frozen contracts in `ADR-0001`; regenerating any
existing golden; or closing the five Known Limitations of `REL-0.22.0`
(this proposal only requires that each is *published or explicitly deferred*).

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "for `program_docs/releases/rel-0.22.0-release-notes.md` … we need to edit and potentially add new files as well … `docs`, `samples` … write a proposal on how to edit those folders" | User, conversation, 2026-09-16 | Fact: the release document is the driver; `docs/` and `samples/` are the two target trees; new files are permitted. Ambiguity: "edit those folders" does not say whether the sample corpus must be *verified* or merely illustrative — resolved as verified, per `PHIL-001` and `QUAL-001`. |
| UQ-02 | (implied by UQ-01) The deliverable is a proposal, not the edits. | Same | No file under `docs/` or `samples/` is touched by this document. |

## Problem and Evidence

### Current User Journey

```text
[author] -> writes a script -> `capy run lib.capy script.capy` -> ONE error, no output
          -> opens docs/errors-and-debugging.md        (the page the error text points to)
          -> reads "Capy stops at the first error"      <- the 0.21.0 model
          -> never learns that `capy ast` would have shown ALL the errors
          -> never learns that a tree is available at all
                                    |
                                    +-> gives up / files a "confusing error" issue
```

```text
[tool author] -> wants the parse tree -> reads syntax-cheat-sheet.md "CLI quick reference"
              -> table lists run/check/docs/watch/fmt/build/lib/new   <- no `ast`
              -> concludes Capy has no structured output -> writes a regex scraper
```

| Problem | Affected Users | Evidence and Inline Source | Consequence | UQ IDs |
|---|---|---|---|---|
| P-01 | script authors | `grep -rni recover docs/errors-and-debugging.md docs/troubleshooting.md docs/getting-started.md` → **0 hits**; `grep -rln diagnostic docs/` does not list either page | The two pages a failing user lands on describe a stop-at-first-error engine that no longer exists | UQ-01 |
| P-02 | tool authors, agents | `grep -rln "capy ast" docs/` → only `cli.md`, `ast-json.md`, `diagnostics.md`, `whats-new.md` (4 of 76 pages). Absent from `syntax-cheat-sheet.md:156` CLI table, `docs/CAPY_FOR_LLMS.md:436` CLI quick reference, `docs/features.md:270` CLI section, `docs/editor-tooling.md:90` "CLI helpers for tooling" | The one command built for tooling is missing from every tooling-facing summary | UQ-01 |
| P-03 | library authors | `grep -rln precedence docs/` → 5 pages, of which `language-reference.md` and `inner-dsl.md` are correct and `tutorials/04-custom-operators.md` states the opposite ("Capy doesn't have operator precedence"). Absent from `syntax-cheat-sheet.md`, `features.md:100` "Lexical features", `CAPY_FOR_LLMS.md:236` "Expressions", `faq.md:60` "How do I define `x = 1`?" | Authors keep writing three-function workarounds for `a + b * c` | UQ-01 |
| P-04 | everyone | `samples/` has 125 directories, **none** covering diagnostics, recovery, `capy ast` or precedence; `rust/tests/golden.rs:91-92` only knows `.expected.txt` / `.expected-error.txt` and only calls `run::run` | The 0.22.0 surface has unit tests (`rust/tests/diagnostics.rs`, `precedence.rs`) but **zero** corpus coverage — a user-visible regression here would not show up in the golden suite | UQ-01 |
| P-05 | evaluators | `docs/roadmap.md:8` "Already shipped" lists `capy watch`, `fmt`, `build`, … but **not** diagnostics, recovery, AST output or precedence | The published roadmap understates the current release | UQ-01 |
| P-06 | contributors | `samples/README.md` says "**50 self-contained demos**" (there are 125), "Every demo ships a `lib.yaml`" (they ship `lib.capy`), "116 cases", and `go test ./...` in a Rust repo | The on-ramp for adding a sample is wrong in four places, independent of 0.22.0 | UQ-01 |
| P-07 | evaluators | `docs/showcase.md:902` "🩺 Errors that tell you how to fix them" quotes three error renderings produced before 0.22.0 and re-verified by nobody | Quoted output may silently diverge from the binary — the exact failure mode `REL-0.22.0` "Changed" already recorded once for `samples/bbcode-parser/mismatch` | UQ-01 |
| P-08 | Rust consumers | `grep -rn "byte offset" docs/*.md` → **0 hits**; `grep -rn wasm docs/diagnostics.md docs/ast-json.md` → **0 hits**. Known Limitations 1, 2, 4 and 5 of `REL-0.22.0` have no published home | A consumer discovers the limits by hitting them | UQ-01 |

**Verified counter-evidence (do not "fix" these).** `docs/cli.md:116`,
`docs/ast-json.md`, `docs/diagnostics.md`, `docs/language-reference.md:98`
(including the `(foo)` grouping caveat — Known Limitation 3), `docs/embedding.md:400`
("Getting the tree", `ParseResult`, the partial-parse warning) and
`docs/whats-new.md:7` are **already correct at 0.22.0**. They are the anchors
this proposal links *to*, not pages it rewrites.

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-01 | A user who hits a parse error reaches recovery and `capy ast` within one click of the error page | P-01 | A "when one error is not enough" section in `errors-and-debugging.md` and a diagnosis step in `troubleshooting.md`, both linking `diagnostics.md` | `grep -c "capy ast" docs/errors-and-debugging.md docs/troubleshooting.md` ≥ 1 each |
| G-02 | Every CLI summary in the tree lists `capy ast` | P-02 | Four table/list rows | 8 pages match `capy ast`, up from 4 |
| G-03 | Precedence is reachable from the three pages authors skim first | P-03 | Cheat-sheet block, `features.md` bullet, FAQ answer, LLM-brief grammar line | 4 new pages match `precedence` |
| G-04 | The 0.22.0 surface fails the golden suite if it regresses | P-04 | Two new samples + one test-harness branch | `cargo test --test golden` covers ≥ 3 new cases incl. one AST transcript |
| G-05 | Published status claims match the release | P-05, P-07 | Roadmap rows moved to "Already shipped"; every quoted error re-produced from the 0.22.0 binary | Each quoted block reproduced by a recorded command |
| G-06 | Adding a sample is documented correctly | P-06 | `samples/README.md` rewritten for counts, `lib.capy`, cargo, and the golden kinds | A contributor following it verbatim produces a passing sample |
| G-07 | Each 0.22.0 Known Limitation is published or explicitly deferred | P-08 | Limitation notes in `diagnostics.md`, `ast-json.md`, `embedding.md`, `roadmap.md` | 5/5 rows resolved in the traceability table |

**Explicit non-goals and boundaries**

- **No shipped-code change.** Nothing under `rust/src/` is touched. The single
  Rust edit proposed is in `rust/tests/golden.rs` (test-only, `ARCH-002` and
  `GOAL-002` unaffected).
- **No existing golden regenerated.** `CAPY_UPDATE_GOLDENS=1` is not run.
- **No new docs *page* for diagnostics or AST.** `diagnostics.md` and
  `ast-json.md` already exist and are correct; duplicating them would create the
  drift `PHIL-002` warns about. Exactly one new page is proposed (a tutorial).
- **No playground entry for the recovery sample.** Known Limitation 5 — the
  wasm ABI does not expose the AST or diagnostics — makes it unrunnable in the
  browser. See Risk RK-04.
- **Not closing any Known Limitation.** Publishing a limitation is not fixing it.
- **`docs/launch/` and `docs/legacy/`** are out of scope: `mkdocs.yml`
  `not_in_nav` excludes both from the published site.

## Proposed User Journey

```text
[author] -> `capy run lib.capy script.capy`
          -> error: expected `b`, `i`, `quote`, or `url`, found "/" in `bold`
          -> docs/errors-and-debugging.md
                 |
                 +-- "Anatomy of a Capy error"         (unchanged)
                 +-- NEW "One error, or all of them?"
                        |
                        +--> `capy ast lib.capy script.capy`
                        |      <error> 1:1-1:55  9 token(s) skipped
                        |      error[E0001] 1:1: expected `b`, `i`, …
                        +--> docs/diagnostics.md   (codes, labels, cascade)
                        +--> docs/ast-json.md      (--json, for tooling)
                        +--> samples/parse-recovery/   <- runnable, golden-verified
                                            |
                                            +-> fix -> `capy run` -> output
```

```text
[tool author] -> syntax-cheat-sheet.md CLI table  --(new row)--> cli.md `capy ast`
                                                                     |
              editor-tooling.md "CLI helpers"  --(new section)-------+--> ast-json.md
                                                                     |
              CAPY_FOR_LLMS.md CLI quick ref   --(new line)----------+

[agent]      -> emits source -> `capy ast --json` -> reads diagnostics[] -> repairs -> re-emits
                (documented once in docs/ai-agents.md, the repair-loop pattern)
```

## Requirements

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal IDs |
|---|---|---|---|---|---|
| R1 | `docs/errors-and-debugging.md` explains that `capy run` stops at the first error while `Library::parse` / `capy ast` recover, and links `diagnostics.md` | docs | P-01 | New section present; both links resolve under `mkdocs build --strict` | G-01 |
| R2 | `docs/troubleshooting.md` §"My script fails to parse" opens with `capy ast` as the diagnosis step | docs | P-01 | Step 0 added ahead of the four existing causes | G-01 |
| R3 | `capy ast` appears in `syntax-cheat-sheet.md`, `features.md`, `CAPY_FOR_LLMS.md`, `editor-tooling.md` | docs | P-02 | `grep -l "capy ast" docs/**/*.md` lists ≥ 8 pages | G-02 |
| R4 | Operator precedence is summarized in `syntax-cheat-sheet.md`, `features.md`, `CAPY_FOR_LLMS.md` and answered in `faq.md`, each linking `language-reference.md#operator-precedence` | docs | P-03 | 4 pages updated; anchor resolves | G-03 |
| R5 | Every error or CLI transcript quoted in a changed page is reproduced verbatim from `rust/target/release/capy` at `v0.22.0` | docs | P-07, `PHIL-001` | Each block has a recorded producing command in the plan's evidence log | G-05 |
| R6 | `docs/roadmap.md` "Already shipped" lists diagnostics, recovery, `capy ast`, precedence, each with its version and page link | docs | P-05 | 4 bullets added; no near-term bullet implies them | G-05 |
| R7 | Each of the five `REL-0.22.0` Known Limitations is published on a user-facing page or recorded as deferred with its reason | docs | P-08 | 5/5 rows resolved in *Requirements Alignment* | G-07 |
| R8 | A new `samples/operator-precedence/` demonstrates `* / % + -`, comparison, `and`/`or` and grouping, with a success golden | samples | P-04 | `cargo test --test golden` passes with the new case | G-04 |
| R9 | A new `samples/parse-recovery/` shows a clean script (success golden), a broken script (error golden) and the recovered tree (AST golden) | samples | P-04 | 3 golden files, all compared by the harness | G-04 |
| R10 | The new samples use `lib.capy` as the library filename | samples | `rust/tests/golden.rs:45` skips any directory without `lib.capy` | Discovery picks both directories up | G-04 |
| R11 | Every broken script in a new sample ships a golden file | samples | `golden.rs:137` counts a golden-less script as `skip`, silently | `skip` count does not increase | G-04 |
| R12 | `rust/tests/golden.rs` compares `<base>.expected-ast.txt` via `Library::parse` when the file exists | test | R9 needs a verifier | New branch; suite still asserts `pass >= 100` | G-04 |
| R13 | `samples/README.md` states the real directory count, `lib.capy`, the `cargo test` command, and all three golden kinds | samples | P-06 | Every fact re-derived from the tree at edit time | G-06 |
| R14 | Both new samples are listed in `samples/README.md` under a "Parser surface" section | samples | P-06 | Section present with 2 rows | G-06 |
| R15 | `docs/showcase.md` "🩺 Errors" gains a recovery tab and its three existing transcripts are re-verified | docs | P-07 | 4th tab present; transcripts reproduced per R5 | G-05, G-01 |
| R16 | A new tutorial `docs/tutorials/05-reading-diagnostics.md` walks break → `capy ast` → read codes → fix, using `samples/parse-recovery/` | docs | P-01, P-04 | Page added to `mkdocs.yml` nav under Learn | G-01 |
| R17 | `docs/ai-agents.md` documents the `capy ast --json` repair loop | docs | P-02 | Pattern section added; references `ast-json.md` | G-02 |
| R18 | `mkdocs build --strict` exits 0 and `docs/whats-new.md` records the documentation additions | release | `GATE-001` | Exit 0; a "Docs" entry under 0.22.0 | G-05 |
| R19 | `samples/parse-recovery/` is **not** added to `rust/playground/src/curated.rs` | samples | Known Limitation 5 | Absent from the curated list; reason noted in its README | G-07 |

## Use Cases

### Use-Case Catalogue

| ID | User Outcome | Actor | Surface and Trigger | Preconditions / Environment | Inputs | Outputs / Visible Result | Negative and Error Paths | Goal IDs | Requirement IDs | Test IDs |
|---|---|---|---|---|---|---|---|---|---|---|
| UC-01 | Sees every error in a file, not just the first | script author | CLI + docs; `capy run` failed | 0.22.0 binary | broken `script.capy` | tree + all diagnostics | script is clean → exit 0, no diagnostics | G-01 | R1, R2, R9, R16 | T-01, T-04 |
| UC-02 | Feeds the tree into a tool | tool author | `capy ast --json`, from a CLI summary | 0.22.0 binary | lib + script | one JSON document on stdout | diagnostics present → exit 1, JSON still emitted | G-02 | R3, R17 | T-02 |
| UC-03 | Writes `x = 1 + 2 * 3` and gets `7` | library author | cheat sheet → language reference → sample | 0.22.0 binary | `samples/operator-precedence/` | golden output | grouping a bare ident → still a call (documented) | G-03 | R4, R8 | T-03 |
| UC-04 | An agent repairs its own output | LLM agent | `capy ast --json` in a loop | MCP or CLI | generated source | diagnostics array | no diagnostics → accept output | G-02 | R3, R17 | T-02 |
| UC-05 | Adds a sample that actually asserts something | contributor | `samples/README.md` | repo checkout | new sample dir | passing golden case | golden file omitted → case silently skipped (called out) | G-06 | R11, R13, R14 | T-05 |

### UC-01 — A script author sees every error

1. `capy run lib.capy script.capy` fails with one error and no output.
2. The author opens `errors-and-debugging.md` (linked from the CLI help).
3. The new section states the contract: **`run` renders or fails; `ast` always
   returns a tree.**
4. The author runs `capy ast` and sees the error regions and codes.
5. The author fixes the first region and repeats until the tree is clean.

#### CLI Contract — verified against `rust/target/release/capy` at `v0.22.0`

```text
$ capy run samples/bbcode-parser/lib.capy samples/bbcode-parser/mismatch.capy
error: expected `b`, `i`, `quote`, or `url`, found "/" in `bold`
  1 │ [b]"opened bold but closed with the wrong tag"[/quote]
    │                                                ^
$ echo $?
1

$ capy ast samples/bbcode-parser/lib.capy samples/bbcode-parser/mismatch.capy
<error> 1:1-1:55  9 token(s) skipped
error[E0001] 1:1: expected `b`, `i`, `quote`, or `url`, found "/" in `bold`
```

This pair is the exact contrast the new section is built around: same input,
one command refuses, the other reports. Diagnostics go to stderr in tree mode,
so `stdout` stays pipeable.

### UC-02 — A tool author pipes the tree

#### CLI Contract

```text
$ capy ast lib.capy script.capy --json | jq '.diagnostics[].code'
"E0001"
$ echo $?
1                      # 1 = diagnostics were produced; JSON is still on stdout
```

Documented once in `ast-json.md`; the change here is only that four summaries
now point at it.

### UC-03 — A library author uses precedence

#### CLI Contract

```text
$ capy run samples/operator-precedence/lib.capy samples/operator-precedence/script.capy
… golden output showing (a * b) + c and (a + 1) == (b * 2) evaluation …
$ echo $?
0
```

### UC-05 — A contributor adds a sample

```text
+------------------- samples/<name>/ -------------------+
|  lib.capy            <- REQUIRED, exact name          |
|  script.capy         --+                              |
|  script.expected.txt   |  success golden              |
|  broken.capy         --+                              |
|  broken.expected-error.txt   error golden             |
|  broken.expected-ast.txt     AST golden (proposed)    |
|  README.md                                            |
+-------------------------------------------------------+
        |
        +-- no golden file?  -> harness SKIPS it silently (golden.rs:137)
        +-- lib named lib_x.capy? -> directory not discovered at all (golden.rs:45)
```

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 | 2026-09-16 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 (zero default grammar) | Applies | No grammar is added; both new samples define their own `lib.capy` | PASS | — |
| GOAL-002 (engine changes additive) | Applies | No `rust/src/` change; the only Rust edit is a new branch in `rust/tests/golden.rs` | PASS | — |
| PHIL-001 (verify before recording) | Applies | R5 requires every quoted transcript to be reproduced from the tagged binary; the two transcripts in UC-01 already were | PASS | — |
| PHIL-002 (record deviations) | Applies | P-06 (stale `samples/README.md`) and Known Limitation exposure (R7) are recorded rather than quietly fixed | PASS | — |
| CODE-001 (commit only when asked) | Applies | The implementing plan stages files by name | PASS | — |
| CODE-002 (helper list in sync) | Not applicable | No helper added or removed | NOT APPLICABLE | — |
| CODE-003 (keyword list in sync) | Not applicable | No directive or capture type added | NOT APPLICABLE | — |
| ARCH-001 (public field never changes type) | Not applicable | No public type touched | NOT APPLICABLE | — |
| ARCH-002 (`capy-core` keeps one dependency) | Applies | No dependency added; the harness change is in a dev target | PASS | — |
| ARCH-003 (engine never aborts its host) | Not applicable | No engine change | NOT APPLICABLE | — |
| QUAL-001 (regression across the corpus) | Applies | R8–R12 add corpus coverage where there is none; the full golden suite runs unchanged | PASS | — |
| QUAL-002 (measurable claim predeclared) | Applies | The only measurable claims are doc-coverage counts, predeclared in *Measurement and Validation* | PASS | — |
| QUAL-003 (a test that cannot fail is not a test) | Applies | R11 exists precisely because a golden-less sample is a test that cannot fail; R12 makes the AST transcript assertive instead of prose | PASS | — |
| GATE-001 (pre-commit gate) | Applies | `cargo build`, `clippy --all-targets`, `cargo test --workspace`, `mkdocs build --strict` — R18 | PASS | — |
| GATE-002 (regression gate) | Applies | Golden corpus must stay at 117 passing plus the new cases; no existing golden regenerated | PASS | — |

## Implementation Design

### Environment and Feasibility

| Environment / Version | Required Capability | Feasibility Evidence and Source | Constraint or Unknown | Resolution |
|---|---|---|---|---|
| Local dev, 0.22.0 | `capy ast` exists and recovers | Run recorded in UC-01 against `rust/target/release/capy` | — | proven |
| `rust/tests/golden.rs` | Discovers `samples/*/lib.capy` + every other `*.capy`; compares `.expected.txt` / `.expected-error.txt`; calls `run::run` only | Read at `golden.rs:33-60, 88-140` | No AST/diagnostic golden kind exists | Option B adds one branch |
| `mkdocs.yml` | Nav must gain the tutorial page | `nav:` → Learn block; `not_in_nav` covers only `/launch/*` and `/legacy/*` | A page absent from nav fails `--strict` | R16 adds the nav row |
| `rust/playground/src/curated.rs` | Curated playground list; JSON is gitignored and CI-rebuilt | `AGENTS.md` "Repo conventions" | wasm ABI exposes neither AST nor diagnostics (Known Limitation 5) | R19: precedence sample may be curated; recovery sample may not |

### Methods by Use Case

| Change ID | UC IDs | Method and Execution Order | Positive Path | Negative / Failure Path | Requirement Fulfilment | Feasibility |
|---|---|---|---|---|---|---|
| C-01 | UC-01 | Add "One error, or all of them?" to `errors-and-debugging.md`; add step 0 to `troubleshooting.md` | Reader reaches `diagnostics.md` in one hop | Wrong link → `--strict` fails | R1, R2 | proven |
| C-02 | UC-02, UC-04 | Add `capy ast` rows/lines to the four CLI summaries; add the repair-loop pattern to `ai-agents.md`; add an "AST as the tooling backbone" note to `editor-tooling.md` | 8 pages mention it | — | R3, R17 | proven |
| C-03 | UC-03 | Add a precedence block to the cheat sheet, a bullet to `features.md`, a grammar line to the LLM brief, an FAQ answer — all linking the canonical table | One canonical table, four pointers | Duplicating the table invites drift → pointers only | R4 | proven |
| C-04 | UC-03 | Create `samples/operator-precedence/` (`lib.capy`, `script.capy`, golden, README); optionally curate into the playground | Golden passes | — | R8, R10 | proven |
| C-05 | UC-01, UC-05 | Create `samples/parse-recovery/` with clean + broken scripts and three goldens | 3 cases compared | Missing golden → silent skip (R11) | R9, R10, R11, R19 | proven |
| C-06 | UC-01, UC-05 | Extend `golden.rs` with an `<base>.expected-ast.txt` branch calling `Library::parse` | AST transcripts asserted | Harness change rejected → fall back to Alternative A | R12 | proven (test-only) |
| C-07 | UC-05 | Rewrite `samples/README.md` header, goldens section and add-a-sample recipe; add "Parser surface" rows | Counts re-derived at edit time | Hard-coded counts re-rot → state how to re-derive | R13, R14 | proven |
| C-08 | UC-01 | Add `docs/tutorials/05-reading-diagnostics.md` + nav row | Learn tab gains a 5th tutorial | — | R16 | proven |
| C-09 | UC-01 | Re-verify `showcase.md` transcripts; add the recovery tab | Quotes match the binary | A quote diverges → update it and note it in `whats-new.md` | R15, R5 | proven |
| C-10 | all | Roadmap "Already shipped" rows; Known-Limitation notes; `whats-new.md` docs entry | Published status matches the release | — | R6, R7, R18 | proven |

### Added, Changed and Removed Contracts

| Item | CRUD | Kind | Name / Key / Route / Event | Type, Default or Schema | Scope / Lifetime | Consumers | Requirements |
|---|---|---|---|---|---|---|---|
| AST golden | CREATE | file convention | `<base>.expected-ast.txt` | UTF-8 text; `capy ast` tree-mode stdout + diagnostics, normalized (BOM/CRLF) | `samples/` corpus | `golden.rs`, contributors | R9, R12, R13 |
| Sample dir | CREATE | sample | `samples/operator-precedence/` | `lib.capy` + `script.capy` + `script.expected.txt` + `README.md` | permanent | golden suite, docs, playground | R8 |
| Sample dir | CREATE | sample | `samples/parse-recovery/` | `lib.capy` + `script.capy`(+golden) + `broken.capy`(+error and AST goldens) + `README.md` | permanent | golden suite, tutorial 5 | R9 |
| Docs page | CREATE | page | `docs/tutorials/05-reading-diagnostics.md` | mkdocs page, nav: Learn | permanent | site readers | R16 |
| Nav entry | UPDATE | nav | `mkdocs.yml` → Learn | one row | permanent | site | R16 |
| Docs section | CREATE | section | `errors-and-debugging.md#one-error-or-all-of-them` | anchor | permanent | CLI help, tutorial, showcase | R1 |
| Curated playground entry | CREATE | playground | `operator-precedence` in `curated.rs` | `{id, category: "Features", title, description, hint, library, script}` | permanent | playground | R8 (optional) |
| Curated playground entry | — | playground | `parse-recovery` | **deliberately not added** | — | — | R19 |

### Architecture, Data, State and Interaction Visuals

Documentation graph after the change — arrows are links a reader can follow:

```text
                             ┌──────────────────────┐
  CLI error text ───────────▶│ errors-and-debugging │  (UPDATE: C-01)
                             └───────┬──────────────┘
                                     │ new section
              ┌──────────────────────┼───────────────────────┐
              ▼                      ▼                       ▼
      ┌───────────────┐      ┌──────────────┐        ┌───────────────┐
      │ diagnostics.md│      │   cli.md     │        │ tutorials/05  │ (NEW: C-08)
      │  (unchanged)  │      │ `capy ast`   │        │ walk-through  │
      └───────┬───────┘      └──────┬───────┘        └───────┬───────┘
              │                     │                        │
              ▼                     ▼                        ▼
      ┌───────────────┐      ┌──────────────┐        ┌────────────────────┐
      │  ast-json.md  │◀─────│ editor-tool. │        │ samples/parse-     │ (NEW: C-05)
      │  (unchanged)  │      │ ai-agents.md │        │ recovery/          │
      └───────────────┘      └──────────────┘        └────────────────────┘
           ▲   ▲                (UPDATE: C-02)
           │   └───────────────── CAPY_FOR_LLMS.md, features.md, cheat sheet
           └───────────────────── troubleshooting.md  (UPDATE: C-01)

  language-reference.md#operator-precedence  (unchanged, canonical)
           ▲        ▲        ▲        ▲
           │        │        │        │                       (UPDATE: C-03)
     cheat sheet  features  faq   CAPY_FOR_LLMS ──▶ samples/operator-precedence/ (NEW: C-04)
```

Golden-harness state machine, before and after C-06:

```text
  BEFORE                                     AFTER
  for each (lib.capy, *.capy):               for each (lib.capy, *.capy):
    ┌────────────────────────┐                 ┌──────────────────────────────┐
    │ .expected-error.txt?   │─yes─▶ compare   │ .expected-ast.txt?  │─yes─▶ Library::parse
    ├────────────────────────┤       run() err ├──────────────────────────────┤  → compare tree
    │ .expected.txt?         │─yes─▶ compare   │ .expected-error.txt? │─yes─▶ compare run() err
    ├────────────────────────┤       run() ok  ├──────────────────────────────┤
    │ neither                │─────▶ SKIP (!)  │ .expected.txt?       │─yes─▶ compare run() ok
    └────────────────────────┘                 │ neither              │─────▶ SKIP
                                               └──────────────────────────────┘
       assert pass >= 100                         assert pass >= 100  (unchanged guard)
```

A script may carry both an `.expected-error.txt` and an `.expected-ast.txt`:
they assert different commands on the same input, which is the whole point of
`samples/parse-recovery/broken.capy`.

## Expected Code and Documentation Changes

| ID | Path | CRUD | Symbol / Region | Exact Planned Edit | Reason | Change IDs | Requirement IDs | Dependencies | Test / Documentation Impact |
|---|---|---|---|---|---|---|---|---|---|
| F-01 | `docs/errors-and-debugging.md` | UPDATE | after "Anatomy of a Capy error" (line ~28) | Insert `## One error, or all of them?` with the verified `run`/`ast` contrast from UC-01, plus links to `diagnostics.md`, `ast-json.md`, `samples/parse-recovery/` | P-01 | C-01 | R1, R5 | F-14 | T-01, T-06 |
| F-02 | `docs/errors-and-debugging.md` | UPDATE | `## no library function matches on deeply nested input` (line 232) | Add the diagnostic code and cross-link `diagnostics.md#diagnostic-codes`; keep the honest "reports the generic message" wording (Known Limitation 2) | P-08 | C-01, C-10 | R7 | — | T-06 |
| F-03 | `docs/troubleshooting.md` | UPDATE | `## My script fails to parse` (line 29) | Add a step 0: "Run `capy ast` first — it shows every failure, not just the first", before the four existing causes | P-01 | C-01 | R2 | — | T-06 |
| F-04 | `docs/syntax-cheat-sheet.md` | UPDATE | `## CLI quick reference` table (line 156) | Add row: `` `capy ast <lib> <script> [--json]` `` → "Print the parse tree; recovers, so a broken file still reports." | P-02 | C-02 | R3 | — | T-06 |
| F-05 | `docs/syntax-cheat-sheet.md` | UPDATE | after "Inner DSL" (line ~106) | Add a 6-line precedence ladder (`or` → `and` → comparison → `+ -` → `* / %`, `not` tightest, left-associative) linking the canonical table | P-03 | C-03 | R4 | — | T-06 |
| F-06 | `docs/features.md` | UPDATE | `## CLI` (line 270) and `## Lexical features` (line 100) | Add a `capy ast` entry and a precedence entry, each one line, each linking out | P-02, P-03 | C-02, C-03 | R3, R4 | — | T-06 |
| F-07 | `docs/CAPY_FOR_LLMS.md` | UPDATE | `## CLI quick reference` (line 436) and `### Expressions` (line 236) | Add `capy ast <lib.capy> <script.capy> [--json]  # parse tree + diagnostics` and one precedence sentence. Keep the brief single-page | P-02, P-03 | C-02, C-03 | R3, R4 | — | T-02, T-06 |
| F-08 | `docs/editor-tooling.md` | UPDATE | `## CLI helpers for tooling` (line 90) | Add "`capy ast --json` is the tooling backbone" with a one-command example and a link to `ast-json.md` | P-02 | C-02 | R3 | — | T-02 |
| F-09 | `docs/ai-agents.md` | UPDATE | after `### Pattern D` (line 199) | Add `### Pattern E: "Emit, inspect, repair"` — the `capy ast --json` → `diagnostics[]` → re-emit loop, with exit-code semantics | P-02 | C-02 | R17 | — | T-02 |
| F-10 | `docs/faq.md` | UPDATE | after `## How do I define x = 1?` (line 60) | Add `## Does Capy stop at the first error?` and `## Can I get the parse tree?`; extend the `x = 1` answer with the precedence link | P-01, P-02, P-03 | C-01, C-02, C-03 | R1, R3, R4 | — | T-06 |
| F-11 | `docs/roadmap.md` | UPDATE | `## Already shipped` (line 8) | Add 4 bullets: actionable diagnostics with stable codes, error recovery, `capy ast [--json]`, infix precedence — each `(0.22.0)` with its page link | P-05 | C-10 | R6 | — | T-06 |
| F-12 | `docs/roadmap.md` | UPDATE | `## Near-term` (line 36) | Add byte offsets in `Span`, the depth-limit message, and AST/diagnostics over the wasm ABI (Known Limitations 1, 2, 5) as explicitly-planned items | P-08 | C-10 | R7 | — | T-06 |
| F-13 | `docs/showcase.md` | UPDATE | `## 🩺 Errors that tell you how to fix them` (line 902) | Re-verify the three transcripts against the 0.22.0 binary; add a 4th tab "Parsing that keeps going" with the UC-01 pair | P-07 | C-09 | R15, R5 | F-14 | T-06 |
| F-14 | `docs/tutorials/05-reading-diagnostics.md` | CREATE | whole file | New tutorial: break a script → `capy ast` → read `E0001` and the labels → fix → clean tree. Built on `samples/parse-recovery/` | P-01 | C-08 | R16 | F-16 | T-04, T-06 |
| F-15 | `mkdocs.yml` | UPDATE | `nav:` → Learn | Add `- "Tutorial 5 · Reading diagnostics": tutorials/05-reading-diagnostics.md` after tutorial 4 | R16 | C-08 | R16, R18 | F-14 | T-06 |
| F-16 | `samples/parse-recovery/` | CREATE | directory | `lib.capy`, `script.capy` + `script.expected.txt`, `broken.capy` + `broken.expected-error.txt` + `broken.expected-ast.txt`, `README.md` (README states why it is not in the playground) | P-04 | C-05 | R9, R10, R11, R19 | F-18 | T-01, T-04 |
| F-17 | `samples/operator-precedence/` | CREATE | directory | `lib.capy`, `script.capy`, `script.expected.txt`, `README.md` | P-04 | C-04 | R8, R10 | — | T-03 |
| F-18 | `rust/tests/golden.rs` | UPDATE | `golden_samples_match` (lines 88-140) | Add an `ast_path` branch before the error branch: on `<base>.expected-ast.txt`, call `Library::parse`, render tree + diagnostics the way `capy ast` does, `normalize`, compare, honour `CAPY_UPDATE_GOLDENS` | R9 needs a verifier | C-06 | R12 | — | T-01, T-05 |
| F-19 | `samples/README.md` | UPDATE | header (lines 1-20) | Replace "50 self-contained demos" / `lib.yaml` / "116 cases" / `go test ./...` with values re-derived at edit time, and state how to re-derive them | P-06 | C-07 | R13 | — | T-05 |
| F-20 | `samples/README.md` | UPDATE | `## How goldens work` (line 129) | Document all three golden kinds and the two silent-failure traps (no golden → skip; non-`lib.capy` library → not discovered) | P-06, P-04 | C-07 | R11, R13 | F-18 | T-05 |
| F-21 | `samples/README.md` | UPDATE | `## Adding a new sample` (line 143) | Correct to `lib.capy` + cargo commands; add the "pre-create an empty golden placeholder" step from `AGENTS.md` | P-06 | C-07 | R13 | — | T-05 |
| F-22 | `samples/README.md` | UPDATE | section list | Add `## Parser surface` with rows for the two new samples | P-04, P-06 | C-07 | R14 | F-16, F-17 | T-05 |
| F-23 | `docs/whats-new.md` | UPDATE | `## 0.22.0` (line 7) | Add a "Docs & samples" paragraph naming the new tutorial and the two samples | R18 | C-10 | R18 | all | T-06 |
| F-24 | `docs/diagnostics.md` | UPDATE | after `## Diagnostic codes` (line 53) | Add a short "Not yet" note: byte offsets absent from `Span`; cascade constants untuned; diagnostics not exposed over the wasm ABI (Known Limitations 1, 4, 5) | P-08 | C-10 | R7 | — | T-06 |
| F-25 | `docs/ast-json.md` | UPDATE | `## Stability` (line 15) | Note that the wasm ABI does not expose the AST, so `--json` is CLI/embedding only (Known Limitation 5) | P-08 | C-10 | R7, R19 | — | T-06 |
| F-26 | `rust/playground/src/curated.rs` | UPDATE | curated list | Add `operator-precedence` under category `Features`. **Do not** add `parse-recovery` | R8, R19 | C-04 | R8, R19 | F-17 | T-03 |
| F-27 | `program_docs/index/document-index.md` | UPDATE | `## All documents` | Add the row for this proposal and, later, its plan | `DOCUMENTATION.md` §17 | — | — | — | — |

**Counts.** 8 created (1 docs page, 2 sample directories comprising 11 files,
plus the index row), 18 updated, 0 deleted. Of the updates, 14 are under
`docs/`, 4 under `samples/`, 1 in `mkdocs.yml`, 1 in `rust/tests/golden.rs`,
1 in `rust/playground/src/curated.rs`.

### Proposed content for the created files (illustrative)

The snippets below define the **approved behaviour and shape**; they are
illustrative and do not authorize coding. Exact wording is settled in the plan.

**`samples/operator-precedence/lib.capy`** — shape:

```text
function assign
    arg capture var ident
    arg literal "="
    arg capture value any        # the whole expression as ONE value
    run:
        set context.vars[var] value
    write `${var} = ${value}`
end
```

The point the sample must make visible: before 0.22.0 an author needed three
functions or a non-associative comparison step for the same source; now one
`any` capture yields a tree with conventional precedence.

**`samples/parse-recovery/broken.capy`** — a file with **three** good
statements and **two** bad regions, so the golden proves the "3 statements,
2 errors" claim in `REL-0.22.0` U-02 from the sample corpus rather than from a
unit test alone.

**`rust/tests/golden.rs`** — the added branch, illustrative:

```rust
let ast_path = dir.join(format!("{}.expected-ast.txt", base));
if ast_path.exists() {
    let lib_src = std::fs::read_to_string(lib)?;
    let library = Library::new(&lib_src)?;
    let parsed = library.parse(&std::fs::read_to_string(script)?);
    let got = render_ast_like_cli(&parsed);          // same renderer `capy ast` uses
    compare_or_update(&ast_path, &got, update, &mut pass, &mut updated, &mut failures);
    continue;                                        // an AST golden is its own case
}
```

`render_ast_like_cli` must be the *same* code path the CLI prints, not a second
renderer — otherwise the golden asserts a formatter that no user ever sees
(`QUAL-003`). If that function is not reachable from the test crate, the plan
must record it as a discovery step and either expose it or fall back to
Alternative A.

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected | Requirement Impact |
|---|---|---|---|---|
| **A. Docs + samples only; AST transcript lives in the sample README, unverified** | Zero Rust change; smallest diff | The transcript rots exactly like `samples/README.md` did (P-06); `QUAL-003` — an assertion nothing checks | **Rejected** as the primary path, **retained as the fallback** if C-06 proves infeasible | R12 dropped; R9 weakened to 2 goldens |
| **B. Extend `golden.rs` with `.expected-ast.txt` (recommended)** | The 0.22.0 surface becomes corpus-covered; one convention, documented in one place; test-only | Adds a third golden kind contributors must learn | **Selected** — it is the only option that satisfies G-04 honestly | all |
| C. A separate `rust/tests/samples_ast.rs` suite | Leaves `golden.rs` untouched | Two discovery walkers to keep in sync; a sample could pass one and be skipped by the other | Rejected — duplicated discovery is the bigger long-term cost | R12 re-targeted |
| D. Write a brand-new `docs/parsing-errors.md` hub page | One obvious destination | `diagnostics.md` already is that page; a second hub guarantees drift | Rejected — link to the existing page instead (non-goal) | R1 re-targeted |
| E. Update only `docs/`, leave `samples/` alone | Half the work | Leaves P-04 entirely open; the release's flagship capability stays untested from the user's side | Rejected — UQ-01 names both folders | R8–R14 dropped |
| F. Add both new samples to the playground | Consistent with other samples | `capy ast` and diagnostics are not on the wasm ABI (Known Limitation 5) — the recovery sample would appear broken in the browser | Partially rejected: precedence yes, recovery no | R19 |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback Action | Owner |
|---|---|---|---|---|---|
| RK-01 | A new sample ships without a golden → `golden.rs:137` counts it as `skip`, suite still green | A sample that asserts nothing, presented as verified | R11 makes the golden mandatory; T-05 asserts the `skip` count does not rise | Delete the sample directory | Capy Engine |
| RK-02 | A new sample's library is named `lib_x.capy` → the directory is never discovered (`golden.rs:45`) | Silent non-coverage | R10 fixes the filename; T-05 asserts both directories appear in the case list | Rename to `lib.capy` | Capy Engine |
| RK-03 | `render_ast_like_cli` is not reachable from the test crate | C-06 blocked | Fall back to Alternative A, recording the deviation (`PHIL-002`) | Revert `golden.rs` | Capy Engine |
| RK-04 | The recovery sample is curated into the playground by habit | A browser demo that cannot work (Known Limitation 5) | R19 states the exclusion; the sample README says why | Remove from `curated.rs` | Capy Engine |
| RK-05 | A re-verified transcript differs from the published one | `showcase.md` / `errors-and-debugging.md` quotes change | Precedent exists: `REL-0.22.0` already recorded one such deliberate change. Review each diff, never bulk-regenerate | Restore the prior text and open a defect | Capy Engine |
| RK-06 | A new or moved link breaks `mkdocs build --strict` | CI docs job fails | R18 gates on exit 0 before commit | Fix the link | Capy Engine |
| RK-07 | Precedence is restated in four places and they diverge | Contradictory documentation | C-03 mandates pointers to one canonical table, never copies of it | Replace the copy with a link | Capy Engine |
| RK-08 | Hard-coded counts in `samples/README.md` rot again | P-06 recurs | F-19 records the command that re-derives each count | Re-derive | Capy Engine |

## Security Impact

None. No new execution surface, no new dependency, no host capability. One
caution carried forward: `docs/ast-json.md:145` already warns that a tree
echoes source text, so `--json` output can carry whatever the source carried.
The new tutorial (F-14) links that note rather than repeating it.

## Operational Impact

CI gains: the golden suite gains ≥ 3 cases (sub-millisecond each); the docs
job builds one more page. `mkdocs build --strict` remains the gate. No release
process change — this work is documentation and test coverage for an already
tagged release and does not require a new version.

## Compatibility Impact

None for users. For contributors, `<base>.expected-ast.txt` is a new,
**optional** file convention: every existing sample is unaffected because the
branch only activates when the file exists.

## Migration Requirements

None.

## Test and Validation Design

| ID | Type | UC / Requirement IDs | Scenario and Purpose | Environment / Data | Exact Procedure or Command | Expected Result | Test File / Evidence Destination |
|---|---|---|---|---|---|---|---|
| T-01 | integration | UC-01, R9, R11, R12 | The recovery sample's three goldens are compared, not skipped | 0.22.0 workspace | `cargo test --manifest-path rust/Cargo.toml --test golden` | `parse-recovery/script`, `/broken` (error) and `/broken` (ast) all pass; `skip` count unchanged from baseline | golden suite output, recorded in the plan |
| T-02 | manual | UC-02, UC-04, R3, R17 | Every documented `capy ast` invocation runs as written | built binary | Execute each command quoted in F-04, F-07, F-08, F-09 | Output matches the doc byte-for-byte; exit codes 0/1 as documented | evidence log |
| T-03 | integration | UC-03, R8 | Precedence sample renders its golden and runs in the playground | workspace + `cargo run -p capy-playground-bundle` | `cargo test --test golden`; regenerate the playground JSON | Golden passes; the sample appears in the bundle | golden suite, bundle output |
| T-04 | manual | UC-01, R16 | The tutorial is followable start to finish by someone who has not read `diagnostics.md` | clean checkout | Follow `docs/tutorials/05-reading-diagnostics.md` verbatim | Reader ends with a clean tree; every command succeeds as printed | evidence log |
| T-05 | regression | UC-05, R11, R13 | The corpus does not shrink and the README is true | workspace | `cargo test --test golden`; re-derive each count in `samples/README.md` | `pass` ≥ baseline + 3; every stated count matches the tree | golden suite output |
| T-06 | regression | R1–R7, R15, R18 | Documentation coverage and link integrity | workspace | `mkdocs build --strict`; run the coverage greps below | Exit 0; every predeclared count met | evidence log |

### Measurement and Validation

The only measurable claims are documentation-coverage counts. Predeclared per
`QUAL-002`:

| Measurement | Baseline Method | Test Command or Procedure | Controlled Environment | Acceptance Threshold | Report Destination |
|---|---|---|---|---|---|
| Pages mentioning `capy ast` | `grep -rl "capy ast" docs --include=*.md \| wc -l` → **4** at 2026-09-16 | same command after the change | repo at the implementing commit | **≥ 8** | plan evidence log, then RPT |
| Pages mentioning precedence (excluding `launch/`, `legacy/`, `design/`) | **5** at 2026-09-16 — `language-reference.md`, `inner-dsl.md`, `editor-tooling.md`, `whats-new.md`, and `tutorials/04-custom-operators.md`, the last of which asserted the *opposite* (see D-06). The proposal's revision 1 recorded this baseline as 1, which was wrong. | same command | same | **≥ 5** | same |
| Error pages mentioning recovery | `grep -rcil recover docs/errors-and-debugging.md docs/troubleshooting.md` → **0, 0** | same command | same | **≥ 1 each** | same |
| Golden cases | `cargo test --test golden` `pass` count at baseline | same command | same | **baseline + 3**, `skip` unchanged | same |
| Known Limitations published | Manual audit: 1 of 5 covered (grouping caveat, `language-reference.md:123`) | Audit against F-02, F-11, F-12, F-24, F-25 | same | **5 of 5** published or explicitly deferred | same |
| `mkdocs build --strict` | exit 0 today | same command | same | exit 0 | same |

## Documentation, Demo and Release Impact

| Artifact | Exact Path or Destination | CRUD | Required Content / Verification | Owner | Release Gate |
|---|---|---|---|---|---|
| Manual / reference | `docs/diagnostics.md`, `docs/ast-json.md` | UPDATE | Known-Limitation notes only (F-24, F-25) | Capy Engine | `mkdocs --strict` |
| Tutorial | `docs/tutorials/05-reading-diagnostics.md` | CREATE | Followable end to end (T-04) | Capy Engine | `mkdocs --strict` + T-04 |
| CLI docs | `docs/cli.md` | READ | Already correct — the canonical target for F-04/F-06/F-07 links | Capy Engine | link check |
| Samples | `samples/operator-precedence/`, `samples/parse-recovery/` | CREATE | Goldens compared, not skipped (T-01, T-05) | Capy Engine | golden suite |
| Sample index | `samples/README.md` | UPDATE | Counts re-derived (T-05) | Capy Engine | T-05 |
| Demo | `program_docs/demos/demo-2026-0002-release-verification-0.22.0.md` | READ | Its verification steps are the source for F-01/F-13 transcripts | Release Management | PHIL-001 |
| Version source | `rust/Cargo.toml` | READ | Unchanged — no version bump | Release Management | — |
| Release | `docs/whats-new.md` 0.22.0 entry | UPDATE | Docs & samples paragraph (F-23) | Release Management | GATE-001 |
| Program docs | `program_docs/index/document-index.md` | UPDATE | Row for PROP-2026-0002 | Capy Engine | — |

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases / Internal Constraint | Changes | Files | Tests | Manual / Demo / Release Evidence |
|---|---|---|---|---|---|---|---|
| R1 | UQ-01 | G-01 | UC-01 | C-01 | F-01, F-10 | T-06 | DEMO-2026-0002 transcripts |
| R2 | UQ-01 | G-01 | UC-01 | C-01 | F-03 | T-06 | — |
| R3 | UQ-01 | G-02 | UC-02, UC-04 | C-02 | F-04, F-06, F-07, F-08 | T-02, T-06 | coverage grep |
| R4 | UQ-01 | G-03 | UC-03 | C-03 | F-05, F-06, F-07, F-10 | T-06 | coverage grep |
| R5 | UQ-01 | G-05 | PHIL-001 | C-09 | F-01, F-13 | T-02, T-06 | evidence log |
| R6 | UQ-01 | G-05 | UC-03 | C-10 | F-11 | T-06 | REL-0.22.0 "Added" |
| R7 | UQ-01 | G-07 | Known Limitations 1–5 | C-10 | F-02 (L2), F-12 (L1, L2, L5), F-24 (L1, L4, L5), F-25 (L5); L3 already published at `language-reference.md:123` | T-06 | REL-0.22.0 §Known Limitations |
| R8 | UQ-01 | G-04 | UC-03 | C-04 | F-17, F-26 | T-03 | golden suite |
| R9 | UQ-01 | G-04 | UC-01 | C-05 | F-16 | T-01 | golden suite |
| R10 | UQ-01 | G-04 | `golden.rs:45` | C-04, C-05 | F-16, F-17 | T-05 | case list |
| R11 | UQ-01 | G-04 | `golden.rs:137`, QUAL-003 | C-05 | F-16, F-20 | T-01, T-05 | skip count |
| R12 | UQ-01 | G-04 | R9 | C-06 | F-18 | T-01, T-05 | golden suite |
| R13 | UQ-01 | G-06 | UC-05 | C-07 | F-19, F-20, F-21 | T-05 | re-derived counts |
| R14 | UQ-01 | G-06 | UC-05 | C-07 | F-22 | T-05 | — |
| R15 | UQ-01 | G-05, G-01 | UC-01 | C-09 | F-13 | T-06 | evidence log |
| R16 | UQ-01 | G-01 | UC-01 | C-08 | F-14, F-15 | T-04, T-06 | mkdocs nav |
| R17 | UQ-01 | G-02 | UC-04 | C-02 | F-09 | T-02 | — |
| R18 | UQ-01 | G-05 | GATE-001 | C-10 | F-15, F-23 | T-06 | gate output |
| R19 | UQ-01 | G-07 | Known Limitation 5 | C-05 | F-16, F-25, F-26 | T-03 | curated list diff |

**Reverse check.** Every `C-NN` maps to at least one requirement: C-01→R1/R2,
C-02→R3/R17, C-03→R4, C-04→R8, C-05→R9/R10/R11/R19, C-06→R12, C-07→R13/R14,
C-08→R16, C-09→R15/R5, C-10→R6/R7/R18. Every `F-NN` maps to a change except
**F-27** (`document-index.md`), which is a `DOCUMENTATION.md` §17 obligation
rather than a requirement of this proposal — kept, and labelled as such.

## Plan Strategy and Estimated Work

One implementation plan is sufficient, executed in four ordered increments.
Ordering is forced by evidence flow: samples must exist and pass before the
docs may quote their output (`PHIL-001`).

```text
 INC-1  Samples + harness        INC-2  Error-path docs
 ┌───────────────────────────┐   ┌───────────────────────────┐
 │ F-16 parse-recovery/      │   │ F-01, F-02 errors-and-…   │
 │ F-17 operator-precedence/ │──▶│ F-03 troubleshooting      │──┐
 │ F-18 golden.rs branch     │   │ F-14, F-15 tutorial 5     │  │
 │ F-26 curated.rs           │   │ F-13 showcase tab         │  │
 │ gate: T-01, T-03, T-05    │   │ gate: T-04, T-06          │  │
 └───────────────────────────┘   └───────────────────────────┘  │
        goldens must be green ───────────▲                      │
        before any doc quotes them       │                      ▼
 ┌───────────────────────────┐   ┌───────────────────────────┐
 │ INC-4  Status + limits    │◀──│ INC-3  Reach & summaries  │
 │ F-11, F-12 roadmap        │   │ F-04…F-10 cheat sheet,    │
 │ F-24, F-25 limitation notes│  │ features, LLM brief,      │
 │ F-19…F-22 samples/README  │   │ editor-tooling, ai-agents,│
 │ F-23 whats-new, F-27 index│   │ faq                       │
 │ gate: T-06, GATE-001      │   │ gate: T-02, T-06          │
 └───────────────────────────┘   └───────────────────────────┘
```

| Increment | Files | Dominant effort | Blocking gate |
|---|---|---|---|
| INC-1 | F-16, F-17, F-18, F-26 | Writing two libraries whose failure modes are *legible*, not just failing | T-01, T-03, T-05 |
| INC-2 | F-01, F-02, F-03, F-13, F-14, F-15 | Prose; every transcript copied from INC-1 output | T-04, T-06 |
| INC-3 | F-04 … F-10 | Mechanical, but must link rather than copy (RK-07) | T-02, T-06 |
| INC-4 | F-11, F-12, F-19 … F-25, F-27 | Auditing 5 limitations and re-deriving 4 stale counts | T-06, GATE-001 |

If RK-03 materialises, INC-1 drops F-18, R12 is withdrawn, Alternative A is
recorded as the deviation, and INC-2 through INC-4 proceed unchanged.

## Open Questions

| ID | Question | Why It Matters | Proposed Default If Unanswered |
|---|---|---|---|
| OQ-01 | **RESOLVED — Option B, via a renderer moved into `capy-core`.** The tree printer was CLI-private in `rust/cli/src/cmd_ast.rs`, printing straight to stdout/stderr, and `rust/tests/golden.rs` is an integration test of `capy-core` and cannot reach `capy-cli`. Resolved as the default predicted: the renderer moved to the new `capy_core::domain::ast_text` module (`render_tree`, `render_diagnostics`, `render`), and `cmd_ast.rs` now calls it. One renderer, two callers; CLI output verified byte-identical before and after. | Decides B vs A | — |
| OQ-02 | Should `samples/parse-recovery/` also carry an `.expected-ast.json` for the `--json` shape? | `ast-json.md` is currently verified by unit tests only | Default **no** — one AST golden per sample; revisit if the JSON schema changes |
| OQ-03 | Does the precedence sample belong in the playground's `Features` category or a new `Parser` category? | Playground taxonomy | Default `Features`, consistent with the existing parser-flavoured entries |
| OQ-04 | Should `docs/getting-started.md` mention `capy ast` at all? | It is a 5-minute page; adding a fourth command may dilute it | Default **no** — out of scope, listed here so the omission is deliberate |
| OQ-05 | Is a version bump wanted for a docs-only change? | `samples/README.md` corrections are user-visible | Default **no** — fold into the next release's notes via F-23 |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | pending | | |
| Reviewer | Release Management | pending | | Confirms no version bump is implied (OQ-05) |

On approval, record an ADR only if Option B is contested; otherwise the
selection stands on this document, and the implementing plan is written against
the four increments above.

## Related Documents

- `REL-0.22.0` — `program_docs/releases/rel-0.22.0-release-notes.md` (driver)
- `PROP-2026-0001` — the capabilities being documented
- `PLAN-2026-0002`, `RPT-2026-0002`, `DEMO-2026-0002` — implementation, validation and verification of 0.22.0
- `ADR-0001` — the frozen contracts this proposal must not re-open
- `STD-2026-0000` rev 1 — the standards index validated against
- `AGENTS.md` — golden-harness, playground and pre-commit conventions cited throughout
- `DOCUMENTATION.md` §12.1 — the template this document follows

## Implementation Deviations

Recorded per `PHIL-002`. The proposal was implemented in full; these five points
differ from what it predicted.

| ID | Deviation | Why | Effect on scope |
|---|---|---|---|
| D-01 | A **new `capy-core` module** (`src/domain/ast_text.rs`) was added, against the "no `rust/src/` change" non-goal | OQ-01's resolution required it: the renderer was CLI-private and the golden test cannot reach `capy-cli`. Duplicating it would have asserted a format no user sees (`QUAL-003`). | Purely additive — a new module and three `pub fn`s, no existing type or field touched. `GOAL-002`, `ARCH-001`, `ARCH-002` unaffected. `cmd_ast.rs` output verified byte-identical before and after. |
| D-02 | Golden cases grew by **4**, not the predicted 3 (117 → 121) | `samples/parse-recovery/` ships a clean script as well as a broken one, so it contributes three cases, not two. | Exceeds the threshold; `skip` unchanged at 8. |
| D-03 | `docs/diagnostics.md`'s **code table was corrected**, beyond the limitation note F-24 planned | Verification found `E0002` and `E0003` are declared in `domain::errors::codes` but **never emitted** — only `E0001` is. The table presented all three as live. | The table now marks the two as reserved; `docs/errors-and-debugging.md` F-02 says the depth limit still arrives as `E0001`. An accuracy fix inside a file already in scope. |
| D-04 | Two **stale YAML snippets** in `docs/faq.md` were rewritten as `.capy` | F-10 edits both answers; leaving a `lib.yaml`-era snippet beside a corrected one would publish a contradiction. Both replacements were executed against the binary before publishing. | Confined to the two answers F-10 touches. The FAQ has no other YAML snippets; other pages were not audited — see Follow-Ups. |
| D-06 | **`docs/tutorials/04-custom-operators.md` was corrected**, a file the inventory did not list | It told readers "Capy doesn't have operator precedence — patterns are flat", which 0.22.0 made false. Found by the R4 coverage grep. Leaving a live contradiction while adding four pointers to the correct table would have defeated G-03. | One bullet rewritten; the tutorial's own subject (multi-token patterns for operators the engine does not know, like `\|>`) is unchanged and still correct. |
| D-05 | `samples/operator-precedence/` **omits** a `not (…)` grouping case | Building it surfaced a live defect (see Follow-Ups FU-01). A golden for it would have frozen wrong output. | The case is described in the sample README as a caveat instead. |

## Follow-Ups Found During Implementation

Neither is in this proposal's scope; both are recorded so they are not lost.

| ID | Finding | Evidence | Severity |
|---|---|---|---|
| FU-01 | **`expr_to_text` drops parentheses around `not` and nested comparison operands, breaking the source-text round-trip.** `Expr::Not` renders as `format!("not {}", …)` and `Expr::Compare` renders its children bare — neither applies the `wrap_if_looser` guard that `Expr::Binary` uses. So `not (1 == 2)` renders as the text `not 1 == 2`, which re-parses as `(not 1) == 2` — a different tree and, here, the opposite value. A library emitting `${cond}` into a target language emits wrong code, silently. | `rust/src/orchestrator/features/expr_to_text.rs:45` (`Not`) and `:25-27` (`Compare`). Reproduced: `let p = not (1 == 2)` evaluates `true` but renders `not 1 == 2`; `let r = 1 == (2 == 2)` renders `1 == 2 == 2`. `rust/tests/precedence.rs` has a T-27 round-trip test, but its corpus contains no `not` and no nested comparison, so the gap is untested rather than known-broken. | **Correctness.** Same class as the bug T-27 was written to prevent, and the one `Expr::Binary` already guards against. |
| FU-02 | **An unquoted date now lexes as arithmetic.** With infix `-` in 0.22.0, `due 2026-09-16` parses as a subtraction expression rather than three tokens, so a pattern-typed capture over a bare date no longer matches. The resulting error is also confusing: `no library function matches token "task"` / `hint: did you mean "task"?`. | Hit while writing `samples/parse-recovery/`; worked around by quoting the dates. | **Compatibility.** `REL-0.22.0` records the release as additive for all 117 sample libraries, which held — but a library using bare hyphenated tokens outside the corpus would break, and it is not in the upgrade notes. |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial proposal — docs and samples scope for the 0.22.0 parser surface |
| 2 | 2026-09-16 | Olivier | Implemented. OQ-01 resolved to Option B; deviations D-01…D-05 and follow-ups FU-01, FU-02 recorded |
