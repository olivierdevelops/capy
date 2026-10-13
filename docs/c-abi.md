---
title: Calling Capy from C
---

# Calling Capy from C

`capy-ffi` is a C ABI over the parse tree. Compile a grammar once, parse as many files as you
like, and read statements, captures, typed expressions, spans, error regions and diagnostics —
without Rust, without a process per file, and without a JSON reader.

```text
   your program (C, Zig, Ambit, …)                      libcapy_ffi.{a,dylib,so} + capy.h

   capy_library_new(grammar) ──────────────────────────► CapyLibrary   compile ONCE, read-only,
                                                                        shareable across threads
   capy_parse(lib, source)   ──────────────────────────► CapyParse     one owner; ids + borrowed text
   capy_node_* capy_cap_* capy_expr_* capy_diag_*  ────►     ▲
   capy_parse_free(p)  ────── every borrowed string dies here ┘
   capy_library_free(lib)
```

It parses; it does not transpile. `run`, `introspect` and `set_host` are not exposed, so a C host
always gets the sandboxed default host.

## Build

```sh
cargo build --manifest-path rust/Cargo.toml --profile ffi -p capy-ffi
#   rust/target/ffi/libcapy_ffi.a        static
#   rust/target/ffi/libcapy_ffi.dylib    shared  (.so on Linux)
#   rust/ffi/include/capy.h              the header (checked in)

cc -O2 -Irust/ffi/include walk.c rust/target/ffi/libcapy_ffi.a -lc -lm -Wl,-dead_strip -o walk                    # macOS
cc -O2 -Irust/ffi/include walk.c rust/target/ffi/libcapy_ffi.a -lc -lm -lpthread -ldl -Wl,--gc-sections -o walk   # Linux
zig cc … # works the same way
```

**Use `--profile ffi`.** It is `release` with `panic = "unwind"`, which is what lets the library turn
a Rust panic into an error code instead of aborting your program. Ask a build which one it is:

```c
if (!capy_panics_are_caught()) { /* built with panic = "abort": a panic would end the process */ }
```

**Sizes (measured, macOS arm64).** The static archive is about 31 MB because it carries the whole Rust
standard library. What matters is what you link: the complete program above is about 1.3 MB with
`-Wl,-dead_strip` (`--gc-sections` on Linux), 4.4 MB with clang's defaults and 21.6 MB with `zig cc`'s
defaults, which keep every object. Pass the dead-code flag. The shared library is 0.8 MB.

## A complete program

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

This program is `examples/c/walk.c`; `examples/c/run.sh walk` builds and runs it and expects `greet(who=world)`.

## The rules

| Rule | Meaning |
|---|---|
| **Strings in** | a pointer and a length, UTF-8, never NUL-terminated. Invalid bytes in *source* are replaced (as in the wasm ABI); an invalid *grammar* is `CAPY_E_UTF8`. |
| **Strings out** | `CapyStr { ptr, len }`: UTF-8, not NUL-terminated, **borrowed** from the handle you read it from and dead after that handle is freed. `ptr == NULL` means absent; a non-NULL `ptr` with `len == 0` is an empty string. |
| **Ids** | `CapyId` (`uint32_t`) names a block, statement, capture, expression or error node of **one** parse. `0` means none. An id from another parse, a wrong-kind id and an out-of-range id are all rejected and read as zero. |
| **Misuse** | a bad id or index returns `0`, `{NULL, 0}` or a status code. It never reads out of bounds. Using a handle after its free, freeing twice, or passing a pointer/length that is not readable memory is a caller error no library can detect. |
| **Threads** | a `CapyLibrary` is read-only after creation and may be used by many threads at once. A `CapyParse` has one owner; reading it from several threads at once is fine, freeing it while another thread reads is not. |
| **Order** | captures and sections are enumerated in sorted name order and statements in source order — the same order as `capy ast --json`. |
| **Panics** | with `--profile ffi` a panic never crosses into your code: the call returns its zero value (or `CAPY_E_PANIC`) and the handle becomes *poisoned*; every later call on it returns zero. `capy_parse_is_poisoned` tells you. |

## Status codes

| Code | Name | When |
|---|---|---|
| 0 | `CAPY_OK` | success |
| 1 | `CAPY_E_NULL` | a required pointer was NULL |
| 2 | `CAPY_E_ARG` | an index, flag or argument was out of range / unknown |
| 3 | `CAPY_E_KIND` | the id names the wrong kind of node |
| 4 | `CAPY_E_UTF8` | a grammar or path was not valid UTF-8 |
| 5 | `CAPY_E_PANIC` | a Rust panic was caught |
| 6 | `CAPY_E_LIMIT` | the tree needs more than 2^24 − 2 ids |
| 7 | `CAPY_E_LOAD` | the grammar failed to compile; read the `CapyError` |

## Function reference

`capy.h` is the authority; it is generated from the crate and checked in
(`rust/ffi/check_header.sh` fails when it drifts). Within one ABI version, changes are additive only.

