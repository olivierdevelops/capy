---
document_id: MAN-2026-0002
title: Manual — Ordered Choice in a Capture Type
document_type: manual
status: active

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 3

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

affected_versions:
  from: "0.23.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - technical-writers

scope: The reader-facing chapter for ordered choice (`arg capture v call | name | num`) — when to use it, how to write it, how to read the tree and the errors it produces, and what it does not do.

reason: A library author must be able to declare a recursive or multi-shape grammar without reading the source, the proposal or the plan.

related_documents:
  - MAN-2026-0001
  - DEMO-2026-0003
  - DEMO-2026-0004
  - PLAN-2026-0003
  - PLAN-2026-0004
  - PROP-2026-0004
  - PROP-2026-0005
  - ADR-0003
  - ADR-0004
  - SYS-2026-0001

supersedes: null
superseded_by: null

tags:
  - manual
  - grammar
  - alternation

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-04-07
---

# Manual — Ordered Choice in a Capture Type

> **Status:** Active
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.23.0 and later (revised for 0.24.0)
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Summary

Capy gives a library author four grammar combinators. Three existed; 0.23.0 adds
the fourth.

```text
  ┌────────────┬──────────────────────────────────────────┬─────────┐
  │ sequence   │ arg literal / arg capture, in order       │ always  │
  │ repetition │ param*   param+   sep ","   join ", "     │ always  │
  │ recursion  │ a capture whose type is a function        │ 0.21.0  │
  │ CHOICE     │ arg capture v call | name | num           │ 0.23.0  │
  └────────────┴──────────────────────────────────────────┴─────────┘
```

## Purpose

Say "this argument is a nested call **or** a name **or** a number" in one line, so
a grammar can nest and a parameter list can mix shapes.

## Reading Order

```text
  MAN-2026-0001   spans, comments, recursion limits, errors, recovery
        │         (the first three combinators and how they fail)
        ▼
  MAN-2026-0002   ordered choice — the fourth combinator          <── you are here
        │
        ▼
  DEMO-2026-0003  run every procedure below and compare output
  DEMO-2026-0004  0.24.0: the named depth bound, `alts` in the browser, `capy version`
```

Read MAN-2026-0001 §3 (recursion limits) first: choice composes with recursion,
and the left-recursion rule it describes governs every alternative here.

## What's New in the Current Supported Release

