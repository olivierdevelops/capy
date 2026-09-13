> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Value-position indexing: `${context.buf[i]}` reads

**Status:** ✅ implemented (Phases 1 & 2) · Phase 3 (parser unification)
still optional/deferred
**Scope:** let a `[<expr>]` index appear in a *read* position — inside
`${…}` templates and inner-DSL expressions — the way it already can in a
*write* target (`set context.buf[i] …`).

> **Shipped.** `VarRef` is now step-based (`Steps []PathStep`), the value
> parser grows a postfix `[expr]` loop, a shared `descendRead` resolves
> one step over maps **and** lists, and the `${…}` atom scanner
> (`evalInterpPath`) understands brackets. Verified by
> `samples/value-index-read/` and the `Test{DescendRead,
> ResolvePathStepsIndex, EvalInterpPathIndex}` unit tests. Phase 3
> (retiring the ad-hoc `${…}` parser, §4.4 Option B) remains a future
> cleanup — the two parsers still co-exist.

---

## 1. The gap

Capy can already **write** a list/map element by a computed index:

```capy
set context.buf[i] "    ; rewritten"   # list element, in place
set context.known[name] val            # map key, dynamic
```

But it cannot **read** one back by a computed index. Both of these
silently fail today:

```capy
write `${context.buf[i]}`          # ${…} template position
if context.buf[i] == "x"           # inner-DSL value position
```

So the in-library idioms that need a lookup (constant propagation,
register-state tables, anything keyed by a captured name) fall back to a
linear `for`-scan workaround:

```capy
for k, v in context.known
    if k == name
        write `${v}`          # the value we wanted is bound as a loop var
    end
end
```

That works, but it's O(n) per lookup and obscures intent. Direct
`${context.known[name]}` is the affordance this doc specifies.

---

## 2. Why reads can't index but writes can

The two sides use **different path representations**, and only the write
side is index-aware.

### Write side — already index-capable

`inner_parser.go:parsePath` (≈line 170) parses a path into a rich
structure:

```go
// domain/ast.go
type Path struct {
    Root  string
    Steps []PathStep
}
type PathStep struct {
    IsIndex bool
    Field   string
    Index   Expr      // a full expression — `i`, `name`, `(sub n 1)`
}
```

It recognises `[expr]` (`inner_parser.go:189`) and stores the index as a
real `domain.Expr`. Resolution walks the steps in
`inner_evaluator.go:descend` (≈line 273), which **already** handles both
parents:

```go
case map[string]any: return p[toString(idx)], nil       // map key
case []any:          // int index, negative-from-end, bounds-checked
```

So the engine can already *navigate* a mixed map/list path by computed
index — it just only does so on the write path.

### Read side — three routes, all flat `[]string`

Every read route models a path as a plain `[]string` of dotted names,
with **no index step**:

| Route | Where | Path build | Resolver |
|---|---|---|---|
| Inner-DSL value (`if`, `set` RHS, `for … in`, helper args) | `value_parser.go:95–105` | `VarRef{Path []string}`, splits on `.` only | `eval` → `resolvePath` (`inner_evaluator.go:1189`) |
| `${…}` template | `interpolateRender` → `evalInterpAtom` (`inner_evaluator.go:760`) | `strings.Split(s, ".")` | `resolveRender` (line 920) |
| String-literal interpolation | `eval` `StringLit` (line 954) | `interpolateGeneric` callback | `resolvePath` |

`domain.VarRef` is literally `struct{ Path []string }`
(`domain/ast.go:64`), and all three resolvers loop with
`cur = m[step]` over `map[string]any` only — a `[]any` parent errors
("cannot access … on non-map"). There is **no place to put an index
expression** and **no list handling** on any read route.

The template route is the most divergent: it doesn't even use the
token-based parser. `evalInterpAtom` does `strings.Split(s, ".")`, so
`context.buf[i]` becomes the two literal segments `"context"` and
`"buf[i]"` — the brackets are swallowed into a map key that never exists.

---

## 3. Target capability

After this change, all of these read correctly in both `${…}` and
inner-DSL value position:

```capy
${context.buf[i]}              # list element by int index (local i)
${context.known[name]}         # map value by captured key
${context.grid[i][j]}          # nested index
${context.rows[i].name}        # index then field
${context.buf[(sub n 1)]}      # computed index expression
${context.buf[-1]}             # negative index → last element
```

Semantics match the write side: list indices are integers (negative
counts from the end), map keys are the index's string form.

---

## 4. Implementation

The guiding principle: **one path representation and one read-descend,
shared by every route.** Reuse what the write side already has.

### 4.1 Domain — give `VarRef` index steps

