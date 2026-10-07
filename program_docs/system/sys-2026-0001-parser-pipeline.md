---
document_id: SYS-2026-0001
title: System — Lexer and Parser Pipeline as Implemented
document_type: system
status: active

created_date: 2026-09-16
last_updated: 2026-10-07
document_revision: 4

authors:
  - Olivier

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-core

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Describes the implemented lexer-to-AST pipeline through 0.24.0, including trivia handling, span population, the recursion guards (and how the nesting bound is reported), diagnostics, value expressions, ordered choice in a capture type, and the version string.

reason: DOCUMENTATION.md section 31 requires system documentation to describe the current implemented system whenever behaviour or internal interfaces change.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001
  - PLAN-2026-0003
  - PROP-2026-0004
  - ADR-0003
  - DEMO-2026-0003
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - DEMO-2026-0004

supersedes: null
superseded_by: null

tags:
  - parser
  - lexer
  - spans

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# System — Lexer and Parser Pipeline as Implemented

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.22.0 and later (current through 0.24.0)
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Summary

This describes what the code does through 0.24.0, not what was intended.

## Pipeline

```text
source
  │
  ▼
make_lexer::tokenize            (manifest + inner DSL)  ─┐
make_lexer::tokenize_with       (user scripts, no trivia) ├─► tokenize_impl
make_lexer::tokenize_with_trivia(user scripts, + trivia) ─┘
  │                                   emits TokenKind::Comment only when
  │                                   keep_comments is set
  ▼
make_parser::parse
  │  1. strip TokenKind::Comment out of the stream, filing each comment's
  │     span under the index of the next SOURCE token (col != 0)
  │  2. match statements against library shapes
  │  3. populate spans from first/last token
  │  4. attach filed comments to the statement they lead
  ▼
Block { stmts: Vec<FuncCall> }
```

## Components

| Area | File | Responsibility at 0.21.0 |
|---|---|---|
| Token | `rust/src/domain/token.rs` | `kind`, `text`, `line`, `col`, `width`; `TokenKind::Comment` added |
| Lexer | `rust/src/orchestrator/features/make_lexer.rs` | `tokenize_impl(source, markers, keep_comments)`; the two public wrappers differ only in that flag |
| AST | `rust/src/domain/ast.rs` | `Span`; `FuncCall.span`, `FuncCall.leading_comments`, `CaptureValue.span`; both structs `#[non_exhaustive]` |
| Parser | `rust/src/orchestrator/features/make_parser.rs` | trivia strip, span population, `MAX_PARSE_DEPTH`; `capture_func_type_inner` and `match_alt` (ordered choice, 0.23.0); `depth_err` and `fail_code` (named nesting bound, 0.24.0) |
| Loader | `rust/src/orchestrator/features/make_library_loader.rs` | `reject_left_recursion` after `is_func` resolution; alternatives validated as library functions and carried through `compile_args` / `compile_elements` (0.23.0) |
| Lib-parser | `rust/src/infra/capy_lib_parser.rs` | reads `arg capture NAME A \| B \| C` into `RawArg.type_` + `RawArg.alts` (0.23.0) |
| Data model | `rust/src/infra/raw_library.rs`, `rust/src/domain/library.rs` | `alts: Vec<String>` on `RawArg`, `ArgEntry`, `PatternElement`; `PatternElement::alternatives()` (0.23.0) |
| Public surface | `rust/src/capy.rs`, `rust/src/domain/docs.rs` | `ArgInfo.alts`; the `capy docs` Type column prints the union (0.23.0) |

## Span semantics

- 1-indexed, source-absolute; `end_col` is **exclusive**.
- A statement's span covers its block body and closer, extended after the body is
  parsed — `try_match` cannot know it, because the body has not been parsed yet.
- A capture's span covers exactly the tokens consumed for it; a capture that bound
  a default consumed nothing and stays **unset** (`start_line == 0`).
- Structural tokens (NEWLINE, INDENT, DEDENT, EOF) carry `col == 0` and are
  excluded, so a span never starts or ends at column 0.
- `width == 0` means "unset"; the span builder falls back to `text.len()`.

## Recursion limits

Two independent mechanisms, both required:

1. **Load time** — `reject_left_recursion` builds a graph of "left edges"
   (`F -> G` where `F` can reach `G` without consuming) and refuses any cycle.
   Iterative DFS with white/grey/black marking; explicitly not recursive, since
   the whole point is not to blow the stack while checking for stack blowups.
2. **Parse time** — `MAX_PARSE_DEPTH = 64` bounds nonterminal descent. This is
   not redundant: a *valid* right-recursive library fed deeply nested input
   aborted without it.

Since 0.24.0 the parse-time bound is reported by name; see "The nesting bound
is reported (0.24.0)" below.

