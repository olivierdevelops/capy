#!/usr/bin/env bash
# Compares the Go and Rust wasm builds through the SAME JS harness, exercising
# every field the playground reads (output, files, extension, docs, error, hint,
# line, col, pretty). Requires `deno`.
#
#   ./rust/devtools/wasm_compare.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"; cd "$ROOT"
command -v deno >/dev/null || { echo "deno is required"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# Build both wasm modules.
GOOS=js GOARCH=wasm go build -ldflags="-s -w -X main.version=0.12.0" -o "$TMP/go.wasm" ./cmd/capy-wasm
( cd rust && cargo build --release --quiet --target wasm32-unknown-unknown -p capy-wasm-abi )
RS_WASM=rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm

# Go's own loader, wherever this toolchain keeps it.
GOROOT="$(go env GOROOT)"
for p in "$GOROOT/lib/wasm/wasm_exec.js" "$GOROOT/misc/wasm/wasm_exec.js"; do
  [ -f "$p" ] && cp "$p" "$TMP/go_wasm_exec.js" && break
done
[ -f "$TMP/go_wasm_exec.js" ] || { echo "wasm_exec.js not found under $GOROOT"; exit 1; }

# Case list: every (library, script) pair in samples/.
python3 - "$TMP/cases.json" <<'PY'
import json, os, glob, sys
cases=[]
for d in sorted(glob.glob('samples/*/')):
    lib=os.path.join(d,'lib.capy')
    if not os.path.exists(lib): continue
    for s in sorted(glob.glob(os.path.join(d,'*.capy'))):
        if os.path.basename(s)=='lib.capy': continue
        cases.append({"id": f"{os.path.basename(d.rstrip('/'))}/{os.path.basename(s)}",
                      "lib": open(lib, encoding='utf-8', errors='replace').read(),
                      "script": open(s, encoding='utf-8', errors='replace').read()})
json.dump(cases, open(sys.argv[1],'w'))
print(f"  {len(cases)} cases")
PY

H=rust/devtools/wasm_harness.js
deno run --allow-read "$H" "$TMP/cases.json" "$TMP/go_wasm_exec.js"        "$TMP/go.wasm" > "$TMP/go.json" 2>/dev/null
deno run --allow-read "$H" "$TMP/cases.json" rust/playground/web/wasm_exec.js "$RS_WASM"   > "$TMP/rs.json" 2>/dev/null

python3 - "$TMP/go.json" "$TMP/rs.json" <<'PY'
import json, sys
g=json.load(open(sys.argv[1])); r=json.load(open(sys.argv[2]))
same=sum(1 for a,b in zip(g,r) if a==b)
print(f"  {same}/{len(g)} cases identical")
bad=0
for a,b in zip(g,r):
    if a==b: continue
    bad+=1
    for sect in ('run','introspect','docs','version'):
        if a.get(sect)!=b.get(sect):
            if isinstance(a.get(sect),dict):
                for k in set(a[sect])|set(b[sect]):
                    if a[sect].get(k)!=b[sect].get(k):
                        print(f"    {a['id']} {sect}.{k}:\n      go  : {str(a[sect].get(k))[:110]}\n      rust: {str(b[sect].get(k))[:110]}")
            else:
                print(f"    {a['id']} {sect}")
sys.exit(1 if bad else 0)
PY
