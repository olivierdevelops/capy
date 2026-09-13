# Capy samples

**50 self-contained demos.** Each shows a compact source DSL producing a
substantial, useful target file. Every demo ships a `lib.yaml`,
`script.capy`, `README.md`, and a verified golden output.

Run any sample:

```sh
cargo build --release --manifest-path rust/Cargo.toml -p capy-cli
./capy run samples/<name>/lib.yaml samples/<name>/script.capy
```

Run every sample's golden test:

```sh
cargo test --manifest-path rust/Cargo.toml --test golden   # 116 cases, ~0.2s
```

---

## Concept demos (start here)

These teach the core model. Read them in order to see how every feature
plugs together.

| Folder | What it shows |
|--------|----------------|
| [empty-engine/](empty-engine/) | Zero default grammar: every script is rejected without a library. |
| [types/](types/)               | `pattern:` / `options:` / `base:` validation, with a failing-validation companion script. |
| [scene-dsl/](scene-dsl/)       | A pure declarative DSL with no control flow defined. |
| [builtin-functions/](builtin-functions/) | Live tour of every built-in template helper (`pascalCase`, `decoded`, `escapeHtml`, `percent`, `stars`, `indent`, …). Verifies the [function cookbook](../docs/function-cookbook.md). |
| [library-keywords/](library-keywords/) | The canonical `if … end` block — `function` / `arg literal` / `arg capture` / `block_closer` / `write` / `${indent N body}`. Verifies the [library keyword cookbook](../docs/library-keywords.md). |

---

## Web frontend

Compact DSLs that produce substantial frontend assets.

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-canvas-game/](transpile-canvas-game/)       | HTML5 canvas game | Full runnable page: sprites + key handlers + RAF game loop, ~50 LoC from 10. |
| [transpile-css-animations/](transpile-css-animations/) | CSS               | `@keyframes` + classes with animation bindings; `dasherize` helper for snake_case → kebab. |
| [transpile-react-component/](transpile-react-component/) | React TSX       | Typed `useState`/`useEffect` component from 7-line spec. |
| [transpile-landing-page/](transpile-landing-page/)     | HTML page         | Responsive landing page with embedded CSS, hero, features, CTAs. |
| [html-component/](html-component/)                     | HTML              | Mode-B `{...}` blocks for component DSLs. |
| [html-xml-parser/](html-xml-parser/)                   | HTML / XML        | Parse any well-formed `<tag>…</tag>` with ONE generic function — capture-bound `block_close_seq` + `attribute*` nonterminal; mismatched nesting errors. |
| [bbcode-parser/](bbcode-parser/)                       | HTML              | `[b]…[/b]` BBCode tags → HTML via per-tag literal `block_close_seq`; bracket close-sequences; mismatch errors. |
| [markdown-from-tags/](markdown-from-tags/)             | Markdown          | The SAME `<tag>…</tag>` markup retargeted to Markdown — proves close-seq parsing is target-agnostic; template alone picks the output. |
| [signature-parser/](signature-parser/)                 | Text              | Function-signature DSL: `param* sep "," join ", "` repeats a `bare` nonterminal — `sep` (input) and `join` (output) separators are independent. |
| [transpile-form/](transpile-form/)                     | HTML form         | Form block wraps field/textarea statements. |
| [transpile-email-html/](transpile-email-html/)         | HTML email        | Inline-styled email that survives every client. |

## Backend / server

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-express-server/](transpile-express-server/) | Node Express      | Routes + middleware → complete `server.js`. |
| [transpile-flask-app/](transpile-flask-app/)           | Python Flask      | Route DSL → Flask app with jsonify/request wiring. |
| [transpile-fastapi-app/](transpile-fastapi-app/)       | Python FastAPI    | Pydantic models + endpoints → typed FastAPI app. |

## Code generation

Source-to-source DSLs that emit code in a target programming language.

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-py/](transpile-py/)             | Python  | Full transpiler: imports, blocks, control flow, indented bodies. |
| [transpile-typescript/](transpile-typescript/) | TS  | Mode-A blocks; combining `template:` and `run:`. |
| [transpile-go/](transpile-go/)             | Go      | Struct generation; required imports tracked in context. |
| [transpile-sql/](transpile-sql/)           | SQL     | Multi-literal patterns (`select ... from ... where ...`). |
| [transpile-protobuf/](transpile-protobuf/) | .proto  | Leading capture (no function-name literal) for field declarations. |
| [transpile-graphql/](transpile-graphql/)   | SDL     | Two block kinds (`type`, `enum`); `required` variant adds `!`. |
| [transpile-tests/](transpile-tests/)       | Go test | Block per test; assertions render to `t.Errorf`. |
| [transpile-cli/](transpile-cli/)           | Cobra Go| Two context lists (commands + flags) assembled in file template. |
| [transpile-bash/](transpile-bash/)         | Bash    | Defensive script with `set -euo pipefail`, logging, guards. |
| [assembly/](assembly/)                     | NASM x86-64 | High-level source → real assembly; `.data` section built from accumulated symbols. |

