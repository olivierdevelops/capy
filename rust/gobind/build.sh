#!/usr/bin/env bash
# Regenerate the engine that binding.go embeds.
#
# capy_core.wasm is a build artefact that is nonetheless committed, because the
# Go module system cannot run a build step: `go get` on this package has to find
# the engine already in the tree. That makes it the one file here that can go
# stale silently, so run this after any change to the Rust sources and commit
# the result alongside them.
set -euo pipefail

cd "$(dirname "$0")/.."   # rust/

cargo build --release --target wasm32-unknown-unknown -p capy-wasm-abi
cp target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm gobind/capy_core.wasm

echo "engine: $(wc -c < gobind/capy_core.wasm) bytes"

# A smoke run is the cheapest proof the artefact is loadable and the ABI still
# matches what binding.go expects.
( cd gobind && go build ./... )
echo "ok: binding builds against the regenerated engine"
