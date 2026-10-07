---
document_id: DEMO-2026-0003
title: Release Verification Guide — 0.23.0
document_type: demo
status: active

created_date: 2026-10-07
last_updated: 2026-10-07
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
  - docs
  - samples

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - technical-writers

scope: An executable verification procedure for every update released in 0.23.0 — ordered choice in a capture type (PROP-2026-0004) and the carried public docs and samples work (PROP-2026-0002) — written for a reader who did not implement the change.

reason: DOCUMENTATION.md section 29 makes a release verification guide mandatory for every release.

related_documents:
  - PLAN-2026-0003
  - PROP-2026-0004
  - PROP-2026-0002
  - ADR-0003
  - MAN-2026-0002
  - SYS-2026-0001
  - DEMO-2026-0002

supersedes: null
superseded_by: null

tags:
  - demo
  - verification
  - release
  - alternation

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-04-07
---

# Release Verification Guide — 0.23.0

> **Status:** Active
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, docs, samples

## Summary

Thirteen released updates, each verifiable from a shell. The first eight are the
engine change: a capture type may now name several functions (`call | name | num`),
tried left to right. The last five are the carried documentation and samples
work, whose commands are re-run here so the pages cannot drift from the binary.

```text
  WHAT 0.23.0 ADDS                             WHERE YOU VERIFY IT
  ──────────────────────────────────────────   ──────────────────────────
  CHOICE  arg capture v call | name | num      U-01 .. U-08   (engine)
  carried public docs and samples              U-09 .. U-13   (PROP-2026-0002)
```

## Release Identity

| Field | Value |
|---|---|
| Version | 0.23.0 |
| Tag | `v0.23.0` (created at release; the commit hash is recorded in `REL-0.23.0`, which is written after the tag) |
| Plan | PLAN-2026-0003 |
| Proposal | PROP-2026-0004 (engine), PROP-2026-0002 (carried docs and samples) |
| Decision | ADR-0003 |
| Validation | RPT-2026-0003 (per the plan; written in phase P5) |

## Purpose

Show, with output a reader can compare byte for byte, that each user-visible
change in 0.23.0 does what its inciting requirement asked.

## Verified Against Version

0.23.0, working tree on `main` at base commit `84f984c`, **before** the version
bump: `rust/Cargo.toml` still read `0.22.0` when these commands were run, so the
binary reports `capy dev` rather than `0.23.0`. Behaviour, not the version string,
is what is verified here. Re-run `capy version` after the release commit to
confirm the string.

## Prerequisites

```sh
cd rust && cargo build --workspace && cd ..          # debug profile, as executed here
export CAPY=$PWD/rust/target/debug/capy              # or the release binary
export W=/tmp/capy-verify-23 && mkdir -p $W
command -v jq                                        # U-07 and U-12 need jq
```

All commands run from the repository root. Scratch files go under `$W`, never in
the repo. The commands below were executed with `$W` set to the session scratch
directory; nothing printed depends on that path.

## Setup

None beyond the build. Every library and script used is either already in the
repository (`samples/…`) or is written by the step that needs it.

## How to Read This Guide

```text
   ┌────────────┐   ┌──────────────┐   ┌───────────────┐   ┌──────────────┐
   │ run command│──►│ compare with │──►│ record PASS / │──►│ next update  │
   │            │   │ Expected     │   │ FAIL          │   │              │
   └────────────┘   └──────────────┘   └───────────────┘   └──────────────┘
                          │ differs
                          └──► Troubleshooting table, then raise it
```

