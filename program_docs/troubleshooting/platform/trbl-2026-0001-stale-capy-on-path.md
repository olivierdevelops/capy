---
document_id: TRBL-2026-0001
title: Troubleshooting — A Stale `capy` on PATH Rejects Shipped Samples
document_type: troubleshooting
status: active

created_date: 2026-10-07
last_updated: 2026-10-07
document_revision: 2

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

scope: Why a test run against `capy` can fail on valid libraries or ignore your edits — a stale binary on PATH, or a `cargo build` that never rebuilt the CLI — and how to run the build you actually mean to test.

reason: DOCUMENTATION.md section 26 — the problem was non-obvious, cost investigation time during release verification, and can recur on any developer machine.

related_documents:
  - PLAN-2026-0003
  - TEST-2026-0009
  - DEMO-2026-0003
  - DEMO-2026-0004
  - PLAN-2026-0004
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

macOS arm64. Through 0.23.0 `capy --version` printed `capy dev` for every build, which did not identify it; since 0.24.0 it prints the crate version (see Remaining Limitations). The binary on PATH was
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

There are two independent causes. Both end with a binary that is older than the source.

**Cause 1 — a stale `capy` on PATH.** An older `capy` installed outside the repository and
earlier on PATH. Through 0.23.0 `capy --version` reported `capy dev` for every build, so the
version string could not reveal it.

**Cause 2 — a plain `cargo build` does not rebuild the CLI (found during 0.24.0 work).**
`rust/Cargo.toml` is a workspace whose root package is `capy-core`; the other members, including
the CLI (`capy-cli`, binary `capy`), are not default members. A plain `cargo build` in `rust/`
therefore builds only `capy-core` and leaves `rust/target/debug/capy` as it was. Verified on
2026-10-07 at `cb23972`:

```sh
cd rust
touch cli/src/main.rs && cargo build
touch cli/src/main.rs && cargo build --workspace
```

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
   Compiling capy-cli v0.24.0 (/Users/oliverlaleau/Documents/projects/capylang-claude/rust/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.58s
```

The first command touched the CLI source and did not compile it; the second compiled
`capy-cli`. (An earlier observation during the work recorded the first as `0.01s`; the figure above
is the run just quoted.)

```text
   cargo build              ─► capy-core only        rust/target/debug/capy  unchanged  ✗
   cargo build --workspace  ─► every member + CLI    rust/target/debug/capy  rebuilt    ✓
```

## Solution or Workaround

Run the build under test explicitly, and always build with `--workspace`:

```sh
cd rust && cargo build --workspace
./target/debug/capy version
./target/debug/capy run ../samples/<name>/lib.capy ../samples/<name>/script.capy
```

or reinstall (`scripts/install.sh`). Check which binary a bare `capy` resolves to with `which capy`.
`cargo build -p capy-cli` also rebuilds the CLI, but `--workspace` is the form every verification
guide uses.

## Verification

All verification in `TEST-2026-0009`, `TEST-2026-0011`, `DEMO-2026-0003` and `DEMO-2026-0004` used
`rust/target/debug/capy`, built with `cargo build --workspace`.

## Remaining Limitations

Since 0.24.0 `capy version` and `capy --version` print the crate version (`capy 0.24.0`) for a local
build, so a stale binary is identifiable: compare the printed version with `version` in
`rust/Cargo.toml`. The limits that remain:

- It does not identify a stale binary **within one crate version**: two builds of the same
  `0.24.0` tree before and after an edit print the same string. Use `ls -l` or rebuild.
- A release build stamped with `CAPY_VERSION` prints the stamp, not the crate version.
- A pre-0.24.0 stale binary still prints `capy dev`; that string itself is the tell.

## Current Status

Resolved with the workaround above. The version-string gap was closed in 0.24.0
(`PROP-2026-0005` R6, `DEMO-2026-0004` U-06). The plain-`cargo build` cause is a property of the
workspace layout and is documented, not changed.

## Related Documents

- `PLAN-2026-0003`, `TEST-2026-0009`, `DEMO-2026-0003`, `REL-0.23.0`
- `PLAN-2026-0004`, `DEMO-2026-0004` — 0.24.0, where the version string was fixed and cause 2 was found

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-10-07 | Olivier | Initial record |
| 2 | 2026-10-07 | Olivier | Added the second cause (plain `cargo build` does not rebuild the CLI); updated Remaining Limitations and Current Status for the 0.24.0 version string |
