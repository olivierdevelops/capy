# Changelog

All notable changes to Capy are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) —
with the important caveat that **while pre-1.0, the library YAML schema
may break between minor versions**.

## [Unreleased]

## [0.23.0] — 2026-10-07

Ordered choice for grammars (`PROP-2026-0004`).

### Added

- **Ordered alternation in a capture type.** `arg capture v call | name | num`
  names several library functions; the matcher tries them left to right and the
  first match wins. Composes with repetition (`mut_param | plain_param* sep ","`)
  and with recursion, so a declared grammar nests to the engine's depth bound.
- `ArgInfo.alts` (public introspection) — alternatives 2…n; `type_` keeps meaning
  alternative 1. `capy docs` prints the whole choice in the Type column.
- Two samples: `samples/expression-grammar/` and `samples/mixed-parameters/`.

### Changed

- The left-recursion guard walks **every** alternative, so a cycle reachable only
  through a later alternative is refused at load.

### Documentation

- Parser-surface documentation and samples from `PROP-2026-0002` ship in this
  release: `docs/language-frontend.md`, `docs/tutorials/05-reading-diagnostics.md`,
  `samples/language-frontend/`, `samples/operator-precedence/`,
  `samples/parse-recovery/`, the shared `ast_text` renderer used by `capy ast`
  and the golden runner, and the `<base>.expected-ast.txt` golden kind.

## [0.20.0] — 2026-05-24

Dev-loop tools that complete the "libraries as CLIs" story:
watch, formatter, git-based install, and a single-binary
compiler so library authors can ship their tools to users who
don't have Capy installed.

### Added — CLI subcommands

- **`capy watch <library> [args…]`** — polling-based file
  watcher (250ms). Re-runs the command whenever any watched
  file changes. Watches every `.capy` in the library's directory
  plus any file-path args. Works for both new-shape (library
  command dispatch) and legacy (`capy run lib.capy script.capy`)
  invocations.

- **`capy fmt <files…>`** — conservative formatter for `.capy`
  files: strips trailing whitespace, converts leading tabs to
  4-space indent, collapses multi-blank-line runs, ensures one
  trailing newline. Does NOT touch the inside of backtick
  literals (whitespace is significant). Flags: `--check`,
  `--diff`, `--stdout`. The full canonical-form normaliser
  (declaration ordering, arg alignment) is deferred.

- **`capy lib add <source> [--as name]`** — install a library
  from a git URL or local path. Common shorthand:
  `github.com/X/Y` → `https://github.com/X/Y`. Library name is
  inferred from the trailing path segment (strips `-capy` /
  `_capy` suffixes); override with `--as`. Land in the first
  writable directory on `CAPY_LIBS`.

- **`capy lib remove <name>`** (alias `rm`) — delete an
  installed library.

- **`capy build <library> [-o output] [--keep-temp]`** —
  produce a standalone executable with the library baked in.
  Implementation: writes a tiny Go wrapper `main.go` that embeds
  the library source + dispatches via
  `orchestrator.RunCommand`, then shells out to `go build`.
  Compiled binary is self-contained — no Capy install required
  on the target host. Cross-compile by setting `GOOS` /
  `GOARCH` before the build.

### Verified end-to-end

- `fmt`: rewrites a deliberately messy library; backtick
  internals untouched.
- `lib add` (local-path form): installs a directory; the library
  resolves by name immediately afterwards.
- `lib remove`: cleans up the installed entry.
- `build`: produces a 5.4 MB binary that dispatches commands
  exactly like `capy <lib> <cmd>` does.
- `watch`: streams stdout on first run, detects a write, and
  re-runs cleanly.

### Engine notes

- `cmd_build.go` uses `findCapyModuleRoot` to locate a local
  capy checkout (via `go.mod`) so dev-mode `capy build` doesn't
  need the published module. Falls back to `go get latest` if
  not in a checkout.
- The wrapper's `main.go` template uses string concatenation
  (not Go raw strings) so library sources containing backticks
  serialize cleanly.

## [0.19.1] — 2026-05-24

Round-out of the library-commands feature: declarative
arg/flag parsing with auto-generated help, cross-command
composition, shebang-style scripts, and a basic trust warning.

### Added

- **Declarative `arg` / `flag` declarations** inside `command`
  blocks:
  ```
  command "build"
      arg "script" required "Path to the input script."
      arg "out" optional "Output path."
      flag "--minify" bool "Strip whitespace."
      flag "--prefix" "Optional prefix." default ""
      …
  end
  ```
  Declared positionals appear under their NAME in the context
  (`context.script`, `context.out`). Flags appear under
  `context.flags.NAME` (trimmed of leading dashes). Unknown
  flags + missing-required-arg errors out cleanly.
- **Auto-generated help** for every command and library:
  - `capy <lib> --help` lists declared commands.
  - `capy <lib> <cmd> --help` shows declared args, flags, and
    descriptions for a specific command.
- **Cross-command composition**: `call CMD_NAME args…` inside a
  command body runs another command of the same library. Use it
  as a statement (`call "build" context.script`) or as a value
  (`let x = (call "build" context.script)`).
- **Shebang-style scripts**: a `#!/usr/bin/env capy --lib X`
  line at the top of a script is stripped before lexing. With
  `chmod +x`, the file becomes a directly executable Capy script.
- **`capy --lib <name> <script>`** flag form so the shebang
  interpreter has somewhere to dispatch to.
- **Trust warning**: a one-line stderr warning before running a
  command from a library that's NOT under `CAPY_LIBS`. Suppress
  with `CAPY_TRUST=1`.
- **Better error** when `capy <lib> <cmd>` is used but the
  library doesn't resolve — previously fell through to
  `cmdRun`'s confusing "no such file" message; now surfaces the
  resolver's "library X not found on CAPY_LIBS" error.

### Engine surface

- New types `domain.CommandArg` + `domain.CommandFlag` declared
  on `CommandDef`.
