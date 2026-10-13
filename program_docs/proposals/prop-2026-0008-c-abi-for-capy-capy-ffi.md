---
document_id: PROP-2026-0008
title: A C ABI for Capy — a `capy-ffi` Crate That Exposes Handles, the Parse Tree, Expressions and Diagnostics
document_type: proposal
status: draft

created_date: 2026-10-13
last_updated: 2026-10-13
document_revision: 1

authors:
  - Olivier
  - Claude (Sonnet 5.5)

owner: Capy Engine
reviewers:
  - Capy Engine
  - Release Management

systems:
  - Capy

components:
  - capy-ffi
  - capy-core
  - capy-cli
  - docs

affected_versions:
  from: "0.25.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - architects
  - embedders

scope: Adds a new workspace crate `capy-ffi` (static and shared library plus a generated C header) that lets a C-ABI host compile a grammar once, parse source, and walk the resulting tree — statements, captures, typed expressions, spans, error regions and diagnostics — without Rust. It also adds an opt-in `expr` field to the AST JSON. Non-goal - no change to the grammar language, to any rendered output, to any existing public Rust type, or to `capy-core`'s single dependency; transpilation (`run`) is not exposed in this proposal.

reason: The Capylar project (the Ambit compiler being rewritten in its own language) must call Capy through a C interface. Capy has none - `capy-core` is an `rlib`, and the only non-Rust surface is a stateless wasm ABI that returns text, no tree and no diagnostics. The typed `Expr` tree a compiler needs is not even present in the AST JSON. A research record (RES-2026-0007 in the Capylar repository) lists what is missing; this proposal turns that list into a reviewable design, a decision request and a test plan.

dependencies:
  - Owner decision D1 to D7 below

related_documents:
  - PROP-2026-0001
  - PROP-2026-0007
  - STD-2026-0004
  - STD-2026-0005
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - ffi
  - c-abi
  - embedding
  - ast
  - capylar

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# A C ABI for Capy — a `capy-ffi` Crate That Exposes Handles, the Parse Tree, Expressions and Diagnostics

> **Status:** Draft
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.25.0 →
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi (new), capy-core, capy-cli, docs

## Summary

```text
   WHAT A C HOST CAN REACH TODAY                          WHAT THIS PROPOSAL ADDS

   C host                                                 C host
     │                                                      │  capy_library_new(grammar)        compile ONCE
     │  (nothing links)                                     │  capy_parse(lib, src)             handle, never tree-less
     ▼                                                      │  capy_node_* / capy_cap_* /       ids + borrowed UTF-8
   capy-core  = rlib only  ── Rust callers only             │  capy_expr_* / capy_diag_*        all of the tree
   capy-wasm-abi = wasm32 ── JSON text out, no tree,        ▼
                              grammar recompiled per call  libcapy_ffi.a / .dylib / .so + capy.h
                                                              │ thin wrapper, no logic
                                                              ▼
                                                           capy-core (unchanged, 1 dependency)
```

The Capylar compiler cannot start without this: its front end is "parse with Capy, then walk the tree",
and today there is no way to do either from a program that is not Rust. This document is a **decision
request**. It asks the owner to approve a new crate and an interface; it authorizes no code.

**Complexity: 3/5.** Difficulty is not effort: about 70 small accessors is long but easy; the three hard
parts are the panic policy, the lifetime of borrowed strings, and proving the C view equals the Rust tree.

| Factor | Rating | Why |
|---|---|---|
| Engine reuse | High | A wrapper over `Library::new`, `from_file`, `parse` and the public AST types; no engine logic is duplicated |
| New math / kernels | None | No algorithm; flattening a tree into an id table |
| Blast radius | Low | New crate; `capy-core` is untouched except an opt-in JSON field. Existing consumers see byte-identical output (R15). The risk is a **new public surface** that must then be kept stable |
| Existing quality methodology | Yes | `capy ast --json` already serializes the tree, so a C walk can be compared with it byte for byte (R13); the golden corpus and sanitizers exist |
| Unknowns | Some | Panic policy under `panic = "abort"`, cross-compilation of the static library, and whether to add byte offsets to `Span` |

Verdict: **safe to build in small steps**. The one change that touches shipped behaviour (an `expr`
field in the JSON) is opt-in, so default output cannot move.

## Decision Requested

Approve, change or refuse each item. Approval of this proposal does **not** authorize changing `rust/`,
tagging a release, publishing a crate, or changing any existing public Rust type; each of those follows
in a plan and, where it adopts a method, an ADR.

| # | Decision | Recommended |
|---|---|---|
| D1 | Put the C ABI in a **new crate `capy-ffi` inside this workspace**, beside `capy-wasm-abi` | Yes |
| D2 | Expose the tree through **opaque handles, `u32` ids and borrowed UTF-8 strings**; keep JSON as a second, optional layer | Yes |
| D3 | Panic policy: build the crate under a dedicated `ffi` cargo profile with `panic = "unwind"` and wrap every export in `catch_unwind` | Yes (alternative: keep `abort`) |
| D4 | Add an **opt-in** `expr` field to `capy ast --json` and to the C JSON dump; default output stays byte-identical | Yes |
| D5 | **Defer byte offsets** in `Span` to a separate proposal | Yes |
| D6 | Expose **parsing only** now; transpilation (`run`, `introspect`, docs rendering) is a later proposal | Yes |
| D7 | Treat the first release containing `capy-ffi` as a **minor version** with ABI version 1; the tag is decided at release time | Owner choice |

## Original User Request

| ID | What Was Asked or Said | Source and Date | Interpretation Notes |
|---|---|---|---|
| UQ-01 | "i have a project here which want to use capy ffi for a task but seems like capy does not expose certain features" | User, this conversation, 2026-10-13 | "ffi" is read as a C-ABI foreign-function interface; "certain features" is what the consuming project needs and cannot reach |
| UQ-02 | "can you write a proposal to add missing features" | User, same message | The deliverable is this proposal, in this repository's proposal format. The consuming project's own proposal (`capyler/docs/proposals/prop-2026-0001-rewrite-the-compiler-in-capylar.md`) was supplied as the statement of need |
| UQ-03 | "we need to use capy as a c ffi lib for now" | Capylar `TASK.md` line 16, quoted by PROP-2026-0001 UQ-06 | "for now" implies the FFI is a bridge until a native port exists; it must still be stable enough to build on |
| UQ-04 | "ALWAYS USE LOTS OF ASCII VISUALS IN DOCUMENTS" | This repository's `CLAUDE.md` | Applied throughout |

## Problem and Evidence

### Current User Journey

```text
  Capylar author ──► "parse Ambit source with Capy, then walk the tree"
        │
        ├─► cargo dependency?      works only for a Rust program ── the compiler is to be written in Ambit
        ├─► link a C library?      none exists; capy-core is `crate-type = ["rlib"]`
        ├─► wasm module?           exports run/introspect/docs only; no tree, no diagnostics; grammar recompiled every call
        └─► `capy ast --json`?     a process spawn per file; no typed Expr tree in the output; needs a JSON reader in a language with no maps yet
                         │
                         ▼
               blocked: the consumer reimplements the parser, or stops
```