## Configuration / IaC

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-json/](transpile-json/)               | JSON config       | Pure context-driven; empty templates. |
| [transpile-env/](transpile-env/)                 | `.env` file       | Library-defined type enforces SCREAMING_SNAKE. |
| [transpile-dockerfile/](transpile-dockerfile/)   | Dockerfile        | Linear instruction-by-instruction. |
| [transpile-makefile/](transpile-makefile/)       | Makefile          | Map-keyed context; file-template ranges. |
| [transpile-nginx/](transpile-nginx/)             | nginx.conf        | Mode-B `{...}` blocks for server blocks. |
| [transpile-systemd/](transpile-systemd/)         | systemd unit      | Three context maps render as INI sections. |
| [transpile-kubernetes/](transpile-kubernetes/)   | k8s manifest      | Pure context accumulation → multi-section YAML. |
| [transpile-gh-actions/](transpile-gh-actions/)   | GH workflow       | Job blocks emit YAML body fragments. |
| [transpile-cron/](transpile-cron/)               | crontab           | Multiple preset shapes (`daily`, `weekly`, `every`). |
| [transpile-terraform/](transpile-terraform/)     | Terraform HCL     | Resource blocks with arbitrary set/tag/relation statements. |
| [transpile-openapi/](transpile-openapi/)         | OpenAPI 3 YAML    | Endpoints + schemas → Swagger-ready spec. |
| [transpile-prometheus-alerts/](transpile-prometheus-alerts/) | Prometheus rules | Alert blocks with `expr`/`for`/`severity`/`summary`. |
| [transpile-chrome-extension/](transpile-chrome-extension/) | MV3 manifest.json | Extension spec → ready-to-load Chrome extension manifest. |

## Schemas / models

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-postgres-schema/](transpile-postgres-schema/) | PostgreSQL DDL | Tables + columns + indexes + foreign keys. |
| [transpile-prisma-schema/](transpile-prisma-schema/)     | Prisma schema  | Datasource + generator + models with relations. |
| [transpile-zod-schema/](transpile-zod-schema/)           | Zod (TS)       | Zod object schemas + typed `Schemas` export. |
| [transpile-xstate-machine/](transpile-xstate-machine/)   | XState v5 (TS) | States + transitions → createMachine call. |

## Markdown / documentation

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-markdown-todo/](transpile-markdown-todo/)   | Markdown checklist  | `unquote` helper for clean output. |
| [transpile-blog/](transpile-blog/)                     | Markdown post       | YAML front matter + body; tag list via JSON. |
| [transpile-changelog/](transpile-changelog/)           | Keep-a-Changelog    | Version blocks with categorised entries. |
| [transpile-resume/](transpile-resume/)                 | Markdown CV         | Header info in context; experience entries as blocks. |
| [transpile-api-docs/](transpile-api-docs/)             | API reference       | Route blocks; counts via context list length. |
| [transpile-invoice/](transpile-invoice/)               | Markdown invoice    | Line items as a list of objects; table render. |

## Data / diagrams / specs

| Folder | Target | Notable feature |
|--------|--------|-----------------|
| [transpile-csv/](transpile-csv/)                   | CSV               | List captures via `join`. |
| [transpile-mermaid/](transpile-mermaid/)           | Mermaid flowchart | Two edge shapes; longer pattern wins on overlap. |
| [transpile-statemachine/](transpile-statemachine/) | Mermaid state     | Five-token transition pattern (`A -> B on "event"`). |
| [transpile-slack-blocks/](transpile-slack-blocks/) | Slack Block Kit   | Message DSL → Block Kit JSON; paste into webhook. |

---

## How goldens work

Each script has a paired expected file:

- `<base>.expected.txt` — for runs that should succeed.
- `<base>.expected-error.txt` — for runs that should error.

`go test ./...` compares actual output to its golden. To regenerate
after intentional changes:

```sh
CAPY_UPDATE_GOLDENS=1 cargo test --manifest-path rust/Cargo.toml --test golden
```

## Adding a new sample

```sh
capy init samples/my-new-sample
# edit lib.yaml + script.capy
capy run samples/my-new-sample/lib.yaml samples/my-new-sample/script.capy > samples/my-new-sample/script.expected.txt
cargo test --manifest-path rust/Cargo.toml --test golden
```

Then add your sample to the appropriate section above and write a brief
`README.md` explaining what it teaches.
