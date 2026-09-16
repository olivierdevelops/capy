# Program Documentation

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Engineering Documentation Team

Entry point for all documentation governed by
[`DOCUMENTATION.md`](../DOCUMENTATION.md) — the Documentation, Traceability and
Release Standard (`REF-2026-0001`).

## Summary

This tree holds planning, decision and release documentation. It is distinct from
[`docs/`](../docs/), which is the **user-facing** documentation site published to
gh-pages via `mkdocs`. Nothing here is published to that site.

| Tree | Audience | Published |
|---|---|---|
| `program_docs/` | engineers, architects, release managers | no |
| `docs/` | Capy users and library authors | yes — gh-pages |

## Directories in use

The canonical structure is defined in `DOCUMENTATION.md` §3.1. Folders are created
as they are needed; the standard does not require them all up front.

| Directory | Type | Prefix | Present |
|---|---|---|---|
| `proposals/` | proposal | `PROP` | yes |
| `index/` | — | — | yes |
| `system/` | system | `SYS` | yes |
| `architecture/` | architecture | `ARCH` | yes |
| `manuals/` | manual | `MAN` | yes |
| `demos/` | demo | `DEMO` | yes |
| `standards/` | standard | `STD` | **not yet** — see PROP-2026-0001 Open Question 10 |
| `plans/` | plan | `PLAN` | yes |
| `decisions/` | decision | `ADR` | yes |
| `releases/` | release | `REL` | yes |
| `reports/` | report | `RPT` | yes |
| `testing/` | test | `TEST` | yes |

## Index

- [Document index](index/document-index.md)
- [Component index](index/component-index.md)
- [Version index](index/version-index.md)
- [Decision index](index/decision-index.md)

## Conventions

- Every document carries the YAML front matter of `DOCUMENTATION.md` §5 and the
  visible header of §7, and the two must agree.
- File names are `<document-id-in-lowercase>-<short-description>.md` (§8).
- Dates are ISO 8601 (§9).
- Significant changes are recorded in each document's Change History table, whose
  last row matches its `document_revision` and `last_updated` (§10).
