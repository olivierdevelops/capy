# Operating instructions

When invoked, follow this loop:

## 1. Understand intent

If the user hasn't already, ask:

- **Target language?** (Python, JSON, SQL, …)
- **Source-language shape?** (Show 2–3 example lines they want to support.)
- **Output shape?** (Show what the target should look like for those examples.)
- **Required validation?** (e.g. "names must be lowercase", "must be a valid email")

## 2. Sketch the design

Before writing YAML, decide:

- Which features are **plain `template:` functions** (each source line →
  one output line)?
- Which need **`context:` accumulation** (collected at the top of output)?
- Which need **block functions**? Mode A (indent + named closer) or
  Mode B (`{ ... }` delimiters)?
- Which need **typed args**? Patterns? Enums?

## 3. Write `lib.yaml`

Start with the simplest possible version. Add features one at a time.
Run `capy check lib.yaml` after each addition.

## 4. Write `script.capy`

Exercise every pattern in the library. Aim for ~10 lines maximum.

## 5. Verify

```sh
capy run lib.yaml script.capy
```

Check output matches what you sketched in step 1. If not, iterate.

## 6. Capture the golden

```sh
CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden
```

(only relevant if the user has already added the sample to the repo)

## 7. Report

Tell the user:

1. The files you wrote.
2. The exact `capy run` command.
3. The actual output you observed.
4. Any features they asked for that aren't expressible in v0.1 and the
   workaround you used.

## Heuristics

- Prefer **auto-name-prepend** for normal function calls. Use literals
  only when you need operator-style or multi-keyword shapes (`set X = Y`,
  `depend on X`).
- Prefer **library-defined types** with `pattern:` or `options:` over
  ad-hoc validation in `run:`.
- Prefer **empty templates + run-driven context** for declarative DSLs.
  Prefer **non-empty templates with no run** for pure code-gen DSLs.
- When choosing block mode: use Mode A for indent-friendly languages
  (Python, YAML, Ruby), Mode B for brace languages (C, JS, Go, Rust).

## What to refuse

- **Conditional execution of user source.** Capy transpiles; it doesn't
  evaluate. If the user wants "skip this if x is false at compile time",
  tell them to do it in `run:` based on `context`, not on user variables.
- **Side-effect operations** (HTTP calls, file I/O, etc.) in `run:`.
  None of the primitives support this. Generate the target language to
  do the side effect if needed.

## Reference files

Always consult these when in doubt:

- [reference/schema.md](reference/schema.md) — the full schema.
- [reference/inner-dsl.md](reference/inner-dsl.md) — `run:` operations.
- [reference/template-helpers.md](reference/template-helpers.md) — helpers.
- [reference/examples.md](reference/examples.md) — canonical libraries.
- [reference/pitfalls.md](reference/pitfalls.md) — read this first.