Replace the flat list with the same step model the write side uses:

```go
// domain/ast.go
type VarRef struct {
    Steps []PathStep   // was: Path []string
}
```

`PathStep` already exists. A dotted-only path is just steps with
`IsIndex == false`, so the representation is a strict superset.

**Blast radius — 6 consumers of `VarRef.Path`** (all small):

| File:line | Use | Migration |
|---|---|---|
| `value_parser.go:105` | constructs it | build `Steps` (see 4.2) |
| `inner_evaluator.go:961` | `eval` → `resolvePath(n.Path)` | pass `n.Steps` to new resolver |
| `inner_evaluator.go:529` | `evalRender` | same |
| `inner_evaluator.go:1176` | `evalExprFallback` bare-ident rule | read `Steps[0].Field` |
| `make_evaluator.go:273` | `cap.Expr.(VarRef)` type check | unchanged (type switch only) |
| `expr_to_text.go:33` / `translate_new_shape.go:180` | `strings.Join(n.Path, ".")` for source-text round-trip | render steps: `.field` and `[idx]` |

A lower-churn alternative is to **keep `Path []string` and add
`Steps []PathStep` alongside**, populating `Path` for dotted-only refs so
the six sites keep compiling unchanged and only the resolvers learn about
`Steps`. That avoids touching `expr_to_text`/`translate` but leaves two
fields to keep in sync — acceptable for a phased landing, but the clean
end state is a single `Steps`.

### 4.2 Parser — postfix `[expr]` in value position

In `value_parser.go` `parsePrimary`, after the dotted-path loop that
currently ends at line 104, add a postfix loop that mirrors
`inner_parser.go:parsePath`:

```go
// after building the dotted path…
steps := /* dotted names as PathStep{Field: …} */
for r.Peek().Kind == domain.TokLBrack {
    r.Advance()                       // [
    idx, err := parsePrimary(r, nil)  // full index expression
    if err != nil { return nil, err }
    if r.Peek().Kind != domain.TokRBrack {
        return nil, fmt.Errorf("expected ]")
    }
    r.Advance()                       // ]
    steps = append(steps, domain.PathStep{IsIndex: true, Index: idx})
    // a following `.field` or another `[…]` continues the path
}
return domain.VarRef{Steps: steps}, nil
```

**No ambiguity with list literals.** `parsePrimary` only treats `[` as a
list literal when it's the *first* token of a primary (line 138,
`parseListLit`). Here `[` is *postfix* after an identifier path, so the
two never collide. Interleaving with `.field` (for
`context.rows[i].name`) falls out naturally by alternating the two loops.

### 4.3 Resolver — one shared read-descend

Add a single helper that resolves one step against a parent, handling
both container kinds (this is the read-side twin of the write-side
`applyOp`/`descend` list branch already added):

```go
func descendRead(parent any, key any) (any, bool) {
    switch p := parent.(type) {
    case map[string]any:
        v, ok := p[toString(key)]
        return v, ok
    case []any:
        i, ok := key.(int64)
        if !ok { return nil, false }
        n := int64(len(p))
        if i < 0 { i += n }            // negative-from-end, like writes
        if i < 0 || i >= n { return nil, false }
        return p[int(i)], true
    }
    return nil, false
}
```

Then `resolvePath` and `resolveRender` change from "loop over `[]string`,
`cur = m[step]`" to "loop over `[]PathStep`": for a field step pass
`step.Field`; for an index step `eval(step.Index)` first, then
`descendRead(cur, idxVal)`. Both resolvers collapse onto the same walk —
arguably a simplification, since the map-only special-casing goes away.

### 4.4 Template route — the harder half

The `${…}` path does **not** go through `value_parser`; it has its own
ad-hoc string tokeniser (`evalInterpAtom`, `tokeniseInterpRuntime`,
`splitInterpPipeRuntime`). Two options:

- **Option A (surgical).** Teach `evalInterpAtom` to recognise a postfix
  `name[...]`: instead of a bare `strings.Split(s, ".")`, scan the atom
  for `[`, split the dotted head from the bracketed remainder, and
  recursively `evalInterp` each bracket's contents to get the index
  value, then `descendRead`. Smallest change; keeps the existing template
  semantics (tolerant empty on miss, the double-unescape rule, pipe
  handling) untouched. Risk: the atom string-scanner must correctly skip
  `]` inside nested brackets / quoted keys (`m["a]b"]`).

- **Option B (unify).** Retire the ad-hoc `${…}` parser and route each
  atom through the token-based `value_parser` + `eval`, so template and
  value position share one grammar and one resolver. Cleanest long-term —
  one code path, automatic feature parity — but it's a behavioural-risk
  refactor: the template route has special tolerances (missing path →
  empty string, the two-pass string-literal unescape at
  `inner_evaluator.go:748`, the `|` pipe operator) that must be preserved
  bit-for-bit or goldens shift.

