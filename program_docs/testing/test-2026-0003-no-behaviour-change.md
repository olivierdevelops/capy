---
document_id: TEST-2026-0003
title: Test — No Behaviour Change, and Comment Retention
document_type: test
status: completed

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
  - capy-cli
  - capy-wasm-abi

affected_versions:
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the regression, compatibility, build, lint and comment-retention tests proving nothing that parsed before behaves differently now.

reason: This release changes the parser and the lexer; the binding constraint is that no existing library or script changes behaviour.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - regression
  - compatibility
  - comments

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — No Behaviour Change, and Comment Retention

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi

## Purpose

Prove PLAN-2026-0001 R12 (nothing that parses today changes) and R27 (comments
are retained and attached, with the node's own span excluding them).

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0001 R12 | No library or script changes behaviour |
| PLAN-2026-0001 R27 | Comments retained, leading comments attached, span excludes them |
| PLAN-2026-0001 R5 | `line`/`col` render locals unchanged |

## Preconditions

Full workspace built; `deno` available for the wasm harness.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable 1.90.0, `deno` for `wasm_check.sh`,
Python 3.9 + mkdocs for the docs build.

## Test Data

All 117 checked-in sample libraries, the golden corpus (124 cases), and the wasm
conformance corpus.

## Procedure

```sh
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
for f in samples/*/lib.capy; do capy check "$f"; done
cargo test --manifest-path rust/Cargo.toml --test golden
./rust/devtools/wasm_check.sh
mkdocs build --strict
```

## Expected Results

Every gate green; golden output byte-identical; no library rejected.

## Actual Results

| Gate | Result |
|---|---|
| `cargo build --workspace` | green | 
| `cargo clippy --all-targets -D warnings` | 0 findings |
| `cargo test --workspace` | 90 passed, 0 failed |
| `capy check` over samples | 117/117 |
| Golden corpus | 116 pass / 0 fail / 8 skip — byte-identical |
| wasm check | PASS 113 / FAIL 0 |
| `mkdocs build --strict` | exit 0 |

Comment-retention results (in `rust/tests/ast_spans.rs`):

| Test | Assertion | Outcome |
|---|---|---|
| `leading_comment_is_attached` | comment above a statement attaches to it | PASS |
| `node_span_excludes_its_comments` | statement span starts at `greet`, not the comment | PASS |
| `stacked_comments_all_attach_in_order` | three comments, source order preserved | PASS |
| `trailing_comment_does_not_attach_backwards` | a trailing comment leads nothing | PASS |
| `comment_at_eof_attaches_to_nothing` | no panic, no mis-attachment | PASS |
| `comments_do_not_change_rendered_output` | same output with and without comments | PASS |
| `no_markers_means_no_comments` | a library declaring no markers has no comment syntax | PASS |

## Result

PASS

## Evidence

The trivia design is why R12 holds: comment tokens are emitted only by
`tokenize_with_trivia` (the user-script path) and are stripped by the parser
before any matching occurs, so no matcher ever sees a token kind it has not seen
before. `tokenize` — the manifest and inner-DSL path — is unchanged.

## Evidence Sources

- `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --manifest-path rust/Cargo.toml --test golden`
- `./rust/devtools/wasm_check.sh`
- `rust/tests/ast_spans.rs` (committed)

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None.

## Related Documents

- PLAN-2026-0001
- PROP-2026-0001 review finding 3 (comment retention), review residual 1 (span exclusion)

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
