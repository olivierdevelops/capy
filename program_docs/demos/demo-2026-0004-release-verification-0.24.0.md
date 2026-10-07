---
document_id: DEMO-2026-0004
title: Release Verification Guide — 0.24.0
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
  - capy-wasm-abi
  - docs

affected_versions:
  from: "0.24.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - operators
  - technical-writers

scope: An executable verification procedure for every update released in 0.24.0 — the named nesting bound (E0003), the `alts` field in the browser introspection JSON, and a `capy version` that identifies the build (PROP-2026-0005) — written for a reader who did not implement the change.

reason: DOCUMENTATION.md section 29 makes a release verification guide mandatory for every release.

related_documents:
  - PLAN-2026-0004
  - PROP-2026-0005
  - ADR-0004
  - RPT-2026-0004
  - MAN-2026-0002
  - SYS-2026-0001
  - TRBL-2026-0001
  - DEMO-2026-0003

supersedes: null
superseded_by: null

tags:
  - demo
  - verification
  - release
  - diagnostics
  - alternation

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Release Verification Guide — 0.24.0

> **Status:** Active
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.24.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli, capy-wasm-abi, docs

## Summary

Eight released updates, each verifiable from a shell or from `cargo`. They close
the three follow-ups that `REL-0.23.0` left open: a statement that dies against
the 64-capture nesting bound now says so (U-01 to U-04); the browser
introspection JSON carries the choice (U-05); and `capy version` prints the
crate version, so a stale binary can be told apart (U-06). U-07 is the
regression run; U-08 checks that the documentation builds.

```text
  WHAT 0.24.0 CHANGES                              WHERE YOU VERIFY IT
  ──────────────────────────────────────────────   ──────────────────────
  depth bound named, code E0003                    U-01 .. U-04   (engine)
  wasm capy_introspect args carry "alts":[…]       U-05           (wasm ABI)
  capy version prints `capy 0.24.0`, not `dev`     U-06           (CLI)
  no existing library changed                      U-07           (regression)
  documentation true and building                  U-08           (docs)

  NOT changed: the bound itself (64 captures = 31 call levels).      U-03
```

## Release Identity

| Field | Value |
|---|---|
| Version | 0.24.0 |
| Tag | `v0.24.0` (created at release; the commit hash is recorded in `REL-0.24.0`, which is written after the tag) |
| Plan | PLAN-2026-0004 |
| Proposal | PROP-2026-0005 |
| Decision | ADR-0004 |
| Validation | RPT-2026-0004 (per the plan; written in phase P4) |
| Commits | `a046032` (plan documents), `8769f51` (the fix), `cb23972` (version bump) |

## Purpose

Show, with output a reader can compare byte for byte, that each user-visible
change in 0.24.0 does what its inciting requirement asked.

## Verified Against Version

0.24.0, on `main` at `cb23972` (the version bump), which is the tree the tag
`v0.24.0` is to be created on. `git status --short` printed nothing before the
commands were run. The CLI was built with `cd rust && cargo build --workspace`
and reports `capy 0.24.0` (U-06), so unlike the 0.23.0 guide the binary under
test can be identified by its own version string.

**Process note (not hidden).** The code in `8769f51` was first drafted before
`PROP-2026-0005` existed, then set aside; the proposal, ADR and plan were
committed (`a046032`), and the code was re-applied from the drafted change. The
tests and this guide were produced after the proposal. The deviation is recorded
in `PLAN-2026-0004`, `RPT-2026-0004` and `REL-0.24.0`.

## Prerequisites

```sh
cd rust && cargo build --workspace && cd ..          # NOT plain `cargo build`; see Troubleshooting
export CAPY=$PWD/rust/target/debug/capy
export W=/tmp/capy-verify-24 && mkdir -p $W
command -v jq mkdocs                                 # U-04 needs jq, U-08 needs mkdocs
$CAPY version                                        # must print: capy 0.24.0
```

All commands run from the repository root. Scratch files go under `$W`, never in
the repo. The commands below were executed with `$W` set to the session scratch
directory; nothing printed depends on that path.

