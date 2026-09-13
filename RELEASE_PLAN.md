> **HISTORICAL PLANNING RECORD.** Written for the Go implementation, before the
> Rust port. The file paths, linters and release tooling named below describe
> that codebase; the engine is now Rust under `rust/` and releases are built with
> cargo. Kept for the plan and acceptance criteria, not as current instructions.

# Capy Public Release Plan

> **Status: completed for v0.1.0.** Phases 0–7 have been executed in this
> repository. See [CHANGELOG.md](CHANGELOG.md) for what shipped. This
> document is kept as a reference for future releases — clone the
> phase/checklist structure for v0.2 etc.



Phased plan to take Capy from "works on my machine" to a credible public open-source release. Each phase lists deliverables, concrete file paths, and a priority tag.

Legend:
- **🔴 MUST** — required for a credible v0.1.0 release.
- **🟡 SHOULD** — strongly recommended; do soon after launch.
- **🟢 NICE** — fine to defer; track as roadmap.

---

## Phase 0 — Repo hygiene (🔴 MUST)

The bare minimum to make the repo look like an open-source project, not a personal scratch dir.

| Task | File / Action |
|---|---|
| Choose & add a LICENSE (MIT or Apache-2.0 most common for Go tooling) | `LICENSE` |
| Pick a Go module path and update | `go.mod` → e.g. `module github.com/<you>/capy` |
| Remove personal/agent state from the repo | Delete or `.gitignore` `.claude/`, `.codestyle` (move to `docs/architecture.md`) |
| Verify `.gitignore` covers binaries, OS junk, editor configs | `.gitignore` |
| Public-facing README (NOT the language spec) | `README.md` rewrite (see §Phase 3 for structure) |
| Initial changelog | `CHANGELOG.md` (Keep-a-Changelog format, `## [Unreleased]` + `## [0.1.0]`) |
| Code of Conduct (Contributor Covenant) | `CODE_OF_CONDUCT.md` |
| Contribution guide | `CONTRIBUTING.md` |
| Security policy + disclosure email | `SECURITY.md` |
| Issue/PR templates | `.github/ISSUE_TEMPLATE/bug_report.md`, `feature_request.md`, `library_request.md`, `.github/pull_request_template.md` |
| Sponsorship/funding (optional) | `.github/FUNDING.yml` |
| Codeowners | `.github/CODEOWNERS` |

**Acceptance:** `gh repo view` shows a healthy "community" tab with all green checks.

---

## Phase 1 — Engine quality (🔴 MUST)

You can't release a parser/interpreter with no tests.

### 1.1 Unit tests

| Package | Tests to write | File |
|---|---|---|
| Lexer | INDENT/DEDENT, strings, multi-line objects, brackets, numbers/floats, all punct ops | `orchestrator/features/make_lexer_test.go` |
| Outer parser | Each PatternElement kind, auto-name-prepend rule, delimiter-block vs closer-block, error positions | `orchestrator/features/make_parser_test.go` |
| Value parser | Lists, objects (quoted + ident keys), nested, multi-line | `orchestrator/features/value_parser_test.go` |
| Inner parser | All statements (set/append/prepend/merge/delete/if/loop/call) | `orchestrator/features/inner_parser_test.go` |
| Inner evaluator | Path writes (dot + bracket index), conditional skip, loop, regex_match, error | `orchestrator/features/inner_evaluator_test.go` |
| Library loader | kind discriminator validation, type cross-refs, closer cross-refs, run-snippet parse errors | `orchestrator/features/make_library_loader_test.go` |
| Outer evaluator | Type validation pass/fail, body rendering, closer rendering, captureToText, file_template assembly | `orchestrator/features/make_evaluator_test.go` |
| Template engine | All helpers (indent, lower, upper, join, toQuoted, toPyLit, toJSON, toJSONIndent) | `infra/template_engine_test.go` |

### 1.2 Golden-file integration tests

Run every `samples/*/` end-to-end and compare against a stored `expected.txt`.

| Item | File |
|---|---|
| Golden harness | `internal/goldentest/runner.go` + `cmd/capy/main_test.go` |
| Expected outputs per sample | `samples/<name>/expected.txt` (and `expected-error.txt` for failing cases) |

### 1.3 Lint / static analysis

| Tool | Config |
|---|---|
| `go vet` | run in CI |
| `golangci-lint` | `.golangci.yml` (default linters: errcheck, gosimple, staticcheck, ineffassign, govet, unused, gofmt) |
| `goimports` enforcement | pre-commit + CI |

### 1.4 Continuous integration

```yaml
# .github/workflows/ci.yml
name: ci
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        go: ['1.22', '1.23']
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - actions/checkout
      - actions/setup-go
      - cargo test --workspace
      - golangci-lint run
```

