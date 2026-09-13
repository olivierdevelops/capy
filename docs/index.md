---
title: Capy
hide:
  - navigation
  - toc
---

<div class="capy-hero" markdown>

<span class="capy-eyebrow">A TRANSPILER ENGINE WITH ZERO DEFAULT GRAMMAR</span>

# Describe what you want. Capy produces what you need.

Capy takes a tiny declaration — say, *"a recipe with these
ingredients"*, *"an API with these endpoints"*, *"an Android app
with these screens"* — and turns it into the **real artifact**:
HTML, Swift, Kotlin, Go + tests, Terraform, libtorch C++, anything
textual.

<iframe src="assets/hero/hero.html" width="100%" height="500" style="border: 0; border-radius: 12px; box-shadow: 0 12px 40px rgba(0,0,0,0.18); display: block; margin: 18px 0 24px;" title="Capy in action"></iframe>

[Open the playground :material-play-box-outline:](playground.md){ .md-button .md-button--primary }
[What's new :material-star-shooting:](whats-new.md){ .md-button }
[See the samples :material-folder-open:](showcase.md){ .md-button }
[Get started in 5 min :material-rocket-launch:](getting-started.md){ .md-button }
[For AI agents :material-robot-outline:](ai-agents.md){ .md-button }

</div>

---

<div class="capy-section capy-ai-banner" markdown>

## :material-robot-outline: Built for AI agents

Instead of asking an LLM to write **the artifact** (HTML, DOCX, PDF,
Terraform, Kubernetes…), have it emit one terse Capy source. A
library — not the agent — deterministically renders it into whichever
target your environment needs.

<div class="capy-ai-grid" markdown>

<div class="capy-ai-cell" markdown>
**5–10× fewer tokens** per generation. The agent ships ~50–100 tokens
of DSL; the library expands it into 800+ tokens of target code.
</div>

<div class="capy-ai-cell" markdown>
**One source → many targets.** Same `meeting.capy` source renders to
HTML, Markdown, LaTeX, DOCX — pick the right library at run time. No
re-prompting the agent for each format.
</div>

<div class="capy-ai-cell" markdown>
**Sandboxed by construction.** The library is the agent's full
vocabulary. Out-of-grammar emissions are rejected by the parser
before any renderer runs. No shell, no filesystem, no
`subprocess.run` exposed to the model.
</div>

<div class="capy-ai-cell" markdown>
**Environment-agnostic agents.** The agent doesn't need to know
about `pandoc`, `pdflatex`, `python-docx`, fonts, or the operator's
OS. It just writes DSL; the host handles the rest.
</div>

</div>

[Read the AI agents guide :material-arrow-right:](ai-agents.md){ .md-button .md-button--primary }
[MCP Apps / AI widgets :material-view-grid:](mcp-widgets.md){ .md-button }
[MCP server setup :material-server:](mcp.md){ .md-button }
[One-page LLM brief :material-file-document:](CAPY_FOR_LLMS.md){ .md-button }

</div>

---

<div class="capy-section" markdown>

## What you'll build

Below are real artifacts produced by short Capy sources, **rendered
live**. Each card embeds the actual generated output.

</div>

<div class="show-grid" markdown>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">MOBILE</div>
<h3>Habit Tracker — phone-ready app</h3>
<p>15 lines declare screens and features. The Android and iOS
libraries scaffold native code; this preview is the parallel web
build for live interaction.</p>
</div>
<div class="canvas" markdown>
<div class="phone-frame">
  <iframe src="assets/demos/habit-tracker.html" sandbox="allow-scripts allow-same-origin" title="Habit Tracker"></iframe>
</div>
</div>
<div class="footer">
<a href="one-source-many-files/#mobile-same-source-two-platforms">Read the mobile pattern →</a>
<code>samples/android-app + ios-app</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">GAMES</div>
<h3>Breakout — interactive HTML5 game</h3>
<p>18 lines of declarative DSL declare entities, key bindings, and
event handlers. 226 lines of working JS generated.</p>
</div>
<div class="canvas" markdown>
<iframe src="assets/demos/breakout.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; max-width: 480px; height: 320px; border: 1px solid #333; border-radius: 6px; background: #0a0a14;" title="Breakout"></iframe>
</div>
<div class="footer">
<a href="showcase/">More playable demos →</a>
<code>samples/interactive-breakout</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">MCP / AI WIDGETS</div>
<h3>Interactive widgets from one DSL line</h3>
<p>MCP Apps (Claude Desktop, mcp-ui, mcp-widgets) render
sandbox-iframe HTML returned by tool calls. With Capy, the
LLM emits <code>weather "Tokyo" temp 17 unit "C" … end</code>
(20 tokens); a compiled library renders the complete weather
card, chart, or product widget. 5–10× fewer tokens per
response, sandboxed by parser-as-grammar.</p>
</div>
<div class="canvas" markdown>
<iframe src="assets/demos/mcp-widgets.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; max-width: 480px; height: 320px; border: 1px solid #333; border-radius: 6px; background: #0a0a14;" title="MCP widget"></iframe>
</div>
<div class="footer">
<a href="mcp-widgets/">Widget guide →</a>
<code>samples/mcp-widgets</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">MATH</div>
<h3>Math plots — one DSL line per curve</h3>
<p><code>plot "sin(x)" … end</code> → a self-contained HTML page with
five canvas plots, axes, and an inline plotter. Demonstrates the
new <code>template … end</code> sugar, <code>${escapeHtml}</code>,
and <code>${decoded}</code> composing to drive a JS render at
runtime.</p>
</div>
<div class="canvas" markdown>
<iframe src="assets/demos/math-plots.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; max-width: 480px; height: 320px; border: 1px solid #333; border-radius: 6px; background: #0a0a14;" title="Math plots"></iframe>
</div>
<div class="footer">
<a href="whats-new/">See the new primitives →</a>
<code>samples/math-plots</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">3D</div>
<h3>Three.js scene — interactive 3D</h3>
<p>~25 declarative lines (meshes + abstract motions + `click` / `key` /
`hover` / `button` bindings) → a runnable HTML page that imports
three.js, raycasts clicks, dispatches keyboard shortcuts, and shows
an HTML HUD. Click objects, press <code>space</code> / <code>d</code> / <code>r</code>, drag to orbit.</p>
</div>
<div class="canvas" markdown>
<iframe src="assets/demos/threejs.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; max-width: 480px; height: 320px; border: 1px solid #333; border-radius: 6px; background: #0a0a14;" title="Three.js scene"></iframe>
</div>
<div class="footer">
<a href="playground/">Try it in the playground →</a>
<code>samples/transpile-threejs</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">EVERYDAY</div>
<h3>Recipe card — printable HTML</h3>
<p>A home cook writes 6 keywords (`recipe`, `serves`, `ingredient`,
`step`, `tip`); the library produces a polished printable card.</p>
</div>
<div class="canvas" markdown>
<iframe src="assets/demos/recipe-card.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; max-width: 480px; height: 480px; border: 1px solid #e8d9b0; border-radius: 6px; background: #fdf6e3;" title="Recipe card"></iframe>
</div>
<div class="footer">
<a href="for-everyone/">Capy for non-programmers →</a>
<code>samples/recipe-card</code>
</div>
</div>

<div class="show-card" markdown>
<div class="show-card-head" markdown>
<div class="show-card-eyebrow">BACKEND</div>
<h3>Auto-wired handler + test code</h3>
<p>Every <code>handler</code> declaration emits the Go stub <em>and</em> a
matching smoke test. The team's directory layout is enforced by the
library. <strong><code>go test</code> on the generated code passes.</strong></p>
</div>
<div class="canvas" markdown>
<div class="terminal-frame" style="width: 100%;">
  <div class="chrome">
    <span class="lights" style="display:inline-flex;gap:6px;"><span style="width:10px;height:10px;border-radius:50%;background:#ff5f57;display:inline-block;"></span><span style="width:10px;height:10px;border-radius:50%;background:#ffbd2e;display:inline-block;"></span><span style="width:10px;height:10px;border-radius:50%;background:#28c940;display:inline-block;"></span></span>
    <span class="title">go test</span>
  </div>
<pre style="font-size:11.5px;"><span class="muted">$ cargo test --workspace</span>
<span class="ok">--- PASS: Test_ListUsers_RouteRegistered (0.00s)</span>
<span class="ok">--- PASS: Test_GetUser_RouteRegistered (0.00s)</span>
<span class="ok">--- PASS: Test_CreateUser_RouteRegistered (0.00s)</span>
<span class="ok">--- PASS: Test_DeleteUser_RouteRegistered (0.00s)</span>
PASS
ok    example/handlers   <span class="ok">0.448s</span>
<span class="cursor"></span></pre>
</div>
</div>
<div class="footer">
<a href="backend-codegen/">Backend codegen →</a>
<code>samples/backend-with-tests</code>
</div>
</div>

</div>

---

<div class="capy-section" markdown>

## For your team

Capy isn't one feature; it's a **substrate** that absorbs your team's
conventions and replays them across every project. Pick the role
that matches you:

</div>

<div class="audience-grid" markdown>

<div class="audience-card" markdown>
<div class="role">:material-react: FRONTEND</div>
<h4>Design system across React + Vue + Svelte</h4>
<p>One component declaration compiles to all three frameworks with
**identical Tailwind classes**, identical layout, identical sizing.
Add new components in the library once — every framework regenerates.</p>
<a href="design-systems/">Pattern docs →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-server: BACKEND</div>
<h4>Conventions enforced, tests auto-wired</h4>
<p>Every `handler` line emits the stub **and** the test. Directory
layout, "every handler has a test", router placement — encoded in
the library. New contributors can't violate them. Generated `go test`
passes.</p>
<a href="backend-codegen/">Pattern docs →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-cellphone: MOBILE</div>
<h4>Android + iOS from a single declaration</h4>
<p>One `script.capy` declares the app's screens and features. Two
libraries (`lib_android.capy`, `lib_ios.capy`) emit Kotlin + manifest
+ gradle, or SwiftUI + Info.plist + Package.swift. Drop into Android
Studio / Xcode.</p>
<a href="one-source-many-files/#mobile-same-source-two-platforms">Mobile demo →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-robot-outline: AI / AGENTS</div>
<h4>One DSL source → HTML, PDF, DOCX, anything</h4>
<p>Stop asking the agent to rewrite the same artifact for every
target. It emits one Capy source; per-target libraries render the
final HTML / Markdown / LaTeX / DOCX / config. Sandboxed by the
parser, 5–10× fewer tokens, environment-agnostic.</p>
<a href="ai-agents/">For AI agents →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-cog-outline: DEVOPS</div>
<h4>Configs as a library</h4>
<p>Dockerfile, Kubernetes, Terraform, GitHub Actions, Prometheus,
nginx — all become libraries that absorb your house style. The
output is plain target syntax; your runtime doesn't know Capy ran.</p>
<a href="extending-existing-syntax/">Pattern docs →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-palette-outline: DESIGN</div>
<h4>One UI source, no drift</h4>
<p>Encode your tokens (button variants, card padding, sizing scale)
in the library; every consumer produces identical visuals. The
designer reads the library once and knows every shape the system
can ever produce.</p>
<a href="design-systems/">Design systems →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-account-multiple: MANAGER</div>
<h4>Stop the rewrite cycle</h4>
<p>Capy is for ideas; libraries are implementations. When Go isn't
fast enough and you want Rust — swap the library, not rewrite the
system. The contract stays stable while the implementation evolves.</p>
<a href="idea-language/">Idea-language thesis →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-account-multiple-outline: NON-PROGRAMMER</div>
<h4>You don't need code background</h4>
<p>The 8 "Everyday" samples (recipe / invitation / meal plan /
reading log / trip itinerary / resume / invoice / to-do) use
plain-English keywords. Edit a few lines, get a polished printable
artifact.</p>
<a href="for-everyone/">Start here →</a>
</div>

<div class="audience-card" markdown>
<div class="role">:material-package-variant: SHIPPING</div>
<h4>One library → native binaries + WASM</h4>
<p>`capy build` turns any library into a self-contained CLI. Cross-
compile for Linux / Windows / ARM / WebAssembly from one machine.
Ship as a release tarball, Homebrew tap, Docker image, or
`npm`-installable WASM module.</p>
<a href="cookbook-compile/">Compile cookbook →</a>
</div>

</div>

---

<div class="capy-section" markdown>

## What enterprises ask for, and what Capy gives them

</div>

| Enterprise concern | How Capy addresses it |
|---|---|
| **"Our design system drifts across stacks"** | One library encodes the tokens. Frontend, Vue, mobile all consume the same declarations. Drift becomes physically impossible. |
| **"Every new dev rediscovers our conventions"** | Conventions live in the library, not in tribal knowledge. Scaffolding a new module always lays files in the right places. |
| **"Rewrites kill velocity"** | Capy is for ideas; libraries are implementations. Swap Go to Rust by adding a Rust library; the source never changes. |
| **"We have an API spec, but it drifts from the code"** | The Capy source IS the spec. OpenAPI, TypeScript clients, Markdown docs all generated from one source. CI diffs detect drift instantly. |
| **"Our AI tools hallucinate target-language code"** | Capy gives the agent a finite, library-defined vocabulary. The agent emits short DSL; Capy produces guaranteed-shape output. |
| **"Spec-first is impossible; backend always lags"** | The DSL parses today → frontend mocks against it today. Backend ships a library against the same source later. The contract is stable from day one. |
| **"Multi-platform is exponentially expensive"** | One source → Android + iOS + web from three libraries. New platform = new library, not new project. |
| **"Conventions in PR-review comments don't scale"** | A library is enforced by the tool, not by the reviewer. Whole categories of "you forgot the test / wrong package / missing audit field" disappear. |
| **"Power users want to extend the DSL without forking"** | `define NAME ... end` in the source declares new functions [inline → see metaprogramming](metaprogramming.md). Library stays untouched. |
| **"High-level tools trap you when you need more control"** | Libraries can expose [progressive abstraction](progressive-abstraction.md) — one-shot, block-style, AND escape hatches in the same library. Drop a level, never switch tools. |
| **"Library docs go stale the moment they're written"** | `capy docs lib.capy` regenerates [reference Markdown](library-documentation.md) from `description` annotations in the same library file. Commit it; CI catches drift. |
| **"Same config, twelve environments"** | Libraries can pull env vars, CLI args, and sibling files at transpile time via `(env ...)`, `(arg N)`, `(read_file ...)` — see [host capabilities](host-capabilities.md). One 5-line source generates per-env Kubernetes/Terraform/.env outputs. Sandboxed by default in the playground. |

---

<div class="capy-section" markdown>

## In one paragraph

Most teams end up writing the same things over and over — config
files, API stubs, schema migrations, design-system primitives,
docs, manifests, tests. **Capy lets you describe *what* you want in
a few plain lines**; a library encodes *how* each target produces
it. Same source → many artifacts, byte-identical every time, all
typed and validated at the boundary. The grammar is expressive
enough to parse **matched-pair HTML and XML** — one generic
`<tag>…</tag>` function, with mismatched nesting caught as a parse
error ([see it in the playground](playground.md)). Use it from the
CLI, embed it as a Rust library, ship it through an MCP server to AI
agents, or [try it right now in your browser](playground.md) — the
compiler runs as WebAssembly.

</div>

---

<div class="capy-section" markdown>

## Install

</div>

```sh
# CLI
cargo install --git https://github.com/olivierdevelops/capy capy-cli

# Embed as a Rust library
cargo add --git https://github.com/olivierdevelops/capy capy-core

# MCP server for AI agents
cargo install --git https://github.com/olivierdevelops/capy capy-mcp

# Or grab a release tarball (no Rust toolchain needed)
curl -fsSL https://raw.githubusercontent.com/olivierdevelops/capy/main/scripts/install.sh | sh
```

Run a sample:

```sh
git clone https://github.com/olivierdevelops/capy
cd capy
capy run samples/recipe-card/lib.capy samples/recipe-card/script.capy > my-recipe.html
open my-recipe.html
```

…or just open the [playground](playground.md) — no install at all.

---

<small>**Capy v0.10** — engine + library schema stable. CLI ships
binaries for linux / darwin / windows × amd64 / arm64. MCP server
ships in every release. Browser playground runs the same engine.
[Full changelog](https://github.com/olivierdevelops/capy/blob/main/CHANGELOG.md).</small>

*Open source under MIT. Contributions at
[github.com/olivierdevelops/capy](https://github.com/olivierdevelops/capy).*
