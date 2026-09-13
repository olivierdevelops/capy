---
title: Compile cookbook — recipes for shipping Capy libraries
---

# Compile cookbook

Recipes for packaging and distributing a Capy library — as a native
CLI, a WASM module, a Docker image, an `npm` package, or a GitHub
release. Each recipe is a complete scenario with the commands to
run and the artefacts you end up with.

For the conceptual walkthrough first, see
[Compiling a Capy library](compiling-libraries.md).

The running example throughout is a tiny `greet` library — same one
the walkthrough uses — so every command in this page is verifiable
against a real on-disk artifact.

---

## Recipe 1 — Ship a CLI to your team (multi-target tarball)

**Scenario:** you have a library; you want to give your team a
single tarball with every binary they might need.

```sh
mkdir -p dist
# Each triple needs `rustup target add <triple>` and a linker for it first.
for t in \
  "linux amd64 x86_64-unknown-linux-gnu" \
  "linux arm64 aarch64-unknown-linux-gnu" \
  "darwin amd64 x86_64-apple-darwin" \
  "darwin arm64 aarch64-apple-darwin" \
  "windows amd64 x86_64-pc-windows-msvc"
do
  set -- $t                                # $1=os $2=arch $3=triple
  out="greet-$1-$2"
  [ "$1" = "windows" ] && out="$out.exe"
  capy build greet --target "$3" -o "dist/$out"
done
(cd dist && shasum -a 256 greet-* > SHA256SUMS)
tar czf greet-v0.1.0.tgz dist/
```

Output (real measurements, stripped):

| File | Size |
|---|---|
| `greet-linux-amd64` | 3.8 MB |
| `greet-linux-arm64` | 3.7 MB |
| `greet-darwin-arm64` | 3.7 MB |
| `greet-windows-amd64.exe` | 4.0 MB |
| `SHA256SUMS` | 421 B |

Anyone on your team `tar xzf`s the tarball and runs the right binary
for their platform.

---

## Recipe 2 — GitHub release workflow

**Scenario:** push a tag → CI builds every target → uploads them to
the GitHub release page.

`.github/workflows/release.yml`:

```yaml
name: release

on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        # Build each target on its own OS runner. A Rust cross-compile needs a
        # linker for the target, so this is simpler than cross-compiling all
        # five from ubuntu.
        include:
          - { os: ubuntu-latest,  name: linux-amd64,   target: x86_64-unknown-linux-gnu,  ext: ''     }
          - { os: ubuntu-latest,  name: linux-arm64,   target: aarch64-unknown-linux-gnu, ext: ''     }
          - { os: macos-latest,   name: darwin-amd64,  target: x86_64-apple-darwin,       ext: ''     }
          - { os: macos-latest,   name: darwin-arm64,  target: aarch64-apple-darwin,      ext: ''     }
          - { os: windows-latest, name: windows-amd64, target: x86_64-pc-windows-msvc,    ext: '.exe' }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install capy
        run: cargo install --git https://github.com/olivierdevelops/capy capy-cli

      - name: Build
        shell: bash
        run: |
          OUT="greet-${{ matrix.name }}${{ matrix.ext }}"
          capy build greet --target ${{ matrix.target }} -o "$OUT"
          ls -lh "$OUT"

      - uses: actions/upload-artifact@v4
        with:
          name: greet-${{ matrix.name }}
          path: greet-*

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with: { path: dist, merge-multiple: true }
      - name: Checksum
        run: |
          cd dist
          sha256sum greet-* > SHA256SUMS
      - uses: softprops/action-gh-release@v2
        with:
          files: dist/*
```

Tag and push:

```sh
git tag v0.1.0
git push --tags
```

The release page ends up with six binaries plus `SHA256SUMS`.

---

## Recipe 3 — Browser-embedded library as a JS module

**Scenario:** you want a small webpage where users paste source code
and see your library render it, with no server.

There are two viable strategies:

### A. Use the engine's wasm build + load library dynamically

Best when you want to swap libraries at runtime (e.g. a playground).

```sh
# 1. Build the engine for wasm.
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm engine.wasm

# 2. Copy Capy's wasm loader shim.
cp rust/playground/web/wasm_exec.js .
```

```html
<!doctype html>
<html>
<body>
  <textarea id="src">greet "world"</textarea>
  <pre id="out"></pre>
  <script src="wasm_exec.js"></script>
  <script>
    const go = new Go();
    const LIBRARY = `
      name "greet"
      extension "txt"
      function greet
          arg literal "greet"
          arg capture who string
          write \`Hello, \${unquote who}!
\`
      end
    `;
    WebAssembly.instantiateStreaming(fetch("engine.wasm"), go.importObject)
      .then(r => {
        go.run(r.instance);
        // After go.run, globalThis.capyRun is defined:
        document.getElementById("src").addEventListener("input", e => {
          const res = capyRun(LIBRARY, e.target.value);
          document.getElementById("out").textContent =
            res.ok ? res.output : res.error;
        });
      });
  </script>
</body>
</html>
```

The library lives in the JS source — you can edit it without
rebuilding the wasm. Good for tutorials, playgrounds, and live
demos.

### B. Use `capy build … --wasm` with the library baked in

Best when the library is fixed and you want a smaller surface area.

```sh
# NOTE: `capy build --target wasm32-…` does NOT produce a usable module — the
# generated wrapper stages the library through a temp file and wasm has no
# filesystem. Build the engine instead and pass the library source at run time
# (Recipe above).
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
```

`greet.wasm` is ~7 MB (unstripped) and contains the library hard-
coded. Use the same `wasm_exec.js` shim, but pass `argv` so the
embedded binary dispatches to the right command:

```html
<script>
  const go = new Go();
  go.argv = ["greet", "run", "/dev/stdin"];
  // Call capyRun(libSrc, "auto", scriptSrc) directly — no stdin wiring. See rust/wasm/src/lib.rs
  // for a polished version of this pattern.
</script>
```

The library is now an immutable artefact you can pin and version. If
you want to update the library, rebuild the wasm.

---

## Recipe 4 — Static-site generator

**Scenario:** Markdown is too restrictive, you want your own
authoring DSL (`*.recipe`, `*.recipedb`, whatever). Run it at build
time to emit a tree of HTML files.

```sh
# In your site repo:
recipes/
├── pasta-carbonara.recipe
├── ramen.recipe
└── ...

