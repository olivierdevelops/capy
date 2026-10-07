# Capy for AI agents

Capy was designed to be useful to humans, but its model unlocks two
properties that are quietly enormous for AI workflows:

1. **Token compression** — agents emit short structured source instead
   of long boilerplate-heavy code. The library does the rest.
2. **Sandboxing** — the library is a contract. Whatever the agent
   produces is constrained to shapes the library defined. No prompt
   injection, no arbitrary shell escape, no malformed output.

This page explains how to use Capy with LLMs and agentic frameworks to
get both.

---

## Why the model fits LLMs

When an LLM generates code, it spends most of its tokens on boilerplate
the surrounding context already implies: function signatures, import
statements, repeated patterns, error handling scaffolding, type
declarations the user didn't ask for.

Capy turns that boilerplate into **library data** — written once, reused
forever. The agent emits the **semantic content**, which is typically
5–10× smaller, and the engine deterministically expands it into the
target language.

| What the LLM emits             | What gets produced               | Compression |
|--------------------------------|----------------------------------|-------------|
| 12 lines of Capy game spec     | 67 lines of runnable HTML5 game  | 5.5×        |
| 9 lines of landing-page DSL    | 54 lines of responsive HTML+CSS  | 6.0×        |
| 4 lines of Kubernetes spec     | 13 lines of multi-section YAML   | 3.2×        |
| 8 lines of Express route DSL   | 24 lines of Node server          | 3.0×        |
| 5 lines of schema DSL          | 16 lines of PostgreSQL DDL       | 3.2×        |

The ratio is biggest for boilerplate-heavy targets (UI, infra,
plumbing). For dense, semantically-rich targets (Python with control
flow) the savings are smaller — but the agent still benefits from the
**determinism** (no off-by-one bugs, no missing imports) and
**validation** (the library type-checks args at parse time).

These numbers are for one-shot generation. **In agentic loops the gap
compounds**: the same library is reused across hundreds of calls; the
per-call cost approaches the size of the source the agent emits.

---

## Why sandboxing emerges for free

The library is **the complete grammar** of the source language. Anything
not declared as a function is a parse error. This makes Capy a natural
sandbox for AI code generation in domains where you can't trust the
model to emit safe text.

### Example 1: a restricted SQL builder

Define a tiny SQL DSL that **only** supports `SELECT … FROM … WHERE …`
on a whitelist of tables. The model cannot emit `DROP TABLE` — the
library doesn't define that shape.

```
type TableName
    options "users" "posts" "comments"          # ← whitelist
end

function query
    arg literal "select"
    arg capture cols any
    arg literal "from"
    arg capture tbl TableName                    # ← validated
    arg literal "where"
    arg capture cond any
    write `SELECT ${cols} FROM ${tbl} WHERE ${cond};
`
end
```

The LLM can write:

```
select id from users where active
```

…and gets parameterised SQL. The LLM **cannot** write:

```
DROP TABLE users; --
```

…because there is no `DROP` pattern, and even `select * from secrets …`
fails because `secrets` is not in the `TableName` options. The agent is
constrained by data, not by an after-the-fact filter.

### Example 2: a sandboxed shell-script writer

Define commands the agent is allowed to invoke; nothing else exists.

```
type Command
    options "ls" "cat" "grep" "head" "tail"        # ← read-only commands
end

function run
    arg literal "run"
    arg capture cmd Command
    arg capture args any
    write `${cmd} ${args}
`
end
```

Even if the agent emits `run rm -rf /`, the type check fails: `rm` is
not in `options`. The output never leaves the engine in a runnable
form.

### Example 3: a typed API client

```
type Method
    options "GET" "POST"
end

type Host
    pattern "^https://api\\.mycompany\\.com/"
end

function api
    arg literal "api"
    arg capture method Method
    arg capture url Host
    arg capture body any
    write `fetch(${url | toQuoted}, {
  method: ${method | toQuoted},
  body: JSON.stringify(${body}),
});
`
end
```

The model can hit your API. It can't hit `evil-corp.example.com`,
because the URL must match the host regex. It can't `DELETE`, because
the method enum forbids it.

### Why this matters for agents

For an agent that has tool-use access to your codebase, the standard
risk story is "what if the model emits something nasty?" With raw
codegen you need post-hoc filtering, sandboxes, or human review.

