---
document_id: PROP-2026-0007
title: Parser and Pipeline Speed Experiments — Measure Candidate Speed-Ups and Three Architectures Before Building Any of Them
document_type: proposal
status: draft

created_date: 2026-10-08
last_updated: 2026-10-08
document_revision: 4

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
  - capy-wasm-abi
  - docs

affected_versions:
  from: "0.25.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Defines a controlled experiment programme that implements each candidate speed-up in an isolated sandbox copy of the 0.25.0 engine, assembles them into three complete architectures (optimised indexed parser, compiled progressive matching tree, fused scanner with compiled execution), measures each against a frozen baseline with frozen predictions, and ranks the candidates by measured gain. It adopts nothing; adoption is a separate decision per method. No grammar, output, diagnostic or public-type change.

reason: A comparison against regex on 2026-10-08 measured Capy at 19–30x slower on anchored line-shaped work and 940x slower on a regex-in-template case. A stage profile put 65 % of the time in the parser at 39 allocations per line. A list of eight ranked speed-up ideas exists but is unranked and unmeasured. The owner asked for a proposal that says how each would be tested and what each is expected to show. A review of the first draft asked for broader architectures to be tested too (an ungated progressive tree, a fused scanner, compiled execution, a memoised matcher) and for a wider benchmark matrix; revision 2 adds them. A second review found four specification gaps (ordering against unsupported candidates, exclusion proof for the scanner, late-fallback overhead, AST compatibility) and one adoption-rule flaw; revision 3 fixes them.

related_documents:
  - PROP-2026-0006
  - REL-0.24.0
  - TEST-2026-0010
  - TEST-2026-0013
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - performance
  - parser
  - lexer
  - evaluator
  - benchmark
  - experiment

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-08
---

# Parser and Pipeline Speed Experiments — Measure Candidate Speed-Ups and Three Architectures Before Building Any of Them

> **Status:** Draft
> **Created:** 2026-10-08
> **Last Updated:** 2026-10-08
> **Affected Versions:** 0.25.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi, docs

## Summary

```text
  WHERE THE TIME GOES TODAY            100 000 lines, anchored `import X`, release build
  (measured 2026-10-08, macOS arm64)   capy 0.24.0, in-process, outputs verified identical

     lex     ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░   63 ms   11 allocs/line
     parse   ██████████████████████████░░░░░░░░  228 ms   39 allocs/line   <── 65 %
     eval    ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░   58 ms    6 allocs/line
     ───────────────────────────────────────────
     total                                      ~350 ms   regex, same job:  11 ms   (≈ 30x)

  `regex_match` in a template:  3 386 ms vs 3.6 ms regex   (≈ 940x)  — Regex::new per call
```

Capy is slower than regex because it does more per line (tokenise, try ordered
candidates with backtracking, build a captured tree, render a template), and because
the current implementation does that work with a **lot of allocation and cloning**
(parse: 39 allocations per line; regex: ≈ 0). Eight ranked speed-up ideas have been proposed
(`.ignore/improvement_idea.md`). Nobody knows which are worth building, because
**none has been measured and several interact**.

This proposal does **not** build any of them into the engine. It defines:

1. a **sandbox** (`.ignore/speedlab/`, untracked, a clean export of commit `75a9f68`),
2. **eighteen experiments** (E0–E15, with E4 in three parts: fifteen candidate speed-ups, one correctness fix, one baseline, one measure-only spike), assembled into **three architectures** (A optimised indexed parser = the stack S1, B compiled progressive tree, C fused scanner with compiled execution), each experiment a small isolated
   patch with a hypothesis, step-by-step method, correctness gate, and a **predicted
   result that is frozen in this document before anything is measured**,
3. one **ranking rule** that turns the measurements into a recommendation, and
4. the exit: a `TEST` document with every sample, and an ADR per method worth adopting.

```text
  THE PROGRAMME

   E0  freeze baseline + noise floor ─────────────────────────────┐
   E8  fix punctuation-split token mutation (correctness control) │  TIER 0  controls
                                                                  │
   E7  cache compiled regex          E1  borrowed peek()          │
   E9  fuse comment filtering        E2  lazy failure path        │  TIER A  local edits,
   E3  no element clone / slot caps                               │          no new structures
                                                                  │
   E4a hoist candidate sort to load  E5  source-backed tokens     │
   E4b first-token dispatch index    E6  precompiled templates    │  TIER B  new structures
   E4c token-kind filter for capture-first                        │
                                                                  │
   E10 AST-share spike (measure only)                             │  TIER C  measurement
   E15 memoisation — only if E15's repetition gate is met         │
                                                                  │
   E11 deterministic compiled matcher / shared-prefix tree        │  TIER D  architectures
   E12 progressive tree with active states (ambiguity)            │          (ungated since rev 2)
   E13 fused scanner (bytes → matches, no Token vector)           │
   E14 specialised execution (compiled render, no FuncCall)       │
                                                                  │
   S1 = architecture A (stack the winners)                        │
   B  = foundation + E11 + E12      C = foundation + E13 + E14    │
   compare A / B / C on identical workloads ──────────────────────┘
```

| Question the owner asked | Answer this proposal gives |
|---|---|
| Which idea gives the biggest speed-up? | Ranked by measurement (§Ranking Rule), with the **predicted** ranking frozen below so a wrong prediction is visible |
| What do you expect to find? | Every experiment states a numeric prediction and its basis (§Frozen Predictions) |
| Is regex parity reachable? | Not on the general path: S1 (architecture A) is predicted at ≈ 12–15x. Architecture C measures how close a fused scanner gets on the subset it can safely handle (predicted ≈ 4–8x on eligible scripts); that is the least certain prediction here (§Expected Overall Result) |

## Decision Requested

Approve **running the experiment programme** E0–E14, the three architectures A/B/C and S1 in the sandbox, and approve the
**thresholds and predictions in §Frozen Predictions** as frozen at approval time (QUAL-002).

Approve **E15 only as a gated experiment**: it runs only if the repetition measurement in E15 meets its gate. (Revision 1 gated E11; revision 2 removes that gate — see E11.)

Approval does **not** authorize changing `rust/` in the repository tree, adopting any method,
adding a dependency (ARCH-002), changing a public type (ARCH-001), or changing any rendered output.
Adoption is a separate ADR per method, citing this proposal's resulting `TEST` document.

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "can this project be compared to regex ?" | User, this conversation, 2026-10-08 | Led to a capability test and a head-to-head benchmark |
| UQ-02 | "i am asking if it can do what regex does, if so we could run some benchmarks to do speed and perf comparsions on same target" | User, 2026-10-08 | Same-target comparison; delivered in `.ignore/regex-compare/` |
| UQ-03 | "why is it that much slower than regex" | User, 2026-10-08 | Delivered the stage/allocation profile quoted in the Summary |
| UQ-04 | "here are ideas on how to speed it up `.ignore/improvement_idea.md`" | User, 2026-10-08 | The idea list this proposal turns into experiments. Note: the file's own ranking table and its "implementation order" disagree (dispatch first vs allocation first); the experiments settle it by measurement |
| UQ-05 | "now i want you to do benchmarks for different implementations … you can write a proposal for different methods you want to test to see where we can get best speedups, describe them all fully step by step and what you expect to find, follow documentation.md" | User, 2026-10-08 | This document. "Do benchmarks" is read as: define them now with frozen expectations; **run them after approval**, because §21.3 freezes the expected result before measurement |
| UQ-06 | A pasted architecture review (user message, same session, 2026-10-08): consider better architectures than a trie plus token cursor; test a full progressive tree **without** the E11 gate; add a fused lexer+matcher, specialised execution and a memoised matcher; use a benchmark matrix (library sizes up to 10 000, prefix sharing, capture kinds, input shapes, valid/invalid/ambiguous); separate grammar-compilation time from execution time; require identical observable behaviour | User message with pasted review, 2026-10-08 | Treated as proposals to evaluate, not instructions. Accepted: ungated tree, matrix, compile-time split, fused scanner, compiled execution. Tempered by safety rules: a whole-script fallback and a differential test for anything that skips the lexer (R17); a repetition gate for memoisation (E15); adoption of any tree still follows shipped-library evidence (E4b rule) |
| UQ-07 | A second pasted review (user message, same session, 2026-10-08): approve the exploration but fix four issues before freezing — (1) E12 must evaluate an unsupported higher-priority candidate before accepting a lower-priority tree match; (2) E13 must prove unsupported definitions cannot match, or it may silently select the wrong function; (3) scan-then-restart on a late ineligible statement can cost far more than the 5 % target; (4) E14 must keep `capy ast --json` identical. Also: test every independent optimisation instead of stopping at the first one below threshold; add no more architectures | User message with pasted review, 2026-10-08 | All five points accepted; each checked against the text first. (1) and (2) are real specification gaps (revision 2 let a compiled match pre-empt an interpreted candidate). (3) is addressed by replacing the restart with a one-way prefix handoff, not by a bigger tolerance. (4) is made a requirement (R19). The adoption rule is changed (§Ranking Rule). No architecture is added |

## Problem and Evidence

### Current User Journey

```text
  maintainer ──► "Capy is 30x slower than regex on the same job"
       │
       ├─► sees eight ranked speed-up ideas, unranked, one of them contradicting the file's own order
       ├─► cannot tell which ideas overlap (E2/E3 touch the same lines) or interact (E4 shrinks
       │   as E2 lands, because a failed candidate becomes cheap)
       └─► the only safe option today is to build the biggest idea first, and learn afterwards
                 ── cost: weeks of work on a method that may buy 5 %
```

All timings: release build, macOS arm64 (Darwin 25.4.0), Rust `regex` 1.13.1, in-process
(`Library::run`), outputs byte-compared with a regex implementation of the same task. Harness and
raw output live in `.ignore/regex-compare/` (**untracked** — CODE-001; E0 re-measures and records
them in a tracked `TEST` document, because an untracked file is not evidence a reviewer can open).

| ID | Problem | Evidence | Consequence | UQ |
|---|---|---|---|---|
| P-01 | Capy is 19–30x slower than regex on anchored line-shaped work | 100 000 lines: `import` 342 ms vs 11.3 ms (30x); `say` 332 vs 11.0; `assign` 401 vs 13.5; all three in one library 471 vs 24.3 (19x). Outputs identical (`equal=true` in every row) | The cost of Capy's generality is large; users who only need extraction pay it | UQ-01, UQ-02 |
| P-02 | 65 % of the time is the parser | `phases` bench, 100 000 lines, `t_import.capy`: lex 63 ms, **parse 228 ms**, evaluate 58 ms; `t_all.capy`: 61 / **288** / 149 ms | A speed-up that does not touch the parser cannot exceed ≈ 2.9x end to end (Amdahl: 121 ms of lex+eval remains) | UQ-03 |
| P-03 | The parser allocates ≈ 39 times per line; the lexer 11; the evaluator 6 | counting global allocator in `phases`, same run | Allocation, not comparison, is the likely dominant cost | UQ-03 |
| P-04 | A failed candidate does real work | `make_parser.rs:956-1060` (`try_match`, `try_match_inner`): pushes a `ContextFrame` (owns a `String`), clones each `PatternElement`, builds a `BTreeMap`, calls `format!("expected …")`, clones the literal into `note_failure` | Every line pays for every candidate that does **not** match it | UQ-03 |
| P-05 | `peek()` clones a `Token` (which owns a `String`) on every call | `make_parser.rs:255-265` | Several clones per matched or failed element | UQ-03 |
| P-06 | `regex_match` recompiles its pattern on every call | `inner_evaluator.rs:823-832` (`Regex::new` inside the arm; no cache anywhere in the file) | `contains` task: 3 386 ms vs 3.6 ms (940x) | UQ-02 |
| P-07 | Template `${…}` bodies are re-scanned and re-parsed on every render | `inner_evaluator.rs:506` `interpolate_render` scans bytes per render and calls `eval_interp` (`:566`), documented as "Parses + evaluates the body of a single `${...}`" | Evaluate stage cost scales with template complexity, per statement | UQ-04 |
| P-08 | The candidate list is rebuilt and re-sorted on **every run** | `make_parser.rs:76` `lib.functions.values().cloned().map(Arc::new).collect()`, then a sort (`:77-98`); each `FuncDef` is deep-cloned | Negligible for 100 000 lines, **not** for a 15-line script or an editor keystroke; scales with library size F | UQ-04 |
| P-09 | A failed candidate can corrupt the token stream for later candidates | `make_parser.rs:1204-1240` `match_literal` rewrites `self.toks[self.pos]` when a punct literal is a prefix of a merged punct token; `restore` resets only `pos`. **Reproduced:** library `a` (priority 10: `">"` then `"zzz"`) and `b` (`"><"` then ident); script `>< foo` → `no library function matches token "><"`; with `a` removed, `b` matches and prints `B foo` (`.ignore/regex-compare/demos/d8*`) | A correctness defect, and a hazard for E2, E3, E5, E11, all of which touch the same cursor | UQ-04 |
| P-10 | There is no benchmark that varies **library size** | `nativebench` loads one library; `PROP-2026-0006` P-05 notes none varies input size either | The dispatch ideas (E4, E11) have no measurable baseline. Their value depends on F, which has never been measured | UQ-04 |

**Provenance and a variance warning (recorded, not smoothed over — PHIL-002).** The table above is the
first measurement session. A second session on `75a9f68` the same day reproduced the **stage** figures
(lex 59 ms, parse **221** ms, evaluate 60 ms, parse **39.2** allocations/line) but the **end-to-end**
`import` task read **401 ms** against 342 ms earlier (+17 %), and the regex ratio moved from 30x to 37x, on code whose
parser and lexer are byte-identical. So end-to-end timing on this host varies by well over the 3 % first assumed.
Consequences carried into this proposal: stage-level timings (in-process, repeated) are the **primary** measure;
the E0 spread prediction is widened (§Frozen Predictions); any claim below 2× the measured spread is `NOT SUPPORTED`
whatever its sign.

```text
  P-02 / P-03, one line through the pipeline (anchored `import mod0`, library of 2 functions)

   "import mod0"
        │ LEX      6 tokens, each token owns a String                       ≈ 11 allocs
        ▼
   [Ident][Ident][Newline] …
        │ PARSE    for each candidate in sorted list:
        │            push ContextFrame(String)  clone PatternElement(s)  BTreeMap::new
        │            peek() = Token clone       fail → format!() + note_failure(clone)
        │            restore pos, next candidate            ──► the failed candidates are pure cost
        ▼                                                                    ≈ 39 allocs
   FuncCall{ captures: BTreeMap<String,CaptureValue> }
        │ EVAL     re-scan template bytes, re-parse each ${…}, render       ≈ 6 allocs
        ▼
   "mod0\n"

   regex, same line:  one pass over bytes through a compiled automaton       ≈ 0 allocs
```

### What was NOT measured

- Anything on a library with more than 4 functions (P-10). The value of E4/E11 is **unknown**, not small.
- Where inside `parse` the 228 ms goes. The allocation counts say "allocation", not which allocation
  site. E0 closes this with a per-site attribution before any method is built.
- Memory use, wasm size and wasm speed of any method.
- Deeply nested blocks and ordered-choice (`PROP-2026-0004`) corpora: backtracking cost there is untested.
- The first measurements ran against a working tree holding uncommitted operator work (`Expr::Unary`, bitwise operators), committed afterwards as `75a9f68`. `make_parser.rs`, `make_lexer.rs` and `token.rs` are byte-identical between `ac3f7be` (0.24.0) and `75a9f68` (checked with `git diff --stat`, no output), so the parser and lexer figures carry over; the evaluator figures are re-measured by E0 on the clean export.

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-01 | Rank the candidate speed-ups by **measured** end-to-end and per-stage gain | P-01…P-08 | One harness, one corpus set, interleaved A/B runs, every sample reported | A ranked table in a `TEST` document |
| G-02 | Learn whether predictions were right, and where they were wrong | P-02, P-03 | Predictions frozen before measurement (§Frozen Predictions) | A prediction-vs-actual table with every miss explained |
| G-03 | Establish how cost scales with **library size** | P-10 | Synthetic libraries of 4, 10, 100, 1 000 functions | Per-F µs/line and the slope |
| G-04 | Prove no variant changes behaviour | P-09 | Byte-equality of output and `ast --json`, plus the golden corpus, for every variant | 0 differences, or each reviewed and recorded |
| G-05 | Decide the order in which to adopt, with evidence | all | The ranking rule, interaction measurement (S1) | One ADR per adopted method |
| G-06 | Replace the defect P-09 with a tested fix | P-09 | E8 | The reproduction passes; no golden changes |
| G-07 | Know how much of tokenisation, matching, AST construction and evaluation can be compiled away **safely**, by comparing three whole architectures | P-01, P-02, P-10 | Architectures A, B, C on identical workloads and identical observable behaviour (R17, R18) | One comparison table: throughput, latency, allocations, peak memory, grammar-compile time, eligibility coverage |

