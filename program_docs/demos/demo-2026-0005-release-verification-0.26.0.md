---
document_id: DEMO-2026-0005
title: Release Verification Guide — 0.26.0
document_type: demo
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
  - capy-cli
  - docs

affected_versions:
  from: "0.26.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - release-managers

scope: Gives a reader who did not implement 0.26.0 an executable procedure to verify every released update.

reason: DOCUMENTATION.md section 29 requires a release verification guide and demo for every release.

related_documents:
  - PLAN-2026-0005
  - PROP-2026-0008
  - TEST-2026-0015
  - RPT-2026-0005
  - MAN-2026-0003
  - INC-2026-0002
  - REL-0.26.0

supersedes: null
superseded_by: null

tags:
  - demo
  - verification
  - ffi
  - c-abi

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-13
---

# Release Verification Guide — 0.26.0

> **Status:** Active
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.26.0
> **Owner:** Capy Engine
> **Affected Components:** capy-ffi, capy-core, capy-cli, docs

## Summary

```text
  U-01 build the C library          U-05 typed expressions from the CLI       U-09 the header matches the crate
  U-02 run a complete C program     U-06 deep nesting no longer aborts        U-10 the documentation builds
  U-03 errors and diagnostics       U-07 misuse and hostile input, sanitized  U-11 faster run  (not directly testable here)
  U-04 read a typed expression      U-08 default JSON is unchanged
```

Eleven updates, each with a command and an expected result. Ten were executed against the release
tree and recorded below; U-11 is `NOT DIRECTLY USER-TESTABLE` and cites its evidence.

## Release Identity

| Field | Value |
|---|---|
| Release | 0.26.0 |
| Tag | `v0.26.0` |
| Commit | recorded in `REL-0.26.0` (the tag is placed on the `docs(program)` commit that contains this guide) |
| Previous | 0.25.0 |

## Purpose

Lets a reader confirm, without reading the implementation, that the C ABI works, that the new JSON option
adds only what it says, that the crash fix holds, and that nothing existing changed.

## Verified Against Version

`capy 0.26.0` (`rust/target/debug/capy version` prints `capy 0.26.0`), macOS 26.4.1 arm64, rustc 1.90.0,
Apple clang 21.0.0, zig 0.16.0, cbindgen 0.29.4, deno 2.5.6.

## Prerequisites

- A Rust toolchain (1.74 or newer), a C compiler (`clang` or `zig cc`), `cbindgen` for U-09, `mkdocs` for U-10.
- A checkout of the release commit. Commands run from the repository root unless a `cd` is shown.

## Setup

```sh
git checkout v0.26.0
cd rust && cargo build --workspace && cd ..          # builds target/debug/capy
```

For U-06's comparison, a baseline binary of 0.25.0 is optional:

```sh
D_BASE=$(mktemp -d); git archive 69fb4ac rust | tar -x -C "$D_BASE" && (cd "$D_BASE/rust" && cargo build --release -p capy-cli)   # $D_BASE/rust/target/release/capy
```

## How to Read This Guide

Each `U-NN` states what changed, the command, and a pass condition specific enough to decide without reading
code. `Evidence` names the test that also covers it.