With Capy, the boundary is **declarative and static**. You hand the
agent a library; whatever it emits is by construction within that
contract. The agent never sees a way out — there is no way out — so
you don't spend prompt budget trying to fence it off.

---

## Five workflow patterns

### Pattern A: "Design the library once, agent emits source forever"

1. A human (or one initial LLM pass) writes the `lib.capy`.
2. From then on, the agent only emits Capy source.
3. The transpiler does the heavy lifting deterministically.

Cost profile:

- One-time library design: ~2000–5000 tokens.
- Per-invocation source: ~50–200 tokens.
- Output complexity is unbounded.

Best for: scaffolding tools, CRUD generators, anything where the same
target shape is produced many times.

### Pattern B: "Two-pass code generation"

1. Agent first emits a high-level Capy plan.
2. The transpiler expands it into the target language.
3. Optionally, a second agent reviews the target output.

This decouples *intent* from *expression*. The first agent worries
about what the program should do. The library worries about how to say
it correctly.

Best for: cases where the target language has many footguns (memory
safety, async correctness, security) but the semantic structure is
simple.

### Pattern C: "Validate user-provided code against a Capy DSL"

If users (or another agent) submit code in your custom DSL, Capy is
also the validator: load the library, try to parse the input, surface
caret-pointed errors. The same artifact is grammar + validator +
generator.

Best for: user-facing scripting (admin DSLs, low-code platforms,
shareable snippets).

### Pattern D: "Human defines the syntax, AI fits a library to it"

This one is the most underrated for non-programmers and the most
collaborative for everyone else. The human starts by writing the
**source** in whatever way feels natural — the keywords they wish
existed, the indentation they prefer, the verbs that map onto how
they think about the domain. The AI's job is to make Capy parse
that source by iterating on the **library**.

The split of responsibilities:

| Who | Owns |
|---|---|
| **Human** | The DSL surface — `main.<ext>`. The words, the shapes, the structure they want to write in. |
| **AI** | The library — `<ext>.capy`. Function shapes, captures, `write` templates, commands. Whatever it takes to make the human's source compile to the target output. |

A typical loop:

1. Human writes a few lines of their imagined DSL.
2. They paste it to the AI with a one-sentence goal: *"this should
   produce an HTML calculator"* / *"this should generate a Kubernetes
   manifest"* / *"this should compile to a SQL view."*
3. AI iterates a library (adding `function`, `arg`, `write`, `block_*`,
   commands…) until `capy main.<ext>` produces the right output.
4. AI may also iterate Capy itself — if the human's source style
   reveals a gap (`bare` for keyword-less rows, `tail` for free-form
   values, `block_dedent` for indent-only blocks, `top_level` for
   position-aware emission), the engine grows to fit. **The user's
   syntax is the spec; the toolchain adapts.**
5. Once the library is good, the human edits `main.<ext>` freely
   without AI help — they're writing in their own vocabulary. They
   only return to the AI when they want to grow the DSL itself.

Why this matters for non-programmers:

- The artifact they read and edit is **the DSL they designed** —
  not React, not Tailwind, not Terraform. The vocabulary fits the
  domain.
- The library + Capy engine + AI prompt are the "compiler stack",
  but the user never has to look at them. They just write.
- Edits stay deterministic. The library is fixed; the same DSL
  source always produces the same output. No "the AI wrote
  something different this time."
- The AI is on tap for *grammar-level* changes (new keywords,
  new constructs) but doesn't need to be in the loop for
  *content-level* edits.

Why it matters for AI workflows:

- Less prompt context burned on re-explaining the target language
  on every call.
- The AI's outputs are tiny (`50–200` tokens of DSL) instead of
  large (`800+` tokens of HTML/YAML/SQL). Cost and latency both
  drop.
- Reviewability: a human can read `main.<ext>` and audit it in
  seconds. They can't audit `index.html` plus inline `<script>`
  plus inline `<style>` as fast.

Concrete shape of a session:

```
# Round 1: human-authored source
USER (paste):

    TodoList "Today"
        DO "Buy groceries"
        DO "Finish report" priority high
        DONE "Email client"
    end

USER (goal): "Generate a clean HTML page with a checkbox list."

# AI iteration: add `TodoList`, `DO`, `DONE`, and a `priority`
# attribute function; wire a file_template. Commit `todolist.capy`.

# Round 2: human iterates the source freely
USER (edit main.todolist):

    TodoList "Today"
        DO "Buy groceries" priority high
        DO "Pick up dry cleaning"
        DONE "Email client"

    TodoList "This week"
        DO "Tax filing"
        DO "Annual review prep"
    end

# Re-run `capy main.todolist` — works without an AI call.

# Round 3: human wants something new
USER: "Add a `due TOMORROW` annotation that renders as a yellow badge."

# AI adds one function to `todolist.capy`. Human keeps writing.
```

