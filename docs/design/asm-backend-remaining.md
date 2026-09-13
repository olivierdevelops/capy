> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Cross-arch assembler: the remaining problems (5–10)

This continues
[Cross-arch assembler: the 4 hard problems](asm-backend-limits.md). That
doc covered register allocation, instruction selection, fresh labels,
and ABI marshalling — and found the key split: problems that need only
**bookkeeping** (a counter/stack) are solvable inside the library with
Capy's mutable `context`, while problems that need **reasoning** (type
inference, liveness, per-position computation) are out of reach.

This doc applies that same lens to the *next* six things a real
back-end does. Every example below is real engine output. The result is
a four-way classification:

| Badge | Meaning |
|-------|---------|
| ✅ **in-library** | Solvable today with `context` bookkeeping |
| ⚠️ **partial** | Works for the common cases; a residual reasoning case stays out |
| ❌ **out of reach** | Needs reasoning over the program or the emitted output |
| ⤷ **delegated** | Not Capy's job — the assembler/linker handles it |
| ◆ **design choice** | A constraint on *how you design the source*, not a blocker |

A fact used throughout: Capy's arithmetic helpers are `add`, `sub`,
`mul`, `div`, `mod`, `align`, and `percent` (source of truth:
`infra/helpers.go`). `div`/`mod`/`align` were
[added in this release](../whats-new.md) precisely because alignment math
used to run out of road — there's still no general *bitwise* op, but
`align n a` (round `n` up to the next multiple of `a`) covers the layout
and stack-frame cases directly.

---

## Problem 5 — Constant folding *and propagation* ✅

Evaluating `2 + 3` to `5` at compile time. Because helper calls run at
emit time, the library *can* fold **literal** constants:

```capy
function li
    arg literal "li"
    arg capture dst ident
    arg capture a int
    arg capture b int
    write `mov ${dst}, ${add a b}    ; folded ${a}+${b}
`
end
```

**✅ Real output** — `li rax 2 3`:

```asm
mov rax, 5    ; folded 2+3
```

**Constant *propagation* is also in-library.** The first version of this
doc classified propagation — "track that `x` was set to `5` three
statements ago and fold `x + 1` into `6`" — as out of reach. That was
wrong. `context` carries a map across statements, so the library can
record `x → 5` when it sees the assignment, look it up when `x` is used,
fold, and `clear` the map at a block boundary (new basic block /
function) so a stale value never propagates across a branch — exactly
the "clear on new branch or function" discipline a real folder uses:

```capy
context
    known {}
end
function let
    arg literal "let"
    arg capture name ident
    arg capture val int
    set context.known[name] val          # record x -> 5
    write `mov ${name}, ${val}
`
end
function addk
    arg literal "addk"
    arg capture name ident
    arg capture k int
    # Direct dynamic map read — `context.known[name]` resolves the
    # captured key (see "value-position indexing" below). Before that
    # landed this was a `for kk,v in context.known / if kk == name` scan.
    write `mov rax, ${add context.known[name] k}    ; PROPAGATED ${name}=${context.known[name]}
`
end
function clear
    arg literal "clear"
    set context.known {}                 # block boundary: forget all
    write `; --- block boundary ---
`
end
```

**✅ Real output** — `let x 5` / `addk x 1` / `clear`:

```asm
mov x, 5
mov rax, 6        ; PROPAGATED x=5  (folded 5 + 1)
; --- block boundary ---
```

The value flows: `x = 5` is recorded, `addk x 1` finds it and emits the
folded immediate `6`, and `clear` resets the map so nothing leaks past
the boundary. This is real constant propagation done with bookkeeping,
not dataflow reasoning — the library author encodes *where* the boundary
is (it's a source op), so Capy never has to infer it.

**Value-position indexing — the former gap, now closed.** The lookup
used to be written as a `for … / if kk == name` scan because a *dynamic*
map read in value or template position didn't resolve a computed key —
only literal `.field` paths and `set …[key]` *writes* took a dynamic
index. That's fixed: `context.known[name]` (and list `context.buf[i]`,
nested `grid[i][j]`, negative `buf[-1]`, computed `buf[(sub n 1)]`) now
read by index in both inner-DSL and `${…}` positions, with the same
semantics as the write side. See
[value-position indexing](value-position-indexing.md). The only residual
arithmetic gap is folding a mask like `n & 0xff`, which still needs a
**bitwise** op Capy doesn't have (it has `div`/`mod`/`align` but no
`and`/`shl`).

---

## Problem 6 — Struct layout & alignment ✅

