---
document_id: TEST-2026-0007
title: Test — AST Output Surface
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

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records the tests for Library::parse, capy ast and the JSON schema.

reason: An external analyzer cannot consume the parse result without a supported entry point and a documented shape.

related_documents:
  - PLAN-2026-0002
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - ast
  - json
  - cli

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — AST Output Surface

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Purpose

Prove R6–R9: the public parse entry point, the CLI surface and the JSON schema.

## Procedure

```sh
capy ast samples/transpile-py/lib.capy samples/transpile-py/script.capy
capy ast samples/transpile-py/lib.capy samples/transpile-py/script.capy --json | jq .
cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1
```

## Expected Results

Tree mode prints spans; JSON mode emits one document on stdout with empty
stderr; exit 0 on a clean parse and 1 otherwise; `capy-core` keeps one
dependency.

## Actual Results

| Check | Result |
|---|---|
| Tree mode prints nodes with spans | PASS |
| `--json \| jq .` parses | PASS |
| stderr empty on success | PASS — 0 bytes |
| Exit 0 clean / 1 broken | PASS |
| `schema_version` present | PASS — 1 |
| Broken file still yields a tree | PASS — 3 stmts, 2 errors, 2 diagnostics |
| `capy-core` direct dependencies | PASS — 1 (`regex`) |
| External crate compiles the documented example | PASS |

## Result

PASS

## Evidence

D-01 resolved during P1: `gojson::marshal` handles nested `Obj` and `List`
recursively, so the AST serializes through the existing writer and `capy-core`
adds no dependency.

## Evidence Sources

- `capy ast … --json | jq .`
- `cargo tree --manifest-path rust/Cargo.toml -p capy-core --depth 1`
- `rust/src/domain/ast_json.rs` (committed)

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

None. One compile error was informative rather than a defect: `Severity` is
`#[non_exhaustive]`, so `capy-cli` — an external crate — must carry a wildcard
arm. That is the attribute working as intended.

## Related Documents

- PLAN-2026-0002
- `docs/ast-json.md`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
