# contract-first-api

**The grammar IS the contract.** One source file, three targets,
infinite future targets — none of which can drift because all of them
read from the same `script.capy`.

## The workflow

```
1.  User describes intent:
      "I want to declare REST endpoints with methods, paths,
       params, and return types."

2.  Agent drafts lib_openapi.capy (the grammar / contract).

3.  Validate immediately:
      capy check lib_openapi.capy
      → ok — 7 function(s), 0 type(s)

4.  Write the contract (script.capy):
      endpoint GET "/pets"
          returns "Pet[]"
      end

5.  Frontend devs can START BUILDING NOW.
    The DSL is a contract — they can mock against it.

6.  Library evolves over time:
      lib_openapi.capy     (initial)
      lib_typescript.capy  (added a week later)
      lib_markdown.capy    (added a month later)
      lib_postman.capy     (added when QA needs it)

    All read the SAME script.capy. Source never changes.

7.  Golden tests catch regressions:
      capy run lib_X.capy script.capy | diff - script_X.expected.txt
    If a library change breaks a target, the diff fails — CI rejects.
```

## Why this is the killer pattern

**Without Capy:**

- The "API spec" is one of: a Google Doc, an outdated OpenAPI file, a
  Postman collection, a few TypeScript interfaces. They drift.
- A new target (e.g. "we need Python clients now") means hand-writing
  the same shapes a third time.
- "Define the contract first" is impossible because the contract is
  whatever language the spec happens to be written in.

**With Capy:**

- The contract is a *grammar*, written ONCE in
  [`script.capy`](script.capy).
- Each target consumer (OpenAPI server, TypeScript client, Markdown
  docs, Postman collection, mock server, …) is a small library file
  that reads the same source.
- Adding a target = writing one library. **The contract is
  guaranteed-stable** because changing the library can't change what
  `script.capy` declares.

## Three targets, one source

```sh
# OpenAPI 3.0 YAML
../../capy run lib_openapi.capy script.capy > openapi.yaml

# TypeScript HTTP client
../../capy run lib_typescript.capy script.capy > client.ts

# Markdown API docs
../../capy run lib_markdown.capy script.capy > API.md
```

All three reference the same endpoint definitions. Edit `script.capy`
to add a new endpoint and all three outputs regenerate.

## Continually tested

The committed `script_*.expected.txt` files are **golden snapshots**.
Running `cargo test --manifest-path rust/Cargo.toml --test golden` re-runs every library against
`script.capy` and diffs against the golden — any drift fails CI.

This is the test-loop that makes the contract trustworthy: the agent
can add a new library tomorrow, but it can't *change* what
`script.capy` produces from existing libraries without the golden
diff catching it.

## Add a new target

Want a Postman collection? Python FastAPI server? Rust `reqwest`
client? Write a 30-line `lib_<target>.capy` with templates for
`endpoint`, `param`, `returns`. Commit a golden. Done.

The DSL grammar — `endpoint METHOD PATH ... end`, `param LOC NAME TYPE`,
`returns SCHEMA` — never changes. The contract is the contract.
