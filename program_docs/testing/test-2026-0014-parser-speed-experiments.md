---
document_id: TEST-2026-0014
title: Test — Parser and Pipeline Speed Experiments (PROP-2026-0007)
document_type: test
status: completed

created_date: 2026-10-08
last_updated: 2026-10-08
document_revision: 2

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core
  - capy-wasm-abi

affected_versions:
  from: "0.25.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Records the experiment run defined by PROP-2026-0007 — thirteen sandbox variants of the 0.25.0 engine, three architectures (A optimised parser, B compiled tree, C fused scanner + direct emit), a regex comparison, correctness gates, scaling in library size, and the prediction-versus-actual scoreboard. Nothing in `rust/` of the repository tree was changed.

reason: DOCUMENTATION.md section 21.3 and QUAL-002 require every measurable claim to be executed against a predeclared baseline, command and threshold, with all samples and limitations reported. PROP-2026-0007 R13 requires the results in a tracked document.

related_documents:
  - PROP-2026-0007
  - PROP-2026-0006
  - TEST-2026-0013
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - performance
  - parser
  - benchmark
  - regex
  - experiment

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-08
---

# Test — Parser and Pipeline Speed Experiments (PROP-2026-0007)

> **Status:** Completed (result **PARTIAL**, see §Result)
> **Created:** 2026-10-08
> **Last Updated:** 2026-10-08
> **Affected Versions:** 0.25.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-wasm-abi

## Purpose

Find out, by measurement, where Capy's time goes, which proposed speed-ups are worth building, and how
close a different architecture can get to a compiled regex on the same task — without changing any
observable behaviour.

```text
  HEADLINE (100 000 lines, release build, interleaved, paired per round, 9/9 rounds faster for every task)

  task        baseline 0.25.0     A  optimised parser        C  scanner + direct emit      regex (same task)
  ─────────   ───────────────     ────────────────────────   ──────────────────────────    ─────────────────
  import      347 ms  ██████████  218 ms (1.6x)  ███████░░░    18.9 ms (18.4x) ▏             11.2 ms
  say         339 ms  ██████████  218 ms (1.6x)  ███████░░░    19.2 ms (17.5x) ▏             11.7 ms
  assign      390 ms  ███████████ 234 ms (1.6x)  ███████░░░    19.4 ms (20.2x) ▏             13.6 ms
  all-3       466 ms  █████████████ 270 ms (1.7x) ████████░░    21.7 ms (21.7x) ▏             24.8 ms  ◄ C beats regex
  contains*   338 ms  ██████████  68 ms (5.0x)   ██░░░░░░░░    56.4 ms (6.0x)  ██░░░░░░░░    0.39 ms
                                                              * 10 000 lines; `if` in the template, so no direct emit

  Capy / regex:   baseline 29–37x   →   A 18–24x   →   C 0.9–1.7x   (contains: 863x → 172x → 148x)
```

## Validated Requirements

| PROP-2026-0007 Requirement | Description | Result |
|---|---|---|
| R1 | Variants built from a clean export of `75a9f68` | PASS (detached git worktrees, one build directory each) |
| R3 | Spread measured first; gains judged against it | PARTIAL (frozen stop line exceeded; paired ratios used, see §E0) |
| R4 | A/B interleaved | PASS |
| R5 | Output and `ast --json` identical | PASS with 2 reviewed `ast --json` differences (E8) |
| R6 | Repo tests pass on every ranked variant | PASS (132/132 release; 132/132 debug for C; baseline 122) |
| R7 | No new `capy-core` dependency | PASS (no `Cargo.toml` change in any variant) |
| R9 | Bounded caches, no unbounded recursion | PASS (regex cache 64, template cache 4 096; tests `regex_cache`) |
| R10, R16 | Library-size and matrix runs | PARTIAL (F up to 1 000 measured; 10 000 and the full matrix not run) |
| R11 | Ranking inputs recorded | PARTIAL (peak bytes and allocations yes; compile-time split not run) |
| R12 | E8 regression test fails on baseline | PASS |
| R17 | Exclusion proof, prefix handoff, differential test | PASS |
| R18 | Architectures compared on identical workloads | PARTIAL (A and C yes; B not built) |
| R19 | `ast --json` unchanged by execution fast paths | PASS (fast paths exist only in `Library::run`) |