| ID | Problem | Evidence and Inline Source | Consequence | UQ |
|---|---|---|---|---|
| P-01 | **No C-linkable artifact.** | `rust/Cargo.toml` `[lib] crate-type = ["rlib"]`, with the comment "no `extern "C"` surface"; `rust/wasm/Cargo.toml` builds a `cdylib` for **wasm32** only. `docs/embedding.md`: "no C symbols" | A C host has nothing to link | UQ-01, UQ-03 |
| P-02 | **No handle API: the grammar cannot be compiled once.** | The wasm ABI is "stateless", rebuilding the library on every `capy_run` (`rust/wasm/src/lib.rs`, `rust/gobind/binding.go`: "no object handles"). The Rust API has `Library::new` / `parse(&self)` (`rust/src/capy.rs`) but nothing C can hold | A 2,100-line grammar (the Ambit grammar) would be recompiled per file | UQ-01 |
| P-03 | **The tree and the diagnostics are unreachable outside Rust.** | `docs/ast-json.md`: "The wasm ABI exposes neither the AST nor diagnostics". `Library::parse` returns `ParseResult { tree, diagnostics }` (`capy.rs`), Rust types only | A non-Rust host cannot read what Capy parsed | UQ-01 |
| P-04 | **The typed `Expr` tree is missing from the AST JSON.** | `rust/src/domain/ast_json.rs` `capture_val` emits only `text`, `is_expr`, `span`, `sub`; `CaptureValue.expr: Option<Expr>` (`ast.rs`) is dropped | JSON is not a substitute for the Rust tree: a compiler that lowers `Binary`, `Unary`, `Var` paths and `Call` cannot be built on it | UQ-01 |
| P-05 | **Grammar-load failure detail is Rust-only.** | `CapyError { line, col, msg, hint, file, plain }` (`domain/errors.rs`) is returned by `Library::new`; no other form exists | A C host learns "failed", not where or why | UQ-01 |
| P-06 | **Panic policy contradicts the engine's own rule.** | Root `Cargo.toml` `[profile.release] panic = "abort"`; `ARCH-003` "The engine never aborts its host"; no `catch_unwind` exists in `rust/src` | Over a C boundary a panic aborts the embedding compiler; unwinding into C is undefined behaviour | UQ-01 |
| P-07 | **Positions are line/column only.** | `Span` (`ast.rs`) holds four `usize` line/col fields and is `#[non_exhaustive]` with "byte offsets are a planned addition" | A host re-slices source by line and column; workable, slower, and fragile on multi-byte text | UQ-01 |
| P-08 | **Nothing pins an ABI.** | No header, no version function, no C test anywhere in `rust/` | A consumer cannot detect a mismatched library, and an accessor change breaks callers silently | UQ-03 |

```text
  P-01…P-08 against one consumer step (the Capylar front end)

   read grammar ─► compile ─► parse file ─► check clean ─► walk stmts ─► read captures ─► lower Expr ─► report errors
        P-02          P-01       P-01,P-06     P-03          P-03          P-03             P-04         P-03,P-05
                                                                                                    (spans: P-07)   (contract: P-08)
```

## Goals and Non-Goals

| ID | Goal and Observable Outcome | Problems Solved | How It Solves Them | Success Signal |
|---|---|---|---|---|
| G-01 | A C program links a Capy static library and parses source | P-01, P-02 | `capy-ffi` crate, `staticlib` + `cdylib`, library handle | A C smoke test compiles with `zig cc`, links `libcapy_ffi.a`, and exits 0 |
| G-02 | The whole parse result is readable from C | P-03, P-04, P-05 | Id-addressed accessors for statements, captures, expressions, error regions, diagnostics and load errors | C dump equals `capy ast --json` for the corpus (R13) |
| G-03 | No input and no misuse of an id can abort or corrupt the host | P-06 | `catch_unwind`, validated ids, null checks, status codes | Fuzz and misuse tests pass under ASan and UBSan |
| G-04 | The interface is versioned and checked | P-08 | `capy_abi_version`, a generated and checked-in header, a drift test | A header that differs from the generated one fails CI |
| G-05 | Existing users see no change | all | Opt-in JSON field; new crate only | `GATE-001` and `GATE-002` pass; default `capy ast --json` byte-identical |

**Non-goals.**

- No grammar-language feature, no helper, no capture type: `CODE-002` and `CODE-003` do not apply.
- No change to `capy run`, rendering, or any rendered output (`GOAL-002`).
- No exposure of `Library::run`, `run_multi`, `introspect`, `render_library_docs` or `set_host` (D6). A C host gets the sandboxed `NoOpHost` default and nothing more.
- No byte offsets in `Span` (D5), no spans on `Expr` nodes (the consumer already stamps the capture span), no incremental parse (`PROP-2026-0006` owns that).
- No new dependency in `capy-core` (`ARCH-002`); `capy-ffi` adds none either (header generation uses an external tool, not a crate dependency).
- No promise of a stable ABI across **major** ABI versions; the promise is "additive within a version" (R11).

## Proposed User Journey

```text
  embedder ──► cargo build --profile ffi -p capy-ffi ──► libcapy_ffi.a + capy.h
       │
       ├─► CapyLibrary *lib = capy_library_new(grammar)         ──fail──► CapyError: msg, hint, line, col, file
       │
       ├─► for each source file:
       │      CapyParse *p = capy_parse(lib, src)               never NULL (except out of memory)
       │      capy_is_clean(p)?  ──no──► capy_diag_* : code, message, help, primary span, labels
       │      walk:  root block ─► statements ─► captures ─► text / span / sub-statements / Expr tree
       │      capy_parse_free(p)                                 every borrowed string dies here
       │
       └─► capy_library_free(lib)
```

## Requirements

