---
document_id: MAN-2026-0003
title: Manual — Calling Capy from C
document_type: manual
status: active

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 1

authors:
  - Olivier
  - Claude (Sonnet 5.5)

owner: Capy Engine
reviewers:
  - Capy Engine

systems:
  - Capy

components:
  - capy-ffi
  - capy-cli
  - docs

affected_versions:
  from: "0.26.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - embedders

scope: The task-oriented manual for the C ABI and the --with-expr option, current as of 0.26.0.

reason: DOCUMENTATION.md section 30 requires the manual to be updated for every user-visible capability.

related_documents:
  - PROP-2026-0008
  - PLAN-2026-0005
  - DEMO-2026-0005
  - ARCH-2026-0002
  - SYS-2026-0001
  - TEST-2026-0015
  - INC-2026-0002
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - manual
  - ffi
  - c-abi

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Manual — Calling Capy from C

> **Status:** Active
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-cli, docs

## Summary

How to link Capy into a program that is not written in Rust, parse source with a grammar you compile once, and
read the result. The complete function reference lives in the user documentation
([`docs/c-abi.md`](../../docs/c-abi.md)); this manual is the task-oriented guide and the maintainer's view.

## Purpose

Answer: *what can I do, why would I, and where are the instructions?*

## Reading Order

1. What's new → 2. Concepts → 3. Install → 4. Tasks → 5. Errors → 6. Limits and troubleshooting.

## What's New in the Current Supported Release

| Since | Capability | Task | Verified demo |
|---|---|---|---|
| 0.26.0 | A C ABI over the parse tree | all tasks below | `DEMO-2026-0005` U-01…U-04, U-07, U-09 |
| 0.26.0 | `capy ast --json --with-expr` | *Task: read the typed expression from the CLI* | U-05 |
| 0.26.0 | Deep expression nesting is bounded | *Edge cases* | U-06 |

## Project Goals and Boundaries

Goal: let a compiler or tool in another language use Capy as its parser without a process per file or a JSON
reader. Non-goals: transpilation through the C interface (`run`), host callbacks, byte offsets, a stable ABI
across major versions (the promise is *additive within a version*).

## Concepts

| Term | Meaning |
|---|---|
| Library (`CapyLibrary`) | a compiled grammar; read-only; share it across threads |
| Parse (`CapyParse`) | the result of parsing one source; one owner; frees everything borrowed from it |
| Id (`CapyId`) | a number naming a node of one parse; `0` = none |
| `CapyStr` | `{ptr, len}` UTF-8, borrowed, not NUL-terminated; `ptr == NULL` = absent |
| Poisoned | a handle on which a Rust panic was caught; every later call returns zero |

## Architecture and Mental Model

```text
   grammar text ──► capy_library_new ──► CapyLibrary ──┐
                                                       ├─► capy_parse(lib, source) ──► CapyParse ──► ids, text, spans
   source text  ────────────────────────────────────────┘                                   │
                                                              free it ◄─────────────────────┘ (and every CapyStr dies)
```

Detail: `ARCH-2026-0002`.

## Installation and Setup

```sh
git clone … && cd capy
cargo build --manifest-path rust/Cargo.toml --profile ffi -p capy-ffi     # use the ffi profile
#   rust/target/ffi/libcapy_ffi.a   rust/target/ffi/libcapy_ffi.dylib   rust/ffi/include/capy.h
```

Link with dead-stripping (`-Wl,-dead_strip` on macOS, `-Wl,--gc-sections` on Linux): a complete program is about
1.3 MB, against 4.4 MB (clang defaults) or 21.6 MB (`zig cc` defaults) without it.

## Feature Catalogue

| What can I do | Why | Instructions |
|---|---|---|
| Compile a grammar once and parse many files | no recompilation per file | *Task: parse a file* |
| Walk statements, captures, sections, bodies | build your own front end | *Task: walk the tree* |
| Read typed expressions | lower `a + b * c` without re-parsing text | *Task: read an expression* |
| Report Capy's diagnostics unchanged | same codes and positions as `capy ast` | *Task: report errors* |
| Dump the parse as JSON | tooling and differential tests | *Task: dump JSON* |
| Parse from several threads | one library, many parses | *Task: use threads* |

## Configuration and Environment Variables

None. The library always uses the sandboxed no-op host: `env`, `arg` and `read_file` are inert.

## Task-Oriented Workflows

### Task: parse a file

Prerequisites: U-01 done. Input: a grammar and a source, both UTF-8.

```text
  capy_library_new ──ok──► capy_parse ──► CapyParse*  ──► capy_is_clean?
        │ fail                  │ status != OK
        ▼                       ▼
   CapyError*              NULL + status (NULL / UTF-8 / limit / panic)
```

Complete code: `examples/c/walk.c`. Expected: `greet(who=world)` from `examples/c/run.sh walk`.
Errors: a grammar that does not compile gives status `7` and a `CapyError`; free it with `capy_error_free`.

### Task: walk the tree

