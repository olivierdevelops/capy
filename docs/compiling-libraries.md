---
title: Compiling a Capy library to a standalone binary
---

# Compiling a Capy library

Capy ships a `build` subcommand that turns a `.capy` library into a
**standalone executable**. The binary has the library source baked
in as a string constant and dispatches to its commands at runtime —
nobody needs `capy` installed to run the resulting tool.

The same machinery works as a cross-compiler: one `capy build` can produce
binaries for Linux, Windows and ARM devices. Under the hood it shells out to
`cargo build`, so any installed Rust target is reachable.

This page is a walkthrough — author a tiny library, build it for the
host, then cross-compile it for four other targets, with concrete
size numbers and tips at each step.

---

## Prerequisites

- The `capy` CLI ([install](getting-started.md#1-install)).
- A Cargo toolchain (Rust 1.74+, via [rustup](https://rustup.rs)). `capy build`
  runs `cargo build` under the hood. **Only the developer needs Rust — the
  output binaries don't require it.**

Check:

```sh
capy version
go version
```

---

## Step 1 — author a library

A library is just a `.capy` file. The minimal shape that
`capy build` accepts has a manifest (`name`, `version`,
`description`) plus at least one `command "..."` block — that's the
verb the resulting binary will dispatch to.

Save the following as `greet.capy`:

```
name        "greet"
version     "0.1.0"
description "A tiny greet DSL."
extension   "txt"

function greet
    arg literal "greet"
    arg capture who string
    write `Hello from greet, ${unquote who}!
`
end

command "run"
    description "Print the greeting."
    let out = (compile context.arg0)
    print out
end
```

…and a sample script `hello.greet`:

```
greet "world"
```

Sanity-check with the in-tree CLI before building:

```sh
capy greet run hello.greet
# Hello from greet, world!
```

---

## Step 2 — build for the host

```sh
capy build greet -o greet
```

Output:

```
building greet (this needs the Cargo toolchain)…
✓ wrote greet (5.4 MB)
  try:  greet --help
```

The resulting binary is **self-contained** — copy it anywhere on the
same OS / arch and it'll dispatch the library's commands:

```sh
./greet run hello.greet
# Hello from greet, world!
```

`greet --help` lists every declared command with its auto-generated
arg/flag help, exactly the way `capy <library> --help` does when
running from source.

---

## Step 3 — cross-compile

`capy build` takes a `--target <triple>` flag.
One developer machine produces binaries for every common deployment
target:

| Target | Command |
|---|---|
| **host** | `capy build greet -o greet` |
| **Linux x86-64** | `capy build greet --target x86_64-unknown-linux-gnu -o greet-linux` |
| **Linux ARM64** (Raspberry Pi 4, AWS Graviton…) | `capy build greet --target aarch64-unknown-linux-gnu -o greet-arm` |
| **Windows x86-64** | `capy build greet --target x86_64-pc-windows-msvc -o greet.exe` |

A host build of the `greet` sample is **1.5 MB** (measured on macOS arm64, with
the release profile's `opt-level="z"`, LTO and symbol stripping). Cross-compiled
sizes land in the same ballpark.

Unlike Go, a Rust cross-compile needs two things installed up front: the target's
standard library (`rustup target add <triple>`) and a linker that can emit that
target's format. For Linux-from-macOS or similar, [`cross`](https://github.com/cross-rs/cross)
or a CI runner of the right OS is the path of least resistance.

The output of every cross-compile is a true target-format binary:

```sh
$ file greet-linux
greet-linux: ELF 64-bit LSB executable, x86-64, statically linked

$ file greet.exe
greet.exe:   PE32+ executable (console) x86-64, for MS Windows
```

Statically linked = drop on any matching kernel and it just runs.
No glibc compatibility dance, no `LD_LIBRARY_PATH`, no DLLs to
collect.

For the **complete target matrix** see `rustc --print target-list` —
freebsd, openbsd, illumos, netbsd, every `arm` revision, riscv64 — Capy inherits
all the tier-1 and tier-2 targets because the build step is plain `cargo build`.

---

## Step 4 — WebAssembly walkthrough

**`capy build --target wasm32-…` is not the browser path.** The wrapper it
generates stages the embedded library through a temp file, and
`wasm32-unknown-unknown` has no filesystem while WASI needs the host to grant
one — so the module's `--help` works but its commands fail. `capy build` warns
you when you aim it at a wasm target.

For the browser, compile the **engine** instead and hand it your library source
at run time. That is exactly what the [playground](playground.md) does:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm capy.wasm
cp rust/playground/web/wasm_exec.js .
```

That is a 1.28 MB module exposing `capy_run` / `capy_docs` / `capy_introspect`
over linear memory — no files, no args, nothing for a sandbox to refuse. The
`wasm_exec.js` shim installs `capyRun`/`capyDocs`/`capyIntrospect`/`capyVersion`
as globals, so a page calls them directly.

Minimal `index.html` host — no stdin wiring, because the ABI is a plain
function call:

```html
<!doctype html>
<html>
<body>
  <textarea id="src">greet "browser"</textarea>
  <pre id="out"></pre>
  <script src="wasm_exec.js"></script>
  <script>
    const go = new Go();
    WebAssembly.instantiateStreaming(fetch("capy.wasm"), go.importObject)
      .then(r => go.run(r.instance))
      .then(() => {
        const libSrc = `extension txt

function greet
    arg literal "greet"
    arg capture who string
    write \`hello ${unquote who}
\`
end
`;
        const res = capyRun(libSrc, "auto", document.getElementById("src").value);
        document.getElementById("out").textContent = res.ok ? res.output : res.error;
      });
  </script>
</body>
</html>
```

`capyRun(libSrc, format, scriptSrc)` returns `{ok, output, extension}` or
`{ok: false, error, hint, line, col, pretty}`. `capyDocs` and `capyIntrospect`
take just the library source. See
[`rust/wasm/src/lib.rs`](https://github.com/olivierdevelops/capy/blob/main/rust/wasm/src/lib.rs)
for the full contract and
[`rust/playground/web/wasm_exec.js`](https://github.com/olivierdevelops/capy/blob/main/rust/playground/web/wasm_exec.js)
for the shim.

---

## Tips & tricks

### The binary is already size-optimised

The generated `Cargo.toml` sets `opt-level = "z"`, `lto = true`,
`codegen-units = 1` and `strip = true`, which is why the `greet` sample lands
at ~1.5 MB rather than the several MB an unoptimised build would produce. There
is no extra flag to pass — it is the default.

For another ~40% shrink, run `upx --best` on the output. UPX-packed binaries
start a touch slower but ship smaller.

### Reproducible builds

Set `RUSTFLAGS` to remap the build paths, so the same input source produces the
same bytes regardless of which machine compiled it:

```sh
RUSTFLAGS="--remap-path-prefix=$HOME=~" capy build greet -o greet
```

Useful for release artefacts that you publish a checksum for. Note the temp
build directory changes every run, so combine this with `--keep-temp` if you
need to audit exactly what was compiled.

### The built binary has no `--version` of its own

`capy build` embeds the library, not a version string: the binary answers
`--help` and then dispatches everything else to your library's declared
commands. If you want `mytool version`, declare it as a command in the
library — that way the version lives with the source it describes:

```
command "version"
    description "Print the tool version."
    print "greet v1.4.2"
end
```

Then `./greet version` prints `greet v1.4.2`.

### Bundle multiple targets in one tarball

A common release recipe — produce binaries for every supported
target plus checksums:

```sh
# Each target needs `rustup target add <triple>` and a linker for it first.
for t in \
  "linux amd64 x86_64-unknown-linux-gnu" \
  "linux arm64 aarch64-unknown-linux-gnu" \
  "darwin amd64 x86_64-apple-darwin" \
  "darwin arm64 aarch64-apple-darwin" \
  "windows amd64 x86_64-pc-windows-msvc"
do
  set -- $t   # $1=os $2=arch $3=triple
  out="greet-$1-$2"
  [ "$1" = "windows" ] && out="$out.exe"
  capy build greet --target "$3" -o "dist/$out"
done
(cd dist && shasum -a 256 greet-* > SHA256SUMS)
```

You now have a `dist/` folder with five binaries + a checksum file —
upload that to a GitHub release and your library is `curl`-installable
on any of them.

### `capy build` does NOT pull commands from disk at runtime

The library source is **embedded** at build time. Once the binary
exists, editing `greet.capy` won't change the binary's behaviour —
you have to rebuild. That's a feature: the binary is a snapshot, so
shipping `greet-v1.0` is a meaningful artefact you can pin and
reproduce.

### Build directly from a local `.capy` path

`capy build` accepts a path, not just a library name:

```sh
capy build ./libs/greet.capy -o greet
capy build /tmp/scratch/draft.capy -o draft
```

Useful for project-local libraries you haven't installed on
`CAPY_LIBS`.

### Keep the temp build directory for inspection

```sh
capy build greet --keep-temp
```

Prints the path of the temp dir holding the generated `src/main.rs` and
`Cargo.toml`. Helpful when a `cargo build` failure is mysterious — go look at
what was generated and run `cargo build` on it directly to get the full
compiler diagnostics.

### Build cache makes repeated builds fast

The first `capy build greet` compiles the engine from scratch — roughly 20
seconds. Each build uses a fresh temp project with its own `--target-dir`, so
rebuilds do *not* currently share a cache; that is the trade for keeping the
build hermetic and never touching a surrounding workspace.

### Building inside the Capy source tree vs. from a release

If you're working inside a clone of `github.com/olivierdevelops/capy`,
`capy build` detects the local `capy-core` crate (looking in the directory and
in a `rust/` subdirectory, walking upwards) and generates a `path` dependency —
your changes to the engine flow into the output binary, and the build works
offline. Otherwise it depends on the published crate version.

### What if the agent / user wants the binary INSIDE the browser?

Compile the **engine** (the [`capy-wasm-abi`](https://github.com/olivierdevelops/capy/tree/main/rust/wasm)
crate) to wasm and load your library source dynamically — the playground's
setup, and the walkthrough in Step 4 above. Embedding the library into a wasm
module with `capy build --target wasm32-…` is *not* a working substitute; see
Step 4 for why.

---

## Caveats

| Caveat | Mitigation |
|---|---|
| **Cargo toolchain required to build** (not to run) | One-time install via [rustup](https://rustup.rs). Minimum supported Rust version 1.74. |
| **Cross-compiling needs the target installed** | `rustup target add <triple>`, plus a linker for that target. Unlike Go's built-in cross-compiler, this is not free — use [`cross`](https://github.com/cross-rs/cross) or a matching CI runner. |
| **A wasm target does not produce a usable module** | The wrapper stages the library through a temp file, which wasm has no filesystem for. Build `capy-wasm-abi` for the browser instead (Step 4). |
| **Library `command` bodies that `exec` external tools** | Those tools must exist on the *target* machine, not the build machine. `exec "pandoc" …` in a library command will fail on a host that doesn't have pandoc installed. |
| **Library `read_file` / `write_file` paths** | Run with the right working directory or pass absolute paths. The binary uses the host filesystem like any other process. |
| **Binary size** | ~1.5 MB. The generated project already uses `opt-level="z"`, LTO and stripping; UPX can roughly halve it again. The library source itself contributes a few KB at most. |

---

## Comparison: `capy build` vs. the alternatives

| Approach | Pros | Cons |
|---|---|---|
| **`capy build greet`** | Self-contained, version-pinnable, embeds the library | Needs Cargo to build; ~1.5 MB minimum binary size; cross-compiling needs the target toolchain |
| **Ship `.capy` + require `capy install`** | Tiny artifact (a `.capy` file is a few KB) | Every user needs `capy` installed; library updates need redistribution |
| **Ship as a Rust library** ([embedding](embedding.md)) | No CLI binary; integrate Capy into a larger Rust program | Only useful when your distribution surface is already Rust code |
| **Ship `capy-wasm-abi` + lib** | Runs in any browser, no install | Two files (wasm + HTML host); only browser context |

For most "I built a DSL, I want to give it to teammates" use cases,
`capy build` is the right answer — one command produces five binaries
your colleagues can `curl` and run.

---

## Next steps

- [Compile cookbook](cookbook-compile.md) — eleven concrete recipes:
  multi-target tarballs, GitHub release workflows, browser-embedded
  WASM, static-site generators, Docker images, Homebrew formulas,
  `npm` packages, CI matrices, reproducible signed builds.
- [Library commands + `CAPY_LIBS`](library-commands.md) — design the
  commands that go inside your library before you ship it.
- [Embedding](embedding.md) — alternative path: link Capy into your
  Rust program instead of producing a CLI.
- [Auto-generated library docs](library-documentation.md) — produce
  a reference `README.md` from your library to bundle with the
  release.
