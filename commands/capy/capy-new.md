---
name: capy-new
description: Scaffold a new Capy library targeting a specific output language.
arguments: <target-language>
---

# /capy-new

Use this when the user types `/capy-new <target>` (e.g. `/capy-new python`, `/capy-new json`).

## Steps

1. Read the target from `$ARGUMENTS`.
2. Create a new directory under `samples/` (or the user's chosen path) using `mkdir` and `capy init`.
3. Customize the scaffolded `lib.yaml` for the target:
   - `extension:` matches the target file extension.
   - Add a 1-line context if the target typically has a header section.
   - Add 2 example functions tuned to the target (e.g. `print` for Python; `set` for JSON).
   - Add a basic `file_template:`.
4. Replace the stub `script.capy` with something that exercises both example functions.
5. Run `capy run` and show the output.
6. Capture the golden via `CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden` if the user wants to commit.

## Target-specific defaults

- **python**: function `say <msg>` → `print({{ .msg }})`; context.imports list.
- **json**: empty template per function; `file_template:` is `{{ .context | toJSONIndent }}`.
- **sql**: function `select <cols> from <tbl> where <cond>` → SQL select.
- **makefile**: function `task <name> <cmd>`; file template renders `.PHONY` and each task.
- **html**: brace-delimited `component` block.
- **other**: ask the user for one or two example shapes and design from those.

## Output

Print the path to the new `lib.yaml`, `script.capy`, and the `capy run` command.