`capy_root` → `capy_block_len` / `capy_block_stmt` → `capy_node_func`, `capy_node_capture` (by name) or
`capy_node_capture_count` + `capy_node_capture_name` (by position, sorted) → `capy_cap_text`, `capy_cap_span`,
`capy_cap_sub`. A statement's block body is `capy_node_body`; a multi-section block's parts are
`capy_node_section*`. Side effect: none; reads are pure.

### Task: read an expression

An `any`-typed capture has `capy_cap_expr`. Walk it by `capy_expr_kind`; see the table in `docs/c-abi.md`.
Verified demo: `DEMO-2026-0005` U-04 (`examples/c/run.sh exprs`).

### Task: report errors

```text
   capy_is_clean(p) == 0 ──► for i in capy_diag_count(p):
        capy_diag_code / _message / _help / _primary / _label* / _context*
   statements that DID parse are still in the tree; failed regions are error nodes (capy_block_error*)
```

Verified demo: U-03.

### Task: dump JSON

`capy_parse_to_json(p, CAPY_JSON_EXPR | CAPY_JSON_PRETTY, &buf, &len)` then `capy_buf_free(buf, len)`.

### Task: read the typed expression from the CLI

```sh
capy ast lib.capy script.capy --json --with-expr
```

Without `--with-expr` the output has no `expr` member (byte-identical to 0.25.0). With it, every capture gains
`expr`. `--with-expr` without `--json` is refused (`--with-expr needs --json`, exit 1). Demo: U-05.

### Task: use threads

Share one `CapyLibrary`; give each thread its own `CapyParse`. Do not free a parse another thread is reading.

## Complete CLI Reference

| Command | Effect |
|---|---|
| `capy ast LIB SCRIPT --json --with-expr` | AST JSON including each capture's typed expression |
| `examples/c/run.sh walk\|diagnose\|exprs\|hostile\|misuse\|all` | build and run the C programs |
| `rust/ffi/check_header.sh [--update]` | verify or regenerate `capy.h` |
| `rust/devtools/ast_json_compat.sh BASELINE [NEW]` | compare default `ast --json` with a baseline binary |

## Complete API and Event Reference

75 functions; see `rust/ffi/include/capy.h` (authoritative) and the grouped table in `docs/c-abi.md`.

## UI Screen and Interaction Reference

Not applicable: no user interface.

## Errors and Recovery Reference

| Status | Meaning | Recovery |
|---|---|---|
| 1 `CAPY_E_NULL` | a required pointer was NULL | fix the call |
| 2 `CAPY_E_ARG` | an index or flag out of range | fix the call |
| 3 `CAPY_E_KIND` | the id is the wrong kind of node | use the right accessor |
| 4 `CAPY_E_UTF8` | grammar or path not UTF-8 | convert |
| 5 `CAPY_E_PANIC` | a Rust panic was caught | report it; the handle is poisoned, free it |
| 6 `CAPY_E_LIMIT` | tree too large for 2^24 − 2 ids | split the input |
| 7 `CAPY_E_LOAD` | grammar failed to compile | read the `CapyError` |

## Examples and Demos

`examples/c/*.c`; `DEMO-2026-0005`.

## Operations, Observability and Maintenance

`capy_abi_version()` at start-up against `CAPY_ABI_VERSION`; `capy_panics_are_caught()` to detect an `abort` build.
Maintainers: regenerate the header with `rust/ffi/check_header.sh --update` after any change to an export, and
bump the ABI version only for a breaking change.

## Edge Cases

- **Deep nesting.** 63 nested pairs of parentheses (or brackets, `not` chains, …) parse; the 64th is refused with
  a diagnostic. Before 0.26.0 about 20 000 aborted the process (`INC-2026-0002`).
- **Invalid UTF-8 in source** is replaced, not rejected; in a grammar it is `CAPY_E_UTF8`.
- **Empty source** is a clean parse of zero statements.
- **An optional capture that took its default** has an unset span (`start_line == 0`).

## Failure Modes, Recovery and Rollback

A poisoned handle returns zero from every call; free it and parse again. To roll back to 0.25.0, remove the
`capy-ffi` build step; nothing else depends on it.

## Security and Compatibility

No pointer is retained after a call returns except borrowed strings tied to the handle. Every id is validated.
Misuse that cannot be detected: use after free, double free, a pointer/length that is not readable memory.
Compatibility: additive within ABI 1; the default `capy ast --json` is unchanged.

## Limitations

macOS arm64 verified; Linux cross-built only. Parsing only. No byte offsets. The archive is 31 MB (link with
dead-stripping). The over-deep expression message is the generic `E0001`.

## Troubleshooting References

`DEMO-2026-0005` §Troubleshooting; `INC-2026-0002`; `INC-2026-0003`; `TRBL-2026-0001`.

## Glossary

*Handle*, *id*, *poisoned*, *borrowed string*: see Concepts.

## Version Applicability

0.26.0 and later.

## Related Features

`MAN-2026-0001` (spans), `MAN-2026-0002` (ordered choice).

## Related Documents

- `PROP-2026-0008`, `PLAN-2026-0005`, `DEMO-2026-0005`, `ARCH-2026-0002`, `SYS-2026-0001`, `TEST-2026-0015`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial manual for 0.26.0 |