Computing each field's byte offset and the struct's total size. The
running-offset half is pure bookkeeping — accumulate in `context` — and
the **alignment** half is now a one-helper call. This release added
`align n a` (round `n` up to the next multiple of `a`), which is exactly
what the C ABI needs: each field aligned to its size, struct padded to
its largest member.

```capy
context
    off 0
end
function field
    arg literal "field"
    arg capture name ident
    arg capture size int
    # align INLINE in the template — a same-function `set` wouldn't be
    # visible to its own write (the one-call lag).
    write `; ${name} @ offset ${align context.off size} (size ${size})
`
    set context.off (add (align context.off size) size)
end
function total
    arg literal "total"
    arg capture maxa int
    write `; struct size = ${align context.off maxa} bytes
`
end
```

**✅ Real output** — fields `id:8`, `flag:1`, `count:4`, largest member 8:

```asm
; id @ offset 0 (size 8)
; flag @ offset 8 (size 1)
; count @ offset 12 (size 4)    ; padded 9 -> 12 (4-byte aligned)
; struct size = 16 bytes         ; padded 13 -> 16 (8-byte aligned)
```

This is byte-for-byte the ABI-correct layout. The earlier version of
this doc marked alignment ⚠️ because the only arithmetic helpers were
`add`/`sub`/`mul`/`percent` and rounding up needs division or a bitwise
AND (`aligned = (off + a - 1) & ~(a - 1)`). Adding `align` (and the
underlying `div`/`mod`) closed that gap — the offset accumulation was
already in-library, and now so is the rounding.

> **One engine detail worth keeping.** The alignment is computed *inline
> in the template* (`${align context.off size}`), not via a
> same-function `set context.off (align …)` before the write — because a
> function's `write` doesn't see a `set` performed in its own body (the
> *one-call lag*; mutations are visible only to later statements). So you
> emit the aligned value directly and advance the running offset for the
> *next* field.

---

## Problem 7 — Stack frames (prologue / epilogue) ✅

A function entry must reserve a frame, align the stack to 16 bytes, and
save the callee-saved registers it actually clobbers; the exit restores
them. Both halves are in-library:

- **Frame size from locals** — bookkeeping, like Problem 6
  (accumulate sizes in `context`), and the **16-byte alignment** is now
  just `${align context.frame 16}`. ✅
- **Which callee-saved registers to push/pop** — this *looked* like it
  needed liveness, but the ABI rule is simply "save every callee-saved
  register you **write**." You don't need to analyse the body — you
  *record* it. The catch is ordering: the prologue is emitted before the
  body, yet the clobber set isn't complete until after it. **Deferred
  emission** resolves that: body ops append to a `context` buffer and add
  each register they touch to a `context` set; the closer runs last —
  when the set is complete — and emits the prologue, the buffered body,
  then the epilogue. ✅

```capy
function use                  # a body op: buffers its text, records the clobber
    arg literal "use"
    arg capture reg ident
    arg capture val int
    append context.body `    mov ${reg}, ${val}`
    set context.clob[reg] 1
end
function endfn                # runs last: the clobber set is now complete
    arg literal "endfn"
    for reg, one in context.clob
        write `    push ${reg}
`
    end
    for i, line in context.body
        write `${line}
`
    end
    for reg, one in context.clob
        write `    pop ${reg}
`
    end
    write `    pop rbp
    ret
`
end
```

**✅ Real output** — body clobbers `rbx` (twice) and `r12`:

```asm
compute:
    push rbp
    mov rbp, rsp
    push r12          ; pushed BECAUSE the body clobbers r12
    push rbx          ; pushed BECAUSE the body clobbers rbx (recorded once)
    mov rbx, 10
    mov r12, 20
    mov rbx, 30
    pop r12
    pop rbx
    pop rbp
    ret
```

Exactly the registers the body writes — no fixed over-save. The one
residual is the *optimization* beyond the ABI rule: omitting a save for a
register that's clobbered but provably dead across every call inside the
body. *That* needs real liveness (a fixpoint over the CFG, see Problem
8); the ABI-correct save set does not.

---

## Problem 8 — Peephole / dead-code elimination ✅

Removing redundant instructions (a reload of a value already in a
register, `mov rax, rax`, a jump to the next line). The naive framing is
"Capy emits text forward-only and never re-reads its output, so it can't
spot redundancy." But you don't *need* to re-read the output — you track
the relevant state **forward** in `context` and check it *before*
emitting. That's exactly the "hash the instruction, then verify if it
was already done" idea, and it works:

```capy
context
    rax_holds "?"     # what symbol rax currently caches; "?" = unknown
end
function load
    arg literal "load"
    arg capture v ident
    # check the cache FIRST (reads state a PRIOR statement set), then
    # update it for the NEXT one — one-call-lag safe.
    if context.rax_holds == v
        write `; (skipped: rax already holds [${v}])
`
    end
    if context.rax_holds != v
        write `mov rax, [${v}]
`
    end
    set context.rax_holds v
end
function clob
    arg literal "clob"
    write `xor rax, rax      ; clobbers rax
`
    set context.rax_holds "?"     # invalidate the cache
end
```

**✅ Real output** — `load x` / `load x` / `load y` / `load y` / `clob` /
`load y`:

```asm
mov rax, [x]
; (skipped: rax already holds [x])
mov rax, [y]
; (skipped: rax already holds [y])
xor rax, rax      ; clobbers rax
mov rax, [y]                       ; correctly re-emitted: clob invalidated
```

The redundant reloads are eliminated, and the clobber correctly
invalidates the cache so the reload after `clob` is *not* skipped. This
is real redundant-load elimination — the signature of "what's live in
rax" is the hash, and we verify it before emitting. The library author
encodes the clobber relationships (which ops invalidate which cached
values), so it's bookkeeping, not whole-program analysis.