## Preconditions

- The baseline is commit `75a9f68` (release 0.25.0), exported as a **detached git worktree** under
  `.ignore/worktrees/` (the owner asked for worktrees; the proposal had specified `git archive`; same commit).
- Every worktree builds into **its own** `target-<name>` directory. A first attempt that shared one directory
  silently made the "base" CLI build contain the variant's code (identical binaries, a 0.02 s build) —
  Cargo hashes path packages relative to the workspace root. That attempt was discarded and rebuilt.
- Nothing in `rust/` of the repository tree was edited; `.ignore/` is not committed (CODE-001).

## Test Environment

```text
  macOS arm64 (Darwin 25.4.0), 10 cores, 16 GB, Rust stable, release profile (opt-level 3)
  host load average during runs: 1.3 (quiet) … 5.5 (VS Code renderers at 60–110 % CPU and another
  project's rustc build running concurrently) — NOT controllable; hence interleaving + paired ratios
  disk: 99 % full, ~8 GB free → per-worktree release-only build directories (~100 MB each)
  profiler: /usr/bin/sample (macOS) — answers OQ-04, a sampling profiler IS available
```

## Test Data

| Dataset | Description |
|---|---|
| `corpora/c100k.txt`, `c10k.txt` | 100 000 / 10 000 lines, four statement kinds interleaved (import, say, key = int, prose) |
| `harness/t_import.capy`, `t_say`, `t_assign`, `t_all`, `t_contains` (`.capy`) | the five task libraries, each paired with a regex implementation of the same job |
| `samples/*/lib.capy` + scripts | 123 shipped libraries, 125 script/expectation pairs, 541 functions |
| mutations | ~1 000 deterministic mutations of the shipped scripts (token drop/insert, truncation, line delete/duplicate, junk char, indent) |
| random libraries | 120–140 random libraries (3–30 functions, shared/punctuation/capture-first shapes, priorities) × 40 random statements |
| `corpora/scale/` | synthetic libraries F = 4, 10, 100, 1 000 (unique and 20-per-bucket shared first literals) |
| `corpora/handoff/` | the 100 000-line script with one tab-containing (scanner-undecidable) line at 0 %, 50 %, 99 % |

## Procedure

```text
   per method:  implement in worktree ─► repo tests (cargo test --release -p capy-core)
                ─► harness build ─► eq: 125 sample hashes (orchestrator::run, ast text, ast json, Library::run)
                ─► regex-task outputs must equal the regex implementation's
                ─► mutcheck / dispcheck / tplcheck / fpcheck (byte compare vs baseline or E8-only reference)
                ─► clippy -D warnings ─► measure (interleaved rounds, paired analysis)
   commands:    .ignore/speedlab/{build.sh, step.sh, speedlab.py, scale.py, paired.py,
                                  mutcheck.py, dispcheck.py, tplcheck.py, fpcheck.py, genlib.py}
```

## Expected Results

The predictions frozen in `PROP-2026-0007` revision 3 (§Frozen Predictions). The scoreboard below scores each.

## Actual Results

### 1. Correctness gates (every step)

| Gate | Result |
|---|---|
| Repo tests, release | baseline 122 pass; every variant 132 pass (122 + `cursor_split` 3, `regex_cache` 4, `lexer_lines` 3); C also 132 pass in the **debug** profile |
| Sample hashes vs baseline (125 pairs × run / ast text / ast json / `Library::run`) | 0 differences, plus 2 **reviewed** `ast --json` differences (E8, below) |
| Regex-task outputs vs the regex implementation | equal in every task, every variant, every round (asserted) |
| Mutation corpus, `run` + `ast` CLI + `Library::run` | A: 2 000 invocations (1 010 error cases) 0 diffs · C: 3 000 invocations (1 522 error cases) 0 diffs |
| Random libraries × statements vs E8-only reference | A: 5 600 pairs 0 diffs · C: 4 800 pairs 0 diffs |
| 35 awkward templates (escapes, pipes, nested, bad helpers, unterminated) | 0 diffs (A and C) |
| Scanner structural differential (prefix == first k statements of the real parser, spans included) | 0 mismatches / 3 279 runs; 41 157 statements decided |
| `clippy -D warnings` | found **7 `needless_borrow` in my own code** (E5a, E3); fixed; clean in A and C |
| **Test strength** (QUAL-003) | 3 planted mutants were all caught: dispatch prefix lookup (9 diffs), scanner span end (mismatch), direct `${col}` (156 diffs) |