## Release Updates

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | `PROP-2026-0004` UQ-01, UQ-04 / R1, R4 / UC-01 | Nested calls parse: an argument is a call, a name or a number | § U-01 | `return add(3, mul(4, 5));` | `rust/tests/alternation.rs` `nested_calls_parse`, `five_deep_nest_parses`; golden `samples/expression-grammar` |
| U-02 | `PROP-2026-0004` UQ-05 / R1, R3 / UC-06 | A parameter list mixes marked and unmarked parameters | § U-02 | `f(&c: Counter, n: int)` | `mixed_parameter_markers_parse`; golden `samples/mixed-parameters` |
| U-03 | `PROP-2026-0004` R2 | First match wins, and the order is observable | § U-03 | `name` vs `num` swaps with the order | `first_matching_alternative_wins`, `failed_alternative_is_rewound` |
| U-04 | `PROP-2026-0004` R6 / UC-01 negative path | A failed choice names every alternative | § U-04 | ``expected a `call`, a `name`, or a `num` `` | `failure_reports_the_union_of_alternatives`; golden `broken.expected-error.txt` |
| U-05 | `PROP-2026-0004` R5 / UC-04 | Left recursion through a later alternative is refused at load | § U-05 | the existing cycle message, exit 1 | `left_recursion_through_second_alternative_is_rejected` |
| U-06 | `PROP-2026-0004` R9 | `capy docs` prints the whole choice | § U-06 | Type column ``call \| name \| num`` (pipes escaped for the Markdown table) | `docs_print_the_union`, `alternation_loads_and_is_introspectable` |
| U-07 | `PROP-2026-0004` R8 / UC-03 | `sub[].func` names the matched alternative | § U-07 | `["ret","call","operand","num",…]` | `discriminator_names_the_matched_alternative` |
| U-08 | `PROP-2026-0004` R1 / OQ-08 (ADR-0003) | A flat built-in type is refused as an alternative | § U-08 | `…alternation names "int", which is not a library function` | `builtin_type_is_not_an_alternative`, `unknown_alternative_is_a_load_error_naming_it` |
| U-09 | `PROP-2026-0002` (public samples) | `capy ast` recovers on a broken file (`samples/parse-recovery`) | § U-09 | 3 statements, 2 error regions | golden `broken.expected-ast.txt` |
| U-10 | `PROP-2026-0002` (public samples) | Operator precedence sample | § U-10 | `a == 7`, `b == 9`, `c == 5` … | golden `script.expected.txt` |
| U-11 | `PROP-2026-0002` (front-end positioning) | The `language-frontend` sample lowers to Python and recovers on error | § U-11 | `def gcd(a: int, b: int):` | goldens in `samples/language-frontend` |
| U-12 | `PROP-2026-0002` (tutorial 05) | Every command in "Reading diagnostics" gives the output the page shows | § U-12 | outputs match the tutorial | `docs/tutorials/05-reading-diagnostics.md`, golden corpus |
| U-13 | `PROP-2026-0004` R7 | No existing library changes behaviour | § U-13 | `cargo test --workspace`: 128 passed, 0 failed | cargo test summary below |

The journey every engine update shares:

```text
 [library author]
       │  writes   arg capture v call | name | num
       ▼
 [capy check]  ── every alternative a function? ── no ──► load error naming it   (U-08)
       │  yes
       │  ── any cycle through ANY alternative? ── yes ─► left-recursion error   (U-05)
       ▼  ok
 [capy run / ast] ── try call ─► ✅ match, stop ─────────────────────────────► (U-01, U-03)
       │               │ ❌ rewind this capture only
       │               ▼
       │            try name ─► ✅ ... else try num ─► ✅ ... else
       ▼                                                       │ none
 [tree / output]                                               ▼
   sub[].func = the winner  (U-07)             union diagnostic: `call`, `name`, `num` (U-04)
```

---

### U-01 — nested calls parse

```sh
printf 'return add(3, mul(4, 5))\n' > $W/n.capy
$CAPY run samples/expression-grammar/lib.capy $W/n.capy; echo "exit $?"
$CAPY run samples/expression-grammar/lib.capy samples/expression-grammar/script.capy
```

**Expected:**

```text
return add(3, mul(4, 5));
exit 0
return add(3, mul(4, 5));
return f(g(h(i(j(1)))), k(2, x));
return now();
```

See the tree, which shows the recursion `operand -> call -> operand -> call`:

```sh
$CAPY ast samples/expression-grammar/lib.capy $W/n.capy
```

```text
ret 1:1-1:25
  value:
    call 1:8-1:25
      args:
        operand 1:12-1:13
          v:
            num 1:12-1:13
              n = "3" 1:12
        operand 1:15-1:24
          v:
            call 1:15-1:24
              args:
                operand 1:19-1:20
                  v:
                    num 1:19-1:20
                      n = "4" 1:19
                operand 1:22-1:23
                  v:
                    num 1:22-1:23
                      n = "5" 1:22
              fname = "mul" 1:15
      fname = "add" 1:8
```

The glued spelling is the same choice. This is the library with every ` | ` rewritten `|`:

```sh
sed 's/call | name | num/call|name|num/' samples/expression-grammar/lib.capy > $W/glued.capy
$CAPY run $W/glued.capy $W/n.capy
```