A function that declares no `arg literal` has its own name prepended as one, so
it always consumes a token and can never be a left-recursive hop.

Since 0.23.0 the graph's edges come from every alternative of a choice, not just
a capture's first type (see "Ordered choice" below).

## Diagnostics and recovery (0.22.0)

```text
make_parser::parse             → Result<Block, CapyError>     first error, no tree
make_parser::parse_recovering  → (Block, Vec<Diagnostic>)     tree + every error
```

`Library::run` uses the first; `Library::parse` uses the second, which is how
`run` keeps its signature and behaviour while `parse` collects everything.

Failure reporting is **furthest-first**: a `Furthest { index, expected, context }`
record threads through the matcher and is deliberately NOT restored on backtrack.
Strictly further replaces, equal-distance unions, nearer is discarded.

Recovery, per failed statement: emit a diagnostic, push an `ErrorNode`, resync,
continue. Resync checks delimiter balance first, then statement boundary (using
the exact set of shape-starting literals), then dedent, then EOF — with a bound,
because a delimiter that is never closed would otherwise hold the depth above
zero to EOF and consume the file.

`make_evaluator::run_multi` refuses to emit when `Block.errors` is non-empty.

## Value expressions (0.22.0)

`value_parser` is a precedence-climbing parser. Comparison still produces
`Expr::Compare` so the existing evaluator path is untouched; only its precedence
changed. Arithmetic and boolean operators produce `Expr::Binary`.

`(` remains the prefix-call form; grouping applies only when the contents parse
as a complete expression that is not a bare identifier, which leaves `(upper n)`
and `(foo)` exactly as they were.

## Ordered choice (0.23.0)

A capture type may name several library functions: `arg capture v call | name | num`.
Each statement below was checked against the code on 2026-10-07.

```text
  lib file                 lib-parser            loader                       matcher
  ─────────────            ───────────           ──────────────────           ─────────────────────
  v call | name | num ──►  type_  = "call"  ──►  every name must be a   ──►  targets = [cap_type,
                           alts   = [name,       library FUNCTION               alts…] in order
                                    num]         guard: edge to EVERY           match_alt: first
                                                 alternative                    match wins
```

**Lib-parser** (`capy_lib_parser.rs`). The line is split on whitespace, so a choice
arrives as the tokens `A`, `|`, `B`, or glued as `A|B`. Every piece that touches a
`|` is rejoined (stopping at `default`, `sep`, `join`), then the text is split on
`|`. A repetition suffix (`*`, `+`) is trimmed from the **last** name before the
split, so it applies to the whole choice. If any name is not an identifier the
load fails with `malformed alternation`. Otherwise `type_` is alternative 1 and
`alts` holds 2…n; `alts` is empty for an ordinary capture.

**Loader** (`make_library_loader.rs`).

- `validate_cross_references`: when `alts` is non-empty, every name — `type_`
  and each of `alts` — must be a library function, else the load fails with
  ``capture "v" alternation names "int", which is not a library function`` and a
  hint to wrap the type in a `bare` function. A single-type capture takes the
  unchanged path.
- `reject_left_recursion`: the leading-position walk adds an edge to **every**
  alternative via `PatternElement::alternatives()` (`cap_type` first, then
  `alts`), so a cycle through any alternative is refused with the existing
  message. The walk is still iterative.
- `compile_args` and `compile_elements` copy `alts` through to `PatternElement`.

**Matcher** (`make_parser.rs`).

- `capture_func_type_inner` resolves the target list from
  `el.alternatives()` — `cap_type`, then `alts` — so a single-type capture has
  exactly one target and behaves as before.
- `match_alt` tries the targets in order and returns the first that matches
  (`find_map` over `match_one`). `match_one` restores the token cursor on failure
  and also treats a match that consumed nothing as a failure, so the rewind is
  local to the capture.
- `match_alt` itself notes no expectation. A repetition that simply ends is not a
  failure, so existing diagnostics are byte-identical.
- The **mandatory** call sites — exactly-one, and `+` with zero matches — note
  `Expectation::Nonterminal` for **every** alternative, in order. The existing
  furthest-failure merge unions them, which is what prints
  ``expected a `call`, a `name`, or a `num` ``.
- The tree node for the winner has `func` set to the winning alternative's name,
  which is what `capy ast --json` shows in `sub[].func`. No serializer change and
  `schema_version` stays `1`.
- Depth is still bounded by `MAX_PARSE_DEPTH`. Before 0.24.0 the bound's message
  was consumed by the rewind, so a too-deep input reported the generic expectation;
  see the next section for how it is reported now.

**Public surfaces.**