**E8 reviewed difference.** In the baseline, a candidate that fails after splitting a merged punctuation token
leaves the remnant in the token stream. Consequences measured: (a) `ErrorNode.tokens` held `<` / `/` where the
source has `><` / `</` (2 of 125 samples' `ast --json`); (b) worse, a `tail` capture held `> -> =>` where the
source says `=> -> =>`, with its span starting at column 2 — a template that prints the capture would emit
**wrong output from `run`**. PROP-2026-0007 P-09 understated this. Fixed by an undo log (E8).

### 2. E0 — the noise floor (frozen prediction: stage spread ≤ 5 %, stop line 8 %)

```text
  baseline parse stage, 15 interleaved process rounds       frozen definition        robust
  t_import  min 214  q1 242  med 246.5  q3 249  max 251         (max-min)/med 14.9 %   IQR/med 2.9 %
  t_say                                                          14.5 %                 4.5 %
  t_assign                                                       13.0 %                 4.4 %
  t_all                                                           8.0 %                 1.1 %
```

**PREDICTION MISS and over the stop line.** The excess is single *fast* outliers, not slow tails. Deviation
(PHIL-002, awaiting owner ratification): proceed using paired per-round ratios (variant/base within one
interleaved round; median, IQR, rounds-faster). End-to-end spread was 8–20 % on a quiet host and up to 50 % under
load. Every conclusion below holds under the pairing: all variants were faster in 9/9 (or 6/6) rounds.

### 3. Stage view of the stack (`t_import`, 100 000 lines, ms; allocations per line lex/parse/eval)

```text
                       lex        parse        eval     total   allocs l/p/e        peak MB
  baseline            61.8       247.8        65.2     374.8    11.4 / 39.2 / 6.0     262
  +E8 E7 E1 E2        62.1       184.3        66.5     312.9    11.4 / 24.5 / 6.0     262   parse -23 %
  +E3                 60.6       147.8        58.5     266.9    11.4 / 21.5 / 6.0     262   parse -35 %
  +E6                 60.8       145.8        56.3     262.8    11.4 / 21.5 / 4.2     286*  (* E9 slack, fixed)
  +E5a                32.5       154.9        60.6     247.9     5.8 / 21.5 / 4.2     262   lex  -47 %
  +E4 +E4a (A)        32.1       150.6        61.1     243.8     5.8 / 20.7 / 4.2     262
```

Parse allocations by step (counting build): 39.2 → 31.0 (E1) → 24.5 (E2) → 21.5 (E3).

### 4. Marginal end-to-end gain per step (6 interleaved rounds, paired; `*` = IQR spans 1.0, not distinguishable)

| Step | import | say | assign | all-3 | contains |
|---|---|---|---|---|---|
| E8+E7+E1 (cumulative vs base) | 1.11x | 1.10x | 1.11x | 1.08x | 4.65x |
| E2 | 1.093x | 1.096x | 1.097x | 1.142x | 1.009x* |
| E3 | 1.079x* | 1.036x* | 1.137x | 1.123x | 1.005x* |
| E9 | 1.039x | 1.073x | 0.984x* | 1.003x* | 1.010x |
| E6 | 1.040x | 1.026x* | 1.101x | 1.101x | 0.999x* |
| E5a (lexer) | 1.107x | 1.127x | 1.108x | 1.097x | 1.023x |
| E4 dispatch / E4a | 0.995x* | 1.003x* | 0.997x* | 0.997x* | 1.000x* |
| **A cumulative** | **1.60x** | **1.59x** | **1.67x** | **1.69x** | **5.01x** |
| **C = A + E13 + E14** | **17.98x** | **17.42x** | **20.35x** | **21.43x** | **5.84x** |

By the ranking rule (marginal gain ≥ 2× spread): **E2, E5a** clearly; **E7** (decisive on `contains`); **E6, E3** on some tasks;
**E9** and **E4** are within noise on 4-function libraries (as predicted). Isolated single-method runs were **not** made.

### 5. E7 — the regex-in-template case

`contains` (10 000 lines): 335.4 → 79.4 ms (−76 %, 4.2x; −80 % / 5x in later runs). Count build: eval 302 ms @ 497 allocs/line →
51 ms @ 67 allocs/line, lex+parse 22 ms. The task is **evaluation-bound** after the fix (inner-DSL `if`), not parse-bound.

### 6. Scaling in library size (Library::run; µs per line, 20 000 → 200 lines per cell; 3 rounds)

```text
  µs/line                      F=4     F=10    F=100   F=1 000   (15-line script, ms/run: F=100 | F=1 000)
  baseline (unique first lit)  3.5     4.8     24.6    235.4          0.709 | 5.67
  A                            2.1     1.9     1.8       1.9          0.028 | 0.028
  speed-up                    1.7x    2.5x    13.9x    123.9x          25x   | 202x
  baseline (20 per bucket)      -       -      20.8    183.0          0.637 | 5.42
  A                             -       -       2.3      2.5          0.036 | 0.037

  baseline cost is linear in F: ≈ 0.22 µs per extra function per line.
  Two per-run O(F) costs removed by E4a: candidate re-sort + deep clone of every FuncDef (parser) and a
  second deep clone of every FuncDef into the evaluator's map.
  E8 alone: 0.98–1.02x (neutral).  Shared prefixes cost A only 1.33x of unique at F = 1 000 (bound for B).
```

### 7. Architecture C (E13 scanner + E14 direct emit), versus regex

```text
                  baseline        A              C            regex       C / regex
  import (100k)    346.9 ms     217.8 ms       18.9 ms       11.2 ms        1.7x
  say              338.9        218.2          19.2          11.7           1.7x
  assign           390.4        234.3          19.4          13.6           1.4x
  all-3            465.9        270.2          21.7          24.8           0.9x   (Capy faster)
  contains (10k)   337.8         67.7          56.4           0.39        147.7x  (not direct-renderable)
```

`E13` alone (scanner builds the same `FuncCall`s, evaluator unchanged), measured before E14: `import` 135 ms (9/9 rounds faster, 2.58x).

**Profile of C** (`sample`, `t_import`): scan 25 %, evaluator 37 %, dropping the `FuncCall` tree 24 % — the reason E14 pays.
**Profile of A:** parse 53 %, evaluate 21 %, **dropping the tree 13.5 %** (invisible to stage timers), tokenise 4.5 %; freeing memory is the largest single item.

**Handoff (T-20)** — `t_all`, 100 000 lines, one undecidable (tab) line; outputs identical in all cases:

| Undecidable line at | base | A | C | C vs A |
|---|---|---|---|---|
| none | 493 ms | 275 ms | 21.8 ms | 0.08x |
| 0 % | 484 | 284 | 284.3 | **1.00x (+0.0 %)** |
| 50 % | 485 | 286 | 160.3 | 0.56x |
| 99 % | 487 | 285 | 32.9 | 0.12x |

**Coverage on shipped material (the honest limit).** The scanner compiles **65.2 %** of the 541 shipped functions
(29 of 122 libraries compile fully, 10 not at all); only 21.6 % can render directly. On the shipped sample scripts it
decides **168 of 534 statements (31.5 %)**, fully handles 19 of 124 scripts and decides nothing in 84. The benchmark
corpora are the best case, not the typical case; the 18–22x applies to flat, single-line, `write`-only workloads.

### 8. E15 gate, wasm size

| Measurement | Result |
|---|---|
| Repetition rate of (function, position) within a top-level statement | shipped scripts 6.7 % overall (3 661 attempts), max 23.3 % → **gate (25 %) not met, memoisation not built**; benchmark libraries 0.00 % |
| wasm size (`capy-wasm-abi`, release) | base 1 394 418 B · A 1 419 911 B (**+25 493 B, +1.8 %**) · C 1 434 085 B (**+39 667 B, +2.8 %**). Proposed ceiling +24 KiB total: **A exceeds it by ~0.9 KB** (OQ-03 for the owner) |

## Prediction scoreboard (frozen in PROP-2026-0007 rev 3; "hit" = inside the stated range)

| Prediction | Predicted | Observed | |
|---|---|---|---|
| E0 stage spread | ≤ 5 % (stop 8 %) | 8–15 % (IQR 1–4.5 %) | MISS |
| E8 `ast --json` diffs | 0 | 2 samples (error tokens now verbatim) | MISS |
| E8 time | ±3 % | 0.98–1.02x | HIT |
| E7 `contains` (scaled to 100k) | 300–460 ms | ≈ 790 ms (eval-bound) | MISS (slower) |
| E5 lex | −40…−60 % | −47 % (first part only) | HIT |
| E6 eval `t_all` / `t_import` | −30…−50 % / −20…−35 % | −19 % / 0 % | MISS (smaller) |
| E4a 15-line script, F = 1 000 | −60…−90 % | −99.5 % | MISS (better) |
| baseline µs/line F=100 / F=1 000 | est. 30–60 / 300–600 | 23 / 226 | MISS (lower) |
| E4b ≥ 8x (F=100), ≥ 50x (F=1 000) | | 13.9x / 123x (whole stack) | HIT |
| E11 heavy-prefix gain over E4b | 1.5–4x | bounded ≤ 1.33x by measurement | MISS (over-estimate) |
| S1 `t_import` / `t_all` | 130–170 / 190–250 ms | 218–235 / 268–270 ms (1.6–1.7x) | MISS (smaller) |
| E13 alone `t_import` | 110–170 ms | 135 ms | HIT |
| E14 / C `t_import` | 40–90 ms | 18.9 ms | MISS (better) |
| C vs regex | 4–8x | 1.4–1.7x (0.9x on all-3) | MISS (better) |
| Handoff 0 % / 50 % / 99 % | ≤ +2 % / 1.3–1.5x / 2.0–2.6x | +0.0 % / 1.78x / 8.7x | HIT / MISS / MISS (better) |
| Shipped functions compiled | 50–75 % | 65.2 % | HIT |
| Libraries fully decidable | 20–50 % | 23.8 % (29/122) | HIT |
| E15 verdict | not built | gate not met (max 23.3 %) | HIT |
| E9 verdict | not supported (≈ noise) | ≈ noise | HIT |
| Isolated single-method gains (E1, E2, E3 …) | stated | **not measured** (only marginal-in-stack) | N/M |

## Deviations from PROP-2026-0007 (PHIL-002)

1. Sandbox is `git worktree` (owner request), not `git archive`; one build directory per worktree after the shared-target hazard.
2. E0 stop line exceeded; paired ratios used instead of stopping. **Owner to ratify.**
3. **E4c as written would have been unsound.** `int`/`string`/`bool`/`any` captures go through the expression parser and do not restrict the
   first token's kind; only `ident`/`dotted_ident` (Ident) and `raw` (Ident|Str) are filterable. Implemented the sound subset.
4. E5: only the first part (borrowed lines, one token vector) was built; source-backed tokens were not.
5. **Not built / not run:** E10 spike (profile gave the share instead), E11 and E12 (architecture B), the full R16 matrix, F = 10 000, the
   compile-time/break-even split, isolated per-method measurements. Architecture B is therefore **unmeasured** (bounded only).
6. E9 initially raised peak memory 262 → 286 MB (kept lexer slack); fixed with `shrink_to_fit`, time within noise.
7. `clippy -D warnings` flagged 7 issues in the experiment's own code; fixed.

### 9. Peak memory, and architecture C applied to the repository tree

| Variant | Peak live bytes (`Library::run`, 100 000 lines, `t_import` / `t_all`) |
|---|---|
| baseline and A | 265 MB / 265 MB |
| **C** | **15 MB / 16 MB** (about 17x less: no token vector, no syntax tree) |

On the owner's instruction ("choose the best performing option in terms of memory and speed"), architecture **C** (which contains A) was
applied to `rust/` of the repository tree, uncommitted: the experiment patch minus the test-only hooks, plus the in-repo differential tests
`tests/fast_path.rs`, `cursor_split.rs`, `lexer_lines.rs`, `regex_cache.rs`. Gate results on the real tree: `cargo build --workspace`, `clippy --workspace
--all-targets -D warnings` clean, **153 tests pass** (workspace), `check_publishable.py` PASS, `mkdocs build --strict` exit 0, every shipped library passes
`capy check` except `left-recursion-rejected`, which the baseline also rejects (intended). Sample hashes vs baseline: 0 differences + the 2 reviewed E8
`ast --json` differences. Confirmation measurement on the real tree (5 interleaved rounds, 5/5 faster): import **21.0x**, say 20.9x, assign 23.3x, all-3 24.5x
(22.1 ms against a regex at 24.6 ms), contains 6.0x. The owner also accepted the wasm size increase and ratified paired-ratio analysis.

## Result

**PARTIAL.** Correctness gates PASS everywhere they were run. The programme answered its central questions, but architecture B, the full
matrix, isolated method gains and the compile-time split were not executed.

```text
  WHAT THE MEASUREMENTS SAY
  ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
  │ 1. A  (E8 E7 E1 E2 E3 E9 E6 E5a E4 E4a): 1.6–1.7x end to end, 5x on regex-in-template,        │
  │       and 25–200x on small scripts with big libraries (per-run O(F) removed). Low risk.       │
  │ 2. The biggest single wins: E4a (small-script latency), E2, E5a, E7. E4 is irrelevant at       │
  │       F ≤ 14 (shipped max) — adopt only if user libraries grow.                               │
  │ 3. C reaches 18–22x on flat write-only scripts and beats a compiled regex on one task, but     │
  │       decides only 31.5 % of shipped statements; the handoff costs +0.0 % when it cannot help.│
  │ 4. B (tree) is bounded at ≤ 1.33x even at F = 1 000: not worth building for shipped sizes.     │
  └──────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Evidence

`.ignore/speedlab/results/*.json` (all samples), `logs/findings.md`, `logs/S1-curve.txt`, `logs/measure-ABC-E14.txt`, profiles `logs/profile-*.txt`,
patches `patches/*-cumulative.diff` (complete from E4 on), worktrees `.ignore/worktrees/speedlab-{base,a,c,e8,e15}`. **All untracked** (CODE-001); this document
carries the figures, and the patches reproduce the code.

## Evidence Sources

```text
  cd .ignore/speedlab
  ./build.sh <name> <worktree>            ./step.sh <label> <variant>
  python3 speedlab.py measure 9 a c       python3 paired.py            python3 scale.py 3 base a
  REF=e8 python3 mutcheck.py c 8 1        REF=e8 python3 dispcheck.py c 120 40 11
  python3 tplcheck.py c                   python3 fpcheck.py 6 3        (corpora/handoff for T-20)
  sample <pid> 6 -file …                  (profiles)
```

## Executed By

Claude (assistant), on behalf of the owner.

## Executed At

2026-10-08, macOS arm64, release profile, host noisy (see §Test Environment).

## Defects Raised

| Defect | Where | Status |
|---|---|---|
| P-09: a failed candidate's punctuation split leaks into later candidates, token text, captures and `run` output | `make_parser.rs` `match_literal` / close-sequence consumer | fixed in the sandbox (E8); **not yet in the repository tree** |
| `regex_match` and `pattern` re-compile every call | `inner_evaluator.rs`, `make_evaluator.rs` | fixed in the sandbox (E7) |
| Per-run O(F) deep clones (parser candidate list, evaluator function map) | `make_parser.rs`, `make_evaluator.rs`, `capy.rs` | fixed in the sandbox (E4a) |

## Related Documents

- `PROP-2026-0007` (the plan and frozen predictions), `PROP-2026-0006`, `TEST-2026-0013`, `STD-2026-0000`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-08 | Olivier | Initial record of the experiment run (assistant-executed) |
| 2 | 2026-10-08 | Olivier | Added peak-memory results and the application of architecture C to the repository tree |
