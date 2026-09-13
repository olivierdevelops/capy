#!/usr/bin/env bash
# Cross-implementation conformance gate: proves the Rust engine and the Go
# reference implementation produce IDENTICAL output at every layer.
#
#   ./rust/devtools/conformance.sh
#
# Run from the repo root. Requires a Go toolchain and a Rust toolchain; the wasm
# comparison additionally needs `deno` and is skipped when absent.
#
# IMPORTANT: this must never mutate the working tree. Two Capy behaviours make
# that easy to get wrong, so both are avoided deliberately:
#   * `capy fmt FILE` rewrites in place — only `capy fmt --stdout` is read-only.
#   * `capy run` honours a library's `output_file`, writing next to the SCRIPT —
#     so CLI run comparisons happen against a throwaway copy of samples/.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
GO_BIN="$TMP/go"; RS_BIN="rust/target/debug"
mkdir -p "$GO_BIN"
# Snapshot the tree so the guard at the end reports only what THIS RUN changed,
# not pre-existing local edits.
git status --porcelain samples/ 2>/dev/null | sort > "$TMP/tree_before" || : > "$TMP/tree_before"

fail=0
ok()   { printf '  \033[32m✅\033[0m %s\n' "$1"; }
bad()  { printf '  \033[31m❌\033[0m %s\n' "$1"; fail=1; }
cmp_files() { if diff -q "$2" "$3" >/dev/null 2>&1; then ok "$1"; else bad "$1 ($(diff "$2" "$3" | grep -c '^[<>]') differing lines)"; diff "$2" "$3" | head -6 | sed 's/^/       /'; fi; }

echo "── building both engines ──"
for d in rust/devtools/go/*/; do go build -o "$GO_BIN/$(basename "$d")" "./$d" || exit 1; done
go build -o "$GO_BIN/capy" ./cmd/capy
go build -o "$GO_BIN/capy-mcp" ./cmd/capy-mcp
( cd rust && cargo build --workspace --quiet ) || exit 1
for b in lexdump helperdump innerdump libdump loaddump parsedump multidump embeddump cmddump goldenrun; do
  ( cd rust && cargo build --quiet --bin "$b" ) || exit 1
done

# Every .capy in the repo, and every (lib, script) pair. A while-read loop rather
# than `mapfile`, which macOS's bundled bash 3.2 doesn't have.
CAPY=()
while IFS= read -r f; do CAPY+=("$f"); done < <(find samples examples -name '*.capy' | sort)
PAIRS=()
for d in samples/*/; do
  [ -f "$d/lib.capy" ] || continue
  for s in "$d"*.capy; do
    [ "$(basename "$s")" = lib.capy ] && continue
    PAIRS+=("$d/lib.capy" "$s")
  done
done

echo "── engine layers ──"
"$GO_BIN/lexdump"   "${CAPY[@]}"  >"$TMP/g1" 2>&1; "$RS_BIN/lexdump"   "${CAPY[@]}"  >"$TMP/r1" 2>&1; cmp_files "lexer (${#CAPY[@]} files)" "$TMP/g1" "$TMP/r1"
"$GO_BIN/helperdump"              >"$TMP/g2" 2>&1; "$RS_BIN/helperdump"              >"$TMP/r2" 2>&1; cmp_files "built-in helpers" "$TMP/g2" "$TMP/r2"
"$GO_BIN/libdump"   "${CAPY[@]}"  >"$TMP/g3" 2>&1; "$RS_BIN/libdump"   "${CAPY[@]}"  >"$TMP/r3" 2>&1; cmp_files "library parser (RawLibrary)" "$TMP/g3" "$TMP/r3"
"$GO_BIN/loaddump"  "${CAPY[@]}"  >"$TMP/g4" 2>&1; "$RS_BIN/loaddump"  "${CAPY[@]}"  >"$TMP/r4" 2>&1; cmp_files "library loader (compiled Library)" "$TMP/g4" "$TMP/r4"
"$GO_BIN/parsedump" "${PAIRS[@]}" >"$TMP/g5" 2>&1; "$RS_BIN/parsedump" "${PAIRS[@]}" >"$TMP/r5" 2>&1; cmp_files "outer parser (parse trees)" "$TMP/g5" "$TMP/r5"
"$GO_BIN/multidump" "${PAIRS[@]}" >"$TMP/g6" 2>&1; "$RS_BIN/multidump" "${PAIRS[@]}" >"$TMP/r6" 2>&1; cmp_files "evaluator (rendered output + files)" "$TMP/g6" "$TMP/r6"
"$GO_BIN/embeddump" "${PAIRS[@]}" >"$TMP/g7" 2>&1; "$RS_BIN/embeddump" "${PAIRS[@]}" >"$TMP/r7" 2>&1; cmp_files "embedding API + docs renderer" "$TMP/g7" "$TMP/r7"
CMDLIBS=(samples/math-plots/lib.capy samples/mcp-widgets/lib.capy samples/shebang-greet/lib.capy)
"$GO_BIN/cmddump" "${CMDLIBS[@]}" >"$TMP/g8" 2>&1; "$RS_BIN/cmddump" "${CMDLIBS[@]}" >"$TMP/r8" 2>&1; cmp_files "library commands (help + arg parsing)" "$TMP/g8" "$TMP/r8"

