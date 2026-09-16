# Decision Index

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Engineering Documentation Team

Lookup by decision, per `DOCUMENTATION.md` §17.

| ID | Decision | Status | Date | Supersedes | Related |
|---|---|---|---|---|---|
| ADR-0001 | Approve PROP-2026-0001 and freeze the `Span`, comment-attachment and error-node contracts | approved | 2026-09-16 | — | PROP-2026-0001, PLAN-2026-0001 |

## Contracts frozen by ADR-0001

| Contract | Decision |
|---|---|
| `Span` indexing | 1-indexed, source-absolute |
| `Span.end_col` | exclusive |
| Byte offsets | deferred; `#[non_exhaustive]` keeps the door open |
| Comment attachment | leading only |
| Node span vs comments | span excludes attached comments |
| Error nodes | parallel `Block.errors`, not a change to `stmts`' type |
| Left recursion | rejected at library-load time |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial index |