## Release Updates

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | PROP-2026-0008 R1 | A C-linkable library and header exist | `cargo build --manifest-path rust/Cargo.toml --profile ffi -p capy-ffi` | `libcapy_ffi.a`, `libcapy_ffi.dylib`/`.so` under `rust/target/ffi/`; header at `rust/ffi/include/capy.h` | TEST-2026-0015 T-13 |
| U-02 | R2, R3, R4 | A C program compiles a grammar once, parses, walks | `examples/c/run.sh walk` | `walk: PASS (greet(who=world))` | T-01 |
| U-03 | R3, R6, R7, R8 | Load errors and diagnostics are readable from C; bad ids read as nothing | `examples/c/run.sh diagnose` | status 7 and the grammar message; a parse that is not clean with `E0001 2:1`; a bad id gives NULL | T-05, T-06 |
| U-04 | R5, R12 | The typed expression behind a capture is readable from C | `examples/c/run.sh exprs` | an indented tree: `binary *` with a `var` path containing an index, and `number -2` | T-03 |
| U-05 | R12 | `capy ast --json --with-expr` adds the typed expression | see U-05 below | no `"expr"` by default; two with the flag; refused without `--json` | T-10 |
| U-06 | PROP-2026-0008 R9 (INC-2026-0002) | A deeply nested expression no longer aborts the process | see U-06 below | 0.25.0: exit 134; 0.26.0: exit 1 with a diagnostic; 63 pairs still parse | T-11 |
| U-07 | R8, R14 | Misuse and hostile input are survived, under sanitizers | `examples/c/run.sh misuse && examples/c/run.sh hostile` | `misuse: 76 checks, 0 failed`; `hostile: 30 checks, 0 failed` | T-05, T-07 |
| U-08 | R15 | Default `capy ast --json` is byte-identical to 0.25.0 | `rust/devtools/ast_json_compat.sh <baseline-capy>` | `compared 133 scripts, 0 differences` | T-10 |
| U-09 | R11 | The header cannot drift from the crate | `rust/ffi/check_header.sh` | `capy.h is up to date` | T-09 |
| U-10 | R16 | The documentation is true and builds | `mkdocs build --strict` | exit 0; `docs/c-abi.md` is in the nav | T-13 |
| U-11 | PROP-2026-0007 | `Library::run` is faster on flat statements | `NOT DIRECTLY USER-TESTABLE` in this guide | — | TEST-2026-0014 (measured at the commit before this release) |

### U-01 — build the C library

```sh
cargo build --manifest-path rust/Cargo.toml --profile ffi -p capy-ffi
ls rust/target/ffi/libcapy_ffi.a rust/target/ffi/libcapy_ffi.dylib rust/ffi/include/capy.h
```

Expected: all three files listed (`.so` instead of `.dylib` on Linux). Use `--profile ffi`: it keeps
`panic = "unwind"`, which lets the library turn a Rust panic into an error code.

### U-02 — run a complete C program

```sh
examples/c/run.sh walk
```

```text
walk: PASS (greet(who=world))
```

The program is `examples/c/walk.c`, built with `zig cc -O2` and `-Wl,-dead_strip` (or the system compiler
if `zig` is absent).

### U-03 — errors and diagnostics

```sh
examples/c/run.sh diagnose
```

```text
load: status=7 lib=NULL
load: function "broken": capture "x" has unknown type "nonsense_type"
parse: status=0 clean=0 diagnostics=1
  E0001 2:1 no library function matches token "hello"
statements=1 error_regions=1
bad id: ptr=NULL len=0
```

Pass condition: status `7` (`CAPY_E_LOAD`) with the grammar message, a parse that returns a handle that is not
clean, one diagnostic with a position, the good statement still present, and a NULL string for an id that does not exist.

### U-04 — read a typed expression

```sh
examples/c/run.sh exprs
```

```text
value text: a.b[i + 1] * -2
binary *
  var
    .a
    .b
    [index]
      binary +
        var
          .i
        number 1
  number -2
json: 776 bytes, has expr: yes
```

### U-05 — the `--with-expr` option

```sh
D=$(mktemp -d)
printf 'extension txt\nfunction let\n  arg literal "let"\n  arg capture name ident\n  arg literal "="\n  arg capture value any\n  write `${name}`\nend\n' > $D/lib.capy
printf 'let y = a + 2 * 3\n' > $D/s.capy
rust/target/debug/capy ast $D/lib.capy $D/s.capy --json            | grep -c '"expr"'     # 0
rust/target/debug/capy ast $D/lib.capy $D/s.capy --json --with-expr | grep -c '"expr"'     # 2
rust/target/debug/capy ast $D/lib.capy $D/s.capy --with-expr; echo "rc=$?"
```

```text
0
2
--with-expr needs --json
rc=1
```

With `jq`: `… --json --with-expr | jq -c '.tree.stmts[0].captures.value.expr'` prints
`{"kind":"binary","left":{"kind":"var","path":[{"field":"a"}]},"op":"+","right":{"kind":"binary","left":{"int":2,"is_int":true,"kind":"number"},"op":"*","right":{"int":3,"is_int":true,"kind":"number"}}}`.

### U-06 — deep nesting

