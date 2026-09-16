---
document_id: ARCH-2026-0001
title: Architecture — AST Node Shape and the Trivia Boundary
document_type: architecture
status: active

created_date: 2026-09-16
last_updated: 2026-09-16
document_revision: 1

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
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - architects
  - engineers

scope: Records the AST node shape at 0.21.0 and the boundary that keeps comment trivia out of the matcher.

reason: DOCUMENTATION.md section 31 requires architecture documentation to be synchronized when component boundaries or data shape change.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - ast
  - architecture

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Architecture — AST Node Shape and the Trivia Boundary

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Node shape

```text
Block
 └─ stmts: Vec<FuncCall>          ← type UNCHANGED at 0.21.0
     FuncCall
       ├─ func: String
       ├─ line, col               ← unchanged; == span.start
       ├─ span: Span              ← NEW  (whole statement, body + closer)
       ├─ leading_comments: Vec<Span>  ← NEW
       ├─ captures: BTreeMap<String, CaptureValue>
       │    CaptureValue
       │      ├─ text / expr / is_expr
       │      ├─ span: Span       ← NEW  (exactly this value's tokens)
       │      └─ sub: Vec<FuncCall>    ← nonterminal matches
       ├─ body / closer / sections
```

`Block.stmts` keeps its type deliberately. PLAN-C adds recovery, and the error
representation chosen there is a **parallel `Block.errors`** rather than widening
`stmts` to an enum — changing an existing public field's type breaks every walker,
and `#[non_exhaustive]` does not protect against it.

## The trivia boundary

```text
              ┌──────────────── tokenize_impl ────────────────┐
              │                                                │
 tokenize ────┤ keep_comments = false → no Comment tokens      │
 tokenize_with┤ keep_comments = false → no Comment tokens      │
 ..._trivia ──┤ keep_comments = true  → Comment tokens emitted │
              └───────────────────────┬────────────────────────┘
                                      ▼
                             make_parser::parse
                                      │  strips Comment here
                        ┌─────────────┴─────────────┐
                        ▼                           ▼
                  matcher (never sees        leading: index → Vec<Span>
                   a Comment token)                 │
                                                    ▼
                                         FuncCall.leading_comments
```

The boundary is the design's load-bearing part. Retaining comments is only safe
because no matcher ever encounters the new token kind — which is also why the
manifest and inner-DSL path (`tokenize`) opts out entirely rather than being
taught to skip them.

## Ownership

| Concern | Owner |
|---|---|
| Where a node came from | `Span`, populated in `make_parser` |
| Whether a grammar can terminate | `reject_left_recursion`, in the loader |
| Whether *input* can terminate | `MAX_PARSE_DEPTH`, in the parser |
| Whether trivia is visible | the lexer entry point the caller chooses |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial document |
