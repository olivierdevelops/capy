# Twitter / Mastodon / Bluesky thread (draft)

Short version (single post):

> 🌱 Capy v0.1.0 — a transpiler engine in Rust where the grammar is a `.capy`
> file. Define your DSL, get a code generator. Zero default keywords.
> Single-binary install. Six worked examples (Python, JSON, SQL,
> Makefile, HTML, TS).
>
> github.com/olivierdevelops/capy

---

Thread version (5 posts):

1/ 🌱 Just shipped Capy v0.1.0. It's a small Rust binary that
turns a `.capy` file describing a grammar into a working transpiler. No
parser-generator, no code generation, no template engine alone — one
runtime that does all three.

2/ A library declares functions, types, and a file template. Each
function has an args shape (literals + typed captures) and a body of
`write` / `set` / `append` statements that emit output and update an
accumulated context.

```
function greet
    arg capture name any
    write `Hello, ${name}!
`
end
```

3/ The engine has zero default keywords. `if`, `loop`, `=` only exist
when the library defines them. The README example builds a tiny
Python-flavored DSL from scratch — `import`, `say`, `assign`, `if`,
`loop` — that transpiles to runnable Python.

4/ Six samples in the repo covering Python, JSON, SQL, Makefile, HTML
components, and TypeScript interfaces. Install via `cargo install` or one
of the binary releases. MIT.

5/ Pre-1.0 — the `.capy` schema will likely evolve. Roadmap includes:
`else` arm, multi-output, configurable surface syntax, `validate`
snippets in inner Capy. Feedback wanted.
github.com/olivierdevelops/capy