**Non-goals and boundaries**

- Adopting, merging or releasing any method. (Separate ADR per method.)
- Changing `rust/` in the repository tree. Experiments live in `.ignore/speedlab/`.
- Guaranteeing regex-class speed. The general path (architecture A, S1) is predicted to stop at ≈ 12–15x.
  Architecture C measures how close a fused scanner gets on the subset it can decide with proof, handing the
  rest of the script to the full parser. Recursive blocks, arbitrary captures and context-dependent evaluation stay on
  the general path.
- Any new dependency in `capy-core` (ARCH-002). The idea list's `SmallVec` is therefore **not** used;
  the equivalent is a reusable `Vec` or a fixed-size array in the matcher state.
- Changing the type of any public field (ARCH-001). `Token` (`pub text: String`) is public and reachable
  through `ErrorNode.tokens` (`ast.rs:19`), so E5 keeps `Token` unchanged and uses an **internal** token type.
- Multi-threaded parsing, memoisation (packrat), incremental parsing (`PROP-2026-0006` owns the latter).
- Comparing Capy to tools other than regex (`PROP-2026-0006` OQ-04).

## Proposed User Journey

```text
  maintainer ──► scripts/speedlab.sh run all
       │
       ├─► builds 20 engine variants from one clean export       (base, E1…E15, S1=A, B, C)
       ├─► for each corpus × variant: interleaved A/B runs, every sample kept
       ├─► checks output + `ast --json` byte-identical to baseline      ──► FAIL stops that variant
       └─► prints the ranked table
                      │
                      ▼
           method        target stage    predicted    measured    verdict
           E2 lazy fail  parse           −30 %        ?           ?       ◄── filled in AFTER the run
                      │
                      ▼
           TEST-2026-0014  (all samples)  ──►  ADR per adopted method  ──►  PLAN
```

## Requirements

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal |
|---|---|---|---|---|---|
| R1 | Every variant is built from a **clean export of commit `75a9f68`**, not the working tree | measurement | The first measurements were taken on a working tree holding uncommitted operator work (since committed as `75a9f68`, release 0.25.0; `git log`, 2026-10-08); a fixed commit makes the baseline reproducible; `QUAL-002` requires a controlled environment | `git archive 75a9f68 rust` is the only source; variant diffs are `diff -ru base variant` and stored | G-01 |
| R2 | Predictions and thresholds are frozen in this document at approval, before the first measured run | quality | `DOCUMENTATION.md` §21.3; `QUAL-002` | The approved revision's §Frozen Predictions is unchanged in the `TEST` document; later edits go in a new revision with the reason | G-02 |
| R3 | The baseline's run-to-run spread is measured first (E0) and every claimed gain must exceed **2× the spread of the quantity it is claimed on** (stage gains vs stage spread, end-to-end gains vs end-to-end spread) | measurement | `QUAL-002` ("all samples are reported, not only the best") | E0 reports ≥ 15 samples per corpus with min / median / max / spread | G-01 |
| R4 | A and B runs are **interleaved**, not run in blocks | measurement | `PROP-2026-0006` R7 ("interleaved") | The runner alternates baseline and variant per iteration | G-01 |
| R5 | A variant must produce byte-identical output **and** byte-identical `capy ast --json` on every corpus and every shipped sample, except E8 on inputs where baseline errors | compatibility | `GOAL-002`; `QUAL-001` | 0 differences across `samples/*/lib.capy` + the benchmark corpora | G-04 |
| R6 | `cargo test --workspace` and the golden corpus pass on every variant that is to be ranked | compatibility | `QUAL-001`; `GATE-002`; `PROP-2026-0006` T-04 records the current golden result (131 passed, 8 skipped, 0 failed) | Golden result recorded per variant; a failing variant is excluded from the ranking, not repaired silently | G-04 |
| R7 | No variant adds a dependency to `capy-core` | architecture | `ARCH-002` | `cargo tree -p capy-core --depth 1` shows exactly one dependency per variant | G-04 |
| R8 | No variant changes the type of an existing public field or signature | architecture | `ARCH-001` | `cargo doc`-level API diff (`cargo public-api` is **not** used — no new tool; use `diff` of `pub` items via `grep -n "^\s*pub "` on changed files) shows only additions | G-04 |
| R9 | No variant introduces unbounded recursion, an unbounded cache, or a panic reachable from input | architecture | `ARCH-003` | E7's cache has a fixed capacity; E4's index lookup cannot index out of range; the recursion-guard tests pass | G-04 |
| R10 | Library size F is a controlled variable: synthetic libraries at F = 4, 10, 100, 1 000, 10 000 with a declared shape mix (full matrix: R16) | measurement | P-10; shipped libraries have median F = 4, max F = 14 (123 libraries, counted 2026-10-08), so growth beyond that is a question about user-repository libraries (OQ-01) | Generator committed with the `TEST` document; shape mix recorded | G-03 |
| R11 | Every experiment records the **ranking inputs**: target-stage time, end-to-end time, allocations/line, peak live bytes, grammar-compilation time, wasm size delta | measurement | `PROP-2026-0006` M-08 (wasm ceiling), P-03 and UQ-06 | All columns filled for every variant | G-01 |
| R12 | E8 (bug fix) ships with a regression test that **fails on the baseline** and passes on the fix | quality | `QUAL-003` | The d8 reproduction is a test; reverting E8 makes it fail | G-06 |
| R13 | The programme's results are recorded in a tracked `TEST` document, including every sample | documentation | `DOCUMENTATION.md` §4.17, §21.3; `PHIL-001` | `TEST-2026-0014` exists with raw samples and the verdict table | G-01, G-02 |
| R14 | The programme feeds `PROP-2026-0006` INC-3 without duplicating its work | traceability | `PROP-2026-0006` §Plan Strategy ("INC-3 is deliberately unspecified … whatever INC-2's profile ranks first") | This proposal's Related Documents and the TEST cite PROP-2026-0006 R6/R7; no experiment here re-implements its R1–R3 (in-place `append`) | G-05 |
| R15 | Experiments do not edit tracked files, and `.ignore/` is never committed | process | `CODE-001` | `git status` shows no change outside `program_docs/` and (later) `rust/devtools/` after the experiments | G-01 |
| R16 | The benchmark matrix is run for the architecture comparison: library size F ∈ {4, 10, 100, 1 000, 10 000}; prefix sharing ∈ {unique, moderate, heavy}; captures ∈ {fixed, variable-length, nonterminal/recursive}; input ∈ {15-line script, large file, nested blocks}; outcome ∈ {valid, invalid, ambiguous}. One-factor sweeps around a declared centre point plus the full F × prefix-sharing cross; corpora scaled so no baseline cell exceeds 60 s | measurement | UQ-06; P-10 | `TEST-2026-0014` lists every cell run, its corpus size, and cells deliberately omitted with the reason | G-03, G-07 |
| R17 | Any variant that lets something other than the token matcher decide a statement — a compiled tree (E11, E12), the fused scanner (E13), specialised execution (E14), memoisation (E15), architectures B and C — obeys four rules. **(a) Eligibility:** a written rule for what it can handle. **(b) Exclusion proof:** it accepts a statement for candidate *i* only if every candidate ahead of *i* in the global sorted order has been **evaluated** (by the existing matcher, if unsupported) or is **provably unable to match** (e.g. its first literal differs from the line's first token, or its first capture type cannot start with that token kind). "Cannot prove" counts as "may match". **(c) Prefix handoff:** at the first statement it cannot decide with proof, or cannot match, it hands the rest of the script to the general path, which continues from that line; it never restarts and never re-enters. **(d) Differential test:** ≥ 100 000 generated statements — including libraries that contain unsupported candidates with **higher** priority than supported ones — plus every shipped sample give the same match/no-match, captures, output and `ast --json` as the token matcher | compatibility | `GOAL-002`; `QUAL-001`; `ARCH-003`; P-09 shows the class of cursor-state defect this guards against; UQ-07 (1)–(3) | T-13, T-15 and T-20 clean; handoff overhead within the thresholds in §Frozen Predictions | G-04, G-07 |
| R18 | The three architectures are compared on identical corpora, harness, metrics and verdict rules, and each must show identical observable behaviour (R5) | measurement | UQ-06 | One comparison table in `TEST-2026-0014`; no architecture is ranked on a cell where it handed off to the general path early unless its fast-path line share is reported | G-07 |
| R19 | Execution fast paths apply only to `run` / `run_multi`. `parse`, `parse_recovering`, `capy ast` and `capy ast --json` (and the wasm equivalents) return the **identical full tree** — spans, `leading_comments`, error nodes, diagnostics — for every input, whether or not an execution fast path exists. Where an E13 scanner is used for parsing at all, it must reproduce `leading_comments` and recovery behaviour, or the input is handed off | compatibility | UQ-07 (4); `GOAL-002`; `make_evaluator.rs` never reads `leading_comments` (searched 2026-10-08), so only the `ast` surfaces can observe them; templates *can* observe `line` and `col` (`make_evaluator.rs:206-207`) | T-19 clean: `capy ast --json` byte-identical with every variant enabled | G-04 |

## Use Cases

### Use-Case Catalogue

| ID | User Outcome | Actor | Surface and Trigger | Preconditions / Environment | Inputs | Outputs / Visible Result | Negative and Error Paths | Goal IDs | Requirement IDs | Test IDs |
|---|---|---|---|---|---|---|---|---|---|---|
| UC-01 | Maintainer gets a ranked list of speed-ups from one command | maintainer | `scripts/speedlab.sh run all` | clean export at `.ignore/speedlab/base`; release toolchain | corpora c10k, c100k, synthetic-F | table: method × stage × gain × allocations × verdict | variant fails equality → marked `EXCLUDED`, run continues | G-01, G-05 | R1, R3, R4, R11 | T-01, T-02, T-12 |
| UC-02 | Maintainer verifies a variant changes no behaviour | maintainer | `scripts/speedlab.sh verify <variant>` | variant built | all `samples/*/lib.capy` + scripts, corpora | `0 differences` or a unified diff | any difference → variant `EXCLUDED` | G-04 | R5, R6, R7, R8, R9 | T-03…T-06 |
| UC-03 | Maintainer sees how cost scales with library size | maintainer | `scripts/speedlab.sh scale` | synthetic generator | F ∈ {4, 10, 100, 1000, 10000} | µs/line per F per variant, and the slope | a library that fails `capy check` is a generator bug and stops the run | G-03 | R10 | T-07 |
| UC-04 | Maintainer learns whether predictions held | maintainer | reading `TEST-2026-0014` | run complete | frozen predictions | prediction-vs-actual table; each miss > 2× explained | missing sample → the row is `INCOMPLETE`, not interpolated | G-02 | R2, R13 | T-12 |
| UC-05 | Maintainer chooses what to adopt, in what order | maintainer | ADR per method | TEST complete | ranking rule | an ordered adoption list with S1's interaction result | a method whose gain is inside noise (R3) is `NOT SUPPORTED`, not adopted | G-05 | R3, R14 | T-12 |
| UC-06 | A library author with a punctuation-prefix grammar stops hitting a spurious "no match" | language author | `capy run` | grammar like d8 | `>< foo` | `B foo` (today: error) | none new; inputs that already matched are unchanged | G-06 | R5, R12 | T-08 |
| UC-07 | Maintainer compares three whole architectures on one workload set | maintainer | `scripts/speedlab.sh arch` | variants A, B, C built; matrix corpora generated | the R16 matrix | one table: throughput, latency, allocations, peak memory, compile time, eligibility coverage, fast-path line share | an architecture that changes any observable result is `EXCLUDED`; a cell where C handed off early is labelled, not ranked as a C win | G-07 | R16, R17, R18, R19 | T-14, T-15, T-17, T-18, T-19, T-20 |

### UC-01 — Run the whole programme

1. Maintainer runs `scripts/speedlab.sh setup` once. It exports `75a9f68` to `.ignore/speedlab/base` and copies it to one directory per variant.
2. `scripts/speedlab.sh run all` builds each variant's harness binary, then for each corpus alternates
   baseline and variant for N iterations (R4) and writes every sample to `results/<variant>/<corpus>.csv`.
3. The runner computes per-stage and end-to-end gains, compares with the frozen prediction, and prints the table.
4. Variants that fail equality (R5) print `EXCLUDED` and their timings are shown in parentheses, not ranked.

#### CLI Contract

```text
$ scripts/speedlab.sh run all --iters 15
[setup ] base=75a9f68  rustc 1.xx  host=darwin-arm64  profile=release
[build ] 20 variants .......................................................... ok
[verify] E1 output=identical ast-json=identical golden=ok
[verify] E8 output=identical* ast-json=identical* golden=ok   (* except d8: error -> match, expected)
[run   ] corpus c100k.txt  iters=15  interleaved
variant  stage   baseline(ms) variant(ms)  gain   predicted   allocs/line   verdict
E7       eval*     3386.0        412.4      −87.8%   −89 %       —           SUPPORTED
E2       parse      228.1        151.7      −33.5%   −30 %      39 → 22      SUPPORTED
...
$ echo $?
0
```

(Illustrative layout. Numbers in this block are **not** results.)

### UC-02 — Verify equality

Positive: all outputs and `ast --json` equal. Negative: one byte differs → the unified diff is printed
and the variant is excluded from the ranking (it can still be reported, labelled).

### UC-06 — The punctuation-split fix (E8)

```text
$ cat d8.capy                                  $ cat d8.txt
function a      priority 10                    >< foo
  literal ">"   literal "zzz"
function b
  literal "><"  capture w ident
  write `B ${w}`

before:  $ capy run d8.capy d8.txt
         error: no library function matches token "><"
after:   $ capy run d8.capy d8.txt
         B foo
```

### UC-07 — Compare the three architectures

1. Maintainer runs `scripts/speedlab.sh arch --matrix centre` (the centre-point cells) or `--matrix full`.
2. The runner builds A (= S1), B and C, generates the R16 corpora, checks R5 equality on every cell, then runs
   interleaved samples.
3. For C it also reports, per cell, the **fast-path line share** (lines decided by the scanner before the handoff) — a
   cell where the handoff happened early measures A or B, not C.

#### CLI Contract

```text
$ scripts/speedlab.sh arch --matrix centre --iters 15
cell: F=100 prefix=moderate captures=fixed input=large(100k) outcome=valid
arch  lines/s     latency(15 lines)  allocs/line  peak MB  compile(ms)  fast-path  equal
base   290 000        0.21 ms           56           41        0.4         —        —
A      ......         ......            ..           ..        ..          —        yes
B      ......         ......            ..           ..        ..          —        yes
C      ......         ......            ..           ..        ..         100 %     yes
$ echo $?
0
```

(Illustrative layout. Dots are not results.)

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says 2 — the unreconciled `PROP-2026-0004` OQ-05, unchanged) | 2026-10-08 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 | Applies | No grammar added; E4/E11–E14 index, compile and scan the **library-declared** shapes; the scanner's lexical rules are the lexer's own, shared, not a source language | PASS | — |
| GOAL-002 | Applies | R5/R6/R19: byte-identical output and `ast --json`; E8 changes behaviour only where baseline errors | PASS | E8 is the one non-identical variant by design (error → match); recorded under R5 |
| PHIL-001 | Applies | Every number in §Problem was measured; P-07 and P-09 were checked by reading the code and by reproduction. Predictions are labelled predictions | PASS | — |
| PHIL-002 | Applies | The idea file's internal ordering conflict (UQ-04) is stated, not smoothed over | PASS | — |
| CODE-001 | Applies | R15: no tracked change; `.ignore/` not committed | PASS | — |
| CODE-002 / CODE-003 | Applies | No helper, directive or capture type is added or renamed | NOT APPLICABLE | If an adopted method adds either, its plan carries the doc sync |
| ARCH-001 | Applies | `Token` is public (`token.rs:36`, `ErrorNode.tokens`); E5 keeps it and adds an internal type | PASS | E5 verdict contingent on R8 |
| ARCH-002 | Applies | R7; `SmallVec` from the idea file is rejected for this reason; the harness crate depends on `regex` and the engine only, and lives outside `capy-core` | PASS | — |
| ARCH-003 | Applies | R9; E7 cache is bounded; E4 index is built at load and indexed with checked lookups; recursion-guard tests run on every variant | PASS | The E12 thread buffer and E15 table are bounded; the scanner (E13) and trie (E11/E12) are iterative; any new recursive path adds a depth case |
| QUAL-001 | Applies | R5/R6 | PASS | — |
| QUAL-002 | **Applies** | §Frozen Predictions and §Measurement and Validation | PASS | Freezes at approval |
| QUAL-003 | Applies | R12; every correctness test must fail when its fix is reverted | PASS | — |
| GATE-001 / GATE-002 | Applies | R6, run on each ranked variant; the final adoption plans run both | PASS | — |

