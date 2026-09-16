---
document_id: STD-2026-0005
title: Quality Expectations and Validation Gates
document_type: standard
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
  - All

affected_versions: not-applicable

applicable_environments:
  - development

audience:
  - engineers
  - architects

scope: The evidence a change must produce, and the commands that produce it.

reason: The gates were already run on every change; recording them makes them citable from a proposal's validation table.

related_documents:
  - REF-2026-0001

supersedes: null
superseded_by: null

tags:
  - quality
  - gates
  - standards

confidentiality: internal
review_cycle: 6-months
next_review_date: 2027-03-16
---

# Quality Expectations and Validation Gates

> **Status:** Active
> **Created:** 2026-09-16
> **Last Updated:** 2026-09-16
> **Owner:** Capy Engine

## Summary

---

## QUAL-001 — Regression is demonstrated across the whole corpus

**Required.** Every engine change demonstrates, across **all** checked-in
libraries and the **full** golden corpus:

- `capy check` succeeds for every `samples/*/lib.capy`;
- golden output is byte-identical, or each difference is reviewed and recorded;
- the wasm corpus passes.

**Prohibited.** Sampling. Regenerating goldens with `CAPY_UPDATE_GOLDENS=1` to
make a diff disappear — a changed golden is reviewed and the reason recorded.

**Example.** 0.22.0 changed exactly one error golden; the old message named the
wrong construct entirely, so it was updated deliberately and the reason recorded
in RPT-2026-0002.

**Validation.** `GATE-002`. **Enforcement.** Blocking. **Owner.** Capy Engine.

---

## QUAL-002 — A measurable claim is predeclared

**Required.** Any claim about time, size, memory or count records its baseline,
method, controlled environment, exact command and acceptance threshold **before**
implementation, in a `TEST` document per §21.3. Actuals are recorded afterwards
and are not editable retroactively. All samples are reported, not only the best.

**Example.** TEST-2026-0008 reports all five timing samples including the
first-run outlier, with best-of-5 as the declared method.

**Validation.** Review of the `TEST` document. **Enforcement.** Blocking.
**Owner.** Capy Engine.

---

## QUAL-003 — A test that cannot fail for the right reason is not a test

**Required.** A test names the defect it detects. Where a defect is invisible to
the existing suite, the gap is stated and a test that *can* see it is added.

**Rationale.** `expr_to_text` dropping parentheses would emit wrong target code
silently, and no golden could catch it — arithmetic was a parse error, so no
golden contained any. The round-trip property test exists solely for that, and
asserts structural equality so a formatting change cannot mask a structural one.

**Validation.** Review. **Enforcement.** Advisory. **Owner.** Capy Engine.

---

## GATE-001 — Pre-commit gate

```sh
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
mkdocs build --strict
python3 rust/devtools/check_publishable.py
```

All must pass. `--all-targets` is required: the narrower invocation does not lint
tests or devtools, and has hidden real findings.

**Enforcement.** Blocking. **Owner.** Capy Engine.

---

## GATE-002 — Regression gate

```sh
for f in samples/*/lib.capy; do capy check "$f"; done      # expect all ok
cargo test --manifest-path rust/Cargo.toml --test golden   # expect 0 failed
./rust/devtools/wasm_check.sh                              # expect PASS
```

**Enforcement.** Blocking for any change to `capy-core`. **Owner.** Capy Engine.

## Change History

| Revision | Date | Author | Change |
|---|---|---|---|
| 1 | 2026-09-16 | Olivier | Initial standard, codifying rules already enforced in `CLAUDE.md` and in the verification gates |