| Surface | What it shows |
|---|---|
| `ArgInfo.type_` | alternative 1 (unchanged meaning) |
| `ArgInfo.alts` | alternatives 2…n; empty for a single-type capture |
| `capy docs` | the Type column joins `type_` and `alts` with ` \| ` |
| `capy ast --json` | `sub[].func` is the matched alternative |

The browser introspection JSON (`rust/wasm/src/lib.rs`, `capy_introspect`) writes,
for each argument, `"type":…` followed by `"alts":[…]` and then `"description"`. `type`
keeps meaning alternative 1; `alts` holds 2…n, and is `[]` for a plain capture, so a
consumer may read the field unconditionally (0.24.0; before it the JSON emitted `type`
only). Two unit tests in that file assert both shapes
(`introspect_json_carries_alts`, `introspect_json_alts_is_empty_for_a_plain_capture`).

## The nesting bound is reported (0.24.0)

`capture_func_type` increments a depth counter around every function-typed capture
and refuses to descend once `depth >= MAX_PARSE_DEPTH` (64). `match_one` turns any
sub-failure into a rewind, which used to discard the refusal and let the shape-level
expectation stand. Two fields on the parser carry the information past the rewind:

```text
  field        type                  set where                         reset where
  ──────────   ───────────────────   ───────────────────────────────   ──────────────────────
  depth_err    Option<CapyError>     capture_func_type, at the         parse_stmt entry; taken
                                     bound; only if still None         (take) when reported
                                     (the first one reached is kept)
  fail_code    &'static str          parse_stmt, when it returns       parse_stmt entry (E0001);
                                     the bound error (E0003)           consumed by the recovery
                                                                       diagnostic (mem::replace)
```

- The error message is
  ``nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow``;
  the quoted part is `el.alternatives()` joined with ` | `. Its position is the token
  where the bound was reached.
- `parse_stmt` resets `depth_err = None` and `fail_code = NO_MATCH` on entry, so a
  bound error from an earlier statement cannot colour a later one. `parse_stmt` is
  also entered for block bodies, so the reset applies there too.
- When no shape matched, the order of reporting at the end of `parse_stmt` is:
  (1) `block_err`, an error from a matched block opener's body; (2) `depth_err`,
  with `fail_code = NESTING_TOO_DEEP` (`E0003`); (3) the furthest-failure error
  (`furthest_error`, code `E0001`); (4) "no library function matches token …".
  So the bound is reported **before** the furthest-failure error, which is why a
  statement that died against it no longer reads as ``expected `)`, found "1"``.
- The recovery loop (the `Err(e) if self.recover` arm) takes `fail_code` with `mem::replace`
  (back to `NO_MATCH`) when it builds the `Diagnostic`, so the diagnostic carries
  `E0003` for that statement and `E0001` for the next. `Library::run` (no recovery)
  returns the same error as `Err`, so `capy run` prints the new message with the
  caret at the bound token.
- Only the reporting changed. `MAX_PARSE_DEPTH` is still 64; 31 call levels parse
  and 32 are refused in `samples/expression-grammar` (`DEMO-2026-0004` U-03).

## The version string (0.24.0)

`rust/cli/src/main.rs` defines `VERSION` as `option_env!("CAPY_VERSION")` when set at
compile time, else `env!("CARGO_PKG_VERSION")`. `capy version` and `capy --version`
print `capy <VERSION>`; an unstamped build therefore prints `capy 0.24.0`, no longer
`capy dev`. The wasm `capy_version` export already used the same fallback (it also
ignores an empty `CAPY_VERSION`).

## Last Verified Version

0.24.0, `main` at `cb23972`, 2026-10-07; `cargo test --workspace`: 135 passed,
0 failed (`DEMO-2026-0004` U-07). Each statement in the 0.24.0 sections above was
checked against `make_parser.rs`, `rust/wasm/src/lib.rs` and `rust/cli/src/main.rs`.

## Known limitations

- No byte offsets on `Span`.
- `E0002` (an unclosed delimiter) is still reserved and unemitted.
- The nesting bound is fixed at 64 captures and is not configurable.
- Only leading comments are attached; trailing and interior are retained but
  unattached.
- Alternatives must be library functions; built-in and declared types are
  refused at load.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document |
| 2 | 2026-09-16 | Olivier | Added the 0.22.0 diagnostics, recovery and value-expression sections |
| 3 | 2026-10-07 | Olivier | Added ordered choice (0.23.0): lib-parser, loader validation and guard, matcher `match_alt`, public surfaces; refreshed scope and component table |
| 4 | 2026-10-07 | Olivier | 0.24.0: documented `depth_err` / `fail_code` and `E0003`, the wasm `alts` field and the version fallback; removed the statements that became false (bound message consumed by the rewind; generic "no library function matches" at the bound; browser JSON without `alts`); last verified 0.24.0 |
