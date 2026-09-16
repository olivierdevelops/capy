#!/usr/bin/env bash
# Verifies the browser-facing wasm surface, using the checked-in goldens as the
# oracle. Requires `deno`.
#
#   ./rust/devtools/wasm_check.sh
#
# This replaces wasm_compare.sh, which diffed the Rust wasm against a Go wasm
# build. With the Go implementation gone the durable oracle is the golden corpus
# in samples/, which `cargo test --test golden` already holds the NATIVE engine
# to. Running the same corpus through wasm therefore checks what the comparison
# was really there to check: that the wasm boundary — linear memory, the
# length-prefixed JSON envelopes, the JS shim — doesn't alter results.
#
# Also asserts the four globals the playground calls actually get installed, so a
# shim regression fails here rather than in a browser.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"; cd "$ROOT"
command -v deno >/dev/null || { echo "deno is required"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

echo "── building wasm ──"
( cd rust && cargo build --release --quiet --target wasm32-unknown-unknown -p capy-wasm-abi ) || exit 1
RS_WASM=rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm
SHIM=rust/playground/web/wasm_exec.js

echo "── shim installs the playground globals ──"
deno run --allow-read rust/playground/web/shim_test.js "$RS_WASM" || exit 1

echo "── golden corpus through wasm ──"
python3 - "$TMP/cases.json" <<'PY'
import json, os, glob, sys
cases = []
for d in sorted(glob.glob('samples/*/')):
    lib = os.path.join(d, 'lib.capy')
    if not os.path.exists(lib):
        continue
    for s in sorted(glob.glob(os.path.join(d, '*.capy'))):
        if os.path.basename(s) == 'lib.capy':
            continue
        base = os.path.splitext(s)[0]
        # Mirror tests/golden.rs: an .expected-error.txt case must fail, an
        # .expected.txt case must produce exactly that text, anything else is
        # skipped for want of a golden.
        ok, err = base + '.expected.txt', base + '.expected-error.txt'
        want, kind = None, 'skip'
        if os.path.exists(err):
            want, kind = open(err, encoding='utf-8').read().strip(), 'error'
        elif os.path.exists(ok):
            want, kind = open(ok, encoding='utf-8').read(), 'ok'
        cases.append({
            "id": f"{os.path.basename(d.rstrip('/'))}/{os.path.basename(s)}",
            "lib": open(lib, encoding='utf-8', errors='replace').read(),
            "script": open(s, encoding='utf-8', errors='replace').read(),
            "kind": kind, "want": want,
        })
json.dump(cases, open(sys.argv[1], 'w'))
print(f"  {len(cases)} cases")
PY

deno run --allow-read rust/devtools/wasm_harness.js "$TMP/cases.json" "$SHIM" "$RS_WASM" \
  > "$TMP/rs.json" 2>/dev/null || { echo "  harness failed"; exit 1; }

python3 - "$TMP/cases.json" "$TMP/rs.json" <<'PY'
import json, sys
cases = json.load(open(sys.argv[1]))
got = {r['id']: r for r in json.load(open(sys.argv[2]))}

def norm(s):
    return (s or '').lstrip('﻿').replace('\r\n', '\n')

# Cases that CANNOT work through the wasm surface, by design. The sandbox runs on
# NoOpHost — no filesystem — and the playground hands the engine a single library
# string rather than a path, so anything resolving a sibling file is out of reach.
# These were equally unreachable in the Go wasm build; the old Go-vs-Rust diff hid
# them because BOTH engines failed identically. Listed explicitly so the check
# stays honest: a case leaving this list must start passing, not silently skip.
SANDBOX_LIMITED = {
    # library-side `import "common/…"` needs to read sibling files
    'lib-composition/script.capy': 'library imports a sibling file',
    # script-side `@import "shared/…"` needs to read sibling files
    'source-imports/script.capy': 'script @imports a sibling file',
    # the CLI strips a leading `#!` line; the wasm entry point receives raw source
    'shebang-greet/script.capy': 'script starts with a shebang line',
}

passed = skipped = limited = 0
fails = []
for c in cases:
    if c['id'] in SANDBOX_LIMITED:
        limited += 1
        continue
    r = got.get(c['id'])
    if r is None:
        fails.append(f"{c['id']}: harness produced no result")
        continue
    run = r['run']
    if c['kind'] == 'skip':
        skipped += 1
    elif c['kind'] == 'error':
        if run['ok']:
            fails.append(f"{c['id']}: expected error per golden, got success")
        else:
            # The ABI deliberately splits what the CLI prints as one string: a
            # BARE message plus separate line/col fields (the browser formats it
            # itself, and `pretty` carries the caret rendering). Reassemble it
            # before comparing with the golden, which is in CLI form.
            msg = norm(run['error']).strip()
            # line/col are 0 when the error carries no source position (a type
            # pattern violation, say); the CLI prints no prefix in that case.
            if run.get('line') and run.get('col'):
                msg = f"{run['line']}:{run['col']}: {msg}"
            if msg != norm(c['want']).strip():
                fails.append(f"{c['id']}: error mismatch\n      want: {norm(c['want']).strip()!r}\n      got:  {msg!r}")
            else:
                passed += 1
    else:
        if not run['ok']:
            fails.append(f"{c['id']}: expected success, got error: {run['error']}")
        elif norm(run['output']) != norm(c['want']):
            w, g = norm(c['want']), norm(run['output'])
            i = next((i for i, (a, b) in enumerate(zip(w.split('\n'), g.split('\n'))) if a != b), 0)
            fails.append(f"{c['id']}: output mismatch at line {i+1}\n"
                         f"      want: {w.split(chr(10))[i][:100]!r}\n"
                         f"      got:  {g.split(chr(10))[i][:100]!r}")
        else:
            passed += 1
    # Every case must also return a usable docs + introspect payload; the
    # playground calls all three on every keystroke.
    #
    # EXCEPT when the LIBRARY itself is intentionally unloadable — a sample whose
    # golden is a library-level error (samples/left-recursion-rejected, say).
    # You cannot introspect a library that does not compile, and the playground
    # shows the same load error in all three panes. Detected by the introspect
    # error matching the run error, which only happens when the failure came from
    # loading rather than from the script.
    lib_unloadable = (
        not run['ok']
        and not r['introspect']['ok']
        and norm(r['introspect']['error']).strip() == norm(run['error']).strip()
    )
    if lib_unloadable:
        continue
    if not r['introspect']['ok']:
        fails.append(f"{c['id']}: introspect failed: {r['introspect']['error']}")
    if not r['docs']['ok']:
        fails.append(f"{c['id']}: docs failed: {r['docs']['error']}")

print(f"  PASS {passed}  FAIL {len(fails)}  SKIP {skipped} (no golden)  "
      f"SANDBOX-LIMITED {limited}")
for cid, why in sorted(SANDBOX_LIMITED.items()):
    print(f"    – {cid}: {why}")
for f in fails[:20]:
    print("    ✗ " + f)
sys.exit(1 if fails else 0)
PY
rc=$?
echo
if [ $rc -eq 0 ]; then echo "WASM CHECK PASS"; else echo "WASM CHECK FAIL"; fi
exit $rc
