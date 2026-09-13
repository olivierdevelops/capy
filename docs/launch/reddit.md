# Reddit launch posts (drafts)

## /r/rust

**Title**: Capy — a transpiler engine in Rust where the grammar is a `.capy` file

**Body**:

I shipped v0.1.0 of Capy today.

It's a small Rust binary that reads input text, matches it
against library-defined patterns from a `.capy` file, and produces target
output. Think: "Jinja templates but with a real parser, scoped to code
generation."

Library shape:

```
function greet
    arg capture name any
    write `Hello, ${name}!
`
end
```

Source `greet "Alice"` produces `Hello, "Alice"!`.

Beyond plain templating, libraries accumulate a `context` value across
all statements via a small inner DSL — useful for collecting imports at
the top of the output, deduplicating, etc.

Repo: https://github.com/olivierdevelops/capy

Six worked examples in `samples/` covering Python, JSON, SQL, Makefile,
HTML components, and TypeScript. Single-binary install, MIT.

Curious what shapes this would or wouldn't fit for /r/rust.

---

## /r/ProgrammingLanguages

**Title**: Capy — a transpiler engine where the user's language is defined in a `.capy` file

**Body**:

Posting here for design feedback. Capy is a transpiler engine I shipped
yesterday. Architecturally:

- The user's source language is **entirely library-defined**. No
  keywords reserved by the engine; an empty library rejects every input.
- Each library function declares an `arg` pattern (literals + typed
  captures) and a body that emits output via `write` and/or mutates an
  accumulated `context`.
- Functions can open block bodies in one of two modes: INDENT/DEDENT
  with a named closer, or explicit `{...}` delimiters.
- There's a small fixed **inner DSL** used inside each library function's
  body — this is the only "hardcoded grammar." It does
  `set`/`append`/`merge`/`if`/`for`/`write` over the context and the
  output, and nothing else. No user-source execution.

Two grammars in one engine. The library author has surface freedom
through the outer grammar (which they configure entirely); the engine
has just enough imperative power through the inner DSL to thread state
through the transpilation.

Design questions I'd love feedback on:

1. **Flat vs hierarchical patterns**: I deliberately don't have
   operator-precedence parsing. Multi-token expressions like `4 + 5`
   are matched by patterns with `+` as a literal between two captures.
   This keeps the matcher simple but means library authors can't define
   "infix operator with precedence" naturally.

2. **Two-grammar split**: comments?

3. **`context`-as-state**: should this be untyped (current) or should
   the schema be enforced? Trade-off between authoring friction and
   typo-catching.

Repo with worked examples: https://github.com/olivierdevelops/capy

Pre-1.0 — open to letting the schema evolve based on real use.

---

## /r/learnprogramming

**Title**: Capy — turn a `.capy` file into a transpiler

**Body**:

If you've ever wanted to build a tiny DSL or code generator but didn't
want to learn ANTLR / write a recursive-descent parser by hand, Capy
might fit. You write a `.capy` file that describes the grammar, and Capy
gives you a transpiler.

15-minute tutorial: https://github.com/olivierdevelops/capy/blob/main/docs/tutorials/03-transpile-python.md

The example builds a tiny language that compiles to Python:

```
import json
say "hello"
x = 42
if x
    say "x is set"
end
```

…produces valid Python.

Open-source (MIT), single Rust binary. New, so feedback welcome.
