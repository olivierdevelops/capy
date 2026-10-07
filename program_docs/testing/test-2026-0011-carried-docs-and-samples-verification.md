---
document_id: TEST-2026-0011
title: Test — Carried Docs and Samples Verification (PROP-2026-0002 R1 to R19)
document_type: test
status: completed

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
  - docs
  - samples
  - capy-core
  - playground

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Verifies, against the working tree of 2026-10-07, each of the nineteen requirements of PROP-2026-0002 (public docs and samples) that this release carries in by owner instruction.

reason: PROP-2026-0002 was implemented but never committed or validated; it ships in 0.23.0, so DOCUMENTATION.md sections 27 and 28 require evidence per requirement before release.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0002
  - PROP-2026-0004
  - TEST-2026-0009
  - TEST-2026-0010
  - RPT-2026-0003
  - REL-0.22.0

supersedes: null
superseded_by: null

tags:
  - docs
  - samples
  - carried
  - verification

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Test — Carried Docs and Samples Verification (PROP-2026-0002 R1 to R19)

> **Status:** Completed
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** docs, samples, capy-core, playground

## Purpose

`PROP-2026-0002` (public docs and samples) was implemented in the working tree
before this release. This document checks each of its requirements R1 to R19 with
an exact command or grep and the real output, so the carried work enters 0.23.0
with evidence rather than on trust.

```text
   CARRIED WORK, AS IT ENTERS 0.23.0

   PROP-2026-0002 (docs + samples, written for 0.22.0)
           │   not committed at 0.22.0
           ▼
   working tree 2026-10-07 ──► PLAN-2026-0003 "Carried" row
           │
           ▼
   TEST-2026-0011 (this) ──► RPT-2026-0003 section "Carried PROP-2026-0002"
           │
           ▼
   REL-0.23.0
```

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0003 Carried (PROP-2026-0002 R1 to R19) | Docs pages, samples, golden-runner extension, README, tutorial, nav and playground exclusion described in `PROP-2026-0002` |

## Preconditions

- Working tree of 2026-10-07; `rust/target/debug/capy` built from it
  (`cd rust && cargo build --workspace`).
- Run from the repository root unless a command says otherwise.
- `PROP-2026-0002` R5 asks for transcripts from `rust/target/release/capy`; this
  run used the **debug** binary of the same source tree. The CLI text path is the
  same code in both profiles; the transcripts below are byte-compared.

## Test Environment

macOS arm64 (Darwin 25.4.0), `rustc 1.90.0`, `mkdocs` strict mode.

## Test Data

`docs/**`, `samples/**`, `mkdocs.yml`, `rust/tests/golden.rs`,
`rust/src/domain/ast_text.rs`, `rust/playground/src/curated.rs`.

## Procedure

For each requirement the command is listed in the table; `B=rust/target/debug/capy`.

## Expected Results

Every requirement is met as written in `PROP-2026-0002`; any shortfall is marked
`PARTIAL` or `FAIL` with its reason.

## Actual Results