```sh
python3 -c "n=20000; print('let x = '+'('*n+'1'+')'*n)" > $D/deep.capy
"$D_BASE/rust/target/release/capy" ast $D/lib.capy $D/deep.capy 2>&1 | head -2 ; echo "rc=${PIPESTATUS[0]}"   # the 0.25.0 binary
cargo build --release -p capy-cli --manifest-path rust/Cargo.toml
rust/target/release/capy ast $D/lib.capy $D/deep.capy >/dev/null 2>$D/err; echo "rc=$?"; grep -m1 E0003 $D/err
```

```text
thread 'main' has overflowed its stack
rc=134                                                              <- 0.25.0
rc=1                                                                <- 0.26.0
error[E0003] 1:1: expression nesting too deep (limit about 63 nested parentheses) — the source nests further than the parser will follow
```

The boundary: `n=63` pairs of parentheses exits 0; `n=64` exits 1, with the same `E0003` message.
`capy run` prints the message with a caret under the column where the parser stopped.

### U-07 — misuse and hostile input

```sh
examples/c/run.sh misuse
examples/c/run.sh hostile
```

```text
misuse: 76 checks, 0 failed
hostile: 30 checks, 0 failed
```

Both are compiled with `-fsanitize=address,undefined -fno-sanitize-recover=undefined`; any sanitizer report fails the run.

### U-08 — default JSON unchanged

```sh
rust/devtools/ast_json_compat.sh "$D_BASE/rust/target/release/capy"     # the 0.25.0 binary from Setup
```

```text
compared 133 scripts, 0 differences
```

### U-09 — header drift

```sh
rust/ffi/check_header.sh
```

```text
capy.h is up to date
```

### U-10 — documentation

```sh
mkdocs build --strict ; echo "rc=$?"
```

Expected: `rc=0`. Open `docs/c-abi.md` in the built site under *Integrate → Call Capy from C*.

## Cleanup

```sh
rm -rf "$D" "$D_BASE"; rm -rf rust/target/c-examples
```

## Known Caveats

- U-06's baseline needs a 0.25.0 build; without one, the 0.26.0 half still demonstrates the exit 1.
- `examples/c/run.sh` uses `zig` for `walk` when present and the system `clang` for the sanitizer builds.
- Timings are not part of this guide (`TEST-2026-0015` records them).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `capy: command not found` / wrong version | a stale `capy` on PATH (`TRBL-2026-0001`) | use `rust/target/debug/capy`; `cargo build --workspace`, not a bare `cargo build` |
| `cbindgen is required` | not installed | `cargo install cbindgen` |
| `capy_panics_are_caught() == 0` | built with `--release` instead of `--profile ffi` | rebuild with `--profile ffi` |
| link error `undefined reference to ... pthread` on Linux | missing system libraries | add `-lpthread -ldl -lm` |

## Verification Record

Executed 2026-10-13 against the release tree on macOS 26.4.1 arm64. Output is as shown in each section.

| Update | Result | Observed |
|---|---|---|
| U-01 | PASS | `libcapy_ffi.a` 31,225,392 B, `libcapy_ffi.dylib` 806,608 B, header present |
| U-02 | PASS | `walk: PASS (greet(who=world))` |
| U-03 | PASS | output exactly as shown |
| U-04 | PASS | output exactly as shown |
| U-05 | PASS | `0`, `2`, `--with-expr needs --json`, `rc=1`; jq output as shown |
| U-06 | PASS | baseline rc=134 (stack overflow); 0.26.0 rc=1 with `E0003`; 61, 62, 63 pairs rc=0; 64 and 65 rc=1 |
| U-07 | PASS | 76 and 30 checks, 0 failed |
| U-08 | PASS | 133 scripts, 0 differences |
| U-09 | PASS | `capy.h is up to date` |
| U-10 | PASS | `mkdocs build --strict` rc=0 |
| U-11 | NOT DIRECTLY USER-TESTABLE | evidence `TEST-2026-0014` |

## Limitations

- Only macOS arm64 was run. The Linux static library was cross-built, not linked or run.
- U-11 is a performance change measured in `TEST-2026-0014`, not exercised here.

## Actual Verification Status

10 of 10 testable updates PASS.

## Related Documents

- `PLAN-2026-0005`, `PROP-2026-0008`, `TEST-2026-0015`, `RPT-2026-0005`, `MAN-2026-0003`, `INC-2026-0002`, `REL-0.26.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial guide; every update executed against the release tree |