## Implementation Design

### Environment and Feasibility

| Environment / Version | Required Capability | Feasibility Evidence and Source | Constraint or Unknown | Resolution |
|---|---|---|---|---|
| macOS arm64, Darwin 25.4.0 | release build, in-process timing | the existing `.ignore/regex-compare/bench` crate builds `capy-core` as a path dependency and ran all numbers above | Laptop thermal and background load | R3 spread measured first; interleaving (R4); `caffeinate` + close other work during runs |
| `capy-core` as a library | `make_lexer`, `make_parser`, `make_evaluator`, `define_extractor` are `pub` | `phases.rs` calls them directly (verified, it compiled and ran) | E5 changes the lexer→parser interface | harness times `lex+parse` combined when the stage API differs; `Library::run` end-to-end is always measured |
| Counting global allocator | allocs/line per stage | implemented in `phases.rs`; produced 11 / 39 / 6 | counts calls, not bytes retained | bytes/line also recorded |
| `git archive` | clean export of `75a9f68` | `git rev-parse 75a9f68` resolves (HEAD at 2026-10-08, release 0.25.0) | none | R1 |
| wasm target | wasm size delta per method | `rust/target/wasm32-unknown-unknown` exists; `rust/devtools/wasm_check.sh` is the gate | wasm timing is out of scope | only size is measured (R11) |
| Profiler | per-site attribution | `perf`, `samply`, `cargo-flamegraph` **not installed** (checked 2026-10-08) | no sampling profiler | E0 attributes by **allocation site** with a labelled counting allocator and by deleting work (stub experiments), not by sampling |

### Methods by Use Case

| Change ID | UC IDs | Method and Execution Order | Positive Path | Negative / Failure Path | Requirement Fulfilment | Feasibility |
|---|---|---|---|---|---|---|
| C-01 | UC-01 | `scripts/speedlab.sh` + `.ignore/speedlab/` layout (§Sandbox) | one command ranks all variants | build failure of one variant does not stop the others | R1, R4, R15 | proven (all pieces already ran) |
| C-02 | UC-01 | E0 baseline + noise floor + allocation-site attribution | frozen baseline | spread > 5 % → rerun, then stop and report; no ranking on a noisy host | R3 | proven |
| C-03 | UC-06 | E8 token-split fix | d8 passes, no golden diff | any golden diff → investigate before anything else is measured (it would invalidate E1–E5) | R5, R12 | experiment needed |
| C-04 | UC-01 | E7 regex cache | `contains` collapses | cache growth unbounded → bound it (R9) | R9 | proven locally |
| C-05 | UC-01 | E1 borrowed `peek()` | parse faster | borrow-checker conflict with `&mut self` methods → restructure to index-based accessors | R5 | experiment needed |
| C-06 | UC-01 | E2 lazy failure path | parse faster, fewer allocations | diagnostics text must be byte-identical → build the message only on final failure | R5 | experiment needed |
| C-07 | UC-01 | E3 no element clone; indexed capture slots | parse faster | evaluator reads `captures` as a map → convert on success only | R5 | experiment needed |
| C-08 | UC-01 | E9 fuse comment filtering | small parse gain | none beyond equality | R5 | proven |
| C-09 | UC-03 | E4a hoist sort; E4b first-token index; E4c kind filter | scales with F | punct-prefix literals break a naive hash lookup (see E4b) | R5, R9, R10 | experiment needed |
| C-10 | UC-01 | E5 source-backed internal tokens | lexer faster | `ErrorNode.tokens` needs public `Token` → convert only there | R8 | experiment needed |
| C-11 | UC-01 | E6 precompiled templates | evaluator faster | first step attributes eval cost; if `eval_interp` < 25 % of eval, abandon | R5 | experiment needed |
| C-12 | UC-01 | E10 AST-share spike (measure only) | decides whether direct evaluation is worth a variant | none | R11 | proven |
| C-13 | UC-03 | E11 deterministic compiled matcher / shared-prefix tree (ungated since rev 2) | gain grows with F and prefix sharing | trie loses to hash dispatch on unique prefixes → reported, not hidden | R5, R9 | experiment needed |
| C-14 | UC-01 | S1 stack of winners | interaction measured | non-additive → report the factor | R11 | experiment needed |
| C-15 | UC-05 | `TEST-2026-0014` + ADR per adopted method | traceable adoption | — | R13, R14 | proven |
| C-16 | UC-03, UC-07 | E12 progressive tree with active states | ambiguity and invalid input handled without per-candidate retry | committing a lower-priority accept too early → ordered-choice test | R5, R9, R17 | experiment needed |
| C-17 | UC-07 | E13 fused scanner | no Token vector for the leading run of decidable statements | first statement not decidable with proof → prefix handoff to the general path | R5, R17, R19 | experiment needed |
| C-18 | UC-07 | E14 specialised execution | flat emit ops for write-only functions | helper/pipe/`set` in the body → ineligible | R5, R17 | experiment needed |
| C-19 | UC-04 | E15 memoised matcher (gated by a repetition measurement) | measured repetition decides | gate not met → not built, recorded | R5 | blocked until gate |
| C-20 | UC-07 | Architectures A / B / C and the R16 matrix | one comparison table | cell where C handed off early is labelled | R16, R17, R18 | experiment needed |

### Sandbox

```text
  .ignore/speedlab/                 (untracked; CODE-001)
  ├── base/rust/…                   git archive 75a9f68 rust | tar -x   (never edited)
  ├── v/E1/rust/… … v/S1/rust/…     cp -R base, then the variant's patch only
  ├── patches/E1.diff … S1.diff     `diff -ru base v/En` — the reviewable artefact
  ├── harness/                      one crate; `capy-core = { path = "$VARIANT/rust" }`
  │   ├── src/bin/e2e.rs            Library::run, end to end, hashes output
  │   ├── src/bin/phases.rs         lex / parse / eval split + counting allocator
  │   ├── src/bin/scale.rs          F = 4, 10, 100, 1000, 10000
│   ├── src/bin/matrix.rs         R16 matrix cells; peak-bytes allocator; compile-time split
│   └── src/bin/fuzz.rs           differential test: fast path == token matcher (R17)
  ├── corpora/  gen.py c10k c100k nested/ synthetic-F/ matrix/ (scaled so no baseline cell > 60 s)
  ├── results/<variant>/<corpus>.csv     every sample
  └── scripts/speedlab.sh           setup | build | verify | run | scale | report
```

```text
  interleaving (R4)                         equality gate (R5)
  ─────────────────                         ─────────────────
  for i in 1..N:                            for corpus in corpora + samples/*:
      run baseline   ──► sample              out_b  = baseline.run(corpus)
      run variant    ──► sample              out_v  = variant.run(corpus)
  (never AAAA…BBBB…)                         ast_b  = baseline.ast_json ; ast_v = variant.ast_json
                                             out_b == out_v  &&  ast_b == ast_v   else EXCLUDED
```

### Three Architectures Compared

The experiments below are single methods. They assemble into three complete designs, because "faster token
matching" and "do less work per statement" are different bets (UQ-06).

```text
        ┌──────────────────────── common foundation F0 ────────────────────────────┐
        │  E8 cursor fix · E1 borrowed peek · E2 lazy failure · E3 slots · E9 ·      │
        │  and E5/E6/E7 where SUPPORTED                                              │
        └────────────┬─────────────────────────┬──────────────────────────┬──────────┘
                     ▼                         ▼                          ▼
   A  OPTIMISED INDEXED PARSER     B  COMPILED PROGRESSIVE TREE    C  FUSED SCANNER + COMPILED EXECUTION
      (= stack S1)                     F0 + E11 + E12                  F0 + E13 + E14, general path takes over
      bytes → tokens → candidate       bytes → tokens → trie with      bytes ──► eligible plan ──► output
      list narrowed by E4b             active states (ordered)          first undecided statement ─► handoff to A or B
      FuncCall tree → evaluator        FuncCall tree → evaluator       no Token vector, no FuncCall
```

| Architecture | Removes | Keeps | Main risk | Expected best case |
|---|---|---|---|---|
| A — optimised indexed parser | allocation inside the existing passes; irrelevant candidates | tokens, tree, evaluator | low (local edits) | broad, modest gain on every script |
| B — compiled progressive tree | repeated work across candidates that share a prefix | tokens, tree, evaluator | medium (ordered-choice semantics) | large F with shared prefixes |
| C — fused scanner + compiled execution | the lexer pass, Token objects, `FuncCall`/capture maps, template interpretation — **for eligible scripts** | the general path for everything else | high (must replicate lexer rules exactly) | flat, single-line, write-only grammars |

Mapping of the review's architecture list onto the experiments:

| Review's architecture | Experiment | Note |
|---|---|---|
| Linear candidate matching | baseline | measured in E0 |
| First-token hash dispatch | E4b (+E4a, E4c) | "good for small libraries": tested at every F |
| Progressive matching tree | E11 | now ungated |
| Trie + NFA active states | E12 | new |
| Compiled DFA-like fast paths | E11 deterministic paths; E13 on bytes | the deterministic subset |
| Fused lexer + matcher | E13 | new |
| Memoised matching graph | E15 | new, gated by a measurement |
| Streaming execution without an AST | E14 (and the E10 spike sizes it) | new |

**The trie is not assumed to win.** A hash lookup into a short bucket may beat a trie for a library of a few
unrelated definitions; the matrix (R16) is built to show where each design crosses over.

**Comparison rules.** Same corpora, same harness, same metrics (throughput, 15-line latency, allocations/line, peak
live bytes, grammar-compilation time, generated-plan size), identical observable behaviour (R5). Compilation time is
reported **separately** from execution time, with the **break-even number of runs** (compile-time increase ÷
per-run saving), because Capy libraries are reused many times in some uses (batch transpile, editor) and once in others.
An architecture is preferred only where it wins on the cells that represent libraries the project actually has
(OQ-01), by more than twice the measured spread, and — for C — with its fast-path line share stated.

### The Experiments

Each experiment lists: **Hypothesis and basis**, **Steps**, **Correctness risks**, **Measurement**,
**Expected result** (frozen, repeated in the table in §Frozen Predictions) and **Decision rule**.
Line numbers refer to commit `75a9f68`.

#### E0 — Freeze the baseline and the noise floor (control)

**Hypothesis.** The baseline is stable to within 3 %, and the parse stage's 39 allocations per line
split into identifiable sites.

**Steps**

1. Export `75a9f68`; build the harness against `base`.
2. For c10k, c100k, `t_import`, `t_say`, `t_assign`, `t_all`, `t_contains`: run `e2e` and `phases` 15 times each,
   alternating libraries so thermal drift hits all equally. Record every sample.
3. Compute min / median / max / spread = (max−min)/median per cell. This spread is the **noise floor** (R3).
4. Add a labelled counter around each allocation-heavy site in a **copy** of `base` (not the baseline binary):
   `peek` clone, `ContextFrame::new`, `PatternElement` clone, `BTreeMap::new` for captures, `format!` in the
   literal-failure arm, `note_failure` clone, `FuncCall` construction. Report allocations/line per site.
5. Stub experiments to bound each cost without implementing it: (a) make `try_match` return `Err` immediately
   for every non-first candidate ("free failure" upper bound); (b) make `peek()` return a cached empty token
   ("free peek" upper bound). Outputs will be wrong; only the **time** is read. This gives each method an
   upper bound before it is built.
6. Count the shipped libraries (`samples/*/lib.capy`): functions per library, and classify every function as
   **eligible** or **ineligible** for the E13/E14 subset by the rule in E13 (directive scan plus capture types).
   Already counted on 2026-10-08: 123 libraries, functions per library min 0, **median 4, max 14**, total 544;
   about 84 % of functions carry no `block_*`, `when_*` or `priority` directive (directive scan only — capture
   types not yet checked).
7. Extend the counting allocator to track **live bytes** (allocations minus deallocations) and record the peak; it
   currently counts allocations only.

**Measurement.** Spread per cell; allocations/line per site; the two upper bounds; functions per shipped library;
fast-path-eligible share of shipped functions; peak live bytes per stage.

**Expected result.** Stage-level spread (lex, parse, evaluate, in-process) ≤ 5 %; end-to-end `Library::run` spread **5 – 15 %** (a +17 % swing between two same-day sessions was already observed, P-02 note). Stop-and-rerun line: stage spread > 8 %. Failed-candidate work
(ctx frame + element clones + capture map + `format!` + `note_failure`) accounts for **40–60 %** of parse
allocations; `peek()` clones account for **15–25 %**; the successful match path for the remainder.
"Free failure" upper bound: parse stage **−35 % to −55 %**. "Free peek": **−10 % to −20 %**. Fast-path-eligible share of shipped functions: **50 – 75 %** (upper bound 84 % from the directive scan alone).

**Decision rule.** If the **stage** spread exceeds 8 % after reruns, stop; no ranking is published (R3). End-to-end gains are judged against the end-to-end spread, stage gains against the stage spread. If the free-failure
upper bound is below 15 %, E2 is dropped before it is built.

#### E8 — Fix the punctuation-split mutation (correctness control; do first)

**Hypothesis.** The in-place rewrite of `self.toks[self.pos]` in `match_literal` (`make_parser.rs:1204-1240`)
is the cause of P-09, and a cursor-state representation fixes it without changing any previously-working parse.
It is a control because E2, E3, E5 and E11 all rewrite the cursor.

**Steps**

1. Add the d8 reproduction as a failing test in the variant (R12). Confirm it fails on `base`.
2. Replace the token rewrite with a **split offset** held in the parser: `pos` plus `split: usize` (bytes of the
   current punct token already consumed). `peek()` returns the token's remaining text through the offset;
   `advance()` clears `split`; `save()`/`restore()` capture and restore `(pos, split)`.
3. Keep every other code path identical.
4. Run the d8 test (now passes), the golden corpus, and the equality gate on all corpora.

**Correctness risks.** A `save()` taken mid-split must restore the split; `resync` and error spans read token
text/columns (`span_between`) and must see the adjusted col/width. A grammar that depends on the **buggy**
behaviour would change — the golden corpus is the detector (QUAL-001).

**Measurement.** Parse-stage time on c100k (should be neutral) and the golden result.

**Expected result.** d8 prints `B foo`. Golden: 0 differences. Time: **within the parse-stage spread** (≈ ±3–5 %).
If time moves more than that, the offset representation itself is costly and E2/E3/E5 must be re-based on it.

**Decision rule.** Always adopt if the golden corpus shows 0 differences, regardless of speed. If any golden
differs, stop the programme and report: the defect has been load-bearing.

#### E7 — Cache compiled regexes in `regex_match`

**Hypothesis.** The 940x on `contains` is `Regex::new` per call (`inner_evaluator.rs:823-832`); caching compiled
patterns removes it. The remaining cost is the ordinary pipeline plus one `is_match`.

**Steps**

1. Add a bounded cache: `thread_local!` map `String → Regex`, capacity 64; when full, clear it (no LRU, no new type).
   `Regex::clone` is cheap (reference-counted internals).
2. `regex_match` looks up the pattern string; on a miss compiles, inserts, and uses it. A compile **error** is not
   cached and keeps the exact current message (`go_regex_error`).
3. Apply the same lookup to the other `Regex::new` sites found by `grep -n "Regex::new" rust/src` (the `pattern`
   type validation in `make_evaluator.rs:394-406` and `define_extractor.rs`); each is measured separately.
4. Run `t_contains` and the five other tasks.

