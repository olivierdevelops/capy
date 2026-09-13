> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Capy as a cross-arch assembler: the 4 hard problems

**Who this is for:** anyone considering Capy as a "universal assembler"
— one neutral source language, one backend library per CPU/OS. That
idea *works* for straight-line lowering (see
[`samples/assembly/`](https://github.com/olivierdevelops/capy/tree/main/samples/assembly),
which lowers a tiny source DSL to real x86-64 NASM). This doc walks the
**four hard problems** a real compiler back-end solves, with runnable
examples and real engine output for each.

The headline result: **two of the four are solvable inside the library**
using Capy's mutable `context` state (register allocation and fresh
labels — Problems 1 and 3, both shown hitting the *ideal* output below),
and **two are genuinely out of reach** because they need program
*reasoning* a template engine doesn't have (type-directed instruction
selection and ABI marshalling — Problems 2 and 4).

## The one root cause

Capy is a **syntactic** transpiler: it pattern-matches tokens and
expands templates. It has **no semantic model** — it does not know a
variable's *type*, does not track which *register* holds a value, does
not *count*, and cannot compute *per-position* decisions. Every one of
the four problems below is the same gap seen from a different angle.

A real compiler back-end (LLVM, GCC) keeps an in-memory model of the
program — types, a value graph, register liveness, a label counter — and
*reasons* over it. Capy has a *little* state — mutable `context` plus a
small inner DSL (`set`, `if`, `for`, `(add …)`/`(sub …)`) that runs
between statements — so a library can carry a **counter or a stack**
across the program. That's enough to mechanically *hand out* registers
or labels (Problems 1 and 3). What it can't do is *reason*: infer types,
analyse liveness, or index/compute per argument position (Problems 2 and
4). So the four problems split into two that a clever library can work
around with `context`, and two that are genuinely out of reach.

The workarounds have one of two shapes: **carry state in `context`**
(when a counter/stack suffices), or **push the missing knowledge back
into the source** (when real reasoning is required). The more you do the
latter, the less "neutral" and "high-level" your source language is.

Each problem below shows three things from the **same neutral source**:

- **✅ Ideal output** — what a real compiler (LLVM/GCC) produces by
  *reasoning* over the program.
- **❌ What Capy emits** — the real engine output, and why it's wrong.
- **Workaround** — how to recover correct output: a `context`-based
  allocator/counter *in the library* (Problems 1, 3), or a source
  rewrite that does the compiler's job by hand (Problems 2, 4).

---

## Problem 1 — Register allocation

A compiler decides *which register* holds each value and reuses
registers safely. Capy can't: a template just writes a fixed register
name. The stock `add` lowering hardcodes `rax`.

**What you'd write** — sum two independent pairs and combine them
(`(x+y)` and `(a+b)` must both be alive at once):

```
add x y      # want this result kept somewhere
add a b      # want this result too, then add the two
```

**✅ Ideal output** (what a real compiler produces) — it picks a second
register so both results stay live, then combines, no memory touched:

```asm
mov rax, [x]
add rax, [y]      ; rax = x + y
mov rbx, [a]      ; compiler chose a FREE register
add rbx, [b]      ; rbx = a + b
add rax, rbx      ; rax = (x+y) + (a+b)
```

**❌ What Capy emits** (real output):

```asm
mov rax, [x]
add rax, [y]      ; rax = x + y
mov rax, [a]      ; <-- CLOBBERS rax; the x+y result is GONE
add rax, [b]
```

The *stock* `add` template overwrites `rax` because it has no idea the
first result is still needed. But — unlike the other three problems —
**this one is fixable inside the library**, because Capy gives you
[mutable `context` state](../library-authoring.md) and a small inner
DSL (`set`, `if`, `(add …)`/`(sub …)` arithmetic) that runs *between*
statements. You can carry an allocator's state across the whole program.

**✅ Better — allocate registers in the library with `context`.** Keep a
"next free register" plus a small register stack in `context`, and pick
a fresh register per op:

```capy
context
    nf "rax"          # next free register
    t0 "?"            # top result register
    t1 "?"            # second result register
    sp 0              # stack depth
end

function bin                       # bin a b  ->  a + b into a fresh reg
    arg literal "bin"
    arg capture a ident
    arg capture b ident
    write `mov ${context.nf}, [${a}]
add ${context.nf}, [${b}]
`
    set context.t1 context.t0      # shift the result stack
    set context.t0 context.nf
    set context.sp (add context.sp 1)
    if context.sp == 1
        set context.nf "rbx"       # advance the free-register pointer
    end
    if context.sp == 2
        set context.nf "rcx"
    end
end

function fold                      # fold  ->  add top two results, pop one
    arg literal "fold"
    write `add ${context.t1}, ${context.t0}
`
    set context.t0 context.t1
    set context.sp (sub context.sp 1)
end
```

