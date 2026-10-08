---
document_id: PROP-2026-0006
title: Scaling and Incremental Parsing — Remove the Quadratic State Cost, Establish a Scaling Baseline, Decide on Incremental Reparse
document_type: proposal
status: draft

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 1

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
  - capy-cli
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

scope: Fixes one measured quadratic cost (list state operations copy the whole list), adds a repeatable scaling benchmark so every later performance claim has a baseline, and defines the decision gate for an incremental-reparse API for editor use. No grammar change.

reason: A question about whether Capy is fast led to a measurement session on 2026-10-07. Parsing is linear and fast; one state operation is quadratic and turns a 240 000-line transpile into 24 seconds. The owner asked for a proposal to improve on this.

related_documents:
  - REL-0.24.0
  - PROP-2026-0004
  - PROP-2026-0005
  - TEST-2026-0010
  - TEST-2026-0013
  - STD-2026-0000

supersedes: null
superseded_by: null

tags:
  - performance
  - scaling
  - incremental
  - editor

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-07
---

# Scaling and Incremental Parsing — Remove the Quadratic State Cost, Establish a Scaling Baseline, Decide on Incremental Reparse

> **Status:** Draft
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.25.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

```text
  WHAT WAS MEASURED (release build, macOS arm64, 2026-10-07)

  parse only            ████░░░░░░░░░░░░░░░░   3.4 – 3.9 µs/line   LINEAR  (15k → 240k lines)
  run, no list state    ████████░░░░░░░░░░░░   8.2 – 8.5 µs/line   LINEAR
  run, `append` state   ██████████████████████  18 → 34 → 101 µs/line   QUADRATIC

  cause of the last row:  `append` clones the whole list, pushes one item, re-inserts it
                          (inner_evaluator.rs, "append" arm)  →  n appends cost O(n²)
```

Three changes, in order of certainty:

| # | Change | Certainty |
|---|---|---|
| 1 | Make `append` / `merge` mutate in place | **proven** — cause located, fix is local |
| 2 | Add a repeatable scaling benchmark and record a profile | needed — the constant factor (3.5 µs parse, ~5 µs render per line) is **unprofiled** |
| 3 | Decide, by a measured gate, whether to build incremental reparse for editors | **unproven** — no measurement yet says a full reparse is too slow |

## Decision Requested

Approve changes 1 and 2 for implementation. Approve change 3 **only as a decision gate**
(R8): build it if, and only if, a full reparse of a representative editor file misses the
latency budget declared below. Approval does **not** authorize a table-driven parser rewrite,
memoization, or any grammar or output change.

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "is capy fast compare to other methods, dont run test just answer based on knowledge" | User, 2026-10-07 | Answered from reasoning: expected linear, larger constants than generated parsers |
| UQ-02 | "why?? what makes other methods more suitable?? is it a caching/state issue?" | User, 2026-10-07 | Two causes given: no state kept between parses (editors) and per-run constant factors (large files) |
| UQ-03 | "so we also want a proposal to improve on this" | User, 2026-10-07 | This document. The assistant then measured, because §21.3 forbids a performance claim without a baseline |

**A correction to the earlier answer.** The assistant said it expected cost "roughly linear in
file size". Measurement shows that is true of parsing and of rendering, **false** of list state:
one operation is quadratic. The earlier answer's two stated causes (no incremental state, constant
factors) are unchanged and still unmeasured as to size; the quadratic cost is a third, found by
measuring.

## Problem and Evidence

All timings: `rust/target/release/capy`, macOS arm64 (Darwin 25.4.0), best of 1–3 runs,
process startup (3.5 ms, `capy version`) subtracted for per-line figures. Source scripts are the
shipped samples repeated *n* times (`samples/transpile-py`, `samples/language-frontend`,
`samples/operator-precedence`); 15 to 100 lines per repeat.