```text
return add(3, mul(4, 5));
```

Depth. The engine's existing nesting bound is 64 captures, which is 31 call
levels in this grammar:

```sh
nest(){ s="1"; for i in $(seq 1 $1); do s="f($s)"; done; echo "return $s"; }
nest 31 > $W/d31.capy; nest 32 > $W/d32.capy
$CAPY run samples/expression-grammar/lib.capy $W/d31.capy > $W/d31.out; echo "exit $?"
$CAPY run samples/expression-grammar/lib.capy $W/d32.capy >/dev/null 2>$W/d32.err; echo "exit $?"; head -1 $W/d32.err
```

```text
exit 0
exit 1
error: expected `)`, found "1" in `call`
```

**Before 0.23.0** nesting stopped after one level: `PROP-2026-0004` P-01 records
``error: expected `)`, found "(" in `call` `` for `add(3, mul(4, 5))` on 0.22.0
(a prior-release observation from that proposal, not re-run here). Note the
depth-32 message names the innermost `call`, not the limit: the bound's own
message is consumed by capture-local backtracking (see Limitations).

---

### U-02 — marked and unmarked parameters in one list

```sh
$CAPY run samples/mixed-parameters/lib.capy samples/mixed-parameters/script.capy
printf 'def f(mut c: Counter, n: int)\n' > $W/m.capy
$CAPY run samples/mixed-parameters/lib.capy $W/m.capy
$CAPY ast samples/mixed-parameters/lib.capy $W/m.capy
```

**Expected:**

```text
open_door(&d: Door)
poke(d: Door)
update(&c: Counter, n: int)
swap(n: int, &c: Counter, &d: Door)
now()
f(&c: Counter, n: int)
def_fn 1:1-1:30
  name = "f" 1:5
  ps:
    mut_param 1:7-1:21
      pname = "c" 1:11
      ptype = "Counter" 1:14
    plain_param 1:23-1:29
      pname = "n" 1:23
      ptype = "int" 1:26
```

The failing contrast: the same library with a **single** shape in that position
(what a 0.22.0 library could express) rejects the mixed signature:

```sh
sed 's/arg capture ps mut_param | plain_param\* sep/arg capture ps mut_param* sep/' \
    samples/mixed-parameters/lib.capy > $W/single.capy
$CAPY run $W/single.capy $W/m.capy; echo "exit $?"
```

```text
error: expected `mut`, found "n" in `mut_param`
  1 │ def f(mut c: Counter, n: int)
    │                       ^
exit 1
```

---

### U-03 — first match wins, and you can see which

`num` is declared `int`, which accepts a bare identifier at parse time, so the
order decides what `x` becomes.

```sh
printf 'return f(x)\n' > $W/o.capy
$CAPY ast samples/expression-grammar/lib.capy $W/o.capy        # call | name | num
sed 's/arg capture v call | name | num/arg capture v call | num | name/' \
    samples/expression-grammar/lib.capy > $W/swapped.capy
$CAPY ast $W/swapped.capy $W/o.capy                            # call | num | name
```

**Expected:** the innermost node is `name` in the first, `num` in the second.

```text
ret 1:1-1:12
  value:
    call 1:8-1:12
      args:
        operand 1:10-1:11
          v:
            name 1:10-1:11
              id = "x" 1:10
      fname = "f" 1:8
```

```text
ret 1:1-1:12
  value:
    call 1:8-1:12
      args:
        operand 1:10-1:11
          v:
            num 1:10-1:11
              n = "x" 1:10
      fname = "f" 1:8
```

```text
   call | name | num            call | num | name
   ───────────────────           ───────────────────
   x: call ❌                    x: call ❌
      name ✅  STOP                 num  ✅  STOP     name is never tried
      num  (never tried)
```

---

### U-04 — a failed choice names every alternative

```sh
cat samples/expression-grammar/broken.capy
$CAPY run samples/expression-grammar/lib.capy samples/expression-grammar/broken.capy; echo "exit $?"
$CAPY ast samples/expression-grammar/lib.capy samples/expression-grammar/broken.capy; echo "exit $?"
```

**Expected:**

```text
return add(3, +)
error: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
  1 │ return add(3, +)
    │               ^
exit 1
<error> 1:1-1:17  8 token(s) skipped
error[E0001] 1:1: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
exit 1
```

All three alternatives are listed, in declaration order, at the furthest failure.
`capy ast` also reports the region it skipped and still exits 1.