- New helpers `orchestrator.ParseCommandArgs` +
  `orchestrator.PrintCommandHelp` for tools that want to
  introspect command definitions.
- `commandEnv.dispatch` learns the `call` primitive alongside
  `compile`.

## [0.19.0] — 2026-05-24

Libraries become CLIs. A library declares custom commands; the
CLI dispatches `capy <lib> <command>`. Library lookup by name via
a `CAPY_LIBS` search path. `capy new` scaffolds a project from a
library.

### Added — engine

- **Manifest fields in `.capy` libraries:** `name "X"` and
  `version "X"` at the top level.
- **`command "NAME" ... end` blocks** at the top level of a
  library. Body is the inner DSL extended with shell primitives.
- **`let NAME = EXPR`** inner-DSL statement for local bindings
  inside command bodies.
- **Shell-flavoured inner-DSL primitives** (statement form):
  `write_file`, `mkdir`, `exec`, `cd`, `print`.
- **Value-position primitives**: `mktemp ".ext"`, `mktemp_dir`,
  `exec_capture cmd args…`.
- **`compile script_path`** primitive (registered via the
  command-runner's hook) — renders the library on a script and
  returns the output as a string.
- **`OnUnknownCall` hook on `InnerEvaluator`** so embedders can
  inject extra primitives without growing the global set.
- **`domain.Host` gains** `WriteFile`, `Mkdir`, `MkTemp`,
  `MkTempDir`, `Exec`, `ExecCapture`. `NoOpHost` returns
  "filesystem / exec not available in this runtime"; `OSHost`
  implements all of them.
- **`domain.CommandDef`** + `domain.Library.Commands` map.
- **`orchestrator.RunCommand(libPath, cmdName, args)`** — the
  entry point the CLI uses to run a library command.

### Added — CLI

- **`capy lib list`** — print every library on `CAPY_LIBS`.
- **`capy lib which <name>`** — show resolved path.
- **`capy lib new <name>`** — scaffold a starter library with
  manifest + a tiny `greet` function + `run`/`compile` commands +
  example script.
- **`capy lib path`** — print the search path.
- **`capy new <dir> --using <library>`** — scaffold a new project
  from a library. If the library declares a `new` command, it
  gets dispatched with the project directory; otherwise a minimal
  default scaffold lands.
- **`capy <library> <command> [args]`** — library-name short
  form. Only fires if the first arg looks like a library name AND
  resolves on `CAPY_LIBS`.
- **`capy <script>.<libname>`** — auto-detect the library from
  the script's extension (e.g. `capy cake.recipe` resolves the
  `recipe` library and runs its `run` command).

### Added — docs

- New page `docs/library-commands.md` covering the search path,
  manifest fields, command body DSL, `capy lib` / `capy new`
  subcommands, and a Python-library walkthrough.
- Linked in mkdocs nav under Reference.

## [0.18.0] — 2026-05-24

Unified `write:` block. The split between `template:` (Go text/template
output) and `run:` (Capy inner DSL state mutation) collapses into a
single function body: a sequence of inner-DSL statements that may
include `write \`...\`` calls. Backtick string literals support
multi-line content and `${EXPR}` interpolation.

### Added

- **Backtick string literals** in `.capy` library files. Multi-line
  bodies stay on the same logical line via an in-parser merger that
  is scoped to function bodies (so `template:` blocks containing
  Markdown code fences are unaffected).
- **`write EXPR`** inner-DSL statement. Translated at library-load
  time into Go-template syntax appended to the function's
  `Template`. Most commonly used with a backtick string literal:
  `write \`Hello, ${name}!\n\``.
- **`${EXPR}`** value interpolation inside backtick strings. Supports
  paths (`${name}`, `${context.foo}`, `${body}`) and inline helper
  calls (`${indent 4 body}`, `${pascalCase name}`).
- **`else` / `else if`** arms for the inner-DSL `if`.
- **`for`** as a synonym for `loop`. Reads more naturally inside
  `write`-heavy function bodies.
- **New-shape function body.** A function declaration may have
  inner-DSL statements directly in its body (after the `arg` /
  `block_closer` / `description` / `priority` header lines), with
  no `template:` or `run:` block required.
- **`infra.RawFunction.Body`** (`body:` in YAML) — the new-shape
  body string. Mutually exclusive with `Template` + `Run`.
- **`domain.WriteStmt`** — new inner-DSL AST node.
- **`orchestrator/features/translate_new_shape.go`** — the
  translator that splits a unified body into Template + Run before
  the engine sees it. Control-flow blocks (for / if) that contain
  a mix of writes and state mutations are duplicated: one copy
  goes into the template (writes only, with state stripped), one
  goes into the run block (mutations only). Both phases see the
  same iteration / branching shape.

### Migrated

- **39 of ~51 library files** in `samples/` converted to the
  unified shape. Output byte-identical to v0.17 in every case;
  goldens unchanged. The remaining ~12 keep the legacy shape (they
  use template / range / if patterns the bulk migrator chose not
  to auto-convert) — both shapes work, mix freely.
- Key docs (`library-authoring.md`, `getting-started.md`,
  `CAPY_FOR_LLMS.md`) lead with the unified shape; legacy form
  documented as the backwards-compat path.

### Backwards compatibility

- The legacy `template_str` / `template:` / `run:` blocks continue
  to work. The engine accepts either shape per function; libraries
  can mix.
- Existing libraries don't need any change to keep running.

## [0.17.0] — 2026-05-24

Honour "zero predefined grammar" all the way down: source-level
inclusion (`@import` / `@include`) was the last engine-level
preprocessor surface left over. It now requires explicit library
opt-in via a top-level `preprocess` block.

### Changed (BREAKING for scripts using `@import` without library
opt-in)

- `infra.Preprocess(source, dir, directives)` now takes an opt-in
  list of directive names. With an empty list it's a no-op.
- `orchestrator.RunMulti` and `capy.Library.RunMulti` now load the
  library FIRST, then run the preprocessor with the library's
  declared directives.
