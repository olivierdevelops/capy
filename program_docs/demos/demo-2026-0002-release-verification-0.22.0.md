---
document_id: DEMO-2026-0002
title: Release Verification Guide — 0.22.0
document_type: demo
status: active

created_date: 2026-09-16
last_updated: 2026-09-16
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

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - operators

scope: An executable verification procedure for every update released in 0.21.0, written for a reader who did not implement the change.

reason: DOCUMENTATION.md section 29 makes a release verification guide mandatory for every release.

related_documents:
  - PLAN-2026-0002
  - RPT-2026-0002
  - TEST-2026-0005
  - TEST-2026-0006
  - TEST-2026-0007

supersedes: null
superseded_by: null

tags:
  - demo
  - verification
  - release

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Release Verification Guide — 0.22.0

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.22.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Summary

Four released updates, each verifiable from a shell. Unlike 0.21.0's guide — where
the assertions were about a process *not dying* — these are about output you read.

## Release Identity

| Field | Value |
|---|---|
| Version | 0.22.0 |
| Tag | `v0.22.0` |
| Plan | PLAN-2026-0002 |
| Validation | RPT-2026-0002 |

## Prerequisites

```sh
cargo build --release --manifest-path rust/Cargo.toml -p capy-cli
export CAPY=./rust/target/release/capy
mkdir -p /tmp/capy-verify-22 && cd /tmp/capy-verify-22
```

Save this as `lib.capy`:

```
extension txt
function fn
    arg literal "fn"
    arg capture name ident
    arg literal "("
    arg capture params any
    arg literal ")"
    block_closer end
    write `f`
end
function let
    arg literal "let"
    arg capture name ident
    arg literal "="
    arg capture value any
    write `l`
end
function end
end
```

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | `PROP-2026-0001` R14 (from `capy_error.md` Mechanism 1) | A parse error names what was expected and where | § U-01 | `expected \`)\`, found …` | TEST-2026-0005 |
| U-02 | `PROP-2026-0001` R20 (Mechanism 3) | Parsing recovers; every mistake reported in one run | § U-02 | 3 statements, 2 errors | TEST-2026-0005 |
| U-03 | `PROP-2026-0001` R7 (from `needs2.md` §3) | The parse tree is available as JSON | § U-03 | One JSON document | TEST-2026-0007 |
| U-04 | `PROP-2026-0001` R10 (from `needs2.md` §1) | Infix operators with precedence | § U-04 | 7, then 9 | TEST-2026-0006 |

---

### U-01 — the error says what was wanted

```sh
printf 'fn add(x\n    end\n' > a.capy
$CAPY run lib.capy a.capy
```

**Expected:**

```text
error: expected `)`, found end of statement in `fn`
  1 │ fn add(x
```

**Before 0.22.0** this said `no library function matches token "fn"` — which named
the thing that was fine and not the thing that was missing.

---

### U-02 — every mistake in one run

```sh
printf 'let a = 1\nbogus\nlet b = 2\nalso bad\nlet c = 3\n' > b.capy
$CAPY ast lib.capy b.capy
```

**Expected:** three `let` statements, then two error regions, then two
diagnostics on stderr:

```text
<error> 2:1-2:6  2 token(s) skipped
<error> 4:1-4:9  3 token(s) skipped
error[E0001] 2:1: no library function matches token "bogus"
error[E0001] 4:1: no library function matches token "also"
```

**Before 0.22.0** parsing stopped at `bogus` and the user fixed one error per run.

---

### U-03 — the tree as JSON

```sh
$CAPY ast lib.capy b.capy --json | jq -c '{schema_version, stmts:(.tree.stmts|length), errors:(.tree.errors|length), diags:(.diagnostics|length)}'
```

**Expected:**

```json
{"schema_version":1,"stmts":3,"errors":2,"diags":2}
```

Note the tree is present **even though the file is broken** — that is the point.
Schema: [`docs/ast-json.md`](../../docs/ast-json.md).

---

### U-04 — operator precedence

Save as `p.capy`:

```
extension txt
context
    r 0
end
function calc
    arg literal "calc"
    arg capture v any
    set context.r v
end
file_template
    write `${context.r}
`
end
```

```sh
printf 'calc 1 + 2 * 3\n'   > q.capy && $CAPY run p.capy q.capy   # 7
printf 'calc (1 + 2) * 3\n' > q.capy && $CAPY run p.capy q.capy   # 9
```

**Expected:** `7` then `9`. The second shows grouping parentheses working without
colliding with the prefix-call form.

`file_template` is used rather than the function body because a function's own
template renders *before* its state mutations apply — the accumulator is a strict
prefix fold.

---

## Cleanup

```sh
cd / && rm -rf /tmp/capy-verify-22
```

## Limitations

- U-02's diagnostic still says "no library function matches" for a wholly
  unrecognised token; the improved message appears when some shape got *further*
  into the statement, as in U-01.
- The depth-limit error is not yet reported with its own message.

## Troubleshooting

| Symptom | Cause |
|---|---|
| U-01 prints `no library function matches token "fn"` | Pre-0.22.0 binary |
| U-02 shows 1 diagnostic | Recovery is off; check you used `capy ast`, not `capy run` |
| U-04 prints `0` | The template renders before `set` applies — use `file_template` |

## Actual Verification Status

Executed 2026-09-16 on macOS arm64 (Darwin 25.4.0), release profile.

| Update | Status | Observed |
|---|---|---|
| U-01 | **PASS** | `expected \`)\`, found end of statement in \`fn\`` |
| U-02 | **PASS** | 3 statements, 2 error regions, 2 diagnostics |
| U-03 | **PASS** | `{"schema_version":1,"stmts":3,"errors":2,"diags":2}` |
| U-04 | **PASS** | 7 and 9 |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial guide; all four updates executed and passing |
