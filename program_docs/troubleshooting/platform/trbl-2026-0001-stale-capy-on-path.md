---
document_id: TRBL-2026-0001
title: Troubleshooting — A Stale `capy` on PATH Rejects Shipped Samples
document_type: troubleshooting
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
  - capy-cli

current_status: resolved-with-workaround

affected_versions:
  from: "0.22.0"
  to: null

applicable_environments:
  - development

audience:
  - engineers

scope: Why a test run against `capy` from PATH can fail on valid libraries, and how to run the build you actually mean to test.

reason: DOCUMENTATION.md section 26 — the problem was non-obvious, cost investigation time during release verification, and can recur on any developer machine.

related_documents:
  - PLAN-2026-0003
  - TEST-2026-0009
  - DEMO-2026-0003
  - REL-0.23.0

supersedes: null
superseded_by: null

tags:
  - troubleshooting
  - cli
  - environment

confidentiality: internal
review_cycle: on-release
next_review_date: 2027-01-07
---

# Troubleshooting — A Stale `capy` on PATH Rejects Shipped Samples

> **Status:** Active
> **Created:** 2026-10-07
> **Last Updated:** 2026-10-07
> **Affected Versions:** 0.22.0 and later
> **Owner:** Capy Engine
> **Affected Components:** capy-cli

## Problem

Running `capy run samples/<name>/lib.capy …` fails on a library that the golden suite passes.

## Symptoms

```text
$ capy run samples/signature-parser/lib.capy samples/signature-parser/script.capy
line 26: arg capture NAME [TYPE] [DESCRIPTION]
```

The message is a **usage line**, not a diagnostic, and it appears even for samples shipped in the repo.

## Environment and Versions

macOS arm64. `capy --version` prints `capy dev`, which does not identify the build. The binary on PATH was
`/Users/oliverlaleau/Documents/bin/capy`, built 2026-05-26 (4 343 026 bytes); the repo build is
`rust/target/debug/capy` (about 11 MB at the time).

## Investigation

```text
   same library ─► capy (PATH)             ─► usage line   ✗
                 ─► rust/target/debug/capy ─► correct      ✓
                 ─► cargo test --test golden ─► passes     ✓
```

The failing line is `arg capture … param* sep "," join ", "` — syntax the May build predates.

## Possible Causes

1. The library really is invalid. Ruled out: the golden suite loads it.
2. A regression in the working tree. Ruled out: the same file passes through the repo build.
3. A stale installed binary shadowing the repo build. **Confirmed.**

## Experiments and Attempts

Compared `ls -l` of both binaries; ran the identical command with each; ran the golden suite.

## Root Cause

An older `capy` installed outside the repository and earlier on PATH. `capy --version` reports
`capy dev` for every build, so the version string cannot reveal it.

## Solution or Workaround

Run the build under test explicitly:

```sh
cd rust && cargo build --workspace
./target/debug/capy run ../samples/<name>/lib.capy ../samples/<name>/script.capy
```

or reinstall (`scripts/install.sh`). Check which binary a bare `capy` resolves to with `which capy`.

## Verification

All verification in `TEST-2026-0009`, `TEST-2026-0011` and `DEMO-2026-0003` used
`rust/target/debug/capy`.

## Remaining Limitations

`capy --version` still prints `capy dev`. A build that stamped the Cargo version would make this class of
problem self-diagnosing; not planned here.

## Current Status

Resolved with the workaround above. The underlying version-string gap is open and is listed under
`REL-0.23.0` Follow-up Work.

## Related Documents

- `PLAN-2026-0003`, `TEST-2026-0009`, `DEMO-2026-0003`, `REL-0.23.0`

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
