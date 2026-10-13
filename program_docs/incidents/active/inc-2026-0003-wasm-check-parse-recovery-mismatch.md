---
document_id: INC-2026-0003
title: Incident — wasm_check.sh Fails on parse-recovery/broken.capy (Pre-Existing)
document_type: incident
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
  - capy-wasm-abi

severity: low
start_time: 2026-10-13T00:00:00+08:00
end_time: null
root_cause_status: identified

affected_versions:
  from: "0.25.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Records a GATE-002 check that already fails on the baseline commit, found while running the gates for 0.26.0.

reason: DOCUMENTATION.md section 25 requires a defect found during implementation to be recorded and listed under Known Issues if unresolved.

related_documents:
  - PLAN-2026-0005
  - REL-0.26.0
  - RPT-2026-0005
  - TEST-2026-0015

supersedes: null
superseded_by: null

tags:
  - incident
  - wasm
  - gate

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-04-13
---

# Incident — wasm_check.sh Fails on parse-recovery/broken.capy (Pre-Existing)

> **Status:** Active
> **Created:** 2026-10-13
> **Last Updated:** 2026-10-13
> **Affected Versions:** 0.25.0 (and after)
> **Owner:** Capy Engine
> **Affected Components:** capy-wasm-abi

## Incident Summary

`rust/devtools/wasm_check.sh` (part of `GATE-002`) reports one failure:

```text
  ✗ parse-recovery/broken.capy: error mismatch
      want: '2: expected `due`, found end of statement in `task`'
      got:  'expected `due`, found end of statement in `task`'
  WASM CHECK FAIL
```

The golden (native) message carries a `2: ` line prefix; the wasm run's does not. **It fails identically on
the clean baseline `69fb4ac`** (a fresh `git worktree`, the same script), so it is not caused by 0.26.0.

## Severity

Low. The wasm module's results differ from the native engine's only in the position prefix of one
recovered-parse error message.

## Status

Active; not fixed in this release.

## Discovery Context

Running `GATE-002` for `PLAN-2026-0005` T-14.

## Start Time

Not bisected. Present at `69fb4ac`.

## End Time

Unresolved.

## Affected Systems

`rust/devtools/wasm_check.sh`, the browser playground's error text for a recovered parse.

## Affected Versions

0.25.0 and the baseline commit `69fb4ac`; unchanged by 0.26.0.

## Affected Components

`capy-wasm-abi`.

## Customer Impact

None observed.

## Detection

`rust/devtools/wasm_check.sh`.

## Reproduction Steps

```sh
rust/devtools/wasm_check.sh          # needs deno; ends with WASM CHECK FAIL
```

## Timeline

| Time | Event |
|---|---|
| 2026-10-13 | Failure seen on the 0.26.0 tree |
| 2026-10-13 | Same single failure on a clean checkout of `69fb4ac` (`✗` lists compared: identical) |

## Logs and Evidence

Saved outputs of both runs were compared line for line; one `✗` in each, the same case.

## Source Files

`rust/devtools/wasm_check.sh`; `rust/wasm/src/lib.rs`; `samples/parse-recovery/broken.expected-error.txt`.

## Root Cause

Not investigated. The two paths format the same recovered error with and without its `LINE:` prefix.

## Contributing Factors

`GATE-002` needs `deno`; whether it runs automatically anywhere was not checked.

## Resolution

None in this release.

## Verifying Tests

`rust/devtools/wasm_check.sh` once fixed.

## Corrective Actions

Decide whether the wasm message or the golden is right; make the other agree.

## Preventive Actions

Run `wasm_check.sh` where `deno` is available as part of the release gate (done for 0.26.0, which is how
this was found).

## Owners

Capy Engine.

## Remaining Risks

None known. Both runs listed exactly one failing case and the lists were compared line for line; whether
the script would also report a second kind of mismatch was not tested.

## Lessons Learned

Compare a failing gate with the baseline before attributing it to the change.

## Related Documents

- `PLAN-2026-0005`, `RPT-2026-0005`, `REL-0.26.0`, `TEST-2026-0015`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-13 | Olivier, Claude | Initial record |
