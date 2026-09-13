> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Design — unified `write:` block

> Status: **proposed, not implemented**. Captured here so the
> rationale, edge cases, and migration story aren't lost.
> Implementation is a multi-week swing; merging this doc commits us
> only to the shape, not the schedule.

## Motivation

Today a library author writes **two** blocks per function:

```
function import
    arg literal "import"
    arg capture name ident
    template_str ""               # ← Go text/template (output)
    run:                          # ← Capy inner DSL (state)
        append context.imports name
end
```

Two surface languages for one conceptual rule:

| Block       | What it expresses                              | Syntax  |
|-------------|------------------------------------------------|---------|
| `template:` | Body text contribution                         | Go      |
| `run:`      | Context mutation                               | Capy    |

This works, but newcomers consistently hit the same friction:

1. "Which block does what?" — the names don't both name what they do
   ("run what?").
2. "Why are the syntaxes different?" — `{{ .name | indent 4 }}` vs.
   `append context.imports name` for things that feel related.
3. Template errors surface as `template: capy:27: function "split"
   not defined` — pointing at the wrong file because Go's template
   engine is the source.

The deeper problem: the per-function rule "when source matches X,
contribute Y to the body AND maybe update state Z" is conceptually
one thing. Splitting it across two blocks is an implementation
artefact, not a design choice the author asked for.

## Proposal — function body IS Capy code; `write` is a statement

The earlier draft of this proposal kept template-internal control
flow (`${for x in xs}` … `${end}`) and a separate "statement
interpolation" form. That was a half-measure — a hangover from Go
`text/template`'s "the template is its own world" model.

The cleaner shape: a function declaration has a **header**
(`arg literal` / `arg capture` / `block_closer`) followed by a
**body** of plain Capy statements from the inner DSL. `write` is
just one of those statements; control flow uses Capy's existing
`for` and `if`. There is no template language — only string
literals with `${name}` interpolation.

```
function import
    arg literal "import"
    arg capture name ident
    append context.imports name
    # no `write` call → contributes nothing to body
end

function greet
    arg capture name any
    write `Hello, ${name}!
`
end

function loop
    arg literal "loop"
    arg capture v ident
    arg literal "in"
    arg capture i any
    block_closer end
    write `for ${v} in ${i}:
`
    write `    ${body}`
end

file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

### The whole spec in one paragraph

A function body is a sequence of inner-DSL statements. `write EXPR`
appends EXPR (coerced to string) to the function's body
contribution. Backtick strings span multiple lines and accept
`${EXPR}` interpolation, where EXPR is any inner-DSL value
expression (paths, function calls, literals). The function's body
contribution is rendered FIRST (so `body`, the inner block's
rendered text, is available in scope); the function body executes
top-down, mixing `write` calls with `set` / `append` / `for` / `if`
freely.

That's it. No `{{- -}}` whitespace dance, no `${for}` block-form
interpolation, no pipeline syntax, no `define` / `template`
machinery — control flow IS the host language.

### Why this is better than the half-measure

| Pain in v0 draft                          | Fix in this version                      |
|-------------------------------------------|------------------------------------------|
| `${for x in xs}` inside a template        | `for x in xs` outside it (plain Capy)    |
| Statement vs. value interpolation rule    | Gone. `${…}` is value-only               |
| `${- -}` whitespace control               | Gone. Author controls whitespace by where `write` is called |
| "Which is the template language?"         | There isn't one. There's just Capy + interpolated strings |
| Helpers as `${pascalCase name}`           | Same, but now they're regular inner-DSL functions called from anywhere |
| Two engines living side by side           | One: the inner-DSL evaluator with an `out` buffer attached |

### How common patterns look

**Conditional output**
```
function field
    arg literal "field"
    arg capture name ident
    arg capture optional bool
    if optional
        write `${name}?: any;
`
    end
    if not optional
        write `${name}: any;
`
    end
end
```

**Loop over a list**
```
file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

**Mixed state + output (e.g. emit a section header AND record it for a TOC)**
```
function section
    arg literal "section"
    arg capture title string
    block_closer end
    append context.toc title
    write `## ${title}

${body}
`
end
```

The decision "do I update state or emit output?" disappears. You
write what should happen, top-down.

### Interpolation rules