---

### U-05 — left recursion through a later alternative is refused

Write a library where the recursion hides in the **second** alternative. `atom`
is a `bare` function with `arg literal "x"`:

```sh
cat > $W/leftrec.capy <<'EOF'
extension txt
function expr
    bare
    arg capture e atom | expr
end
function atom
    bare
    arg literal "x"
end
EOF
$CAPY check $W/leftrec.capy; echo "exit $?"
```

**Expected** (a single line; wrapped here for reading):

```text
function "expr": left recursion — it can match itself without consuming a token (cycle: expr -> expr). Rewrite the rule so something is consumed first: put a literal before the capture, or make the recursion trail (right-recursive) instead of lead
exit 1
```

The guard adds an edge to **every** alternative, so `expr -> expr` is found even
though the first alternative (`atom`) consumes `x`. Putting `expr` first
(`expr | atom`) is refused with the same message. The sample grammar is
right-recursive and loads:

```sh
$CAPY check samples/expression-grammar/lib.capy; echo "exit $?"
```

```text
ok — 5 function(s), 0 type(s)
  function call
  function name
  function num
  function operand
  function ret
exit 0
```

---

### U-06 — `capy docs` prints the union

```sh
$CAPY docs samples/expression-grammar/lib.capy
```

**Expected** (excerpt — the `operand` section):

````text
### `operand`

```
<v>
```

| Argument | Type | Description |
|---|---|---|
| `v` | `call \| name \| num` | *(no description)* |
````

and, for the mixed-parameters library:

```sh
$CAPY docs samples/mixed-parameters/lib.capy | grep -n "mut_param"
```

```text
22:| `ps` | `mut_param | plain_param` | *(no description)* |
24:### `mut_param`
```

Every alternative is shown, not only the first. The repetition suffix (`*`) is
not part of the printed type in either library.

---

### U-07 — the tree names the matched alternative

```sh
$CAPY ast --json samples/expression-grammar/lib.capy $W/n.capy \
  | jq -c '[.. | objects | select(has("func")) | .func], .schema_version'
$CAPY ast --json samples/mixed-parameters/lib.capy $W/m.capy \
  | jq -c '[.. | objects | select(has("func")) | .func]'
```

**Expected:**

```text
["ret","call","operand","num","operand","call","operand","num","operand","num"]
1
["def_fn","mut_param","plain_param"]
```

`schema_version` is still `1`: no field was added. `sub[].func` already carried
the function name; with ordered choice it is the winner's name.

---

### U-08 — a flat built-in type is refused as an alternative

```sh
cat > $W/flat.capy <<'EOF'
extension txt
function call
    arg literal "call"
    arg capture v call | int
end
EOF
$CAPY check $W/flat.capy; echo "exit $?"
```

**Expected:**

```text
function "call": capture "v" alternation names "int", which is not a library function
exit 1
```

Recovery: wrap the flat type in a `bare` function with one capture (`function
num / bare / arg capture n int`) and name that function. A malformed choice is
refused at parse time too:

```sh
printf 'extension txt\nfunction a\n bare\n arg capture v b || c\nend\n' > $W/bad.capy
$CAPY check $W/bad.capy
```

```text
line 4: arg capture: malformed alternation "b || c" — write `A | B | C`, each a function name (a repetition suffix `*` or `+` goes after the last name and applies to the whole choice)
```

---

### U-09 — `capy ast` recovers (carried, PROP-2026-0002)

```sh
$CAPY ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy; echo "exit $?"
diff <($CAPY ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy 2>&1) \
     samples/parse-recovery/broken.expected-ast.txt && echo MATCH-GOLDEN
```

**Expected:**

```text
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
exit 1
MATCH-GOLDEN
```

---

### U-10 — operator precedence (carried)

```sh
$CAPY run samples/operator-precedence/lib.capy samples/operator-precedence/script.capy
diff <($CAPY run samples/operator-precedence/lib.capy samples/operator-precedence/script.capy) \
     samples/operator-precedence/script.expected.txt && echo MATCH
```

**Expected:** each `let` echoes twice, source then evaluated; the first lines
are:

```text
ARITHMETIC — `*` `/` `%` bind tighter than `+` `-`
let a = 1 + 2 * 3
      a == 7
let b = (1 + 2) * 3
      b == 9
let c = 10 - 2 - 3
      c == 5
```

