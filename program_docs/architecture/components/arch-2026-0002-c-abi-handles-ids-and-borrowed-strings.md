---
document_id: ARCH-2026-0002
title: Architecture — The C ABI: Handles, Ids and Borrowed Strings
document_type: architecture
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
  - capy-core

affected_versions:
  from: "0.26.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: Describes the component boundary, ownership and id model of capy-ffi as built in 0.26.0.

reason: DOCUMENTATION.md section 31 requires architecture documentation when a component boundary is added.

related_documents:
  - PROP-2026-0008
  - ADR-0005
  - SYS-2026-0001
  - ARCH-2026-0001
  - MAN-2026-0003
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - architecture
  - ffi
  - c-abi
  - ownership

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Architecture — The C ABI: Handles, Ids and Borrowed Strings

> **Status:** Active
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core

## Summary

`capy-ffi` is a thin crate over `capy-core`. It owns no parsing logic; it owns **who frees what** and **how a
tree is named across a language boundary**.

```text
   ┌──────────────┐   ┌────────────────────────┐   ┌─────────────────────────────┐
   │  C host      │   │ capy-ffi               │   │ capy-core  (1 dependency)    │
   │              │   │  rlib+staticlib+cdylib │   │                              │
   │  lib  ───────┼──►│ CapyLibrary { Library }├──►│ Library (Send + Sync)        │
   │  parse ──────┼──►│ CapyParse {            │   │ ParseResult { tree, diags }  │
   │  ids / text ◄┼───┤   Box<ParseResult>,    │   │                              │
   │              │   │   entries, kids, tag } │   │                              │
   └──────────────┘   └────────────────────────┘   └─────────────────────────────┘
     no Rust types cross; only opaque pointers, u32 ids, (ptr,len) strings, i32 statuses
```

## Boundaries

| Boundary | Rule |
|---|---|
| `capy-core` → `capy-ffi` | one direction only; `capy-core` has no knowledge of the ABI (`ARCH-002`: still one dependency) |
| `capy-ffi` → host | opaque handles; no Rust layout is exposed except `CapyStr` and `CapySpan` (`repr(C)`) |
| `capy-wasm-abi` ↔ `capy-ffi` | independent crates; the wasm module does not link the FFI (verified: wasm size changes only by the parser fix) |
| build profile | `[profile.ffi]` = `release` + `panic = "unwind"`; CLI and wasm stay `abort` |

## Ownership and lifetime

```text
   CapyLibrary ── read-only after creation ── Send + Sync ── freed LAST
        │ capy_parse
        ▼
   CapyParse  ── single owner ── frees everything below on capy_parse_free
        ├── Box<ParseResult>  never moved, never mutated
        ├── entries: Vec<Entry>   raw pointers INTO the box   (the one invariant the unsafe code relies on)
        ├── kids:    Vec<u32>     child lists, by (start,len)
        └── poisoned: AtomicBool  set by the first caught panic
   CapyStr  = borrowed view into the box; dead at capy_parse_free
   CapyError = own strings; freed by capy_error_free
```

## The id model

```text
   id (u32) = [ tag : 8 bits ][ index + 1 : 24 bits ]          0 = none
              │                └─ position in `entries` (so a parse holds at most 2^24 − 2 nodes)
              └─ per-parse tag 1..=255, from a process-wide counter; a foreign id fails the tag check
                 with high probability (not certainty: 255 tags)

   entries[i]  Block | Stmt | Cap | Expr | Err        ◄─ kind check on every access
     Block  a = statements      b = error nodes
     Stmt   a = captures        b = sections         x = body    y = closer
     Cap    a = sub statements  x = expression
     Expr   a = children (one slot per Var step: 0 for a field step)
```

Why ids, not pointers: a Rust tree has recursive records; the consumer languages that need this ABI (Ambit)
cannot hold a self-containing record, and an id can be validated where a pointer cannot.

## Flattening

`CapyParse::build` is breadth-first over `entries` — each entry is processed in index order and allocates its
children at the tail, so a node's children are contiguous in `kids` and the work needs no recursion. A 200 000-
level input cannot overflow the stack here (the parser's own bounds stop it earlier).

## Failure behaviour

```text
   accessor(p, id) ─► null handle?   ─► zero value
                    ─► poisoned?      ─► zero value
                    ─► catch_unwind {
                          id ok? tag ok? kind ok? ─► read ─► value
                          else ─► zero value
                       }  panic ─► poison handle ─► zero value
   capy_library_* / capy_parse ─► catch_unwind ─► status CAPY_E_PANIC, no handle
```

## Decisions and alternatives

`ADR-0005` and `PROP-2026-0008` §Alternatives. Rejected: C symbols in `capy-core`; JSON only; the wasm ABI;
copying strings into caller buffers.

## Last Verified Version

0.26.0 (`d1cf161` plus documentation): read against `rust/ffi/src/arena.rs`, `lib.rs`, `diag.rs`, `expr.rs`, `json.rs`.

## Known limitations

- The foreign-id tag is 8 bits.
- `unsafe` is concentrated in `arena.rs` (pointer dereference, one SAFETY note each) and at the argument edge in `lib.rs`.
- Parsing only; no `run`, `introspect`, `set_host`; no byte offsets; no spans on expressions.

## Related Documents

- `PROP-2026-0008`, `ADR-0005`, `SYS-2026-0001`, `ARCH-2026-0001`, `MAN-2026-0003`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial document |
