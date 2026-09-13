# HN launch post (draft)

**Title (≤80 chars)**

> Show HN: Capy – a transpiler engine where the grammar is a YAML file

**URL**

`https://github.com/olivierdevelops/capy`

**Comment (first)**

> Hi HN. I keep writing tiny parsers — a config DSL here, a route DSL there
> — and the shape is always the same. Capy is the engine I wanted to stop
> rewriting: define your source language in YAML, get a transpiler.
>
> Concretely: each function in the library declares an args pattern (mix
> of literal tokens and typed captures), a template fragment, and an
> optional `run:` snippet that updates an accumulated context. A
> file-level template assembles `body` + `context` into the final output.
>
> The engine has zero default grammar — no keywords reserved by the
> engine, no hardcoded `if`/`loop`. If your library doesn't define
> assignment, `x = 1` is a parse error.
>
> v0.1.0 today. Six worked examples in the repo (Python, JSON, SQL,
> Makefile, HTML components, TypeScript interfaces). Built in Rust, single
> binary install, MIT.
>
> Caveats up front:
> - Pre-1.0, the YAML schema may break.
> - No `else` arm yet on inner `if` (use two `if` blocks).
> - No multi-output yet.
> - Grammar is flatter than a full PEG/CFG parser — multi-token
>   expressions decompose into multi-capture patterns rather than
>   parsing precedence.
>
> Genuinely curious whether this fills a hole for anyone else or if
> existing tools cover the use case for you. Happy to dig into design
> trade-offs.

**Tips for posting**

- Post Tuesday or Wednesday morning Pacific time.
- Don't ask for upvotes anywhere.
- Stay in the comments for the first 4 hours.
- If asked "why not Jinja / ytt / gomplate / X" — answer with examples,
  not generalities.