| Req | Requirement (short) | Command or grep | Real output (abridged) | Result |
|---|---|---|---|---|
| R1 | `errors-and-debugging.md` explains `run` stops at first error while `Library::parse` / `capy ast` recover, links `diagnostics.md` | `grep -n "diagnostics.md\|Library::parse" docs/errors-and-debugging.md` | `:80 `docs/diagnostics.md` (Diagnostics)`, `:86 Embedding Capy? Library::parse returns the same ParseResult the CLI…`; both links resolve under `mkdocs build --strict` (R18) | PASS |
| R2 | `troubleshooting.md` "My script fails to parse" opens with `capy ast` | `grep -n "capy ast" docs/troubleshooting.md` | `:33 **Start here: capy ast shows every failure, not just the first.**`, `:36 capy ast lib.capy script.capy`, `:39 capy run stops at the first bad line…` | PASS |
| R3 | `capy ast` in cheat sheet, features, `CAPY_FOR_LLMS`, `editor-tooling`; at least 8 pages mention it | `grep -l "capy ast" docs/*.md docs/**/*.md \| sort -u \| wc -l`; `grep -l "capy ast" docs/syntax-cheat-sheet.md docs/features.md docs/CAPY_FOR_LLMS.md docs/editor-tooling.md` | `18` pages; all four named pages listed | PASS |
| R4 | Precedence summarised in cheat sheet, features, `CAPY_FOR_LLMS`, answered in `faq`, **each linking** `language-reference.md#operator-precedence` | `grep -c "language-reference.md#operator-precedence" docs/syntax-cheat-sheet.md docs/features.md docs/CAPY_FOR_LLMS.md docs/faq.md`; `grep -n "### Operator precedence" docs/language-reference.md` | cheat sheet `1`, features `1`, faq `1`, **`CAPY_FOR_LLMS.md` `0`** (it has the precedence summary at lines 242 to 247 but no link; `grep -c "language-reference" docs/CAPY_FOR_LLMS.md` is `0`); anchor exists at `language-reference.md:98` | PASS |
| R5 | Every error or CLI transcript in a changed page is verbatim from the binary | see *R5 transcript comparison* below | `parse-recovery` and `language-frontend` transcripts and the four showcase error tabs match; only `<-` annotations, `...` elisions and a blank separator differ | PASS |
| R6 | `roadmap.md` "Already shipped" lists diagnostics, recovery, `capy ast`, precedence with version and link | `grep -n "✅" docs/roadmap.md \| sed -n 1,8p` | `:35 Actionable parse diagnostics`, `:39 Error recovery`, `:42 capy ast <lib> <script> [--json]`, `:45 Infix operator precedence … (0.22.0)` with ``docs/language-reference.md#operator-precedence` (language reference)` | PASS |
| R7 | Each of the five `REL-0.22.0` Known Limitations is published or deferred with reason | per-limitation greps below | L1 byte offsets: `ast-json.md`, `diagnostics.md`, `embedding.md`; L2 depth message: `errors-and-debugging.md:302`; L3 grouping: `language-reference.md:123`; L4 cascade: `diagnostics.md`, `errors-and-debugging.md`, `language-frontend.md`; L5 wasm ABI: `ast-json.md:26`, `diagnostics.md`, `faq.md`, `language-frontend.md` | PASS |
| R8 | `samples/operator-precedence/` demonstrates `* / % + -`, comparison, `and`/`or`, grouping, with a success golden | `ls samples/operator-precedence`; `head -20 samples/operator-precedence/script.capy` | `README.md lib.capy script.capy script.expected.txt`; script has arithmetic, grouping `(1 + 2) * 3`, comparison and boolean sections; compared by the golden suite (131 passed, 0 failed) | PASS |
| R9 | `samples/parse-recovery/` has clean script (success golden), broken script (error golden), recovered tree (AST golden) | `ls samples/parse-recovery` | `README.md broken.capy broken.expected-ast.txt broken.expected-error.txt lib.capy script.capy script.expected.txt` (three goldens) | PASS |
| R10 | New samples use `lib.capy` | `head -3 samples/operator-precedence/lib.capy samples/parse-recovery/lib.capy` | both files exist and are discovered: suite count rose to 131 | PASS |
| R11 | Every broken script in a new sample ships a golden | `cd samples/parse-recovery; for f in *.capy; do …ls $b.expected*; done` and the golden skip count | `broken.expected-ast.txt broken.expected-error.txt`, `script.expected.txt`; `goldens: 131 passed, 8 skipped (no golden file), 0 failed`, skip count 8 unchanged from baseline | PASS |
| R12 | `golden.rs` compares `<base>.expected-ast.txt` via `Library::parse` when present; suite still asserts `pass >= 100` | `grep -n "expected-ast\|pass >= " rust/tests/golden.rs`; `ls rust/src/domain/ast_text.rs` | `:109 let ast_path = dir.join(format!("{}.expected-ast.txt", base))`; branch calls `ast_text::render(&library.parse(&src))`; `:191 assert!(pass >= 100, …)`; `ast_text.rs` is the renderer `capy ast` also uses | PASS |
| R13 | `samples/README.md` states the real directory count, `lib.capy`, the cargo test command, all three golden kinds | `ls -d samples/*/ \| wc -l`; `sed -n 1,30p samples/README.md`; `sed -n 155,165p samples/README.md` | `130` directories, README says "130 self-contained demos"; `lib.capy`; `cargo test --manifest-path rust/Cargo.toml --test golden   # 131 cases`; table of `.expected.txt`, `.expected-error.txt`, `.expected-ast.txt`; README also gives the commands to re-derive both counts | PASS |
| R14 | Both new samples listed under "Parser surface" | `grep -n "Parser surface" -A9 samples/README.md` | `:45 ## Parser surface` with rows for `operator-precedence/`, `parse-recovery/` (line 55), plus `language-frontend/` and, from 0.23.0, `expression-grammar/` | PASS |
| R15 | `showcase.md` Errors gains a recovery tab; three existing transcripts re-verified | `grep -n '=== "Parsing that keeps going"' docs/showcase.md`; scratch libraries for the three existing tabs | tab at `:1079`; the keyword-typo, enum and enum-with-hint transcripts (and the regex tab) reproduced byte-identical, see below | PASS |
| R16 | Tutorial `05-reading-diagnostics.md` added to nav under Learn | `grep -n "05-reading" mkdocs.yml`; `wc -l docs/tutorials/05-reading-diagnostics.md` | `mkdocs.yml:76 - "Tutorial 5 · Reading diagnostics": tutorials/05-reading-diagnostics.md`; page is 261 lines; its Step 1 `run` and `ast` outputs on `script.capy` match the binary | PASS |
| R17 | `ai-agents.md` documents the `capy ast --json` repair loop | `grep -n "Pattern E" -A6 docs/ai-agents.md`; `grep -c "ast-json.md" docs/ai-agents.md` | `:305 ### Pattern E: "Emit, inspect, repair"`; `ast-json.md` referenced (1 occurrence); example at `:328 capy ast lib.capy out.capy --json` | PASS |
| R18 | `mkdocs build --strict` exits 0; `whats-new.md` records the docs additions | `mkdocs build --strict >/dev/null 2>&1; echo $?`; `sed -n 60,75p docs/whats-new.md` | `mkdocs exit 0`; `:66 **Docs & samples.**` under `## 0.22.0` (`:38`) names both samples and Tutorial 5 | PASS |
| R19 | `samples/parse-recovery/` is **not** a Curated entry in `curated.rs`; reason in its README | `grep -n "parse-recovery" rust/playground/src/curated.rs`; `grep -n -i playground samples/parse-recovery/README.md` | `curated.rs:9` is a doc comment only (`/// Deliberately ABSENT: samples/parse-recovery/`); no `Curated { … }` entry; README `:83 ## Not in the playground — on purpose` explains the wasm ABI exposes neither AST nor diagnostics | PASS |