| Problem | Evidence | Consequence | UQ |
|---|---|---|---|
| P-01 | `transpile-py`, `capy run`: 15 000 lines 266 ms (17.5 µs/line); 60 000 lines 2 070 ms (34.4); 240 000 lines **24 289 ms (101 µs/line)**. Doubling-by-4 the input multiplies the time by 7.8× and then 11.8× | A large file that mutates a list is unusable well before it is large | UQ-02 |
| P-02 | The same script with the `import` lines (the only `append`) removed: 119 ms / 457 ms / 1 857 ms at 14k / 56k / 224k lines — **8.2–8.5 µs/line, flat**. `capy ast` (parse only) on the original: 59 / 223 / 871 ms — **3.6–3.9 µs/line, flat** | Isolates the quadratic part to list state, not parsing or rendering | UQ-02 |
| P-03 | `inner_evaluator.rs`, `"append"` arm (map target): `let mut list = match m.get(&key) { Some(Val::List(v)) => v.clone(), … }; list.push(value); m.insert(key, Val::List(list));` — and the list-index arm clones the inner list the same way. `prepend` and `merge` copy likewise. `deep_copy_map` is called **once** at setup (`make_evaluator.rs:74`), so ordinary state is *not* copied per statement | The cost is exactly one list copy per `append`: O(list length) | UQ-02 |
| P-04 | `language-frontend` and `operator-precedence` (no list state): 7.7–8.9 µs/line from 16 000 to 100 000 lines | The linear regime holds for grammars that do not append | UQ-02 |
| P-05 | There is no benchmark that varies **size**. `nativebench` times one tiny script (≈195 µs) | A scaling regression cannot be seen; this one went unnoticed | UQ-02 |
| P-06 | Nothing in `Library::parse` accepts a previous tree or an edit range. Every call re-lexes and re-matches the whole source | An editor integration pays full cost per keystroke, however small the edit. **No measurement yet shows this is too slow** | UQ-02 |

```text
  P-01 / P-03, drawn:

   append #1   [a]                 copy 0, push        ┐
   append #2   [a b]               copy 1, push        │  total copies after n appends
   append #3   [a b c]             copy 2, push        │  = 0 + 1 + 2 + … + (n-1)
   …                                                   │  = n(n-1)/2     →  O(n²)
   append #n   [a b … n]           copy n-1, push      ┘

   in place:   push onto the existing list  →  O(1) per append, O(n) total
```

### What was NOT measured

- Memory use, allocation counts, and **where the 3.5 µs parse / ~5 µs render per line goes**
  (P-02 gives the split, not the cause). No profile exists.
- Backtracking cost on grammars with deep ordered choice (`PROP-2026-0004`); the sample used
  has shallow choice.
- Any comparison against another tool. The earlier answer's comparison table is reasoning, not data.
- The browser (wasm) build.

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems | Success Signal |
|---|---|---|---|
| G-01 | State operations cost O(1) amortized per call, so run time is linear in statements | P-01, P-02, P-03 | run-with-`append` per-line cost flat from 15k to 240k lines |
| G-02 | A repeatable benchmark that varies input size, so scaling is visible and gated | P-05 | `devtools` bin prints µs/line at several sizes and the exponent |
| G-03 | Know where the constant factor goes before changing it | P-02 | a recorded profile and a ranked list of costs |
| G-04 | Decide on incremental reparse from evidence, not intuition | P-06 | a latency budget, a measurement against it, and a build / don't-build record |

**Non-goals**

- A table-driven or generated parser; replacing the grammar engine.
- Packrat memoization **unless** G-03's profile shows repeated sub-parses dominate (OQ-02).
- Changing any rendered output, diagnostic, JSON schema or grammar rule.
- Multi-threaded parsing.
- Comparing Capy to other tools (OQ-04).

## Proposed User Journey

```text
  author's script appends 100 000 items to a list
        │  capy run lib script
        ▼
  before:  ~24 s   (list copied on every append)
  after :  ~2 s    (≈ the no-state cost)
```

## Requirements

| ID | Requirement | Type | Source | Acceptance Criteria | Goal |
|---|---|---|---|---|---|
| R1 | `append` pushes onto the existing list in place for both the map-key and list-index targets | functional | P-03 | the 240k-line run drops to within 1.5× the no-state run (M-01) | G-01 |
| R2 | `merge` into an existing object mutates it in place | functional | P-03 | same measurement pattern on a `merge`-heavy script | G-01 |
| R3 | `prepend` is either O(1) amortized (a deque-backed list) or documented as O(n) with the reason | functional / docs | P-03 | decision recorded; if left O(n) the docs say so | G-01 |
| R4 | Observable behaviour is unchanged: output, aliasing, error messages and ordering are byte-identical | compatibility | `GOAL-002` | all pre-existing goldens byte-identical; a test pins that a list read **before** an append is not changed by it | G-01 |
| R5 | A scaling benchmark prints µs/line at three or more sizes for parse-only and run, and the size exponent, from a committed script | tooling | P-05 | `devtools` bin or script committed; output recorded in a `TEST` document | G-02 |
| R6 | A CPU profile of parse and render on the benchmark is recorded, with a ranked list of the top costs | evidence | P-02 | profile summary in a `TEST` or `RPT` document | G-03 |
| R7 | Any constant-factor change made under G-03 must show a measured win and no output change | quality | `QUAL-001`, `QUAL-002` | per-change before/after in the same session, interleaved | G-03 |
| R8 | **Decision gate:** incremental reparse is built only if a full reparse of a representative editor file exceeds the declared budget | decision | P-06 | a recorded measurement against the budget and a build / don't-build ADR | G-04 |
| R9 | If built (R8 = build): `Library::parse_incremental(prev, edit, new_src)` reuses unchanged top-level statements, falls back to a full parse when an edit touches `define`, `preprocess` or a block that cannot be isolated, and returns a tree **equal to** a full parse | functional | P-06 | property test: for random edits, incremental result == full-parse result | G-04 |
| R10 | Documentation states the complexity of each state operation and the benchmark command | docs | `CODE-003` | `docs/` page updated; `mkdocs --strict` exit 0 | all |

