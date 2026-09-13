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

for name in checked:
    print(f"  checked   {name}")
if failures:
    print("\nPUBLISHABILITY FAIL", file=sys.stderr)
    for f in failures:
        print("  ✗ " + f, file=sys.stderr)
    sys.exit(1)
print(f"\nPUBLISHABILITY PASS — {len(checked)} crate(s) publish-ready")