- Libraries that want `@import`/`@include` (or any other surface
  name) must declare them:

      preprocess
          include "@import"
          include "@include"
      end

  Library imports through the loader (`mergeRaw`) union the
  `preprocess` lists so a child library can inherit directives.
- Domain: `Library.Preprocess []string`. Raw DTO: `preprocess:` in
  YAML / `preprocess … end` block in `.capy`.

### Updated

- `samples/source-imports/lib.capy` — declares both directives.
- README + multi-file-and-imports docs updated to spell out the
  opt-in and to call out that `file "..."` paths are themselves Go
  templates (so filenames can be dynamic, e.g.
  `file "{{ .context.name | pascalCase }}.tsx":`).

### Added — tests

- `infra/preprocessor_test.go`:
  - `TestPreprocess_NoDirectivesIsNoOp` — empty opt-in list leaves
    `@import` lines untouched even when the file exists.
  - `TestPreprocess_UnknownDirectiveIgnored` — a library that opts
    into `@use` only doesn't see `@import` expanded.

## [0.16.0] — 2026-05-24

Custom assembly DSL targeting multiple ISAs. A 5-op
architecture-neutral assembly DSL produces runnable hello-world on
x86_64, AArch64, AND RISC-V64 from one source — swap the library to
retarget. New ISA = new library; the source doesn't change.

### Added

- **Sample: `samples/custom-asm/`** — five ops (`data`, `func`,
  `write`, `exit`, `end`) + three libraries:
  - `lib-x86_64-linux.capy` — System V AMD64, `syscall` trap
  - `lib-arm64-linux.capy` — AArch64, `svc #0` trap
  - `lib-riscv64-linux.capy` — RV64I, `ecall` trap

  Each library encodes its ISA's syscall ABI: argument registers,
  syscall numbers, trap instruction. All three emit assembly that
  builds with the native GNU toolchain and prints "Hello, world".
- **`unescape` template helper** — reverses Go's string-quoting so
  templates can produce literal escape sequences (`\n`, `\t`) when
  the target language wants them as escapes rather than raw bytes.
- All three assembly variants surface as separate dropdown entries
  in the playground (`⚙️ Assembly · x86_64 / arm64 / riscv64`).
- Showcase tab with side-by-side x86_64 / AArch64 / RV64 outputs.

### Fixed

- `interpolateGeneric` documentation clarified: `\X` consumes one
  backslash, so library authors who want a target-language escape
  sequence to survive into the output must double-escape in source
  (e.g. `"hello\\n"` to emit `\n` for an assembler `.ascii` line).

## [0.15.0] — 2026-05-24

OS/arch host introspection and three network/cross-platform samples.
Libraries can now branch on `(os)` / `(arch)`, and samples show the
spec-as-source pattern applied to non-trivial network code.

### Added

- **Inner-DSL primitives**: `(os)`, `(arch)`, `(cwd)`, `(home_dir)`.
  Backed by `runtime.GOOS` / `runtime.GOARCH` / `os.Getwd` /
  `os.UserHomeDir` via `infra.OSHost`. `NoOpHost` returns "".
- **Sample: `samples/cross-platform-installer/`** — one 9-line source
  emits `install.sh` (POSIX), `install.ps1` (PowerShell), AND
  `install.bat` (cmd.exe) via multi-file output. Cross-platform
  installers without bash-in-PowerShell horror.
- **Sample: `samples/transpile-websocket-server/`** — 9-line ws DSL →
  ~80-line Go WebSocket server with typed JSON envelope, dispatch,
  hub, and broadcast helper.
- **Sample: `samples/multi-target-ws-server/`** — one source, three
  libraries: `lib-go.capy` / `lib-node.capy` / `lib-python.capy`.
  Same 7-line source → runnable echo server in Go (gorilla),
  Node (`ws`), or Python (asyncio + `websockets`). Library choice
  picks the runtime; source survives.
- **`LibraryFile` field on playground-bundle's curated entries** so
  multi-library samples (like multi-target-ws-server) can expose
  each variant as its own dropdown entry.
- All four new samples linked in showcase with tabbed source/output
  views.

## [0.14.0] — 2026-05-24

Host capabilities. Libraries can pull values from outside the source at
transpile time — environment variables, positional CLI args, and sibling
files — via four inner-DSL primitives. Sandboxed by default; the CLI
opts in to real OS access automatically.

### Added

- **Inner-DSL primitives** (use inside `run:` blocks):
  - `(env "NAME")` → OS env var (string, `""` if unset)
  - `(arg N)` → Nth positional CLI arg (string)
  - `(arg_count)` → number of positional args
  - `(args)` → full args list
  - `(read_file "PATH")` → file contents; relative paths resolved
    against the script's directory; errors abort the transpilation
- **`domain.Host` interface** + two implementations:
  - `infra.OSHost` — real `os.Getenv` / `os.Args` / `os.ReadFile`,
    used by the CLI
  - `domain.NoOpHost` — sandboxed default for embedded callers and
    the wasm playground; every primitive returns the empty zero value
    and `read_file` errors with a clear message
- **`capy.Library.SetHost(h)`** — opt-in API for embedded Go programs
  that want libraries to see their env/files
- **CLI positional args** — `capy run lib.capy script.capy a b c` now
  makes `"a"`, `"b"`, `"c"` visible to `(arg 0)`, `(arg 1)`, `(arg 2)`
- **`samples/host-capabilities/`** — generates a Kubernetes Deployment
  from env vars, CLI args, and a sibling `api-keys.txt`
- **Template helpers** `split` and `nonEmpty` for iterating over
  `read_file` output
- **`docs/host-capabilities.md`** — full pattern doc with the Host
  abstraction, CLI/embedded/playground semantics, and when NOT to
  reach for these primitives

### Changed

- `orchestrator.RunMulti` gains a sibling `RunMultiWithArgs` that
  threads positional CLI args through to the inner host

## [0.13.0] — 2026-05-24