| Group | Functions |
|---|---|
| Version | `capy_abi_version`, `capy_panics_are_caught` |
| Library | `capy_library_new`, `capy_library_from_file`, `capy_library_free` |
| Load error | `capy_error_msg`, `capy_error_hint`, `capy_error_file`, `capy_error_line`, `capy_error_col`, `capy_error_free` |
| Parse | `capy_parse`, `capy_parse_free`, `capy_is_clean`, `capy_parse_is_poisoned`, `capy_root` |
| Block | `capy_block_len`, `capy_block_stmt`, `capy_block_verbatim`, `capy_block_error_count`, `capy_block_error` |
| Statement | `capy_node_func`, `capy_node_span`, `capy_node_line`, `capy_node_col`, `capy_node_capture_count`, `capy_node_capture_name`, `capy_node_capture`, `capy_node_body`, `capy_node_closer`, `capy_node_section_count`, `capy_node_section_name`, `capy_node_section_at`, `capy_node_section`, `capy_node_comment_count`, `capy_node_comment_span` |
| Capture | `capy_cap_text`, `capy_cap_span`, `capy_cap_is_expr`, `capy_cap_expr`, `capy_cap_sub_count`, `capy_cap_sub` |
| Expression | `capy_expr_kind`, `capy_expr_int`, `capy_expr_float`, `capy_expr_bool`, `capy_expr_str`, `capy_expr_child_count`, `capy_expr_child`, `capy_expr_name_count`, `capy_expr_name`, `capy_var_step_count`, `capy_var_step_is_index`, `capy_var_step_field`, `capy_var_step_index`, `capy_obj_key_count`, `capy_obj_key` |
| Diagnostic | `capy_diag_count`, `capy_diag_code`, `capy_diag_severity`, `capy_diag_message`, `capy_diag_help`, `capy_diag_primary`, `capy_diag_label_count`, `capy_diag_label`, `capy_diag_context_count`, `capy_diag_context` |
| Error node | `capy_err_span`, `capy_err_diag_index`, `capy_err_token_count`, `capy_err_token_text`, `capy_err_token_line`, `capy_err_token_col` |
| JSON | `capy_parse_to_json`, `capy_buf_free` |

### Spans

`CapySpan` is four `uint32_t`: `start_line`, `start_col`, `end_line`, `end_col`. Lines and columns are
1-indexed and `end_col` is exclusive. `start_line == 0` means *unset* (an optional capture that took its
default). There are no byte offsets; re-slice the source by line and column.

### Reading an expression

An `any`-typed capture carries a typed expression tree. `capy_cap_expr` gives its id (or `0`), and
`capy_expr_kind` says which of twelve shapes it is:

```text
  kind (CAPY_EXPR_*)   read with                                    children (capy_expr_child)
  ───────────────────  ───────────────────────────────────────────  ───────────────────────────────
  NUMBER               capy_expr_int (1 if integer) / capy_expr_float
  STR                  capy_expr_str
  BOOL                 capy_expr_bool
  NULL
  VAR                  capy_var_step_count, _is_index, _field        one slot per step: the index
                                                                    expression, or 0 for a field step
  CALL                 capy_expr_name_count / capy_expr_name         arguments
  COMPARE  BINARY      capy_expr_str  (the operator)                 left, right
  UNARY                capy_expr_str  (the operator)                 operand
  NOT                                                                operand
  LIST                                                               items
  OBJ                  capy_obj_key_count / capy_obj_key             values, in key order
```

```text
  `a.b[i + 1]`   VAR  steps: field "a", field "b", index ──► BINARY "+"
                                                              ├─ VAR [field "i"]
                                                              └─ NUMBER 1
```

A grammar's `any` capture does not currently produce `CALL`; the accessors still read it.

### Diagnostics

`capy_diag_message` is the message *without* its context clause; the clause is in
`capy_diag_context` (`shape`, `arg_index`, `arg_name`). The `message` in `capy ast --json` is the
message with the last context frame appended — compose the same way if you want identical text.
`capy_diag_severity` is `0` error, `1` warning, `2` for an invalid index and `3` for a severity a newer
engine added.

### JSON

```c
uint8_t *buf; size_t len;
if (capy_parse_to_json(p, CAPY_JSON_EXPR | CAPY_JSON_PRETTY, &buf, &len) == CAPY_OK) {
    fwrite(buf, 1, len, stdout);
    capy_buf_free(buf, len);
}
```

`CAPY_JSON_EXPR` adds the typed `expr` member to every capture (see [AST JSON](ast-json.md));
`CAPY_JSON_PRETTY` indents. An unknown flag bit is `CAPY_E_ARG`.

## Limits and known behaviour

| Item | Behaviour |
|---|---|
| Expression nesting | beyond 64 levels (parentheses, brackets, `not`, `**` chains, braces) the statement is refused with a diagnostic instead of overflowing the stack. Before 0.26.0 about 20 000 nested parentheses aborted the process. |
| Capture nesting | the existing bound of 64 captures still applies (`E0003`) |
| Tree size | more than 2^24 − 2 nodes in one parse is `CAPY_E_LIMIT` |
| Ids from another parse | detected with high probability (an 8-bit tag), not with certainty; do not mix ids across parses |
| Allocation failure | Rust aborts on out-of-memory; it cannot be turned into a status |
| Host | always the sandboxed no-op host: `env`, `arg` and `read_file` are inert |

## Testing it yourself

```sh
examples/c/run.sh walk      # the program above           -> walk: PASS (greet(who=world))
examples/c/run.sh misuse    # 76 misuse checks under ASan + UBSan
examples/c/run.sh hostile   # empty, NUL, 1 MiB line, 10 000 openers, random bytes
cargo test --profile ffi -p capy-ffi       # C view == Rust JSON for every sample, all 12 expression kinds, threads
rust/ffi/check_header.sh                    # capy.h matches the crate
```
