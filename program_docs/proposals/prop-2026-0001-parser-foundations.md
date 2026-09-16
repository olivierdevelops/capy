---
document_id: PROP-2026-0001
title: Parser Foundations — Spans, Error Recovery, Structured AST Output and Expression Trees
document_type: proposal
status: draft

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 4

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
  - capy-wasm-abi
  - docs

affected_versions:
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Defines five additive parser changes — a left-recursion guard, source spans on every AST node, actionable parse diagnostics with error recovery, a structured AST output surface, and infix operator precedence — so that a language can be hosted on Capy and its users can read its error messages.

reason: A downstream consumer (Glang) cannot host a language on Capy today: a left-recursive library aborts the process, interior AST nodes carry no source position, a failed parse reports only "no function matched" and stops at the first error, the parse result is reachable only as rendered text, and infix arithmetic never becomes a tree.

related_documents: []

supersedes: null
superseded_by: null

tags:
  - parser
  - ast
  - spans
  - diagnostics
  - error-recovery
  - json

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Parser Foundations — Spans, Error Recovery, Structured AST Output and Expression Trees

> **Status:** Draft
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0 and later
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

A consumer building a language (Glang) on Capy reported four sets of blockers across three documents. All
were verified against the current tree; one turned out to be a process-aborting crash. This proposal covers
five additive changes in dependency order: a **left-recursion guard**, **spans** on every AST node,
**actionable diagnostics with error recovery**, **structured AST output**, and **infix operator precedence**.
It explicitly excludes the dataflow-analysis capabilities reported separately, which do not belong in Capy.

---

## Decision Requested

Approve five additive engine changes, in this order:

0. **Left-recursion guard** — detect a left-recursive library and report it, instead of overflowing the stack
   and aborting the process.
1. **Spans** — a `Span` on every `FuncCall` and every `CaptureValue`, with interior nodes carrying real
   positions instead of zeros.
2. **Diagnostics** — furthest-failure tracking and a typed expectation vocabulary, so a failed parse says
   what was expected and where.
3. **Error recovery** — statement-level resync, error nodes, and a `ParseResult` carrying a partial tree plus
   all diagnostics.
4. **Structured AST output** — `Library::parse()` in the embedding API, `capy ast` on the CLI, and a
   documented JSON schema.
5. **Operator precedence** — infix arithmetic and boolean operators in value-expression positions.

Approval authorizes the **behaviour and contracts** defined here and the writing of implementation plans.
It does **not** authorize coding, does not approve the JSON schema as stable beyond what R9 states, does not
fix the cascade constants of R19, and does not approve anything under Non-Goals.

---

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | Four capabilities Capy lacks for Glang: flow-sensitive lattice with joins/narrowing/fixpoints/widening; whole-program region and call graphs; rejection on analysis rather than grammar; per-path specialization and monomorphization. | `.ignore/needs.md`, 2026-09-14 | A capability comparison, not a feature request. Assessed; all four out of scope here (Non-Goals 1–3, and Non-Goal 4 for the rejection item). |
| UQ-02 | "NO OPERATOR PRECEDENCE … `value_parser.rs` handles ONE comparison; no + - * / at all ⇒ `a * b + c[i].field` cannot become a tree ⇒ fatal: every Glang analysis operates on expression trees". | `.ignore/needs2.md` §1, 2026-09-16 | Correct that no arithmetic exists. "Cannot become a tree" is too strong — function-as-type captures already produce trees (P-05) — but that workaround is unusable because of UQ-03 and crashes because of UQ-06. |
| UQ-03 | "SPANS ARE STATEMENT-LEVEL, AND ZERO ON NESTED NODES … `ast.rs:34-38` line/col on FuncCall only; `make_parser.rs:604` nonterminal children get line: 0, col: 0; no byte offsets, no end positions, no per-capture spans ⇒ fatal … diagnostics and provenance chains are unbuildable." | `.ignore/needs2.md` §2, 2026-09-16 | Verified exactly, including the literal `line: 0, col: 0`. Reporter calls this the "make-or-break detail"; the error-recovery note (UQ-06) independently calls spans "a HARD prerequisite". |
| UQ-04 | "NO STRUCTURED OUTPUT — `capy.rs:116 run() -> String`. No --json/--ast/--emit. No serde … the JSON seam I proposed doesn't exist." | `.ignore/needs2.md` §3, 2026-09-16 | Verified. The quoted engine line is from `docs/integration-guide.md` and is accurate. |
| UQ-05 | "so you suggest that we dont do anything about?" | Conversation, 2026-09-16 | Asked after an assessment that two of UQ-01's items are not Capy's job. Resolved as: act on UQ-02/03/04/06, decline UQ-01. This proposal is that scope decision made explicit. |
| UQ-06 | "Error Recovery for Capy … Capy's matcher is all-or-nothing per statement. When no library shape matches, the user learns that nothing matched — which is the one thing they already knew." Three deficiencies: **no expectation**, **no position**, **no recovery**. Four mechanisms: furthest-failure tracking, an expectation vocabulary, statement-level resync, error nodes. Plus a stated ordering whose step 0 is a "left-recursion guard — correctness bug; crashes on the ladder pattern Capy currently recommends". | `garantees_lang/.ignore/capy_error.md`, 2026-09-16 | The most consequential of the four documents. Its step-0 crash claim was verified and is now P-07. Its `ParseResult` design supersedes revision 1's `Library::parse` signature — see Open Question 3 and R6. Its closing argument: this is "the difference between a transpiler engine and something a language can be built on". |

---

## Problem and Evidence

### Current User Journey

```text
[Glang user] -> writes `fn add(x: int, y: int {`        (one missing paren)
     -> capy tries every shape, all fail
     -> [gap] "no library function matches token \"fn\""
     -> [consequence] the message names what failed, never what was wanted,
                      points at the statement not the token, and parsing
                      STOPS — one error fixed per run

[Glang author] -> declares `expr`/`term` functions to get an expression tree
     -> capy check says: ok — 2 function(s)
     -> capy run  says: thread 'main' has overflowed its stack
                        fatal runtime error: stack overflow, aborting
     -> [consequence] the recommended ladder pattern ABORTS THE PROCESS,
                      and an embedder cannot catch it as Err

[Glang author] -> wants the parse result as data
     -> capy exposes run() -> String only
     -> [consequence] hand-write JSON in templates, re-parse it, no schema,
                      and the spans still aren't there
```

| Problem | Affected Users | Evidence and Inline Source | Consequence | UQ IDs |
|---|---|---|---|---|
| P-01 | Consumers doing any analysis | `rust/src/domain/ast.rs` — `FuncCall` carries `pub line: usize` / `pub col: usize` only: no end position, no byte offset. | A node is locatable only to a start line/column; no range highlighting, no slicing back to source bytes. | UQ-03, UQ-06 |
| P-02 | Consumers doing any analysis | `rust/src/domain/ast.rs` — `CaptureValue { is_expr, expr, text, sub }`. No span field of any kind. | The value a rule is *about* has no position. An expectation at a capture has nowhere to point. | UQ-03, UQ-06 |
| P-03 | Consumers using function-as-type trees | `rust/src/orchestrator/features/make_parser.rs` — nested `FuncCall` constructed with literal `line: 0, col: 0`. | The only mechanism for real expression trees yields nodes that all claim line 0. A label on an opening delimiter inside a nonterminal points at 0:0. | UQ-03, UQ-02, UQ-06 |
| P-04 | Consumers wanting structure | `rust/src/capy.rs:116` — `run(&self, &str) -> Result<String, CapyError>`. `rust/Cargo.toml` — one dependency, `regex`; no serde. CLI has no `ast` subcommand. | The parse result is reachable only as rendered target text. `make_parser::parse` is public but undocumented and unsupported. | UQ-04 |
| P-05 | Consumers writing infix source languages | `rust/src/orchestrator/features/value_parser.rs` — no `+ - * /`, no precedence climbing; exactly one non-associative comparison at line 36. `docs/language-reference.md:98` states multi-token arithmetic is not parsed as one expression. | Infix arithmetic cannot be written in a condition or `${…}`; in *source* it is text or hand-built grammar rules. | UQ-02 |
| P-06 | Every Capy user, not only Glang | `rust/src/orchestrator/features/make_parser.rs:390` — the total-failure message is `no library function matches token "…"`. Matching is all-or-nothing per statement; there is no record of which shape got furthest, no expectation set, and no resync. | Three deficiencies in one message: **no expectation**, **no position**, **no recovery**. A language whose users read its errors cannot be hosted. | UQ-06 |
| P-07 | Any library using recursive function-as-type captures | **Verified 2026-09-16.** A library with `function expr / arg capture lhs expr / …` passes `capy check` (`ok — 2 function(s)`) and then, on `capy run`, prints `thread 'main' has overflowed its stack` / `fatal runtime error: stack overflow, aborting` and exits **rc=134**. | Not a bad message — an **uncatchable process abort**. An embedder cannot convert it to `Err`; in a server it takes down the host. Static validation does not detect it. | UQ-06 |
| P-08 | Editor/LSP integrators | `capy.rs:116` returns `Result<String, CapyError>` — on failure there is no tree at all. | An editor is always parsing broken code. With no partial tree there is no completion, hover or highlighting below the first typo. | UQ-06 |

**Verification note.** P-01…P-05 were read from the current tree during assessment. P-07 was reproduced.
P-05 corrects UQ-02's conclusion: trees are possible today via `CaptureValue.sub`, which is why P-03 and
P-07 — not P-05 — are the load-bearing defects.

---

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-00 | A left-recursive library is rejected with a normal diagnostic; the engine never aborts the process. | P-07 | Detect left recursion in the loader, or bound recursion depth at parse time, and return `CapyError`. | The P-07 reproduction exits 1 with a message naming the cycle; rc is never 134. |
| G-01 | Every AST node an external consumer can reach reports a real, non-zero source range. | P-01, P-02, P-03 | Add `Span`; put one on `FuncCall` and `CaptureValue`; populate interior nodes from their first/last token. | A three-level tree has distinct, correct, non-zero, properly nested spans, asserted by test. |
| G-02 | A failed parse says what was expected, and where, with secondary labels. | P-06 | Furthest-failure tracking with expectation union; a typed expectation vocabulary; two-span rendering. | The `fn add(x: int, y: int {` case names `)` and labels the unclosed `(`. |
| G-03 | Parsing continues after an error; a partial tree and all diagnostics are returned. | P-06, P-08 | Statement-level resync with delimiter-balance priority; error nodes; `ParseResult`. | A file with three broken statements yields exactly three diagnostics and a tree containing the intact statements. |
| G-04 | An external analyzer consumes the parse result as structured data, with spans, without re-parsing text. | P-04 | `Library::parse()`; `capy ast [--json]`; a documented schema. | A separate crate parses a script, walks the tree, and reports a diagnostic positioned on a nested operand. |
| G-05 | Infix arithmetic and boolean operators parse into a correctly-shaped tree in value positions. | P-05 | Precedence-climbing layer in `value_parser.rs`, with documented precedence and associativity. | `a * b + c` parses as `(a*b)+c`, asserted by test. |

### Explicit Non-Goals and Boundaries

1. **No dataflow analysis in Capy.** No lattice, joins, branch narrowing, loop fixpoints or widening
   (UQ-01). Capy stays a syntax-directed transducer. These belong in consumer code over the AST G-04 exposes.
2. **No whole-program graphs.** No region graph, call graph or owner assignment (UQ-01).
3. **No analysis-driven emission.** No per-path specialization or monomorphization; the `file` set stays
   fixed at library-load time (UQ-01).
4. **No `error` statement in the inner DSL.** UQ-01's "reject on ANALYSIS" item is unaddressed here.
   A related latent defect was observed — every bare-call statement in a function body fails with
   `expected statement, got "("`, naming a token absent from the input, while the same form works in command
   bodies — but it was not isolated. Both need their own proposal.
5. **No change to zero-default-grammar.** G-05 adds operators to **value expressions** (the engine's own
   fixed grammar), not a built-in expression grammar for *source* languages.
6. **No serde dependency in `capy-core`** (R8).
7. **No semantic-error reporting.** This proposal is parsing only (UQ-06, "What this does not cover").
8. **No error repair** — inserting a missing token and continuing *within* a statement. Worth doing after
   G-03, not before (UQ-06).
9. **No incremental reparse.** Error nodes are its prerequisite; incrementality is a separate design (UQ-06).
10. **No doc-comment *semantics*.** R27 requires comments to be **retained and attached**, nothing more. Deciding that `///` means "documentation", parsing structured annotations out of it, or attaching trailing rather than leading comments is out of scope — see Open Question 12.

---

## Proposed User Journey

```text
[Glang user] -> capy run lib.capy main.gl      (three separate mistakes)
     -> error[E0012]: expected `)` to close the parameter list
          --> main.gl:1:22
           |
         1 | fn add(x: int, y: int {
           |       -              ^ expected `)` here
           |       |
           |       unclosed `(` opened here
        error[E0007]: expected an expression after `+`   --> main.gl:2:16
        error[E0012]: expected `)` …                     --> main.gl:5:16
        3 errors
     -> [visible result] every mistake in one run, each pointing at a token

[Glang author] -> capy ast main.gl --json
     -> {"schema_version":1,"diagnostics":[],"tree":{...spans on every node...}}
     -> [analysis in Rust] walk tree, run lattice, report on an operand
              |
              +-> [left-recursive library] error: left recursion in `expr`
                                           exit 1 — never a stack overflow
```

