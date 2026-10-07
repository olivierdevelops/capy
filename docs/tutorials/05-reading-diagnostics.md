---
title: "Tutorial 5 · Reading diagnostics"
---

# Tutorial 5 · Reading diagnostics

**Time:** 10 minutes.
**You will need:** a built `capy` and a checkout of this repository — every
command below runs against `samples/parse-recovery/`, whose output is
golden-checked in CI, so what you see is what this page says.

**What you will learn:** how to stop debugging a DSL one line per run.

---

## The problem this solves

`capy run` is a transpiler. Given a file with a mistake in it, the only honest
thing it can do is refuse: half-parsed source would produce half-correct target
code. So it reports the first error and emits nothing.

That is right for building, and painful for fixing. A file with four mistakes
costs four edit-run cycles, and you cannot see the shape of the damage.

`capy ast` answers the other question. It parses the same file, **recovers**
past each failure, and hands you the tree plus one diagnostic per region it
could not use.

```text
   ┌──────────────┐        ┌──────────────┐
   │   capy run   │        │   capy ast   │
   ├──────────────┤        ├──────────────┤
   │ first error  │        │ EVERY error  │
   │ no output    │        │ full tree    │
   │ "am I done?" │        │ "what's wrong?"│
   └──────────────┘        └──────────────┘
```

---

## Step 1 — a file that works

```sh
capy run samples/parse-recovery/lib.capy samples/parse-recovery/script.capy
```

```text
[ ] write the proposal (due 2026-09-16)
[x] read the release notes
[ ] ship the samples (due 2026-09-30)
--- end of list ---
```