Source `bin x y` / `bin a b` / `fold` then produces the **ideal output
byte-for-byte** — distinct registers, no memory spill:

```asm
mov rax, [x]
add rax, [y]
mov rbx, [a]      ; library picked a fresh register from the pool
add rbx, [b]
add rax, rbx      ; (x+y) + (a+b)
```

So the workaround is *not* "spill everything to memory" — it's "write a
register allocator in the library." That's a real, working answer.

**Two engine constraints you must design around** (both verified):

1. **No array indexing in value position.** You can read `context.sp`
   but *not* `context.pool[context.sp]` — value paths support `.field`
   only. So you map a counter to a register name with an `if`-ladder (as
   above) or a scalar register-stack, not `pool[i]`.
2. **A function's own `write` doesn't see its own `set`s** — mutations
   are visible to *later* statements, not to the template that performs
   them. That's why the allocator advances `nf` *after* the `write`
   (for the next op) rather than before. Get the ordering backwards and
   you emit `mov ?, …`.

**Where it still stops.** What you've built is a **fixed, naive
strategy** — a stack/linear-scan allocator — not liveness-based graph
coloring. It is perfect for **stack-machine / tree-expression** code
(which is exactly the portable-IR shape you'd want anyway). It does
**not** give you: spilling when the register pool runs out, reusing a
register the moment a value becomes dead (no liveness analysis), or
correct allocation for values whose live range crosses branches and
loop back-edges. Those still need a real allocator. (The same
`context`-counter trick also solves Problem 3's fresh labels, below.)

---

## Problem 2 — Instruction selection by operand type/size

The right instruction often depends on the operand's **type or width**:
a 64-bit integer add is `add rax, …`; a `double` add is
`addsd xmm0, …`; a 32-bit add is `add eax, …`. Capy matches *syntax*,
not *types*, so it can't tell them apart.

**What you'd write** (assume `counter/step` are ints, `price/tax` are
doubles):

```
add counter step
add price tax
```

**✅ Ideal output** (what a real compiler produces) — from the *same*
neutral `add`, it reads the operand types and selects the right
instruction + register file:

```asm
; counter/step : i64  -> integer unit
mov rax, [counter]
add rax, [step]
; price/tax : f64     -> SSE unit
movsd xmm0, [price]
addsd xmm0, [tax]
```

**❌ What Capy emits** (real output) — identical integer code for both,
which is **wrong** for the floats:

```asm
mov rax, [counter]
add rax, [step]
mov rax, [price]   ; <-- should be movsd xmm0, [price]
add rax, [tax]     ; <-- should be addsd xmm0, [tax]
```

Capy has no type inference (beyond `type` pattern/enum *validation*),
so `add` can't branch on "is this a float?"

**Workaround** — encode the type in the op name, so each maps to its own
template (real output):

```
addi counter step     →   mov rax, [counter] / add rax, [step]
addf price tax         →   movsd xmm0, [price] / addsd xmm0, [tax]
```

**Cost:** the source author must know and spell out the type at every
op. The grammar can't do type-directed selection for them — the
intelligence lives in the human's choice of `addi` vs `addf`.

---

## Problem 3 — Fresh label generation (loops & branches)

Every loop/branch needs **unique** labels (`.L0`, `.L1`, …). A compiler
keeps a counter and gensyms a new one each time. Capy has no counter and
no gensym — a template emits the *same literal text* every time it runs.

**What you'd write** — two loops:

```
loop rcx
    nop
end
loop rdx
    nop
end
```

**✅ Ideal output** (what a real compiler produces) — a label counter
hands each loop its own fresh, non-colliding labels automatically:

```asm
.L0:                    ; loop #1
    cmp rcx, 0
    je .L1
    nop
    jmp .L0
.L1:
.L2:                    ; loop #2 — fresh labels, no collision
    cmp rdx, 0
    je .L3
    nop
    jmp .L2
.L3:
```

**❌ What Capy emits** (real output) — both loops use `.Lstart` /
`.Lend`, a **duplicate-label assembler error**:

```asm
.Lstart:                ; loop #1
    cmp rcx, 0
    je .Lend
    nop
    jmp .Lstart
.Lend:
.Lstart:                ; loop #2 — SAME labels! assembler rejects this
    cmp rdx, 0
    je .Lend
    nop
    jmp .Lstart
.Lend:
```

The *naive* template emits the same literal label text every time. But
like Problem 1, this is **fixable in the library** with a `context`
counter — the engine has no `${unique}` built in, but you can keep your
own:

```capy
context
    n 0
end
function loop
    arg literal "loop"
    arg capture reg ident
    block_closer end
    write `.L${context.n}:
    cmp ${reg}, 0
${indent 4 body}    jmp .L${context.n}
.Lend${context.n}:
`
    set context.n (add context.n 1)    # bump AFTER emit (the one-call lag, again)
end
```

Two loops now get distinct labels automatically (real output):

```asm
.L0:                    ; loop #1
    cmp rcx, 0
    nop
    jmp .L0
.Lend0:
.L1:                    ; loop #2 — fresh, no collision
    cmp rdx, 0
    nop
    jmp .L1
.Lend1:
```

**Cost / limits:** unique labels — solved. But the counter is purely
*positional* (Nth loop in source order), not tied to real control-flow
structure. It's enough for straightforward loops/branches; it won't help
with anything that needs to *reason* about the control-flow graph
(irreducible loops, computed jumps, label liveness). The alternative —
having the author name each label in the source — also works but puts the
bookkeeping back on them. Forget to make one
unique and you get the same collision back.

---

## Problem 4 — ABI argument marshalling (function calls)

To call a function, each argument goes in a specific place determined by
its **position** and the platform ABI: SysV x86-64 uses
`rdi, rsi, rdx, rcx, r8, r9`, then the stack; arm64 uses `x0…x7`. So
"the *i*-th argument → the *i*-th ABI register" is an **indexed,
positional** computation — and Capy can't index.

**What you'd write:**

```
call printf fmt count name
```

**✅ Ideal output** (what a real compiler produces) — it walks the
argument list *by index* and drops each into its ABI slot (and spills
arg #7+ to the stack automatically):

```asm
    mov rdi, fmt       ; arg 0 -> rdi
    mov rsi, count     ; arg 1 -> rsi
    mov rdx, name      ; arg 2 -> rdx
    call printf
```

**❌ What Capy emits** (real output) — it can match the argument *list*
(via a nonterminal repetition), but every argument renders identically;
it can't assign arg #0→`rdi`, #1→`rsi`, #2→`rdx`:

```asm
    ; arg -> ??? (which register? depends on position)
    mov ???, fmt
    ; arg -> ??? (which register? depends on position)
    mov ???, count
    ; arg -> ??? (which register? depends on position)
    mov ???, name
    call printf
```

The repetition gives you the list, but there's no "render the *n*-th
item differently" — no index variable, no positional lookup table.

**Workaround** — make the source spell out the register for each
argument (real output), or hand-write one function per arity
(`call1`, `call2`, `call3`):

```
setarg rdi fmt       →   mov rdi, fmt
setarg rsi count      →   mov rsi, count
setarg rdx name       →   mov rdx, name
call printf           →   call printf
```

**Cost:** the ABI table now lives in the author's head, repeated at
every call site, and must change for every target arch. The whole point
of an ABI abstraction is gone.

---

## Summary

| # | Problem | In-library with `context`? | Best resolution | What it costs |
|---|---------|----------------------------|-----------------|---------------|
| 1 | Register allocation | **Yes** — counter + register stack | Library allocator; hits ideal output for tree/stack-machine code | Fixed strategy only — no liveness, spilling, or cross-branch ranges |
| 2 | Instruction selection by type | **No** — needs type inference | Encode the type in the op name (`addi`/`addf`) | Author picks the instruction, not the compiler |
| 3 | Fresh labels | **Yes** — context counter | Library auto-numbers labels (`.L0`, `.L1`, …) | Positional only; no control-flow-graph reasoning |
| 4 | ABI marshalling | **No** — needs positional indexing | Spell out registers, or per-arity functions | ABI table duplicated per call site, per arch |

The split is the point: **1 and 3 need only *bookkeeping*** (a
counter/stack), which `context` provides, so the library can solve them
and even reach the ideal output. **2 and 4 need *reasoning*** (type
inference; per-position indexing/computation), which a template engine
fundamentally lacks — so the knowledge has to come from the source.

## The throughline

Capy is excellent at the *lowering* step — "this op becomes these
instructions on this target" — and, thanks to mutable `context`, it can
also carry the *bookkeeping* state (counters, register stacks, label
numbers) that a straight-line / stack-machine backend needs. That covers
a **thin macro assembler** or a **portable virtual ISA** very well,
including a naive register allocator and fresh-label generation.

What it cannot do is *reason* over a program model — infer types, analyse
liveness across control flow, or compute per-argument-position decisions.
Those are Problems 2 and 4, and they are where a real optimizing compiler
(LLVM/GCC) earns its keep.

**Practical takeaway:** design your universal-asm source as an
**explicit stack machine / fixed-register virtual ISA**. Then Problems 1
and 3 are handled by small `context`-based allocators/counters in each
backend library, and Problems 2 and 4 are answered by making the source
carry the type and the calling convention explicitly. With that shape,
"new arch = new library" genuinely holds — reach for LLVM/GCC only when
you need types and liveness *solved for you*.

**Continued:** [Cross-arch assembler: the remaining problems
(5–10)](asm-backend-remaining.md) — constant folding, struct layout,
stack frames, peephole optimization, ISA model differences, and linking.

See also: [How Capy parses & extracts content](../how-capy-parses.md)
for the lex → match → capture → render pipeline these examples build on.