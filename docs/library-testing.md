# Testing your library

A `.capy` library is code: it deserves tests. This page shows the three ways
to verify a library — quick CLI checks, golden tests, and Rust embedding tests —
mirroring how Capy itself is tested.

## 1. `capy check` — does it even load?

The fastest gate. Loads the library, compiles every function/type, and reports
what it found without running a script:

```bash
capy check my-lib.capy
```

It surfaces parse errors, duplicate literals, unknown types, and broken
templates at load time. Wire it into CI as a first, cheap step:

```yaml
# .github/workflows/ci.yml
- run: capy check libs/*.capy
```

## 2. Golden tests — does it transpile correctly?

Capy's own test suite uses **golden files**: a known input is run through the
library and compared against a checked-in expected output. The harness
(`rust/tests/golden.rs`) walks a directory of samples and, for each
`<name>.capy` script, compares the result against:

- `<name>.expected.txt` — the exact output a successful run must produce, **or**
- `<name>.expected-error.txt` — the error message a run that *should* fail must produce.

### Layout

```
my-lib/
  lib.capy                 # the library (also lib.yaml is supported)
  hello.capy               # a test script
  hello.expected.txt       # its expected output
  bad-input.capy           # a script that should fail
  bad-input.expected-error.txt
```

### Run / regenerate

If you embed your library in a Rust crate, copy the golden harness, or run
Capy's own:

```bash
cargo test --manifest-path rust/Cargo.toml --test golden   # run goldens
CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden  # regenerate after intentional changes
```

The `-update` flag rewrites every `.expected*.txt` from the current output —
use it only when you've **intentionally** changed behaviour, then review the
diff before committing. An unreviewed `-update` defeats the test.

### Without Rust: a shell golden loop

If you ship a library but not a Rust test crate, a few lines of shell give you
the same guarantee:

```bash
for script in tests/*.capy; do
  base="${script%.capy}"
  capy run my-lib "$script" | diff -u "$base.expected.txt" - \
    || { echo "FAIL: $script"; exit 1; }
done
```

## 3. Rust embedding tests — does the API behave?

When you embed Capy as a library (see [embedding.md](embedding.md)), test
against the public API in `capy_core::capy`:

```rust
use capy_core::capy::Library;

#[test]
fn button() {
    let lib = Library::from_file("my-lib.capy").expect("library compiles");
    let out = lib.run(r#"button "Save" "#0a0""#).expect("run");
    let want = r#"<button style="background:#0a0">Save"#;
    assert!(out.contains(want), "got {out:?}, want substring {want:?}");
}
```

Key entry points:

| Function | Use |
|----------|-----|
| `Library::new(src)` / `Library::from_file(path)` | Compile a library. |
| `lib.run(script)` | Transpile to a single output. |
| `lib.run_multi(script)` | Transpile, returning the multi-file map too. |
| `lib.introspect()` | Function metadata — assert your library exposes what you expect. |
| `lib.set_host(host)` | Inject a fake `Host` so `env`/`read_file`/`exec` are deterministic in tests. |

### Sandboxing side effects

Library `command` bodies can touch the filesystem and shell. In tests, install
a no-op or recording host with `set_host` so a test never reads real env vars
or runs real commands. The default embedding host is already a sandbox
(`domain::host::NoOpHost`); only the CLI wires in `infra::os_host::OsHost`. A
test host must be `Send + Sync`, so record into a `Mutex`, not a `RefCell`.

## Table-driven tests (recommended)

Capy's own suite favours many small cases over a few big ones. Mirror that:

```rust
let cases = [
    ("plain", r#"button "OK""#, ">OK"),
    ("colored", r##"button "OK" "#f00""##, "background:#f00"),
];
for (name, script, want) in cases {
    let out = lib.run(script).expect(name);
    assert!(out.contains(want), "{name}: got {out:?} want {want:?}");
}
```

## What to test

- **Happy path** per function — one output golden each.
- **Defaults** — captures with `default` values, omitted optional args.
- **Type validation** — a script that violates a `type` `pattern`/`options`
  produces a `.expected-error.txt`.
- **Blocks** — nesting, the closer output, multi-section ordering.
- **Context accumulation** — `run:` mutations across multiple statements.
- **Multi-file** — assert each declared `file` appears in `RunMulti`'s map.

See [troubleshooting.md](troubleshooting.md) when a test fails and the cause
isn't obvious.
