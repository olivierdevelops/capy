> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Migration: Go templates → write-style

## Goal

Remove `infra/template_engine.go` so the surface DSL is **one
language** (Capy inner-DSL + `write` interpolation), not two.

## Progress

This session migrated **40 sample libraries** from raw Go-template
`file_template:` / per-function `template:` blocks to write-style
`file_template ... end` blocks. All migrated samples produce
byte-identical golden output. The full list of migrated libs is
visible in git log (`feat(samples): write-style migration — batch
1..10`).

## Remaining work, broken into phases

### Phase A — finish sample migration (~70 files)

**Blocked on engine features** (7 YAML libs use these; can't be
hand-converted until the engine supports them):

| Lib | Blocker |
|---|---|
| `transpile-blog/lib.yaml` | `{{ range $k, $v := .map }}` map iteration |
| `transpile-systemd/lib.yaml` | same |
| `transpile-makefile/lib.yaml` | same |
| `transpile-gh-actions/lib.yaml` | `{{ range $i, $t := .list }}` index iteration |
| `transpile-xstate-machine/lib.yaml` | nested ranges + `{{ $var := . }}` capture |
| `interactive-breakout/lib.yaml` | large; many nested ranges |
| `interactive-snake/lib.yaml` | same |

**Inner-DSL extensions needed before we can finish:**
1. `for k, v in MAP ... end` — two-var iteration over maps.
2. `for i, x in LIST ... end` — index iteration over lists.
3. (Optional) `let X = EXPR` inside the renderer scope, to capture
   outer-loop values from inner ones (xstate uses this).

**Mechanical .capy migrations remaining** (62 files still use
`{{ ... }}` syntax in their `file_template:` or per-function
`template:`). These can be converted by hand following the patterns
established in this session — see "Conversion patterns" below.

### Phase B — engine swap

`orchestrator/features/translate_new_shape.go` currently
**translates the inner-DSL write-style body into Go-template
source text**, which is then handed to `infra/template_engine.go`.
This indirection is what keeps `template_engine.go` load-bearing
even after the surface migration completes.

To drop the engine entirely:

1. Reimplement helpers as Go funcs callable from the inner
   evaluator: `unquote`, `toQuoted`, `indent`, `pascalCase`,
   `toJSON`, `toJSONIndent`, `snakeCase`, `dasherize`, `upper`,
   `lower`, `trimSuffix`, `trimPrefix`, `join`, `split`, `len`,
   `add`, `sub`, `mul`, `percent`, `stars`, `camelCase`,
   `nonEmpty`, `unescape`. (See `infra/template_engine.go`'s
   `funcs` map for the full list.)
2. Replace `translateNewShape`'s "emit Go template syntax" path
   with a direct evaluator that walks the inner-DSL AST
   (`WriteStmt`, `IfStmt`, `LoopStmt`, etc.) and writes output
   directly to a buffer.
3. Delete `infra/template_engine.go`, `infra/template_engine_test.go`,
   `orchestrator/features/make_template_renderer.go`, and all
   `tplE := infra.TemplateEngine{}` + `MakeTemplateRenderer`
   wiring in `app.go`, `run.go`, `capy.go`,
   `orchestrator/features/make_evaluator.go`.

### Phase C — cleanup

Delete the legacy `file_template:` (colon form) parsing path from
`rust/src/infra/capy_lib_parser.rs` once no sample uses it. Same for the
legacy `template:` field on `RawFunction` once every function
declares its output via `write` in the function body.

## Conversion patterns (used to migrate the 40 libs in this session)

### File template body

```yaml
file_template: |
  <h1>{{ .context.title | unquote }}</h1>
  {{ range .context.items }}- {{ . }}
  {{ end }}
```

becomes

```
file_template
    write `<h1>${unquote context.title}</h1>
`
    for it in context.items
        write `- ${it}
`
    end
end
```

### Per-function template

```yaml
greet:
  args: [...]
  template: "Hello, {{ .name | unquote }}!\n"
```

becomes (inside a `function greet ... end` block):

```
write `Hello, ${unquote name}!
`
```

### Multi-arm if/else if

Inner DSL supports `else if` chains natively:

```yaml
{{- if eq .kind "cube" }}
  geo = new BoxGeometry();
{{- else if eq .kind "sphere" }}
  geo = new SphereGeometry();
{{- else }}
  geo = new PlaneGeometry();
{{- end }}
```

becomes

```
if eq kind "cube"
    write `geo = new BoxGeometry();
`
else if eq kind "sphere"
    write `geo = new SphereGeometry();
`
else
    write `geo = new PlaneGeometry();
`
end
```

### Helpers as function-call interpolation

