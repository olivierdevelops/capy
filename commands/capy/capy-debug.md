---
name: capy-debug
description: Diagnose why a Capy script isn't matching what the user expects.
---

# /capy-debug

Use when the user has a `lib.yaml` + `script.capy` that don't produce expected output, OR when they get a parse/validation error they don't understand.

## Triage

1. Run `capy check lib.yaml` first. If it fails, the library is malformed — fix that first.
2. Run `capy run lib.yaml script.capy`. Read the error carefully — it has line + column + caret.
3. Identify the failure mode:

### "no library function matches token X"

The parser tried every function and none matched at that position.

Common causes:
- A function's `args:` doesn't anticipate the user's source shape (missing literal, wrong type).
- A typo in the user's source (`improt` instead of `import`).
- A literal in args doesn't match the lexer's tokenization (e.g. user wrote `+ =` but lexer combined into one PUNCT token `+=`).

Fix: inspect the failing token, compare to each function's leading literal, adjust.

### "expected end of block" / "expected closer X"

A block opener consumed the body but couldn't find its closer.

Common causes:
- DEDENT happens too early (indentation off-by-one).
- The closer function isn't defined or is misnamed.
- For Mode B blocks, missing the `}` delimiter.

### "function X arg Y: value Z does not match pattern for type T"

Type validation failed.

Common causes:
- Pattern is too strict.
- Pattern is missing `^...$` anchors.
- User's source has surrounding quotes that the pattern doesn't expect (Capy strips quotes from string literals before validation, but doesn't strip them from idents).

### Output is empty / mostly empty

Common causes:
- `template:` is empty (intentional? if not, add one).
- Capture name in template doesn't match name in args (typo).
- Function never matches (so template never renders) — add a temp `print x` template to confirm.

### Output has extra newlines / whitespace

Common causes:
- Missing `{{-` `-}}` trimming.
- The template has a trailing newline that should be stripped.

## Iteration loop

```sh
capy check lib.yaml
capy run lib.yaml script.capy
# read error, edit lib.yaml or script.capy
# repeat until happy
CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden   # capture golden once stable
```