**Recommendation:** ship **Option A** for the index feature (low blast
radius, matches the rest of this change), and track **Option B**
separately as a parser-unification cleanup — it's worth doing but
shouldn't gate indexing.

### 4.5 `interpolateGeneric` callback

The string-literal path (`eval` `StringLit`, line 954) calls
`interpolateGeneric` with a `func(path []string)` callback. Once
`resolvePath` is step-based, give this callback the same postfix-index
parse (or, under Option B, the unified parser handles it for free).

---

## 5. Semantics to pin down

| Question | Proposed answer | Rationale |
|---|---|---|
| Out-of-range / missing **in a template** | empty string | matches today's tolerant `${…}` (missing path → `""`) |
| Out-of-range / missing **in value position** | `nil` (falsy), not an error | lets `if context.buf[i]` guard cleanly; mirrors map-miss returning `nil` |
| List index type | must be `int64`; non-int → miss (nil/empty) | same rule as the write side |
| Map key | `toString(idx)` | identical to write side, so reads and writes agree |
| Negative index | counts from end (`-1` = last) | parity with write `descend` |
| `len` of a list | already supported via the `len` helper | no new work |

Keeping read and write semantics **identical** is the main correctness
lever: `set context.buf[i] x` then `${context.buf[i]}` must round-trip.

---

## 6. Phasing

1. **Phase 1 — value position.** Domain `Steps`, `value_parser` postfix
   loop, `descendRead`, switch `resolvePath`/`resolveRender` to steps.
   Unblocks `if context.buf[i]`, `set y context.m[k]`, helper args.
   Small, self-contained, fully testable without touching the template
   parser.
2. **Phase 2 — template position (Option A).** Extend `evalInterpAtom`.
   Unblocks `${context.buf[i]}`. This is what deletes the `for`-scan
   workaround from real libraries.
3. **Phase 3 (optional) — unify (Option B).** Retire the ad-hoc `${…}`
   parser; one grammar everywhere. Pure cleanup, no new surface.

Phases 1 and 2 are independently shippable and independently valuable.

---

## 7. Testing

- **Unit (resolver):** `descendRead` over map, list (positive, negative,
  out-of-range), wrong-type index, nested `grid[i][j]`, mixed
  `rows[i].name`.
- **Round-trip:** `set context.buf[i] v` then read `${context.buf[i]}`
  returns `v`; write a map key then read it back.
- **Golden samples:** add `samples/value-index-read/` — rewrite the
  constant-propagation lib (currently the `for`-scan idiom) to use
  `${context.known[name]}` directly and assert identical output. Extend
  `samples/list-index-assign/` with a read-back assertion.
- **Tolerance:** `${context.buf[99]}` renders empty, doesn't error;
  `if context.missing[k]` is falsy.
- **No-regression:** existing dotted-path goldens (`a.b.c`) unchanged —
  they're just index-free step lists now.

---

## 8. Risks & alternatives

- **Two ad-hoc parsers (template vs. value).** The single biggest source
  of effort and drift. Option A sidesteps it at the cost of duplicating
  the bracket logic in the string-scanner; Option B fixes it but risks
  golden shifts. The phasing isolates that risk to Phase 3.
- **`VarRef` field migration.** Six call sites; the "keep `Path`, add
  `Steps`" variant (4.1) reduces this to the resolvers only, at the cost
  of a transient dual representation.
- **Do-nothing alternative.** The `for k,v in … / if k == name` scan
  already works and is O(n) over typically-small tables. If no library
  hits a hot lookup path, this stays a quality-of-life win, not a
  correctness fix — which is why it's documented as a proposal rather
  than shipped alongside the write-side change.

---

## 9. Summary

The write side already has the representation (`PathStep` with an `Index`
expression) and the navigation (`descend` over maps **and** lists). The
read side is three parallel routes hard-coded to dotted `[]string` over
maps only. "Supporting value-position indexing" is therefore mostly an
act of **unification**: a step-based `VarRef`, a postfix `[expr]` parse
mirroring the write path, and one shared `descendRead` — with the only
genuinely new work being teaching the standalone `${…}` string-parser
about brackets (Phase 2). Keep read and write semantics identical and the
feature is a strict superset of today's behaviour.

See also: [Inner DSL (run blocks)](../inner-dsl.md) ·
[cross-arch assembler design notes](asm-backend-remaining.md) (where the
`for`-scan workaround appears in the constant-propagation example).