| Addition (0.23.0) | Instructions | Verified demo |
|---|---|---|
| A capture type may name several functions, first match wins | [Nested calls](#task-write-a-grammar-that-nests) | DEMO-2026-0003 U-01, U-03 |
| A parameter list can mix marked and unmarked parameters | [Mixed parameters](#task-mix-marked-and-unmarked-parameters) | DEMO-2026-0003 U-02 |
| A failed choice lists every alternative | [Reading a failed choice](#task-read-a-failed-choice) | DEMO-2026-0003 U-04 |
| Left recursion through any alternative is refused at load | [Errors and recovery](#errors-and-recovery-reference) | DEMO-2026-0003 U-05 |
| `capy docs` prints the union; `ArgInfo.alts` | [Introspection](#task-inspect-a-choice) | DEMO-2026-0003 U-06 |
| `capy ast --json` names the matched alternative | [Reading the tree](#task-inspect-a-choice) | DEMO-2026-0003 U-07 |
| A built-in type is refused as an alternative | [Errors and recovery](#errors-and-recovery-reference) | DEMO-2026-0003 U-08 |
| **0.24.0** — a too-deep source is reported as `nesting too deep (limit 64) …`, code `E0003` under `capy ast` | [Errors and recovery](#errors-and-recovery-reference) | DEMO-2026-0004 U-01 to U-04 |
| **0.24.0** — the browser introspection JSON carries `alts` | [Introspection](#task-inspect-a-choice) | DEMO-2026-0004 U-05 |
| **0.24.0** — `capy version` prints the crate version (`capy 0.24.0`) | [Installation and setup](#installation-and-setup) | DEMO-2026-0004 U-06 |

History lives in `program_docs/releases/`; the user-facing summary is
[`docs/whats-new.md`](../../docs/whats-new.md).

## Project Goals and Boundaries

Capy ships **no** source grammar; the library declares every shape. Choice is a
combinator for library authors, not syntax the engine provides.

| In scope | Not in scope |
|---|---|
| Ordered, first-match-wins choice among library **functions** | Unordered or ambiguous choice |
| Choice inside `*` / `+` repetition with `sep` / `join` | Backtracking across a statement boundary |
| Introspection of the choice (`capy docs`, `ArgInfo.alts`) | A named union declared once (`nonterminal x = a \| b`) |
| | Built-in or declared types as alternatives |

## Concepts

```text
  arg capture  v  call | name | num
               │  └─────┬────────┘
               │        └─ the CHOICE: three library functions, tried in this order
               └─ the capture's name; `${v}` in a template is whichever one won
```

- **Alternative** — one function name in the choice. Alternative 1 is the capture's
  `type_`; alternatives 2…n are `alts`.
- **First match wins** — the matcher stops at the first alternative that matches
  and never looks at the rest. Order is meaning.
- **Rewind** — an alternative that consumes input and then fails hands the input
  back before the next one is tried. The rewind is local to this one capture.
- **Bare function** — a function with no leading keyword (`bare`). It is how a flat
  alternative (a name, a number) is written.

## Architecture and Mental Model

```text
   source tokens:   add ( 3 , mul ( 4 , 5 ) )

   call ──► fname=add  (  args: operand* sep ","  )
                              │
                              ▼
                        operand  v = call | name | num
                              │
              ┌───────────────┼────────────────┐
              ▼               ▼                ▼
        3: call ❌       3: call ❌       3: call ❌
           name ❌          name ❌    mul(4,5): call ✅ ─► recurses
           num  ✅
```

Recursion terminates because the recursive alternative (`call`) consumes a name
and a `(` **before** it descends — it is right-recursive. Descent is bounded at 64
captures regardless (about 31 call levels in the sample). A source nested past the
bound is refused with a message that names it (`E0003`, see Errors below).

Implementation detail is in `SYS-2026-0001`.

## Installation and Setup

Prerequisite: a `capy` binary built from 0.23.0 or later. Since 0.24.0, `capy version`
(and `capy --version`) print the crate version for a local build, so the build is
identifiable:

```sh
$CAPY version
```

```text
capy 0.24.0
```

A release build stamped with `CAPY_VERSION` prints the stamp instead. A 0.23.0 or earlier
build printed `capy dev` for every build, so on those, check behaviour instead: a library
with a cycle through a later alternative must be refused by `capy check`
(DEMO-2026-0003 U-05). See `TRBL-2026-0001` for a stale binary on PATH and for the plain
`cargo build` pitfall (use `cargo build --workspace`). Nothing else is required; the
feature is in the library grammar.

```sh
cargo build --release --manifest-path rust/Cargo.toml -p capy-cli
export CAPY=./rust/target/release/capy
```

## Feature Catalogue

| Feature | Why / When to Use It | Supported Surfaces | Since Version | Instructions | Demo |
|---|---|---|---|---|---|
| Ordered choice `A \| B \| C` | An argument can take more than one shape; recursive expression grammars | `.capy` library files; `Library::new` | 0.23.0 | [Nested calls](#task-write-a-grammar-that-nests) | DEMO-2026-0003 U-01 |
| Choice under repetition `A \| B*` | Argument lists whose items vary in shape | `.capy` library files | 0.23.0 | [Mixed parameters](#task-mix-marked-and-unmarked-parameters) | DEMO-2026-0003 U-02 |
| Union diagnostic | See every alternative that was expected | `capy run`, `capy ast`, `Library::parse` | 0.23.0 | [Reading a failed choice](#task-read-a-failed-choice) | DEMO-2026-0003 U-04 |
| Left-recursion guard over alternatives | A cycle hidden in a later alternative is caught at load | `capy check`, any library load | 0.23.0 | [Errors](#errors-and-recovery-reference) | DEMO-2026-0003 U-05 |
| Choice introspection | Tooling and docs show the whole choice | `capy docs`, `ArgInfo.alts` | 0.23.0 | [Introspection](#task-inspect-a-choice) | DEMO-2026-0003 U-06 |
| Matched-alternative discriminator | A tool needs to know which shape matched | `capy ast`, `capy ast --json` (`sub[].func`) | 0.23.0 | [Reading the tree](#task-inspect-a-choice) | DEMO-2026-0003 U-07 |

## Configuration and Environment Variables

NOT APPLICABLE — choice is declared in the library file. It adds no flag,
environment variable or setting.

## Task-Oriented Workflows

### Task: write a grammar that nests

**Why / when.** A call's argument can itself be a call. With a single-function
capture type that stops at one level; choice lets one rule say "call or atom".

**Prerequisites.** `capy` 0.23.0. A library with a call rule. **Inputs.** A
library file and a script.

#### Journey Overview

```text
[write alternatives as functions] -> [capy check] -> [capy run / ast] -> [output]
        │                                │                   │
        │                                └─ error: load ─────┴─ error: union ─┐
        └────────────── fix and re-run ◄───────────────────────────────────────┘
```

#### CLI Procedure

1. Declare each alternative as a function. Flat ones are `bare`:

```
function name
    bare
    arg capture id ident
    write `${id}`
end
function num
    bare
    arg capture n int
    write `${n}`
end
```

2. Declare the choice and the recursive alternative. `call` consumes a name and
   `(` before it recurses:

```
function operand
    bare
    arg capture v call | name | num
    write `${v}`
end
function call
    bare
    arg capture fname ident
    arg literal "("
    arg capture args operand* sep "," join ", "
    arg literal ")"
    write `${fname}(${args})`
end
```

3. Validate and run (`samples/expression-grammar/` is this library, complete):

```sh
$CAPY check samples/expression-grammar/lib.capy
printf 'return add(3, mul(4, 5))\n' > /tmp/n.capy
$CAPY run samples/expression-grammar/lib.capy /tmp/n.capy; echo "exit $?"
```

```text
ok — 5 function(s), 0 type(s)
  function call
  function name
  function num
  function operand
  function ret
return add(3, mul(4, 5));
exit 0
```

**Parameters and defaults.** The choice is `NAME TYPE1 | TYPE2 | …`, with spaces
around `|` or none (`call|name|num` is the same choice). A `*` or `+` goes after
the **last** name and applies to the whole choice. `sep`, `join` and `default`
follow as usual.

**Order matters.** `int` accepts a bare identifier at parse time, so `num` before
`name` would capture `x` as a number. List the narrower alternative first.

#### Expected Result and Side Effects

stdout carries the rendered output; exit `0`. No file is written and no state
outside the run changes.

#### Verified Demo

`DEMO-2026-0003` U-01 (nesting, glued spelling, depth) and U-03 (order),
verified 2026-10-07 against the 0.23.0 working tree.

---

### Task: mix marked and unmarked parameters

**Why / when.** A signature where only some parameters carry a marker
(`mut`, `ref`, `out`). A per-parameter rule that requires the marker on every
parameter rejects the rest.

```text
  def f(mut c: Counter, n: int)
        └ mut_param ┘  └ plain_param ┘      one list, two shapes
```

**Procedure.** Put the marked shape first (a plain shape that begins with a bare
`ident` would swallow `mut` as a parameter name):

```
arg capture ps mut_param | plain_param* sep "," join ", "
```

```sh
printf 'def f(mut c: Counter, n: int)\n' > /tmp/m.capy
$CAPY run samples/mixed-parameters/lib.capy /tmp/m.capy
```

```text
f(&c: Counter, n: int)
```

**Failure and recovery.** A library with a single shape in that position rejects
the same line; the fix is the choice above.

```text
error: expected `mut`, found "n" in `mut_param`
  1 │ def f(mut c: Counter, n: int)
    │                       ^
```

**Verified demo.** `DEMO-2026-0003` U-02.

---

### Task: read a failed choice

When no alternative matches, the diagnostic names all of them, in declaration
order, at the furthest failure.

```sh
$CAPY run samples/expression-grammar/lib.capy samples/expression-grammar/broken.capy; echo "exit $?"
```

```text
error: expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand`
  1 │ return add(3, +)
    │               ^
exit 1
```

`capy ast` on the same file recovers, reports the skipped region and the same
message as `E0001`, and exits `1`. **Recovery:** add a rule for the missing shape,
or fix the source.

**Verified demo.** `DEMO-2026-0003` U-04.

A source that nests **deeper than the parser will follow** is a different failure,
and since 0.24.0 it says so. Using a 70-deep `return f(f(…1…))`:

```sh
$CAPY ast samples/expression-grammar/lib.capy deep.capy; echo "exit $?"
```

```text
<error> 1:1-1:219  213 token(s) skipped
error[E0003] 1:1: nesting too deep (limit 64) while matching "call | name | num" — the source nests further than the parser will follow
exit 1
```

**Recovery:** flatten the source (31 call levels is the most this grammar accepts) or
restructure the grammar. An ordinary error in the *next* statement is still `E0001`
(`DEMO-2026-0004` U-04).

---

### Task: inspect a choice

```sh
$CAPY docs samples/expression-grammar/lib.capy | grep 'call .| name'
$CAPY ast --json samples/expression-grammar/lib.capy /tmp/n.capy \
  | jq -c '[.. | objects | select(has("func")) | .func]'
```

```text
| `v` | `call \| name \| num` | *(no description)* |
["ret","call","operand","num","operand","call","operand","num","operand","num"]
```

- `capy docs` prints every alternative in the Type column.
- In `capy ast --json`, `sub[].func` is the **winning** alternative's function name.
  `schema_version` stays `1`; no field was added.
- From Rust, `ArgInfo.type_` is alternative 1 and `ArgInfo.alts` is alternatives
  2…n (empty for a single-type capture), so existing code reading `type_` keeps
  working.

```text
   ArgInfo for  v call | name | num
   ┌────────────────────────────┐
   │ type_ = "call"             │
   │ alts  = ["name", "num"]    │
   └────────────────────────────┘
```

**Verified demo.** `DEMO-2026-0003` U-06, U-07.

## Complete CLI Reference

| Command / Flag | Purpose and When to Use | Syntax / Type / Default | Inputs | Output / Exit Codes | Errors | Example | Since |
|---|---|---|---|---|---|---|---|
| `capy check <lib>` | Validate a library, including every alternative | no new flag | library file | `ok — …` and `0`, or a message and `1` | see Errors reference | task 1 above | choice: 0.23.0 |
| `capy run <lib> <script>` | Transpile | no new flag | library, script | output and `0`; error and `1` | union diagnostic | task 1 above | choice: 0.23.0 |
| `capy ast <lib> <script> [--json]` | Show the tree; JSON for tools | no new flag | library, script | tree on stdout, diagnostics on stderr; `1` if any diagnostic | union `E0001` | task "inspect" | choice: 0.23.0 |
| `capy docs <lib>` | Print the library reference | no new flag | library | Markdown on stdout | — | task "inspect" | union: 0.23.0 |

## Complete API and Event Reference

| Method / Route / Event | Purpose | Auth | Request | Success Response | Errors / Status | Idempotency / Side Effects | Example | Since |
|---|---|---|---|---|---|---|---|---|
| `Library::introspect()` → `ArgInfo.alts: Vec<String>` | Read the declared choice | none | a loaded library | `alts` populated for a choice; empty otherwise | — | none | task "inspect" | 0.23.0 |

HTTP and event surfaces: NOT APPLICABLE — Capy exposes none for this feature.
Since 0.24.0 the browser build's `capyIntrospect` JSON carries `alts` after `type` on
every capture (`"type":"call","alts":["name","num"]`; `"alts":[]` for a plain capture),
verified by the wasm crate's unit tests (`DEMO-2026-0004` U-05).

## UI Screen and Interaction Reference

NOT APPLICABLE — a command-line and library feature with no screen.

## Errors and Recovery Reference

All messages below were produced by the 0.23.0 and 0.24.0 binaries (DEMO-2026-0003, DEMO-2026-0004).

| Error / Code / Message | Surface | Cause | User-Visible Result | Recovery | Retry Safe | Related Feature |
|---|---|---|---|---|---|---|
| ``expected a `call`, a `name`, or a `num`, found "+" in argument `v` of `operand` `` (`E0001` under `capy ast`) | `run`, `ast` | No alternative matched | exit `1`; `ast` still prints the recovered tree | Add the missing shape or fix the source | yes | Union diagnostic |
| `function "expr": left recursion — it can match itself without consuming a token (cycle: expr -> expr). Rewrite …` | `check`, any load | A cycle exists through **any** alternative before a token is consumed | library refused, exit `1` | Put a literal before the capture, or make the recursion trail | yes | Guard |
| `function "call": capture "v" alternation names "int", which is not a library function` | `check`, any load | An alternative is a built-in or declared type, or unknown | library refused, exit `1` | Wrap the type in a `bare` one-capture function and name that | yes | Alternatives are functions |
| `line 4: arg capture: malformed alternation "b \|\| c" — write \`A \| B \| C\` …` | `check`, any load | An empty or non-identifier name in the choice | library refused | Write `A \| B \| C` | yes | Syntax |
| ``nesting too deep (limit 64) while matching "call \| name \| num" — the source nests further than the parser will follow`` (`E0003` under `capy ast`) at 32 call levels | `run`, `ast`, `Library::parse` | The 64-capture nesting bound was reached; the message names the bound and the alternatives (0.24.0) | exit `1`, no crash; `ast` prints one `<error>` region | Flatten the source or the grammar | yes | Depth bound |
| ``expected `)`, found "1" in `call` `` at 32 call levels | `run`, `ast` | The same bound **before 0.24.0**: its message was consumed by the rewind | exit `1` | Upgrade; `capy version` shows the build | yes | Depth bound |

## Examples and Demos

`samples/expression-grammar/` (nested calls, 4 goldens) and
`samples/mixed-parameters/` (marker mix, 2 goldens). Both run in the golden suite.
The step-by-step verified procedures are `DEMO-2026-0003`.

## Operations, Observability and Maintenance

No daemon, no state. `capy check` in CI catches a bad choice at load. `capy docs`
regenerated and diffed in CI shows when an alternative was added or reordered.

## Edge Cases

| Case | Behaviour |
|---|---|
| An alternative matches but consumes nothing | Treated as a failure and rewound; the next alternative is tried |
| Repetition ends because no alternative matches | Not an error; the list simply stops. Existing diagnostics are unchanged |
| `+` with zero matches | An error listing every alternative |
| `a\|b` with no spaces | The same choice as `a \| b` |
| The `\|` in source (`\|>`) | Unaffected — `\|` is the choice token only in a capture type of a library file |

## Failure Modes, Recovery and Rollback

A bad choice is caught at load, not at run time, and never aborts the process.
Rollback is removing the `|`: a library without it behaves exactly as at 0.22.0.

## Security and Compatibility

No new execution surface, I/O or dependency. Compatibility: `|` in a capture type
was a load error before 0.23.0, so no existing library can contain it; every
pre-existing golden is byte-identical (DEMO-2026-0003 U-13). `ArgInfo` gained a
public field; `type_` is unchanged.

## Limitations

| Limitation | Consequence |
|---|---|
| Alternatives are library functions only | Wrap a built-in type in a `bare` function |
| First match wins, no ambiguity detection | A permissive early alternative hides later ones; order carefully |
| No named union | Repeat the `A \| B` list where it recurs; revisit if unions repeat (PROP-2026-0004 OQ-04) |
| The depth bound (64 captures, 31 call levels in the sample) is fixed | A deeper source is refused with `nesting too deep` (`E0003`); it cannot be raised from a library |
| Optional captures before a block opener and `} else {` continuation | Not part of this release; on the roadmap |

## Troubleshooting References

[`docs/errors-and-debugging.md`](../../docs/errors-and-debugging.md) (left-recursion
and alternation load errors), [`docs/troubleshooting.md`](../../docs/troubleshooting.md),
and the DEMO-2026-0003 Troubleshooting table.

## Glossary

| Term | Meaning |
|---|---|
| Ordered choice | Alternatives tried in declaration order; the first match wins |
| Alternative | One function name in `A \| B \| C` |
| Bare function | A function with no leading keyword; the form of a flat alternative |
| Rewind | Handing consumed input back before the next alternative |
| Left recursion | A rule that can reach itself without consuming a token; refused at load |

## Version Applicability

| Feature / Interface | Introduced | Changed | Deprecated / Removed | Applicable Environment |
|---|---|---|---|---|
| `A \| B \| C` in a capture type | 0.23.0 | — | — | development |
| Union diagnostic | 0.23.0 | extends the 0.22.0 furthest-failure union | — | development |
| `ArgInfo.alts` | 0.23.0 | additive field | — | development |
| `capy docs` union Type column | 0.23.0 | Type column prints the choice | — | development |
| Depth-bound error names the bound, code `E0003` | 0.24.0 | replaces the generic `expected …` message | — | development |
| `alts` in the wasm `capy_introspect` JSON | 0.24.0 | additive field after `type` | — | development |
| `capy version` prints the crate version | 0.24.0 | was `capy dev` for every unstamped build | — | development |

## Related Features

Repetition and recursion (MAN-2026-0001 §3), error recovery (MAN-2026-0001 §5),
the AST JSON schema ([`docs/ast-json.md`](../../docs/ast-json.md)).

## Related Documents

- `MAN-2026-0001` — spans, comments, recursion limits, errors, recovery
- `DEMO-2026-0003`, `DEMO-2026-0004` — the verified procedures for this chapter
- `PLAN-2026-0003`, `PROP-2026-0004`, `ADR-0003` — plan, proposal, decision (0.23.0)
- `PLAN-2026-0004`, `PROP-2026-0005`, `ADR-0004` — the 0.24.0 follow-ups
- `SYS-2026-0001` — how the matcher, loader and lib-parser implement it
- [`docs/library-authoring.md`](../../docs/library-authoring.md#ordered-choice) — the user-site page

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial chapter for 0.23.0 |
| 2 | 2026-10-07 | Olivier | Corrections from a full read-through after the tag (documentation only; no code change) |
| 3 | 2026-10-07 | Olivier | 0.24.0: the depth bound is named (`E0003`); the browser JSON carries `alts`; `capy version` prints the crate version; removed the two limitations and the `capy dev` statement that became false; linked DEMO-2026-0004 |