# Library that renders each .recipe into HTML.
recipe.capy
```

Build script (`build.sh`):

```sh
#!/usr/bin/env bash
set -euo pipefail
mkdir -p _site
for f in recipes/*.recipe; do
  out="_site/$(basename "${f%.recipe}.html")"
  capy run recipe.capy "$f" > "$out"
  echo "  wrote $out"
done
echo "✓ rendered $(ls _site/*.html | wc -l) pages"
```

Wire it into Netlify / Vercel / GitHub Pages as the build command.
The deployed site never runs Capy — the artifacts are plain HTML.

For something fancier, the library can declare a `build` command
that does the loop internally:

```
command "build"
    description "Render every .recipe in this dir into _site/"
    let recipes = (exec_capture "ls" "recipes/")
    for f in (split recipes "\n")
        if f
            let out = (compile (concat "recipes/" f))
            write_file (concat "_site/" (replace f ".recipe" ".html")) out
        end
    end
    print "done"
end
```

Now `capy recipe build` is your whole publishing pipeline.

---

## Recipe 5 — Docker container shipping the binary

**Scenario:** put the CLI in a slim image for use in CI pipelines or
container deployments.

`Dockerfile`:

```dockerfile
# ─── build stage ────────────────────────────────────────────
FROM rust:1-alpine AS build
RUN apk add --no-cache git musl-dev
RUN cargo install --git https://github.com/olivierdevelops/capy capy-cli
WORKDIR /src
COPY greet.capy .
RUN capy build greet -o /out/greet

# ─── runtime stage ──────────────────────────────────────────
FROM gcr.io/distroless/static:nonroot
COPY --from=build /out/greet /usr/local/bin/greet
ENTRYPOINT ["/usr/local/bin/greet"]
```

```sh
docker build -t greet:0.1.0 .
docker run --rm -v "$PWD:/work" -w /work greet:0.1.0 run hello.greet
```

Image size: ~3 MB (distroless static + a ~1.5 MB stripped binary).
The image has **no shell**, **no package manager**, **no Rust
toolchain** — just the embedded library + the dispatching runtime.

---

## Recipe 6 — Homebrew formula

**Scenario:** make your library `brew install`-able on a Homebrew tap.

After Recipe 2 ships releases:

`homebrew-greet/Formula/greet.rb`:

```ruby
class Greet < Formula
  desc "A tiny greet DSL"
  homepage "https://github.com/you/greet"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/you/greet/releases/download/v0.1.0/greet-darwin-arm64"
      sha256 "<paste from SHA256SUMS>"
    end
    on_intel do
      url "https://github.com/you/greet/releases/download/v0.1.0/greet-darwin-amd64"
      sha256 "<paste from SHA256SUMS>"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/you/greet/releases/download/v0.1.0/greet-linux-arm64"
      sha256 "<paste from SHA256SUMS>"
    end
    on_intel do
      url "https://github.com/you/greet/releases/download/v0.1.0/greet-linux-amd64"
      sha256 "<paste from SHA256SUMS>"
    end
  end

  def install
    bin.install Dir["greet-*"].first => "greet"
  end

  test do
    assert_match "Hello", shell_output("#{bin}/greet --help")
  end
end
```

After the release publishes:

```sh
brew tap you/greet
brew install greet
```

---

## Recipe 7 — `npm` package wrapping the WASM

**Scenario:** distribute the library as a JavaScript dependency that
`npm install greet`-ers can use without seeing Capy at all.

```sh
# 1. Build the wasm.
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm pkg/capy.wasm
cp rust/playground/web/wasm_exec.js pkg/
```

`pkg/index.js`:

```js
import "./wasm_exec.js";
import fs from "node:fs/promises";

let initialized = null;

async function init() {
  if (initialized) return initialized;
  const go = new Go();
  const wasm = await fs.readFile(new URL("./greet.wasm", import.meta.url));
  const inst = await WebAssembly.instantiate(wasm, go.importObject);
  // Call the capy* globals the shim installs (see rust/wasm/src/lib.rs for
  // an alternative that exposes a function-style API instead).
  initialized = { go, inst };
  return initialized;
}

export async function greet(scriptSrc) {
  await init();
  // ... feed scriptSrc to stdin, capture stdout ...
}
```

`pkg/package.json`:

```json
{
  "name": "@yourname/greet",
  "version": "0.1.0",
  "main": "index.js",
  "type": "module",
  "files": ["index.js", "greet.wasm", "wasm_exec.js"]
}
```

```sh
cd pkg && npm publish --access public
```

For a smoother JS API, use the engine `capy-wasm-abi` approach (Recipe
3A) and publish a thin JS wrapper that calls `capyRun(library,
script)` — no stdin plumbing required.

---

## Recipe 8 — CI testing the library across platforms

**Scenario:** make sure your `*.recipe` parsing works on every OS
your users might be on.

```yaml
name: test

on: [push, pull_request]

jobs:
  parse:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --git https://github.com/olivierdevelops/capy capy-cli
      - run: capy check greet.capy
      - name: Parse every example script
        shell: bash
        run: |
          for f in examples/*.greet; do
            capy run greet.capy "$f" > /dev/null
            echo "  ok $f"
          done
```

`capy check` validates the library standalone (catches grammar
errors, missing closers, unknown types). The loop ensures every
example script in the repo still produces an output across all three
OSes.

---

## Recipe 9 — Pin both Capy AND the library into the binary

**Scenario:** you ship a `greet v1.4.2` binary and want both
`greet --version` AND the library's reported version to be visible.

`capy build` embeds the library, not a version string — the binary answers
`--help` and dispatches everything else to your library's commands. So declare
the version as a command, and it lives with the source it describes:

```
command "version"
    description "Print the tool version."
    print "greet v1.4.2"
end
```

```sh
capy build greet -o greet
./greet version
# greet v1.4.2
```

The library also carries its own manifest version (`version "0.1.0"` at the top
of `greet.capy`). Note that neither `greet --help` nor `greet <cmd> --help`
prints it today — the help header shows the command's `description` only — so the
`version` command above is what actually surfaces a version to users. The two can
differ (`greet` the CLI vs `greet` the language spec); for releases you usually
want them in sync, so bump them together with a release script:

```sh
#!/usr/bin/env bash
set -euo pipefail
NEW="$1"
sed -i.bak "s/^version[[:space:]]*\".*\"/version \"$NEW\"/" greet.capy
git add greet.capy && git commit -m "release: $NEW"
git tag "v$NEW" && git push --follow-tags
```

CI (Recipe 2) takes it from there.

---

## Recipe 10 — Reproducible signed releases

**Scenario:** prove the binary on the release page matches the
source. Anyone can rebuild the exact same bytes.

Two ingredients:

1. **`--remap-path-prefix`** removes the build-machine's absolute file paths.
   The generated project already strips symbols and debug info via its release
   profile, so there is no separate flag for that.
2. **Fix `SOURCE_DATE_EPOCH`** for any timestamps that get embedded.

```sh
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
RUSTFLAGS="--remap-path-prefix=$HOME=~" \
  capy build greet -o greet-v1.4.2

# Verify bit-for-bit reproduction:
shasum -a 256 greet-v1.4.2
```

Run the same incantation on a different machine with the same Capy and
Rust versions and the same source tree — the SHA-256 should match. Note
`capy build` compiles inside a fresh temp directory each run, so pass
`--keep-temp` if you need to prove what was fed to the compiler.

Sign the artefacts with [minisign](https://jedisct1.github.io/minisign/)
or [cosign](https://github.com/sigstore/cosign):

```sh
minisign -Sm greet-v1.4.2 -s ~/.minisign/greet.key
```

Publish the `.minisig` file alongside the binary on the release
page. Users verify with:

```sh
minisign -Vm greet-v1.4.2 -P "$(cat greet.pub)"
```

---

## Recipe 11 — A no-build distribution path

**Scenario:** your audience already has `capy` installed (e.g.
internal teammates, or you've documented the prereq). Skip building
binaries entirely.

```sh
# Just ship the .capy file.
mkdir -p ~/.capy/libs
cp greet.capy ~/.capy/libs/greet.capy

# Or via `capy lib install` if you've published to a tap-like URL.
capy lib new greet ~/.capy/libs/         # scaffolds a template
```

Users put the file on their `CAPY_LIBS`, and from any directory:

```sh
capy greet --help
capy greet run hello.greet
```

Tiny artefact (a few KB), no compilation step. The cost is that
every user needs `capy` installed — which is itself a single
[`go install`](getting-started.md) or release-binary download.

---

## Recipe 12 — Shebang scripts: `.greet` files that run themselves

**Scenario:** make your DSL source files directly executable so
users `chmod +x script.greet && ./script.greet` instead of having
to remember the `capy` invocation.

Capy strips a leading `#!` line before lexing, so any of these
shebangs work. Three forms, three trade-offs:

### Form A: `env -S` (most portable, recommended)

```
#!/usr/bin/env -S capy --lib greet
greet "world"
```

```sh
chmod +x hello.greet
./hello.greet
# → Hello from greet, world!
```

Works on macOS (any modern version) and Linux (GNU coreutils 8.30+
ships `env -S`, which is every distro from 2018 onward — Ubuntu
20.04+, Debian 11+, RHEL/Rocky 9+, Alpine 3.16+). The `-S` ("split
string") flag lets `env` see `capy --lib greet` as two arguments.

### Form B: `env` without `-S`

```
#!/usr/bin/env capy --lib greet
greet "world"
```

Works on macOS (it splits on whitespace by default) and on Linux
distros with GNU env 8.30+. Older Linux (CentOS 7, Ubuntu 18.04)
would treat `"capy --lib greet"` as a single binary name and fail.
Prefer Form A unless you control the deployment.

### Form C: Absolute path (no `env`)

```
#!/usr/local/bin/capy --lib greet
greet "world"
```

Most portable — no `env` quirks — but hard-codes the path to the
`capy` binary. Use for internal tooling where everyone has the
same install location.

### Form D: Standalone binary (no `capy` required)

After [Recipe 1 / 2](#recipe-1-ship-a-cli-to-your-team-multi-target-tarball)
ships a built `greet` binary, the shebang invokes the binary
directly:

```
#!/usr/local/bin/greet run
greet "world"
```

The user doesn't need `capy` installed; the library is embedded in
the `greet` binary. The trust warning that fires for "library not
on CAPY_LIBS" is automatically suppressed in built standalone
binaries — they set `CAPY_TRUST=1` internally because the embedded
library is the binary's identity.

### `PATH` and library resolution

For Forms A–C the library must be discoverable. Either:

- Drop it on `CAPY_LIBS` (`~/.capy/libs/greet.capy`), or
- Run the script from a directory that contains `greet.capy` (CWD
  is part of the default search path), or
- Use the `--lib /absolute/path/to/lib.capy` form to pin a
  specific file.

For Form D the binary IS the library — no resolution needed.

### Putting it together

```sh
# Install once, system-wide.
sudo cp capy /usr/local/bin/capy
sudo mkdir -p /usr/local/share/capy/libs
sudo cp greet.capy /usr/local/share/capy/libs/
export CAPY_LIBS=/usr/local/share/capy/libs

# Now any user can write:
cat > /tmp/hello.greet <<'EOF'
#!/usr/bin/env -S capy --lib greet
greet "world"
EOF
chmod +x /tmp/hello.greet
/tmp/hello.greet
# → Hello from greet, world!
```

The file is now indistinguishable from a Python / Ruby / Lua
script as far as the OS is concerned — execute permission flag +
shebang, the kernel does the rest.

---

## When to pick which recipe

| You want… | Recipe |
|---|---|
| One binary your team can `curl` and run | 1 (multi-target tarball) |
| Automated GitHub releases on every tag | 2 (release workflow) |
| A live playground for your DSL | 3A (engine wasm + dynamic library) |
| An embeddable DSL renderer in a Node/browser package | 3B + 7 (wasm + npm) |
| A docs site whose authoring format is your own | 4 (static-site generator) |
| A Docker image for CI pipelines | 5 (distroless container) |
| `brew install greet` | 2 + 6 (releases + Homebrew tap) |
| Cross-OS CI for your library | 8 (test matrix) |
| Reproducible, signed builds | 10 (`-trimpath` + minisign) |
| Lowest-friction internal sharing | 11 (just ship the `.capy` file) |
| Source files that run themselves (`./script.greet`) | 12 (shebang scripts) |

---

## See also

- [Compiling a Capy library](compiling-libraries.md) — the
  fundamentals + walkthrough.
- [Library commands + `CAPY_LIBS`](library-commands.md) — how
  command bodies actually work.
- [Embedding Capy in Rust](embedding.md) — when you want Capy linked
  into a larger Rust program instead of producing a standalone binary.
- [Capy for AI agents](ai-agents.md) — the agent-side story, where
  the binary becomes a tool an LLM can call.
