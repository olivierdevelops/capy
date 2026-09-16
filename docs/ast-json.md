---
title: AST JSON schema
---

# AST JSON schema

The machine-readable form of a parse, produced by `capy ast --json`. Written by
Capy's own JSON writer — `capy-core` has exactly one dependency and keeps it, so
there is no serde in the engine.

```sh
capy ast lib.capy script.capy --json | jq .
```

## Stability

The root carries `schema_version`, currently **1**.

- Fields may be **added** without a bump. Ignore keys you do not recognise.
- Removing a field, or changing what one means, bumps the version.
- Diagnostic **codes** (`E0001`, …) are part of the contract too: a code's
  meaning does not change once published.

## Root

```json
{
  "schema_version": 1,
  "tree": { ... },
  "diagnostics": [ ... ]
}
```

`diagnostics` is empty when the parse was clean. **A non-empty `tree.stmts` does
not mean success** — check `diagnostics` or `tree.errors` first.

## `tree` — a block

| Field | Type | Meaning |
|---|---|---|
| `stmts` | array of node | The statements that parsed |
| `errors` | array of error node | Regions that did not parse |
| `is_verbatim` | bool | Body captured as raw bytes (`block_verbatim`) |
| `verbatim_text` | string | Those raw bytes |

`stmts` and `errors` are **parallel**: the statements that parsed are in one,
the regions that did not in the other. Interleaving is recoverable from spans.

## Node

| Field | Type | Meaning |
|---|---|---|
| `func` | string | The library function that matched |
| `span` | span or null | The whole statement, including block body and closer |
| `captures` | object | Argument name → capture |
| `leading_comments` | array of span | Comments immediately above this statement |
| `body` | block or null | Block body, when it has one |
| `closer` | node or null | The closing statement, when it has one |
| `sections` | object | Named sub-bodies of a multi-section block |

## Capture

| Field | Type | Meaning |
|---|---|---|
| `text` | string | The captured source text |
| `is_expr` | bool | Whether it carries a parsed expression |
| `span` | span or null | Exactly the tokens this value came from |
| `sub` | array of node | Sub-matches, when the capture's type names a function |

## Span

```json
{ "start_line": 1, "start_col": 8, "end_line": 1, "end_col": 12 }
```

1-indexed and source-absolute. **`end_col` is exclusive** — one past the last
byte — so `end_col - start_col` is a width on a single line.

**`null` means unset**, not "position zero". A capture that fell back to its
default consumed nothing and therefore has no source range; rendering that as
zeros would be indistinguishable from a real position.

**A node's `span` excludes its `leading_comments`.** A formatter needs the node's
range without them and the comments' ranges separately; folding the two together
loses one irrecoverably.

Byte offsets are not present yet.

## Error node

| Field | Type | Meaning |
|---|---|---|
| `span` | span | The skipped region |
| `tokens` | array of string | What was there, verbatim |
| `diagnostic_index` | int | Index into the root `diagnostics` array |

## Diagnostic

| Field | Type | Meaning |
|---|---|---|
| `severity` | `"error"` \| `"warning"` | |
| `code` | string | Stable, e.g. `"E0001"` |
| `message` | string | Including the context clause |
| `primary` | span | Where the problem is |
| `labels` | array | Secondary annotations: `{span, text}` |
| `help` | string or null | A suggestion, e.g. did-you-mean |
| `context` | array | `{shape, arg_index, arg_name}` — where the matcher was |

See [diagnostics](diagnostics.md) for how a message is chosen.

## Worked example

```sh
$ capy ast lib.capy script.capy --json | jq '.tree.stmts[0]'
```

```json
{
  "func": "import",
  "span": { "start_line": 1, "start_col": 1, "end_line": 1, "end_col": 12 },
  "captures": {
    "name": {
      "text": "json",
      "is_expr": false,
      "span": { "start_line": 1, "start_col": 8, "end_line": 1, "end_col": 12 },
      "sub": []
    }
  },
  "leading_comments": [],
  "body": null,
  "closer": null,
  "sections": {}
}
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Clean parse |
| 1 | The parse produced diagnostics, or the file could not be read |

The tree is printed either way, so a tool can inspect a broken file.

## A note on sensitivity

The JSON embeds source text (as rendered output always has). Treat
`capy ast --json` output as exactly as sensitive as the script it came from
before putting it in a CI log.