and the run ends with `MATCH` (output identical to the golden). The last block
reads `h == false`, `i == true`, `j == false`.

---

### U-11 — the language front-end sample (carried)

```sh
$CAPY run samples/language-frontend/lib.capy samples/language-frontend/main.capy; echo "exit $?"
$CAPY run samples/language-frontend/lib.capy samples/language-frontend/broken.capy; echo "exit $?"
$CAPY ast samples/language-frontend/lib.capy samples/language-frontend/broken.capy 2>&1 | tail -4
```

**Expected:**

```text
def gcd(a: int, b: int):
    while b != 0:
        t = b
        b = a % b
        a = t
    return a

def classify(n: int, limit: int):
    score = n * 2 + 1
    if score > limit and n != 0:
        return 1
    return 0

exit 0
error: expected `)`, found "{" in `fn`
  1 │ fn add(x: int, y: int {
    │                       ^
exit 1
<error> 3:1-3:2  1 token(s) skipped
error[E0001] 1:1: expected `)`, found "{" in `fn`
error[E0001] 3:1: no library function matches token "}"
```

The `capy ast` output matches `broken.expected-ast.txt` byte for byte
(`diff` printed nothing). Note: `broken.expected-error.txt` records the `run`
error in the one-line `1:23: expected …` form the golden harness uses, not the
caret block the CLI prints; the golden test passes (`cargo test --test golden`,
inside U-13).

---

### U-12 — tutorial 05 reproduces (carried)

Run each command the tutorial gives, against `samples/parse-recovery/`:

```sh
L=samples/parse-recovery/lib.capy
$CAPY run $L samples/parse-recovery/script.capy
$CAPY ast $L samples/parse-recovery/script.capy; echo "exit $?"
$CAPY run $L samples/parse-recovery/broken.capy; echo "exit $?"
$CAPY ast $L samples/parse-recovery/broken.capy --json 2>&1 \
  | jq -r '.diagnostics[] | "\(.primary.start_line):\(.primary.start_col) \(.code) \(.message)"'
```

**Expected:**

```text
[ ] write the proposal (due 2026-09-16)
[x] read the release notes
[ ] ship the samples (due 2026-09-30)
--- end of list ---
task 1:1-1:43
  name = "\"write the proposal\"" 1:6
  when = "2026-09-16" 1:31
done 2:1-2:30
  name = "\"read the release notes\"" 2:6
task 3:1-3:41
  name = "\"ship the samples\"" 3:6
  when = "2026-09-30" 3:29
list 4:1-4:5
exit 0
error: expected `due`, found end of statement in `task`
  2 │ task "missing its due date"
exit 1
2:1 E0001 expected `due`, found end of statement in `task`
4:1 E0001 no library function matches token "tsak"
```

Step 6 of the tutorial (fix and confirm): add the missing `due` date to line 2 and
change `tsak` to `task` with a date; `capy ast` then prints five `task`/`done`/
`list` nodes, no `<error>` line, and exits `0`. This was executed with a corrected
copy of `broken.capy`, giving `exit 0`.

---

### U-13 — nothing existing changed

```sh
cd rust && cargo test --workspace 2>&1 | grep -E "^test result"
```

**Expected:** every line says `ok` with `0 failed`. Executed result (thirteen
binaries and doc-test groups ran tests; three further binaries ran 0 tests):

```text
test result: ok. 9 passed; 0 failed     (capy-cli unit)
test result: ok. 21 passed; 0 failed    (capy-core unit)
test result: ok. 18 passed; 0 failed    (tests/alternation.rs)
test result: ok. 13 passed; 0 failed    (tests/ast_spans.rs)
test result: ok. 7 passed; 0 failed     (tests/define_extractor.rs)
test result: ok. 14 passed; 0 failed    (tests/diagnostics.rs)
test result: ok. 15 passed; 0 failed    (tests/embed.rs)
test result: ok. 2 passed; 0 failed     (tests/golden.rs — the whole sample corpus)
test result: ok. 6 passed; 0 failed     (tests/precedence.rs)
test result: ok. 11 passed; 0 failed    (tests/preprocessor.rs)
test result: ok. 9 passed; 0 failed     (tests/recursion_guard.rs)
test result: ok. 1 passed; 0 failed     (capy-mcp)
test result: ok. 2 passed; 0 failed     (doc-tests capy_core)
```

