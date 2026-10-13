#!/usr/bin/env bash
# Regenerates capy.h and fails when it differs from the checked-in copy (T-09 / R11).
#
#   rust/ffi/check_header.sh           # check
#   rust/ffi/check_header.sh --update  # rewrite the checked-in header
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
command -v cbindgen >/dev/null || { echo "cbindgen is required (cargo install cbindgen)"; exit 2; }
TMP="$(mktemp)"; trap 'rm -f "$TMP"' EXIT
cbindgen --config "$HERE/cbindgen.toml" --crate capy-ffi --output "$TMP" "$HERE" 2>/dev/null
if [ "${1:-}" = "--update" ]; then cp "$TMP" "$HERE/include/capy.h"; echo "updated include/capy.h"; exit 0; fi
if diff -u "$HERE/include/capy.h" "$TMP"; then echo "capy.h is up to date"; else echo "capy.h has drifted: run rust/ffi/check_header.sh --update" >&2; exit 1; fi
