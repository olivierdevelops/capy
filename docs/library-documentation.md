---
title: Auto-generated library docs
hide:
  - toc
---

<div class="capy-hero" markdown>

<span class="capy-eyebrow">SELF-DOCUMENTING LIBRARIES</span>

# `description` directives → instant reference docs

Library authors annotate functions, types, args, and the library
itself with `description "..."` directives. `capy docs <library>`
produces Markdown reference documentation that lists every function
with its signature, arg table, and description — ready to drop into
a README, wiki, or onboarding doc.

The same renderer powers the **Docs tab** in the browser playground,
so the moment you load a library you see exactly what it can do.

</div>

---

## The annotation surface

Add a `description` line at any of four scopes:

```
extension html
description "Recipe DSL for home cooks. Six keywords (recipe, serves,
             time, ingredient, step, tip) produce a polished printable
             HTML recipe card."

type Email
    description "An email address used for owner contact info."
    pattern "^[^@]+@[^@]+\\.[^@]+$"
end

function recipe
    description "Open a new recipe with a title. Wraps the rest of the
                 file; closed by `end`."
    arg literal "recipe"
    arg capture title string  "Display name of the dish, shown as the H1."
    block_closer end
    set context.title title
end
```

- **Library description** — `description "..."` at the top level
  (alongside `extension`).
- **Type description** — `description "..."` inside `type NAME ... end`.
- **Function description** — `description "..."` inside
  `function NAME ... end`.
- **Argument description** — an optional 4th (capture) or 3rd (literal)
  string after the arg declaration: `arg capture title string "Display
  name of the dish, shown as the H1."`

YAML libraries support the same fields via `description:` keys (with
`args:` items growing a `description:` field of their own).

## Three ways to surface the docs

### CLI

```sh
capy docs samples/recipe-card/lib.capy
capy docs samples/recipe-card/lib.capy --out RECIPE_LIB.md
```

The output is plain Markdown — open it in any viewer, paste it into
a README, ship it with the library.

### Embedded in your Rust program

```rust
use capy_core::capy::{render_library_docs, Library};

let lib = Library::from_file("lib.capy")?;
let md = render_library_docs(&lib);
// → "# Library reference (→ `.html`)\n\n..."
```

Useful for tools that ship docs alongside generated code (e.g. a CLI
that emits both a config file and a "what does this config support"
reference next to it).

### Inside the browser playground

The [Capy playground](playground.md) has a **DOCS** tab in the
left pane. Click it and see Markdown-rendered reference for the
currently-loaded library, including every annotation the author
added. Switch samples and the docs swap with them — visible
documentation for every demo, no separate hosting.

## What's in the generated output

`capy docs` produces:

1. **Title + library description** at the top.
2. **Metadata strip** — output extension, default output file,
   function/type/file counts.
3. **Types section** — every declared type with its description,
   base, regex pattern, and enum options.
4. **Functions section** — alphabetical (priority-tagged ones first):
   - Function name + description
   - Reconstructed call shape (`recipe <title>`, `endpoint <method>
     <path> returns <type>`)
   - Arg table with name, type, description
   - Note when the function opens a block + which closer ends it
   - Note when the function has a non-default priority

A typical 100-line library produces a 2-page reference doc that
reads like a hand-written API reference but is regenerated from
source every time.

## Why this matters

- **Onboarding.** A new contributor reads the auto-generated docs
  and knows every function the team uses without grep-ing the
  library file.
- **Reviews.** A PR that adds a function but no description fails
  the team's "all DSL functions have descriptions" lint.
- **AI agents.** Hand the agent both the library and the generated
  reference — the agent has a typed contract AND human-readable
  explanations, drastically improving call-site accuracy.
- **Marketing.** "Look at our 50-keyword schema DSL" lands harder
  when each keyword has a one-sentence explanation a non-expert
  can read.

## Sample reference doc

The recipe-card library ships its annotated source plus a generated
[`LIB_REFERENCE.md`](https://github.com/olivierdevelops/capy/blob/main/samples/recipe-card/LIB_REFERENCE.md)
— a side-by-side example of what your library's docs would look
like after annotation.

[Recipe library source →](https://github.com/olivierdevelops/capy/blob/main/samples/recipe-card/lib.capy)
·
[Generated reference →](https://github.com/olivierdevelops/capy/blob/main/samples/recipe-card/LIB_REFERENCE.md)

## Tips for writing good descriptions

- **Lead with the verb.** "Add one ingredient…", "Open a new recipe…",
  "Validate an email address…". Past-tense gets confusing.
- **Mention side effects.** If a function appends to a list, say
  "Listed in a two-column grid above the method."
- **For args, say what units / shape the value takes.** `"Time as a
  free-form string, e.g. '45 minutes'"` beats `"the time"`.
- **For types, lead with the rule.** `"An ISO-8601 date (YYYY-MM-DD)."`
  beats `"A date."`.

## Suggested workflow

1. Write the library functionally first; get it working.
2. Add a top-level `description` summarizing what it generates.
3. Annotate each function as you commit it (or in batches during
   PR review).
4. `capy docs lib.capy > LIB_REFERENCE.md` and commit alongside
   the library.
5. In CI, fail the build if `capy docs lib.capy` doesn't match the
   committed `LIB_REFERENCE.md` — keeps the docs in lock-step with
   the implementation.