Sources are outside this proposal (the consuming project's requirement list, source files, standards). R16
is satisfied by documents, not code.

| ID | Requirement | Type | Source and Relevance | Acceptance Criteria | Goal |
|---|---|---|---|---|---|
| R1 | A new workspace member `capy-ffi` builds `staticlib` and `cdylib`, is `publish = false`, depends only on `capy-core`, and adds no dependency to `capy-core` | architecture | Capylar PROP-2026-0001 R8; `ARCH-002`; `rust/wasm/Cargo.toml` (same shape, "a normal Rust dependency inherits no C symbols") | `cargo tree -p capy-core --depth 1` still shows exactly one dependency; `cargo tree -p capy-ffi --depth 1` shows `capy-core` only | G-01 |
| R2 | Library handle: create from grammar text and from a file path; free; compile once, reuse for many parses; a library handle may be shared across threads read-only | functional | `Library: Send + Sync` pinned by `library_is_send_and_sync` (`capy.rs`); Capylar RES-2026-0007 s2 (grammar recompiled per parse today) | One library, ≥ 2 threads each parsing, no data race under TSan; second `capy_parse` does not recompile (T-04) | G-01 |
| R3 | Parse handle: `capy_parse` always yields a handle for any UTF-8 input (even invalid text yields diagnostics); one free releases everything borrowed from it | functional | `Library::parse`: "The tree is ALWAYS returned" (`capy.rs`); `ARCH-003` | Table of hostile inputs returns a handle and a diagnostic, never a crash (T-05) | G-01, G-03 |
| R4 | Accessors cover every public field of `Block`, `FuncCall`, `CaptureValue`, `ErrorNode`: function name, line/col, span, leading comments, body, closer, sections by keyword and by index, captures by name **and by enumeration**, capture text / `is_expr` / `sub` / span, verbatim text, error regions with skipped tokens | functional | `rust/src/domain/ast.rs`; the consumer enumerates captures and reads `.sub` 35 times and `.span` 295 times in one pass (Capylar RES-2026-0007 s2) | A C walker rebuilds the same tree as `capy ast --json` for the corpus (R13) | G-02 |
| R5 | Expression accessors cover every `Expr` variant (`Number`, `Str`, `Bool`, `Null`, `Var`, `Call`, `Compare`, `Binary`, `Not`, `Unary`, `List`, `Obj`) including integer-vs-float numbers and `Var` path steps (field or index) | functional | `ast.rs` `enum Expr`; `Unary` added in 0.25.0; the consumer lowers eleven of them (Capylar RES-2026-0007 s2) | Property test: render each `Expr` from C and from Rust, compare (T-03) | G-02 |
| R6 | Diagnostic accessors expose `code`, `severity`, `msg`, `help`, `primary`, labels (span + text) and context frames (`shape`, `arg_index`, `arg_name`) | functional | `Diagnostic`, `Label`, `ContextFrame` in `domain/errors.rs`; consumer uses `.primary .msg .code .severity .labels .help .context` | C diagnostics equal the `diagnostics` array of `capy ast --json` (T-02) | G-02 |
| R7 | A failed grammar load returns an error object carrying `msg`, `hint`, `line`, `col`, `file`; no global or thread-local "last error" | functional | `CapyError` fields (`errors.rs`); thread-local state is hostile to the consumer's possible threads | Broken grammar returns the same five fields the CLI prints (T-06) | G-02 |
| R8 | Every handle and id is validated: null, wrong kind, out of range, or foreign-parse ids return a status code or a zero value, never undefined behaviour | security | OWASP input validation (repository security stance, `SECURITY.md`); `ARCH-003` | Misuse suite of ≥ 40 cases (T-07) passes under ASan and UBSan | G-03 |
| R9 | No Rust panic crosses the boundary. A panic is caught, returned as `CAPY_E_PANIC`, and the affected handle is poisoned (further calls return an error) | security | `ARCH-003` "The engine never aborts its host"; P-06 | Test forces a panic through a test-only hook and observes the status, not an abort (T-08) | G-03 |
| R10 | Enumeration order is deterministic and equals the Rust order (`BTreeMap` order for captures and sections; source order for statements) | compatibility | `FuncCall.captures: BTreeMap`; Capylar needs byte-equal differential dumps | Two runs, same output; order equals `ast --json` (T-02) | G-02 |
| R11 | `capy_abi_version()` returns an integer; the header carries the same constant; within one version, changes are additive only (new functions, never changed signatures) | compatibility | P-08; `docs/ast-json.md` "Stability" sets the precedent for additive-only | A test compares the checked-in header with a freshly generated one and fails on drift (T-09) | G-04 |
| R12 | `capy ast --json --with-expr` and `capy_parse_to_json(p, flags)` add an `expr` field per capture; without the flag the output is byte-identical to today | compatibility | P-04; `GOAL-002`; `ARCH-001` (new information goes in a new field) | Golden diff for the whole corpus is empty without the flag (T-10) | G-02, G-05 |
| R13 | **Differential equivalence:** for every `samples/*/lib.capy` with its scripts, and the Ambit grammar when available, a C-side dump of the tree equals the Rust-side `ast --json --with-expr` byte for byte | quality | `QUAL-001` "regression is demonstrated across the whole corpus" | 0 differences across the corpus (T-02) | G-02 |
| R14 | Memory safety is verified: C smoke and misuse tests run under ASan and UBSan; Rust unsafe blocks are covered by tests and a stated invariant per block | quality | `SECURITY.md`; `QUAL-003` | Sanitizer runs recorded in a `TEST` document | G-03 |
| R15 | Existing behaviour is unchanged: `GATE-001` and `GATE-002` pass; the wasm module size does not change | compatibility | `GOAL-002`; `QUAL-001`; `GATE-002` runs `wasm_check.sh` | Gates green; wasm byte size equal before and after (`capy-ffi` is not in the wasm build) | G-05 |
| R16 | The interface is documented: a new `docs/c-abi.md` page in the mkdocs nav, a C section in `docs/embedding.md`, `docs/whats-new.md`, `CHANGELOG.md` | documentation | `AGENTS.md` "Repo conventions"; `GATE-001` runs `mkdocs build --strict` | `mkdocs build --strict` exits 0; every exported function appears in the page | G-04 |
| R17 | Size and cost are predeclared and measured: static library size, arena build overhead relative to `Library::parse`, per-accessor call cost | measurement | `QUAL-002`; Capylar RES-2026-0007 s4 measured 5.15 MB for `capy-core` alone | Predictions below are frozen at approval; actuals go in `TEST-2026-0015` | G-01, G-05 |

## Use Cases

### Use-Case Catalogue

| ID | User Outcome | Actor | Surface and Trigger | Preconditions / Environment | Inputs | Outputs / Visible Result | Negative and Error Paths | Goal IDs | Requirement IDs | Test IDs |
|---|---|---|---|---|---|---|---|---|---|---|
| UC-01 | Compile a grammar once and keep it | C host | `capy_library_new(grammar, len, &lib, &err)` | `libcapy_ffi.a` linked | grammar text or a path | a `CapyLibrary *` | bad grammar → status ≠ 0 and a `CapyError *` with msg, hint, line, col, file | G-01, G-02 | R2, R7 | T-04, T-06 |
| UC-02 | Parse a file and learn whether it was clean | C host | `capy_parse(lib, src, len)` then `capy_is_clean` | UC-01 done | UTF-8 source | a `CapyParse *`; clean flag; diagnostic count | hostile input (empty, deeply nested, binary) → handle plus diagnostics, never a crash | G-01, G-03 | R3, R8, R9 | T-05, T-07, T-08 |
| UC-03 | Walk statements and read captures | C host | `capy_block_*`, `capy_node_*`, `capy_cap_*` | UC-02 done | ids | names, text, spans, sub-statements, body, closer, sections | stale or foreign id → zero or error status | G-02 | R4, R10 | T-01, T-02 |
| UC-04 | Read the expression behind an `expr` capture | C host | `capy_cap_expr`, `capy_expr_*` | UC-03 done | an expression id | kind, literal value, operator text, children, path steps | asking a child of a leaf → zero | G-02 | R5 | T-03 |
| UC-05 | Report Capy's diagnostics unchanged | C host | `capy_diag_*` | UC-02 done | diagnostic index | code, severity, message, help, spans, labels, context | index out of range → empty string and status | G-02 | R6 | T-02 |
| UC-06 | Dump the whole parse as JSON for tools and tests | tool author / CI | CLI: `capy ast --json --with-expr`; C: `capy_parse_to_json` | UC-02 done | a flag word | JSON text | unknown flag bit → status `CAPY_E_ARG` | G-02, G-05 | R12, R13 | T-02, T-10 |
| UC-07 | Detect a mismatched library at start-up | C host | `capy_abi_version()` against the header constant | none | none | an integer | mismatch → the host refuses to continue | G-04 | R11 | T-09 |

### UC-02 — Parse a file and learn whether it was clean

Normal journey: the host passes pointer and length (never a NUL-terminated string), receives a parse
handle, tests `capy_is_clean`, and walks. Error branches: invalid UTF-8 is converted lossily (the rule the wasm ABI
already uses, `from_utf8_lossy`) and the parse proceeds on the converted text; an internal panic returns
`NULL` and `CAPY_E_PANIC` through the status out-parameter described next.

Design note: because `capy_parse` returns a pointer, a panic cannot be reported through the return value
alone. The signature therefore carries an out-status: `CapyParse *capy_parse(const CapyLibrary *, const uint8_t *src, size_t n, int32_t *status)`.
A non-NULL result with `*status == CAPY_OK` is the only success.

### UC-03 — Walk statements and read captures

```text
  capy_root(p) ──► block id
        │ capy_block_len(p, b) = n
        ▼
  capy_block_stmt(p, b, i) ──► node id ──► capy_node_func(p, n) = "import_stmt"
        │                                  capy_node_span(p, n, &span)
        │                                  capy_node_capture_count(p, n) = k
        │                                  capy_node_capture_name(p, n, j)   ◄── enumeration, sorted
        │                                  capy_node_capture(p, n, "name") ──► capture id (0 = absent)
        ▼
  capture id ──► capy_cap_text   capy_cap_span   capy_cap_sub_count / capy_cap_sub (statements)
              └► capy_cap_expr ──► expression id ──► UC-04
```

## Sample API Calls and CLI Commands

A user-facing surface changes (a new C API and a new CLI flag), so copy-pasteable samples are required.

### C: compile once, parse, walk, report

```c
#include "capy.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    if (capy_abi_version() != CAPY_ABI_VERSION) { fputs("capy: ABI mismatch\n", stderr); return 2; }

    const char *grammar = "extension txt\nfunction greet\n  arg literal \"greet\"\n  arg capture who ident\n  write `hi ${who}`\nend\n";
    CapyLibrary *lib = NULL; CapyError *err = NULL;
    if (capy_library_new((const uint8_t *)grammar, strlen(grammar), &lib, &err) != CAPY_OK) {
        CapyStr m = capy_error_msg(err), h = capy_error_hint(err);
        fprintf(stderr, "grammar:%u:%u: %.*s\n  hint: %.*s\n", capy_error_line(err), capy_error_col(err),
                (int)m.len, m.ptr, (int)h.len, h.ptr);
        capy_error_free(err);
        return 1;
    }

    const char *src = "greet world\n";
    int32_t st = 0;
    CapyParse *p = capy_parse(lib, (const uint8_t *)src, strlen(src), &st);
    if (st != CAPY_OK) { capy_library_free(lib); return 1; }

    if (!capy_is_clean(p)) {
        for (uint32_t i = 0; i < capy_diag_count(p); i++) {
            CapyStr c = capy_diag_code(p, i), m = capy_diag_message(p, i);
            CapySpan sp; capy_diag_primary(p, i, &sp);
            printf("%.*s at %u:%u: %.*s\n", (int)c.len, c.ptr, sp.start_line, sp.start_col, (int)m.len, m.ptr);
        }
    } else {
        CapyId root = capy_root(p);
        for (uint32_t i = 0; i < capy_block_len(p, root); i++) {
            CapyId n = capy_block_stmt(p, root, i);
            CapyStr f = capy_node_func(p, n);
            CapyId who = capy_node_capture(p, n, (const uint8_t *)"who", 3);
            CapyStr t = capy_cap_text(p, who);
            printf("%.*s(who=%.*s)\n", (int)f.len, f.ptr, (int)t.len, t.ptr);   /* greet(who=world) */
        }
    }
    capy_parse_free(p);          /* f.ptr and t.ptr are dead from here */
    capy_library_free(lib);
    return 0;
}
```

### Build and run (commands)

```sh
# build the library under the ffi profile (panic = "unwind", D3) and generate the header
cargo build --manifest-path rust/Cargo.toml --profile ffi -p capy-ffi
cbindgen --config rust/ffi/cbindgen.toml --crate capy-ffi --output rust/ffi/include/capy.h

# link a C program with zig cc (the toolchain the consumer already uses)
zig cc examples/c/walk.c -Irust/ffi/include rust/target/ffi/libcapy_ffi.a -lc -lm -o walk
./walk                      # prints: greet(who=world)

# the Rust-side oracle for the same file, including the typed expressions (D4)
cargo run --manifest-path rust/Cargo.toml -p capy-cli --bin capy -- ast lib.capy script.capy --json --with-expr
```

### What a misuse returns (the declined path)

```text
  capy_node_func(p, 999999)            ──► CapyStr{ptr=NULL, len=0}      (id out of range)
  capy_cap_text(p, <a node id>)        ──► CapyStr{ptr=NULL, len=0}      (id has the wrong kind)
  capy_node_func(p_other, n)           ──► CapyStr{ptr=NULL, len=0}      (id from a different parse)
  capy_parse(NULL, src, n, &st)        ──► NULL, st = CAPY_E_NULL
  capy_parse_to_json(p, 0x8000, ...)   ──► CAPY_E_ARG                    (unknown flag bit)
  any call after a caught panic        ──► CAPY_E_PANIC                  (handle poisoned)
```

JSON sample, the new opt-in field only (everything else is the schema in `docs/ast-json.md`):

```json
{ "text": "a + 2", "is_expr": true, "span": {"start_line":3,"start_col":9,"end_line":3,"end_col":14},
  "sub": [],
  "expr": { "kind": "binary", "op": "+",
            "left":  { "kind": "var",    "path": [ {"field": "a"} ] },
            "right": { "kind": "number", "is_int": true, "int": 2 } } }
```

## Project Standards Baseline

| Standards Index | Revision | Validated At |
|---|---|---|
| [`program_docs/standards/index.md`](../standards/index.md) | 1 | 2026-10-13 |

## Project Validation

| Rule | Applicability | Proposal Evidence | Initial Result | Exception or Follow-Up |
|---|---|---|---|---|
| GOAL-001 | Applies | No grammar and no source language shipped; the ABI is generic over any `.capy` library | PASS | none |
| GOAL-002 | Applies | R12 is opt-in, R15 gates; new crate only | PASS | JSON golden diff (T-10) is the proof |
| PHIL-001 | Applies | Every problem row cites a file read on 2026-10-13; costs are labelled predictions | PASS | none |
| PHIL-002 | Applies | The tension between `ARCH-003` and `panic = "abort"` is stated (P-06), not smoothed over | PASS | decided in D3 |
| CODE-001 | Applies | No commit, no `.ignore/` file staged | PASS | none |
| CODE-002 / CODE-003 | Applies | No helper, directive or capture type is added | NOT APPLICABLE | none |
| ARCH-001 | Applies | New information goes in new accessors and a new JSON field; no existing field changes type | PASS | none |
| ARCH-002 | Applies | R1: `capy-core` keeps `regex` only; `capy-ffi` has no dependency beyond `capy-core` | PASS | `cbindgen` is a tool, not a dependency |
| ARCH-003 | Applies | R9 and D3: panics are caught at the boundary; recursion depth is already bounded (`MAX_PARSE_DEPTH = 64`) and the tree walk in `capy_parse_to_json` must be iterative or depth-bounded | PASS | add a depth case to `recursion_guard` (T-11) |
| QUAL-001 | Applies | R13, R15 | PASS | none |
| QUAL-002 | Applies | R17 and §Frozen Predictions | PASS | freezes at approval |
| QUAL-003 | Applies | Each test names the defect it detects (§Test and Validation Design) | PASS | none |
| GATE-001 / GATE-002 | Applies | R15 | PASS | `capy-ffi` joins `cargo build/clippy/test --workspace` |

## Implementation Design

### Environment and Feasibility

| Environment / Version | Required Capability | Feasibility Evidence and Source | Constraint or Unknown | Resolution |
|---|---|---|---|---|
| Rust 1.74+ (workspace `rust-version`), edition 2021 | Custom cargo profile (`inherits`), `catch_unwind`, `staticlib` | Custom profiles stable since Rust 1.57; `catch_unwind` is std | `panic` cannot be set per member inside a workspace; the setting lives in a profile | D3: a dedicated `[profile.ffi]` at the workspace root, `inherits = "release"`, `panic = "unwind"` |
| macOS arm64, rustc 1.90 | Build `capy-core` as a static library | Measured by the consumer: 9.4 s cold, 5,154,280 bytes, no exports (Capylar RES-2026-0007 s4) | Linux and cross targets unverified | T-12 builds on macOS arm64 and Linux x86_64; cross targets recorded as unverified until run |
| `cbindgen` | Generate `capy.h` | Installed on the consumer's machine; used in Ambit's bridge docs | A tool, not a dependency | Header is checked in; CI compares (R11) |
| `zig cc` | Compile and link the C smoke tests with sanitizers | The consumer drives it already | Sanitizer support in the installed zig is unverified | T-07 falls back to `clang -fsanitize=address,undefined` |
| Ambit grammar `ambit.capy` | Large real corpus for R13 | 210 functions, 2,117 lines (Capylar RES-2026-0007 s0) | Lives in another repository | Corpus = `samples/*` always; Ambit grammar added when its path is configured |

### Methods by Use Case

| Change ID | UC IDs | Method and Execution Order | Positive Path | Negative / Failure Path | Requirement Fulfilment | Feasibility |
|---|---|---|---|---|---|---|
| C-01 | UC-01 | Workspace member `capy-ffi`; `[profile.ffi]`; `lib.rs` with library and error exports | library handle works | grammar error returns `CapyError` | R1, R2, R7 | proven (wasm crate is the same shape) |
| C-02 | UC-02 | `capy_parse` builds a flat arena of entries over a boxed `ParseResult` (§Arena); status out-param | handle returned | panic → `CAPY_E_PANIC`, handle poisoned | R3, R9 | experiment needed (lifetime invariant) |
| C-03 | UC-03 | Block, node, capture accessors; enumeration by index | tree readable | invalid id → zero value | R4, R8, R10 | proven (plain reads) |
| C-04 | UC-04 | Expression accessors; `Var` path steps; `Number` int/float split | all variants readable | wrong-kind id → zero | R5, R8 | proven |
| C-05 | UC-05 | Diagnostic and error-node accessors | diagnostics readable | index out of range → empty | R6, R8 | proven |
| C-06 | UC-06 | `ast_json.rs` gains an `expr` writer behind a flag; `capy ast --with-expr`; `capy_parse_to_json` | JSON with expressions | unknown flag → error | R12 | proven (writer exists) |
| C-07 | UC-07 | `capy_abi_version`, cbindgen config, header drift test | mismatch detectable | drift fails CI | R11 | proven |
| C-08 | UC-02, UC-03 | C smoke and misuse tests; sanitizer runs; differential harness | corpus equal | any diff names the file | R13, R14 | experiment needed |
| C-09 | all | Docs, changelog, whats-new, mkdocs nav | `mkdocs build --strict` passes | broken link fails | R16 | proven |
| C-10 | all | Measurement of size and overhead | numbers recorded | outside prediction → investigate | R17 | experiment needed |

### Added, Changed and Removed Contracts

| Item | CRUD | Kind | Name / Key / Route / Event | Type, Default or Schema | Scope / Lifetime | Consumers | Requirements |
|---|---|---|---|---|---|---|---|
| Static and shared library | CREATE | artifact | `libcapy_ffi.a`, `libcapy_ffi.{dylib,so}` | C ABI, version 1 | process | C hosts | R1 |
| Header | CREATE | file | `rust/ffi/include/capy.h` | generated, checked in | repo | C hosts | R11 |
| Library handle | CREATE | resource | `CapyLibrary` | opaque, read-only after creation, shareable across threads | until `capy_library_free` | C hosts | R2 |
| Parse handle | CREATE | resource | `CapyParse` | opaque, single owner | until `capy_parse_free` | C hosts | R3 |
| Error object | CREATE | resource | `CapyError` | opaque; msg, hint, line, col, file | until `capy_error_free` | C hosts | R7 |
| Ids | CREATE | type | `CapyId` | `uint32_t`, 0 = none, valid for one parse | one parse | C hosts | R4, R8 |
| Strings | CREATE | type | `CapyStr` | `{const uint8_t *ptr; size_t len}`, UTF-8, not NUL-terminated, borrowed | until the owning handle is freed | C hosts | R4 |
| Spans | CREATE | type | `CapySpan` | four `uint32_t`, 1-indexed, `end_col` exclusive, `start_line == 0` means unset | by value | C hosts | R4 |
| Status codes | CREATE | enum | `CAPY_OK`, `CAPY_E_NULL`, `CAPY_E_ARG`, `CAPY_E_KIND`, `CAPY_E_UTF8`, `CAPY_E_PANIC`, `CAPY_E_OOM` | `int32_t` | call | C hosts | R8, R9 |
| ABI version | CREATE | function | `capy_abi_version` | `uint32_t`, 1 | process | C hosts | R11 |
| AST JSON `expr` field | CREATE (opt-in) | schema | `capy ast --json --with-expr` | object per expression capture | per run | tools, tests | R12 |
| `[profile.ffi]` | CREATE | build | workspace `Cargo.toml` | `inherits = "release"`, `panic = "unwind"` | build | maintainers | R9 |
| `capy-core` public API | none | — | — | unchanged | — | — | R1, R15 |
| Default `capy ast --json` | none | schema | — | byte-identical | — | — | R12, R15 |

### Architecture, Data, State and Interaction Visuals

```text
  rust/  (workspace)
  ├── Cargo.toml            capy-core (rlib, 1 dependency)         ◄── UNCHANGED, plus [profile.ffi]
  ├── src/                  engine                                  ◄── UNCHANGED (ast_json.rs gains an opt-in writer)
  ├── cli/                  capy                                    ◄── `ast --with-expr` flag
  ├── wasm/                 capy-wasm-abi (cdylib, wasm32)          ◄── UNCHANGED
  └── ffi/                  capy-ffi            NEW
      ├── Cargo.toml        crate-type = ["staticlib", "cdylib"], publish = false
      ├── cbindgen.toml
      ├── include/capy.h    generated, checked in, drift-tested
      ├── src/lib.rs        exports + catch_unwind shim
      ├── src/arena.rs      flatten ParseResult → id table
      ├── src/expr.rs  src/diag.rs  src/json.rs
      └── tests/            Rust-side tests; ../../examples/c/ holds the C smoke tests
```

#### Arena: how ids and borrowed strings stay valid

```text
  capy_parse(lib, src)
        │ Library::parse(&str) ──► ParseResult  (owned)
        ▼
  Box<ParseResult>  ── never moved, never mutated after this point
        │
        │ one pass over the tree, depth-bounded
        ▼
  entries: Vec<Entry>          Entry = Block(*const Block) | Stmt(*const FuncCall) | Capture(*const CaptureValue)
        │                            | Expr(*const Expr) | ErrorNode(*const ErrorNode) | Diag(*const Diagnostic)
        │  id = index + 1 (0 = none)           ▲
        ▼                                      │ raw pointers into the Box above; sound only because the Box is
  CapyParse { parse_id, result: Box<..>, entries, poisoned }     pinned for the life of CapyParse (documented invariant,
                                                                 one SAFETY comment per dereference)
  accessor(p, id):  check p non-null ─► check id in range ─► check entry kind ─► read ─► return CapyStr{ptr,len} into `result`
```

#### Ownership and lifetime

```text
   CapyLibrary ──(read-only, Send+Sync)──► may be used by many threads, freed last
        │ parse
        ▼
   CapyParse  ◄── single owner; not thread-safe for concurrent free (documented)
        │ lends
        ├──► CapyStr  (borrowed; dies at capy_parse_free)
        ├──► CapyId   (valid only for this CapyParse; foreign ids are detected by an id tag)
        └──► CapySpan (by value, never dangles)
```

A `CapyId` carries a small per-parse tag in its high bits so an id from another parse is rejected rather than
read (R8). Final bit split is a detail for the plan; the requirement is "foreign and stale ids are detected".

#### Panic flow (D3)

```text
  exported fn ─► catch_unwind( closure ) ─┬─ Ok(v)  ─► return v
                                          └─ Err(_) ─► mark handle poisoned ─► return CAPY_E_PANIC / zero value
  profile.ffi: panic = "unwind"   (profile.release keeps "abort": capy-cli and wasm are unaffected)
```

### Interface Appendix (illustrative; the generated header is the authority)

```text
  version / status     capy_abi_version
  library              capy_library_new   capy_library_from_file   capy_library_free
  load error           capy_error_msg  _hint  _line  _col  _file   capy_error_free
  parse                capy_parse   capy_parse_free   capy_is_clean   capy_root
  block                capy_block_len   capy_block_stmt   capy_block_verbatim
                       capy_block_error_count   capy_block_error   (error node: span, diag index, token count/token)
  statement            capy_node_func   capy_node_span   capy_node_line  capy_node_col
                       capy_node_capture_count  capy_node_capture_name  capy_node_capture
                       capy_node_body   capy_node_closer
                       capy_node_section_count  capy_node_section_name  capy_node_section
                       capy_node_comment_count  capy_node_comment_span
  capture              capy_cap_text   capy_cap_span   capy_cap_is_expr   capy_cap_expr
                       capy_cap_sub_count   capy_cap_sub
  expression           capy_expr_kind   capy_expr_int   capy_expr_float   capy_expr_bool   capy_expr_str
                       capy_expr_child_count   capy_expr_child
                       capy_expr_name_count  capy_expr_name        (Call: dotted name segments)
                       capy_var_step_count   capy_var_step_is_index   capy_var_step_field   capy_var_step_index
                       capy_obj_key          (Obj: key i; values via capy_expr_child)
  diagnostics          capy_diag_count   capy_diag_code   capy_diag_severity   capy_diag_message   capy_diag_help
                       capy_diag_primary   capy_diag_label_count   capy_diag_label
                       capy_diag_context_count   capy_diag_context
  json                 capy_parse_to_json   capy_buf_free
```

Roughly 70 functions. Expression child order: `Binary` / `Compare` = left, right; `Unary` / `Not` = operand;
`List` = elements; `Call` = arguments; `Obj` = values in key order.

## Expected Code and Documentation Changes

| ID | Path | CRUD | Symbol / Region | Exact Planned Edit | Reason | Change IDs | Requirement IDs | Dependencies | Test / Documentation Impact |
|---|---|---|---|---|---|---|---|---|---|
| F-01 | `rust/Cargo.toml` | UPDATE | `[workspace] members`, new `[profile.ffi]` | add `"ffi"`; add profile `inherits = "release"`, `panic = "unwind"` | D1, D3 | C-01 | R1, R9 | D1, D3 | `GATE-001` |
| F-02 | `rust/ffi/Cargo.toml` | CREATE | package `capy-ffi` | `crate-type = ["staticlib","cdylib"]`, `publish = false`, one dependency | new crate | C-01 | R1 | F-01 | T-01 |
| F-03 | `rust/ffi/src/lib.rs`, `arena.rs`, `expr.rs`, `diag.rs`, `json.rs` | CREATE | exports | implement the Interface Appendix with the shim | the ABI | C-01..C-05 | R2-R10 | F-02 | T-01..T-08 |
| F-04 | `rust/src/domain/ast_json.rs` | UPDATE | `capture_val` | add `expr_val` and an `include_expr` parameter; default path unchanged | D4 | C-06 | R12 | none | T-10 |
| F-05 | `rust/cli/` (the `ast` command) | UPDATE | `--json` handling | accept `--with-expr` | D4 | C-06 | R12 | F-04 | T-10, `docs/cli.md` |
| F-06 | `rust/ffi/cbindgen.toml`, `rust/ffi/include/capy.h` | CREATE | header | generated and checked in | version pin | C-07 | R11 | F-03 | T-09 |
| F-07 | `examples/c/walk.c`, `examples/c/misuse.c`, `examples/c/run.sh` | CREATE | C smoke, misuse, sanitizer script | the samples above plus ≥ 40 misuse cases | verification | C-08 | R13, R14 | F-03, F-06 | T-02, T-07 |
| F-08 | `rust/tests/` and `rust/ffi/tests/` | CREATE | differential and panic tests | C dump equals `ast --json --with-expr` over `samples/*` | R13 | C-08 | R13 | F-04, F-07 | T-02, T-08 |
| F-09 | `rust/tests/recursion_guard.rs` | UPDATE | add depth case for the JSON dump | `ARCH-003` | C-06 | R9 | F-03 | T-11 |
| F-10 | `rust/devtools/` | UPDATE | `check_publishable.py` | assert `capy-core` still has one dependency; assert `capy-ffi` is not publishable | `ARCH-002` | C-01 | R1 | F-02 | `GATE-001` |
| F-11 | `docs/c-abi.md`, `docs/embedding.md`, `docs/whats-new.md`, `docs/ast-json.md`, `mkdocs.yml`, `CHANGELOG.md` | CREATE / UPDATE | new page and nav entry; C section; `expr` field documented | R16 | C-09 | R16 | F-03 | `mkdocs build --strict` |
| F-12 | `program_docs/testing/test-2026-0015-*.md`, `program_docs/index/*` | CREATE / UPDATE | measurement and index rows | R17 | C-10 | R17 | F-03 | link check |

For F-04 the shape of the `expr` object is the JSON sample above; `kind` values are `number`, `str`, `bool`,
`null`, `var`, `call`, `compare`, `binary`, `not`, `unary`, `list`, `obj`. All code shown in this document is illustrative.

## Alternatives Considered

| Alternative | Advantages | Disadvantages | Why Selected or Rejected | Requirement Impact |
|---|---|---|---|---|
| A New `capy-ffi` crate with id accessors (D1, D2) | No change to `capy-core`; zero-copy reads; one free per handle | About 70 functions to design, test and keep stable | **Selected** | R1-R11 |
| B Make `capy-core` itself `staticlib` with `#[no_mangle]` exports | One crate | Every Rust dependant inherits C symbols — the reason `capy-wasm-abi` is separate (`rust/Cargo.toml` comment); breaks `GOAL-002` in spirit | Rejected | R1 |
| C JSON only: `capy_parse_json(lib, src)` returns text | Cheapest: reuses `ast_json.rs`; one function | The host needs a JSON reader (the consumer has no maps or growable lists yet); a tree rebuilt from text per file; no handle unless added anyway | Rejected as the only path; **kept as the optional second layer** (R12) | R12 |
| D Extend the wasm ABI to return the tree and run it in a wasm runtime from C | Reuses the existing module | Needs a wasm runtime inside every host; linear-memory marshalling for 295 span reads per pass; 1.3 MB module; still no `Expr` | Rejected | R4, R5 |
| E Spawn `capy ast --json` per file | No build work | A process per file; no `Expr`; JSON reader; startup cost per file | Rejected | R4, R5 |
| F Hand-written parser in the consumer | No FFI | Abandons the single-source-of-syntax design (the grammar is the contract) | Out of scope here; the consumer's own fallback | — |
| G Keep `panic = "abort"` for the FFI build (D3 alternative) | No profile, no `catch_unwind`, smaller code | A Capy bug kills the embedding compiler; contradicts `ARCH-003` | Rejected, but acceptable if the owner prefers it (D3) | R9 |
| H Copy strings out on every call (`bytes_out` style) | No lifetime rule for the host | A copy and a caller buffer per accessor; the consumer then needs a growable text it does not have yet | Rejected; borrowed views tied to the handle are simpler and fit the consumer's handle-lifetime checks | R4 |

## Risks and Rollback

| Risk | Trigger / Detection | Impact | Mitigation | Rollback Action | Owner |
|---|---|---|---|---|---|
| A new public surface Capy must keep stable | A consumer pins ABI 1 and a later change wants to alter a signature | Churn, broken consumers | Additive-only rule (R11); header drift test; ABI bump is a deliberate act | Do not publish a tag; delete the crate (nothing else depends on it) | Capy Engine |
| Raw pointers in the arena dangle | Box moved or mutated; accessor after free | Memory-unsafe read | Box is pinned for the handle's life; one SAFETY comment per dereference; ASan/UBSan runs; foreign-id tags | Switch the arena to copied strings (safe, slower) | Capy Engine |
| A panic inside `catch_unwind` leaves state inconsistent | Forced-panic test | Wrong results after a caught panic | Handle poisoned on first panic; every later call returns `CAPY_E_PANIC` | Abort policy (alternative G) | Capy Engine |
| `[profile.ffi]` diverges from `[profile.release]` and hides a bug | Size or behaviour differs | A bug seen only in one profile | The profile only changes `panic`; the full test suite also runs under it | Remove the profile, use alternative G | Capy Engine |
| Static library is large | Measured above the prediction | Bigger consumer binaries | `opt-level = "z"`, LTO inherited; regex is the bulk | Accept; document | Capy Engine |
| JSON `expr` field diverges from the C tree | T-02 differs | False confidence in the oracle | Both are produced from the same `Expr` walk helper where possible; the differential test compares them | Fix the later of the two | Capy Engine |
| Cross-compilation of the static library fails | Linux musl or other triples | Consumer cannot target them | Verified targets are listed; unverified ones are marked so | Document as unsupported until tested | Capy Engine |
| Scope grows into "expose everything" | Requests for `run`, `set_host`, callbacks | Larger unsafe surface | D6 defers them to their own proposal; `NoOpHost` stays the only host | Refuse in review | Capy Engine |
| Recursive tree dump overflows the stack | Deeply nested input | Process abort | `MAX_PARSE_DEPTH = 64` already bounds the tree; the dump is depth-checked (T-11) | Make the dump iterative | Capy Engine |

## Security Impact

This adds the first `extern "C"` surface that accepts raw pointers from foreign code. The controls are:
pointer-plus-length for every string (never NUL-terminated input, so no unbounded read); every id and
handle validated before use (R8); no panic crossing the boundary (R9); no global mutable state added (the
regex cache already bounded at 64 entries stays internal); the sandboxed `NoOpHost` is the only host, so a
grammar cannot read the environment or files through this API (`from_file` reads only the grammar the
caller names). Remaining unsafe code is limited to dereferencing arena pointers and slices built from
caller pointers; each carries a stated invariant and is exercised under ASan and UBSan (R14). Passing a
freed handle, or a pointer and length that do not describe readable memory, is a caller contract
violation the library cannot detect; the header documents it. No secrets are handled.

## Operational Impact

None for existing users. Maintainers gain one workspace member that `cargo build --workspace` compiles,
one extra profile, and a C test script. CI gains a sanitizer job and a header drift check.

## Compatibility Impact

Rust API: unchanged. CLI: one new optional flag. `capy ast --json` default output: byte-identical.
wasm module: unchanged in size (R15). ABI: new, version 1. The `Span` type is not touched (D5).

## Migration Requirements

None. No existing user has anything to migrate. The first consumer is the Capylar project, which would
pin the release that contains the crate and check `capy_abi_version()` at start-up.

## Test and Validation Design

| ID | Type | UC / Requirement IDs | Scenario and Purpose | Environment / Data | Exact Procedure or Command | Expected Result | Test File / Evidence Destination |
|---|---|---|---|---|---|---|---|
| T-01 | integration | UC-01..UC-03, R1-R4 | C smoke: compile, parse, walk the sample above | macOS arm64; `zig cc` | `examples/c/run.sh walk` | prints `greet(who=world)`, exit 0 | `examples/c/`, `TEST-2026-0015` |
| T-02 | regression | UC-03, UC-05, UC-06, R6, R10, R13 | Differential: C dump equals Rust JSON with expressions | every `samples/*/lib.capy` and its scripts; Ambit grammar if configured | `rust/ffi/tests/differential.rs` | 0 differences, or each reviewed and recorded | `TEST-2026-0015` |
| T-03 | property | UC-04, R5 | Every `Expr` variant read from C equals the Rust value | generated expressions across all 12 kinds, ints at ±2^63 edges, floats, nested paths | property run, seeded | equal | `rust/ffi/tests/expr.rs` |
| T-04 | integration | UC-01, R2 | One library, many parses, two threads | 2 threads × 1,000 parses | TSan build of the test | no race; grammar compiled once (counter) | `rust/ffi/tests/threads.rs` |
| T-05 | failure | UC-02, R3 | Hostile inputs return a handle plus diagnostics | empty, 1 MB line, 10,000 nested openers, NUL bytes, invalid UTF-8 | `examples/c/run.sh hostile` | no crash, diagnostics present | `TEST-2026-0015` |
| T-06 | failure | UC-01, R7 | A broken grammar reports msg, hint, line, col, file | a grammar with a known error at line 3 | C test compares with the CLI's message | the five fields equal | `examples/c/` |
| T-07 | failure | UC-02, UC-03, R8, R14 | ≥ 40 misuse cases: null, wrong kind, out of range, foreign id, double free of an error object, unknown flag | ASan + UBSan | `examples/c/run.sh misuse` | status codes or zero values; sanitizers silent | `TEST-2026-0015` |
| T-08 | failure | R9 | A forced panic returns `CAPY_E_PANIC` and poisons the handle | a `#[cfg(test)]` hook inside `capy-ffi` | `cargo test --profile ffi -p capy-ffi` | status returned, process alive, later calls error | `rust/ffi/tests/panic.rs` |
| T-09 | regression | UC-07, R11 | Header drift | `cbindgen` | regenerate and `diff` with the checked-in header | no difference | CI job |
| T-10 | regression | UC-06, R12, R15 | Default JSON unchanged | golden corpus | `cargo test --manifest-path rust/Cargo.toml --test golden` and `capy ast --json` vs. baseline commit | 0 differences without the flag | `TEST-2026-0015` |
| T-11 | failure | R9, `ARCH-003` | JSON dump depth | input at `MAX_PARSE_DEPTH` and one beyond | `recursion_guard` new case | bounded, no abort | `rust/tests/recursion_guard.rs` |
| T-12 | system | R1, R15, R17 | Build on a second platform; wasm size unchanged | Linux x86_64; `wasm_check.sh` | build static library; compare wasm bytes with `HEAD` | builds; wasm size equal | `TEST-2026-0015` |

### Measurement and Validation

Per `QUAL-002`, baselines, commands and thresholds are declared **before** any measurement and frozen at approval.

| Quantity | Baseline | Method and Command | Environment | Threshold | Report |
|---|---|---|---|---|---|
| Static library size | `capy-core` alone as a static library, 5,154,280 bytes (measured by the consumer, 2026-10-13) | `cargo build --profile ffi -p capy-ffi`, `stat -f %z libcapy_ffi.a` | macOS arm64, rustc 1.90 | ≤ 5.5 MB | `TEST-2026-0015` |
| Arena overhead | `Library::parse` time on each sample script | in-process: time `capy_parse` (parse plus flatten) vs `Library::parse`, 15 interleaved samples each (spread rule of `PROP-2026-0007` R3) | same host | ≤ +25 % over `Library::parse` | `TEST-2026-0015` |
| Per-accessor cost | none (new) | 10^7 calls to `capy_node_func` in a C loop | same host | recorded only | `TEST-2026-0015` |
| wasm size | current `capy_core.wasm` bytes | `rust/devtools/wasm_check.sh` plus `stat` | same host | exactly equal | `TEST-2026-0015` |

#### Frozen Predictions (expected result before measurement)

These are predictions, not results. A result far better than predicted must be investigated before it is believed.

| Quantity | Prediction | Basis | If it misses |
|---|---|---|---|
| Static library size | 5.2 – 5.4 MB | The consumer measured 5.15 MB with no exports; the wrapper is a few hundred lines; `regex` dominates | Look for accidental generic instantiation; consider `strip`/`opt-level` already inherited |
| Arena overhead | +8 % to +18 % of `Library::parse` time | One extra pass over a tree already built, pushing one pointer per node; no string copies | A larger figure means copying crept in; check the arena for clones |
| Per-accessor cost | 2 – 6 ns | A bounds check, a kind check and a pointer read | Re-read the validation path |
| wasm size | 0 bytes change | `capy-ffi` is not a dependency of `capy-wasm-abi` | Check the dependency graph |

## Documentation, Demo and Release Impact

| Artifact | Exact Path or Destination | CRUD | Required Content / Verification | Owner | Release Gate |
|---|---|---|---|---|---|
| C ABI page | `docs/c-abi.md` and `mkdocs.yml` nav | CREATE | every export, ownership rules, the sample above, error table; `mkdocs build --strict` | Capy Engine | yes |
| Embedding guide | `docs/embedding.md` | UPDATE | a "From C" section replacing "no C symbols" | Capy Engine | yes |
| AST JSON | `docs/ast-json.md` | UPDATE | the opt-in `expr` field and `--with-expr` | Capy Engine | yes |
| What's new, changelog | `docs/whats-new.md`, `CHANGELOG.md` | UPDATE | one entry each | Capy Engine | yes |
| Plan, TEST, ADR | `program_docs/plans/`, `testing/`, `decisions/` | CREATE (after approval) | plan with increments below; `TEST-2026-0015` with all samples | Capy Engine | yes |
| Manual, demo, release notes | `program_docs/manuals/`, `demos/`, `releases/` | CREATE at release | a C walk-through demo; `REL-*` | Release Management | yes |

## Requirements Alignment

| Requirement | User Request | Goal | Use Cases / Internal Constraint | Changes | Files | Tests | Manual / Demo / Release Evidence |
|---|---|---|---|---|---|---|---|
| R1 | UQ-01, UQ-03 | G-01 | UC-01 | C-01 | F-01, F-02, F-10 | T-01, T-12 | build log |
| R2 | UQ-01 | G-01 | UC-01 | C-01 | F-03 | T-04 | demo |
| R3 | UQ-01 | G-01, G-03 | UC-02 | C-02 | F-03 | T-05 | demo |
| R4 | UQ-01, UQ-02 | G-02 | UC-03 | C-03 | F-03 | T-01, T-02 | `docs/c-abi.md` |
| R5 | UQ-01, UQ-02 | G-02 | UC-04 | C-04 | F-03 | T-03 | `docs/c-abi.md` |
| R6 | UQ-01 | G-02 | UC-05 | C-05 | F-03 | T-02 | `docs/c-abi.md` |
| R7 | UQ-01 | G-02 | UC-01 | C-01 | F-03 | T-06 | `docs/c-abi.md` |
| R8 | UQ-03 | G-03 | UC-02, UC-03 | C-02..C-05 | F-03, F-07 | T-07 | `TEST-2026-0015` |
| R9 | UQ-03 | G-03 | UC-02 | C-02 | F-01, F-03, F-09 | T-08, T-11 | `TEST-2026-0015` |
| R10 | UQ-02 | G-02 | UC-03 | C-03 | F-03 | T-02 | `TEST-2026-0015` |
| R11 | UQ-03 | G-04 | UC-07 | C-07 | F-06 | T-09 | `docs/c-abi.md` |
| R12 | UQ-01, UQ-02 | G-02, G-05 | UC-06 | C-06 | F-04, F-05 | T-10 | `docs/ast-json.md` |
| R13 | UQ-02 | G-02 | UC-03, UC-06 | C-08 | F-07, F-08 | T-02 | `TEST-2026-0015` |
| R14 | UQ-03 | G-03 | UC-02, UC-03 | C-08 | F-07 | T-07 | `TEST-2026-0015` |
| R15 | UQ-03 | G-05 | internal constraint (`GOAL-002`) | C-06 | F-04 | T-10, T-12 | release gate |
| R16 | UQ-04 | G-04 | internal constraint (`AGENTS.md`) | C-09 | F-11 | `mkdocs build --strict` | docs |
| R17 | UQ-03 | G-01, G-05 | internal constraint (`QUAL-002`) | C-10 | F-12 | T-12 | `TEST-2026-0015` |

Reverse check: C-01 to C-10 and F-01 to F-12 each appear against at least one requirement above.

## Plan Strategy and Estimated Work

One plan is enough: the increments below ship together as one minor release and share a test harness. They
are ordered so that each one is useful and reviewable on its own.

```text
  INC-1  crate + profile + library/error/parse handles + block/node/capture/span + diagnostics   ──► T-01, T-05, T-06
  INC-2  expression accessors                                                                    ──► T-03
  INC-3  JSON `expr` field + `capy ast --with-expr` + C JSON dump + differential test            ──► T-02, T-10
  INC-4  header, drift test, misuse suite, sanitizers, panic test, threads                       ──► T-04, T-07, T-08, T-09, T-11
  INC-5  docs, changelog, measurement (`TEST-2026-0015`)                                         ──► T-12
        ▲                                                       │
        └──────── stop after any increment; each leaves the repo green ┘
```

Size is not estimated in calendar time. The only grounded figures are the function count (about 70) and the
consumer's own estimate of 600 to 900 lines of Rust for the wrapper (Capylar RES-2026-0007 s6, an estimate,
not a measurement).