Inside a backtick string:
- `${EXPR}` evaluates EXPR (any inner-DSL value expression: path,
  literal, function call, parenthesised sub-expression) and splices
  the result.
- `\$` escapes a literal `$` (rare; only needed for shell scripts).
- Newlines, tabs, indentation are literal. No magic trimming. You
  put exactly the bytes you want.
- Backticks are escaped as `\\\`` inside the body, or use a
  triple-backtick form for strings that contain unescaped backticks
  (Markdown templates, fenced-code generators).

### Template helpers become inner-DSL functions

Current template helpers (`indent`, `pascalCase`, `snakeCase`,
`unquote`, `toQuoted`, `toJSON`, `join`, `split`, etc.) move into
the inner DSL as ordinary callable functions. The function library
is the same; only the call syntax changes:

| Today                                  | Tomorrow                       |
|----------------------------------------|--------------------------------|
| `{{ .name \| pascalCase }}`            | `${pascalCase name}`           |
| `{{ .name \| upper \| trimSuffix "X" }}` | `${trimSuffix "X" (upper name)}` |
| `{{ indent 4 .body }}`                 | `${indent 4 body}`             |
| `{{ join ", " .tags }}`                | `${join ", " context.tags}`    |

Pipeline syntax is lost (nested calls instead). That's a small
ergonomic regression and a big consistency win.

### `body` is just a value

In the original design `${body}` was a magic interpolation token.
In this design `body` is a regular value available in scope inside
a function with a `block_closer` (or `block_open`/`block_close`).
It's the rendered text of the inner block. You can `write body`,
`write indent 4 body`, pass it to a helper — anything you'd do
with any other value.

Outside a block-opening function, `body` is undefined and reading
it errors at load time. Inside `file_template`, `body` is the
concatenated top-level rendered output.

### Errors

Every error points at a `.capy` line:col, not at a synthetic Go
template position. `${foo bar}` where `foo` isn't a helper or a
reserved keyword produces:

```
script.capy:14:8: unknown identifier "foo"
hint: did you mean "for"? (statement) or one of: pascalCase,
indent, join, … (helper)
```

## What this replaces

| Today                              | Tomorrow                                          |
|------------------------------------|---------------------------------------------------|
| `template:` (Go text/template)     | `write \`...\`` calls in the function body        |
| `template_str "..."`               | `write "..."`                                     |
| `run:`                             | gone — its statements are now just function-body statements |
| `file_template:`                   | `file_template ... end` block; body is Capy code  |
| `{{ .name }}`                      | `${name}`                                         |
| `{{ .name \| pascalCase }}`        | `${pascalCase name}`                              |
| `{{- range .xs }}` … `{{ end }}`   | `for x in xs ... end` (around the `write` calls)  |
| `{{ if .cond }}` … `{{ end }}`     | `if cond ... end`                                 |
| `{{ template "x" . }}`             | `call x` (any library function is reusable as a sub-routine) |
| `{{ define "x" }}`                 | `function x ... end` at the library top level    |

## Anti-goals (we are NOT doing these)

- **No template language at all.** The single language is Capy's
  inner DSL plus interpolated string literals. There is no
  control-flow syntax that is "template-only" — every loop and
  conditional is regular Capy code that wraps `write` calls.
- **No `{{- -}}` whitespace control.** Author controls whitespace
  by where `write` is called and what bytes the backtick string
  contains. If you don't want a trailing newline, don't put one in.
- **No pipeline syntax** (`x | upper | trim`). Use nested calls
  (`${trim (upper x)}`) or bind to a local first
  (`set s (upper x); set s (trim s); write s`).
- **No `${stmt}` side-effecting interpolation.** Side effects are
  plain statements outside the backtick, not inside it. `${...}`
  is value-only.
- **No new control structures.** Just `for` / `if` (with optional
  `else` arm — TBD; today's inner DSL doesn't have one). No
  `while`, no `switch`, no `try`. Anything more is the host
  language's job.
- **No expression DSL beyond what the inner DSL already has.**
  `${a + b}` does NOT become valid; arithmetic stays inside helper
  calls (`${add a b}`).
- **No partial-template inheritance / overrides.** Reuse via plain
  function calls; no `extends` / `block` machinery.
