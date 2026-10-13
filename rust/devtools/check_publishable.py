#!/usr/bin/env python3
"""Verify every publishable crate in the workspace could actually be published.

`cargo package` can't check the dependent crates until capy-core is on
crates.io — it resolves path dependencies against the registry even with
--no-verify. So check the two things that registry rejects, statically:

  1. crates.io requires `license` and `description`; `repository` is optional
     but a crate without one is unusable in practice.
  2. every `path` dependency must also carry a `version`, or cargo refuses the
     crate outright: a published crate has no path to follow.

Crates carrying `publish = false` are skipped — build artefacts and repo
tooling are intentionally not published.
"""
import json
import subprocess
import sys

REQUIRED = ("license", "description", "repository")

meta = json.loads(
    subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        capture_output=True, text=True, check=True,
    ).stdout
)

failures = []
checked = []
for pkg in meta["packages"]:
    # `publish` is null when unrestricted, [] when `publish = false`.
    if pkg.get("publish") == []:
        print(f"  skip      {pkg['name']} (publish = false)")
        continue
    checked.append(pkg["name"])
    for field in REQUIRED:
        if not pkg.get(field):
            failures.append(f"{pkg['name']}: missing `{field}`")
    for dep in pkg["dependencies"]:
        if dep.get("path") and dep["req"] == "*":
            failures.append(
                f"{pkg['name']}: path dependency `{dep['name']}` has no version "
                f"— cargo will refuse to publish this crate"
            )

# ARCH-002: capy-core keeps exactly one dependency (regex). PROP-2026-0008 R1: capy-ffi is an artefact
# crate that is never published and depends on capy-core alone.
by_name = {p["name"]: p for p in meta["packages"]}
core = by_name.get("capy-core")
if core is not None:
    normal = [d["name"] for d in core["dependencies"] if d.get("kind") in (None, "normal")]
    if normal != ["regex"]:
        failures.append(f"capy-core: ARCH-002 requires exactly one dependency (regex), found {normal}")
ffi = by_name.get("capy-ffi")
if ffi is not None:
    if ffi.get("publish") != []:
        failures.append("capy-ffi: must be `publish = false`")
    deps = [d["name"] for d in ffi["dependencies"] if d.get("kind") in (None, "normal")]
    if deps != ["capy-core"]:
        failures.append(f"capy-ffi: must depend on capy-core alone, found {deps}")

for name in checked:
    print(f"  checked   {name}")
if failures:
    print("\nPUBLISHABILITY FAIL", file=sys.stderr)
    for f in failures:
        print("  ✗ " + f, file=sys.stderr)
    sys.exit(1)
print(f"\nPUBLISHABILITY PASS — {len(checked)} crate(s) publish-ready")