```text
   RESULT COUNT
   ┌────────────────┬─────┐
   │ PASS           │ 19  │
   │ PARTIAL        │  0  │
   │ FAIL           │  0  │
   │ NOT APPLICABLE │  0  │
   └────────────────┴─────┘
```

### R5 transcript comparison

```text
   transcript                                          docs page              vs binary
   ─────────────────────────────────────────────────   ────────────────────   ─────────
   capy run parse-recovery/broken.capy                 errors-and-debugging   identical
   capy ast parse-recovery/broken.capy                 errors-and-debugging   identical
   capy run language-frontend/broken.capy              language-frontend      identical
   capy ast language-frontend/broken.capy              language-frontend      identical*
   capy ast language-frontend/main.capy                language-frontend      identical*
   capy run parse-recovery/script.capy + ast           tutorial 05            identical
   keyword typo / enum / enum+hint / regex tabs        showcase               identical
```

\* The docs add `<- the NEXT function parsed fine` / `<- and so did this`
annotation comments, elide the body of recovered nodes with `...`, and show a
blank separator line between a `run` and an `ast` command. Those are the only
differences; no token of real output is altered.

Real output used for the comparison:

```text
$ rust/target/debug/capy run samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
error: expected `due`, found end of statement in `task`
  2 │ task "missing its due date"
(exit 1)

$ rust/target/debug/capy ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
task 1:1-1:43
  name = "\"write the proposal\"" 1:6
  when = "2026-09-16" 1:31
done 3:1-3:30
  name = "\"read the release notes\"" 3:6
list 5:1-5:5
<error> 2:1-2:28  3 token(s) skipped
<error> 4:1-4:29  3 token(s) skipped
error[E0001] 2:1: expected `due`, found end of statement in `task`
error[E0001] 4:1: no library function matches token "tsak"

$ rust/target/debug/capy run samples/language-frontend/lib.capy samples/language-frontend/broken.capy
error: expected `)`, found "{" in `fn`
  1 │ fn add(x: int, y: int {
    │                       ^
(exit 1)

$ rust/target/debug/capy ast samples/language-frontend/lib.capy samples/language-frontend/broken.capy
return 2:5-2:17
  value = "x + y" 2:12
fn 5:1-5:24
  name = "area" 5:4
  ...
while 10:1-10:17
  ...
<error> 1:1-1:24  12 token(s) skipped
<error> 3:1-3:2  1 token(s) skipped
error[E0001] 1:1: expected `)`, found "{" in `fn`
error[E0001] 3:1: no library function matches token "}"
```