echo "── golden suite (Rust engine vs checked-in goldens) ──"
if out=$("$RS_BIN/goldenrun" samples 2>&1); then ok "$out"; else bad "$out"; fi

echo "── CLI (read-only subcommands) ──"
for cmd in check docs; do
  p=0; t=0
  for lib in samples/*/lib.capy; do
    t=$((t+1))
    [ "$("$GO_BIN/capy" "$cmd" "$lib" 2>&1)" = "$("$RS_BIN/capy" "$cmd" "$lib" 2>&1)" ] && p=$((p+1))
  done
  [ "$p" = "$t" ] && ok "capy $cmd ($p/$t)" || bad "capy $cmd ($p/$t)"
done
p=0; t=0
for lib in samples/*/lib.capy; do
  t=$((t+1))
  [ "$("$GO_BIN/capy" fmt --stdout "$lib" 2>&1)" = "$("$RS_BIN/capy" fmt --stdout "$lib" 2>&1)" ] && p=$((p+1))
done
[ "$p" = "$t" ] && ok "capy fmt --stdout ($p/$t)" || bad "capy fmt --stdout ($p/$t)"

echo "── CLI transpile (against a throwaway copy — output_file writes) ──"
cp -R samples "$TMP/samples"
p=0; t=0
for d in "$TMP"/samples/*/; do
  [ -f "$d/lib.capy" ] || continue
  for s in "$d"*.capy; do
    b="$(basename "$s" .capy)"; [ "$b" = lib ] && continue
    [ -f "$d$b.expected.txt" ] || continue
    t=$((t+1))
    [ "$("$GO_BIN/capy" run "$d/lib.capy" "$s" 2>&1)" = "$("$RS_BIN/capy" run "$d/lib.capy" "$s" 2>&1)" ] && p=$((p+1))
  done
done
[ "$p" = "$t" ] && ok "capy run ($p/$t)" || bad "capy run ($p/$t)"

echo "── MCP server (JSON-RPC over stdio) ──"
cat > "$TMP/mcp.jsonl" <<'JSON'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"ping"}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"capy_check","arguments":{"library":"extension txt\nfunction zz\n    arg literal \"zz\"\nend\nfunction aa\n    arg literal \"aa\"\nend\n"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"capy_check","arguments":{"library":"wibble nonsense"}}}
{"jsonrpc":"2.0","id":6,"method":"wibble/method"}
JSON
"$GO_BIN/capy-mcp" <"$TMP/mcp.jsonl" >"$TMP/g9" 2>/dev/null
"$RS_BIN/capy-mcp" <"$TMP/mcp.jsonl" >"$TMP/r9" 2>/dev/null
cmp_files "capy-mcp responses" "$TMP/g9" "$TMP/r9"

echo "── playground bundle ──"
go run ./cmd/playground-bundle >"$TMP/ga" 2>/dev/null
"$RS_BIN/playground-bundle"    >"$TMP/ra" 2>/dev/null
cmp_files "samples.json" "$TMP/ga" "$TMP/ra"

echo "── wasm (skipped without deno) ──"
if command -v deno >/dev/null 2>&1; then
  ( cd rust && cargo build --release --quiet --target wasm32-unknown-unknown -p capy-wasm-abi ) \
    && ok "wasm built (compare with rust/devtools/wasm_compare.js)" \
    || bad "wasm build failed"
else
  echo "     (deno not found — skipping)"
fi

echo
if [ "$fail" = 0 ]; then
  printf '\033[32mCONFORMANCE PASS\033[0m — Rust and Go agree at every layer\n'
else
  printf '\033[31mCONFORMANCE FAIL\033[0m\n'
fi
# Guard: the run must not have dirtied the tree. Compared against the snapshot
# taken at startup, so pre-existing local edits don't trip it.
git status --porcelain samples/ 2>/dev/null | sort > "$TMP/tree_after" || : > "$TMP/tree_after"
if ! diff -q "$TMP/tree_before" "$TMP/tree_after" >/dev/null 2>&1; then
  printf '\033[31mERROR\033[0m — this run changed samples/; that is a bug in the script:\n'
  diff "$TMP/tree_before" "$TMP/tree_after" | sed 's/^/     /'
  fail=1
fi
exit "$fail"