`{{ .x | helper }}` → `${helper x}`. Helpers chain by nesting:
`{{ .x | upper | snakeCase }}` → `${snakeCase (upper x)}`. (None
of the 40 libs in this session needed the chained form; all
were single-helper.)

### Gotchas surfaced during migration

- **Backslash-n inside backticks**: `\n` unescapes to a real
  newline. To emit a *literal* `\n` (e.g. for `printf "%s\n"`
  in bash) write `\\n`.
- **Trailing newline matters**: many libs' YAML
  `file_template: |` style strips one trailing newline. The
  write-style equivalent must include the final `\n` explicitly
  in the backtick if the golden expects one.
- **`options` declaration**: in .capy types, use positional
  strings `options "a" "b" "c"`, NOT a YAML-style list
  `options ["a", "b", "c"]`. The .capy parser tokenises the
  latter incorrectly.
- **Dotted function names** like `scene.create_sphere` work
  fine in `function NAME` declarations.
- **Reserved-looking names**: `function import`, `function if`,
  `function end` all work — inside a `function NAME` block the
  name is just a key, not a manifest-level directive.

## Running total

| Status | Count |
|---|---|
| Migrated total | 67 / ~107 lib files |
| Remaining .capy with `{{}}` (body) | ~15 |
| Remaining YAML | 2 (interactive-breakout, interactive-snake) |
| `template_engine.go` deletable? | No — still the rendering backend |

## Engine improvements landed this session

Three gaps from the previous analysis are now closed:

1. **Two-var `for`** — `for k, v in MAP` (sorted keys) and
   `for i, x in LIST` (with index). domain.LoopStmt.KeyVar threads
   through parser → translator → evaluator.
2. **Chained / nested helpers in interpolations**:
   - `${a | upper | toQuoted}` → `toQuoted (upper .a)`
   - `${toQuoted (upper a)}` → `toQuoted (upper .a)`
   - Tokeniser keeps parenthesised groups as one atom; pipe form
     rewritten to nested prefix calls (`splitInterpPipe`).
3. **`context.X` references the ROOT** even when used inside a
   range. Translator emits `$.context.X` not `.context.X`.
   (Previously, nested loops looking up `.context.fields` against
   the iteration variable silently produced empty output.)

Plus one related fix:

4. **Brace adjacency escape** — a literal `{` that ended up
   adjacent (after expansion) to a `${...}` interpolation or a
   `{{`/`}}` escape would produce `{{...` in the output and trip
   Go template's lexer with "unexpected `{` in command". The
   translator now looks ahead via `nextEmitStartsWithBrace` and
   escapes the leading brace via `{{"{"}}` whenever there's a
   collision. Bug surfaced in transpile-websocket-server's
   `struct{}{}` patterns inside the Go output.

## Inner-DSL gaps still blocking remaining libs

### ~~1. Two-var `for`~~ — LANDED

Closed this session. See "Engine improvements" above.

### ~~2. Chained / nested helper calls~~ — LANDED

Closed this session. See "Engine improvements" above.

### 3. Arithmetic in expression positions (3+ libs)

Today: `set context.x (add a b)` → `inner call "add" not allowed in
expression`. `add` / `sub` / `mul` / `percent` exist only in template
helpers.

Needed: a small set of arithmetic primitives callable from inner-DSL
expressions (not just templates).

Blockers:
- `reading-log` — `{{ $total = add $total .pages }}` accumulator
- Anything that wants to compute totals/percentages before rendering

### 4. State mutation inside `file_template` (1+ libs)

Today: `file_template` translator rejects residual `set/append/let`.

Needed: relax the rule, OR provide a `let X = EXPR` inside the renderer
that doesn't count as state mutation.

Blockers:
- `lib-composition` — separator-unless-first via a `first` flag
- Workaround for libs that need a small loop-local counter

### 5. Multi-file (`file "X":`) bodies (6 libs)

Today: legacy Go-template form parses fine. Write-style form
(`\x00NEW_SHAPE\x00` sentinel + translation) works but each lib
has several big bodies that need converting.

Mechanical work, no engine blocker. Done last because each is large.

Blockers:
- `android-app`, `ios-app` — 6-7 file blocks each
- `multi-file-project`, `libtorch-train` — 5-6 file blocks
- `backend-with-tests`, `webapp-trio` — 3 each

### 6. `transpile-threejs` weirdness (1 lib)

Threejs file_template, when converted to write-style with nested
`for` / `if eq m.kind "X"` chains, produced `state-mutation
statements aren't allowed in file_template` despite none being
visible in source. Some statement inside the JS body string is
being parsed as a CallStmt. Needs investigation before threejs can
migrate.

Cleanest reproducer: take the migrated threejs lib (in git history at
the `feat(samples/threejs)` commit, reverted shortly after) and bisect
the file_template body to find the offending fragment.