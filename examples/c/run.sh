#!/usr/bin/env bash
# Builds the C ABI under the ffi profile and runs the C programs.
#
#   examples/c/run.sh walk      # the PROP-2026-0008 sample, built with zig cc if present, else $CC
#   examples/c/run.sh diagnose  # error and diagnostic paths
#   examples/c/run.sh exprs     # typed expression tree and JSON dump
#   examples/c/run.sh hostile   # hostile inputs, ASan + UBSan
#   examples/c/run.sh misuse    # misuse suite, ASan + UBSan
#   examples/c/run.sh all
#
# CC defaults to the system clang (it carries the sanitizer runtimes); ZIG=zig is used for `walk`.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${OUT:-$ROOT/rust/target/c-examples}"; mkdir -p "$OUT"
CC="${CC:-$(command -v /usr/bin/clang || command -v clang || command -v cc)}"
INC="$ROOT/rust/ffi/include"; LIB="$ROOT/rust/target/ffi/libcapy_ffi.a"; HERE="$ROOT/examples/c"
case "$(uname)" in
  Darwin) SYS="-lc -lm"; DEAD="-Wl,-dead_strip" ;;
  *) SYS="-lc -lm -lpthread -ldl"; DEAD="-Wl,--gc-sections" ;;
esac

( cd "$ROOT/rust" && cargo build --profile ffi --quiet -p capy-ffi )

build() { # name extra-flags...
  local n="$1"; shift
  "$CC" -std=c11 -g -Wall -Wextra -Werror -I"$INC" "$@" "$HERE/$n.c" "$LIB" $SYS -o "$OUT/$n"
}

run_walk() {
  if command -v "${ZIG:-zig}" >/dev/null; then
    "${ZIG:-zig}" cc -std=c11 -O2 -Wall -Wextra -Werror -I"$INC" "$HERE/walk.c" "$LIB" $SYS $DEAD -o "$OUT/walk"
  else build walk -O2 $DEAD; fi
  local got; got="$("$OUT/walk")"
  [ "$got" = "greet(who=world)" ] && echo "walk: PASS ($got)" || { echo "walk: FAIL got '$got'" >&2; exit 1; }
}
run_san() { build "$1" -fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-omit-frame-pointer; "$OUT/$1"; }

run_show() { build "$1" -O2 $DEAD; "$OUT/$1"; }

case "${1:-all}" in
  walk) run_walk ;;
  diagnose) run_show diagnose ;;
  exprs) run_show exprs ;;
  hostile) run_san hostile ;;
  misuse) run_san misuse ;;
  all) run_walk; run_san hostile; run_san misuse ;;
  *) echo "usage: $0 walk|diagnose|exprs|hostile|misuse|all" >&2; exit 2 ;;
esac