(The `...` lines stand for the indented bodies that the real output prints in
full and the docs elide.) The showcase library fragments for the four non-sample
tabs were reconstructed minimally in a scratch directory (one `function` or
`type` each) and produced the quoted text verbatim.

### Note on `broken.expected-error.txt`

The golden file is in the harness format `LINE:COL: message` (`2: expected \`due\`…`),
which compares the library error, not the terminal rendering shown above; the
golden passes in the suite. The terminal text is what the docs quote and what the
R5 comparison checks.

## Result

PASS

All nineteen requirements are `PASS`. **R4 failed on first run:** `docs/CAPY_FOR_LLMS.md`
carried the precedence summary but not the link to
`language-reference.md#operator-precedence` that R4 requires of each page. The link was
added (`D-01`), `grep -c` now returns `1` on all four pages, and `mkdocs build --strict`
still exits 0.

## Evidence

- All commands above were executed on 2026-10-07 against the working tree.
- `mkdocs build --strict` exit 0 proves every internal link added by these pages
  resolves, including R1's links and R16's nav entry.
- `cargo test --test golden` (131 / 8 / 0) proves R8 to R12 mechanically: the new
  goldens are compared by the harness, not merely present.

## Evidence Sources

- `grep`, `ls`, `diff` commands in the table above
- `rust/target/debug/capy run|ast samples/parse-recovery/…` and `…/language-frontend/…`
- `cd rust && cargo test --test golden -- --nocapture`
- `mkdocs build --strict`
- `rust/tests/golden.rs`, `rust/src/domain/ast_text.rs`, `rust/playground/src/curated.rs`
- `docs/errors-and-debugging.md`, `docs/language-frontend.md`, `docs/showcase.md`, `docs/tutorials/05-reading-diagnostics.md`

## Executed By

Capy Engine (Olivier, with an AI agent running the commands)

## Executed At

2026-10-07

## Defects Raised

| Item | Description | Disposition |
|---|---|---|
| D-01 | R4: `docs/CAPY_FOR_LLMS.md` lacked the `language-reference.md#operator-precedence` link | **Resolved** 2026-10-07 — link added; re-verified |

## Related Documents

- PLAN-2026-0003
- PROP-2026-0002 — requirements R1 to R19
- PROP-2026-0004
- TEST-2026-0009 — alternation tests
- TEST-2026-0010 — measured results
- RPT-2026-0003 — validation report
- REL-0.22.0

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