Auto-generated library reference documentation. Library authors can
annotate functions, args, types, and the library itself with
`description "..."` directives; `capy docs <library>` produces
Markdown reference docs ready to commit alongside the library.

### Added

- **`description` directives** at four scopes:
  - Library top-level: `description "..."`
  - `type NAME ... end` body
  - `function NAME ... end` body
  - `arg literal "TEXT" "DESC"` and `arg capture NAME TYPE "DESC"`
- **`capy docs <library> [--out path]`** CLI subcommand that renders
  Markdown reference documentation (title, library description,
  metadata strip, types, functions with signatures + arg tables).
- **`capy.RenderLibraryDocs(lib)`** top-level Go API for programs
  that ship docs alongside generated code.
- **`window.capyDocs(libSrc, format)`** wasm binding so the
  browser playground can render docs for any loaded library.
- **Playground DOCS tab** — third tab in the left pane renders
  Markdown via marked.js into a sandboxed iframe, updates as you
  edit the library.
- **`samples/recipe-card/`** annotated with descriptions on every
  function and arg, plus committed `LIB_REFERENCE.md` as a
  side-by-side example.
- **`docs/library-documentation.md`** — full pattern doc covering
  the annotation surface, three ways to surface the docs (CLI,
  embedded Go, playground), tips, and a suggested CI workflow.

### Changed

- DTO / domain / loader threaded `Description` through `Library`,
  `FuncDef`, `ArgEntry`, `TypeDef`. YAML libraries gain matching
  `description:` keys.

## [0.12.0] — 2026-05-24

Progressive abstraction. One Capy library can expose primitives at
multiple granularities — start with a one-liner, peel back layers as
you need more control, drop to raw HTML/CSS via escape hatches when
the abstraction isn't enough.

### Added — sample

- **`samples/progressive-abstraction/`** — one landing-page library
  (`lib.capy`) plus THREE sources at three abstraction levels:
  - `script_minimal.capy` (~4 lines) — one-shot `landing` declaration
  - `script_medium.capy` (~12 lines) — block style with explicit
    `hero` / `feature` / `cta` blocks
  - `script_full.capy` (~30 lines) — block style PLUS escape hatches
    (`raw_head`, `style_override`, `raw_section`, `raw_footer`)

  All three produce HTML landing pages. The library never traps you —
  drop a level when the abstraction isn't enough. Tests
  (`cmd/capy/progressive_abstraction_test.go`) diff all three against
  committed goldens on every commit.

### Added — docs + playground

- **`docs/progressive-abstraction.md`** — pattern docs with side-by-
  side browser-frame previews of all three levels and a "when to
  design libraries this way" decision matrix.
- **Playground** gains a new **Patterns** category in the sample
  dropdown with three entries (`🎚️ Abstraction · Level 1/2/3`)
  sharing the same library. 59 total samples across 8 categories.
- **Playground bundler** (`cmd/playground-bundle/main.go`) now
  supports an optional `ScriptFile` field so multiple curated
  entries can share one sample directory with different script
  files — needed by progressive-abstraction.

### Changed

- Homepage "What enterprises ask for" table adds a row mapping
  "High-level tools trap you when you need more control" to
  progressive abstraction.
- Showcase opens with a new "🎚️ Progressive abstraction" section
  (5 tabs: Level 1 / 2 / 3 / Why-it-matters / docs link) above the
  metaprogramming section.
- MkDocs nav: new "Progressive abstraction (pick your level)"
  entry under Patterns.

## [0.11.0] — 2026-05-24

Source-level metaprogramming. A Capy source file can now declare
new functions inline via `define NAME ... end` blocks — the rest
of the source (and any `@import`-ed files) can call them
immediately. The library doesn't need to know about them.

### Added — engine

- **`infra.ExtractDefines(source) (cleaned, libSrc, err)`** — a
  pre-pass that scans for top-level `define NAME ... end` blocks
  with the same body shape as a library `function` declaration.
  Rewrites them as a synthetic `.capy` library and returns it
  alongside the source with the defines stripped.
- **`orchestrator.RunMulti`** wires it in between Preprocess and
  Lex. Compiled defines are merged into the working library with
  source-wins-on-conflict semantics (i.e. `define foo` in the
  source overrides `function foo` in the library — specialization
  without forking).

### Added — sample + tests + docs

- **`samples/metaprogramming/`** — a deliberately minimal library
  (one `print` function) plus a source that defines `heading`,
  `quote`, and `todo` patterns inline and uses them to render a
  Markdown document. Committed golden + golden test.
- **`infra/define_extractor_test.go`** — 7 tests covering basic
  usage, multiple blocks, unclosed `end`, malformed bodies, bad
  identifiers, no-op (no defines), and the indented-define-
  ignoring rule.
- **`docs/metaprogramming.md`** — pattern docs with a worked
  example, the full `define` body reference, when-to-use-it
  decision table, conflict-resolution rules, and implementation
  notes.

### Changed

- MkDocs nav: new "Metaprogramming (source-defined functions)"
  entry under Patterns.
- Showcase: new "🧬 Metaprogramming — source extends its own
  grammar" tabbed section above the Errors section.
- Homepage enterprise-concerns table: added a row mapping "power
  users want to extend the DSL without forking" to metaprogramming.
- Playground curated samples: added the metaprogramming sample (now
  56 total across 7 categories).

## [0.10.0] — 2026-05-24

Three new framings get first-class samples and documentation:
design systems across React + Vue + Svelte, backend code with
auto-wired tests, and the umbrella idea — Capy as an idea language
where libraries are implementers.

### Added — engine

- **`file "..."` paths are now Go templates.** Authors can name
  outputs dynamically:
  `file "{{ .context.page_title | pascalCase | unquote }}Page.tsx":`
  Each path is rendered against the same context+body as the file's
  template body.

### Added — samples