**Correctness risks.** Pattern text built from captures (dynamic patterns) could fill the cache → bound (R9). Error
text and error timing must not change. `thread_local!` on wasm32 (single-threaded) is supported by std.

**Measurement.** `t_contains` end-to-end; the other tasks (should be unchanged); wasm size delta.

**Expected result.** `contains` **3 386 ms → 380 ± 80 ms** (≈ 9x). It cannot approach the regex's 3.6 ms because the
pipeline cost (≈ 350 ms with a catch-all `tail` per line) is still paid — this is the Amdahl floor. The five
other tasks: within noise. Wasm size: **+0.5 to +2 KB**.

**Decision rule.** Adopt if `contains` improves ≥ 5x and nothing else moves outside the noise floor.

#### E1 — Borrowed `peek()` (no Token clone)

**Hypothesis.** `peek()` (`make_parser.rs:255`) clones a `Token` including its `String`, several times per element
(P-05). Reading the kind and text by reference removes those allocations.

**Steps**

1. Add `peek_kind(&self) -> TokenKind` (Copy) and `peek_text(&self) -> &str`, returning from `self.toks[self.pos]`
   or from a static EOF sentinel.
2. Replace the clone-then-compare call sites in `try_match_inner` (the comma auto-skip does `self.peek().kind` and
   `self.peek().text` — two clones), `match_literal`, `at_statement_end`, `parse_program`'s loop, and the capture
   readers in `capture_value` that only inspect kind/text.
3. Where a `Token` value is genuinely needed (building `CaptureValue.text`, error nodes), clone once there.
4. Keep `peek()` for the rest; do not change signatures of anything `pub`.

**Correctness risks.** A borrowed `&str` from `self.toks` conflicts with `&mut self` calls (`advance`) in the same
expression — restructure with a copy of the small decision (`bool`/`kind`) before mutating. No output change expected.

**Measurement.** Parse stage; allocations/line (parse).

**Expected result.** Parse **−10 % to −20 %** (central −15 %: 228 → ≈ 194 ms on `t_import`); parse allocations **39 → ≈ 31 to 34**.
Lex and evaluate unchanged. Basis: E0's "free peek" bound and P-05.

**Decision rule.** Adopt if parse ≥ −8 % beyond 2× spread. Cheap, local, low risk.

#### E2 — Allocation-free failed candidates (lazy failure path)

**Hypothesis.** A failed candidate costs ≈ 5–8 allocations (P-04) that only matter if parsing ultimately fails.
Deferring them removes most of the parse stage's allocations on the success path. This is the idea file's "make
matching allocation-free until success" (§5).

**Steps**

1. `ctx_stack`: push a lightweight frame `(func_index, arg_index)` instead of `ContextFrame::new(&f.name, 0, "")`
   (which owns two `String`s); materialise names only when `note_failure` records a **new furthest** position or an
   error is finally built. (`make_parser.rs:956-960`, `905-921`.)
2. `Expectation::Literal(el.literal.clone())`: store `(func_index, element_index)`; resolve to text only when the
   diagnostic is formatted.
3. The literal-failure arm (`try_match_inner`) returns a zero-allocation sentinel error (`CapyError` kind
   `NoMatch`, no message) for candidates that are merely *tried*; the message is built by `furthest_error()` at the
   end, from the recorded expectation, **only if no candidate matched**.
4. Keep `block_err` (first error from a candidate that matched its header but failed its body) exactly as today.
5. Verify the final error text is byte-identical on a failing-input corpus (every `E00xx` diagnostic code, the
   `hint:` lines, `did you mean`).

**Correctness risks.** The highest of Tier A. Diagnostics are an observable contract (`ast --json` carries
messages and codes; goldens pin them). `furthest` bookkeeping must see the same positions in the same order.

**Measurement.** Parse stage; allocations/line; **and** a failing-input corpus (1 000 distinct malformed scripts
produced from the golden error cases) compared byte for byte.

**Expected result.** Parse **−25 % to −40 %** (central −30 %: 228 → ≈ 160 ms); parse allocations **39 → ≈ 18 to 24**.
The gain grows with the number of candidates tried per line: larger on `t_all` (4 functions) than on `t_import` (2).
Basis: P-04's list and E0's failed-candidate share.

**Decision rule.** Adopt if parse ≥ −15 % **and** the failing-input corpus is byte-identical.

#### E3 — No `PatternElement` clone; indexed capture slots

**Hypothesis.** `try_match_inner` clones every `PatternElement` (`make_parser.rs:~975`, `let el = f.elements[i].clone()`)
and builds a `BTreeMap<String, CaptureValue>` even for attempts that fail.

**Steps**

1. Borrow the element (`let el = &f.elements[i]`) — `f: &FuncDef` is already shared; split the borrow from
   `&mut self` by taking `f` as `&Arc<FuncDef>` held outside `self`.
2. Collect captures into a `Vec<(usize /*element idx*/, CaptureValue)>` local to the attempt (no map, no key
   `String` clone); on **success**, build the `BTreeMap` the evaluator expects, using `el.name.clone()` once per capture.
3. `next_literals(&f.elements[i+1..])` currently allocates a `Vec<String>` of clones per capture
   (`:1872`); precompute per function at load, or return `&[PatternElement]` and compare in place.
4. Leave `FuncCall`'s public shape alone (ARCH-001).

**Correctness risks.** Duplicate capture names (the map silently overwrote); preserve "last wins" when converting.
Optional-capture defaults fill by name.

**Measurement.** Parse stage; allocations/line.

**Expected result.** Parse **−8 % to −15 %** on its own; **less** when stacked on E2 (E2 already removes the failed-attempt
share of these allocations). Isolated: 39 → ≈ 31 to 34 allocations. Basis: three allocations per element visited.

**Decision rule.** Adopt if the **marginal** gain on top of E1+E2 (measured in S1 ordering) is ≥ 5 %.

#### E9 — Fuse comment filtering out of the parser

**Hypothesis.** `parse_impl` re-walks the whole token vector (625 001 tokens on c100k) to strip comments, building a
second vector and a `BTreeMap` (`make_parser.rs:46-76`), even when there are no comments.

**Steps**

1. If the lexer emitted **no** `Comment` token (it knows), pass the vector through untouched (a `bool` or `leading.is_empty()`
   returned alongside the tokens by an internal entry point).
2. Otherwise keep today's path.
3. Keep `tokenize_with_trivia`'s public signature (`make_lexer.rs`); add the flag as an internal function.

**Measurement.** Parse stage on c100k (no comments) and on a commented corpus (should be unchanged).

**Expected result.** Parse **−2 % to −6 %** on comment-free input; **0 ± noise** on commented input. Basis: one pass over
625 001 tokens plus one vector allocation.

**Decision rule.** Adopt only if the **parse-stage** gain is ≥ 2× the parse-stage spread (≈ ≥ 6–10 %). A −2 to −6 % prediction therefore sits at or below the detection limit: **predicted verdict: `NOT SUPPORTED`** unless it measures above its prediction.

#### E4a — Hoist candidate construction and sorting to library load

**Hypothesis.** `parse_impl` deep-clones every `FuncDef`, wraps it in `Arc`, sorts the list and builds `by_name` on every
run (`make_parser.rs:76-103`, P-08). This is invisible at 100 000 lines and large for short scripts and editors.

**Steps**

1. Build `Vec<Arc<FuncDef>>` (sorted) and `by_name` once, in the loader (`make_library_loader`), and store them in
   `Library` as **new, additive** fields (ARCH-001: new field, `#[non_exhaustive]` already on the type).
2. `parse_impl` clones the two `Arc`s (cheap) instead of rebuilding.
3. A `define` block merges functions into a **copy** of the library at run time (`capy.rs:195-215`); in that case
   rebuild the sorted list for the copy, as today.

**Measurement.** Three script sizes: 15 lines, 1 500 lines, c100k; with libraries at F = 4, 100, 1 000.

**Expected result.** c100k: **0 ± noise** (the sort is a fixed cost). 15-line script, F = 4: **−5 % to −15 %** of the
whole `Library::run`. F = 1 000, 15-line script: **−60 % to −90 %** (the per-run clone and sort of 1 000 definitions
dominate). Basis: per-run cost O(F log F) plus F deep clones.

**Decision rule.** Adopt if the 15-line / F = 100 case improves ≥ 20 %. This matters for editors and the wasm playground,
not for batch throughput.

#### E4b — First-token dispatch index (the idea file's "indexed dispatch")

**Hypothesis.** Most candidates fail at their first element. Selecting only candidates whose **first element could match
the current token** removes the failed-candidate work for them entirely. The gain scales with F.

**Steps**

1. At load (after E4a) build, for each function, its **first-element key**:
   - first element a literal → key = that literal's text; `kind = Literal`;
   - first element a capture → `kind = Wildcard`.
2. Build `HashMap<&str, Vec<u32>>` mapping each literal to the indices of the functions that start with it, plus a
   `wildcards: Vec<u32>` list. **Each list is stored in the global sorted order** (priority, literal-start, literal
   run, name), so merging a literal bucket with the wildcard list by index reproduces the exact candidate order.
3. `parse_stmt` computes the current token's text, looks up its bucket, **merges** the bucket and `wildcards` by
   sorted index (two-pointer, no allocation: iterate with two cursors), and tries candidates in that order.
4. **Punct prefixes.** The lexer merges punctuation (`><` is one token) and `match_literal` accepts a literal that is a
   *prefix* of a merged token (`make_parser.rs:1226-1236`). So for a `Punct` token the lookup must consider every
   literal bucket that is a prefix of the token text: iterate prefixes of the token text (≤ the merged run length) and
   union their buckets. Non-punct tokens need exact match only.
5. `when_followed_by` / `when_not_followed_by` and `block_*` functions are filtered by their first element like any
   other; nothing else in `parse_stmt` changes.

```text
   token "show"                                  global sorted order (priority, literal-start, run, name)
        │                                         0 long   ("show","all",ident)
        ▼                                         1 short  ("show",ident)
   bucket["show"] = [0, 1]  ──┐                   2 import ("import",ident)
   wildcards      = [3, 4]  ──┼─ merge by index ─► try 0, 1, 3, 4      (2 is never tried)
                              │                   3 assign (ident "=" int)    ← capture-first
                              ┘                   4 other  (tail)             ← capture-first
```

**Correctness risks.** First-match semantics must be unchanged: the dispatcher returns a **subsequence** of the
original order, never a reordering (idea file §3 says the same). A function whose first element is a
function-typed capture (nonterminal) is a wildcard. Block bodies call `parse_stmt` recursively — same index.

**Measurement.** `scale`: F ∈ {4, 10, 100, 1 000}; shape mix declared in R10 (60 % distinct-first-literal, 20 % shared
first literal with 2–5 variants, 10 % capture-first, 10 % punct-first); lines match uniformly across functions.

**Expected result.**

| F | baseline µs/line | with E4b | predicted gain |
|---|---|---|---|
| 4 | ≈ 2.3 – 2.9 (measured) | within noise to −10 % | `t_import` ≈ 0; `t_all` −5 to −15 % |
| 10 | ≈ 4 – 7 | ≈ 2.5 – 3 | −35 to −60 % |
| 100 | ≈ 30 – 60 | ≈ 3 – 4 | **≥ 8x** |
| 1 000 | ≈ 300 – 600 | ≈ 3 – 5 | **≥ 50x** |
| 10 000 | ≈ 3 000 – 6 000 (corpus scaled to ~2 000 lines) | ≈ 3 – 6 | **≥ 500x** |

Basis: each extra failing candidate costs roughly the E0-measured failed-attempt cost (≈ 0.3–0.5 µs at today's
allocation level), linear in F; the index makes cost depend on bucket size, not F. **The baseline column is a
prediction; F ≥ 10 has never been measured (P-10).** After E2 lands, the baseline slope falls (a failed candidate gets
cheap), so E4b's marginal gain at F = 100 is predicted to shrink from ≥ 8x to **≈ 2–4x**. That interaction is the
main reason S1 exists.

**Decision rule.** Adopt only if a library the project actually ships — or the owner states users will write — benefits.
The shipped maximum is **F = 14** (median 4; counted 2026-10-08), below the F ≈ 20 line, so on shipped data alone E4b (and
E11/E12) are `NOT SUPPORTED` for adoption and are kept as documented future work, **whatever the synthetic result**.
Libraries live in users' repositories (`GOAL-002` rationale), so the answer to OQ-01 can change this.

#### E4c — Token-kind filter for capture-first functions

**Hypothesis.** `assign` (`ident "=" int`) is capture-first, so E4b leaves it in `wildcards`; it can only match when the
token is an `Ident`. Filtering wildcards by acceptable first-token kind removes more failed candidates.

**Steps**

1. Map each wildcard's first capture type to the token kinds it can start with: `ident`/`dotted_ident` → `Ident`;
   `int`/`float` → `Number`; `string` → `Str`; `bool` → `Ident`; `any`/`raw`/`word`/`tail`/function-typed → all.
2. Keep one wildcard list **per token kind** (still in global order); `parse_stmt` uses the list for the current kind.

**Correctness risks.** `any` and `word` can begin with a punct or paren token (`-x`, `(a)`). Conservative rule: any type not in
the table above is "all kinds".

**Expected result.** On `t_all` (`assign` is the only kind-restricted wildcard; `other`/`tail` accepts all): parse
**−3 % to −8 %** on top of E4b; negligible on `t_import`. Larger on libraries with many `ident`/`int`-first shapes.

**Decision rule.** Adopt only with E4b and only if ≥ 3 % on a shipped library.

#### E5 — Source-backed internal tokens

**Hypothesis.** Each token owns a `String` (11 allocations per line in the lexer). Storing `(kind, start, end)` into
the source string and recovering text by slice removes them, and removes `String` clones in the parser's cursor reads.

**Steps**

1. Add an **internal** `STok { kind: TokenKind, start: u32, end: u32, line: u32, col: u32, width: u32 }` and a
   `Lexed { src: Arc<str>, toks: Vec<STok> }`. `Token` stays as-is (public).
2. A new internal lexer entry point produces `Lexed`; `tokenize_with_trivia` remains as a wrapper that materialises `Token`s
   (so every existing caller, and the library-file parser, are unaffected).
3. The parser consumes `Lexed`. Where the string content differs from the source slice (a `Str` token stores its
   content **without quotes**; `width` includes them; escape handling), record the content range, not the lexeme range;
   verify with the `tail`/`requote` paths (`make_parser.rs:1887-1945`).
4. `ErrorNode.tokens: Vec<Token>` (public) is built by materialising the few skipped tokens on error only.
5. Line/column are already stored per token; do not introduce a separate line-index.

**Correctness risks.** The highest of Tier B. `Template` tokens keep raw content with `${}`; `Str` escapes; backtick multi-line
merging (`merge_backtick_lines`); tabs counted as 4 for indent but 1 for column. The lexer is also used for libraries and the
inner DSL (`tokenize`) — they must keep the old path.

**Measurement.** Lex stage; parse stage; end to end; allocations/line; `ast --json` equality (spans must be unchanged).

**Expected result.** Lex **−40 % to −60 %** (63 → ≈ 25 – 38 ms); lex allocations **11 → ≈ 2 – 4**; parse **−5 % to −10 %**
from fewer cache misses and no string clones. End to end on `t_import`: **−15 % to −22 %**. Basis: allocation share of lex.

**Decision rule.** Adopt if end-to-end ≥ −10 % beyond 2× spread **and** `ast --json` is identical **and** R8 holds. Because
it is the largest rewrite in Tier B, it is ordered after the Tier A results are known.

#### E6 — Precompile templates

**Hypothesis.** `interpolate_render` scans the template bytes and calls `eval_interp` (parse + evaluate the `${…}` body)
**on every render** (`inner_evaluator.rs:506`, `:566`, P-07). Parsing each template once at library load, rendering from a
vector of `Static(String)` / `Expr(ParsedExpr)` pieces, removes the repeated work.

**Steps**

1. **Attribution first.** In a copy of `base`, time `eval_interp` and `interpolate_render` separately with a counter
   (`Instant` accumulators). If `eval_interp` is **< 25 %** of the evaluate stage, stop: the expected gain is too small.
2. Add `CompiledTemplate { parts: Vec<Part> }`; `Part::Static(String)` | `Part::Expr(Box<Expr>)` where `Expr` is the
   existing parsed inner expression type.
3. Compile at library load for every `write` template whose text is a literal (the common case); templates built at run time
   keep the current path.
