---
document_id: IDX-2026-0001
title: Document Index
document_type: reference
status: active

created_date: 2026-09-16
last_updated: 2026-10-07
document_revision: 2

authors:
  - Olivier

owner: Engineering Documentation Team
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - docs

affected_versions: not-applicable

applicable_environments:
  - development

audience:
  - engineers
  - release-managers

scope: Lookup of every document by ID, type, status, component and owner.

reason: DOCUMENTATION.md section 17 requires generated indexes allowing lookup by type, component, version, date, owner and lifecycle status.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - index
  - governance

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Document Index

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-10-07
> **Owner:** Engineering Documentation Team

Lookup by ID, type, status, component and owner, per `DOCUMENTATION.md` §17.

## All documents

| ID | Title | Type | Status | Components | Owner | Last Updated | Path |
|---|---|---|---|---|---|---|---|
| REF-2026-0001 | Documentation, Traceability and Release Standard | reference | active | docs | Engineering Documentation Team | 2026-09-16 | [`DOCUMENTATION.md`](../../DOCUMENTATION.md) |
| PLAN-2026-0001 | Implementation Plan — Recursion Guard, Source Spans and Comment Retention | plan | completed | capy-core, capy-cli, docs | Capy Engine | 2026-09-16 | [`plans/plan-2026-0001-recursion-guard-spans-comment-retention.md`](../plans/plan-2026-0001-recursion-guard-spans-comment-retention.md) |
| PROP-2026-0001 | Parser Foundations — Spans, Error Recovery, Structured AST Output and Expression Trees | proposal | implemented | capy-core, capy-cli, capy-wasm-abi, docs | Capy Engine | 2026-09-16 | [`proposals/prop-2026-0001-parser-foundations.md`](../proposals/prop-2026-0001-parser-foundations.md) |

| PROP-2026-0002 | Public Documentation and Samples for the 0.22.0 Parser Surface | proposal | implemented | docs, samples, capy-cli | Capy Engine | 2026-09-16 | [`proposals/prop-2026-0002-public-docs-and-samples-0.22.0.md`](../proposals/prop-2026-0002-public-docs-and-samples-0.22.0.md) |
| PROP-2026-0003 | Expression Trees Over JSON, and the Parenthesis Round-Trip Fix | proposal | draft | capy-core, docs | Capy Engine | 2026-09-17 | [`proposals/prop-2026-0003-expression-trees-and-round-trip.md`](../proposals/prop-2026-0003-expression-trees-and-round-trip.md) |
| PROP-2026-0004 | Ordered Alternation for Nonterminals | proposal | implemented | capy-core, docs, samples | Capy Engine | 2026-10-07 | [`proposals/prop-2026-0004-nonterminal-alternation.md`](../proposals/prop-2026-0004-nonterminal-alternation.md) |
| ADR-0001 | Approve PROP-2026-0001 and its Frozen Contracts | decision | approved | capy-core | Capy Engine | 2026-09-16 | [`decisions/adr-0001-approve-parser-foundations.md`](../decisions/adr-0001-approve-parser-foundations.md) |
| TEST-2026-0001 | Test — AST Source Spans | test | completed | capy-core | Capy Engine | 2026-09-16 | [`testing/test-2026-0001-ast-spans.md`](../testing/test-2026-0001-ast-spans.md) |
| TEST-2026-0002 | Test — Left-Recursion Guard and Parse Depth Bound | test | completed | capy-core | Capy Engine | 2026-09-16 | [`testing/test-2026-0002-recursion-guard.md`](../testing/test-2026-0002-recursion-guard.md) |
| TEST-2026-0003 | Test — No Behaviour Change, and Comment Retention | test | completed | capy-core, capy-cli, capy-wasm-abi | Capy Engine | 2026-09-16 | [`testing/test-2026-0003-no-behaviour-change.md`](../testing/test-2026-0003-no-behaviour-change.md) |
| TEST-2026-0004 | Test — Measured Results for the 0.21.0 Parser Changes | test | completed | capy-core, capy-wasm-abi | Capy Engine | 2026-09-16 | [`testing/test-2026-0004-measured-results.md`](../testing/test-2026-0004-measured-results.md) |
| RPT-2026-0001 | Validation of PLAN-2026-0001 | report | completed | capy-core, capy-cli | Capy Engine | 2026-09-16 | [`reports/rpt-2026-0001-plan-2026-0001-validation.md`](../reports/rpt-2026-0001-plan-2026-0001-validation.md) |
| DEMO-2026-0001 | Release Verification Guide — 0.21.0 | demo | active | capy-core, capy-cli | Capy Engine | 2026-09-16 | [`demos/demo-2026-0001-release-verification-0.21.0.md`](../demos/demo-2026-0001-release-verification-0.21.0.md) |
| MAN-2026-0001 | Manual — Source Positions, Comments and Recursion Limits | manual | active | capy-core | Capy Engine | 2026-09-16 | [`manuals/man-2026-0001-spans-comments-recursion.md`](../manuals/man-2026-0001-spans-comments-recursion.md) |
| SYS-2026-0001 | System — Lexer and Parser Pipeline as Implemented | system | active | capy-core | Capy Engine | 2026-09-16 | [`system/sys-2026-0001-parser-pipeline.md`](../system/sys-2026-0001-parser-pipeline.md) |
| ARCH-2026-0001 | Architecture — AST Node Shape and the Trivia Boundary | architecture | active | capy-core | Capy Engine | 2026-09-16 | [`architecture/components/arch-2026-0001-ast-node-shape.md`](../architecture/components/arch-2026-0001-ast-node-shape.md) |
| REL-0.21.0 | Release 0.21.0 — Parser Foundations | release | completed | capy-core, capy-cli | Release Management | 2026-09-16 | [`releases/rel-0.21.0-release-notes.md`](../releases/rel-0.21.0-release-notes.md) |

