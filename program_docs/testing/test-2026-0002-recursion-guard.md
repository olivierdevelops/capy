---
document_id: TEST-2026-0002
title: Test — Left-Recursion Guard and Parse Depth Bound
document_type: test
status: completed

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

affected_versions:
  from: "0.21.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Defines and records the tests proving the engine never aborts its host process on recursive grammars or deeply nested input.

reason: A left-recursive library passed `capy check` and then killed the process with rc=134, which an embedder cannot catch as `Err`.

related_documents:
  - PLAN-2026-0001
  - PROP-2026-0001

supersedes: null
superseded_by: null

tags:
  - recursion
  - reliability
  - parser

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Test — Left-Recursion Guard and Parse Depth Bound

> **Status:** Completed
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Affected Versions:** 0.21.0
> **Owner:** Capy Engine
> **Affected Components:** capy-core

## Purpose

Prove PLAN-2026-0001 R0 and R0b: a left-recursive library is rejected at load
time, and no input — however deeply nested — can abort the process.

## Validated Requirements

| Plan Requirement | Description |
|---|---|
| PLAN-2026-0001 R0 | The engine never aborts on recursion depth |
| PLAN-2026-0001 R0b | Left recursion detected at library-load time |

## Preconditions

`capy-core` and the `capy` CLI built from the working tree.

## Test Environment

macOS arm64 (Darwin 25.4.0), Rust stable 1.90.0. Note that a Rust **test thread**
gets a 2 MiB stack while the **main thread** gets 8 MiB — a limit tuned only to
the main thread still aborted under `cargo test`, which is why the depth bound is
64 rather than 256.

## Test Data

Inline fixtures in `rust/tests/recursion_guard.rs`: direct self-recursion, an
indirect two-hop cycle, a right-recursive grammar, a single-capture function
(which receives an auto-prepended name literal), a 200-function chain, and a
20 000-level nested input.

## Procedure

```sh
cargo test --manifest-path rust/Cargo.toml --test recursion_guard
capy check  <left-recursive library>     # expect exit 1
capy run    <valid library> <deep input> # expect exit 1, never 134
```

## Expected Results

Every left-recursive library is refused at load with a message naming the cycle;
every non-left-recursive library still loads; no case aborts.

## Actual Results

9 tests passed, 0 failed.

**Baseline captured before the change** (the defect being fixed):

```text
$ capy check lib.capy
ok — 2 function(s), 0 type(s)
$ capy run lib.capy s.capy
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
rc=134
```

**After:**

```text
$ capy check lib.capy
function "expr": left recursion — it can match itself without consuming a token
(cycle: expr -> expr). Rewrite the rule so something is consumed first...
rc=1
```

| Test | Outcome |
|---|---|
| `direct_left_recursion_is_rejected` | PASS |
| `indirect_left_recursion_is_rejected` | PASS |
| `right_recursion_still_loads` | PASS |
| `non_recursive_function_as_type_still_loads` | PASS |
| `single_capture_function_is_not_left_recursive` | PASS |
| `self_reference_after_a_literal_is_fine` | PASS |
| `deep_nonterminal_chain_terminates` | PASS |
| `deeply_nested_input_errors_instead_of_aborting` | PASS |
| `ordinary_block_nesting_still_parses` | PASS |

## Result

PASS

## Evidence

```text
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Compatibility evidence: `capy check` over all 117 sample libraries — 117/117 ok,
zero false rejections by the new guard.

## Evidence Sources

- `rust/tests/recursion_guard.rs` (committed)
- `cargo test --manifest-path rust/Cargo.toml --test recursion_guard`
- `for f in samples/*/lib.capy; do capy check "$f"; done`

## Executed By

Capy Engine

## Executed At

2026-09-16T00:00:00+08:00

## Defects Raised

**One genuine finding that changed the plan.** The load-time guard alone did NOT
satisfy R0: a *valid* right-recursive library fed 20 000 nested parentheses still
aborted with rc=134. R0 requires both the load-time guard and the depth bound;
the depth bound is not the "backstop" the earlier discovery note implied. Recorded
as a Finding in PLAN-2026-0001.

Two defects in the **tests themselves** were found and fixed: an indirect-cycle
fixture that was not actually left-recursive (the auto-prepended name literal
consumes a token), and a "deep nesting still parses" fixture whose grammar had no
base case and so could never match.

## Related Documents

- PLAN-2026-0001 — the plan under test
- PROP-2026-0001 — P-07 (the reported crash)

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial record |
