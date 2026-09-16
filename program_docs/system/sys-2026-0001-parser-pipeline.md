---
document_id: SYS-2026-0001
title: System — Lexer and Parser Pipeline as Implemented
document_type: system
status: active

created_date: 2026-09-16
last_updated: 2026-09-16
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

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Describes the implemented lexer-to-AST pipeline at 0.21.0, including trivia handling, span population and the recursion guards.

reason: DOCUMENTATION.md section 31 requires system documentation to describe the current implemented system whenever behaviour or internal interfaces change.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

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
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Summary

This describes what the code does at 0.21.0, not what was intended.

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
| Parser | `rust/src/orchestrator/features/make_parser.rs` | trivia strip, span population, `MAX_PARSE_DEPTH` |
| Loader | `rust/src/orchestrator/features/make_library_loader.rs` | `reject_left_recursion` after `is_func` resolution |

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

## Known limitations

- No byte offsets on `Span`.
- The depth-limit error surfaces as the generic "no library function matches".
- Only leading comments are attached; trailing and interior are retained but
  unattached.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document |
| 2 | 2026-09-16 | Olivier | Added the 0.22.0 diagnostics, recovery and value-expression sections |