| PLAN-2026-0002 | Implementation Plan — Diagnostics, Error Recovery, AST Output and Operator Precedence | plan | completed | capy-core, capy-cli, docs | Capy Engine | 2026-09-16 | [`plans/plan-2026-0002-diagnostics-recovery-ast-output-precedence.md`](../plans/plan-2026-0002-diagnostics-recovery-ast-output-precedence.md) |
| TEST-2026-0005 | Test — Diagnostics and Error Recovery | test | completed | capy-core | Capy Engine | 2026-09-16 | [`testing/test-2026-0005-diagnostics-and-recovery.md`](../testing/test-2026-0005-diagnostics-and-recovery.md) |
| TEST-2026-0006 | Test — Operator Precedence and Expression Round-Trip | test | completed | capy-core | Capy Engine | 2026-09-16 | [`testing/test-2026-0006-precedence-and-round-trip.md`](../testing/test-2026-0006-precedence-and-round-trip.md) |
| TEST-2026-0007 | Test — AST Output Surface | test | completed | capy-core, capy-cli | Capy Engine | 2026-09-16 | [`testing/test-2026-0007-ast-output.md`](../testing/test-2026-0007-ast-output.md) |
| TEST-2026-0008 | Test — Measured Results for 0.22.0 | test | completed | capy-core, capy-wasm-abi | Capy Engine | 2026-09-16 | [`testing/test-2026-0008-measured-results-0.22.0.md`](../testing/test-2026-0008-measured-results-0.22.0.md) |
| RPT-2026-0002 | Validation of PLAN-2026-0002 | report | completed | capy-core, capy-cli | Capy Engine | 2026-09-16 | [`reports/rpt-2026-0002-plan-2026-0002-validation.md`](../reports/rpt-2026-0002-plan-2026-0002-validation.md) |
| DEMO-2026-0002 | Release Verification Guide — 0.22.0 | demo | active | capy-core, capy-cli | Capy Engine | 2026-09-16 | [`demos/demo-2026-0002-release-verification-0.22.0.md`](../demos/demo-2026-0002-release-verification-0.22.0.md) |
| REL-0.22.0 | Release 0.22.0 — Diagnostics, Recovery, AST Output and Precedence | release | completed | capy-core, capy-cli | Release Management | 2026-09-16 | [`releases/rel-0.22.0-release-notes.md`](../releases/rel-0.22.0-release-notes.md) |