- **`samples/design-system-components/`** — one 8-line component
  composition compiles to React TSX, Vue 3 SFC, AND Svelte. The
  three libraries share the same Tailwind variant + size tables, so
  visual semantics are identical across frameworks. Each component
  file is named via `pascalCase(page_title)` so adding a new page
  declaration auto-names the output.

- **`samples/backend-with-tests/`** — every `handler` declaration
  produces a Go stub AND a matching smoke test. The team's
  directory layout (`internal/handlers/`), the "every handler has
  a test" rule, the stub-returns-501 contract, and a
  route-catalog README are all encoded in the library. Generated
  Go compiles; `go test ./...` against the output **passes**.

### Added — docs

- **`docs/design-systems.md`** — pattern doc for "house style as
  library." Walks through the React/Vue/Svelte demo and explains
  what the library is enforcing (variant tables, layout
  primitives, sizing scales).

- **`docs/backend-codegen.md`** — pattern doc for "conventions as
  library." Walks through the handler+test demo, explains the
  contract between code-gen and the developer (stub returns 501;
  test asserts 501; implement → test fails → replace with real
  assertions).

- **`docs/idea-language.md`** — the most ambitious framing: Capy
  as a language for describing ideas, libraries as implementers.
  Lays out the "rewrite libraries, not ideas" thesis with
  references to multi-language-demo, ios-app/android-app, and the
  design-system samples as concrete evidence.

### Changed

- **Homepage**: three new feature cards (design systems / backend
  codegen / idea language) above the project-scaffolding card.
- **Showcase**: three new tabbed sections opening with the
  design-system → React/Vue/Svelte demo, the backend handler+test
  demo, and the idea-language framing. Existing project-scaffolding
  section follows.
- **MkDocs nav**: exposes all three new pattern docs.

### Tests

- `cmd/capy/multitarget_test.go` adds the design-system sample
  (per-framework diff) and the backend-with-tests sample to the
  existing multi-target diff harness.
- The `expected/` tree for `samples/backend-with-tests/` gets a
  `go.mod` so it doesn't accidentally become part of the main
  module's package graph during `go test ./...`.

## [0.9.0] — 2026-05-24

The Capy compiler now runs in the browser via WebAssembly. The
documentation site ships a playground at `/playground/` where
visitors can edit a curated DSL, preview the output, and download
the generated content — no install required.

### Added — WebAssembly compiler

- **`cmd/capy-wasm/main.go`** — the entry point compiled with
  `GOOS=js GOARCH=wasm`. Exposes a single global function
  `capyRun(libSrc, format, scriptSrc)` that returns
  `{ok, output, files, extension}` on success or
  `{ok:false, error, hint, line, col, pretty}` on failure.
- **`capy.Library.RunMulti(scriptSrc)`** — sibling of `Run` that
  also returns the multi-file map. Used by the wasm entry to
  expose multi-file libraries to the browser.

### Added — browser playground

- **`docs/assets/playground/index.html`** — self-contained playground
  UI with a sample picker (6 curated DSLs), live script editor,
  collapsible library editor for tinkering, preview area (renders
  HTML output in a sandboxed iframe), and a Download button that
  produces either a single file or a zip archive (via JSZip CDN).
- **`docs/playground.md`** — MkDocs wrapper page embedding the
  playground via iframe; exposed in the top nav as "Playground".
- **`docs/index.md`** — Homepage CTA reorder: "Open the
  playground" is now the primary button. New feature card
  highlights browser execution.

### Added — build infrastructure

- **`cmd/playground-bundle/main.go`** — bakes six curated samples
  (recipe / event-invite / weekly-meal-plan / reading-log /
  interactive-breakout / interactive-snake) into a single JSON
  file the playground fetches on load.
- **`scripts/build-playground.sh`** — one command that builds
  capy.wasm + copies wasm_exec.js + generates samples.json. Run
  locally before `python3 -m http.server -d docs/assets/playground`.
- **`.github/workflows/docs.yml`** now runs the playground build
  step before MkDocs, so every deploy ships fresh wasm + samples.
- **`.gitignore`** — the three build artifacts
  (capy.wasm, wasm_exec.js, samples.json) are gitignored and
  rebuilt from source on every CI deploy.

### Browser limitations

- `@import "..."` source-file directives don't work in the
  browser (no filesystem). The samples baked into the picker are
  single-file scripts; multi-file libraries still produce the
  full file map but can't load external files via @import.

## [0.8.0] — 2026-05-24

Multi-file generation matures: a `--zip` flag bundles output to a
single archive, three template helpers convert human strings to
language identifiers, and four samples demonstrate the full range —
web app, Android, iOS, libtorch C++ ML trainer.

### Added — CLI / engine

- **`capy run --zip ARCHIVE.zip`** — bundles every declared
  `file:` block into a single zip archive (alternative to
  `--out-dir`). Internal paths use forward slashes for portability.
- **Template helpers** for converting human strings to identifiers:
  - `pascalCase` — `"Habit Tracker"` → `HabitTracker`
  - `camelCase` — `"Habit Tracker"` → `habitTracker`
  - `snakeCase` — `"Habit Tracker"` → `habit_tracker`
  Each strips surrounding quotes so they compose with `string`-typed
  captures.

### Added — samples

- **`samples/webapp-trio/`** — 12-line DSL → `index.html` +
  `app.js` + `styles.css` (habit tracker with localStorage and
  streak counting).
- **`samples/android-app/`** — 15-line declaration → 7 files
  across `app/src/main/{AndroidManifest.xml, java, res/...}` +
  gradle config. Drop into Android Studio.
- **`samples/ios-app/`** — same source shape as android-app →
  SwiftUI App + RootView + Screens + Info.plist + Package.swift.
- **`samples/libtorch-train/`** — 17-line neural-net architecture
  → `model.h` (register_module + forward) + `main.cpp` (training
  loop) + CMakeLists.txt + run.sh. Builds against libtorch with
  CUDA when available.

Each sample's committed `expected/` tree is diffed by CI.

### Added — docs