---

## Requirements

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal IDs |
|---|---|---|---|---|---|
| R0 | A library whose function-as-type captures are left-recursive is rejected with a `CapyError` naming the cycle. The engine never aborts on recursion depth. | functional | P-07, verified: rc=134, `fatal runtime error: stack overflow`. UQ-06 orders this step 0. | The P-07 reproduction exits 1 with a message naming `expr`; a fuzz/depth test never aborts. | G-00 |
| R0b | Detection happens at library-load time if statically decidable; otherwise a bounded parse depth converts the overflow into a `CapyError`. | functional | `capy check` currently reports `ok` on the crashing library, so load-time validation is the better gate where possible. | `capy check` fails on the P-07 library, or — if undecidable — `capy run` returns a depth-limit error. | G-00 |
| R1 | A `Span` type exists in `domain::ast` carrying start line/col, end line/col, and start/end byte offsets. | data | UQ-03 names "no byte offsets, no end positions"; UQ-06 needs `primary: Span` with start **and** end. | `Span` is public, `Copy`, six fields readable externally. | G-01 |
| R2 | `FuncCall` carries a `span` covering the whole statement, including block body and closer. | data | P-01. | For a block function, `span.end` ≥ the closer's last token. | G-01 |
| R3 | `CaptureValue` carries a `span` covering exactly the tokens that produced the value. | data | P-02; UQ-06: "an expectation at a capture has nowhere to point". | For `f a b`, the two capture spans differ and neither overlaps. | G-01 |
| R4 | Every `FuncCall` from a function-as-type capture carries a real span; no node reaches a consumer with a zero span. | functional | P-03 — the reported make-or-break item. | Three-level tree: non-zero, strictly nested spans; test asserts no reachable node has `line == 0`. | G-01 |
| R5 | Existing `line`/`col` render locals keep their current values and meaning. | compatibility | `docs/inner-dsl.md` documents them; samples rely on them; `CLAUDE.md` requires additive changes. | Golden corpus passes unchanged. | G-01 |
| R14 | The matcher records the furthest-progressing attempt: strictly-further replaces, equal-distance **unions** expectations, nearer is discarded. | functional | UQ-06 Mechanism 1, with that exact update rule. Named "the single highest-value change, and the cheapest". | Three shapes dying at one token produce a message listing all three alternatives. | G-02 |
| R15 | A typed expectation vocabulary exists covering at minimum: `Literal`, `Kind`, `Nonterminal`, `OneOf`, `CloseDelim { open_at: Span }`, block `end`. | data | UQ-06 Mechanism 2, enumerated against existing directives (`arg literal`, `arg capture`, `options`, `group_close`). | Each listed matcher rejection contributes its typed expectation; asserted per variant. | G-02 |
| R16 | `OneOf` reuses the existing "did you mean" hint machinery used for type `options`. | UX | UQ-06: "you already compute Levenshtein … Reuse that here verbatim". | A typo'd enum value produces `help: did you mean \`write\`?`. | G-02 |
| R17 | `CloseDelim` carries the span of the **opening** delimiter, taken from the existing `group_open`/`group_close` depth counter rather than discarded. | data | UQ-06 Mechanism 2 — this is what makes the two-span message possible. | The unclosed-paren case labels the opening `(`. | G-02 |
| R18 | A diagnostic carries severity, a stable machine-readable code, a primary span, secondary labels, a message and optional help. | API | UQ-06 "The diagnostic itself" — `CapyError` today has `{line, col, msg, hint, file, plain}` and no severity, code or labels. | `Diagnostic` and `Label` are public; `format_with_source` renders primary plus at least one label. | G-02 |
| R19 | Recovery suppresses a new diagnostic within K tokens of the previous one and caps output at N, then reports the remaining count. K and N are **configurable, not fixed by this proposal**; UQ-06 suggests K≈3, N≈20. | UX | UQ-06 "Cascade suppression". Marked configurable because the constants are a tuning decision, not a contract. | One broken construct → one diagnostic; a 60-error file → N shown plus "and 40 more errors". | G-03 |
| R20 | On statement failure the parser emits a diagnostic, pushes an error node covering the skipped span, resyncs, and continues. | functional | UQ-06 Mechanism 3. | A file with three broken statements yields exactly three diagnostics. | G-03 |
| R21 | Resync honours delimiter balance **first**, then statement boundary, then block boundary, then EOF. Statement-boundary detection uses the known set of first-literals from the shape table. | functional | UQ-06 Mechanism 3, resync priority order. Rule 1 is what stops a missing `)` eating the file. | A missing `)` on line 2 of a 40-line file does not consume the remainder. | G-03 |
| R22 | The parse result carries the recovered statements **and** the skipped regions, plus a diagnostics list. Each error node records its span, the verbatim tokens and its diagnostic index. `Block.stmts` **keeps its type** `Vec<FuncCall>`; error nodes live in a new parallel `Block.errors: Vec<ErrorNode>`, ordered with the statements by span. | data | UQ-06 Mechanism 4 — "Recovery is only half useful if the result is discarded". Additive representation chosen per review finding 2: changing `stmts` to `Vec<Node>` is source-breaking and `#[non_exhaustive]` does not protect a field's *type*. | Statements after a broken one are present and intact in `stmts`; the skipped region appears in `errors`; an existing walker over `stmts` still compiles and sees a degraded-but-correct statement list. | G-03, G-04 |
| R23 | Emission refuses output when any error node is present; that is the default and it is explicit. | functional | UQ-06: "an emitter can choose to refuse output when any error node is present (the safe default)". | `capy run` on a file with an error node writes no output and exits non-zero. | G-03 |
| R6 | `Library::parse(&self, script_src) -> ParseResult` exists in the public embedding API, where `ParseResult { tree: Block, diagnostics: Vec<Diagnostic> }`. `diagnostics` empty means a clean parse. | API | P-04 and UQ-06's API shape. **This supersedes revision 1**, which specified `Result<Block, CapyError>`; adopting `ParseResult` now avoids a breaking change when recovery lands. | An external crate calls it, reads both fields, using only `capy_core::capy` and `capy_core::domain` imports. | G-04 |
| R7 | `capy ast <library> <script>` prints a readable tree; `--json` prints machine-readable JSON to stdout and nothing else. | CLI | P-04: no `--json`/`--ast`/`--emit` exists. | `capy ast … --json \| jq .` succeeds, exit 0, stderr empty on success. | G-04 |
| R8 | JSON is produced by `capy-core`'s existing writer, not serde; `capy-core` keeps exactly one dependency. | data | `rust/Cargo.toml` declares only `regex`; `gojson.rs` already writes JSON. | `cargo tree -p capy-core --depth 1` lists one dependency. | G-04 |
| R9 | The AST **and diagnostics** JSON schema is documented with a stability policy and a `schema_version` field. | docs | UQ-04: "no schema". | `docs/ast-json.md` exists, is in the mkdocs nav, documents every field including spans and diagnostics. | G-04 |
| R24 | `Library::run` keeps its exact signature and behaviour: first error, no output. | compatibility | UQ-06: "`run` keeps its signature and its behaviour … so nothing breaks." | Golden corpus and all error goldens byte-identical. | G-03, G-04 |
| R10 | Value expressions support infix `*`, `/`, `%`, `+`, `-`, comparison, `and`, `or`, with documented precedence and left-associativity, and parenthesised grouping. | functional | P-05 / UQ-02. | `a * b + c` → `(a*b)+c`; `a + b * c` → `a+(b*c)`. | G-05 |
| R11 | Comparison becomes precedence-ordered rather than a single non-associative step. | functional | `value_parser.rs:36`. | `a + 1 == b * 2` → `(a+1) == (b*2)`. | G-05 |
| R12 | No library that parses today changes behaviour, and no program that parses today produces a diagnostic. | compatibility | `CLAUDE.md` additive rule; UQ-06 Test J — "the gate that makes this safe to ship". | Golden corpus, `capy check` over 117 libraries, wasm check all unchanged; every valid program yields zero diagnostics and no error nodes. | all |
| R13 | The wasm module continues to build and its size does not regress materially. | operations | `.github/workflows/docs.yml` publishes `capy-wasm-abi`; 1 342 891 bytes on 2026-09-16. | Within the M-02 threshold. | G-04 |
| R26 | The furthest record carries a **context frame** naming the shape and the argument position being matched, and the diagnostic renders it. | data | UC-06 step 3 requires it and the risk table names it as the sole mitigation for "the furthest shape is not the one the user meant". Review finding 4: it appeared only in narrative, so it could be dropped without failing a test — scope creep by `DOCUMENTATION.md` §21. | A diagnostic for a failure inside a nested shape reads "in the parameter list of `fn_decl`"; asserted by T-28. | G-02 |
| R27 | The lexer **retains** comments as trivia, and each `FuncCall` exposes the spans of the comments immediately preceding it. **A node's own `span` EXCLUDES its attached comments**; the comment spans live only in `leading_comments`. Retention only — no interpretation, no doc-comment semantics. | data | Review finding 3: neither required nor deferred, so a consumer cannot tell which it is getting. Verified: `TokenKind` has **no** `Comment` variant and `tokenize_line` discards comments, so this needs lexer work — the same pass as C-00, and far cheaper now than reopening the lexer later. | `Library::parse` on a script with a leading `# note` exposes that comment's span on the following node; the node's `span.start` is at its first **code** token, not at the comment (T-31); parsing and rendering are unchanged (T-29, T-11). | G-01, G-04 |
| R28 | `docs/embedding.md` states explicitly that `Block.stmts` alone does not indicate a successful parse: a consumer must check `ParseResult.diagnostics` (or `Block.errors`) before trusting the tree. | docs | Review residual 2: this is the accepted cost of the additive `Block.errors` design (R22) — an embedder walking `stmts` and ignoring `errors` silently gets a **partial** tree. One sentence in the guide now, versus a support question later. | The embedding guide contains that statement adjacent to the `parse` example. | G-03, G-04 |
| R25 | Furthest-failure tracking does not materially slow parsing. | operations | UQ-06 risk table: "one integer compare per rejection; the set only grows at ties. Measure on the sample corpus". | Within the M-01 threshold. | G-02 |

