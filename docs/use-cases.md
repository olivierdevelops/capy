# Where Capy is useful

Capy is a **developer tool**. People write libraries; people write
source; people generate output. Most of these use cases involve no
AI at all.

That said, Capy is also unusually well-suited to AI workflows — in
both directions (AI as library author OR AI as library user). The
AI patterns live in their own section near the bottom.

Capy isn't a general-purpose programming language and it isn't a
general-purpose templating engine. It lives in a specific design
space: **anything where you'd otherwise hand-roll a tiny parser or
a hairy Python script to drive code generation**.

Inside that space, it's genuinely useful. Outside it, reach for
something else.

This page walks through the patterns we keep seeing, with concrete
scenarios for each.

---

## 1. Configuration generation at scale

This is the bread-and-butter Capy use case and the one most teams
will hit first. When you have:

- 50+ services, each with k8s manifests for 3 environments
- Plus Terraform modules for each service
- Plus CI workflows
- Plus monitoring rules (Prometheus, Datadog)
- Plus Ansible playbooks for the legacy stuff

…you have an explosion of nearly-identical configuration files.
Existing solutions: Helm (text templates), Cue (typed config),
Dhall (functional config), Jsonnet (programmatic config), or
hand-rolled Go binaries.

Capy fits the same niche differently:

- ➕ **Lower barrier**: a `.capy` library + a small surface DSL
  is easier to adopt than a typed config language. Anyone who reads
  plain config files can read a Capy library.
- ➕ **Multi-target**: same source generates k8s + Terraform + CI
  + monitors with one library per target.
- ➕ **No new query language**: it's just `args + template + run`.
- ➖ **Less typed than Cue/Dhall**: you give up algebraic types
  for friendliness.

**Scenario.** Each service has a 6-line `service.capy`:

```
service api
team    payments
runtime nodejs
port    8080
replicas
    prod    6
    staging 2
    dev     1
```

CI runs `capy run kubernetes-lib.capy service.capy > deploy.yaml`
for each environment. Same source produces a Terraform module via
a different library, a Datadog monitor via a third.

**Why it wins**: the diff for "bump replicas in prod" is one line
in one file, not three identical edits across three manifests.
Adding a new service is one new file, not 12.

Concrete sub-cases:

- **Kubernetes manifests** with per-environment overrides
- **Terraform modules** generated from a service catalogue
- **GitHub Actions / GitLab CI** pipelines
- **systemd unit files** for self-hosted servers
- **nginx / Caddy config** per project
- **`.env` files** with type-validated values
- **Dockerfile + docker-compose** for repeatable dev environments
- **Helm value files** if you're already on Helm

The [transpile-kubernetes](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-kubernetes),
[transpile-terraform](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-terraform),
[transpile-nginx](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-nginx),
and [transpile-systemd](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-systemd)
samples show this pattern.

---

## 2. Internal scaffolding and code generators

If your team has any of these, Capy is probably a better fit than
what you have today:

- Yeoman generators
- Plop / Hygen / Brittany
- Custom Go binaries that emit a microservice template
- A README that ends with "now copy these 15 files and rename X to
  your service name"

Define the conventions once in a library; every new service is
five lines of Capy.

**Scenario.** A platform team owns the company's "golden path": how a
new microservice should be structured (Dockerfile, k8s manifest, CI
workflow, README). Today they maintain a Go binary that generates
those files. With Capy: the conventions move to a `lib.capy`, the
binary is replaced by `capy run`, and the team can ship convention
changes in one PR that touches one file.

**Why it wins**:

- One library file is easier to read than 800 lines of Go template
  string-builder code.
- Diffs to the library are trivial to review.
- Product teams can read the source language without learning the target language.
- New conventions ship by editing one file.

---

## 3. AI builds the library, humans use it

This is the use case most people miss. The setup:

You want a custom DSL for your domain — say a level-design language
for game designers, or a check-list-rule language for compliance
officers, or a "service spec" DSL for your platform team. Writing
the parser is hard if you've never built one.

**Instead, let an AI write the library for you.** Then your
team uses the library to author content. The AI is involved in the
one-time, hard part (parser design) but is **not in the loop** when
content gets written.

The shape:

```
You + AI                You + your team
        ↓                        ↓
   lib.capy ────► Capy ◄──── script.capy
                  │
                  ▼
              target output
```

**Scenario.** A small game studio has 3 designers and 2 engineers.
The engineers want designers to author levels without learning JSON
+ the engine's API. Instead of writing a parser:

1. Engineer prompts Claude: "Build me a Capy library where designers
   declare rooms, exits, items, and NPCs. Output should be the JSON
   our engine expects."
2. Claude emits `lib.capy` (~80 lines). Engineer reviews, tweaks
   one or two patterns.
3. Designers now write `room kitchen contains key, knife exits door:hall`
   in `.capy` files. Capy generates the engine JSON.