**Acceptance:** CI green on Linux/macOS/Windows × Go 1.22 + 1.23. ≥80% line coverage on the engine packages.

---

## Phase 2 — CLI ergonomics (🔴 MUST)

The CLI is what users touch first; it has to feel polished.

### 2.1 Subcommands

```
capy run <script>          # run a script (default if first arg is a .capy file)
capy check <library.yaml>  # parse + validate a library, report errors, no execution
capy init [<dir>]          # scaffold a new library project
capy fmt <script>          # format a Capy source file  [🟡 SHOULD]
capy watch <script>        # re-run on file change       [🟢 NICE]
capy version
capy help [<command>]
```

### 2.2 Better error messages

Show line + column + a caret pointing at the offending token.

```
error: line 3, col 5: no library function matches token "x"
  3 │     x = 1
        │     ^
```

| File | Change |
|---|---|
| `domain/errors.go` (NEW) | Structured `CapyError { Line, Col, Code, Msg }` with `Format(source string) string` |
| `orchestrator/features/make_parser.go` | Return CapyError values instead of plain `fmt.Errorf` |
| `orchestrator/features/make_evaluator.go` | Same |
| `io/cli/view.go` | Render CapyError with source context |

### 2.3 Flags

| Flag | Behavior |
|---|---|
| `--lib <path>` | Library file (existing) |
| `-o, --out <path>` | Override `output_file:` |
| `--debug` | Verbose engine tracing |
| `--no-color` | Disable ANSI |
| `--version` | Print version + build info |

Embed version via `-ldflags "-X main.version=$(git describe --tags)"`.

**Acceptance:** Type a malformed script and you get a useful caret-pointed error in <1 second.

---

## Phase 3 — Documentation (🔴 MUST)

Split the current monolithic `USAGE.md` into a navigable docs tree under `docs/`.

```
docs/
├── README.md                       # docs index
├── getting-started.md              # 5-minute quickstart
├── language-reference.md           # full surface grammar + lexer behavior
├── library-authoring.md            # write your own lib.yaml
├── types.md                        # base / pattern / options + examples
├── inner-dsl.md                    # set/append/.../if/loop reference
├── templates.md                    # Go text/template + Capy helpers + idioms
├── block-functions.md              # closer-mode vs delimiter-mode blocks
├── transpiler-patterns.md          # accumulated context, file_template, etc.
├── cli.md                          # capy run / check / init / fmt
├── architecture.md                 # VHCO layout, internal data flow
├── cookbook.md                     # 15+ snippets for common needs
├── faq.md                          # 20+ common questions
├── roadmap.md                      # what's planned (dynamic syntax, validate: snippets, else, defaults, …)
└── migration-guide.md              # v→v upgrade notes (empty for v0.1.0)
```

### 3.1 Top-level public README

A NEW `README.md` that opens with:
1. One-sentence pitch ("Capy is a transpiler engine: define a tiny source language in YAML, get a code generator.").
2. 30-second teaser code block (input + output).
3. Install commands.
4. Links to the four canonical samples.
5. Why-Capy section (vs templating engines, vs ANTLR, vs ad-hoc scripts).
6. Status badge row (build, coverage, release, license, go report).
7. Link to docs.

The current spec-style README moves to `docs/language-reference.md`.

### 3.2 Tutorials

`docs/tutorials/` — 4 progressive walkthroughs:

| Tutorial | Goal | File |
|---|---|---|
| 01 — Hello world DSL | Define one function + template | `docs/tutorials/01-hello-world.md` |
| 02 — Building a config DSL | Use types + context to make a JSON config language | `docs/tutorials/02-config-dsl.md` |
| 03 — Transpiling to Python | Block functions, file_template, context.imports | `docs/tutorials/03-transpile-python.md` |
| 04 — Custom Operators | Define `+`, `=`, `.` patterns; multi-token shapes | `docs/tutorials/04-custom-operators.md` |

### 3.3 Cookbook recipes

`docs/cookbook.md` — short, self-contained answers to:

- How do I emit an import block at the top of the output?
- How do I deduplicate context entries?
- How do I validate that a string matches multiple patterns (AND)?
- How do I make a function whose output depends on context?
- How do I support both `{...}` blocks and `do...end` blocks?
- How do I emit indented bodies?
- How do I render a list of objects from context?
- How do I output to multiple files? *(needs feature work — roadmap)*

### 3.4 In-repo example libraries

Beyond the 6 current samples, add:

| Sample | Demonstrates |
|---|---|
| `samples/transpile-sql/` | Define a tiny query DSL → SQL |
| `samples/transpile-makefile/` | Build a Makefile from declarative tasks |
| `samples/transpile-typescript/` | Show TypeScript target |
| `samples/transpile-env/` | Generate `.env` from a typed config |
| `samples/html-component/` | Custom JSX-ish syntax → HTML |