**Even *backward-looking* dead-code elimination is in-library** — via
**deferred emission + retroactive rewrite**, not by "re-reading output."
The library buffers the instruction stream in a `context` list instead of
writing it immediately, keeps side-tables as it goes (per location: the
buffer index of the last store and whether it's been read since), and
when a new store lands while the previous one was never read, it
overwrites that earlier buffer slot in place — `set context.buf[idx] …`
([list elements are writable by index](../inner-dsl.md)). The flush at the
end emits the surviving slots:

```capy
function st
    arg literal "st"
    arg capture loc ident
    arg capture v int
    for l, idx in context.laststore
        if l == loc
            for l2, r in context.readsince
                if l2 == loc
                    if r == 0
                        set context.buf[idx] `    ; (dead store to ${loc} eliminated)`
                    end
                end
            end
        end
    end
    append context.buf `    mov [${loc}], ${v}`
    set context.laststore[loc] context.n
    set context.readsince[loc] 0
    set context.n (add context.n 1)
end
```

**✅ Real output** — `st x 1` / `st x 2` / `ld x` / `st x 3` (verified by
[`samples/list-index-assign/`](https://github.com/olivierdevelops/capy/tree/main/samples/list-index-assign)):

```asm
    ; (dead store to x eliminated)    ; x=1 overwritten before any read
    mov [x], 2                        ; survives — read by `ld x` below
    mov rax, [x]
    mov [x], 3
```

The "store 20 instructions ago" is just `context.buf[idx]`, and the
library edits it once the future is known. The same mechanism back-patches
jump offsets. So this isn't a forward-only limit — it's bookkeeping plus
an in-place edit.

**Where it *genuinely* stops (❌).** Two things stay out of reach, and
they're narrower than "backward dataflow":

- **Unbounded fixpoint over a control-flow graph.** Dead-store
  elimination over straight-line code (above) is a single forward pass.
  Real liveness across loops needs facts that *stabilise* over back-edges
  and *meet* at join points — iterate-until-nothing-changes. The inner
  DSL has bounded `for`-over-collections but no general "loop until
  stable," no lattice/meet operators, and no recursion, so a worklist
  solver over an arbitrary CFG isn't expressible.
- **Inferring structure the source never states** — reconstructing a CFG
  from computed jumps, alias analysis over arbitrary pointers. If the
  source doesn't tell you and you must *deduce* it, you're past
  bookkeeping.

Instruction *scheduling* (reorder for pipeline latency) is also still
out — it needs a cost model and a search, not a rewrite.

> **Related (also ❌): immediate encoding limits.** On ARM64 a large
> constant doesn't fit one `mov` and must become a `movz`/`movk`
> sequence; whether it fits depends on the *value*. Choosing the short
> form needs value-range reasoning. The naive fix — *always* emit the
> multi-instruction sequence — is correct but never optimal.

---

## Problem 9 — Flags vs compare-and-branch (ISA model) ◆

Architectures disagree on a fundamental shape: x86 sets **EFLAGS** with
`cmp` then branches on them (`jl`); RISC-V has **no flags** and uses a
single compare-and-branch (`blt`). This is **not a blocker** — it's a
**design choice** about how high-level your source op is. Keep the
neutral op at the level of "branch if less-than" and each backend lowers
its own way:

Same source `blt x y loop_top`:

**→ x86 backend** (real output):

```asm
cmp x, y      ; x86: set EFLAGS
jl loop_top   ; then conditional jump
```

**→ RISC-V backend** (real output):

```asm
blt x, y, loop_top   # RISC-V: compare-and-branch, no flags
```

**The lesson.** "New arch = new library" holds *as long as the source op
is abstract enough for every target to lower it.* If you instead exposed
`set_flags` + `jump_if_flag` in the source (an x86-ism), RISC-V couldn't
lower it. Design the virtual ISA around *intent* (`branch_lt`), not one
machine's mechanism — then the differences live entirely in the per-arch
templates, which is precisely what Capy is for.

---

## Problem 10 — Relocations, symbols & linking ⤷

Resolving a `call printf` to a real address, fixing up data references,
laying out sections — none of this is Capy's problem. Capy emits
**assembly text**; the **assembler and linker** resolve symbols, apply
relocations, and assign final addresses. You emit `call printf` or
`adrp x0, msg` as text and the toolchain does the rest.

This is worth stating plainly because it's easy to over-worry: a whole
category that sounds hard for a "universal assembler" simply isn't in
scope. (Endianness of emitted `.data` is similar — a per-arch constant
you bake into the template, e.g. `.quad` byte order, not a computation.)

---

## Summary

| # | Problem | Class | Why |
|---|---------|-------|-----|
| 5 | Constant folding & propagation | ✅ | Folds literals via `${add …}`; propagation via a `context` map + clear-on-boundary |
| 6 | Struct layout & alignment | ✅ | Offsets accumulate in `context`; `align n a` does the rounding |
| 7 | Stack frames | ✅ | Frame size + 16-byte `align`; callee-saved set = "record what you write" + deferred emission. Only the dead-across-call save *optimization* needs liveness |
| 8 | Peephole / dead code | ✅ | Forward redundancy cache + deferred-emission buffer rewrite (`set buf[i]`); only CFG-fixpoint & scheduling stay out |
| 9 | Flags vs compare-branch | ◆ | Not a blocker — design the source op abstractly, lower per arch |
| 10 | Relocations / linking | ⤷ | Delegated to the assembler/linker entirely |

## The throughline (both docs)

Across all ten problems, one boundary keeps reappearing:

- **Bookkeeping** — counters, stacks, running offsets, literal folding,
  constant propagation, redundancy caches, *and even retroactive edits of
  a buffered instruction stream* — is **in-library** because `context` +
  the inner DSL carry state across statements (and a `context` list can be
  rewritten by index, `set buf[i] …`). This bucket is much bigger than it
  first looked: propagation (P5), full alignment (P6/P7-size), redundant-
  load *and* dead-store elimination (P8) all turned out to be
  bookkeeping, not reasoning. The key realisation: "looking back at
  already-emitted code" is just keeping your own structured history and
  editing it — Capy never has to re-read its output. (Problems 1, 3, 5,
  6, 7, 8.)
- **Reasoning** — type *inference*, liveness as an **unbounded fixpoint
  over a control-flow graph**, and reconstructing structure the source
  never states — is **out of reach**, because the inner DSL has bounded
  iteration only (no loop-until-stable, no lattice/meet, no recursion) and
  no whole-program model. Note this is *narrower* than "backward
  dataflow": a single backward pass over straight-line code is fine; it's
  the *fixpoint over a graph* that isn't expressible. (Problems 2, 4,
  7-dead-across-call optimization, 8-CFG-fixpoint.)
- **Helper-arithmetic ceiling** — `div`/`mod`/`align` now exist, so
  alignment and integer layout math are in-library; only **bitwise** ops
  (masks like `n & 0xff`) remain unexpressible. (Was the blocker for
  Problems 6, 7 — now resolved except for masks.)
- **Not Capy's job** — symbol/relocation/link-time work is **delegated**
  to the toolchain. (Problem 10.)
- **Design choice** — model differences (flags vs branch) are handled by
  keeping the source op abstract and lowering per backend. (Problem 9.)

**Practical takeaway (unchanged, reinforced):** design the universal-asm
source as an **explicit stack-machine / fixed-register virtual ISA**
whose ops express *intent*. Then the bookkeeping problems are handled by
small `context` allocators/counters/accumulators in each backend
library, the model differences are absorbed by per-arch templates,
linking is the toolchain's job — and only the genuine *reasoning*
problems (types, liveness, optimization) remain as the reason you'd
reach for LLVM/GCC instead.

See also: [the first four problems](asm-backend-limits.md) ·
[How Capy parses & extracts content](../how-capy-parses.md) ·
[Library authoring](../library-authoring.md).