4. Iterating on the DSL is one library edit. The designers never see
   it; their files keep working as long as the patterns stay the
   same.

**Why it wins**:

- Designers (or analysts, or lawyers, or whoever) get a friendly,
  domain-natural notation without an engineer hand-writing a parser.
- The engineer reviews and tests the library *once* and is then
  out of the loop.
- The AI's involvement is bounded to the library design. There's
  no per-invocation prompt; no token cost at use time; no
  hallucination risk in the content pipeline.
- The library becomes the documented contract between AI, engineer,
  and end user.

**This is Capy's quiet superpower.** Most "let AI generate code"
patterns put AI in the per-invocation loop. Capy lets you move AI
to the *one-time* library-design loop, then never invoke it again
for content. It turns AI from a runtime dependency into a build-
time author.

---

## 4. One-source-of-truth for multi-target generation

The same conceptual data often gets expressed in 4–6 places:

- A user model lives as PostgreSQL DDL, a TypeScript interface, a
  Pydantic class, a Zod schema, a GraphQL type, an OpenAPI
  component.
- An API endpoint lives in route handlers (Express/FastAPI/Gin), in
  documentation, in client SDKs (Axios/Requests), in mocks, in
  e2e tests.

When these drift, bugs follow. The standard fix is one source of
truth that compiles to many targets. Capy is exactly that pattern,
generalised:

```
                          ┌─→ Postgres DDL
                          │
                          ├─→ TypeScript types
   schema.capy ──capy──┤
                          ├─→ Zod schema
                          │
                          └─→ GraphQL SDL
```

Same source file, multiple invocations with different libraries.
Add a new target language by writing a new library — never touch
the source.

**Scenario.** A startup ships a TypeScript frontend, a Python ML
service, and a Postgres database. The user model needs to evolve.
Today: change 4 places, hope they agree. With Capy: change
`schema.capy` once; re-run four generators in CI; PR includes the
4 generated files for review, but reviewers know the source-of-truth
is the one file that changed.

**Why it wins**: the "did we keep these in sync?" question becomes
a CI assertion.

---

## 5. DSLs for domain experts

The most powerful Capy use case isn't engineering at all — it's
giving non-engineers a syntax that's natural for their domain and
that compiles to runnable code.

| Domain | Source the expert writes | What Capy generates |
|--------|--------------------------|---------------------|
| Finance | `if balance > 10000 then waive_fee` | Python eligibility checker, audit log JSON |
| Healthcare | `if symptom = chest_pain then escalate priority=high` | FHIR observation JSON + clinician dashboard alert config |
| Legal | `clause termination 30_days notice required` | Markdown contract + compliance checklist |
| Game dev | `room kitchen contains key, knife exits door:hall` | Engine JSON + walkthrough markdown |
| Logistics | `route warsaw -> berlin via train depart:08:00` | Booking API call + emissions report row |

The library author (the engineer) talks to the expert, encodes the
domain into a library schema, and from then on the expert is the
language's user. The library is the contract.

**Why it wins**:

- The expert reads/writes a natural-feeling notation, not Python.
- Reviewers can audit business rules without learning the
  implementation language.
- The generated code is consistent (no copy-paste drift).
- The grammar is the audit boundary: "show me what's possible to
  express" = `capy check lib.capy`.

---

## 6. Documentation generation

Most projects maintain documentation in several places that drift:

- README mentions an API endpoint that was renamed
- The Swagger doc still has the old response shape
- The changelog forgot to mention a breaking change
- The blog post is two versions behind

A Capy library can ingest a single source-of-truth and emit:

- A README section
- An OpenAPI yaml
- A markdown reference page
- An RSS-feed JSON for the blog
- A Slack notification payload for the release announcement

All from one input. They stop drifting because they share an input,
not a copy.

---

## 7. Standardisation and review surface reduction

Software diffs are how reviewers think about change. A 300-line
generated YAML diff is hard to review. A 4-line Capy diff is
trivial.

Two patterns:

### "Generated code is a binary artifact"

Treat the generated YAML/HCL/JSON as you treat compiled binaries:
they live in the repo for CI consumption, but reviewers look at the
Capy source diff. Add a CI step that fails if generated files are
stale.

### "Library changes are the policy changes"

When the platform team wants to require a new field on every k8s
deployment, they add it to the library. Every service regenerates.
The policy was a one-line library edit.

---

## 8. Migration and one-shot refactor tools

When a config format changes — old Travis YAML to GitHub Actions,
old Helm v2 to v3, old Webpack config to Vite — engineers
historically write a one-shot Python script with a thousand string
operations.

Capy does this better:

1. Library parses the old format (because Capy lets you define a
   grammar — and the old format already has one).
2. Function bodies accumulate the relevant facts via `set` / `append`.
3. `file_template` emits the new format.