Best for:
- Domain-specific tools authored by domain experts (recipe writers,
  game designers, ops engineers, marketers, teachers).
- Long-lived projects where the SAME human edits the source many
  times, and only occasionally needs grammar growth.
- "I want a `lib.capy`-as-IDE" workflows where the DSL becomes a
  living tool tailored to one person or team.

---

### Pattern E: "Emit, inspect, repair"

The pattern that closes the loop on generation. An agent that emits source into
your DSL should not hand it to `capy run` and hope — `run` stops at the first
error and tells you nothing about the rest of the file. `capy ast --json`
recovers past every failure and returns machine-readable diagnostics, so one
call tells the model everything that is wrong.

```text
   model emits source
          │
          ▼
   capy ast lib.capy out.capy --json
          │
          ├── exit 0, diagnostics: []  ──▶ accept, then capy run
          │
          └── exit 1 ──▶ feed diagnostics[] back to the model
                          (code + message + line:col, every region at once)
                          └──▶ re-emit ──┐
                                         └── bounded retries, then give up
```

```sh
capy ast lib.capy out.capy --json \
  | jq -r '.diagnostics[] | "line \(.primary.start_line): \(.message)"'
```

Why it matters for token cost: one round trip surfaces *all* the errors, so a
file with four mistakes takes one repair turn rather than four. And because the
tree comes back alongside the diagnostics, the model can be shown what it did
parse — usually more useful for repair than the error text alone.

Keep the retry count bounded and fall back to asking the human; a model that
cannot fix its own output in two passes is usually fighting a library gap rather
than a typo. See [AST JSON schema](ast-json.md) and
[Diagnostics](diagnostics.md).

Best for: any agent that writes DSL source unattended.

## Capy as a portable rendering layer for AI agents

This is the most impactful pattern for agentic systems. Instead of
asking an agent to write the **final artifact** (HTML, DOCX, PDF,
LaTeX, JSON…), ask it to write **one Capy source**. A library — not
the agent — turns that source into whatever target format the user
actually wants.

### The shape of the problem today

The usual "let an LLM produce a document" pipeline duplicates work:

```text
User: "make me a meeting agenda as a webpage"
→ LLM writes 200 lines of HTML + inline CSS

User: "now the same thing as a PDF"
→ LLM rewrites with ReportLab Python code

User: "same thing but a DOCX"
→ LLM rewrites with python-docx calls

User: "and a Markdown version for the wiki"
→ LLM rewrites with `#` and `-` and links
```

Same content, four wildly different generations, four chances for
drift, four times the token cost. The agent has to know HTML, PDF,
DOCX, AND Markdown — and the operating environment to produce each.

### The Capy way

The agent produces **one** terse Capy source. Multiple libraries
(one per target format) deterministically expand it into the final
artifacts. The agent never sees the target format. It never imports
`python-docx`, never composes inline styles, never escapes XML.

=== "Capy source (what the agent writes)"

    ```
    Meeting "Q4 planning" date "2026-01-15"

    Section "Goals"
        - "Ship the new auth flow"
        - "Retire the legacy CSV import"
        - "Decide on the on-call rotation"

    Section "Decisions"
        - "Pin Postgres at 16.x for the year"

    Section "Action items"
        Owner "alice" "Draft the auth migration RFC by Fri"
        Owner "bob"   "Audit usage of the legacy import"
    ```

    ~50 tokens. No HTML, no XML, no Python, no escaping.

=== "→ HTML (`meeting-html.capy` library)"

    ```html
    <!doctype html>
    <html><head><title>Q4 planning</title></head>
    <body>
      <h1>Q4 planning · 2026-01-15</h1>

      <h2>Goals</h2>
      <ul>
        <li>Ship the new auth flow</li>
        <li>Retire the legacy CSV import</li>
        <li>Decide on the on-call rotation</li>
      </ul>

      <h2>Decisions</h2>
      <ul><li>Pin Postgres at 16.x for the year</li></ul>

      <h2>Action items</h2>
      <ul>
        <li><strong>alice</strong> — Draft the auth migration RFC by Fri</li>
        <li><strong>bob</strong> — Audit usage of the legacy import</li>
      </ul>
    </body></html>
    ```

=== "→ Markdown (`meeting-md.capy` library)"

    ```markdown
    # Q4 planning · 2026-01-15

    ## Goals
    - Ship the new auth flow
    - Retire the legacy CSV import
    - Decide on the on-call rotation

    ## Decisions
    - Pin Postgres at 16.x for the year

    ## Action items
    - **alice** — Draft the auth migration RFC by Fri
    - **bob** — Audit usage of the legacy import
    ```

=== "→ LaTeX (`meeting-tex.capy` library)"

    ```latex
    \documentclass{article}
    \title{Q4 planning}
    \date{2026-01-15}
    \begin{document}
    \maketitle

    \section{Goals}
    \begin{itemize}
      \item Ship the new auth flow
      \item Retire the legacy CSV import
      \item Decide on the on-call rotation
    \end{itemize}

    \section{Decisions}
    \begin{itemize}
      \item Pin Postgres at 16.x for the year
    \end{itemize}

    \section{Action items}
    \begin{itemize}
      \item \textbf{alice} --- Draft the auth migration RFC by Fri
      \item \textbf{bob} --- Audit usage of the legacy import
    \end{itemize}
    \end{document}
    ```

=== "→ DOCX-XML (`meeting-docx.capy` library)"

    ```xml
    <?xml version="1.0"?>
    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
      <w:body>
        <w:p><w:r><w:t>Q4 planning · 2026-01-15</w:t></w:r></w:p>

        <w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr>
          <w:r><w:t>Goals</w:t></w:r></w:p>
        <w:p><w:r><w:t>• Ship the new auth flow</w:t></w:r></w:p>
        <w:p><w:r><w:t>• Retire the legacy CSV import</w:t></w:r></w:p>
        …
      </w:body>
    </w:document>
    ```

One source. Five libraries (the user, the agent operator, or anyone
else can author them — they're tiny). Pick the right one at render
time:

```sh
capy meeting-html agenda.meeting        # → agenda.html
capy meeting-md   agenda.meeting        # → agenda.md
capy meeting-tex  agenda.meeting        # → agenda.tex   then run pdflatex
capy meeting-docx agenda.meeting        # → agenda.xml   then zip → .docx
```

### Why this is better than "just use Markdown"

Markdown is the closest existing alternative — one source, many
renderers. But it's stuck at the formatting layer Markdown's spec
designed: paragraphs, headings, lists, bold/italic, links, code
fences. The moment you want **domain-specific** structure —
"Section", "Owner", "ActionItem", "Decision" — you have to either:

- Bend Markdown with extensions (custom directives, MDX) that each
  renderer interprets differently, or
- Drop down to raw HTML/JSX, losing portability.

Capy gives you both: a vocabulary you fully control (Markdown is
fixed; Capy is configurable) and a renderer matrix. Your library
**defines** what `Section` and `Owner` mean — and one library per
target tells Capy how to produce that target's version.

In other words: **Markdown is a fixed grammar with portable
renderers. Capy is a programmable grammar with portable renderers.**

### Why it's better for agent safety

This pattern collapses the agent's surface area to almost nothing:

- The agent never invokes a docx writer, a PDF library, or
  `subprocess.run`. It writes text into a single channel.
- The library is **the complete grammar** of that channel. If the
  agent tries to emit something the library doesn't define, the
  parser rejects it before any rendering happens.
- The agent never learns the user's filesystem, OS, or installed
  tooling. It doesn't matter whether `pandoc` is installed, where
  fonts live, or which Python the operator is running.
- Want to whitelist what the agent can talk about? Add `type`
  declarations with `pattern`/`options` to the library. The model
  literally cannot emit an action item against an unknown owner if
  `Owner` is constrained to a list.

```
type OwnerName
    options "alice" "bob" "carol"
end

function Owner
    arg literal "Owner"
    arg capture name OwnerName
    arg capture task any
    write `<li><strong>${unquote name}</strong> — ${unquote task}</li>
`
end
```

The agent now has **zero** capability to write `Owner "evilcorp"
"exfiltrate data"`. The grammar refuses the call before it reaches
any renderer.

### A typical agent skill, end-to-end

```python
# Pseudocode: "create a meeting agenda" agent skill.