## Use Cases

| ID | User Outcome | Actor | Surface | Inputs | Outputs | Negative Paths | Req |
|---|---|---|---|---|---|---|---|
| UC-01 | A script that appends many items runs in linear time | language author | `capy run` | 240 000 lines with one `append` per 15 | same output as today, in ≈ the no-state time | list read before an append is unchanged (aliasing test) | R1, R4 |
| UC-02 | A `merge`-heavy script runs in linear time | language author | `capy run` | repeated `merge` into one object | same output, flat per-line cost | — | R2, R4 |
| UC-03 | A maintainer sees a scaling regression immediately | maintainer | benchmark | the shipped samples at 3+ sizes | µs/line per size and the exponent | exponent above the gate → non-zero exit | R5 |
| UC-04 | A maintainer knows where time goes | maintainer | profile | the benchmark | ranked top costs | — | R6 |
| UC-05 | An editor integration avoids a full reparse **if and only if** it is needed | editor author | Rust API | previous tree, an edit, new source | an equal tree, faster | edit inside `define` → full parse, same result | R8, R9 |

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | 1 (body counter; front matter says 2 — `PROP-2026-0004` OQ-05, still unreconciled) | 2026-10-07 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 | Applies | no grammar added | PASS | — |
| GOAL-002 | Applies | R4: output byte-identical | PASS | — |
| PHIL-001 | Applies | every problem reproduced by measurement before this was written | PASS | — |
| PHIL-002 | Applies | the earlier "roughly linear" answer is corrected above | PASS | — |
| CODE-001 | Applies | no commit until asked | PASS | — |
| CODE-003 | Applies | R10 | PASS | — |
| ARCH-001 | Applies | R1–R3 change no public type; R9 adds a method | PASS | — |
| ARCH-002 | Applies | no dependency; a profiler is a dev tool, not a dependency | PASS | — |
| ARCH-003 | Applies | R9 must not weaken the depth bound or the left-recursion guard | PASS | gate in R9 |
| QUAL-001 | Applies | R4 | PASS | — |
| QUAL-002 | **Applies** | thresholds below, frozen at approval | PASS | M-01…M-05 |
| QUAL-003 | Applies | each new test must fail when its fix is reverted | PASS | — |
| GATE-001 / GATE-002 | Applies | full gate plus corpus | PASS | — |

## Implementation Design

### Methods by Use Case

| Change | UC | Method | Req |
|---|---|---|---|
| C-01 | UC-01 | `inner_evaluator.rs`: in the `append` arms, take `&mut Vec` from the existing slot (`get_mut`) and `push`; create the list only when absent | R1, R4 |
| C-02 | UC-02 | same for `merge`: `get_mut` the object and extend | R2, R4 |
| C-03 | UC-01 | decide `prepend` (deque vs documented O(n)); implement the chosen one | R3 |
| C-04 | UC-03 | `rust/devtools`: a `scalebench` bin running parse-only and run at ≥3 sizes, printing µs/line and the log-log exponent, exiting non-zero above a threshold | R5 |
| C-05 | UC-04 | record a profile (sampling profiler on the benchmark) and a ranked cost list | R6 |
| C-06 | UC-05 | **only if R8 says build:** `Library::parse_incremental` | R8, R9 |
| C-07 | all | docs | R10 |

### Why in-place is safe

`Val` is a plain value type in Rust: there is no `Rc` and no aliasing of a stored list, so mutating it in place is observationally identical to the copy the Go original needed for its shared slices. R4's aliasing test pins this: read the list into a local, append, and check the local is unchanged.

### Sketch for C-06 (not approved; shown so the gate is concrete)

```text
   old source ──► [stmt1][stmt2][stmt3 ···edit···][stmt4][stmt5]
                    │      │         │               │      │
   reuse as-is ─────┴──────┘         │               └──────┘ reuse, shift spans
                                     ▼
                            re-parse only this statement
                            (statements are already the recovery boundary)

   unsound cases → full parse:  `define` blocks (change the grammar),
                   `preprocess` directives, an edit that merges or splits statements,
                   `block_close_seq` blocks (no newline separators)
```