---

## Use Cases

### Use-Case Catalogue

| ID | User Outcome | Actor | Surface and Trigger | Preconditions / Environment | Inputs | Outputs / Visible Result | Negative and Error Paths | Goal IDs | Requirement IDs | Test IDs |
|---|---|---|---|---|---|---|---|---|---|---|
| UC-01 | Locate a nested operand precisely | Analyzer author | Rust API; `Library::parse` | capy-core dependency; library using function-as-type | library + script source | `ParseResult` with non-zero nested spans, empty diagnostics | malformed script → diagnostics non-empty, tree still returned | G-01, G-04 | R1–R4, R6 | T-01, T-02, T-03 |
| UC-02 | Inspect a parse from the shell | Language author | CLI; `capy ast lib s` | capy installed | lib + script paths | indented tree with ranges | missing file → stderr, exit 1 | G-04 | R7 | T-04, T-06 |
| UC-03 | Feed the AST to an external tool | Analyzer author | CLI; `capy ast … --json` | as UC-02 | paths + `--json` | one JSON document on stdout | parse error → diagnostics array in the same document | G-04 | R7, R8, R9 | T-05, T-06 |
| UC-04 | Write arithmetic in a condition | Library author | `.capy` library body | none | `if a * b + c > limit` | evaluates with standard precedence | type mismatch → existing value-error path | G-05 | R10, R11 | T-07, T-08 |
| UC-05 | Report a diagnostic on an operand | Analyzer author | consumer's own CLI | UC-01 satisfied | AST + analysis result | caret under exactly the operand | span absent → impossible per R4 | G-01 | R1–R4 | T-03, T-09 |
| UC-06 | Read an actionable parse error | **Glang end user** | `capy run` (or the consumer's CLI) | a library and a broken script | `fn add(x: int, y: int {` | `expected \`)\` to close the parameter list`, caret at the token, label on the opening `(` | no expectation available → generic message, still positioned | G-02 | R14–R18 | T-14, T-15, T-16, T-17 |
| UC-07 | Fix every mistake in one run | **Glang end user** | `capy run` | script with several independent errors | 3 broken statements in 40 lines | exactly 3 diagnostics, then `3 errors`; no output emitted | 60 errors → N shown + "and 40 more" | G-03 | R19–R21, R23 | T-18, T-19, T-20, T-22 |
| UC-08 | Keep editing broken code | **Editor / LSP integrator** | `Library::parse` on every keystroke | mid-edit buffer | buffer with one syntax error | tree with an error node; statements after it intact | whole file unparseable → tree of error nodes, never a panic | G-03, G-04 | R22, R6 | T-21 |
| UC-09 | Get told about left recursion | Library author | `capy check` / `capy run` | library with `arg capture lhs expr` inside `expr` | that library | diagnostic naming the cycle; exit 1 | undecidable statically → depth-limit error at parse time | G-00 | R0, R0b | T-23, T-24 |
| UC-10 | Read a node's doc comment | **Editor / doc generator** | `Library::parse` | script with comments above a statement | `# owner: platform` above a `service` line | the comment's span on the following node, retrievable from the tree | comment attached to nothing (trailing / EOF) → retained as trivia, unattached | G-01, G-04 | R27 | T-29 |

### UC-06 — Actionable parse error

1. User runs the consumer's CLI, which calls `Library::run` or `parse`.
2. No shape matches the statement.
3. The matcher's furthest record holds token index, unioned expectations and a context frame.
4. A diagnostic is built: code, primary span at the offending token, label at the opening delimiter.
5. `format_with_source` renders primary plus label.

Branch: two shapes tie at the same token → expectations union → "expected an expression, `(`, or an identifier".

#### CLI Contract

```text
$ capy run glang.capy main.gl
error[E0012]: expected `)` to close the parameter list
  --> main.gl:1:22
   |
 1 | fn add(x: int, y: int {
   |       -              ^ expected `)` here
   |       |
   |       unclosed `(` opened here
   |
$ echo $?
1
```

### UC-09 — Left recursion reported, not fatal

```text
BEFORE (verified 2026-09-16)        AFTER (required)
  $ capy check lib.capy               $ capy check lib.capy
  ok — 2 function(s)                  error: left recursion: `expr` can match
  $ capy run lib.capy s.capy                 itself at argument 1 without
  thread 'main' has overflowed               consuming a token
       its stack                        --> lib.capy:4:5
  fatal runtime error: stack            $ echo $?
       overflow, aborting               1
  rc=134   ← uncatchable by an embedder
```

---

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| `program_docs/standards/index.md` | **does not exist** | 2026-09-16 |

`program_docs/` is created by this proposal; no project-standards set exists, so no rule IDs
(`ARCH-nnn`, `QUAL-nnn`, `PHIL-nnn`) are available. The rules below are the *de facto* standards recorded in
`CLAUDE.md`, assessed in their place and labelled as such.

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| `CLAUDE.md` — "Keep engine changes additive — no existing library should break" | Applies | R5, R12, R24; golden corpus, `capy check` over 117 libraries, and UQ-06 Test J are acceptance gates | PASS | none |
| `CLAUDE.md` — "Library-authoring keyword list — KEEP IT IN SYNC" | Applies | G-05 changes value-expression grammar, not directives or capture types; `docs/language-reference.md` and `docs/inner-dsl.md` are in the F-table | PARTIAL | plan must confirm no `docs/library-keywords.md` row changes |
| `CLAUDE.md` — "Before committing: build, clippy, test, mkdocs --strict all green" | Applies | T-10 | PASS | none |
| `CLAUDE.md` — "Git: commit only when asked; stage files by name" | Applies | Proposal authorizes no commits | NOT APPLICABLE | none |
| Formal project-standards set (`ARCH`/`QUAL`/`PHIL`) | Unclear | No `program_docs/standards/` exists | NEEDS HUMAN REVIEW | owner to decide whether to author one before review |
| Reliability — an engine must not abort its host process | Applies | P-07 reproduced (rc=134); R0/R0b make it a `CapyError` | PARTIAL | `FAIL` until R0 is planned; tracked as PLAN-A's first gate |

No rule assessed `FAIL`; nothing blocks promotion to human review. Agent validation is not approval.

---

## Implementation Design

### Environment and Feasibility

| Environment / Version | Required Capability | Feasibility Evidence and Source | Constraint or Unknown | Resolution |
|---|---|---|---|---|
| Rust ≥ 1.74 (declared MSRV) | New public structs + fields | Plain data | Adding public fields is source-breaking for exhaustive literals/patterns | `#[non_exhaustive]` + constructors — C-05 |
| `capy-core`, one dependency | JSON without serde | `rust/src/gojson.rs` already writes Go-compatible JSON | Coverage at AST depth unconfirmed | D-01 |
| Token stream | End positions and byte offsets | `domain::token::Token` carries `line`/`col` | Whether it carries end/byte offset is **unknown** | D-02 — largest unknown |
| Matcher | Enumerable shapes at a token | UQ-06 advantage 1: the library lists every legal shape, so the expected-set is computable directly rather than reconstructed from item sets | — | proven by design |
| Statement loop | A resync point | UQ-06 advantage 2: `parse_stmt` already walks statement by statement | — | proven by design |
| Delimiters | Opening-token position | UQ-06 advantage 3: `group_open`/`group_close` already counts balanced depth | Position currently discarded, not recorded | C-08 |
| Parser recursion | Bounded depth | P-07 shows unbounded native recursion | Whether left recursion is statically decidable for all shapes | D-03 |

**Discovery tasks — resolved by inspection, never by invention:**

| ID | Question | Evidence to inspect | Rule that resolves it |
|---|---|---|---|
| D-01 | Does `gojson` serialize arbitrarily nested maps/lists? | `rust/src/gojson.rs` + tests | Yes → use it (R8). No → extend it, still no serde. |
| D-02 | Does `Token` carry an end position / byte offset? | `rust/src/domain/token.rs`, `make_lexer.rs` | Yes → spans derive directly. No → lexer records them first, as C-00. |
| D-03 | Is left recursion statically decidable across `arg capture` function-as-type edges? | `make_library_loader.rs`, `make_parser.rs` capture path | Decidable → reject at load (R0b preferred). Not → bounded depth at parse. |

### Methods by Use Case

| Change ID | UC IDs | Method and Execution Order | Positive Path | Negative / Failure Path | Requirement Fulfilment | Feasibility |
|---|---|---|---|---|---|---|
| C-00 | UC-01 | *Conditional on D-02.* Record end position and byte offset on `Token`. | Tokens carry ranges | Lexer tests catch regressions | R1 | experiment needed |
| C-01 | UC-01, UC-05 | Add `Span`; add `span` to `FuncCall` and `CaptureValue`. | Types compile | — | R1, R2, R3 | proven |
| C-02 | UC-01, UC-05 | Populate spans in `make_parser.rs`, including the path that hardcodes `line: 0, col: 0`. | Every node gets first-token start, last-token end | Zero span fails T-03 | R2, R3, R4 | proven |
| C-05 | — | `#[non_exhaustive]` + constructors on the two structs. | Downstream keeps compiling | — | R12 | proven |
| C-07 | UC-09 | Left-recursion guard: build the capture-edge graph at load (D-03) and/or bound parse depth. | Cycle reported as `CapyError` | Depth limit hit → error, never abort | R0, R0b | experiment needed |
| C-08 | UC-06 | Record the opening-delimiter span in the `group_open`/`group_close` counter instead of discarding it. | `CloseDelim` carries `open_at` | — | R17 | proven |
| C-09 | UC-06 | Thread a `Furthest { token_index, expected, context }` record through `try_match`/`match_one`/`capture_func_type`, with the replace/union/discard rule. | Furthest attempt reported | Tie → union | R14 | proven |
| C-10 | UC-06 | Typed `Expectation` enum; each rejection site contributes one; `OneOf` reuses the `options` hint code. | Typed expectations accumulate | Unknown site → generic expectation | R15, R16 | proven |
| C-11 | UC-06 | `Diagnostic`/`Label`/`Severity`; extend `format_with_source` to draw a second underline and connector. | Two-span rendering | No label → current single-caret output | R18 | proven |
| C-12 | UC-07, UC-08 | Resync loop with delimiter-balance priority; error nodes; `ParseResult`; `run` becomes a wrapper returning the first error; emission refuses on error nodes. | All errors in one run; partial tree | Cascade → spacing + cap | R19–R23, R6, R24 | proven |
| C-13 | UC-01, UC-10 | Retain comments in the lexer as trivia; attach preceding comment spans to the following statement. Parser skips trivia when matching, so no shape changes. | Comments reachable on the node | Trivia leaking into matching would fail T-11 | R27 | experiment needed |
| C-03 | UC-01 | Expose `Library::parse -> ParseResult`. | Returns tree + diagnostics | — | R6 | proven |
| C-04 | UC-02, UC-03 | `cmd_ast.rs` + dispatch + help; tree renderer and JSON writer including diagnostics. | Tree or JSON on stdout | Missing file → stderr, exit 1 | R7, R8, R9 | proven |
| C-06 | UC-04 | Precedence-climbing layer in `value_parser.rs` between `parse_value` and `parse_unary`. | Correct tree shape | Unknown operator → existing error | R10, R11 | proven |

### Added, Changed and Removed Contracts

| Item | CRUD | Kind | Name / Key / Route / Event | Type, Default or Schema | Scope / Lifetime | Consumers | Requirements |
|---|---|---|---|---|---|---|---|
| `Span` | CREATE | type | `domain::ast::Span` | six `usize`; `Copy` | public API | analyzers | R1 |
| `FuncCall.span` | CREATE | field | `span` | `Span` | public API | analyzers | R2 |
| `CaptureValue.span` | CREATE | field | `span` | `Span` | public API | analyzers | R3 |
| `FuncCall`/`CaptureValue` | UPDATE | type | `#[non_exhaustive]` | attribute | public API | downstream | R12 |
| `Expectation` | CREATE | type | `Literal`/`Kind`/`Nonterminal`/`OneOf`/`CloseDelim`/block-end | enum | parser-internal + diagnostics | engine, consumers | R15 |
| `Severity` | CREATE | type | `error` \| `warning` | enum | public API | consumers | R18 |
| `Diagnostic` | CREATE | type | severity, code, primary, labels, msg, help | struct | public API | consumers | R18 |
| `Label` | CREATE | type | `{ span, text }` | struct | public API | consumers | R18 |
| diagnostic codes | CREATE | error | `E0007`, `E0012`, … | stable strings | public contract | consumers, CI | R18 |
| `ErrorNode` | CREATE | type | `{ span, tokens, diagnostic_id }` | struct | public API | editors | R22 |
| `Block.errors` | CREATE | field | `errors: Vec<ErrorNode>` | `Vec<ErrorNode>`, default empty | public API | editors | R22 |
| `Block.stmts` | READ | field | **unchanged** `Vec<FuncCall>` | — | public API | every existing walker | R22 |
| `ContextFrame` | CREATE | type | `{ shape: String, arg_index: usize, span: Span }` | struct | public API + diagnostics | consumers | R26 |
| `Diagnostic.context` | CREATE | field | `context: Vec<ContextFrame>` | `Vec<ContextFrame>`, default empty | public API | consumers | R26 |
| `FuncCall.leading_comments` | CREATE | field | `leading_comments: Vec<Span>` | `Vec<Span>`, default empty | public API | LSPs, doc generators | R27 |
| `ParseResult` | CREATE | type | `{ tree, diagnostics }` | struct | public API | analyzers, editors | R6, R22 |
| `Library::parse` | CREATE | function | `parse(&self, &str) -> ParseResult` | — | public API | analyzers | R6 |
| `Library::run` | READ | function | unchanged signature and behaviour | — | public API | everyone | R24 |
| resync K / cap N | CREATE | configuration | suppression window, diagnostic cap | integers, defaults ≈3 / ≈20 | parser | end users | R19 |
| `capy ast` | CREATE | command | `capy ast <library> <script> [--json]` | exit 0 / 1 | CLI | authors, CI | R7 |
| `--json` | CREATE | flag | `--json` | bool, default false | `capy ast` | tooling | R7 |
| `schema_version` | CREATE | field | JSON root key | integer, `1` | AST JSON | tooling | R9 |
| infix operators | CREATE | grammar | `* / % + -`, comparisons, `and`, `or` | left-associative | value expressions | library authors | R10, R11 |
| `docs/ast-json.md` | CREATE | documentation | schema + diagnostics | — | docs site | analyzers | R9 |
| `docs/diagnostics.md` | CREATE | documentation | codes, severities, recovery behaviour | — | docs site | end users | R18 |

### Architecture and Interaction Visuals

```text
BEFORE
  script ─► lexer ─► matcher ──(all shapes fail)──► CapyError "no function matched"
                                                    STOP. no tree, one error.
  deep recursion ─────────────────────────────────► stack overflow, rc=134

AFTER
  script ─► lexer ─► matcher ─┬─ match ─► FuncCall(span) ─┐
                              │                            ├─► Block ─┬─► evaluator ─► String
                              └─ fail ──► Furthest{idx,    │          │   (refuses if any Error node)
                                            expected∪,     │          │
                                            context}       │          └─► ParseResult{tree, diagnostics}
                                            │              │               │
                                            ▼              │               ├─► Library::parse
                                     Diagnostic{code,      │               └─► capy ast [--json]
                                       primary, labels} ───┤
                                            │              │
                                            ▼              │
                                    resync (delimiter ─────┘
                                     first) + Error node
  depth guard ────────────────────────────────────► CapyError "left recursion in `expr`"
```

```text
FURTHEST-FAILURE UPDATE RULE (R14)          RESYNC PRIORITY (R21)
  candidate.idx >  furthest.idx → replace     1. delimiter balance   ← stops runaway
  candidate.idx == furthest.idx → UNION       2. statement boundary  ← first-literals
  candidate.idx <  furthest.idx → discard        from the shape table
                                              3. block boundary (`end` / dedent)
  the UNION line is what produces             4. EOF
  "expected an expression, `(`, or
   an identifier"
```

```text
SPAN NESTING INVARIANT (T-03)
  statement   assign 3:1 ──────────────────────── 3:27
  capture       rhs  3:9 ──────────────── 3:27
  sub             add  3:9 ──────── 3:19
  sub               mul  3:9 ─ 3:14
                      ^ child range ⊆ parent range, every start ≠ 0
```

---

## Expected Code and Documentation Changes

| ID | Path | CRUD | Symbol / Region | Exact Planned Edit | Reason | Change IDs | Requirement IDs | Dependencies | Test / Documentation Impact |
|---|---|---|---|---|---|---|---|---|---|
| F-01 | `rust/src/domain/token.rs` | READ, UPDATE* | `Token` | *If D-02:* add end position / byte offset | Spans need ranges | C-00 | R1 | D-02 | T-01 |
| F-02 | `rust/src/orchestrator/features/make_lexer.rs` | READ, UPDATE* | tokenizer | *If D-02:* record the new fields | as F-01 | C-00 | R1 | F-01 | lexer tests |
| F-03 | `rust/src/domain/ast.rs` | UPDATE | `FuncCall`, `CaptureValue`; new `Span` | Add `Span`; `span` on both; `#[non_exhaustive]` + constructors | R1–R3, R12 | C-01, C-05 | R1–R3, R12 | F-01 | T-01, T-02 |
| F-04 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | `FuncCall` construction incl. the `line: 0, col: 0` path | Populate spans from first/last token | P-03 | C-02 | R2, R3, R4 | F-03 | T-03 |
| F-24 | `rust/src/domain/ast.rs` | UPDATE | `Block`; new `ErrorNode` | Add `ErrorNode` and `Block.errors`. **`Block.stmts` keeps type `Vec<FuncCall>`** | R22; avoids the source-breaking field-type change | C-12 | R22 | F-03 | T-21, T-30 |
| F-41 | `rust/src/domain/token.rs` | UPDATE | `TokenKind` | Add a `Comment` variant (none exists today) | R27 | C-13 | R27 | F-01 | T-29 |
| F-42 | `rust/src/orchestrator/features/make_lexer.rs` | UPDATE | `tokenize_line` | Retain comments as trivia instead of discarding them | R27 | C-13 | R27 | F-41 | T-29, T-11 |
| F-43 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | statement entry | Attach preceding comment spans to the following `FuncCall`; skip trivia when matching | R27 | C-13 | R27 | F-42 | T-29 |
| F-44 | `rust/src/domain/errors.rs` | UPDATE | `Diagnostic`; new `ContextFrame` | Add `ContextFrame` and `Diagnostic.context`; render it | R26 | C-09, C-11 | R26 | F-25 | T-28 |
| F-25 | `rust/src/domain/errors.rs` | UPDATE | `CapyError`; new `Diagnostic`, `Label`, `Severity` | Add types; keep `CapyError` for `run` | R18, R24 | C-11 | R18, R24 | F-03 | T-16, T-17 |
| F-26 | `rust/src/domain/errors.rs` | UPDATE | `format_with_source` | Render a secondary underline and connector | R18 | C-11 | R18 | F-25 | T-17 |
| F-27 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | `try_match` / `match_one` / `capture_func_type` | Thread `Furthest`; replace/union/discard | R14 | C-09 | R14 | F-04 | T-14, T-15 |
| F-28 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | rejection sites | Emit typed `Expectation`s | R15 | C-10 | R15, R16 | F-27 | T-15, T-16 |
| F-29 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | `group_open`/`group_close` depth counter | Record the opening span | R17 | C-08 | R17 | F-03 | T-17 |
| F-30 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | statement loop | Resync, error nodes, cascade rules | R19–R22 | C-12 | R19–R22 | F-24, F-27 | T-18…T-22 |
| F-31 | `rust/src/orchestrator/features/make_parser.rs` | UPDATE | capture recursion | Depth bound / cycle rejection | R0, R0b | C-07 | R0, R0b | D-03 | T-23, T-24 |
| F-32 | `rust/src/orchestrator/features/make_library_loader.rs` | UPDATE | validation pass | Reject a left-recursive capture graph at load | R0b | C-07 | R0b | D-03 | T-23 |
| F-33 | `rust/src/orchestrator/features/make_evaluator.rs` | UPDATE | `run_multi` entry | Refuse emission when an error node is present | R23 | C-12 | R23 | F-24 | T-22 |
| F-05 | `rust/src/capy.rs` | UPDATE | `impl Library`; new `ParseResult` | Add `parse`; keep `run` as a wrapper | R6, R24 | C-03, C-12 | R6, R24 | F-30 | T-02, T-25 |
| F-06 | `rust/src/domain/ast_json.rs` | CREATE | new module | Serialize tree + diagnostics via `gojson` | R8, R9 | C-04 | R8, R9 | D-01, F-03, F-25 | T-05 |
| F-07 | `rust/cli/src/cmd_ast.rs` | CREATE | `cmd_ast` | Flags, tree renderer, `--json` | R7 | C-04 | R7 | F-05, F-06 | T-04, T-05 |
| F-08 | `rust/cli/src/main.rs` | UPDATE | dispatch + usage | Route `ast`; help text | R7 | C-04 | R7 | F-07 | T-06 |
| F-09 | `rust/src/orchestrator/features/value_parser.rs` | UPDATE | `parse_value` | Precedence-climbing layer | R10, R11 | C-06 | R10, R11 | — | T-07, T-08 |
| F-10 | `rust/src/domain/ast.rs` | UPDATE | `Expr` | Binary-operation variant | R10 | C-06 | R10 | F-03 | T-07 |
| F-11 | `rust/src/orchestrator/features/inner_evaluator.rs` | UPDATE | expression evaluation | Evaluate the new variant | R10 | C-06 | R10 | F-10 | T-08 |
| F-12 | `rust/src/orchestrator/features/expr_to_text.rs` | UPDATE | round-trip | Render the new variant back to source text | Omission corrupts output | C-06 | R12 | F-10 | T-12 |
| F-13 | `rust/tests/ast_spans.rs` | CREATE | new file | T-01…T-03, T-09 | R1–R4 | C-01, C-02 | R1–R4 | F-04 | — |
| F-34 | `rust/tests/diagnostics.rs` | CREATE | new file | T-14…T-20, T-22, T-26 | R14–R23 | C-09…C-12 | R14–R23 | F-30 | — |
| F-35 | `rust/tests/recursion_guard.rs` | CREATE | new file | T-23, T-24 — the P-07 reproduction | R0 | C-07 | R0, R0b | F-31 | — |
| F-14 | `rust/tests/embed.rs` | UPDATE | embedding tests | `Library::parse` / `ParseResult` | R6 | C-03 | R6 | F-05 | T-02, T-25 |
| F-15 | `rust/cli/src/cmd_ast.rs` | UPDATE | `mod tests` | Renderer + JSON unit tests | R7 | C-04 | R7 | F-07 | T-04, T-05 |
| F-16 | `docs/ast-json.md` | CREATE | whole file | Schema, spans, diagnostics, stability policy. **Must state the comment-span rule**: a node's `span` excludes attached comments, which carry their own spans in `leading_comments` | R9 | C-04 | R9 | F-06 | mkdocs nav |
| F-36 | `docs/diagnostics.md` | CREATE | whole file | Codes, severities, recovery, cascade rules | R18, R19 | C-11, C-12 | R18, R19 | F-25 | mkdocs nav |
| F-17 | `mkdocs.yml` | UPDATE | nav → Reference | Add both new pages | R9, R18 | C-04, C-11 | R9, R18 | F-16, F-36 | `mkdocs build --strict` |
| F-18 | `docs/language-reference.md` | UPDATE | line 98 region | Replace the "arithmetic is NOT parsed" claim with the precedence table | Cited in UQ-02; would become false | C-06 | R10, R11 | F-09 | T-10 |
| F-19 | `docs/inner-dsl.md` | UPDATE | expressions | Operators and precedence | R10 | C-06 | R10 | F-09 | T-10 |
| F-20 | `docs/embedding.md` | UPDATE | API table + `parse` example | `Library::parse`, `ParseResult`, **plus the explicit line that `stmts` alone does not mean the parse succeeded — check `diagnostics` first** | R6, R28 | C-03 | R6, R28 | F-05 | T-10 |
| F-21 | `docs/cli.md` | UPDATE | command list | `capy ast` | R7 | C-04 | R7 | F-07 | T-10 |
| F-37 | `docs/errors-and-debugging.md` | UPDATE | whole page | Rewrite for the new diagnostic shape | R18 | C-11 | R18 | F-36 | T-10 |
| F-38 | `docs/library-authoring.md` | UPDATE | function-as-type section | Warn that left recursion is rejected; show the right-recursive form | R0 | C-07 | R0 | F-32 | T-10 |
| F-22 | `docs/whats-new.md` | UPDATE | current release | Note all five changes | `CLAUDE.md` requires it | C-01…C-12 | R9 | — | — |
| F-39 | `samples/` | READ, CREATE | corpus + one new sample | Regression surface; add a sample whose golden is an error-recovery transcript | R12, R20 | C-12 | R12, R20 | F-30 | T-11, T-26 |
| F-40 | `rust/devtools/wasm_check.sh` | READ | wasm gate | Confirm the ABI is unchanged | R13 | C-04 | R13 | — | T-13 |

**Inventory summary.** CREATE 16 (`Span`, 2 span fields, `Expectation`, `Severity`, `Diagnostic`, `Label`,
error-node variant, `ParseResult`, `Library::parse`, `capy ast`, `ast_json.rs`, 3 test files, 2 doc pages) ·
UPDATE 22 · DELETE 0 · READ 4. F-01/F-02 are conditional on D-02 (`UPDATE*`); if tokens already carry
ranges, both collapse to `READ`.

Illustrative signatures appear in the UC contracts and in UQ-06. They are **illustrative**; this proposal
defines approved behaviour and authorizes no code.

---

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected | Requirement Impact |
|---|---|---|---|---|
| Do nothing; consumer re-parses captured text | No engine change | Reimplements the parser; positions absent; two grammars drift; crash remains | **Rejected** — it is the status quo that produced UQ-02…UQ-06 | none |
| Spans only | Smallest change; fixes the make-or-break item | Errors still say "no function matched"; crash remains | **Rejected as the whole answer, retained as phase 1** | R1–R5 |
| Diagnostics without recovery (Mechanisms 1–2 only) | No API change; improves every existing user immediately | One error per run; no partial tree for editors | **Adopted as an intermediate milestone** — UQ-06 notes 2 and 3 are shippable independently | R14–R18 |
| Error repair inside a statement | One diagnostic instead of three for a single missing delimiter | Larger; needs recovery first | **Deferred** — Non-Goal 8, per UQ-06 | none |
| `Library::parse -> Result<Block, CapyError>` (revision 1) | Simpler; matches `run` | Cannot carry a partial tree **and** diagnostics; would break when recovery lands | **Rejected in revision 2** in favour of `ParseResult` | R6 |
| Add serde and derive `Serialize` | Idiomatic; less code | Large dependency tree on a one-dependency crate; grows the wasm module | **Rejected** — `gojson` exists (R8) | R8 |
| Emit JSON from a `.capy` library | No engine change | Exactly UQ-04's complaint: no schema, no spans, hand-written | **Rejected** | none |
| Built-in source expression grammar | Best ergonomics for infix languages | Breaks zero-default-grammar; Capy would ship opinions about what an expression is | **Rejected** — Non-Goal 5 | R10 scoped to value expressions |
| Raise the stack limit / spawn a deep-stack thread instead of R0 | Trivial | Moves the cliff; still aborts; still `ok` from `capy check` | **Rejected** — a library error must be a diagnostic, not a crash | R0 |
| **Error nodes as `Block.stmts: Vec<Node>`** | One ordered sequence; the obvious shape | **Source-breaking**: changes the type of an existing public field, so every embedder iterating `block.stmts` fails to compile. `#[non_exhaustive]` does not cover it | **Rejected** — review finding 2 | R22 |
| **Error nodes as a reserved `FuncCall` name (`__error`)** — reviewer's option A | Keeps every walker compiling; one sequence, ordering preserved for free | An existing walker **compiles and runs** but silently treats an error node as a real statement. A walker matching on `func` with a `_ => unreachable!()` arm panics; one building output emits garbage. Trades a loud compile error for a silent wrong answer | **Rejected** — the failure mode is worse than the one it avoids | R22 |
| **Parallel `Block.errors: Vec<ErrorNode>`** — reviewer's option B | Purely additive; every walker compiles; an old walker sees the valid statements and simply does not see the errors, which is a degraded-but-correct view; new consumers opt in by reading `.errors` | Statement/error interleaving is not implicit in one sequence | **SELECTED** — spans give the ordering back, and it is the only option where an un-updated consumer cannot silently misread an error as code | R22 |
| Full lattice/graph support (UQ-01) | Satisfies Glang completely | Needs a worklist, fixpoint iteration, termination argument; destroys declarative totality | **Rejected** — Non-Goals 1–3 | out of scope |

---

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback Action | Owner |
|---|---|---|---|---|---|
| Recovery changes what parses | Golden corpus diff (T-11, T-12) | A shape that used to fail now "succeeds" with an error node inside | Error nodes are produced **only** at statement level, never inside a successful shape match. UQ-06 Test J is the gate | Revert C-12 | Capy Engine |
| Runaway resync eats the file | T-20 | A missing `)` swallows 38 lines | Delimiter balance is checked **before** statement boundary (R21 rule 1) | Revert C-12 | Capy Engine |
| Cascade noise | T-19, T-22 | 1 typo → 40 useless errors | Spacing window + cap (R19) | Tune K/N | Capy Engine |
| Furthest-failure misleads | Review of T-15 output | The furthest shape is not the one the user meant | Union expectations at ties; carry a context frame ("in the parameter list of `fn_decl`") | Revert C-09 | Capy Engine |
| Furthest tracking costs parse time | M-01 | Slower transpiles for everyone | One integer compare per rejection; set grows only at ties; measured on the corpus (R25) | Revert C-09 | Capy Engine |
| Adding public fields breaks downstream | Consumer `cargo build` | Compile errors | `#[non_exhaustive]` + constructors (C-05) | Revert F-03 | Capy Engine |
| Precedence changes an existing expression's parse | T-07, T-11 | Silent output change — worst failure mode | Operators are currently parse errors, so behaviour is additive; T-11 over all 117 libraries | Revert C-06 | Capy Engine |
| `expr_to_text` not updated for the new variant | **T-27**, not T-12 | Captured text corrupted in rendered output — silent, and the wrong target code is emitted | F-12 mandatory. **T-12 cannot detect this**: arithmetic is a parse error today (`docs/language-reference.md:98`), so no existing golden contains `a * b + c` and the golden corpus exercises zero new node variants. T-07 proves the *parse*; only T-27's round-trip proves the *render* | Revert C-06 | Capy Engine |
| `Block.stmts` type change breaks every walker | Consumer `cargo build`; T-30 | Source-breaking for all embedders; `#[non_exhaustive]` does **not** protect a field's type | `stmts` keeps `Vec<FuncCall>`; error nodes go in a parallel `errors` field (R22) | Revert F-24 | Capy Engine |
| Comment trivia leaks into matching | T-11, T-29 | A library that parses today stops parsing | Parser skips trivia at match time; T-11 over all 117 libraries is the gate | Revert C-13 | Capy Engine |
| Left-recursion guard rejects a valid library | T-11 | A working library stops loading | Guard must reject only cycles that cannot consume a token; T-11 over all 117 libraries | Revert C-07 | Capy Engine |
| wasm module grows | M-02 | Slower playground load | Measure before merge | Revert F-06 | Capy Engine |
| D-02 turns out large | Discovery | PLAN-A cost rises | Sequence C-00 first; re-estimate | Defer G-01 | Capy Engine |
| JSON schema churns after adoption | Consumer report | Lost trust in the seam | `schema_version` + policy (R9) | Bump version, keep old shape one release | Capy Engine |

---

## Security Impact

No new host capability, no new I/O, no new dependency. R0 **removes** an availability risk: a library can
currently abort the host process via unbounded recursion, which an embedded caller cannot catch. The AST and
diagnostics embed source text (already true of rendered output), so `--json` output is exactly as sensitive
as the input script — stated in `docs/ast-json.md` for CI use.

## Operational Impact

One new CLI subcommand; no new required CI job. Diagnostic codes (`E0012`, …) become a public contract that
consumers may match on, so they need the same stability discipline as the JSON schema. `capy ast --json` is
available to consumers' pipelines.

## Compatibility Impact

- **Source-breaking for exhaustive struct literals/patterns** over `FuncCall`/`CaptureValue`, mitigated by
  C-05. `capy-core` is not yet published to crates.io — the cheapest moment to make this change.
- **`.capy` libraries:** additive. Operators that now parse were previously parse errors.
- **`Library::run`:** unchanged signature and behaviour (R24).
- **Error message text changes** for previously-failing programs. Error goldens
  (`*.expected-error.txt`) will need review: their *content* is expected to improve, so T-11 asserts
  successful output is unchanged while error goldens are re-reviewed deliberately, not auto-updated.
- **wasm ABI:** unchanged. Exposing the AST or diagnostics over wasm is deliberately out of scope.

## Migration Requirements

None for library authors, except any library relying on left recursion — which cannot work today (it
crashes), so no working library migrates. Consumers on the undocumented `make_parser::parse` path move to
`Library::parse` (R6); `docs/embedding.md` records the move (F-20).

---

## Test and Validation Design

Tests T-14…T-24 correspond to UQ-06's test table A–K; the source ID is named in each row.

| ID | Type | UC / Requirement IDs | Scenario and Purpose | Environment / Data | Exact Procedure or Command | Expected Result | Test File / Evidence Destination |
|---|---|---|---|---|---|---|---|
| T-01 | unit | R1, R2 | Statement span covers first→last token incl. block and closer | fixture with `block_closer` | `cargo test --manifest-path rust/Cargo.toml --test ast_spans` | span.end ≥ closer's last token | `rust/tests/ast_spans.rs` |
| T-02 | integration | R3, R6 | Two captures have distinct, non-overlapping spans | `f a b` | as T-01 | spans differ | `rust/tests/ast_spans.rs` |
| T-03 | regression | R4 | **No reachable node has a zero span** | 3-level function-as-type tree | as T-01 | all non-zero; child ⊆ parent | `rust/tests/ast_spans.rs` |
| T-04 | unit | R7 | Tree renderer shape | `samples/transpile-py` | `cargo test -p capy-cli` | matches UC-02 contract | `rust/cli/src/cmd_ast.rs` |
| T-05 | unit | R7–R9 | JSON well-formed, carries `schema_version` and `diagnostics` | as T-04 | `cargo test -p capy-cli` | parses; required keys present | `rust/cli/src/cmd_ast.rs` |
| T-06 | E2E | R7 | Exit codes and stream discipline | built CLI | `capy ast lib s --json \| jq .` ; `capy ast lib missing.capy` | 0/empty stderr; 1/message | CI log |
| T-07 | unit | R10, R11 | Precedence and associativity | `a*b+c`, `a+b*c`, `a+1==b*2`, `(a+b)*c` | `cargo test -p capy-core` | trees per R10/R11 | `value_parser.rs` tests |
| T-08 | integration | R10 | Arithmetic evaluates in a condition and in `${…}` | fixture library | `cargo test --workspace` | expected rendered output | `rust/tests/` |
| T-09 | manual | R1–R4, UC-05 | Consumer renders a caret under a nested operand | scratch crate + path dep | build and run sample analyzer | caret under exactly the operand | attached to PLAN-B |
| T-14 | unit | R14 — *UQ-06 test A* | Furthest attempt is reported, not the aggregate | `fn add(x: int {` | `cargo test --test diagnostics` | message names `)`, not "no shape matched" | `rust/tests/diagnostics.rs` |
| T-15 | unit | R14 — *UQ-06 test B* | Expectation union at a tie | 3 shapes dying at one token | as T-14 | message lists all three alternatives | `rust/tests/diagnostics.rs` |
| T-16 | unit | R16 — *UQ-06 test C* | `OneOf` did-you-mean | typo'd `options` value | as T-14 | `help: did you mean \`write\`?` | `rust/tests/diagnostics.rs` |
| T-17 | unit | R17, R18 — *UQ-06 test D* | Two-span rendering | unclosed delimiter | as T-14 | label points at the **opening** token | `rust/tests/diagnostics.rs` |
| T-18 | integration | R20 — *UQ-06 test E* | Recovery count | 3 broken statements | as T-14 | exactly 3 diagnostics | `rust/tests/diagnostics.rs` |
| T-19 | integration | R19 — *UQ-06 test F* | Cascade suppression | 1 broken statement | as T-14 | exactly 1 diagnostic, not 4 | `rust/tests/diagnostics.rs` |
| T-20 | integration | R21 — *UQ-06 test G* | Delimiter fence | missing `)` at line 2 of a 40-line file | as T-14 | recovery does not consume the rest | `rust/tests/diagnostics.rs` |
| T-21 | integration | R22 — *UQ-06 test H* | Error node; statements after it still parse | broken statement mid-file | as T-14 | later statements present in the tree | `rust/tests/diagnostics.rs` |
| T-22 | integration | R19, R23 — *UQ-06 test I* | Cap and refusal | file with 60 errors | as T-14 | N shown + "and 40 more"; no output emitted | `rust/tests/diagnostics.rs` |
| T-23 | regression | R0, R0b — *new, from P-07* | **The P-07 reproduction must not abort** | left-recursive `expr`/`term` library | `cargo test --test recursion_guard` | `CapyError` naming the cycle; process never aborts; rc ≠ 134 | `rust/tests/recursion_guard.rs` |
| T-24 | property | R0 | Deep nesting terminates | generated deeply-nested input | as T-23 | error or success, never abort | `rust/tests/recursion_guard.rs` |
| T-25 | unit | R24 | `run` behaviour unchanged | existing error fixtures | `cargo test --workspace` | first error, no output, same text where unchanged | `rust/tests/embed.rs` |
| T-26 | property | R12 — *UQ-06 test K* | Any valid program → zero diagnostics, no error nodes | all 117 libraries + their scripts | `cargo test --test diagnostics` | diagnostics empty; tree free of error nodes | `rust/tests/diagnostics.rs` |
| T-27 | property | R10, R12 — *review finding 1* | **Round-trip: `parse(render(parse(E))) == parse(E)`, structural equality, over a new arithmetic corpus.** Catches dropped and misplaced parentheses — `(a+b)*c` rendering as `a+b*c` is silent output corruption that no golden can catch, because arithmetic is a parse error today so no golden contains it. | generated + hand-written expression corpus incl. nested parens, mixed precedence, unary | `cargo test -p capy-core` | structurally equal trees; **not** string equality | `value_parser.rs` / `rust/tests/` |
| T-28 | unit | R26 | Context frame appears in the diagnostic | failure nested inside a shape's argument list | `cargo test --test diagnostics` | message contains "in the parameter list of `fn_decl`" | `rust/tests/diagnostics.rs` |
| T-29 | unit | R27 | Comments retained and attached; parsing unchanged | script with leading, trailing and interior comments | `cargo test --workspace` | preceding-comment spans exposed on the node; rendered output identical to today | `rust/tests/ast_spans.rs` |
| T-31 | unit | R27 — *review residual 1* | A node's `span` excludes its attached comments | `# note` on the line above a statement | `cargo test --workspace` | `span.start` is the first code token; the comment appears only in `leading_comments` with its own span | `rust/tests/ast_spans.rs` |
| T-30 | compatibility | R22 | An existing walker over `Block.stmts` still compiles and sees only valid statements | scratch crate pinned to the pre-change walker shape | build + run against a partially-broken file | compiles; iterates `Vec<FuncCall>`; errors visible only via `.errors` | attached to PLAN-C |
| T-10 | regression | R5, R12, R9 | Full gate | repo | `cargo build --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo test --workspace` ; `mkdocs build --strict` | all green | CI |
| T-11 | regression | R12 — *UQ-06 test J* | **No library changes behaviour** | all 117 libraries | `capy check` each; golden corpus | 117 ok; goldens 116 pass / 0 fail | CI |
| T-12 | regression | R12 | Expression round-trip to source text | libraries using captured expressions | golden corpus | byte-identical output | CI |
| T-13 | regression | R13 | wasm surface unchanged | deno | `./rust/devtools/wasm_check.sh` | PASS 113 / FAIL 0 | CI |

### Measurement and Validation

Per §21.3. Baselines are frozen **before** implementation; measured values are recorded in an `RPT` document
and may not be edited retroactively.

| Measurement | Baseline Method | Test Command or Procedure | Controlled Environment | Acceptance Threshold | Report Destination |
|---|---|---|---|---|---|
| M-01 · native transpile time (covers R25) | `nativebench`, 200 iterations after 5 warm-up, best of 5 | `cargo build --release -p capy-devtools --bin nativebench` then `./rust/target/release/nativebench samples/transpile-py/lib.capy samples/transpile-py/script.capy` | idle machine, release profile, same host | **≤ 10 % regression** vs. **187–219 µs** measured 2026-09-14 | `program_docs/reports/` |
| M-02 · wasm module size | `ls -l` of the release artifact | `cargo build --release --target wasm32-unknown-unknown --manifest-path rust/Cargo.toml -p capy-wasm-abi` | release profile (`opt-level="z"`, LTO, strip) | **≤ 5 % growth** vs. **1 342 891 bytes** measured 2026-09-16 | same |
| M-03 · capy-core dependency count | `cargo tree` | `cargo tree -p capy-core --depth 1` | — | **exactly 1** (`regex`) | same |
| M-04 · failing-parse cost | Time a parse that fails at the last token of a long statement | `nativebench`-style harness over a deliberately failing corpus | as M-01 | **no worse than 2×** a successful parse of the same input | same |

---

## Documentation, Demo and Release Impact

| Artifact | Exact Path or Destination | CRUD | Required Content / Verification | Owner | Release Gate |
|---|---|---|---|---|---|
| AST + diagnostics JSON schema | `docs/ast-json.md` | CREATE | Every field; `schema_version` policy; **span semantics incl. the rule that a node's span excludes its attached comments** | Capy Engine | yes |
| Diagnostics reference | `docs/diagnostics.md` | CREATE | Codes, severities, recovery and cascade behaviour | Capy Engine | yes |
| Nav | `mkdocs.yml` | UPDATE | Reference → both new pages | Capy Engine | yes |
| Language reference | `docs/language-reference.md` | UPDATE | Replace the line-98 claim with the precedence table | Capy Engine | yes |
| Inner DSL | `docs/inner-dsl.md` | UPDATE | Operators, precedence, associativity | Capy Engine | yes |
| Errors and debugging | `docs/errors-and-debugging.md` | UPDATE | Rewrite for the new diagnostic shape | Capy Engine | yes |
| Library authoring | `docs/library-authoring.md` | UPDATE | Left recursion is rejected; show the right-recursive form | Capy Engine | yes |
| Embedding guide | `docs/embedding.md` | UPDATE | `Library::parse`, `ParseResult`, compiled example, **and the "check diagnostics before trusting the tree" line** | Capy Engine | yes |
| CLI reference | `docs/cli.md` | UPDATE | `capy ast`, both modes, exit codes | Capy Engine | yes |
| What's new | `docs/whats-new.md` | UPDATE | All five changes | Capy Engine | yes |
| Recovery sample | `samples/` | CREATE | A sample whose golden is an error-recovery transcript | Capy Engine | no |
| Worked analyzer | scratch crate | CREATE | Demonstrates UC-05 end to end | Capy Engine | no |
| Version source | `rust/Cargo.toml` `[workspace.package] version` | UPDATE | Minor bump to 0.21.0 | Capy Engine | yes |
| Release record | `program_docs/releases/` | CREATE | Per Part II of `DOCUMENTATION.md` | Release Management | yes |

---

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases / Internal Constraint | Changes | Files | Tests | Manual / Demo / Release Evidence |
|---|---|---|---|---|---|---|---|
| R0 | UQ-06 | G-00 | UC-09 | C-07 | F-31, F-32, F-35, F-38 | T-23, T-24 | `docs/library-authoring.md` |
| R0b | UQ-06 | G-00 | UC-09 | C-07 | F-32 | T-23 | `docs/library-authoring.md` |
| R1 | UQ-03, UQ-06 | G-01 | UC-01 | C-00, C-01 | F-01, F-02, F-03, F-13 | T-01 | `docs/ast-json.md` |
| R2 | UQ-03 | G-01 | UC-01 | C-01, C-02 | F-03, F-04, F-13 | T-01 | `docs/ast-json.md` |
| R3 | UQ-03, UQ-06 | G-01 | UC-01 | C-01, C-02 | F-03, F-04, F-13 | T-02 | `docs/ast-json.md` |
| R4 | UQ-03 | G-01 | UC-01, UC-05 | C-02 | F-04, F-13 | T-03, T-09 | worked analyzer |
| R5 | internal — additive rule | G-01 | internal constraint | C-02 | F-04 | T-10, T-11 | — |
| R14 | UQ-06 | G-02 | UC-06 | C-09 | F-27, F-34 | T-14, T-15 | `docs/diagnostics.md` |
| R15 | UQ-06 | G-02 | UC-06 | C-10 | F-28, F-34 | T-15 | `docs/diagnostics.md` |
| R16 | UQ-06 | G-02 | UC-06 | C-10 | F-28, F-34 | T-16 | `docs/diagnostics.md` |
| R17 | UQ-06 | G-02 | UC-06 | C-08 | F-29, F-34 | T-17 | `docs/diagnostics.md` |
| R18 | UQ-06 | G-02 | UC-06 | C-11 | F-25, F-26, F-34, F-36, F-37 | T-17 | `docs/errors-and-debugging.md` |
| R19 | UQ-06 | G-03 | UC-07 | C-12 | F-30, F-34, F-36 | T-19, T-22 | `docs/diagnostics.md` |
| R20 | UQ-06 | G-03 | UC-07 | C-12 | F-30, F-39 | T-18, T-26 | recovery sample |
| R21 | UQ-06 | G-03 | UC-07 | C-12 | F-30, F-34 | T-20 | `docs/diagnostics.md` |
| R22 | UQ-06 | G-03, G-04 | UC-08 | C-12 | F-24, F-30, F-34 | T-21 | `docs/ast-json.md` |
| R23 | UQ-06 | G-03 | UC-07 | C-12 | F-33, F-34 | T-22 | `docs/diagnostics.md` |
| R6 | UQ-04, UQ-06 | G-04 | UC-01, UC-08 | C-03, C-12 | F-05, F-14, F-20 | T-02, T-25 | `docs/embedding.md` |
| R7 | UQ-04 | G-04 | UC-02, UC-03 | C-04 | F-07, F-08, F-15, F-21 | T-04, T-06 | `docs/cli.md` |
| R8 | UQ-04 | G-04 | UC-03 | C-04 | F-06 | T-05 | M-03 |
| R9 | UQ-04 | G-04 | UC-03 | C-04 | F-16, F-17, F-22 | T-05, T-10 | `docs/ast-json.md` |
| R24 | UQ-06 | G-03, G-04 | internal constraint | C-12 | F-05 | T-25, T-11 | — |
| R10 | UQ-02 | G-05 | UC-04 | C-06 | F-09, F-10, F-11, F-19 | T-07, T-08, **T-27** | `docs/inner-dsl.md` |
| R11 | UQ-02 | G-05 | UC-04 | C-06 | F-09, F-18 | T-07 | `docs/language-reference.md` |
| R12 | internal — additive rule | all | internal constraint | C-05, C-06, C-07, C-12, C-13 | F-03, F-12, F-24, F-39 | T-10…T-13, T-26, **T-27**, T-30 | — |
| R13 | internal — published artifact | G-04 | internal constraint | C-04 | F-06, F-40 | T-13 | M-02 |
| R25 | UQ-06 risk table | G-02 | internal constraint | C-09 | F-27 | — | M-01, M-04 |
| R26 | UQ-06 (UC-06 step 3) | G-02 | UC-06 | C-09, C-11 | F-44 | T-28 | `docs/diagnostics.md` |
| R27 | review finding 3 | G-01, G-04 | UC-01, UC-10 | C-13 | F-41, F-42, F-43, F-16 | T-29, T-11, T-31 | `docs/ast-json.md` |
| R28 | review residual 2 | G-03, G-04 | internal constraint | C-03 | F-20 | T-10 | `docs/embedding.md` |

**Reverse check.** C-00→R1 · C-01→R1–R3 · C-02→R2–R5 · C-03→R6 · C-04→R7–R9, R13 · C-05→R12 · C-06→R10–R12 ·
C-07→R0, R0b · C-08→R17 · C-09→R14, R25, R26 · C-10→R15, R16 · C-11→R18, R26 · C-12→R6, R19–R24 · C-13→R27 · C-03→R6, R28. Every F-NN appears in
the rows above. UQ-01's lattice, graph and monomorphization items are deliberately unmapped — Non-Goals 1–3.
UQ-06's error-repair and incremental-reparse items are deliberately unmapped — Non-Goals 8 and 9.

---

## Plan Strategy and Estimated Work

One plan is **not** sufficient. Five plans, adopting UQ-06's stated ordering, which matches the dependency
graph: spans are a hard prerequisite for diagnostics; diagnostics precede recovery; `parse()` is "already the
seam recovery needs"; precedence produces trees whose nodes need spans and error nodes.

| Plan | Owns | Requirements | Use Cases | Changes | Depends on | Rationale |
|---|---|---|---|---|---|---|
| **PLAN-A · Recursion guard + spans + comment retention** | R0, R0b, R1–R5, R27, R12 (partial) | UC-01, UC-05, UC-09, UC-10 | C-00, C-01, C-02, C-05, C-07, C-13 | D-02, D-03 | A crash outranks everything; spans are the hard prerequisite for B and C. Comment retention joins this plan because it touches the same lexer pass — deferring it means reopening the lexer later |
| **PLAN-B · Diagnostics** | R14–R18, R25, R26 | UC-06 | C-08, C-09, C-10, C-11 | PLAN-A | Shippable alone; improves every existing Capy user with no API change |
| **PLAN-C · Recovery** | R19–R24, R6 (partial) | UC-07, UC-08 | C-12 | PLAN-B | Needs `ParseResult`; highest behavioural risk, gated by T-11/T-26 |
| **PLAN-D · Structured output** | R6, R7–R9, R13, R28 | UC-01, UC-02, UC-03 | C-03, C-04 | PLAN-C | Emits tree **and** diagnostics; shipping it earlier would freeze a schema that recovery then changes |
| **PLAN-E · Operator precedence** | R10, R11, R12 (partial) | UC-04 | C-06 | none (independent) | Highest regression risk, lowest blocking value; may be unnecessary — Open Question 7 |

PLAN-B is the recommended first *user-visible* release: it is additive, needs no API change, and converts
every "no function matched" into a real message.

Sizing is deliberately withheld: D-02 determines PLAN-A's lexer scope and D-03 its guard strategy. Estimates
belong in the plans, after discovery.

---

## Open Questions

1. **D-02** — does `Token` already carry an end position / byte offset? Decides PLAN-A's size. *(Capy Engine, before PLAN-A)*
2. **D-03** — is left recursion statically decidable across capture edges, or is a depth bound required? Decides whether `capy check` can catch P-07. *(Capy Engine, before PLAN-A)*
3. **`ParseResult` vs `Result`** — revision 2 adopts `ParseResult { tree, diagnostics }` per UQ-06, superseding revision 1. Confirm before PLAN-D freezes the JSON schema. *(Capy Engine + Glang)*
4. **D-01** — does `gojson` handle nested structures at AST depth? *(Capy Engine, before PLAN-D)*
5. **Cascade constants** — R19 leaves K and N configurable with UQ-06's suggested ≈3 and ≈20. Fix defaults during PLAN-C from real corpus behaviour. *(Capy Engine)*
6. **Diagnostic code allocation** — who owns the `E0007`/`E0012` namespace, and is it stable across releases once consumers match on it? *(Capy Engine)*
7. **Is `and`/`or` wanted, or only arithmetic and comparison?** R10 assumes both; `not` already exists. *(Glang)*
8. **Does Glang want source-expression trees from function-as-type captures (works once PLAN-A lands) or a built-in expression grammar (Non-Goal 5)?** Decides whether PLAN-E is needed at all. *(Glang)*
9. **Should the wasm ABI expose the AST and diagnostics?** Out of scope here; ask before the shape is frozen — the playground would benefit from recovery. *(Capy Engine)*
10. **Should a project-standards set be authored first?** `program_docs/standards/` does not exist, so §21 validation has no rule IDs. *(Documentation Owner)*
11. **Error-golden policy** — `*.expected-error.txt` contents will improve. Re-review deliberately, or regenerate with `CAPY_UPDATE_GOLDENS=1` and review the diff? *(Capy Engine, during PLAN-B)*
12. ~~**Comment attachment semantics**~~ — **ANSWERED 2026-09-16 by the Glang consumer.** Scoped by *artifact*, not by consumer, so the answer stays stable as consumers change:

    | Artifact | Needs | Why |
    |---|---|---|
    | compiler / analyzer (Glang's actual need) | **leading only** | doc comments for hover and generated API docs. Glang's semantics live in `where` clauses and `effects` blocks — real syntax, not comments |
    | LSP hover | **leading only** | same |
    | formatter | leading **+ trailing + interior** | a formatter that drops a trailing `// why`, or a comment inside an empty block, is unusable |

    **Decision:** R27 stays leading-only. **Two triggers expand it**, and either one makes trailing and interior mandatory:
    1. **Capy emits a formatter from a `.capy` file** — the strategic differentiator; a formatter cannot drop trivia.
    2. **Capy hosts a `vhco:`-style annotation system** — those are semantically load-bearing *trailing* comments, which inverts the table above. Named by the consumer as a caveat against their own answer.

    Recorded as triggers rather than "more is speculative" so a future reviewer knows exactly what reopens this.

---

## Approval

| Role | Name | Decision | Date | Notes |
|---|---|---|---|---|
| Owner | Capy Engine | pending | — | — |
| Consumer | Glang | pending | — | Open Questions 3, 7, 8 need their input |

Agent validation recorded under *Project Validation* is not approval.

---

## Related Documents

- `.ignore/needs.md` — the four-item capability comparison (UQ-01)
- `.ignore/needs2.md` — three precise blockers (UQ-02…UQ-04)
- `garantees_lang/.ignore/capy_error.md` — the error-recovery design note (UQ-06); source of Mechanisms 1–4, the ordering adopted in Plan Strategy, and tests A–K
- `DOCUMENTATION.md` — the standard this document follows (§4.2, §12.1, §21)
- `CLAUDE.md` — de facto project rules used in *Project Validation*

---

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document — spans, structured output, operator precedence |
| 2 | 2026-09-16 | Olivier | Added error recovery from `capy_error.md` (UQ-06): left-recursion guard (P-07, verified crash), furthest-failure tracking, expectation vocabulary, resync and error nodes. Superseded R6 with `ParseResult`. Retitled; five plans instead of three. |
| 3 | 2026-09-16 | Olivier | Review findings 1–4. Added T-27 (round-trip, structural equality) — T-12 provably cannot detect `expr_to_text` omission, since no golden contains arithmetic. Changed the error-node representation to a parallel `Block.errors`, keeping `Block.stmts: Vec<FuncCall>` — reviewer's option B, chosen over option A because a reserved `__error` name lets an un-updated walker silently misread an error as code. Added R26 (context frame, previously narrative-only) and R27 (comment retention, previously neither required nor deferred; verified that `TokenKind` has no `Comment` variant). |
| 4 | 2026-09-16 | Olivier | Consumer answered OQ-12: comment attachment scoped **by artifact** — leading-only suffices for compiler/analyzer and LSP hover; a formatter needs trailing and interior. Recorded the two expansion triggers (Capy emitting a formatter; hosting a `vhco:`-style annotation system, whose trailing comments are semantically load-bearing). Residual 1: R27 now pins that a node's span **excludes** attached comments, with T-31 and a mandatory line in `docs/ast-json.md`. Residual 2: added R28 — `docs/embedding.md` must state that `stmts` alone does not mean the parse succeeded. |
