---
document_id: DEMO-2026-0001
title: Release Verification Guide — 0.21.0
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
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers
  - operators

scope: An executable verification procedure for every update released in 0.21.0, written for a reader who did not implement the change.

reason: DOCUMENTATION.md section 29 makes a release verification guide mandatory for every release.

related_documents:
  - PLAN-2026-0001
  - RPT-2026-0001
  - TEST-2026-0001
  - TEST-2026-0002
  - TEST-2026-0003

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

# Release Verification Guide — 0.21.0

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core, capy-cli

## Summary

Four released updates, each verifiable from a shell in under a minute. Two of them
(U-01, U-02) verify that something no longer **kills the process**, so the exit
code is the assertion — `1` is success and `134` is the bug returning.

## Release Identity

| Field | Value |
|---|---|
| Version | 0.21.0 |
| Tag | `v0.21.0` |
| Plan | PLAN-2026-0001 |
| Validation | RPT-2026-0001 |

## Prerequisites

- A `capy` binary built from the release commit:
  `cargo build --release --manifest-path rust/Cargo.toml -p capy-cli`
- A scratch directory. Nothing here touches the repository.

```sh
export CAPY=./rust/target/release/capy
mkdir -p /tmp/capy-verify && cd /tmp/capy-verify
```

## Released Updates and Verification

| Update | Inciting User Requirement | What Changed | Do This | Expected Result | Evidence Source |
|---|---|---|---|---|---|
| U-01 | `PROP-2026-0001` R0b (from `capy_error.md`, step 0) | A left-recursive library is refused when it loads, instead of passing `capy check` and then aborting the process | § U-01 below | Exit 1 with the cycle named | TEST-2026-0002 |
| U-02 | `PROP-2026-0001` R0 | Deeply nested input is refused instead of exhausting the stack | § U-02 below | Exit 1 — **never 134** | TEST-2026-0002 |
| U-03 | `PROP-2026-0001` R4 (from `needs2.md` §2) | Nested AST nodes report their real position instead of `line: 0, col: 0` | § U-03 below | All spans non-zero and nested | TEST-2026-0001 |
| U-04 | `PROP-2026-0001` R27 | Comments are retained and attached, without changing output | § U-04 below | Identical output; comment reachable on the node | TEST-2026-0003 |

---

### U-01 — a left-recursive library is refused, not fatal

```sh
cat > bad.capy <<'EOF'
extension txt
function expr
    arg capture lhs expr
    arg literal "+"
    arg capture rhs term
end
function term
    arg capture n any
end
EOF

$CAPY check bad.capy; echo "exit=$?"
```

**Expected:**

```text
function "expr": left recursion — it can match itself without consuming a token
(cycle: expr -> expr). Rewrite the rule so something is consumed first: put a
literal before the capture, or make the recursion trail (right-recursive)
instead of lead
exit=1
```

**Before 0.21.0** this printed `ok — 2 function(s), 0 type(s)` and `capy run`
then died with `fatal runtime error: stack overflow` and exit 134.

---

### U-02 — deep input cannot abort the process

The library here is *valid*; only the input is pathological.

```sh
cat > ok.capy <<'EOF'
extension txt
function expr
    arg literal "("
    arg capture inner expr
    arg literal ")"
    write `[${inner}]`
end
EOF
python3 -c "open('deep.capy','w').write('('*5000+'x'+')'*5000+'\n')"

$CAPY run ok.capy deep.capy >/dev/null 2>&1; echo "exit=$?"
```

**Expected:** `exit=1`.

**The assertion is the exit code.** `1` means the parser refused the input. `134`
means the process aborted and the regression is back — an embedding program
cannot catch that.

---

### U-03 — nested nodes report a real position

Not directly exercisable from the CLI (the AST is not yet a public surface), so
this is a developer check:

```sh
cargo test --manifest-path rust/Cargo.toml --test ast_spans
```

**Expected:** 13 tests pass, including `no_reachable_node_has_an_unset_span`,
which walks a three-level tree and fails if any node reports line 0.

`NOT DIRECTLY USER-TESTABLE` until `Library::parse` ships in PLAN-D; the test is
the evidence in the meantime.

---

### U-04 — comments are kept and change nothing

```sh
cat > c.capy <<'EOF'
extension txt
comments
    line "#"
end
function greet
    arg literal "greet"
    arg capture who any
    write `hi ${who}
`
end
EOF
printf '# a comment\ngreet world # trailing\n' > s1.capy
printf 'greet world\n'                          > s2.capy

$CAPY run c.capy s1.capy
$CAPY run c.capy s2.capy
```

**Expected:** both print `hi world`. Retention must not alter output.

That comments are *reachable* is covered by `leading_comment_is_attached` and
`node_span_excludes_its_comments` in `rust/tests/ast_spans.rs`.

---

## Cleanup

```sh
cd / && rm -rf /tmp/capy-verify
```

## Limitations

- U-03 has no CLI surface until PLAN-D ships `capy ast`.
- U-02's message is the generic "no library function matches" rather than naming
  the depth limit; PLAN-B addresses the wording.
- U-04 verifies leading comments only; trailing ones are retained but unattached.

## Troubleshooting

| Symptom | Cause |
|---|---|
| U-01 prints `ok` | You are running a pre-0.21.0 binary |
| U-02 exits 134 | The depth bound is missing or set too high for your stack |
| U-02 exits 0 | The grammar matched — check `deep.capy` was written correctly |
| U-04 outputs differ | Comment trivia is reaching the matcher; a serious regression |

## Actual Verification Status

Executed 2026-09-16 on macOS arm64 (Darwin 25.4.0), release profile.

| Update | Status | Observed |
|---|---|---|
| U-01 | **PASS** | exit 1, cycle named as expected |
| U-02 | **PASS** | exit 1 on 5 000-level nesting; never 134 |
| U-03 | **PASS** | 13/13 tests pass |
| U-04 | **PASS** | both invocations printed `hi world` |

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial guide; all four updates executed and passing |