That is **128 passed, 0 failed** across the workspace. `tests/golden.rs` walks
`samples/*/` and compares every script's output to its stored golden, which is
what makes "every pre-existing golden is byte-identical" (`PROP-2026-0004` R7)
a checked statement: the two new sample directories and every older one pass in
one run.

---

## Cleanup

```sh
rm -rf /tmp/capy-verify-23
```

No file in the repository is modified by any step.

## Known Caveats

- **The depth bound is not named.** At 32 call levels the error is
  ``expected `)`, found "1" in `call` `` — an ordinary shaped failure — not a
  message about nesting depth. The depth error is swallowed by the capture-local
  rewind that makes choice work, then surfaces as the generic expectation.
  Naming the limit remains roadmap work (see `docs/errors-and-debugging.md`).
- **Choice is among functions only.** A built-in or declared type must be wrapped
  in a `bare` function (U-08).
- **Fixed during verification:** `capy docs` first printed the choice with bare `|` inside a
  Markdown table cell, which a strict renderer reads as column separators. The renderer
  now escapes them (`rust/src/domain/docs.rs`); `docs_print_the_union` asserts the escaped form.
- **The wasm introspection JSON does not carry `alts`.** `capyIntrospect` emits
  `type` only; the Rust `ArgInfo.alts` field is the public surface.
- `capy version` printed `capy dev` at verification time (see Verified Against
  Version).

## Troubleshooting

| Symptom | Cause |
|---|---|
| U-01 prints ``expected `)`, found "(" in `call` `` | Pre-0.23.0 binary, or a grammar where `operand` still names one function |
| U-02 prints ``expected `mut`, found "n" `` for the *shipped* sample | The binary predates choice, or `mut_param \| plain_param` was edited to a single name |
| U-03 gives `name` for both orders | `sed` did not match; check the library line reads `call \| num \| name` |
| U-05 prints `ok` | Pre-0.23.0 binary: a cycle through a later alternative was not checked |
| `jq: command not found` (U-07, U-12) | Install `jq`, or read `sub[].func` from `capy ast` text output as in U-03 |
| U-13 reports a failure in `golden` | A sample or its golden changed; run `cargo test --test golden` and read the diff |

## Verification Record

Executed 2026-10-07 by Olivier's release agent on macOS arm64 (Darwin 25.4.0),
debug profile, working tree on top of `84f984c`.

| Step | Verified By | Verified At | Result |
|---|---|---|---|
| U-01 nested calls (run, ast, glued, depth 31/32) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-02 mixed parameters (run, ast, single-shape contrast) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-03 order observable (`name` vs `num`) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-04 union diagnostic (`run` and `ast`) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-05 left recursion refused; sample loads | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-06 `capy docs` union | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS — after escaping the pipes in the Markdown table cell (see Known Caveats) |
| U-07 `sub[].func` discriminator, `schema_version` 1 | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-08 built-in alternative refused; malformed choice refused | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-09 parse-recovery sample | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-10 operator-precedence sample | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-11 language-frontend sample | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-12 tutorial 05 | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-13 `cargo test --workspace` | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS — 128 passed, 0 failed |

## Limitations

- Verified on the debug profile and the pre-bump working tree. Performance and
  wasm-size claims are not exercised here; they belong to the measurements in the
  plan (`M-01` … `M-04`).
- U-12 step 6 is verified by running the fix, not by quoting new output.
- Only the diagnostics path of recovery is exercised for U-09 and U-11; `capy
  run` still refuses to emit when a region failed.

## Actual Verification Status

```text
  U-01  PASS      U-06  PASS      U-11  PASS
  U-02  PASS      U-07  PASS      U-12  PASS
  U-03  PASS      U-08  PASS      U-13  PASS
  U-04  PASS      U-09  PASS
  U-05  PASS      U-10  PASS      13 PASS · 0 PARTIAL · 0 FAIL
```

## Related Documents

- `PLAN-2026-0003` — the implementation plan for this release
- `RPT-2026-0003` — the validation report for this release
- `PROP-2026-0004` — ordered alternation; `PROP-2026-0002` — public docs and samples
- `ADR-0003` — the decision that alternatives are library functions
- `MAN-2026-0002` — the manual chapter whose procedures these steps verify
- `SYS-2026-0001` — the parser pipeline as implemented
- `DEMO-2026-0002` — the 0.22.0 guide this one follows

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial guide; thirteen updates executed against the 0.23.0 working tree |