| ADR-0003 | Approve PROP-2026-0004 — Ordered Alternation for Nonterminals | decision | approved | capy-core, docs, samples | Capy Engine | 2026-10-07 | [`decisions/adr-0003-approve-nonterminal-alternation.md`](../decisions/adr-0003-approve-nonterminal-alternation.md) |
| PLAN-2026-0003 | Implementation Plan — Ordered Alternation for Nonterminals (0.23.0) | plan | completed | capy-core, docs, samples | Capy Engine | 2026-10-07 | [`plans/plan-2026-0003-nonterminal-alternation.md`](../plans/plan-2026-0003-nonterminal-alternation.md) |
| TEST-2026-0009 | Test — Ordered Alternation in a Capture Type | test | completed | capy-core, samples, docs | Capy Engine | 2026-10-07 | [`testing/test-2026-0009-ordered-alternation.md`](../testing/test-2026-0009-ordered-alternation.md) |
| TEST-2026-0010 | Test — Measured Results for 0.23.0 | test | completed | capy-core, capy-wasm-abi | Capy Engine | 2026-10-07 | [`testing/test-2026-0010-measured-results-0.23.0.md`](../testing/test-2026-0010-measured-results-0.23.0.md) |
| TEST-2026-0011 | Test — Carried Docs and Samples Verification (PROP-2026-0002 R1 to R19) | test | completed | docs, samples, capy-core, playground | Capy Engine | 2026-10-07 | [`testing/test-2026-0011-carried-docs-and-samples-verification.md`](../testing/test-2026-0011-carried-docs-and-samples-verification.md) |
| RPT-2026-0003 | Validation of PLAN-2026-0003 — Ordered Alternation for Nonterminals (0.23.0) | report | completed | capy-core, docs, samples | Capy Engine | 2026-10-07 | [`reports/rpt-2026-0003-plan-2026-0003-validation.md`](../reports/rpt-2026-0003-plan-2026-0003-validation.md) |
| DEMO-2026-0003 | Release Verification Guide — 0.23.0 | demo | active | capy-core, capy-cli, docs, samples | Capy Engine | 2026-10-07 | [`demos/demo-2026-0003-release-verification-0.23.0.md`](../demos/demo-2026-0003-release-verification-0.23.0.md) |
| MAN-2026-0002 | Manual — Ordered Choice in a Capture Type | manual | active | capy-core, capy-cli | Capy Engine | 2026-10-07 | [`manuals/man-2026-0002-ordered-choice.md`](../manuals/man-2026-0002-ordered-choice.md) |
| STD-2026-0000 | Project Standards Index | standard | active | All | Capy Engine | 2026-09-16 | [`standards/index.md`](../standards/index.md) |
| STD-2026-0001 | Project Goals | standard | active | All | Capy Engine | 2026-09-16 | [`standards/project-goals.md`](../standards/project-goals.md) |
| STD-2026-0002 | Engineering Philosophy | standard | active | All | Capy Engine | 2026-09-16 | [`standards/engineering-philosophy.md`](../standards/engineering-philosophy.md) |
| STD-2026-0003 | Codebase Rules | standard | active | All | Capy Engine | 2026-09-16 | [`standards/codebase-rules.md`](../standards/codebase-rules.md) |
| STD-2026-0004 | Architecture Rules | standard | active | All | Capy Engine | 2026-09-16 | [`standards/architecture-rules.md`](../standards/architecture-rules.md) |
| STD-2026-0005 | Quality Expectations and Validation Gates | standard | active | All | Capy Engine | 2026-09-16 | [`standards/quality-expectations.md`](../standards/quality-expectations.md) |
| STD-2026-0006 | Approved Exceptions | standard | active | All | Capy Engine | 2026-09-16 | [`standards/exceptions.md`](../standards/exceptions.md) |
| ADR-0002 | Consolidate PLAN-B through PLAN-E into One Plan and One Release | decision | approved | capy-core | Capy Engine | 2026-09-16 | [`decisions/adr-0002-consolidate-plans-b-to-e.md`](../decisions/adr-0002-consolidate-plans-b-to-e.md) |

## By status

| Status | Documents |
|---|---|
| active | REF-2026-0001, STD-2026-0000…0006, DEMO-2026-0001, DEMO-2026-0002, DEMO-2026-0003, MAN-2026-0001, MAN-2026-0002, SYS-2026-0001, ARCH-2026-0001 |
| draft | PROP-2026-0003 |
| approved | ADR-0001, ADR-0002, ADR-0003 |
| completed | TEST-2026-0001…0011, RPT-2026-0001…0003, PLAN-2026-0001…0003, REL-0.21.0, REL-0.22.0, REL-0.23.0 |
| implemented | PROP-2026-0001, PROP-2026-0002, PROP-2026-0004 |

## By component

| Component | Documents |
|---|---|
| capy-core | PROP-2026-0001, PROP-2026-0003, PROP-2026-0004, PLAN-2026-0001, PLAN-2026-0003, ADR-0003, TEST-2026-0009…0011, RPT-2026-0003, DEMO-2026-0003, MAN-2026-0002 |
| capy-cli | PROP-2026-0001, PROP-2026-0002, PLAN-2026-0001 |
| capy-wasm-abi | PROP-2026-0001 |
| docs | REF-2026-0001, PROP-2026-0001, PROP-2026-0002, PROP-2026-0003, PROP-2026-0004, PLAN-2026-0001, PLAN-2026-0003, ADR-0003 |

## Next available IDs

| Prefix | Next |
|---|---|
| `PROP` | PROP-2026-0005 |
| `STD` | STD-2026-0007 |
| `PLAN` | PLAN-2026-0004 |
| `TEST` | TEST-2026-0012 |
| `DEMO` | DEMO-2026-0004 |
| `MAN` | MAN-2026-0003 |
| `SYS` | SYS-2026-0002 |
| `ARCH` | ARCH-2026-0002 |
| `ADR` | ADR-0004 |
| `RPT` | RPT-2026-0004 |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial index |
| 2 | 2026-10-07 | Olivier | Added the 0.23.0 documents (ADR-0003, PLAN-2026-0003, TEST-2026-0009…0011, RPT-2026-0003, DEMO-2026-0003, MAN-2026-0002); PROP-2026-0002 and PROP-2026-0004 now `implemented`; next-ID table advanced |