4. `render_stmt` renders from the compiled form: concatenate statics, evaluate `Expr` parts with the existing `eval_render`.
5. Preserve the escape semantics of `interpolate_render` exactly (`\n \t \r \` \\ \X`) when compiling the static parts.

**Correctness risks.** Escapes; `${ … }` containing `}` in a nested string (the scanner tracks `{}` depth); evaluation errors
must still be reported at the same line with the same text, which now happens at **load** for a bad template instead of at
first render → this is an **observable change** (error timing). Mitigation: compile lazily on first render and cache, so
errors still surface at the same moment.

**Measurement.** Evaluate stage on `t_import` (1 `${}`), `t_say` (1), `t_all` (mixed), and a template-heavy library
(8 `${}` per template, helper pipes).

**Expected result.** Evaluate **−20 % to −35 %** on `t_import` (58 → ≈ 38 – 46 ms); **−30 % to −50 %** on `t_all` (149 → ≈ 75 – 105 ms);
**−50 % to −70 %** on the template-heavy library. End to end on `t_all`: **−10 % to −20 %**. Basis: P-07; verified only by
reading the code and the doc comment — the step-1 attribution is what actually confirms or kills it.

**Decision rule.** Adopt if evaluate ≥ −15 % on `t_all` and error behaviour (lazy compile) is byte-identical.

#### E10 — AST-construction share (spike, measure only)

**Hypothesis.** Part of parse cost is building the `FuncCall`/`CaptureValue` tree (captures map, spans, `String` clones for
`func`/names) that a pure transpile never inspects beyond rendering. If it is large, "direct evaluation without a full AST"
(idea file §8) deserves a variant; if small, it does not.

**Steps**

1. In a copy of `base`, make `parse_stmt` build the `FuncCall` but drop it immediately (do not push to `stmts`); measure parse time.
2. Separately, make `try_match_inner` skip `span_between` and `leading_comments` cloning (wrong spans, correct output) and measure.
3. Report the AST/span share of parse as a percentage and in allocations/line. **No variant is built.**

**Expected result.** AST + span construction is **10 % to 20 %** of parse. A direct-evaluation variant could therefore buy at most
that much on top of E1–E5, while giving up the whole-program tree the evaluator needs for error refusal (`PLAN-2026-0002` R23, `make_evaluator.rs:63`)
and tooling. **Predicted verdict: not worth a variant.**

**Decision rule.** Build a direct-evaluation variant only if the share is ≥ 25 %.

#### E11 — Deterministic compiled matcher and shared-prefix tree (ungated since revision 2)

**No gate.** Revision 1 ran this only if the optimised parser stayed above a timing line. The review argued that a
tree's value grows with library size even when the 4-function benchmarks become fast, and that the question is
architectural, not a leftover. Agreed for the **experiment**: it is cheap relative to what it teaches. **Adoption** is
unchanged — it still needs the E4b library-size rule (shipped max F = 14; OQ-01).

**Hypothesis.** A compiled matcher (the idea file's §4 "matching program" and the user's progressive-filtering tree) removes
repeated work between functions that share a prefix and replaces interpreted element-by-element matching with straight-line
operations.

**Steps (prototype restricted to a safe subset)**

1. Compile each function whose elements are only `literal` and `ident | int | string | raw | tail` captures (no nonterminals,
   no blocks, no `when_*`, no `sep`/`join`, no optional) into ops: `Lit(text)`, `CapIdent(slot)`, `CapInt(slot)`, `CapString(slot)`,
   `CapRaw(slot)`, `CapTail(slot)`, `Accept(func)`.
2. Build a trie keyed on the first op's literal; states hold the remaining ops; wildcard (capture-first) functions form an
   alternative path in the same state. Ordered-choice semantics: states carry candidate indices in global order; the **first
   accepting candidate in that order** wins, so a lower-priority function that finishes earlier does not pre-empt a higher one.
3. Functions outside the subset stay in the candidate order as **interpreted candidates**. The tree yields candidates in global order; compiled ones are decided by the trie, interpreted ones by the existing matcher. A compiled accept for candidate *i* is committed only after every interpreted candidate with index < *i* has been evaluated or excluded by proof (R17 (b)). A statement the tree cannot decide is handed to the E4b candidate list.
4. Equality gate as for every variant; plus a randomised differential test: 10 000 generated statements over a generated
   library, trie result == linear-scan result (or both fail).

**Correctness risks.** Priority vs earliest-accept — in particular an **unsupported higher-priority candidate** that would match the line but is invisible to the tree; `tail` greed; `word`/`any` multi-token captures excluded by the subset
rule; backtracking semantics when a later element fails after a shared prefix matched.

**Expected result.** Against **E4b** (not against the linear baseline): on libraries with **unique first literals** the
tree is **no faster** — 0.8 – 1.3x of E4b's parse time at F = 100, because a hash lookup into a one-entry bucket is already
minimal. With **heavily shared prefixes** (many functions starting `create user|file|project …`) at F = 1 000 it is
**1.5 – 4x faster** than E4b. At F ≤ 10 it is within ±10 % of A. Grammar-compilation overhead: **+5 % to +30 %** of
`Library::new` at F = 1 000. The ceiling on benefit is the matched-path cost.

**Decision rule.** Adopt only with ≥ 15 % over architecture A on a cell that represents a real library **and** the differential test clean **and** a documented size/complexity cost **and** the E4b library-size rule.

#### E12 — Progressive tree with active states (ordered choice and ambiguity)

**Hypothesis.** E11 handles the deterministic case. Where several candidates share a prefix and diverge later, or where
capture-first and literal-first functions both stay alive, advancing a **set of active states** token by token avoids retrying
the shared prefix per candidate, and detects a no-match at the first divergence instead of after every candidate has failed.

**Steps**

1. Build on E11's trie. Each state holds `(candidate_index, element_index)` threads kept in the **global sorted order**.
2. On each token, advance every thread: a literal thread survives if its literal matches (with the punct-prefix rule from
   E4b); a single-token capture thread (`ident`, `int`, `string`, `raw`) advances one step. Candidates that need multi-token
   captures (`any`, `word`, `tail`) or nonterminals are **interpreted candidates**: they are not threads, they run through
   the existing matcher from the statement start.
3. **Ordered-choice rule (revised in revision 3).** A thread for candidate *i* that accepts may be committed only when
   (a) every thread with a smaller index has died, **and** (b) every *interpreted* candidate with a smaller index than *i*
   has been run by the existing matcher at the statement start (position saved and restored) and **did not match**, or has been
   excluded by proof (R17 (b)). If an interpreted candidate with a smaller index matches, **it wins**, and the tree's accept
   is discarded. Revision 2 ran interpreted candidates only if nothing earlier had accepted, which let a lower-priority tree
   match pre-empt a higher-priority unsupported candidate. This preserves "first match in sorted order wins".
4. Store threads in one fixed-capacity buffer owned by the parser and reused across statements (no `SmallVec`, ARCH-002).
5. **Failure.** When all threads die, record the furthest token index and the expectation set from the threads that died there,
   in the same representation E2 uses, so the final diagnostic is byte-identical (T-05).
6. Run the differential test (T-13: tree result == linear scan) and the diagnostics corpus.

**Correctness risks.** Committing a lower-priority accept too early; a thread that died by consuming a split punct token (E8
dependency: the cursor must be the split-offset form); an interpreted candidate evaluated out of order or not at all.

**Measurement.** Matrix cells with prefix sharing ∈ {moderate, heavy} × outcome ∈ {valid, invalid, ambiguous}; parse µs/line,
allocations/line, peak live bytes, compilation time.

**Expected result.** On valid, unambiguous input: within ±20 % of E11. On **invalid** input (no candidate matches): **1.2 – 2x faster**
than E11, since failure is detected at the first divergence. On **ambiguous** input (≥ 2 candidates share a long prefix): the case
where it should show its value, **1.3 – 3x faster** than E4b at heavy sharing. Peak live bytes: **+0 – 2 %** (one reusable buffer).

**Decision rule.** As E11, plus the ordered-choice differential test clean on a generated library with ≥ 30 % ambiguous statements **and** with at least 20 % of its candidates unsupported (interpreted) and placed **both above and below** supported ones in priority order.

#### E13 — Fused scanner (bytes → matches, no Token vector)

**Hypothesis.** For a script in which every statement is a single-line match of an eligible shape, the `Vec<Token>` (625 001 tokens
on c100k) and the token cursor are pure overhead. Recognising shapes directly over the bytes and recording capture **byte
ranges** removes the lex stage and most parse allocations.

**Step 0 (before any code).** Read `make_lexer.rs` for every rule the scanner must replicate, and list the ones still unknown:
identifier characters (`is_ident_start`/`is_ident_part`, `make_lexer.rs:470-475`), number syntax, string escapes, punct merging, comment
prefixes (`has_comment_prefix`, `:255`), `\r\n` handling (`split_lines`, `:202`), backtick merging (`:216`), tab = 4 for indent but
1 for column. **Inline** (end-of-line) comments: not yet verified — a step-0 finding that decides whether `tail` is eligible.

**Exclusion proof (the scanner may not guess).** An ineligible function (a block, a nonterminal, a `when_*`, an `any`/`word` first capture, …) is *provably unable to match* a line only if (i) its first element is a literal and the line's first token cannot begin with it — using the same punct-prefix rule as E4b — or (ii) its first element is a single-token capture of a type whose acceptable first-token kind (the E4c table) excludes the line's first token. Anything else is "may match". If an ineligible function that **may match** stands ahead of the candidate the scanner would accept, the scanner does not decide that line: it hands off (below). If **no** candidate matches a line, the scanner does not report an error either; it hands the line to the general path, which produces the diagnostic.

**Eligibility (conservative; anything else → handoff).**

| Level | Eligible when |
|---|---|
| function | elements are only literals and captures of type `ident`, `int`, `float`, `bool`, `string`, `raw`, or a **last** `tail`; no `block_*`, `when_*`, `sep`/`join`, optional/default, no nonterminal capture |
| library | no `preprocess`; comment markers known. A library may contain **ineligible** functions only if, for each line the scanner decides, every ineligible function ahead of the accepting one is excluded by proof (rule below) |
| script | no `define` block; every line at indent 0; no open bracket across lines; no backtick (multi-line merge); no tab indentation |

**Steps**

1. At library load, compile each eligible function to ops (`Lit`, `CapIdent`, `CapInt`, `CapString`, `CapRaw`, `CapTail`, `Accept`) and
   build the first-literal index (E4b logic, on bytes).
2. Share the lexer's character-class and string-escape helpers with the scanner (one source of truth), so the two cannot drift.
3. Scan line by line using the lexer's own line splitting and comment skipping. For each line pick candidates in global order and run
   the ops. A `Lit` matches only if the bytes equal the literal **and** the next byte is a token boundary by the lexer's rules (so `sayx`
   does not match `say`, while `say"hi"` still does, exactly as today).
4. Record matches as `Match { func, caps: [(start, end); n], line }` in a reused buffer.
5. **E13 alone** builds today's `FuncCall` from each `Match` and hands it to the unchanged evaluator. **E14** skips even that.
6. **Prefix handoff (revised in revision 3; replaces the whole-script restart).** At the first line the scanner cannot decide with proof
   — an ineligible line, a possible match by an unsupported candidate, no match, or any condition not covered above — it stops. The matches already
   found for the **leading run of lines are kept**, and the general path continues from that line to the end of the script; the two results are
   joined in order. The scanner never restarts the script and never re-enters once the general path has taken over.
   The handoff is sound only at a clean boundary, so it requires: the line starts a top-level statement at indent 0, no bracket or backtick is open, and the
   general lexer is started at that line with **absolute** line numbers (an internal `tokenize_from_line` entry; the parser still receives the full
   source lines for verbatim slices). `define` blocks are detected up front (`extract_defines` already runs over the whole source first); a script that
   contains one never uses the scanner.
7. For **E13 alone** the joined result is a list of `FuncCall`s, evaluated in order by the unchanged evaluator. For **E14** the handoff is sound because
   eligible bodies only `write` and never touch `context`: at the boundary the state is exactly "output so far, empty context", which the general path
   continues from.