The migration tool is then *just a library*. It's
self-documenting, type-checked, and testable. When you find an
edge case, you add a pattern to the library; you don't go hunting
through Python.

---

## 9. Audit, compliance, and lineage

Regulated domains care about provenance: "show me where this
production deployment configuration came from." With Capy:

- Every artifact is generated from a versioned source file.
- The set of legal patterns is a small library; auditors can read it.
- "What's the policy for X?" = "Look at function X's `args` and `run`."

Concrete: a fintech might have a Capy library called
`approved-postgres-roles` whose patterns are the only way to define
a database role in CI. Anything that doesn't fit is a parse error.
The audit becomes mechanical: any role definition either matches a
library function (compliant) or fails CI (rejected).

---

## 10. Education

Compiler courses traditionally use yacc/bison/ANTLR. Capy is a
much gentler on-ramp:

- "Build a calculator language in 30 lines of `.capy`."
- "Implement a simple SQL parser as a learning exercise."
- "Add an `if/else` to your toy language."

For a student new to language design, the library makes the
relationship between **grammar**, **semantics**, and **output**
explicit and inspectable. There's no generated code to read; the
library *is* the parser.

For a programming-language-theory course: the engine itself is
a few thousand lines of Rust, all readable in an afternoon.

---

## 11. Personal automation and dotfiles

If you ever generate the same kind of file repeatedly:

- Tmux configs across machines
- Caddy / nginx configs per project
- VS Code launch.json files
- Personal CI templates
- Cron jobs for a homelab

…Capy lets you encode your preferences once. Adding a new project
is `capy init` + `capy run`.

It's overkill if you generate one file. It's a meaningful upgrade
if you generate ten.

---

## 12. CI/CD pipeline portability

Many shops have CI pipelines duplicated across GitHub Actions,
GitLab CI, and Jenkins because different teams use different
hosts. Each duplicate is a place for drift.

A Capy library that defines pipeline semantics once, with three
sister libraries that target each provider, gives you provider-
agnostic pipelines. Switching CI hosts becomes a library swap, not
a rewrite.

---

## 13. Test data generation

For e2e or load testing, you need realistic fixtures in multiple
formats: SQL inserts, JSON payloads, CSV exports, GraphQL mutation
strings. Capy turns a single fixture file into all of them:

```
user alice email "alice@example.com" age 30 admin
user bob   email "bob@example.com"   age 25
order alice product "widget" quantity 3 total "$12.50"
```

…generates `seed.sql`, `users.json`, `orders.csv`, plus the
GraphQL mutations a frontend e2e test runs.

---

## 14. Internal developer platforms (Backstage, etc.)

Backstage-style "software catalogs" use YAML to describe services,
APIs, and resources. Capy upgrades that pattern from descriptive
to **executable**:

- The Backstage descriptor IS the source of truth.
- Capy libraries generate k8s manifests, dashboards, alert rules,
  and runbook stubs from it.
- One YAML file per service, but it does work.

---

## When Capy is the wrong choice

Be honest about non-fits:

| Situation | Why Capy isn't ideal | What to use instead |
|-----------|----------------------|---------------------|
| One-off generation with no repetition | Library design cost won't amortise | Raw LLM, hand-edit, sed/awk |
| You need to evaluate arbitrary user code at runtime | Capy is a transpiler, not an interpreter | A scripting language with a real sandbox |
| Target grammar has hundreds of distinct shapes | Capy patterns are flat; deeply nested grammars are painful | ANTLR, yacc, tree-sitter |
| You need formal verification of generated output | Capy validates inputs, not output semantics | Cue, Dhall, or a typed config language |
| Your team's strongest skill is the target language itself | A handwritten generator in Go/Python may read more naturally | Go templates, Jinja, custom generator |
| You want to build a general-purpose language | Capy's parser is intentionally simple | Don't use Capy as a host language for general computation |

---

## A quick mental model

> **Capy fits when the same shape of output gets produced many
> times, and the difference between instances is small.**

If you've ever written a comment that says "this is the third place
we have to update when we add a field", Capy probably has leverage
for you.

If you've ever written "I keep writing a one-off Python script for
this", Capy has leverage.

If you've ever asked an LLM to generate the same kind of file 50
times in a session, Capy has *enormous* leverage.

If your job is to think hard about novel algorithms with novel
shapes, Capy is not the tool — but it might still be useful for the
boring scaffolding around them.

---

## Next steps

- **[Getting started](getting-started.md)** — install and run a sample.
- **[Library authoring](library-authoring.md)** — write your own.
- **[AI agents](ai-agents.md)** — token math + sandboxing patterns.
- **[Cookbook](cookbook.md)** — recipes for common needs.
- **[50 sample demos](https://github.com/olivierdevelops/capy/tree/main/samples)**
  — find one near your domain and copy the pattern.