Top-level statements are already the points where error recovery resynchronizes, which is why
they are the natural reuse boundary. Spans are absolute line/column, so reused nodes *after* the
edit need a line shift: that is O(statements), not O(tokens), but it is not free (OQ-03).

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected |
|---|---|---|---|
| **A. In-place state ops + benchmark + gate (recommended)** | fixes the one proven problem cheaply; measures before building the speculative part | defers incremental reparse | **Selected** |
| B. Persistent (structural-sharing) list/map for `Val` | O(1) snapshots, no aliasing worry | a new dependency (ARCH-002) or a hand-rolled structure for a problem in-place solves | Rejected |
| C. Rewrite as a table-driven parser | highest throughput | abandons the declared-grammar model (GOAL-001); enormous | Rejected |
| D. Packrat memoization | removes repeated sub-parses | memory cost; no evidence repeated work dominates | Deferred to OQ-02 |
| E. Build incremental reparse now | solves the editor case | built on an unmeasured assumption; correctness risk (R9 equality) | Rejected until the R8 gate says so |
| F. Do nothing | zero cost | leaves a 13× slowdown at 240k lines | Rejected |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback |
|---|---|---|---|---|
| RK-01 | In-place mutation changes observable aliasing | a read-before-append value changes | R4 aliasing test, written first | revert C-01 |
| RK-02 | `prepend` deque changes list representation (`Val::List(Vec)` is public) | public type change (ARCH-001) | keep `Vec`; document O(n) (R3) instead | n/a |
| RK-03 | The benchmark flakes on a noisy host | false failures | gate on the **exponent** and interleaved baseline, not absolute time | loosen threshold with an ADR |
| RK-04 | Incremental reparse returns a tree unequal to a full parse | wrong editor results | R9 property test; full-parse fallback | don't ship C-06 |
| RK-05 | A constant-factor tweak changes output | regression | R7 + R4 goldens | revert that tweak |

## Security Impact

None.

## Operational Impact

Faster, with no change in behaviour or output.

## Compatibility Impact

None intended (R4). `Library::parse_incremental`, if built, is additive.

## Test and Validation Design

| ID | Type | Req | Scenario | Procedure | Expected |
|---|---|---|---|---|---|
| T-01 | unit | R4 | aliasing | read a list into a local, `append`, assert the local unchanged | passes before **and** after C-01 |
| T-02 | unit | R1 | in-place append | append 100 000 items; assert the result list and that time is within the gate | list equal; within M-01 |
| T-03 | unit | R2 | in-place merge | merge repeatedly; assert result | equal |
| T-04 | integration | R4 | corpus | `cargo test --test golden` | 131 passed, 8 skipped, 0 failed |
| T-05 | tooling | R5 | benchmark runs | `scalebench` at 3 sizes | prints µs/line and exponent |
| T-06 | evidence | R6 | profile recorded | profile the benchmark | ranked list in a `TEST` doc |
| T-07 | decision | R8 | editor-file budget | time a full parse of a 5 000- and a 50 000-line file | recorded against the budget |
| T-08 | property | R9 | incremental == full | random edits | equal trees (only if built) |
| T-09 | regression | GATE-001 | full gate | build, clippy `-D warnings`, test, `mkdocs --strict` | exit 0 |

Every new test must fail when its fix is reverted (QUAL-003); T-01 is the exception by design — it
guards behaviour that must **not** change, so it must pass both before and after.

### Measurement and Validation (QUAL-002)

Baselines are today's measurements (P-01, P-02) on the v0.24.0 release build. The thresholds
below are **proposed**; they freeze when this proposal is approved, before C-01 is written.

| ID | Measurement | Baseline | Test Command | Proposed Threshold |
|---|---|---|---|---|
| M-01 | `capy run`, `transpile-py` ×16 000 (240 000 lines, with `append`) | **24 289 ms** | `capy run samples/transpile-py/lib.capy <x16000>` | ≤ **1.5×** the same script without `import` lines at the same size (baseline ratio today **13.1×**: 24 289 / 1 857 ms, line counts 240k vs 224k) |
| M-02 | size exponent, run with `append`, 4 000 → 16 000 reps | **1.8** (log₄ of 11.8×) | `scalebench` | ≤ **1.1** |
| M-03 | size exponent, parse only | **≈1.0** (3.9× for 4×) | `scalebench` | ≤ **1.1** (no regression) |
| M-04 | parse-only µs/line, `transpile-py` ×4 000 | **3.7 µs** | `capy ast` | ≤ **+10 %** |
| M-05 | run µs/line without list state, ×4 000 | **8.2 µs** | `capy run` | ≤ **+10 %** |
| M-06 | existing transpile bench | 190.9–195.9 µs (`REL-0.23.0`, `TEST-2026-0013`) | `nativebench` | ≤ **+10 %** |
| M-07 | `capy-core` direct dependencies | 1 | `cargo tree -p capy-core --depth 1` | exactly 1 |
| M-08 | wasm size | 1 390 420 B (`TEST-2026-0013`) | wasm build, `ls -l` | ≤ **+3 000 B**; ceiling 1 410 035 |