- **`docs/one-source-many-files.md`** — consistent story behind
  every multi-file sample.
- Homepage card rewritten to lead with project scaffolding.
- Showcase opens with a "Generate a whole project" section.

### Tests

- **`cmd/capy/multitarget_test.go`** diffs every multi-target
  sample against its committed `expected/` tree, plus a
  TestZipOutput that confirms the archive matches the in-memory
  file map.

## [0.7.0] — 2026-05-24

Two complementary features: **actionable errors** with did-you-mean
hints throughout, and **source-file `@import`** for splicing
authored content across files.

### Added — actionable errors

- `domain.CapyError` gains `Hint string` and `File string` fields.
  `FormatWithSource` renders both alongside the caret-pointed
  source line.
- `domain.SuggestClosest(target, candidates, maxDist)` — Levenshtein
  lookup powering "did you mean X?" hints throughout the engine.
- **Upgraded error sites**:
  - **Parser** (no library function matches): suggests the closest
    function name, or lists what's available if no close match.
  - **Type validation** (pattern mismatch): hint shows the offending
    regex so authors can see what's wrong without opening the lib.
  - **Type validation** (options mismatch): hint lists every valid
    option, plus did-you-mean when close.
  - **Library loader** (unknown type): suggests closest type from
    built-ins + library-declared types.
- Error format changed to `file:line:col: message` (was `line N, col
  M: ...`). Editor-clickable. The richer `FormatWithSource` output
  with caret + hint is unchanged.

### Added — source-file `@import`

- `infra.Preprocess(source, dir)` is a line-level preprocessor that
  expands `@import "path"` / `@include "path"` directives at the
  start of a line into the contents of the referenced file.
- **Indentation auto-tracks**: a `    @import "x"` line inlines the
  imported content with each non-blank line prefixed by the same 4
  spaces. Imports nest naturally in surrounding blocks.
- Path resolution is relative to the file containing the directive.
- Cycles detected by absolute path.
- `@import` and `@include` are synonyms.
- `orchestrator.RunMulti` runs the preprocessor over the script
  before the lexer, so this works for every entry point (CLI,
  embedding API, MCP server).

### Added — sample + docs

- **`samples/source-imports/`** — a menu DSL that `@import`s
  shared drinks and desserts sections from `shared/`.
- **`docs/errors-and-debugging.md`** — full reference for the error
  format with worked examples.
- **`docs/multi-file-and-imports.md`** — extended with a
  source-imports section and a comparison table.
- **`infra/preprocessor_test.go`** — 7 tests covering basic import,
  nested imports, indentation preservation, cycle detection, missing
  file, `@include` synonym, malformed directives.
- **`domain/errors_test.go`** — tests for `SuggestClosest`,
  `FormatWithSource` rendering, and the new error format.

### Changed

- Homepage gains an "Errors that tell you how to fix them" card.
- Live showcase opens with two new tabbed sections: error walkthrough
  and source-imports menu sample.
- MkDocs nav exposes `errors-and-debugging.md`.

## [0.6.0] — 2026-05-24

Two structural features: libraries can now declare **multiple
output files** (with subdirectories) in one run, and one library
can **import** others to compose shared types and syntax helpers.

### Added — multi-file output

- **`file "path":` blocks** at the top level of a library. Multiple
  blocks may appear; each declares one output file. Paths may
  contain `/` for subdirectories — the engine creates them as
  needed.

- **`capy run --out-dir DIR`** flag. When the library declares any
  `file:` blocks, --out-dir is required; every block is rendered
  against the same final context+body and written under the dir.

- **Public Go API**: `orchestrator.RunMulti(lib, script)` returns
  both the single-output string and the map of `file:`-block
  outputs. `(*capy.Library).Run` continues to return just the
  string for callers that don't need multi-file.

- **`samples/multi-file-project/`** — 9-line route declaration
  becomes a 6-file FastAPI project tree (README, pyproject.toml,
  .gitignore, src/main.py, src/handlers.py, tests/test_smoke.py).
  Committed under `expected/` as the golden tree; CI test diffs
  every file on every commit.

### Added — library imports

- **`import "path"`** directive at the top of a library. Path is
  relative to the importing file. Imported types, functions,
  context entries, and `file:` blocks are merged in BEFORE the
  importer's declarations. The importer wins on conflict.

- Cycle detection: imports are tracked by absolute path; an
  import cycle is a clean error.

- Format mixing: a `.capy` library can import a `.yaml` library
  and vice versa (the loader sniffs each by file extension).

- **`samples/lib-composition/`** — main `lib.capy` imports
  `common/types.capy` (Email/URL/Semver/Slug) and
  `common/syntax.capy` (tag/note/meta). The merged library
  exposes 6 functions and 4 types.

### Added — docs

- **`docs/multi-file-and-imports.md`** — full reference for both
  features with worked examples and the conflict-resolution rules.
- Homepage gains a "Multi-file projects + library imports" card.
- Live showcase opens with two new tabbed sections:
  "Multi-file projects" and "Library composition."

### Changed

- `domain.Library` gains `Files map[string]string`.
- `infra.RawLibrary` (YAML + Capy parsers) gain `Files` and
  `Imports` fields.
- `features.Evaluator` gains `RunMulti` returning both the single
  output and the per-file map.
- `usecases.RunResult` and `cli.RunOutcome` gain a `Files` field.
- `orchestrator.Run` is preserved for backwards compatibility; new
  callers should prefer `orchestrator.RunMulti`.

## [0.5.0] — 2026-05-24

Repositioning release. Capy is reframed as a tool **anyone** can
use, not just programmers. The homepage opens with an animated
typewriter demo cycling through everyday DSLs (recipe / invite /
meal plan / reading log).

### Added

- **Animated hero** at the top of the homepage
  (`docs/assets/hero/hero.html`). Self-contained HTML/CSS/JS that
  types out source DSL with syntax highlighting, then reveals the
  generated polished output in a side-by-side iframe. Cycles
  through four demos; click dots to jump.