- **No "raw" Go template escape hatch.** If a library needs
  something the unified engine can't express, that's a bug in the
  engine, not a feature flag.

## Open questions

1. **Backtick strings as a global Capy feature?** Today `.capy`
   uses double-quoted strings only. Backtick literals are needed
   for multi-line `write` arguments. Decide: backticks globally
   (so any string can be a backtick literal — useful in `context`
   defaults, `description "..."`, etc.) or only as a `write`
   argument? Lean: globally. Same string form everywhere is the
   consistency win.

2. **Indentation stripping inside backticks.** The current
   `template:` block strips the deepest common leading indent so
   authors can write naturally-indented templates. Backticks
   should do the same. Spec it explicitly: the lexer remembers
   the indent of the opening backtick's line, strips that much
   from every subsequent line.

3. **Closing-backtick escape.** If a `write` literal emits a real
   backtick (Markdown fence, shell command-substitution, etc.),
   you need an escape. Options: `` \` `` inside the body, or a
   triple-backtick variant for strings that contain unescaped
   backticks. Lean: support both. `` \` `` covers the common case;
   triple-backtick handles Markdown templates with fenced blocks.

4. **`else` arm.** Today's inner DSL `if` has no `else`. The
   workaround is two `if` blocks (one with `not`). Adding `else`
   is straightforward and badly missed; do it in the same swing.

5. **`set` followed by `write` for computed values.** Some
   templates need an intermediate computation, e.g.:
   ```
   set qualified (concat module "." name)
   write `${qualified}
`
   ```
   Local variables in a function body already work (the inner DSL
   has `loop x in y` exposing `x` and `set` writes to locals when
   the path root isn't `context`). Spec this explicitly so it's
   clear `write` can use freshly-computed locals.

6. **Sub-routine calls (`call f a b`).** Reusing one library
   function from inside another's body is the new `define` /
   `template` replacement. The inner DSL doesn't have it today.
   Add it: `call FNAME ARGS...` runs another function as if it
   matched source, appending its body output to the caller's.
   Captures map to args by position. (Open: how to pass the
   *block body* when calling a block-opener function? Probably
   `call FNAME ARGS... with body`.)

## Migration

Implementation phases:

1. **Build the new engine alongside the old.** A new `write:`
   block-shaped parser in `infra/`; a new template runtime in
   `orchestrator/features/`. Existing `template:` + `run:` keep
   working untouched. Engine selects per-function based on which
   block the library declared.

2. **Convert 3–5 reference samples.** `transpile-py`,
   `recipe-card`, `source-imports`, `host-capabilities`,
   `multi-target-ws-server`. Diff outputs; everything must be
   byte-identical to today.

3. **Convert remaining samples in waves.** ~60 libraries. Bulk
   conversion is mechanical for the simple ones; the gnarly
   `file_template:` blocks in
   `samples/cross-platform-installer/`, `samples/multi-language-demo/`,
   etc. need manual review.

4. **Deprecate `template:` + `run:`.** Two minor versions of
   warning; remove in v1.0.

5. **Update docs + showcase + playground bundles** at each
   conversion wave.

Estimated effort:
- Engine: 2–3 weeks
- Sample conversion: 1 week
- Docs + playground: 1 week
- Total: ~5 weeks at one-developer-full-time pace

## Decision check

Three things have to be true for this to be worth doing:

1. **The unified shape is genuinely clearer to newcomers.**
   Test: hand someone the `import` example in both forms; ask them
   to add a `from X import Y` variant. Time them.
2. **No regressions in expressive power.** Every existing library
   must port cleanly with byte-identical output.
3. **Errors get better, not just different.** New engine must
   surface `.capy` line:col on every failure path that currently
   says `template:`.

If any of these wobbles in prototyping, abandon.

## Not yet decided

- Should `write:` REPLACE `template:` (one name) or coexist
  briefly under both? Coexistence is gentler; one-name is cleaner.
  Default: coexist for one minor version, then `template:` becomes
  a deprecated alias.
- Should the inner-DSL's `set` keyword remain for context updates,
  or be replaced by `${context.x = value}` assignment syntax?
  Assignment-syntax is shorter; keyword-syntax is consistent with
  the existing inner DSL. Default: keep `set` for parity.