If `$CAPY version` does not print `capy 0.24.0`, stop: you are testing another build.

## Setup

One helper writes a source line of `n` nested calls around a number:

```sh
nest(){ s="1"; for i in $(seq 1 $1); do s="f($s)"; done; echo "return $s"; }
```

Every library used is already in the repository (`samples/expression-grammar/`).

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
| U-01 | `PROP-2026-0005` R1 / UC-01 | A statement that hits the nesting bound says so | § U-01 | ``nesting too deep (limit 64) while matching "call \| name \| num" — …`` for a 70-deep and a 40-deep nest | `rust/tests/alternation.rs` `depth_bound_is_reported_as_e0003` |
| U-02 | `PROP-2026-0005` R2 / UC-01 | `capy ast` reports it as `E0003`, exit 1 | § U-02 | `error[E0003] 1:1: nesting too deep …` | same test; golden corpus |
| U-03 | `PROP-2026-0005` R4 / UC-01 negative path | The bound did not move: 31 levels parse, 32 are refused | § U-03 | exit `0` at 31; the new message and exit `1` at 32 | `thirty_one_levels_still_parse` |
| U-04 | `PROP-2026-0005` R3 / UC-02 | A bound error does not leak into the next statement | § U-04 | diagnostics `E0003` (line 2) then `E0001` (line 3) | `depth_error_does_not_leak_into_the_next_statement` |
| U-05 | `PROP-2026-0005` R5 / UC-03 | The wasm `capy_introspect` JSON carries `"alts":[…]` | § U-05 | `cargo test -p capy-wasm-abi`: 2 passed | `introspect_json_carries_alts`, `introspect_json_alts_is_empty_for_a_plain_capture` |
| U-06 | `PROP-2026-0005` R6 / UC-04 | `capy version` and `--version` print the crate version | § U-06 | both print `capy 0.24.0` | `rust/cli/tests/version.rs` (2 tests) |
| U-07 | `PROP-2026-0005` R7 | No existing library changes behaviour | § U-07 | `cargo test --workspace`: 135 passed, 0 failed; `goldens: 131 passed, 8 skipped (no golden file), 0 failed` | cargo test summary below |
| U-08 | `PROP-2026-0005` R8 | The documentation states the new behaviour and builds | § U-08 | `mkdocs build --strict` exit `0` | mkdocs output |

The journey every engine update shares:

```text
 source line:  return f(f(f( … 70 levels … 1)))
       │
       ▼
 [parse_stmt]  depth_err = None, fail_code = E0001           (reset at entry)
       │
       ▼
 [capture_func_type] depth ≥ 64 ?  ── yes ─► depth_err = "nesting too deep …"   (first one kept)
       │ no                                      │
       ▼                                         ▼
 [match_one] rewinds the failed alternative   the rewind still swallows the Err,
       │                                      but depth_err survives it
       ▼
 no shape matched
       │  depth_err set ? ── yes ─► return it, fail_code = E0003      (U-01, U-02)
       │  no
       ▼
 furthest-failure error, fail_code = E0001                          (U-04, second statement)
```

---

### U-01 — the depth bound is named

```sh
nest 70 > $W/d70.capy; nest 40 > $W/d40.capy
for n in 70 40; do
  $CAPY run samples/expression-grammar/lib.capy $W/d$n.capy >/dev/null 2>$W/d$n.err
  echo "exit $?"; head -1 $W/d$n.err
done
```

**Expected:**

```text
exit 1
error: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
exit 1
error: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
```

Both nests are refused with the same message. A 40-deep nest is *over* the
bound, not under it: the bound is 64 captures, which is 31 call levels in this
grammar (U-03). The message names the limit and the alternatives that were being
matched. `capy run` also prints the source line and a caret under the token where
the bound was reached; only the first line is shown above (`head -1`).

```text
   before 0.24.0:  error: expected `)`, found "1" in `call`          (names the innermost call, not the limit)
   0.24.0:         error: nesting too deep (limit 64) while matching "call | name | num" — …
```

