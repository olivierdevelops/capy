# list-index-assign

A **dead-store eliminator** written entirely in a `.capy` library — a
demonstration of overwriting a `context` **list** element in place by
index (`set context.buf[i] …`).

## The idea: deferred emission + retroactive edit

A code generator often can't decide whether an instruction is needed
until it sees what comes later. The trick is to *not* write each
instruction straight to output. Instead:

1. **Buffer** every instruction into a `context` list (`buf`), instead of
   emitting it immediately.
2. **Track** side-tables as you go: per memory location, the buffer index
   of its last store (`laststore`) and whether that store has been read
   since (`readsince`).
3. **Rewrite the past** when the future makes a decision: when a new
   store to a location lands while the previous store was never read,
   that previous store is dead — overwrite its buffer slot with a comment
   via `set context.buf[idx] …`.
4. **Flush** the buffer in order at the end.

## Run

```
cargo run --manifest-path rust/Cargo.toml -p capy-cli --bin capy -- run samples/list-index-assign/lib.capy samples/list-index-assign/script.capy
```

Input (`script.capy`):

```
st x 1
st x 2
ld x
st x 3
st y 9
flush
```

Output: `st x 1` is eliminated (overwritten by `st x 2` before any read),
while `st x 2` survives (it's read by `ld x` before `st x 3` overwrites
it). `st x 3` and `st y 9` are the final live stores.

```asm
    ; (dead store to x eliminated)
    mov [x], 2
    mov rax, [x]
    mov [x], 3
    mov [y], 9
```

## Why it matters

"Capy emits text forward-only and never re-reads its output" is *not* a
hard limit: a library can keep its own structured history in `context`
and edit a buffered slot after the fact. The same mechanism back-patches
jump offsets and drops redundant reloads. The genuine wall is elsewhere —
analyses that need an unbounded fixpoint over a control-flow graph, or
inferring facts the source never states. See
[the cross-arch assembler design notes](../../docs/design/asm-backend-remaining.md).
