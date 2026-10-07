# parse-recovery

**What it teaches:** `capy run` refuses a broken file and reports the first
error. `capy ast` recovers: it returns a tree of everything that *did* parse,
plus a diagnostic for each region that did not. Shipped in 0.22.0.

```sh
capy run samples/parse-recovery/lib.capy samples/parse-recovery/script.capy   # clean
capy run samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy   # refuses
capy ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy   # recovers
```

## The DSL

Three statements, each short enough that a failure is one line:

```text
task "NAME" due "DATE"    both arguments required; DATE is a pattern type
done "NAME"               one argument
list                      no arguments
```

## Same file, two commands

`broken.capy` holds five lines: three good, two bad.

```text
  broken.capy                        capy run                   capy ast
  ────────────────────────────       ──────────────────────     ─────────────────────
  1  task "write…"  due "2026-…"     ┐                          ✓ task   1:1-1:43
  2  task "missing its due date"     │ stops here, reports      ✗ <error> 2:1-2:28
  3  done "read the release notes"   │ ONE error, emits         ✓ done   3:1-3:30
  4  tsak "a typo in the keyword"    │ NO output                ✗ <error> 4:1-4:29
  5  list                            ┘                          ✓ list   5:1-5:5
                                       exit 1                     exit 1, tree on stdout
                                                                  3 statements, 2 errors
```

```text
$ capy run samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
error: expected `due`, found end of statement in `task`
  2 │ task "missing its due date"

$ capy ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
task 1:1-1:43
  name = "\"write the proposal\"" 1:6
  when = "2026-09-16" 1:31
done 3:1-3:30
  name = "\"read the release notes\"" 3:6
list 5:1-5:5
<error> 2:1-2:28  3 token(s) skipped
<error> 4:1-4:29  3 token(s) skipped
error[E0001] 2:1: expected `due`, found end of statement in `task`
error[E0001] 4:1: no library function matches token "tsak"
```

Two things to notice:

1. **The error names what it wanted.** ``expected `due`, found end of statement
   in `task` `` — the shape that got furthest, and what it needed next. Before
   0.22.0 this said only "no library function matches".
2. **One run, every error.** Fixing line 2 and re-running would have revealed
   line 4; recovery shows both at once. That is the loop an editor or an agent
   needs.

Diagnostics go to **stderr** in tree mode, so stdout stays pipeable. Add
`--json` for the machine-readable form — see
[docs/ast-json.md](../../docs/ast-json.md).

## Files and goldens

| File | Golden | Asserts |
|---|---|---|
| `script.capy` | `script.expected.txt` | the clean file renders |
| `broken.capy` | `broken.expected-error.txt` | what `capy run` refuses with |
| `broken.capy` | `broken.expected-ast.txt` | the tree + diagnostics `capy ast` recovers |

The third is the `<base>.expected-ast.txt` kind added by PROP-2026-0002. One
script legitimately carries both an error golden and an AST golden: they assert
two different commands on the same input. See
[the goldens section](../README.md#how-goldens-work).

## Not in the playground — on purpose

The browser playground runs on the wasm ABI, and that ABI exposes neither the
AST nor diagnostics (`REL-0.22.0` Known Limitation 5). This sample would look
broken there, so it is deliberately absent from `rust/playground/src/curated.rs`.
Run it from the CLI.