The "before" line is the observation recorded in `REL-0.23.0` and
`DEMO-2026-0003` U-01, not re-run here.

---

### U-02 — `capy ast` reports code E0003

```sh
$CAPY ast samples/expression-grammar/lib.capy $W/d70.capy; echo "exit $?"
$CAPY ast --json samples/expression-grammar/lib.capy $W/d70.capy | jq -c '.diagnostics[] | {code, message}'
```

**Expected:**

```text
<error> 1:1-1:219  213 token(s) skipped
error[E0003] 1:1: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
exit 1
{"code":"E0003","message":"nesting too deep (limit 64) while matching \"call | name | num\" — the source nests further than the parser will follow"}
```

`capy ast` recovers: the whole statement is one `<error>` region, there is exactly
one diagnostic, its code is `E0003` (reserved but unemitted before 0.24.0), and the exit
status is `1`.

---

### U-03 — the bound did not move

```sh
nest 31 > $W/d31.capy; nest 32 > $W/d32.capy
$CAPY run samples/expression-grammar/lib.capy $W/d31.capy > $W/d31.out; echo "exit $?"; wc -c < $W/d31.out
$CAPY run samples/expression-grammar/lib.capy $W/d32.capy >/dev/null 2>$W/d32.err; echo "exit $?"; head -1 $W/d32.err
```

**Expected:**

```text
exit 0
     103
exit 1
error: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
```

31 call levels still parse and emit the 103-byte line
`return f(f(…f(1)…));` (the output ends `))))))))))))))))));`); 32 are refused. This
is the same boundary `DEMO-2026-0003` U-01 measured, now with a named error
instead of ``expected `)`, found "1" in `call` ``.

```text
   levels:   … 30   31 │ 32   33 …
                  parse ✅ │ ❌ nesting too deep (E0003)
                          └─ 64 captures
```

---

### U-04 — an ordinary error after a deep statement is still E0001

```sh
{ echo 'return add(1, 2)'; nest 40; echo 'return add(3, +)'; } > $W/seq.capy
$CAPY ast samples/expression-grammar/lib.capy $W/seq.capy; echo "exit $?"
$CAPY ast --json samples/expression-grammar/lib.capy $W/seq.capy \
  | jq -r '.diagnostics[] | "\(.primary.start_line):\(.primary.start_col) \(.code) \(.message)"'
```

**Expected** (the first statement, `return add(1, 2)`, is a valid tree):

```text
ret 1:1-1:17
  value:
    call 1:8-1:17
      args:
        operand 1:12-1:13
          v:
            num 1:12-1:13
              n = "1" 1:12
        operand 1:15-1:16
          v:
            num 1:15-1:16
              n = "2" 1:15
      fname = "add" 1:8
<error> 2:1-2:129  123 token(s) skipped
<error> 3:1-3:17  8 token(s) skipped
error[E0003] 2:1: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
error[E0001] 3:1: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
exit 1
2:1 E0003 nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
3:1 E0001 expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
```

```text
   line 1  return add(1, 2)        ✅ tree
   line 2  return f(f(…40…1))      ❌ E0003   depth_err set
   line 3  return add(3, +)        ❌ E0001   depth_err cleared at parse_stmt entry
```

The third line fails for its own reason (`+` is not a `call`, `name` or `num`), not
because line 2 reached the bound.

---

### U-05 — the wasm introspection JSON carries `alts`

The browser entry point `capy_introspect` is a WebAssembly export and cannot be
called from a shell. Its JSON writer is covered by two unit tests in the
`capy-wasm-abi` crate:

```sh
cd rust && cargo test -p capy-wasm-abi 2>&1 | grep -E "^test |test result"
```

**Expected:**