Each gets `lib.yaml`, `script.capy`, `expected.txt`, `README.md`.

**Acceptance:** A newcomer can go from `go install` to "I wrote my own DSL" in under 30 minutes by following the docs.

---

## Phase 4 — Tooling & editor support (🔴 MUST)

### 4.1 JSON Schema for `lib.yaml`

`schemas/library.schema.json` — full JSON Schema for the YAML library format. Enables:
- VS Code/Cursor `yaml.schemas` integration → instant autocomplete + diagnostics.
- Agent grounding (paste schema into the system prompt).
- `capy check` uses it as a first-pass structural validation.

### 4.2 VS Code language config

`editors/vscode/capy/` — a tiny extension contributing:
- `.capy` file association
- Syntax highlighting (TextMate grammar — punct, idents, strings, numbers, brackets, comments)
- Brace/bracket matching
- Snippet pack
- `yaml.schemas` mapping for `lib.yaml`

Publishing to the Marketplace can wait; ship sources first.

### 4.3 Tree-sitter grammar (🟡 SHOULD)

`grammars/tree-sitter-capy/` — minimal tree-sitter grammar. Enables Neovim/Helix/Emacs/Zed highlighting.

### 4.4 LSP server (🟢 NICE)

Out of scope for v0.1.0. Note in roadmap.

---

## Phase 5 — Distribution (🔴 MUST)

### 5.1 GoReleaser

`.goreleaser.yaml`:
- Build matrix: `linux/amd64`, `linux/arm64`, `darwin/amd64`, `darwin/arm64`, `windows/amd64`.
- SHA256 checksums.
- Cosign signatures (🟡 SHOULD).
- Auto-generated changelog from commit messages.
- Homebrew tap update.

### 5.2 Release workflow

`.github/workflows/release.yml` — triggered by `v*` tag:
1. Run full CI.
2. Run GoReleaser.
3. Publish GitHub Release with assets.
4. Update Homebrew tap.

### 5.3 Install paths

| Method | Command |
|---|---|
| Go | `go install github.com/<you>/capy/cmd/capy@latest` |
| Homebrew | `brew install <you>/tap/capy` |
| Curl | `curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh \| sh` |
| Binary | Download from GitHub Releases |
| Docker | `docker run ghcr.io/<you>/capy:0.1.0 capy ...` |

`install.sh` lives in `scripts/install.sh` (POSIX shell, detects OS/arch, downloads, verifies checksum).

### 5.4 Versioning policy

State explicitly in `CONTRIBUTING.md`:
- Currently **v0.x — pre-1.0**: the library YAML schema may break between minor versions.
- Engine API (the Go packages) is unstable.
- Each breaking change gets a `## Breaking` section in `CHANGELOG.md` and a migration note.

**Acceptance:** `gh release view v0.1.0` shows binaries for 5 platforms with checksums. `brew install` works.

---

## Phase 6 — Agent / AI ecosystem (🟡 SHOULD — the part you specifically called out)

Capy benefits a lot from AI-assisted authoring: writing a `lib.yaml` is exactly the kind of structured task LLMs are good at.

### 6.1 Claude Code skill — "Capy library author"

`skills/capy-author/` — an Anthropic Skill conforming to the Skills spec:

```
skills/capy-author/
├── SKILL.md                  # description + frontmatter
├── instructions.md           # full skill prompt
├── reference/
│   ├── schema.md             # the library schema in prose
│   ├── inner-dsl.md          # primitives reference
│   ├── template-helpers.md   # all template helpers
│   ├── examples.md           # 6+ canonical libraries with annotations
│   └── pitfalls.md           # common mistakes the model should avoid
└── tools.md                  # any required tool permissions
```

Triggers on prompts like "build a Capy library for ...", "transpile this to Capy DSL", "write a function definition for ...".

### 6.2 Claude Code slash commands

`commands/capy/` — slash commands that wrap the skill:

| Command | Behavior |
|---|---|
| `/capy-new <target-language>` | Scaffold a new library tailored for the target |
| `/capy-add-function` | Interactively add a function definition to the current `lib.yaml` |
| `/capy-add-type` | Add a `types:` entry with regex/options |
| `/capy-explain <file>` | Walk through an existing library and explain it |
| `/capy-debug` | Diagnose why a source file isn't matching |

### 6.3 Other agents

| Integration | File |
|---|---|
| Cursor rule | `.cursor/rules/capy.md` (also ship in `editors/cursor/`) |
| Continue config snippet | `editors/continue/capy.json` |
| Aider system prompt addition | `editors/aider/capy-aider.md` |
| Generic system prompt | `agents/capy-system-prompt.md` (drop-in for any model) |