Editor latency budget for R8 (proposed): a full parse of a **50 000-line** file ≤ **100 ms**
(today ≈ 190 ms at 3.7 µs/line, so the gate would currently say *build*; a 5 000-line file at
≈ 19 ms would say *don't*). The budget itself needs the owner's decision (OQ-01).

## Documentation, Demo and Release Impact

| Artifact | Path | CRUD | Required Content | Gate |
|---|---|---|---|---|
| Complexity notes | `docs/library-authoring.md` (state operations) | UPDATE | `append`/`merge` O(1); `prepend` per R3 | T-09 |
| Benchmark | `rust/devtools/` | CREATE | `scalebench` | T-05 |
| Perf record | `program_docs/testing/` | CREATE | scaling + profile `TEST` documents | T-06 |
| Embedding (if C-06) | `docs/embedding.md` | UPDATE | `parse_incremental` | T-08 |
| Release | `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | entry | GATE-001 |
| Version | `rust/Cargo.toml` + pins | UPDATE | minor bump (additive) | Release Management |

## Requirements Alignment

| Req | UQ | Goal | UC | Changes | Tests |
|---|---|---|---|---|---|
| R1 | UQ-02, UQ-03 | G-01 | UC-01 | C-01 | T-01, T-02 |
| R2 | UQ-03 | G-01 | UC-02 | C-02 | T-03 |
| R3 | UQ-03 | G-01 | UC-01 | C-03 | T-02 |
| R4 | UQ-03 | G-01 | UC-01 | C-01, C-02 | T-01, T-04 |
| R5 | UQ-03 | G-02 | UC-03 | C-04 | T-05 |
| R6 | UQ-02 | G-03 | UC-04 | C-05 | T-06 |
| R7 | UQ-03 | G-03 | UC-04 | per change | T-04 |
| R8 | UQ-02 | G-04 | UC-05 | decision | T-07 |
| R9 | UQ-02 | G-04 | UC-05 | C-06 | T-08 |
| R10 | UQ-03 | all | — | C-07 | T-09 |

## Plan Strategy and Estimated Work

```text
  INC-1  R1 R2 R4 R10(part)   in-place state ops + aliasing test        small, proven   ──► release
  INC-2  R5 R6                scalebench + profile                        small           ──► release
  INC-3  R7                   constant-factor changes the profile ranks    open-ended      ──► per change
  INC-4  R8 (R9)              measure editor budget; ADR build/don't-build  gate
```

INC-1 can ship alone and removes the only measured defect. INC-3 is deliberately unspecified:
it is whatever INC-2's profile ranks first, justified by a measurement.

## Open Questions

| ID | Question | Why It Matters | Proposed Default |
|---|---|---|---|
| OQ-01 | What is the editor latency budget, and for what file size? | decides R8 | 100 ms for 50 000 lines; owner to confirm |
| OQ-02 | Is memoization worth it? | possible large win on deep ordered choice | no, until the profile shows repeated sub-parses dominate |
| OQ-03 | Are absolute line/column spans acceptable for reuse, or must spans become relative? | cost and complexity of C-06 | keep absolute; shift in O(statements) |
| OQ-04 | Do we want a measured comparison against other tools (hand-written parser, a PEG crate, tree-sitter) on one shared grammar? | turns the earlier reasoning into data | yes, as a separate `RES` document, not part of this release |
| OQ-05 | Should `prepend` change representation? | `Val::List(Vec)` is public | no — document O(n) (R3) |

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | pending | | Confirm OQ-01 and the proposed thresholds before they freeze |
| Reviewer | Release Management | pending | | |

## Related Documents

- `REL-0.24.0`, `PROP-2026-0004`, `PROP-2026-0005`
- `TEST-2026-0010`, `TEST-2026-0013` — the existing one-size benchmark and the shared wasm allowance
- `STD-2026-0000`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial draft, from measurements taken the same day |