```text
test tests::introspect_json_alts_is_empty_for_a_plain_capture ... ok
test tests::introspect_json_carries_alts ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

What each asserts (read from `rust/wasm/src/lib.rs`): for a library declaring
`arg capture v call | name | num` the JSON contains
`"type":"call","alts":["name","num"]`; for a plain `ident` capture it contains
`"type":"ident","alts":[]`. `type` keeps meaning alternative 1. Loading the
`.wasm` in a browser is not exercised here (see Limitations).

---

### U-06 — `capy version` identifies the build

```sh
$CAPY version; $CAPY --version
[ "$($CAPY version)" = "$($CAPY --version)" ] && echo AGREE
```

**Expected:**

```text
capy 0.24.0
capy 0.24.0
AGREE
```

Neither prints `capy dev`. The value comes from `CARGO_PKG_VERSION` when the
`CAPY_VERSION` environment variable was not set at compile time; a release build
that stamps `CAPY_VERSION` prints the stamp (`rust/cli/src/main.rs`, `VERSION`).
The unstamped case is the one verified here; the stamped case follows the comment
in `main.rs` ("CI stamps the release tag") and is not run in this guide.

---

### U-07 — nothing existing changed

```sh
cd rust && cargo test --workspace 2>&1 | grep -E "^test result"
cd rust && cargo test --test golden -- --nocapture 2>&1 | grep "goldens:"
```

**Expected:** every line says `ok` with `0 failed`. Executed result, per binary
(the fifteen result lines that ran tests are listed; the remaining binaries ran 0 tests):

```text
test result: ok. 9 passed; 0 failed     (capy-cli unit)
test result: ok. 2 passed; 0 failed     (capy-cli tests/version.rs — new)
test result: ok. 21 passed; 0 failed    (capy-core unit)
test result: ok. 21 passed; 0 failed    (tests/alternation.rs — was 18, +3 new)
test result: ok. 13 passed; 0 failed    (tests/ast_spans.rs)
test result: ok. 7 passed; 0 failed     (tests/define_extractor.rs)
test result: ok. 14 passed; 0 failed    (tests/diagnostics.rs)
test result: ok. 15 passed; 0 failed    (tests/embed.rs)
test result: ok. 2 passed; 0 failed     (tests/golden.rs — the whole sample corpus)
test result: ok. 6 passed; 0 failed     (tests/precedence.rs)
test result: ok. 11 passed; 0 failed    (tests/preprocessor.rs)
test result: ok. 9 passed; 0 failed     (tests/recursion_guard.rs)
test result: ok. 1 passed; 0 failed     (capy-mcp)
test result: ok. 2 passed; 0 failed     (capy-wasm-abi — new)
test result: ok. 2 passed; 0 failed     (doc-tests capy_core)
goldens: 131 passed, 8 skipped (no golden file), 0 failed
```

That is **135 passed, 0 failed** (summed from the lines above), against 128 at
0.23.0: three new engine tests, two version tests and two wasm tests. The
`golden` harness compares every sample's output to its stored golden, which makes
"every pre-existing golden is byte-identical" (`PROP-2026-0005` R7) a checked
statement.

---

### U-08 — the documentation is true and builds

```sh
mkdocs build --strict -d $W/site; echo "exit $?"
```

**Expected:** `exit 0`. The run printed a number of `INFO` lines (pages not in the
nav, and one anchor `INFO` in `features.md` that is not part of this release);
`--strict` fails on warnings only, and there were none. The pages carrying the
new behaviour are `docs/diagnostics.md` (`E0003` now emitted), `docs/errors-and-debugging.md`,
`docs/library-authoring.md`, `docs/cli.md`, `docs/embedding.md`, `docs/roadmap.md`,
`docs/language-frontend.md`, `docs/whats-new.md` and `CHANGELOG.md`.

---

## Cleanup

```sh
rm -rf /tmp/capy-verify-24
```

No file in the repository is modified by any step.

## Known Caveats

- **The bound is still 64 captures.** Only its message changed. A source that nests
  deeper than 31 calls in the sample grammar is refused, as before.
- **Choice is among functions only** (carried from 0.23.0; `DEMO-2026-0003` U-08).
- **`E0002` is still reserved and unemitted.** Only `E0003` became live in this release.
- **The version string for a CI build is the stamp, not the crate version.**
  U-06 verifies the unstamped path only.

## Troubleshooting

| Symptom | Cause |
|---|---|
| `capy version` prints `capy dev` or a version below 0.24.0 | A stale binary: a `capy` earlier on PATH, or `$CAPY` pointing at an old build. Run `which capy`; use `$PWD/rust/target/debug/capy` (`TRBL-2026-0001`) |
| `capy version` still prints the old version after you edited the CLI | A plain `cargo build` in `rust/` builds only the root package `capy-core`; the CLI is a workspace member and is not rebuilt (it prints `Finished` in about 0.02s). Use `cargo build --workspace` |
| U-01 prints ``expected `)`, found "1" in `call` `` | A pre-0.24.0 binary (see the two rows above) |
| U-02 prints `E0001` instead of `E0003` | Pre-0.24.0 binary, or the statement did not reach the bound |
| U-03 refuses 31 levels or accepts 32 | The bound was changed in the source tree; check `MAX_PARSE_DEPTH` in `rust/src/orchestrator/features/make_parser.rs` |
| U-04 shows `E0003` on line 3 | The reset at `parse_stmt` entry is missing; run `depth_error_does_not_leak_into_the_next_statement` |
| U-05 prints `0 passed` or no test lines | Wrong directory (run from `rust/`) or the wrong package name (`-p capy-wasm-abi`) |
| `jq: command not found` (U-02, U-04) | Install `jq`, or read the codes from the `capy ast` text output as shown |
| U-07 reports a failure in `golden` | A sample or its golden changed; run `cargo test --test golden` and read the diff |
| `mkdocs: command not found` (U-08) | Install `mkdocs` and `mkdocs-material` |

## Verification Record

Executed 2026-10-07 by Olivier's release agent on macOS arm64 (Darwin 25.4.0),
debug profile, `main` at `cb23972` with a clean working tree.

| Step | Verified By | Verified At | Result |
|---|---|---|---|
| U-01 depth bound named (70 and 40 levels) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-02 `capy ast` code `E0003`, exit 1 | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-03 31 levels parse, 32 refused | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-04 `E0003` then `E0001` | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-05 wasm `alts` (unit tests; browser not exercised) | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS — 2 passed |
| U-06 `capy version` / `--version` | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS |
| U-07 `cargo test --workspace`, golden corpus | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS — 135 passed, 0 failed; 131 goldens passed, 0 failed |
| U-08 `mkdocs build --strict` | Claude Sonnet 5.5 for Olivier | 2026-10-07 | PASS — exit 0 |

One expectation in the brief for this guide differed from what ran: a 40-deep nest
was expected to be an illustration of "deep", and it is refused too, because the
bound is 31 call levels. U-01 states this. No other output differed.

## Limitations

- Verified on the debug profile only. Performance and wasm-size claims are not
  exercised here; they belong to the measurements in the plan.
- U-05 verifies the JSON writer through its unit tests. The `.wasm` build loaded in
  a browser (`capyIntrospect`) was not run.
- U-06 verifies the unstamped build. A build with `CAPY_VERSION` set was not run.
- The tests and this guide were written after the proposal; the code predates it
  (see Verified Against Version).

## Actual Verification Status

```text
  U-01  PASS      U-05  PASS
  U-02  PASS      U-06  PASS
  U-03  PASS      U-07  PASS
  U-04  PASS      U-08  PASS      8 PASS · 0 PARTIAL · 0 FAIL
```

## Related Documents

- `PLAN-2026-0004` — the implementation plan for this release
- `RPT-2026-0004` — the validation report for this release
- `PROP-2026-0005` — the proposal; `ADR-0004` — the decision
- `MAN-2026-0002` — the manual chapter whose procedures these steps verify
- `SYS-2026-0001` — the parser pipeline as implemented
- `TRBL-2026-0001` — a stale `capy` on PATH and the plain-`cargo build` pitfall
- `DEMO-2026-0003` — the 0.23.0 guide this one follows

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial guide; eight updates executed against `main` at `cb23972` |