### 6.4 LLM-friendly artifacts

Ensure these are easy for any model to consume:

- `schemas/library.schema.json` — the canonical schema.
- `docs/CAPY_FOR_LLMS.md` — a single-page, dense, complete summary intended to be pasted into a context window. Includes the schema, the inner-DSL grammar, the template helpers, 3 worked examples, and a list of "common mistakes" to avoid.

**Acceptance:** A user types "build me a Capy library that transpiles Markdown to BBCode" into Claude Code with the skill installed and gets a working `lib.yaml` + `script.capy` + `expected.txt` first try.

---

## Phase 7 — Community & launch (🟡 SHOULD)

### 7.1 Logo + branding

A capybara silhouette. Two assets:
- `assets/logo.svg` (square, transparent)
- `assets/banner.svg` (1280×640 social card)
- Favicon for any future website.

### 7.2 Demo gif

A 20-second terminal recording showing: write `lib.yaml`, write `script.capy`, run `capy run`, see output. Use `vhs` or `asciinema`. Drop in `assets/demo.gif`; embed in `README.md`.

### 7.3 Launch content

- Blog post: "Capy — a transpiler engine in 1000 lines of Go" (architecture + motivation).
- HN post draft: `docs/launch/hn.md` (don't post until v0.1.0 is stable for a week).
- Reddit posts: `/r/golang`, `/r/programminglanguages`.
- Twitter/Mastodon/Bluesky thread template.
- Mention in `r/ProgrammingLanguages` weekly thread first to get early feedback.

### 7.4 Community surfaces

- Enable GitHub Discussions.
- Create `awesome-capy` companion repo (libraries, integrations).
- "Show your DSL" Discussions thread pinned.
- Discord server (or pin a "no chat server, use Discussions" note).

### 7.5 Pre-launch dogfood

Write 3 of YOUR OWN libraries that you actually use, before launch:
- One config DSL.
- One code-gen tool.
- One personal automation.
These become testimonials and good "real usage" samples.

---

## Phase 8 — Stabilization roadmap (🟢 NICE / post-launch)

Document these as "planned" in `docs/roadmap.md` so users see direction without expecting it day-one.

| Feature | Why deferred |
|---|---|
| Configurable surface syntax (`syntax:` section, deferred earlier) | Substantial parser work; not blocking any v1 use case |
| `validate:` types written in inner Capy | Bootstrapping complexity |
| `else` arm for inner `if` | Single-arm is fine for now |
| Argument `default:` values | Easy to re-add; orthogonal |
| Multi-output (`outputs:` list with selectors) | Major feature; needs design |
| `import` between library files | Modularity; design needed |
| LSP server | Significant project |
| WASM build (run Capy in the browser) | Cool demo; not blocking |
| Capy itself written in Capy (the inner-DSL bootstrap) | Self-host milestone |

---

## Execution order recommendation

If you want a v0.1.0 in roughly **3–4 weekends**:

| Weekend | Focus | Outcome |
|---|---|---|
| 1 | Phase 0 + Phase 1 tests | Repo is presentable, CI green |
| 2 | Phase 2 CLI ergonomics + Phase 3 docs (getting-started + language-reference + library-authoring) | New users can self-serve |
| 3 | Phase 4 schema + VS Code + Phase 5 GoReleaser | One-command install for everyone |
| 4 | Phase 6 Claude skill + Phase 7 launch materials | Tag `v0.1.0`, write blog post, soft-launch on r/ProgrammingLanguages |

Phases 6 and 7 can be moved to a v0.2 if you'd rather ship sooner. The minimum credible launch is **0 → 5**.

---

## Pre-flight checklist (the day you tag v0.1.0)

- [ ] All Phase 0 files exist
- [ ] CI green on `main`
- [ ] `cargo test --workspace` passes
- [ ] `golangci-lint run` clean
- [ ] Every `samples/*/expected.txt` matches actual output
- [ ] `capy --version` prints the tag
- [ ] `capy --help` is helpful
- [ ] `README.md` opens with a working code block, install instructions, and links
- [ ] `docs/` has at minimum getting-started, language-reference, library-authoring, cookbook, faq, roadmap
- [ ] `CHANGELOG.md` has `## [0.1.0] — YYYY-MM-DD`
- [ ] GoReleaser produces binaries for all 5 platforms
- [ ] Install script tested on macOS + Linux
- [ ] License is correct; no copy-pasted code with incompatible licenses
- [ ] No `.claude/` or other personal state in the repo
- [ ] One real, non-demo library written by you that you actually use
- [ ] You can do the full "newcomer demo" (install → first DSL) in under 10 minutes flat