SYSTEM_PROMPT = """
You write meeting agendas in the Capy `meeting` DSL.
Output ONLY Capy source — no commentary.

Available constructs:
""" + open("meeting-html.capy").read()  # ~300 tokens of grammar

def run_skill(user_request: str, target: str = "html"):
    # 1. Agent emits ~50–100 tokens of Capy.
    capy_source = llm.complete(SYSTEM_PROMPT, user_request)

    # 2. Capy turns it into the requested artifact.
    #    Different library per target; same source.
    library = f"meeting-{target}.capy"
    output  = subprocess.run(
        ["capy", "run", library, "/dev/stdin"],
        input=capy_source, check=True, capture_output=True,
    ).stdout

    # 3. Hand the artifact to the user / next stage.
    return output
```

Properties of the skill:

| Property | Without Capy | With Capy |
|---|---|---|
| Tokens out per call | 200–800 (full artifact) | 50–100 (DSL only) |
| Knowledge of target format the agent needs | Full (HTML / DOCX / PDF…) | None |
| Filesystem / shell access the agent needs | Whatever the artifact build requires | None |
| Output validates before render | Sometimes | Always (parser is the contract) |
| Add a new output format | Rewrite the prompt + re-train the agent | Author one new `.capy` library |
| Change branding / structure | Regenerate every artifact | Edit one library file |

### Skill template (drop into Claude Code / MCP / any tool runner)

```yaml
name: meeting-agenda
description: Generate a meeting agenda. Specify --target html|md|tex|docx.
parameters:
  - name: request
    type: string
    description: Natural-language meeting prompt.
  - name: target
    type: string
    enum: [html, md, tex, docx]
prompt: |
  You write meeting agendas in the Capy `meeting` DSL.
  Output ONLY Capy source between <capy>…</capy> tags.

  Grammar:
  {{ insert_file ./meeting-html.capy }}

  User request: {{ request }}
execute: |
  capy_src=$(echo "$LLM_OUTPUT" | sed -n 's:.*<capy>\(.*\)</capy>.*:\1:p')
  echo "$capy_src" | capy run "meeting-${target}.capy" /dev/stdin