- **Four non-programmer samples**, each a complete library +
  source + golden + README:
  - `samples/recipe-card/` — recipes for home cooks. Six DSL
    keywords (`recipe`, `serves`, `time`, `ingredient`, `step`,
    `tip`). Output: polished HTML recipe card.
  - `samples/event-invite/` — party invitations. Pastel HTML invite
    with RSVP, notes, and "please bring" lines.
  - `samples/weekly-meal-plan/` — weekly dinner planner.
    Green-and-white HTML grid with day-by-day meals + notes.
  - `samples/reading-log/` — children's reading tracker. Bright
    orange certificate with progress bar (pages-read / yearly goal)
    and star ratings (rendered via new `stars` template helper).

- **`docs/for-everyone.md`** — non-programmer guide. Explains the
  vocabulary model with the recipe sample as the worked example,
  walks the three ways to get a library (use a sample, ask an AI,
  write your own), and gives a 5-minute setup.

- **Template helpers** (`infra/template_engine.go`):
  `add` / `sub` / `mul` for numeric math, `percent` for progress-bar
  calculations clamped to 0-100, and `stars` for rendering 1-5
  ratings as ★★★★☆.

### Changed

- Homepage `docs/index.md` opens with the animated hero and a
  reframed lead ("Write something simple. Get something polished.")
  before the feature grid. Adds a "For everyone" button to the
  hero CTAs.

- Live showcase opens with a new "Capy for everyday things — no
  coding needed" section with playable iframes for all four new
  samples.

- MkDocs nav exposes `for-everyone.md` directly under Home.

## [0.4.0] — 2026-05-24

Two new patterns get first-class documentation and worked samples,
plus a new template helper they both rely on.

### Added

- **`samples/contract-first-api/`** — REST API DSL where the
  grammar IS the contract. One source (`script.capy`) feeds three
  libraries (OpenAPI YAML / TypeScript client / Markdown docs);
  each has a committed golden snapshot. CI test
  (`cmd/capy/contract_first_test.go`) re-runs all three on every
  commit so library drift fails fast. Adding a 4th target is a
  ~30-line `.capy` file.

- **`samples/supercharge-markdown/`** — Capy as a Markdown
  preprocessor. 26-line DSL with `post` / `tag` / `para` / `h2` /
  `bullet` / `callout` / `card` / `code` becomes 48 lines of real
  Markdown with YAML frontmatter, blockquote callouts, and inline
  HTML metric cards. Drop into Hugo / Jekyll / MkDocs / Astro
  unchanged.

- **`samples/supercharge-sql/`** — Capy as a Postgres DDL
  preprocessor. Macros (`pk`, `fk`, `timestamps`, `soft_delete`,
  `index`) expand into idiomatic SQL. The database doesn't know
  Capy ran.

- **`docs/grammar-as-contract.md`** — the workflow doc:
  user describes intent → agent drafts library → grammar acts as
  contract → consumers build against it before targets land → CI
  goldens guard against drift. Includes the "build before tested"
  argument.

- **`docs/extending-existing-syntax.md`** — the supercharge-an-
  existing-format playbook with the canonical recipe and a table
  of host formats this pattern fits (SQL, Markdown, HTML,
  Dockerfile, GitHub Actions, Terraform, Kubernetes, OpenAPI,
  Mermaid, …).

- **Template helpers**: new `trimSuffix` and `trimPrefix` in the
  template engine. Useful for generators that emit trailing
  commas (`{{ .body | trimSuffix ",\n" }}`) — needed by
  `supercharge-sql` and similar patterns.

### Changed

- Homepage adds two cards: "Grammar as contract" and "Supercharge
  an existing syntax."
- Live showcase opens with two new tabbed sections demonstrating
  both patterns end-to-end.
- MkDocs nav exposes both new top-level pages.

## [0.3.1] — 2026-05-24

Reframing release. Capy is now consistently presented as a library
written in **Capy's own `.capy` syntax**, with YAML as one supported
alternative format rather than the canonical form. Plus one missing
parser feature needed to make the docs accurate.

### Added

- **`context ... end` blocks in `.capy` libraries.** Same DTO as the
  YAML `context:` section — declares initial context fields with
  defaults (`[]`, `{}`, scalars). Brings `.capy` to feature parity
  with YAML for the four most common library sections (`functions`,
  `types`, `context`, `file_template`).

### Changed

- README, `docs/index.md`, `docs/getting-started.md`,
  `docs/library-authoring.md`, `docs/CAPY_FOR_LLMS.md`, and both
  `skills/*/SKILL.md` files reframed: Capy is a transpiler engine
  driven by a `.capy` library; YAML is supported for the same
  library when downstream tooling needs it.
- README 30-second teaser rewritten in `.capy` syntax (verified to
  run end-to-end).
- Homepage feature cards no longer say "define in YAML" — they say
  "define in a `.capy` library (or YAML)".
- CAPY_FOR_LLMS now shows the schema in `.capy` form first, with
  YAML alongside.

### Why

For users new to Capy, leading with YAML was misleading: it
implied Capy *is* a YAML configuration framework, when it's actually
a transpiler engine that happens to accept libraries in two
equivalent textual formats. The `.capy` form is more idiomatic and
shares lexical rules with the source files the library will parse —
so authors only need one mental model.

## [0.3.0] — 2026-05-24

AI-integration release. Capy now ships an MCP server, a dedicated
Claude Code skill, and a cookbook of integration patterns.

### Added

- **`capy-mcp`** — a Model Context Protocol server (`cmd/capy-mcp`)
  exposing three tools over stdio JSON-RPC 2.0:
  - `capy_check` — validate a library; list its functions/types.
  - `capy_run` — transpile a script through an inline library string.
  - `capy_run_file` — same, with paths to existing files on disk.

  Format (`yaml` / `capy` / `auto`) is sniffed from the first
  non-comment line. Shipped as a separate binary in every release
  archive (`capy-mcp`, alongside `capy`). Works with Claude Desktop,
  Claude Code, Cursor, Zed, and any MCP-aware client.

