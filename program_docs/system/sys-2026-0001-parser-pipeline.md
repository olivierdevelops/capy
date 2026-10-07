---
document_id: SYS-2026-0001
title: System — Lexer and Parser Pipeline as Implemented
document_type: system
status: active

created_date: 2026-09-16
last_updated: 2026-10-07
document_revision: 3

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

scope: Describes the implemented lexer-to-AST pipeline through 0.23.0, including trivia handling, span population, the recursion guards, diagnostics, value expressions and ordered choice in a capture type.

reason: DOCUMENTATION.md section 31 requires system documentation to describe the current implemented system whenever behaviour or internal interfaces change.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001
  - PLAN-2026-0003
  - PROP-2026-0004
  - ADR-0003
  - DEMO-2026-0003

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
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Summary

This describes what the code does through 0.23.0, not what was intended.

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
| Parser | `rust/src/orchestrator/features/make_parser.rs` | trivia strip, span population, `MAX_PARSE_DEPTH`; `capture_func_type_inner` and `match_alt` (ordered choice, 0.23.0) |
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
- Depth is still bounded by `MAX_PARSE_DEPTH`; the bound's message is consumed
  by the rewind, so a too-deep input reports the generic expectation.

**Public surfaces.**

| Surface | What it shows |
|---|---|
| `ArgInfo.type_` | alternative 1 (unchanged meaning) |
| `ArgInfo.alts` | alternatives 2…n; empty for a single-type capture |
| `capy docs` | the Type column joins `type_` and `alts` with ` \| ` |
| `capy ast --json` | `sub[].func` is the matched alternative |

The browser introspection JSON (`rust/wasm/src/lib.rs`) emits `type` only and does
not carry `alts`.

## Last Verified Version

0.23.0 working tree on top of `84f984c`, 2026-10-07; `cargo test --workspace`:
128 passed, 0 failed (`DEMO-2026-0003` U-13).

## Known limitations

- No byte offsets on `Span`.
- The depth-limit error surfaces as the generic "no library function matches".
- Only leading comments are attached; trailing and interior are retained but
  unattached.
- Alternatives must be library functions; built-in and declared types are
  refused at load.
- The browser introspection JSON does not carry `alts`.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document |
| 2 | 2026-09-16 | Olivier | Added the 0.22.0 diagnostics, recovery and value-expression sections |
| 3 | 2026-10-07 | Olivier | Added ordered choice (0.23.0): lib-parser, loader validation and guard, matcher `match_alt`, public surfaces; refreshed scope and component table |