**Correctness risks.** Divergence from lexer rules (identifier boundaries, numbers like `1.5`, which `int` accepts today; punct
merging; string escapes; CRLF; BOM; non-ASCII; columns counted in bytes); priority order; `tail` re-spacing and re-quoting (`requote`,
`token_source_text`); line numbers in spans. Mitigations: shared helpers, the differential fuzz test (T-15: mutated lines over the
library's alphabet plus every shipped sample), and the equality gate (R5).

**Step 0 additions (verify before any code).** (i) Whether evaluation can observe `leading_comments` — searched: `make_evaluator.rs` and
`inner_evaluator.rs` never read it, so it is observable only through the `ast` surfaces (R19). (ii) Templates **can** read the `line` and `col` locals
(`make_evaluator.rs:206-207`), so E14 must supply them from `Match`, or treat such a template as ineligible.

**Measurement.** End to end and per stage; decidable-statement coverage on the shipped samples; **handoff overhead** with one undecidable line placed at
0 %, 50 % and 99 % of a 100 000-line script (and, adversarially, every k-th line); fast-path line share; allocations/line; peak live bytes.

**Expected result.** E13 alone (still building `FuncCall`): `t_import` end to end **342 → 110 – 170 ms** (central 140 ms): the 63 ms lex and most of the
228 ms parse are replaced by roughly 30 – 60 ms of scanning and `FuncCall` construction, and the 58 ms evaluate stage remains. Handoff cost: the scanned prefix is **kept**, so only the one failed statement is wasted. With an undecidable line at 0 % of the script, total time is **within +2 %** of the general path; at 50 % the run is **1.3 – 1.5x faster** than the general path; at 99 % it is **2.0 – 2.6x faster** (derived from the 140 ms vs 342 ms prefix cost). Share of shipped *libraries* in which every statement shape is decidable with proof: **20 – 50 %**.

**Decision rule.** Adopt only if (a) T-15 is clean on ≥ 100 000 generated statements and every shipped sample, (b) the gain is ≥ 2× the end-to-end
spread on a workload representative of shipped libraries, and (c) with one undecidable line at any position (0 %, 50 %, 99 %) the run is never more than 2 % slower than the general path.

#### E14 — Specialised execution (compiled render, no `FuncCall`)

**Hypothesis.** For eligible functions whose body is only `write` of a template whose `${…}` bodies are plain capture names, rendering can be a
flat op list — `Emit(static)`, `EmitCapture(i)` — executed straight from byte ranges into the output buffer. That skips `CaptureValue`, the capture
`BTreeMap`, `FuncCall` and template interpretation. E6 is the general form of the template half of this; E14 is the fused form.

**Steps**

1. Classify bodies at load: eligible if the body consists only of `write` statements with templates containing only literal text, escapes, and
   `${name}` where `name` is a capture of this function, or `${line}` / `${col}` (supplied from `Match`; they are render locals today, `make_evaluator.rs:206-207`). Anything else (helpers, pipes, `set`, `append`, `if`, `for`, `context`) is ineligible.
2. Compile to ops with the escape semantics of `interpolate_render` (`inner_evaluator.rs:506`: backslash-n, -t, -r, -backtick, -backslash, and backslash-X for any other character), confirmed against the golden corpus.
3. Capture rendering rules to replicate: a `string` capture renders **with** its quotes (observed: `msg = "\"hello 1\""` renders `print("hello 1")`),
   `int`/`ident`/`raw` as their text, `tail` as its re-spaced text.
4. Emit into the same `body` buffer the evaluator uses, so `file_template` (which wraps `body` and `context`) runs unchanged. A library whose
   `file_template` reads `context` populated by ineligible functions never uses E14 (the handoff rule cannot carry that state).
5. **E14 is an execution path, not a parse path (R19).** It exists only inside `run` / `run_multi`. `capy ast`, `capy ast --json`, `parse` and
   `parse_recovering` never go through it and return the identical full tree: spans, `leading_comments`, error nodes, diagnostics. T-19 compares
   `ast --json` with E13/E14 enabled against the baseline on every corpus and shipped sample.
6. Equality gate and differential test as for E13.

**Correctness risks.** Escape edge cases; indentation/blank-line behaviour of `write` templates; interaction with `body` for nested blocks
(excluded — no blocks in the eligible subset).

**Measurement.** `t_import`, `t_say`, `t_assign`, `t_all` (the last mixes eligible and ineligible functions, exercising the exclusion proof and the handoff); end to end,
evaluate stage, allocations/line.

**Expected result.** E14 stacked on E13, `t_import`: **342 → 40 – 90 ms** (central 60 ms), i.e. **≈ 4 – 8x the regex time** (11 ms) and **4 – 8x faster than today**.
`t_all` (includes `other`, a `tail` catch-all with no `write`): **500 → 60 – 130 ms** if every statement is decidable with proof, otherwise the time at which the handoff occurs plus the general-path time for the rest.
**This is the least certain prediction in the document** — no prototype exists, the range is deliberately wide, and it is scored but carries low confidence.

**Decision rule.** Adopt only with E13's rule and when the compiled and interpreted renders are byte-identical on the golden corpus.

#### E15 — Memoised matcher (gated by a repetition measurement)

**Hypothesis.** Caching a shape's result by `(function, token position)` helps only if the parser re-attempts the same pair. On a flat grammar each
position is tried once per candidate, so a cache can only add cost. Repetition may occur when a block header matches, its body fails, and the parser
**backtracks and parses the body again for another candidate** (`block_err` path, `make_parser.rs:~385-470`), and inside nonterminal alternation
(`PROP-2026-0004`).

**Steps**

1. **Measure first, build nothing.** In a copy of `base`, count `try_match` calls keyed `(function, start position)` per statement and across nested
   block bodies; report repeated attempts ÷ total attempts for the benchmark corpora and for every shipped sample with blocks or alternation.
2. **Gate:** build the cache only if the repetition rate is **≥ 25 %** on at least one shipped sample.
3. If built: a per-statement `HashMap<(u32, u32), Outcome>`, bounded and cleared per statement. `Outcome` stores either a failure **with its recorded
   expectation** (so `furthest` and diagnostics are unchanged) or the matched `FuncCall` and end position. It caches a shape's result at a position, never a
   *choice*, so ordered-choice semantics are untouched.
4. The key must include the E8 split offset; E15 depends on E8.

**Correctness risks.** Stale entries after cursor mutation (only sound after E8); diagnostics; memory growth (bounded per statement).

**Measurement.** Repetition rate (step 1); if built, time and peak live bytes on the cells where repetition is high **and** on flat grammars (to quantify the cost).

**Expected result.** Repetition rate: flat benchmark libraries **< 5 %**; shipped samples with blocks or alternation **5 – 40 %**. Gate met on at most a few samples.
If forced on a flat grammar, the cache is **10 – 30 % slower** (hashing per attempt exceeds the work saved). **Predicted verdict: not built.**

**Decision rule.** Adopt only where the gate holds on a shipped sample and the cache is net faster there without slowing flat grammars by more than 3 %.

#### S1 — The stack, and the interaction (architecture A)

**Hypothesis.** Gains are not additive. E2 and E3 overlap (both remove failed-attempt allocation); E4b's gain shrinks once
failures are cheap; E5 and E1 both remove token clones.

**Steps**

1. Order by decreasing **marginal** value measured so far: E8, E7, E1, E2, E3, E9, E5, E6, E4a, E4b, E4c. This is **architecture A**. Architectures B and C are built on the same E8 + E1–E3 foundation, not on E4b.
2. Apply each to the previous cumulative variant (`S1.1 = E8`, `S1.2 = S1.1 + E7`, …), running the equality gate and the
   whole benchmark set after every addition.
3. Record the cumulative curve and each step's **marginal** gain; compare with the isolated gain from the E-experiment
   (the ratio is the interaction factor).

```text
   cumulative end-to-end (c100k, t_import), predicted        isolated gains sum ≠ cumulative gain

   baseline  ████████████████████████████████████  342 ms
   +E8,E7    ███████████████████████████████████░  ~342     (neutral here; E7 matters for contains)
   +E1       ██████████████████████████████░░░░░░  ~305
   +E2       ██████████████████████░░░░░░░░░░░░░░  ~235
   +E3       ████████████████████░░░░░░░░░░░░░░░░  ~220
   +E9       ███████████████████░░░░░░░░░░░░░░░░░  ~215
   +E5       ███████████████░░░░░░░░░░░░░░░░░░░░░  ~170
   +E6       ██████████████░░░░░░░░░░░░░░░░░░░░░░  ~155
   +E4*      ██████████████░░░░░░░░░░░░░░░░░░░░░░  ~150     (tiny: 4-function library)
   (predicted: 342 → 130–170 ms, i.e. 2.0–2.6x;  regex 11 ms)
```

**Expected result.** End to end on `t_import`: **342 → 130–170 ms (2.0–2.6x)**; `t_all`: **500 → 190–250 ms (2.0–2.6x)**;
`contains`: **3 386 → 150–220 ms (15–22x)**. The combined gain is predicted at **≈ 75 % of the product** of the isolated gains
(interaction factor ≈ 0.75). Ratio to regex improves from ≈ 30x to **≈ 12–15x**.

**Decision rule (revised in revision 3).** **Every independent optimisation is tested**, in S1's order, whatever an earlier one produced; a method below the line does not stop the ones after it. The adoption list is every method whose **marginal** gain at its position is ≥ 2× the spread of the quantity it targets. Because marginal gain depends on order, the final stack is then checked **leave-one-out**: removing any adopted method must cost ≥ 2× spread, or it is dropped from the list.

## Expected Code and Documentation Changes

| ID | Path | CRUD | Symbol / Region | Exact Planned Edit | Reason | Change IDs | Requirement IDs | Dependencies | Test / Documentation Impact |
|---|---|---|---|---|---|---|---|---|---|
| F-01 | `program_docs/proposals/prop-2026-0007-parser-and-pipeline-speed-experiments.md` | CREATE | whole file | this document | G-01…G-06 | C-15 | R13 | — | — |
| F-02 | `program_docs/index/document-index.md` | UPDATE | document table, status table, next-ID table, change history | add `PROP-2026-0007` (draft); next `PROP` → 0008; revision 7 | `DOCUMENTATION.md` §17 | C-15 | R13 | F-01 | index check |
| F-03 | `.ignore/speedlab/**` | CREATE | sandbox, harness, scripts, patches | as §Sandbox; **untracked** | experiments | C-01…C-20 | R1, R4, R15 | approval | T-01…T-12 |
| F-04 | `.ignore/speedlab/v/E1/rust/src/orchestrator/features/make_parser.rs` | UPDATE (sandbox) | `peek` (:255), `try_match_inner` comma skip, `match_literal` (:1204), `at_statement_end` | add `peek_kind`/`peek_text`; replace clone-compare sites | E1 | C-05 | R5 | F-03 | T-03 |
| F-05 | `…/v/E2/…/make_parser.rs` | UPDATE (sandbox) | `ContextFrame` use in `try_match` (:956), `note_failure` (:905), literal-failure arm of `try_match_inner` (:969), `furthest_error` | lazy frames and expectations; message built at the end | E2 | C-06 | R5 | F-03 | T-03, T-05 |
| F-06 | `…/v/E3/…/make_parser.rs` | UPDATE (sandbox) | `try_match_inner` element clone, capture map, `next_literals` (:1872) | borrow elements; slot `Vec`; map on success | E3 | C-07 | R5 | F-03 | T-03 |
| F-07 | `…/v/E4a,E4b,E4c/…/make_library_loader.rs`, `…/domain/library.rs`, `make_parser.rs:76-103` | UPDATE (sandbox) | loader tail; `Library` new additive fields; `parse_impl` setup; `parse_stmt` candidate loop | sorted list + index at load; merge-by-index iteration | E4 | C-09 | R5, R8, R9, R10 | F-03 | T-03, T-07 |
| F-08 | `…/v/E5/…/make_lexer.rs`, `…/domain/token.rs`, `make_parser.rs` | UPDATE (sandbox) | `tokenize_impl` (:70) new internal sibling; new `STok`/`Lexed`; parser cursor | source-backed tokens; wrapper keeps `tokenize_with_trivia` | E5 | C-10 | R5, R8 | F-03 | T-03, T-04 |
| F-09 | `…/v/E6/…/inner_evaluator.rs` | UPDATE (sandbox) | `interpolate_render` (:506), `eval_interp` (:566), `render_stmt` | `CompiledTemplate`, lazy compile + cache | E6 | C-11 | R5 | F-03 | T-03 |
| F-10 | `…/v/E7/…/inner_evaluator.rs` (`:823-832`), `make_evaluator.rs:394-406`, `define_extractor.rs` | UPDATE (sandbox) | `"regex_match"` arm (:823); `pattern` validation; extractor | bounded `thread_local!` regex cache | E7 | C-04 | R5, R9 | F-03 | T-03, T-06 |
| F-11 | `…/v/E8/…/make_parser.rs` | UPDATE (sandbox) | `match_literal` (:1204-1240), `save`/`restore` (:275-281), `peek`/`advance` | split-offset cursor; d8 test | E8 | C-03 | R5, R12 | F-03 | T-08 |
| F-12 | `…/v/E9/…/make_parser.rs` | UPDATE (sandbox) | `parse_impl` head (:46-76) | skip the comment pass when none was lexed | E9 | C-08 | R5 | F-03 | T-03 |
| F-17 | `…/v/E11,E12/…/make_parser.rs`, `…/domain/library.rs`, new `…/features/compiled_matcher.rs` | CREATE + UPDATE (sandbox) | trie/threads; additive `Library` field; candidate selection in `parse_stmt` | deterministic compiled matcher and active-state tree; interpreted candidates run through the existing matcher in global order (R17 (b)) | E11, E12 | C-13, C-16 | R5, R9, R17 | F-03, F-11 | T-03, T-05, T-13 |
| F-18 | `…/v/E13,E14/…/capy.rs` (`Library::run_multi`), `…/features/make_lexer.rs` (new internal entry), new `…/features/fast_path.rs` | CREATE + UPDATE (sandbox) | eligibility check and dispatch at the top of `run_multi`; scanner; compiled render | fused scanner and flat emit ops; exclusion proof; prefix handoff; internal `tokenize_from_line` in `make_lexer.rs` so the general path continues from an absolute line | E13, E14 | C-17, C-18 | R5, R17, R19 | F-03 | T-03, T-04, T-15, T-19, T-20 |
| F-19 | `…/v/E15/…/make_parser.rs` | UPDATE (sandbox) | `try_match` counter keyed `(func, pos)`; optional memo table | repetition measurement first | E15 | C-19 | R5 | F-03, F-11 | T-16 |
| F-20 | `.ignore/speedlab/harness/src/bin/{matrix,fuzz}.rs`, `corpora/matrix/**` | CREATE (sandbox) | matrix runner, differential fuzzer, corpus generators | R16 cells; peak-bytes allocator; compile-time split | G-07 | C-20 | R16, R17, R18 | F-03 | T-14, T-15, T-17, T-18 |
| F-13 | `program_docs/testing/test-2026-0014-parser-speed-experiments.md` | CREATE | whole file | every sample, the verdict table, the prediction-vs-actual table | R13 | C-15 | R2, R13 | experiments done | — |
| F-14 | `program_docs/decisions/adr-0005-…` | CREATE (per adopted method) | whole file | one ADR per method the ranking rule adopts | G-05 | C-15 | R14 | F-13 | — |
| F-15 | `program_docs/standards/index.md` | READ | rule index | confirm revision/rules cited | §21.2 | — | — | — | — |
| F-16 | `rust/src/orchestrator/features/make_parser.rs`, `make_lexer.rs`, `inner_evaluator.rs` | READ | symbols listed above | discovery input for the sandbox patches (already read) | feasibility | all | — | — | — |

Counts: **2 tracked CREATE** (F-01, F-13, plus F-14 per adoption), **1 tracked UPDATE** (F-02), **0 tracked source edits**; sandbox
edits F-03…F-12 and F-17…F-20 are untracked and throwaway. No path is invented: every `rust/` path and symbol above was read at `75a9f68`.

Illustrative focus for F-11 (not authorized code):

```rust
// before (make_parser.rs:1226-1236): mutates shared state, lost on restore()
let mut rem = t.clone(); rem.text = t.text[lit.len()..].to_string();
self.toks[self.pos] = rem;  return true;

// after (illustrative): cursor state, restored with pos
struct Cur { pos: usize, split: usize }           // split = bytes of toks[pos] already consumed
fn save(&self) -> Cur { Cur { pos: self.pos, split: self.split } }
fn restore(&mut self, c: Cur) { self.pos = c.pos; self.split = c.split; }
fn peek_text(&self) -> &str { &self.toks[self.pos].text[self.split..] }
```

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected | Requirement Impact |
|---|---|---|---|---|
| **A. Measure each method in a sandbox first (this proposal)** | Ranks by evidence; cheap to discard a bad idea; catches interactions (S1) | Delays adoption; the sandbox patches are throwaway | **Selected** | R1–R15 |
| B. Build the idea file's biggest item (compiled matcher, E11) first | Highest ceiling on large F | Largest effort; unknown value at the F that actually ships (median 4, max 14); interacts with every Tier A change | Rejected as a *first* step; E11/E12 run ungated as architecture B | — |
| C. Implement all ideas directly in `rust/` behind commits | Faster to ship | No ranking, no interaction data, regressions land in the tree (GOAL-002 risk) | Rejected | R1, R5 |
| D. Rewrite as a table-driven/generated parser | Highest throughput | Abandons library-declared grammars (GOAL-001); `PROP-2026-0006` rejects it too | Rejected | GOAL-001 |
| E. Use a sampling profiler to find hot spots instead | Direct attribution | None installed; adding one is a dev tool, not a dependency, but still unavailable offline | Deferred: E0 uses allocation-site counters and stub bounds instead; revisit if a profiler is installed | R3 |
| F. Do nothing | Zero cost | Leaves 19–30x and 940x | Rejected | — |
| G. A fused scanner + compiled execution as a whole separate engine | Highest throughput on flat grammars | Duplicates the lexer's rules; two engines to keep equal; breaks `GOAL-002` if they drift | Selected only as a **prototype with an exclusion proof, a one-way prefix handoff and a differential test** (architecture C, R17) | R17, R19 |
| H. Memoise matching everywhere | Removes repeated work in ambiguous grammars | Adds hashing to every attempt; flat grammars repeat nothing | Gated by a repetition measurement (E15) | — |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback Action | Owner |
|---|---|---|---|---|---|
| RK-01 | Host noise makes small gains unmeasurable (already observed: +17 % end to end between two same-day sessions) | stage spread > 8 % in E0 | R3 stop rule; interleaving; quiet host | no ranking published; rerun | Capy Engine |
| RK-02 | A variant changes output or a diagnostic | R5 diff | equality gate before timing; failing-input corpus for E2 | variant `EXCLUDED` | Capy Engine |
| RK-03 | E8 reveals a grammar relying on the old behaviour | golden diff | stop the programme; report | keep `base` cursor; re-plan | Capy Engine |
| RK-04 | E2's lazy messages differ from today's | diagnostics corpus diff | build the message from the same recorded expectation at the end | drop E2 or fix before ranking | Capy Engine |
| RK-05 | E5 breaks spans (columns, widths, tabs, quotes) | `ast --json` diff | internal token type + wrapper; spans compared on all samples | drop E5 | Capy Engine |
| RK-06 | E6 moves a template error from first render to load | error-timing test | lazy compile on first render | drop E6 | Capy Engine |
| RK-07 | E7's cache grows without bound | memory growth on dynamic patterns | fixed capacity 64, clear-on-full (R9) | drop E7 | Capy Engine |
| RK-08 | Optimiser removes work in a stub experiment, inflating the upper bound | upper bound > E-experiment by > 2× | read the stub's output hash to confirm work was done; report as an upper bound only | discard that bound | Capy Engine |
| RK-09 | Synthetic libraries do not resemble real ones | E4 verdict contradicts shipped libraries | decision rule uses the **largest shipped library's F** | E4b `NOT SUPPORTED` for adoption | Capy Engine |
| RK-10 | Results lost because `.ignore/` is untracked | disk cleanup | R13: results and generator go into `TEST-2026-0014` and `rust/devtools/` (the latter only after approval) | recreate from the TEST document | Capy Engine |
| RK-11 | Working-tree edits by others change `base` | `git status` | `base` is a `git archive` of a fixed commit | none needed | Capy Engine |
| RK-12 | E13/E14 scanner diverges from the lexer on an edge case (identifier boundary, escapes, CRLF, BOM, columns) | T-15 fuzz mismatch; R5 diff | shared character-class helpers; exclusion proof; prefix handoff; fuzz ≥ 100 000 statements | drop C or narrow the eligibility rule | Capy Engine |
| RK-13 | A tree commits a lower-priority accept before a higher-priority thread finishes | T-13 mismatch on an ambiguous generated library | ordered-choice commit rule (E12 step 3) | drop E12 | Capy Engine |
| RK-14 | Grammar-compilation cost outweighs the saving for short scripts or editors | break-even runs high (> OQ-07 limit) | report compile time separately; keep compilation lazy; adopt only above the break-even | do not adopt B/C for that use | Capy Engine |
| RK-15 | The matrix is too large to run (cost explosion) | cell count × variants | one-factor sweeps + one cross (≈ 27 cells); corpus scaled so no cell > 60 s; omissions recorded | shrink the matrix, record it | Capy Engine |
| RK-16 | Peak-memory tracking perturbs timing | allocator overhead | measure timing and memory in separate runs | n/a | Capy Engine |
| RK-17 | The exclusion proof is unsound: the scanner or tree selects a lower-priority function while an unsupported higher-priority one would have matched | T-15 / T-13 mismatch on libraries with unsupported candidates above supported ones | "cannot prove" = "may match" = handoff; E12 runs lower-index interpreted candidates before any commit; the differential libraries deliberately place unsupported candidates both above and below | drop the offending variant | Capy Engine |
| RK-18 | The handoff boundary carries wrong state (indent stack, open bracket or backtick, `define`, line numbers, `body`/`context`) | T-20 mismatch; R5 diff | handoff only at an indent-0 statement start with nothing open; `define` detected up front; absolute-line lexer entry; E14 handoff only when bodies never touch `context` | narrow the boundary rule or drop E13/E14 | Capy Engine |
| RK-19 | An execution fast path leaks into `ast` / `parse` and changes the tree | T-19 diff | R19: execution fast paths exist only in `run` / `run_multi`; `ast --json` compared on every corpus | revert the dispatch | Capy Engine |

Rollback of this proposal: delete `.ignore/speedlab/`. No tracked file other than documents changes.

## Security Impact

None. The experiments run locally on checked-in libraries and synthetic corpora. E7 bounds a cache that could otherwise be
grown by crafted patterns (ARCH-003 spirit).

## Operational Impact

None for users. Maintainers gain a repeatable harness once `rust/devtools/` receives it (post-approval, via a plan).

## Compatibility Impact

None. No variant is adopted by this proposal. E8, if adopted, turns a spurious error into a match (additive, GOAL-002).

## Migration Requirements

None.

## Test and Validation Design

| ID | Type | UC / Requirement IDs | Scenario and Purpose | Environment / Data | Exact Procedure or Command | Expected Result | Test File / Evidence Destination |
|---|---|---|---|---|---|---|---|
| T-01 | measurement | UC-01, R1, R3 | Baseline and noise floor (E0) | base, c10k, c100k, six task libs | `scripts/speedlab.sh run base --iters 15` | stage spread ≤ 5 % (stop at 8 %); end-to-end spread recorded | `TEST-2026-0014` |
| T-02 | measurement | UC-01, R4, R11 | Interleaved variant runs | every variant | `scripts/speedlab.sh run all --iters 15` | all samples written; table printed | `TEST-2026-0014` |
| T-03 | regression | UC-02, R5, R6 | Output and `ast --json` equality | all `samples/*/lib.capy` + scripts, corpora | `scripts/speedlab.sh verify <variant>` | 0 differences | `TEST-2026-0014` |
| T-04 | regression | UC-02, R5 | Span equality for E5 | 20 scripts with tabs, quotes, templates, comments | `capy ast --json` base vs E5 | identical | `TEST-2026-0014` |
| T-05 | failure | UC-02, R5 | Diagnostics identical (E2) | 1 000 malformed scripts derived from golden error cases | `capy run` base vs E2, compare stderr | identical bytes | `TEST-2026-0014` |
| T-06 | unit | R9 | E7 cache bounded | 10 000 distinct dynamic patterns in one run | run `t_dynamic_regex`; read RSS and cache length | length ≤ 64; no growth | `TEST-2026-0014` |
| T-07 | measurement | UC-03, R10 | Cost vs library size | synthetic libraries F = 4, 10, 100, 1 000, 10 000 | `scripts/speedlab.sh scale` | µs/line per F per variant | `TEST-2026-0014` |
| T-08 | regression | UC-06, R12 | d8 reproduction | `.ignore/regex-compare/demos/d8.*` | `capy run d8.capy d8.txt` on base and on E8 | base: error; E8: `B foo`; reverting E8 re-fails | `TEST-2026-0014` + rust test (post-approval) |
| T-09 | integration | R6 | Full test suite per ranked variant | each variant | `cargo test --manifest-path <variant>/rust/Cargo.toml --workspace` | passes (baseline: 131 passed, 8 skipped, 0 failed for golden, per `PROP-2026-0006` T-04) | `TEST-2026-0014` |
| T-10 | architecture | R7, R8 | Dependency and API surface | each variant | `cargo tree -p capy-core --depth 1`; `diff` of `pub` item lines on changed files | 1 dependency; additions only | `TEST-2026-0014` |
| T-11 | measurement | R11 | wasm size per method | each ranked variant | wasm build per `rust/devtools/wasm_check.sh`; `ls -l` | recorded; ≤ +8 KiB proposed | `TEST-2026-0014` |
| T-12 | evidence | UC-04, UC-05, R2, R13 | Prediction vs actual | frozen §Frozen Predictions | `scripts/speedlab.sh report` | every row scored hit/miss; each miss > 2× explained | `TEST-2026-0014` |
| T-13 | differential | E11, E12, R17 | Trie / active-state result == linear scan, including ≥ 30 % ambiguous statements | generated library, 10 000 statements | randomised differential test | 0 mismatches | `TEST-2026-0014` |
| T-14 | measurement | UC-07, R16 | Matrix cells for A, B, C | R16 corpora | `scripts/speedlab.sh arch --matrix full` | every cell run or omitted with a reason; all metrics filled | `TEST-2026-0014` |
| T-15 | differential | UC-07, R17 | Fast path (E13/E14) == token matcher, including libraries with unsupported candidates **above and below** supported ones | ≥ 100 000 mutated lines over each library's alphabet + all shipped samples | `…/fuzz` | 0 mismatches in match/no-match, captures, output, `ast --json` | `TEST-2026-0014` |
| T-16 | measurement | E15 | Repetition rate of `(function, position)` attempts | benchmark corpora + every shipped sample with blocks/alternation | instrumented `try_match` copy | rate per sample; gate ≥ 25 % | `TEST-2026-0014` |
| T-17 | measurement | UC-07, R18 | Architecture comparison table | A, B, C | `scripts/speedlab.sh arch` | one table, identical corpora and metrics | `TEST-2026-0014` |
| T-18 | measurement | UC-07, R16 | Compile time vs execution time; break-even runs | F = 4 … 10 000, 15-line and large scripts | `…/matrix --compile-split` | compile and run reported separately; break-even computed | `TEST-2026-0014` |
| T-19 | regression | UC-07, R19 | `ast --json` and `parse` unchanged with every variant enabled | all corpora + all shipped samples, including scripts with comments and malformed lines | `capy ast --json` base vs each variant | byte-identical (spans, `leading_comments`, error nodes, diagnostics) | `TEST-2026-0014` |
| T-20 | measurement | UC-07, R17 | Handoff overhead and correctness | 100 000-line script with one undecidable line at 0 %, 50 %, 99 %, and at every k-th line; libraries with unsupported higher- and lower-priority candidates | `…/matrix --handoff` | output identical to the general path; total time within the thresholds in §Frozen Predictions | `TEST-2026-0014` |

QUAL-003: T-08 and T-05 are the tests that must **fail** for the right reason when the fix is reverted. T-03 guards behaviour that must
not change, so it passes before and after by design.

### Measurement and Validation

Per `DOCUMENTATION.md` §21.3 and `QUAL-002`. Thresholds below are the **acceptance thresholds** (when a method is `SUPPORTED`); the
**predictions** are in §Frozen Predictions and are scored separately. Both freeze at approval.

| Measurement | Baseline Method | Test Command or Procedure | Controlled Environment | Acceptance Threshold | Report Destination |
|---|---|---|---|---|---|
| M-01 end-to-end `Library::run`, c100k, 6 tasks | best-of-15 **and** median of 15, interleaved | `scripts/speedlab.sh run all --iters 15` | macOS arm64 Darwin 25.4.0, release, `caffeinate`, no other build running, plugged in | a method is `SUPPORTED` if its target-stage gain ≥ 2× the E0 spread of that stage **and** ≥ the method's own decision-rule floor | `TEST-2026-0014` |
| M-02 stage split (lex / parse / eval) | `phases` bin, counting allocator | `scripts/speedlab.sh run all --phases` | same | same | same |
| M-03 allocations per line per stage | counting `GlobalAlloc` | same | same | reported; no threshold | same |
| M-04 scaling in library size | µs/line at F = 4, 10, 100, 1 000, log-log slope | `scripts/speedlab.sh scale` | same; shape mix per R10 | E4b `SUPPORTED` only if slope drops from ≈ 1 to ≤ 0.3 **and** the largest shipped F benefits (E4b decision rule) | same |
| M-05 correctness | byte equality of output and `ast --json` | `scripts/speedlab.sh verify` | same | 0 differences (E8 on d8 excepted) | same |
| M-06 wasm size | `ls -l` of the wasm artefact | `rust/devtools/wasm_check.sh` then `ls -l` | same toolchain | ≤ +8 KiB per method (proposed; owner to confirm, OQ-03) | same |
| M-08 grammar-compilation time | `Library::new` timing, F = 4 … 10 000, 15 samples | `…/matrix --compile-split` | same | reported with break-even runs; adoption limit per OQ-07 | same |
| M-09 peak live bytes | counting allocator tracking live bytes | `…/matrix` (separate run from timing, RK-16) | same | reported; no threshold | same |
| M-10 fast-path coverage and line share | decidable statements per shipped library; per-cell fast-path line share for C | `…/matrix`, E0 step 6 | same | reported | same |
| M-07 prediction accuracy | hit if the measured value is within ±1/3 of the predicted range's width of the central value | `scripts/speedlab.sh report` | — | none; the miss rate is a result | same |

All samples are reported, including the first (cold) iteration. The first iteration of each cell is flagged and **excluded
from min/median** but shown, because instruction-cache and allocator warm-up are real effects (the existing `nativebench` and
`TEST-2026-0008` report the first-run outlier the same way).

### Ranking Rule

```text
  for each method m with target stage s:
      gain(m)      = (median_baseline(s) − median_variant(s)) / median_baseline(s)
      supported(m) = gain(m) ≥ max( 2 × spread_E0(quantity measured) , floor(m) )          floor from m's decision rule
                     AND output + ast-json identical (R5)
                     AND golden/tests pass (R6)
      rank(m)      = gain(m) of the END-TO-END time on the mean of {t_import, t_say, t_assign, t_all}
                     (E7 ranked on t_contains; E4* ranked on the F=100 case — two separate leaderboards)
      adopt list   = every m whose marginal gain at its position in the S1 order ≥ 2 × spread
                     (no cut-off: a weak method does not stop later ones)
                     then leave-one-out on the final stack: removing m must cost ≥ 2 × spread

  for the architectures (A, B, C):
      preferred(X over Y) on a cell  =  throughput gain > 2 × spread
                                         AND identical observable behaviour (R5)
                                         AND, for C, fast-path line share reported for that cell
      choice  = the architecture preferred on the cells that represent libraries the project has (OQ-01),
                subject to break-even runs (compile-time increase ÷ per-run saving) within OQ-07
```

### Benchmark Matrix (R16)

```text
  axis               levels
  ─────────────────  ───────────────────────────────────────────────────────────────
  library size F     4 · 10 · 100 · 1 000 · 10 000
  prefix sharing     unique · moderate (2–5 functions per first literal) · heavy (20+)
  captures           fixed-length · variable-length (any/word/tail) · nonterminal/recursive
  input              15-line script · large file (100 000 lines) · nested blocks
  outcome            valid · invalid (no match) · ambiguous (≥ 2 candidates share a long prefix)

  design:  centre point = F 100 · moderate · fixed · large file · valid
           one-factor sweeps around it  (≈ 12 cells)
           full cross  F × prefix sharing at the centre of the other axes  (15 cells)
           = ≈ 27 cells per variant; F = 10 000 only for A, B, C, E4*, E11, E12
```

Corpus size scales down as F grows so that **no baseline cell exceeds 60 s** (at F = 1 000 and 10 000 each failing candidate costs real time on every
line, so a 100 000-line corpus is infeasible there; the line count is recorded per cell). Metrics per cell: throughput (lines/s), 15-line latency,
allocations/line, **peak live bytes**, grammar-compilation time (`Library::new`), generated-plan size, and — for C — fast-path line share. Compilation time and execution time are reported
separately, with the break-even number of runs.

Cells deliberately **omitted** are listed with the reason in the `TEST` document (R16); an omission is a result, not a gap.

### Frozen Predictions

Frozen at approval (R2). Gains are reductions in the named quantity versus the `75a9f68` baseline; negative = faster.
"central" is the single number used to score a prediction as hit (within ±1/3 of the stated range's width) or miss.

| ID | Quantity | Baseline (measured 2026-10-08) | Predicted range | Central | Basis |
|---|---|---|---|---|---|
| E0 | run-to-run spread, stage level (lex/parse/eval) | parse 228 vs 221 ms across two sessions (3 %) | ≤ 5 % (stop line 8 %) | 3 % | two observed sessions |
| E0 | run-to-run spread, end-to-end `Library::run`, c100k | `import` 342 vs 401 ms across two sessions (17 %) | 5 – 15 % | 8 % | the observed swing |
| E0 | failed-candidate share of parse allocations | not yet measured | 40 – 60 % | 50 % | P-04 site list |
| E0 | "free failure" upper bound, parse | 228 ms | −35 to −55 % | −45 % | P-04 |
| E8 | d8 result; golden diffs; time | error; n/a | `B foo`; 0; ±3 % | — | design |
| E7 | `contains` end to end | 3 386 ms | 300 – 460 ms | 380 ms | pipeline ≈ 350 ms + 3.6 ms regex |
| E1 | parse, `t_import` | 228 ms | −10 to −20 % | −15 % | P-05; free-peek bound |
| E2 | parse, `t_import` | 228 ms | −25 to −40 % | −30 % | P-04 |
| E2 | parse allocs/line | 39 | 18 – 24 | 21 | P-04 |
| E3 | parse, isolated | 228 ms | −8 to −15 % | −11 % | 3 allocs per element visited |
| E9 | parse, comment-free c100k | 228 ms | −2 to −6 % | −4 % | one pass over 625 001 tokens |
| E4a | `Library::run`, 15-line script, F = 4 | not yet measured | −5 to −15 % | −8 % | per-run clone+sort |
| E4a | `Library::run`, 15-line script, F = 1 000 | not yet measured | −60 to −90 % | −80 % | O(F log F) + F deep clones |
| E4b | parse µs/line at F = 100 | not yet measured (est. 30–60) | ≥ 8x faster (isolated) | 12x | linear-in-F failed attempts |
| E4b | parse µs/line at F = 1 000 | not yet measured (est. 300–600) | ≥ 50x faster (isolated) | 100x | same |
| E4b | marginal gain at F = 100 after E2 | — | 2 – 4x | 3x | E2 makes a failed attempt cheap |
| E4c | parse, `t_all`, on top of E4b | 288 ms | −3 to −8 % | −5 % | one kind-restricted wildcard |
| E5 | lex stage | 63 ms | −40 to −60 % | −50 % | 11 allocs/line → 2–4 |
| E5 | end to end `t_import` | 342 ms | −15 to −22 % | −18 % | lex −50 % + parse −7 % |
| E6 | evaluate, `t_all` | 149 ms | −30 to −50 % | −40 % | P-07 |
| E6 | evaluate, `t_import` | 58 ms | −20 to −35 % | −27 % | P-07 |
| E10 | AST+span share of parse | not yet measured | 10 – 20 % | 15 % | allocation sites |
| E0 | fast-path-eligible share of shipped functions | directive scan: ≈ 84 % carry no `block_*`/`when_*`/`priority` (544 functions, 123 libraries) | 50 – 75 % | 62 % | directive scan is an upper bound; capture types not yet checked |
| E0 | functions per shipped library | min 0, median 4, max 14 (measured) | n/a — a measured fact, not a prediction | — | counted 2026-10-08 |
| E11 | parse time vs **E4b**, unique first literals, F = 100 | not yet measured | 0.8 – 1.3x of E4b (no faster) | 1.0x | one-entry hash bucket is already minimal |
| E11 | parse time vs E4b, heavy shared prefixes, F = 1 000 | not yet measured | 1.5 – 4x faster | 2.5x | shared steps walked once |
| E11 | `Library::new` overhead at F = 1 000 | not yet measured | +5 to +30 % | +15 % | trie build is O(total elements) |
| E12 | invalid-input parse time vs E11 | not yet measured | 1.2 – 2x faster | 1.5x | failure detected at first divergence |
| E12 | ambiguous-input parse time vs E4b, heavy sharing | not yet measured | 1.3 – 3x faster | 2x | prefix not retried per candidate |
| E13 | end to end `t_import`, scanner + `FuncCall` + evaluator | 342 ms | 110 – 170 ms | 140 ms | lex+parse ≈ 291 ms → ≈ 30–60 ms |
| E13 | handoff: total time vs general path, one undecidable line at 0 % of a 100k-line script | not yet measured | ≤ +2 % | 0 % | only one failed statement is wasted |
| E13 | handoff: same, undecidable line at 50 % | not yet measured | 1.3 – 1.5x faster | 1.4x | 0.5/2.4 + 0.5 of the general-path time |
| E13 | handoff: same, undecidable line at 99 % | not yet measured | 2.0 – 2.6x faster | 2.3x | prefix at the E13 speed-up |
| E14 | end to end `t_import`, on top of E13 | 342 ms | 40 – 90 ms | 60 ms | ≈ 0.3–0.6 µs/line; **low confidence, no prototype** |
| E14 | ratio to regex (`import`), E13 + E14 | 30x | 4 – 8x | 5.5x | 60 ms / 11 ms; low confidence |
| E15 | repetition rate, flat benchmark libraries | not yet measured | < 5 % | 2 % | each position tried once per candidate |
| E15 | repetition rate, shipped samples with blocks/alternation | not yet measured | 5 – 40 % | 15 % | block-body backtracking; gate is 25 % |
| E15 | verdict | — | **not built** (gate not met on most samples) | — | hashing cost on flat grammars |
| B | parse time vs A at F ≤ 10 | not yet measured | within ±10 % | 0 % | little to share |
| B | parse time vs A, F = 1 000, heavy sharing | not yet measured | 1.5 – 4x faster | 2.5x | E11/E12 |
| C | end to end on a fully eligible script vs base | 342 ms (`t_import`) | 4 – 8x faster | 6x | E13 + E14 |
| C | share of **shipped libraries** in which every statement shape is decidable with proof | not yet measured | 20 – 50 % | 35 % | lowered in revision 3: an ineligible function that may match blocks the scanner unless excluded by proof |
| A / B / C | break-even runs (compile-time increase ÷ per-run saving), 15-line script, F = 100 | not yet measured | 50 – 500 runs for B and C | 150 | compilation cost amortised over small scripts |
| S1 | end to end `t_import` | 342 ms | 130 – 170 ms | 150 ms | cumulative curve above |
| S1 | end to end `t_all` | 500 ms | 190 – 250 ms | 220 ms | same |
| S1 | `contains` | 3 386 ms | 150 – 220 ms | 185 ms | same + E7 |
| S1 | interaction factor | — | 0.65 – 0.85 | 0.75 | E2/E3 and E1/E5 overlap |
| S1 | ratio to regex (`import`) | 30x | 12 – 15x | 13x | S1 / 11.3 ms |
| all | wasm size delta per adopted method | 1 390 420 B (`TEST-2026-0013`) | ≤ +8 KiB each | — | `PROP-2026-0006` M-08 style ceiling, proposed |

**Predicted ranking by isolated parse-or-total gain on the 4-function benchmarks** (frozen; the measured ranking is compared
with this): **E2 > E5 ≈ E1 > E6 (on `t_all`) > E3 > E9**, with **E7 dominant on `contains`**, **E4a/E4b/E4c
near zero on 4-function libraries but dominant at F ≥ 100**, **E11/E12 no better than E4b on unique prefixes and ahead only on heavily shared ones**, and
**E13 + E14 the largest single end-to-end gain on fully eligible scripts**.

**Predicted architecture ranking** (frozen): on fully eligible flat scripts **C > B ≈ A**; at F ≤ 10 **A ≈ B**; at F ≥ 1 000 with heavy prefix sharing **B > A**; on
ineligible scripts **C = A or B** (it falls back), at ≤ 5 % overhead.

### Expected Overall Result

```text
  END TO END, 100 000 lines, anchored `import X`     (predicted central values)

  regex          ▏                                            11 ms
  S1 (all wins)  ██████████████░░░░░░░░░░░░░░░░░░░░░░░░░░    150 ms   ≈ 13x regex
  today          ████████████████████████████████████████    342 ms   ≈ 30x regex
                 0                                         350 ms

  Why S1 does not reach regex (Amdahl, applied twice):
     parse → 0 would still leave lex+eval ≈ 121 ms today        (idea file §6: ≈ 2.9x ceiling)
     with E5+E6 lex+eval fall to ≈ 70 ms, so the floor of "same work, less allocation" is ≈ 70–100 ms
     going below it means not building tokens or a tree for eligible statements — architecture C (E13 + E14),
     in scope since revision 2, with an exclusion proof and a one-way prefix handoff; its prediction (below) is the least certain here

  PREDICTED CENTRAL VALUES BY ARCHITECTURE            fully eligible flat script, 100 000 lines
  today          ████████████████████████████████████████   342 ms
  A  (= S1)      ██████████████████░░░░░░░░░░░░░░░░░░░░░░   150 ms   2.0–2.6x
  B              ██████████████████░░░░░░░░░░░░░░░░░░░░░░   ≈ A at small F; ahead of A only at large F with shared prefixes
  C  (E13+E14)   ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    60 ms   ≈ 4–8x, LOW CONFIDENCE (no prototype)
  regex          ▏                                           11 ms
```

## Documentation, Demo and Release Impact

| Artifact | Exact Path or Destination | CRUD | Required Content / Verification | Owner | Release Gate |
|---|---|---|---|---|---|
| This proposal | `program_docs/proposals/prop-2026-0007-parser-and-pipeline-speed-experiments.md` | CREATE | front matter, header, template sections | Capy Engine | review |
| Document index | `program_docs/index/document-index.md` | UPDATE | row, status, next IDs, revision 7 | Capy Engine | index check |
| Test record | `program_docs/testing/test-2026-0014-parser-speed-experiments.md` | CREATE (after the run) | every sample, equality results, prediction-vs-actual | Capy Engine | R13 |
| Benchmark tooling | `rust/devtools/` | CREATE (later, by plan) | `phases`/`scale` bins promoted from `.ignore/` only if the owner wants them tracked | Capy Engine | `PROP-2026-0006` R5 overlap — decide there |
| Manuals / system / architecture | — | none | no behaviour changes | — | — |
| Version / release | — | none | nothing adopted by this proposal | — | — |

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases / Internal Constraint | Changes | Files | Tests | Manual / Demo / Release Evidence |
|---|---|---|---|---|---|---|---|
| R1 | UQ-05 | G-01 | UC-01 | C-01 | F-03 | T-01 | `TEST-2026-0014` |
| R2 | UQ-05 | G-02 | UC-04 | C-15 | F-01, F-13 | T-12 | `TEST-2026-0014` |
| R3 | UQ-05 | G-01 | UC-01 | C-02 | F-03 | T-01 | `TEST-2026-0014` |
| R4 | UQ-05 | G-01 | UC-01 | C-01 | F-03 | T-02 | `TEST-2026-0014` |
| R5 | UQ-04 | G-04 | UC-02, UC-06 | C-03…C-19 | F-04…F-12, F-17…F-19 | T-03, T-04, T-05 | `TEST-2026-0014` |
| R6 | UQ-04 | G-04 | UC-02 | C-03…C-19 | F-04…F-12, F-17…F-19 | T-09 | `TEST-2026-0014` |
| R7 | UQ-04 | G-04 | internal (ARCH-002) | C-09, C-10 | F-07, F-08 | T-10 | `TEST-2026-0014` |
| R8 | UQ-04 | G-04 | internal (ARCH-001) | C-10 | F-08 | T-10 | `TEST-2026-0014` |
| R9 | UQ-04 | G-04 | internal (ARCH-003) | C-04, C-09, C-13, C-16 | F-07, F-10, F-17 | T-06, T-13 | `TEST-2026-0014` |
| R10 | UQ-04 | G-03 | UC-03 | C-09 | F-07 | T-07 | `TEST-2026-0014` |
| R11 | UQ-05, UQ-06 | G-01 | UC-01 | C-12, C-14 | F-03 | T-02, T-11, T-18 | `TEST-2026-0014` |
| R12 | UQ-04 | G-06 | UC-06 | C-03 | F-11 | T-08 | `TEST-2026-0014` |
| R13 | UQ-05 | G-01, G-02 | UC-04 | C-15 | F-01, F-02, F-13 | T-12 | `TEST-2026-0014` |
| R14 | UQ-04 | G-05 | UC-05 | C-15 | F-14 | T-12 | ADRs |
| R15 | UQ-05 | G-01 | internal (CODE-001) | C-01 | F-03 | `git status` check | — |
| R16 | UQ-06 | G-03, G-07 | UC-03, UC-07 | C-09, C-20 | F-07, F-20 | T-14, T-18 | `TEST-2026-0014` |
| R17 | UQ-06, UQ-07 | G-04, G-07 | UC-07 | C-13, C-16, C-17, C-18 | F-17, F-18 | T-13, T-15, T-20 | `TEST-2026-0014` |
| R18 | UQ-06 | G-07 | UC-07 | C-20 | F-20 | T-17 | `TEST-2026-0014` |
| R19 | UQ-07 | G-04 | UC-07 | C-17, C-18 | F-18 | T-19 | `TEST-2026-0014` |

Reverse check: every `C-01…C-20` appears under at least one requirement; every `F-NN` maps to a `C-NN` and an `R`. F-15 and F-16 are `READ`
discovery inputs with no change. No orphan.

## Plan Strategy and Estimated Work

One implementation plan is **not** needed for this proposal itself (it adopts nothing). It needs one *experiment run* followed by
**one plan per adoption wave**:

```text
  PROP-2026-0007 approved
        │
        ▼
  RUN  (sandbox, no repo change)        E0 ─► E8 ─► Tier A ─► Tier B ─► E10 ─► E15 gate ─► E11, E12 ─► E13, E14 ─► S1 ─► A/B/C
        │                                                                    est.: 4–8 working sessions;
        ▼                                                                    build time per variant ≈ 15–60 s
  TEST-2026-0014  (all samples, verdicts, prediction-vs-actual)
        │
        ▼
  ADR-0005…  one per adopted method (ranking rule)
        │
        ▼
  PLAN-2026-0005  wave 1: E8 + E7 (+E1, E9 if SUPPORTED)           small, local, low risk
  PLAN-2026-000x  wave 2: E2, E3 (then E5, E6 by marginal gain)    medium
  PLAN-2026-000y  wave 3: E4*, E11/E12 — only if the E4 decision rule held (shipped max F = 14 says no, absent OQ-01)
  PLAN-2026-000z  wave 4: E13/E14 (architecture C) — only if C wins the architecture comparison, T-15, T-19 and T-20 are clean and the handoff never costs more than 2 %
```

Relationship to `PROP-2026-0006`: its INC-1 (in-place `append`) and INC-2 (scaling benchmark, profile) are independent and may
proceed first. Its INC-3 ("constant-factor changes the profile ranks") is exactly the set this proposal measures; this proposal
therefore **supplies INC-3's candidate list and ranking evidence** and must not duplicate INC-2's `scalebench` — if both are
approved, the `scale` bin here becomes the shared artefact (decide under `PROP-2026-0006` R5).

## Open Questions

| ID | Question | Why It Matters | Proposed Default |
|---|---|---|---|
| OQ-01 | **Partly answered:** shipped libraries have median F = 4, max F = 14 (123 libraries, 2026-10-08). What F do *users'* libraries reach, or should we plan for? | decides whether E4b/E11/E12 can ever be adopted (E4b decision rule); libraries live in users' repositories | until the owner says otherwise, plan for F ≤ 100 and treat F = 1 000 – 10 000 as a stress test, not a target |
| OQ-02 | Should the harness (`phases`, `scale`, synthetic generator) become tracked tooling in `rust/devtools/`? | R13/RK-10; overlaps `PROP-2026-0006` R5 | yes, after the run, via a plan |
| OQ-03 | Is `+8 KiB` wasm size per method acceptable, and is there a total ceiling? | `PROP-2026-0006` M-08 uses +3 000 B for one change | +8 KiB each, ≤ +24 KiB total, owner to confirm |
| OQ-04 | Is a profiler (`samply`/`perf`) available to install for a second opinion on E0's attribution? | Alternative E | not required; attribution by counters and stubs |
| OQ-05 | Should E8 ship on its own now, ahead of the programme, because it is a correctness defect? | P-09 is a bug independent of speed | yes: a separate small plan, with the d8 test (R12) |
| OQ-06 | May we measure a regex-style fast path for anchored shapes (skip the tree)? | the route to regex-class speed | **Resolved in revision 2:** yes, as E13/E14 (architecture C), with an exclusion proof and a one-way prefix handoff (revision 3) and a differential test (R17) |
| OQ-07 | How much extra grammar-compilation time is acceptable for B or C? | libraries are loaded once per process; editors and short scripts pay it each time | adoption requires break-even ≤ 50 runs of a typical script (proposed), owner to confirm |
| OQ-08 | Should C re-enter the fast path after a handoff (per-region hybrid)? | a hybrid raises coverage but shares evaluator `context` state across two engines | **Resolved in revision 3:** no re-entry; a one-way prefix handoff removes the restart cost without sharing state. Revisit only if the measured fast-path line share is the binding limit |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | pending | | Confirm OQ-01, OQ-03 and the frozen predictions/thresholds before the first measured run |
| Reviewer | Release Management | pending | | |

## Related Documents

- `PROP-2026-0006` — scaling and incremental parsing (R1–R3 in-place state, R5 scaling benchmark, R6 profile, INC-3)
- `REL-0.24.0`, `TEST-2026-0010`, `TEST-2026-0013` — release baseline and existing benchmark conventions
- `STD-2026-0000` — project standards index (GOAL-001/002, PHIL-001/002, CODE-001, ARCH-001/002/003, QUAL-001/002/003, GATE-001/002)
- `DOCUMENTATION.md` §4.2, §12.1, §21 — proposal structure and validation rules
- Working notes (untracked, `.ignore/`): `regex-compare/RESULTS.md`, `how-capy-parses-in-detail.md`, `improvement_idea.md`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-08 | Olivier | Initial draft (assistant-drafted), from the regex comparison, the stage/allocation profile and the speed-up idea list of the same day |
| 2 | 2026-10-08 | Olivier | Responding to an architecture review (UQ-06): removed E11's gate; added E12 (active-state tree), E13 (fused scanner), E14 (specialised execution), E15 (memoisation, gated by a measurement); added the three-architecture comparison (A/B/C), the benchmark matrix (F up to 10 000, compile-time split, peak memory), R16–R18, G-07, UC-07; recorded shipped-library sizes (median 4, max 14) and the fast-path eligibility counts; widened nothing else |
| 3 | 2026-10-08 | Olivier | Responding to a second review (UQ-07): E12 now runs lower-index unsupported candidates before committing a tree accept; E11/E13 gain an exclusion proof ("cannot prove" = "may match"); the whole-script restart is replaced by a one-way prefix handoff with measured worst-case overhead; E14 is an execution path only and `ast --json` stays identical (R19, T-19); adoption no longer stops at the first weak method (every method tested, leave-one-out on the final stack); added R19, T-19, T-20, RK-17–19; lowered the decidable-library prediction; no architecture added |
| 4 | 2026-10-08 | Olivier | Run executed (results in `TEST-2026-0014`). At the owner's instruction architecture C (which contains A) was applied to the repository tree; E5 (full), E10, E11/E12 and the full matrix were not run. Status stays `draft`: no approval was recorded in the Approval table |