- **`skills/capy-mcp/SKILL.md`** — Claude Code skill describing
  *when* to reach for Capy via MCP and how to operate the three
  tools. Pairs with the pre-existing `skills/capy-author/SKILL.md`.

- **`docs/mcp.md`** — full MCP setup guide with config snippets for
  Claude Desktop, Claude Code, Cursor, and direct JSON-RPC.

- **`docs/cookbook-ai.md`** — ten copy-pasteable recipes covering
  drop-in MCP install, sandboxed agent loops, token compression
  math, AI-builds-library-human-uses-it, typed safe surfaces,
  one-DSL-many-targets, embedded Go agents, skill+MCP wiring,
  self-correcting agents, and prompt-side guidance.

### Changed

- `.goreleaser.yaml` builds and ships the `capy-mcp` binary
  alongside `capy` in every release archive across all five
  platforms (linux/darwin/windows × amd64/arm64).
- Homepage feature grid: new "MCP server + Claude Code skill" card.
- MkDocs nav: "For AI agents" group expanded with MCP setup and the
  integration cookbook.

## [0.2.0] — 2026-05-24

A substantial feature release. Two new entry points (Capy-native
library files and embedding-as-Go-library), seven new showcase
samples including playable HTML5 games, and several engine fixes.

### Added

- **`.capy` library format**: libraries can now be written in Capy's
  own native syntax in addition to YAML. Loader dispatches on file
  extension; same DTO, same engine, byte-identical output. `.capy`
  libraries support `extension`, `output_file`, `function`, `type`,
  and `file_template` top-level blocks. See [`docs/capy-libraries.md`](docs/capy-libraries.md).

- **Embedding API** — top-level `github.com/olivierdevelops/capy` Go
  package lets programs define DSLs inline and transpile in-memory
  with no separate binary. Public API:

  ```go
  capy.NewLibrary(src) / NewLibraryYAML(src) / NewLibraryFromFile(path)
  (*Library).Run(scriptSrc) (string, error)
  (*Library).Extension() / OutputFile() / FunctionNames()
  ```

  Runnable example at [`examples/embed-html-dsl/`](examples/embed-html-dsl).
  Guide: [`docs/embedding.md`](docs/embedding.md).

- **`type NAME ... end` blocks in `.capy` libraries** with `base`,
  `pattern`, and `options` directives — full parity with the YAML
  `types:` section.

- **Seven new showcase samples**:
  - `samples/multi-language-demo/` — one source → Python, JavaScript,
    Go, Rust, and C (every library ships in BOTH .yaml and .capy form).
  - `samples/3d-tools-demo/` — one scene → Blender, SketchUp, Rhino,
    Unity, Unreal scripts.
  - `samples/typed-config-dsl/` — named typed captures + custom types
    with pattern/options/base; valid + invalid input both committed.
  - `samples/interactive-breakout/` — event-driven Breakout DSL with
    `on_key` / `on_event` primitives; 18-line DSL → 226-line working
    game with playable iframe in the docs.
  - `samples/interactive-snake/` — event-driven Snake with dual key
    bindings (arrows + WASD), event handlers, localStorage scoring.

### Changed

- Outer parser now recursively dispatches library types with `base:`
  to the base type's token rules. Without this, `type Port { base: int }`
  + `port 8443` failed because the lib-type path only accepted
  ident/string tokens.

- `file_template:` in `.capy` libraries now captures to EOF and uses
  the first non-blank line's indent as the strip width — so authors
  can place template actions (e.g. `{{ .body | indent 8 }}`) at
  column 0 for clean nested indentation.

- Golden test harness now picks up samples whose library is either
  `lib.yaml` or `lib.capy`, and skips `lib.capy` from the script glob.

- Infra parsers gain `ParseBytes` methods for in-memory parsing.
  New public `orchfeatures.LoadLibraryFromBytes(format, src, tokenize)`
  used by the embedding API.

### Docs

- New showcase sections: playable games (event-driven DSL), named
  variables & type checking, one scene → five 3D tools, one source
  → five programming languages, `.capy` library format.
- New pages: `docs/embedding.md`, `docs/capy-libraries.md`.
- Homepage feature grid expanded.

## [0.1.0] — 2026-05-23

Initial public release.

### Added

- **Zero default grammar** engine: every user-facing token shape is defined
  by the loaded library.
- **`functions:`** with kind-discriminated `args:` (`{kind: literal, value}`
  or `{kind: capture, name, type}`). Auto-name-prepend rule when args has
  no literals.
- **`types:`** with three optional validators applied in order: `base`,
  `pattern` (regex), `options` (enum).
- **`context:`** for the accumulated transpilation state; **`run:`** for
  context-mutation snippets in a small inner DSL (`set`, `append`,
  `prepend`, `merge`, `delete`, `if`, `loop`, `regex_match`, `error`).
- **`template:`** per function for body output; **`file_template:`** for
  final-file assembly using `.body` + `.context`.
- **Two block modes**: indent/dedent + named closer (`block: { closer }`)
  or explicit delimiter pair (`block: { open, close }`).
- **Template helpers**: `indent`, `lower`, `upper`, `join`, `toQuoted`,
  `toPyLit`, `toJSON`, `toJSONIndent`.
- Object-literal keys accept quoted strings OR bare identifiers
  (`{name: "x"}` is valid).
- Six samples: `empty-engine`, `assembly`, `types`, `scene-dsl`,
  `transpile-py`, `transpile-json`.
- VHCO project layout.

### Known limitations

- No `else` arm on inner `if` (single-arm only).
- No argument `default:` values.
- No multi-output: each library produces exactly one file.
- No configurable surface syntax (statement terminator, arg separator)
  — deferred to a future version.
- No `import` between library files.

[Unreleased]: https://github.com/olivierdevelops/capy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/olivierdevelops/capy/releases/tag/v0.1.0
