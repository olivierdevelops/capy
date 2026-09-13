#!/usr/bin/env bash
# Build the three artifacts the browser playground needs:
#   docs/assets/playground/capy.wasm      (the Rust engine compiled to wasm)
#   docs/assets/playground/wasm_exec.js   (Capy's own loader shim)
#   docs/assets/playground/samples.json   (bundled curated sample sources)
#
# CI runs the same commands on every docs deploy (see .github/workflows/docs.yml).
# Run locally to preview the playground at docs/assets/playground/index.html via:
#   python3 -m http.server -d docs/assets/playground/ 8000
set -euo pipefail
cd "$(dirname "$0")/.."

OUT=docs/assets/playground

echo "[playground] building capy.wasm…"
# Inject the engine version so the playground UI can display it; capy_version()
# reads CAPY_VERSION at compile time. `git describe` falls back gracefully when
# run outside a git tree (e.g. from a downloaded tarball) — hence `|| echo dev`.
VERSION=$(git describe --tags --always 2>/dev/null || echo dev)
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
CAPY_VERSION="$VERSION" cargo build --release \
  --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm "$OUT/capy.wasm"
echo "[playground] wasm built at version: $VERSION"

echo "[playground] copying wasm_exec.js…"
# Our own shim, not the file Go used to ship: it defines a `Go` class with the
# same surface and installs the same capyRun/capyDocs/capyIntrospect/capyVersion
# globals, so index.html needs no changes.
cp rust/playground/web/wasm_exec.js "$OUT/wasm_exec.js"

echo "[playground] bundling samples.json…"
cargo run --release --manifest-path rust/Cargo.toml -p capy-playground-bundle > "$OUT/samples.json"

echo "[playground] done:"
ls -lh "$OUT/" | tail -n +2