```

The agent's *capability* is one input/output: produce well-formed
DSL. Everything else — file I/O, format conversion, target
selection — lives outside the agent.

### Highlights to take away

- **One agent output, N targets.** Author libraries per format;
  the agent stays target-agnostic.
- **Stronger than Markdown** because the vocabulary is yours, not
  CommonMark's.
- **Stronger than direct codegen** because the parser is the
  capability boundary — out-of-grammar emissions are rejected
  before any renderer runs.
- **Environment-free.** Agents don't need to know about `pandoc`,
  `pdflatex`, `python-docx`, where fonts live, or what the user's
  OS is. The host environment is the operator's problem; the agent
  just writes DSL.
- **Composable.** Same source feeds reports, dashboards, slide
  decks, API responses, configs — anywhere a deterministic
  templated text artifact is needed.

---

## Integrations shipped in this repo

| Tool | Where | What it gives you |
|------|-------|-------------------|
| **Claude Code skill** | [`skills/capy-author/`](https://github.com/olivierdevelops/capy/tree/main/skills/capy-author) | A full skill with `SKILL.md` + instructions + 5 reference docs the model loads on demand. Triggers on "write a Capy library for …" or any `.capy` file in context. |
| **Slash commands** | [`commands/capy/`](https://github.com/olivierdevelops/capy/tree/main/commands/capy) | `/capy-new <target>`, `/capy-add-function`, `/capy-add-type`, `/capy-explain`, `/capy-debug` |
| **One-page LLM brief** | [`CAPY_FOR_LLMS.md`](CAPY_FOR_LLMS.md) | Self-contained prompt for any model. Paste into Cursor/Continue/Aider/raw-API system message. |
| **Cursor rule** | [`editors/cursor/`](https://github.com/olivierdevelops/capy/tree/main/editors/cursor) | Drop in `.cursor/rules/capy.md` |
| **Continue config** | [`editors/continue/`](https://github.com/olivierdevelops/capy/tree/main/editors/continue) | Adds the LLM brief to context |
| **Aider read** | [`editors/aider/`](https://github.com/olivierdevelops/capy/tree/main/editors/aider) | `aider --read docs/CAPY_FOR_LLMS.md` |
| **Generic system prompt** | [`agents/capy-system-prompt.md`](https://github.com/olivierdevelops/capy/blob/main/agents/capy-system-prompt.md) | Drop-in for any tool not listed above |

---

## Token math, in detail

Most LLM cost (and most latency) is in **output tokens**. The naive way
to use an LLM for code generation:

```
User prompt: "build a Python Flask app with these 3 routes ..."
LLM output: ~800 tokens of Python (imports + Flask boilerplate + routes)
```

The Capy way:

```
Library `.capy` (once, in context or a file): ~400 tokens
User prompt: "build a Flask app with these 3 routes ..."
LLM output: ~50 tokens of Capy source
Engine output (deterministic, free): ~800 tokens of Python
```

The model emits **16× fewer tokens** for the same target. Over an
agentic loop of 100 generations, that's the difference between a
$2 task and a $0.10 task.

The library is amortised over its lifetime. Once it exists, every
subsequent call benefits. Many libraries are also reusable across
projects (e.g. a Cobra CLI library, a Drizzle schema library, a
Kubernetes manifest library).

---

## Determinism: an underrated win

LLM output is non-deterministic by design. Capy output is
deterministic by construction:

| Property | Raw LLM codegen | Capy + LLM |
|----------|-----------------|------------|
| Same input → same output | ❌ (varies per sample) | ✅ |
| Output passes a grammar check | ⚠️ (usually) | ✅ (always) |
| Output passes a type check | ⚠️ (sometimes) | ✅ (always, if library has types) |
| Output is well-formed JSON / YAML / SQL | ⚠️ | ✅ |
| Output uses only allowed APIs/tables/hosts | ⚠️ | ✅ |
| Token cost grows with target complexity | ✅ | ❌ (constant w.r.t. boilerplate) |
| Easy to audit what the agent can do | ❌ | ✅ (`capy check lib.capy`) |

The right-hand column is what makes Capy a genuinely useful primitive
in agent toolchains, not just a templating engine.

---

## Putting it together: a minimal agent loop

```python
# Pseudocode for an agent that emits Capy source.

LIBRARY = open("lib.capy").read()          # ~400 tokens
SYSTEM = (
    "You are an agent that emits Capy source code. "
    "Here is the only language you may use:\n\n" + LIBRARY +
    "\n\nReply with ONLY Capy source. The transpiler will run it."
)

for task in tasks:
    resp = llm.complete(system=SYSTEM, user=task)   # ~50 tokens out
    source = resp.text
    target = subprocess.run(
        ["capy", "run", "lib.capy", "/dev/stdin"],
        input=source,
        check=True,                                  # ← rejects malformed
        capture_output=True,
    ).stdout
    deploy(target)
```

Three properties this gives you out of the box:

1. **Cost**: ~50 output tokens per task no matter how complex the
   target.
2. **Safety**: malformed or out-of-spec output is rejected at parse
   time. The agent can never deploy something that doesn't match the
   library's contract.
3. **Auditability**: you can read `lib.capy` in 5 minutes and know
   exactly what the agent is capable of producing.

---

## When Capy is NOT the right fit

Be honest about the trade-offs:

- **One-off generation** where the target shape changes every time.
  The library design cost won't amortise. Use raw LLM codegen.
- **When you actually need arbitrary code.** If you want the agent to
  invent novel algorithms, the constrained grammar is in your way.
  Use raw codegen with sandboxed execution.
- **When the library would be huge.** If your target language has
  100+ distinct shapes the agent might need, expressing them all in
  Capy is a chore. Use a real parser generator.

The sweet spot is **constrained, repetitive, boilerplate-heavy
output** — CRUD apps, infrastructure manifests, schemas, config
files, dashboards, components, scaffolding. Most agent work falls
here.

---

## Next steps

- Read [CAPY_FOR_LLMS.md](CAPY_FOR_LLMS.md) — the single-page brief
  you can paste into any model's context.
- Browse [the 50 sample demos](https://github.com/olivierdevelops/capy/tree/main/samples)
  — each is a tiny grammar producing a complete target file.
- Install the [Claude Code skill](https://github.com/olivierdevelops/capy/tree/main/skills/capy-author)
  if you use Claude Code.
- See [getting started](getting-started.md) and
  [library authoring](library-authoring.md) to design your own library.