The library is three functions — `task "NAME" due "DATE"`, `done "NAME"`, and a
bare `list`. Nothing unusual; read
[`lib.capy`](https://github.com/olivierdevelops/capy/blob/main/samples/parse-recovery/lib.capy)
if you want the detail.

Now look at the same file as a tree:

```sh
capy ast samples/parse-recovery/lib.capy samples/parse-recovery/script.capy
```

```text
task 1:1-1:43
  name = "\"write the proposal\"" 1:6
  when = "2026-09-16" 1:31
done 2:1-2:30
  name = "\"read the release notes\"" 2:6
task 3:1-3:41
  name = "\"ship the samples\"" 3:6
  when = "2026-09-30" 3:29
list 4:1-4:5
```

One line per node, indented by depth, each carrying
`start_line:start_col-end_line:end_col`. Captures show the source text they took
and where it started. The exit code is `0` — a clean parse.

!!! note "Why `name` has quotes and `when` does not"
    `name` is captured as a `string`, so its **source text** includes the quote
    characters you typed. `when` is a pattern-validated type whose token is the
    bare date. The tree shows source text, not evaluated values — which is what
    a transpiler needs.

---

## Step 2 — break it

[`broken.capy`](https://github.com/olivierdevelops/capy/blob/main/samples/parse-recovery/broken.capy)
is the same file with two mistakes planted in it:

```text
1  task "write the proposal" due "2026-09-16"
2  task "missing its due date"                  <- no `due`, no date
3  done "read the release notes"
4  tsak "a typo in the keyword"                 <- `tsak`
5  list
```

Ask `run` about it:

```sh
capy run samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
```

```text
error: expected `due`, found end of statement in `task`
  2 │ task "missing its due date"
```

One error, exit `1`, no output. Note what the message already does for you: it
names the shape that got furthest (`task`) and what that shape wanted next
(`due`). Before 0.22.0 this said only "no library function matches" — true, and
useless.

But line 4 is still out there, and `run` has not mentioned it.

---

## Step 3 — see all of it

```sh
capy ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy
```

```text
task 1:1-1:43
  name = "\"write the proposal\"" 1:6
  when = "2026-09-16" 1:31
done 3:1-3:30
  name = "\"read the release notes\"" 3:6
list 5:1-5:5
<error> 2:1-2:28  3 token(s) skipped
<error> 4:1-4:29  3 token(s) skipped
error[E0001] 2:1: expected `due`, found end of statement in `task`
error[E0001] 4:1: no library function matches token "tsak"
```

Read it in three parts:

| Part | Meaning |
|---|---|
| `task`, `done`, `list` lines | the **3 statements that parsed** — note the line numbers jump 1 → 3 → 5 |
| `<error> 2:1-2:28  3 token(s) skipped` | a **region the parser gave up on**, and how much it threw away to resume |
| `error[E0001] …` | the **diagnostic**: severity, stable code, position, message |

Recovery is what connects them. On hitting line 2 the parser records the error,
skips forward to the start of the next statement, and carries on — which is why
line 3 and line 5 are in the tree at all. One command, the whole picture.

!!! tip "Where did the output go?"
    The tree goes to **stdout**, the diagnostics to **stderr**. So
    `capy ast … > tree.txt` captures a clean tree, and `capy ast … 2>&1` gets
    you what you see above. That split is what makes `capy ast` pipeable.

---

## Step 4 — read the codes

Each diagnostic carries a stable code. `E0001` means "no library function
matches the statement" — both of ours are `E0001`, because a missing required
argument and an unknown keyword both end with the parser unable to match a
statement.

Codes are the part you may safely build tooling on: a message may be reworded to
be clearer, a published code never changes meaning. The full list, including
which codes are reserved but not yet emitted, is in
[Diagnostics](../diagnostics.md).

---

## Step 5 — machine-readable

Everything above is available as JSON:

```sh
capy ast samples/parse-recovery/lib.capy samples/parse-recovery/broken.capy --json
```

stdout carries **one JSON document and nothing else**, so it pipes directly:

```sh
capy ast lib.capy broken.capy --json \
  | jq -r '.diagnostics[] | "\(.primary.start_line):\(.primary.start_col) \(.code) \(.message)"'
```

```text
2:1 E0001 expected `due`, found end of statement in `task`
4:1 E0001 no library function matches token "tsak"
```

That is the whole editor-integration loop in one line. Each `tree.errors[]`
entry also carries a `diagnostic_index` pointing back into `diagnostics[]`, so a
tool can highlight the region and show the message together. The full shape is
in [AST JSON schema](../ast-json.md).

---

## Step 6 — fix and confirm

Give line 2 its date and correct the typo on line 4, then:

```sh
capy ast lib.capy broken.capy ; echo $?
```

Exit `0` and no `<error>` lines mean the file is clean — at which point
`capy run` will produce output. Use the exit code to gate a script:

| Exit | Meaning |
|---|---|
| 0 | clean parse |
| 1 | diagnostics were produced, or the file could not be read |

---

## The loop, in one picture

```text
        write source
             │
             ▼
     ┌──▶ capy ast ──── exit 0 ────▶ capy run ──▶ output
     │        │
     │     exit 1
     │        │
     └── fix every reported region ◀─┘
         (not just the first)
```

---

## Embedding this

The CLI is a thin wrapper. From Rust:

```rust
let result = lib.parse(source);
if !result.is_clean() {
    for d in &result.diagnostics {
        eprintln!("{} {}:{}: {}", d.code, d.primary.start_line, d.primary.start_col, d.full_message());
    }
}
for stmt in &result.tree.stmts { /* what parsed */ }
for region in &result.tree.errors { /* what did not */ }
```

`ParseResult.tree` is **always** returned, so checking `stmts` alone will not
tell you whether the parse succeeded — check `is_clean()`. See
[Embed Capy in a Rust program](../embedding.md).

---

## Where next

- [Diagnostics](../diagnostics.md) — severities, labels, cascade control, limits
- [AST JSON schema](../ast-json.md) — the full `--json` contract
- [Errors & debugging](../errors-and-debugging.md) — the error messages themselves
- [Troubleshooting](../troubleshooting.md) — when it is the library, not the script
