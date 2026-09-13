---
title: Grammar as contract
---

# Grammar as contract

A Capy library is not just a parser definition — it's a **machine-
verified contract** between the people writing source files and the
systems consuming generated output. This page walks through the
workflow it unlocks.

## The pattern

```
       User describes intent
                │
                ▼
   ┌────────────────────────────────┐
   │  Agent drafts lib.capy         │
   │  capy check confirms it parses │  ◀── Iterate until clean
   └────────────────────────────────┘
                │
                ▼
   ┌────────────────────────────────┐
   │  Source: script.capy           │  ◀── Now THE CONTRACT exists.
   │  endpoint GET "/users"         │     Frontend / consumers can
   │      returns "User[]"          │     build against it TODAY.
   │  end                           │
   └────────────────────────────────┘
                │
       ┌────────┼────────┐
       ▼        ▼        ▼
   OpenAPI   TS client  Markdown
    YAML      (.ts)      docs
       │        │        │
       ▼        ▼        ▼
   Mock      Frontend   Wiki
   server    devs       readers
```

The middle box — `script.capy` — is the **contract**. Everything
upstream (the library) and everything downstream (the consumers) can
evolve independently. The contract is the seam.

## What "contract" means here, concretely

A grammar gives you four things that an English-language spec
doesn't:

1. **Machine validation.** `capy check lib.capy` either accepts the
   library or names exactly which rule was violated. There's no
   "the spec is ambiguous on this point."
2. **Round-tripping.** Same source + same library → byte-identical
   output. Always. Forever. (Verified by the golden snapshot tests
   committed alongside every sample.)
3. **Typed inputs.** `arg capture name ServiceName` plus `type
   ServiceName { pattern ... }` means typos at the input boundary
   become errors at *transpile time*, not silent bugs in production.
4. **Reviewable surface.** Anyone reading the library learns the
   complete set of statements the source can contain. Spec drift is
   physically impossible.

## The "build before tested" claim

Here's the part that's genuinely new: **downstream consumers can
start work the moment the grammar parses**, before any library
target is implemented.

Why? Because the grammar tells you:

- What entities exist (`endpoint`, `param`, `returns` in the API
  example).
- What types those entities accept.
- What relationships are allowed (block nesting, ordering).

That's enough to:

- Write the frontend that *will* consume the future API.
- Mock the future generated output for testing.
- Plan database migrations against the schema-to-be.
- Have product/design conversations grounded in real shapes.

The library implementation (OpenAPI YAML, TypeScript client, etc.)
can land *after* — and when it does, the contract guarantees the
generated artifacts match what consumers were planning around.

## The continually-tested loop

Every Capy sample includes a `*.expected.txt` golden file. The Rust
test harness runs `script.capy` through every library and diffs
against the golden:

```sh
cargo test --manifest-path rust/Cargo.toml --test golden
```

What this catches:

- A library author tweaks `lib_X.capy` and accidentally changes the
  output shape for existing source. CI rejects.
- Someone adds a new function pattern that overlaps with an
  existing one and breaks parser priority. CI rejects.
- A template helper changes meaning (e.g. `unquote` strips
  differently). CI rejects.

In other words: **the library implementation can iterate aggressively
without breaking the contract**, because every change must survive
the goldens.

## The worked sample

Look at [`samples/contract-first-api/`](https://github.com/olivierdevelops/capy/tree/main/samples/contract-first-api):

```
samples/contract-first-api/
├── script.capy                  ← THE CONTRACT
├── lib_openapi.capy             ← Target 1: OpenAPI YAML
├── lib_typescript.capy          ← Target 2: TS client stubs
├── lib_markdown.capy            ← Target 3: API documentation
├── script_openapi.expected.txt
├── script_typescript.expected.txt
└── script_markdown.expected.txt
```

One source, three targets. The CI test (`rust/tests/golden.rs`)
runs all three libraries against `script.capy` on every commit. Add a
4th target (Postman, FastAPI server, Rust `reqwest` client) by
writing one new `lib_<X>.capy` + one new `script_<X>.expected.txt`.

## A useful agent workflow

When the user asks for "a DSL for X" — REST APIs, infrastructure
config, scene descriptions, anything declarative — drive this loop:

1. **Clarify the contract.** "What entities? What relationships?
   What downstream consumers?"
2. **Sketch the source first.** Write 3-5 lines of what
   `script.capy` should look like. Confirm with the user that
   *this* is the surface they want to author against.
3. **Draft `lib.capy`.** Implement just enough to make the example
   parse. Validate with `capy_check`.
4. **Pick one target.** Implement `file_template` to produce the
   first useful output (often Markdown or JSON — easy to inspect).
5. **Capture a golden.** Run the transpile, commit the output as
   `script.expected.txt`. The contract is now fully closed.
6. **Hand off.** Consumers can build against the contract; library
   authors can add targets without touching `script.capy`.

The Capy MCP server has `capy_check` exactly for step 3 — it's the
fastest feedback loop the agent can run.

## When this is the wrong tool

- The "source language" you want is fundamentally Turing-complete
  (you need full control flow, closures, IO). Capy is for
  declarative-shaped DSLs. Use a real PL.
- The output isn't structured (free-form prose, image generation).
  Capy is a transpiler; it needs a target with describable shape.
- You only have one consumer and one target forever. The
  contract-as-grammar overhead doesn't pay off until there are at
  least two consumers (or two targets) that need to stay in sync.

For everything else — APIs, configs, scenes, schemas, manifests,
clients-in-N-languages — the grammar-as-contract pattern is what
Capy was designed for.
