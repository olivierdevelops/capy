#!/usr/bin/env bash
# T-10 / R12 / R15 — the default `capy ast --json` must be byte-identical to a baseline binary's, for every
# shipped sample. Usage:  rust/devtools/ast_json_compat.sh <baseline-capy> [<new-capy>]
#
# Build a baseline from a commit with:
#   git archive <sha> rust | tar -x -C "$DIR" && (cd "$DIR/rust" && cargo build --release -p capy-cli)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASE="${1:?baseline capy binary}"; NEW="${2:-$ROOT/rust/target/debug/capy}"
n=0; diffs=0
for d in "$ROOT"/samples/*/; do
  lib="$d/lib.capy"; [ -f "$lib" ] || continue
  for s in "$d"*.capy; do
    [ "$(basename "$s")" = "lib.capy" ] && continue
    a="$("$BASE" ast "$lib" "$s" --json 2>&1)"; b="$("$NEW" ast "$lib" "$s" --json 2>&1)"
    n=$((n+1))
    if [ "$a" != "$b" ]; then diffs=$((diffs+1)); echo "DIFF ${s#$ROOT/}"; fi
  done
done
echo "compared $n scripts, $diffs differences"
[ "$diffs" -eq 0 ]
