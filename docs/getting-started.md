# Getting Started

A five-minute tour of Capy. By the end you'll have run the canonical
Python-transpiling sample and understand the four things every Capy library
controls: `functions`, `types`, `context`, and `file_template`.

A Capy library is **a file written in Capy's own syntax** (`.capy`).
Examples in this doc are all `.capy`.

## 1. Install

```sh
# Rust users
cargo install --git https://github.com/olivierdevelops/capy capy-cli

# Or download a binary from
# https://github.com/olivierdevelops/capy/releases
```

Verify:

```sh
capy version
```

## 2. Run a sample

Clone the repo (samples live there) and run the Python transpiler:

```sh
git clone https://github.com/olivierdevelops/capy
cd capy
capy run samples/transpile-py/lib.capy samples/transpile-py/script.capy
```

Expected output:

```python
import json
import os
print("starting")
x = 42
items = ["a", "b", "c"]
if x:
    print("x is truthy")

for item in items:
    print(item)
```

## 3. What just happened

Capy read **two files**:

1. The library (`lib.capy`). It declares **functions** (each has an
   `arg` shape and a body of inner-DSL statements — `write` to
   produce output, `set` / `append` / `for` / `if` to update state
   and shape what gets emitted).
2. `script.capy` — the input source.

Capy matched each line of the source against the library's function
shapes. For each match, the function body runs in two passes:

- The **render pass** walks `write` statements, interpolating the
  captured values into the **output body**.
- The **state pass** walks `set` / `append` / etc., updating the
  **accumulated context** (lists, maps, scalars).

At the end, the library's `file_template` produces the final output
from `body` (concatenated rendered fragments) and `context` (final
accumulated state).

Concretely, `import json` matches a function that looks like this:

```
function import
    arg literal "import"
    arg capture name ident
    append context.imports name
end
```

`import json` adds `"json"` to `context.imports`. The function body
contains a single `append` — no output is written, only state is
mutated. The file template at the top of the output emits
`import json` from there.

A function that DOES write to the output uses `write`:

```
function say
    arg literal "say"
    arg capture msg any
    write `print(${msg})
`
end
```

The backtick literal is what gets emitted. `${msg}` interpolates the
captured value.

## 4. Make your own

```sh
mkdir my-dsl
cd my-dsl
capy init .
capy run lib.capy script.capy
```

`capy init` scaffolds a starter library + script. Edit `lib.capy`
to add your own functions, types, and file template.

## 5. Where to go next

- [Library Authoring](library-authoring.md) — the reference walkthrough.
- [Cookbook](cookbook.md) — short answers for common patterns.
- [Tutorials](tutorials/01-hello-world.md) — four progressive walkthroughs.
- The [samples/](https://github.com/olivierdevelops/capy/tree/main/samples) directory on GitHub — 50 end-to-end demos.