## Open Questions

| # | Question | Needed for |
|---|---|---|
| OQ-01 | Is the canonical repository `github.com/olivierdevelops/capy` or `luowensheng/capy`? The consumer's research found both names and could not tell which is canonical | where the tag and crate live (D7) |
| OQ-02 | Should a stable `staticlib` be accompanied by a `pkg-config` or CMake file, or is the header plus archive enough? | INC-5 |
| OQ-03 | Should `Span` get byte offsets (D5) before ABI 1 is frozen, so that `CapySpan` can carry them from the start? Adding them later is additive by function, but a changed `CapySpan` layout is not | D5 |
| OQ-04 | Should `Library::run` be exposed soon (D6) for hosts that also need the transpile path? | follow-up proposal |
| OQ-05 | Is `Expr` expected to gain spans of its own? The consumer stamps every lowered node with the capture span today | a future ABI minor |

## Approval

Not approved. Agent validation above is not approval.

## Related Documents

- [PROP-2026-0007](prop-2026-0007-parser-and-pipeline-speed-experiments.md) — the measurement rules (spread, interleaving) reused in §Measurement
- [Project standards index](../standards/index.md), [architecture rules](../standards/architecture-rules.md), [quality expectations](../standards/quality-expectations.md)
- [`docs/ast-json.md`](../../docs/ast-json.md), [`docs/embedding.md`](../../docs/embedding.md)
- Consumer (outside this repository): `capyler/docs/proposals/prop-2026-0001-rewrite-the-compiler-in-capylar.md` (PROP-2026-0001 there) and its research record `res-2026-0007-capy-as-a-c-ffi-library.md`, which contains the evidence summarised in §Problem and Evidence

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial draft from the Capylar consumer proposal and a read of `rust/src`, `rust/wasm`, `docs/ast-json.md` and the standards |
