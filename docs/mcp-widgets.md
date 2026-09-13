---
title: Capy for MCP Apps / AI widgets
---

# Capy for MCP Apps / AI widgets

MCP Apps ([modelcontextprotocol.io/extensions/apps](https://modelcontextprotocol.io/extensions/apps/overview))
and the broader widget ecosystem
([mcp-ui](https://mcpui.dev/), [mcp-widgets/examples](https://github.com/mcp-widgets/examples))
all share one shape: an MCP server returns an interactive
HTML/CSS/JS payload, the host renders it in a sandboxed iframe.
Capy is purpose-built for the "compile a small grammar into
deterministic HTML" half of that story.

## The problem with hand-authored widget HTML

In the current pattern, **the server author writes the widget
HTML by hand** — every weather card, every product list, every
chart. Three downsides compound at scale:

| Today | Cost |
|---|---|
| Each widget is ~50–500 lines of HTML+CSS+JS | High maintenance, easy drift between widgets |
| LLM-generated widget payloads are large | Bandwidth + tokens per response |
| Server-side rendering pipeline custom to each widget | Hard to add new widget kinds; harder to keep them consistent |

## What Capy adds

Capy turns the widget surface into **one library, many one-line
calls**. The MCP server ships a compiled library (native binary or
WASM) once. The LLM emits 3–20 lines of Capy DSL per response. The
library turns that DSL into the complete sandbox-iframe-ready HTML.

```
weather "San Francisco"
    temp 68 unit "F"
    condition "Partly cloudy"
    forecast
        day "Mon" high 70 low 55
        day "Tue" high 68 low 53
        day "Wed" high 72 low 56
    end
end
```

That entire LLM output (10 lines) renders as a complete weather
widget in the host's iframe. The library handles colors,
typography, responsive layout, and any JS interactivity.

**Live preview from [`samples/mcp-widgets/`](https://github.com/olivierdevelops/capy/tree/main/samples/mcp-widgets):**

<iframe src="../assets/demos/mcp-widgets.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; height: 580px; border: 0; border-radius: 12px; box-shadow: 0 12px 40px rgba(0,0,0,0.18); display: block; margin: 18px 0 24px;" title="MCP widgets rendered live from a Capy source"></iframe>

The exact same `lib.capy` + DSL above produced the iframe content.
Three widget shapes — weather card, bar chart with interactive
hover states, product card — driven by a single Capy library.

---

## Architecture options

### Option A — Server-side rendering (simplest)

Server holds a native `capy build` binary that has the library
embedded. When the LLM tool invocation comes in, the server runs
the binary on the LLM's DSL output, captures the HTML, and returns
it in the MCP response.

```
LLM → DSL  →  Server (capy-built binary) → HTML  →  Host iframe
        ↑                                                ↑
        50 tokens                                  Fully rendered widget
```

- **Pros**: zero client-side dependencies, library updates ship with
  the server, plays nicely with existing MCP transports.
- **Cons**: server has to render every payload; no static CDN
  caching of the engine.

Set up:

```sh
capy build widgets -o ./widgets-renderer        # one-time
# server pseudocode:
output := exec("./widgets-renderer", "render", llmDslPayload)
return mcpResponse{HTML: output}
```

### Option B — Client-side rendering (WASM-shipped library)

Server compiles the library to WASM once and serves it as a static
asset alongside a tiny JS wrapper. The MCP response carries just
the Capy DSL; the iframe loads the WASM and renders client-side.

```
LLM → DSL → Server (passthrough)  → Host iframe → WASM renders DSL → DOM
                                          ↑
                                Engine downloaded once, cached
```

- **Pros**: cuts the server out of the render loop, deterministic
  caching (the WASM blob is a stable artifact), the host can preload
  the WASM the moment the tool description references it (MCP's
  `_meta.ui.resourceUri` preloading pattern).
- **Cons**: needs a small JS wrapper in the host iframe; first call
  pays a one-time WASM load.

The compile step is one line — Capy's `capy-wasm-abi` crate is the
engine; pair it with your library:

```sh
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path rust/Cargo.toml -p capy-wasm-abi
cp rust/target/wasm32-unknown-unknown/release/capy_wasm_abi.wasm widgets.wasm
```

The iframe wrapper then does:

```js
import './wasm_exec.js';
const go = new Go();
const { instance } = await WebAssembly.instantiateStreaming(
  fetch('/widgets.wasm'), go.importObject);
go.run(instance);
// globalThis.capyRun is now defined:
const { ok, output } = capyRun(WIDGET_LIBRARY_SOURCE, llmDsl);
document.body.innerHTML = output;
```

There is no fully-baked variant that works in a browser: `capy build
--target wasm32-…` stages the embedded library through a temp file, which wasm
has no filesystem for. Ship the engine module plus your library source as a JS
string constant — the source is a few KB, and the iframe still just feeds DSL
and gets HTML back.

---

## Why this is a good fit for AI agents

The same properties that make Capy good for general agent codegen
(documented at [Capy for AI agents](ai-agents.md)) apply with extra
weight for widgets:

- **5–10× fewer tokens per widget.** The LLM emits 10–50 tokens
  of DSL instead of 500–1500 tokens of HTML+CSS+JS. Multiply by
  every tool call.
- **Sandboxed by construction.** The library is the LLM's full
  vocabulary. Out-of-grammar emissions are rejected by the parser
  before any renderer runs. The LLM literally cannot inject
  arbitrary HTML or JS — even if prompted to.
- **Versionable, signable artifacts.** The library is a
  human-auditable `.capy` file (or a deterministic WASM blob with
  a sha256). A widget rendered today by `widgets-v1.4.2` looks
  identical tomorrow.
- **One library, many hosts.** The same library renders to a
  desktop iframe, a mobile WebView, a terminal-side TUI (with a
  different `lib.capy` for that target), or a static PDF report.
  The LLM-facing DSL doesn't change.
- **Cheap to evolve.** Adding a new widget shape (`map_view`,
  `data_table`) is a 10–30 line library edit, not a full
  HTML+CSS+JS authoring + accessibility + responsive-layout pass
  every time.

---

## Suggested LLM-facing DSL design

Three principles that match how LLMs actually emit content:

1. **One opener per widget, declarative attributes inside.** Mirrors
   the function-call mental model that's everywhere in LLM
   training data:

       weather "Tokyo"
           temp 17 unit "C"
           condition "Light rain"
       end

2. **No deeply nested structure.** Two or three levels at most.
   LLMs are more reliable on flat structures; deep nesting is where
   they hallucinate closing tags.

3. **Type-constrained values.** Use Capy's
   [group types](types.md#group-types) and
   [`pattern`/`options`](types.md#library-defined-types) to lock
   what the LLM can emit:

       type WeatherCondition
           options "Sunny" "Cloudy" "Partly cloudy" "Light rain" "Heavy rain" "Snow" "Fog"
       end

       function condition
           arg literal "condition"
           arg capture c WeatherCondition
           ...
       end

   The LLM cannot emit a free-form `condition "weird thing"` —
   only one of the seven allowed strings, or the parser rejects it
   at the boundary.

---

## A complete worked example

Library: [`samples/mcp-widgets/lib.capy`](https://github.com/olivierdevelops/capy/tree/main/samples/mcp-widgets/lib.capy)
(≈140 lines including the page template and styles).

LLM-side DSL (this is everything the LLM emits):

```
weather "San Francisco"
    temp 68 unit "F"
    condition "Partly cloudy"
    forecast
        day "Mon" high 70 low 55
        day "Tue" high 68 low 53
        day "Wed" high 72 low 56
    end
end

chart "Sales by quarter"
    bar "Q1" 120
    bar "Q2" 185
    bar "Q3" 142
    bar "Q4" 230
end

product_card "Wireless Headphones" price "$129" rating "4.6"
    summary "Active noise cancelling, 30-hour battery, USB-C."
end
```

20 lines in → ~200 lines of clean, sandbox-safe interactive HTML
out. The bar chart even includes a small inline JS for hover
animations, all emitted by the library.

Render locally to inspect the output:

```sh
git clone https://github.com/olivierdevelops/capy
cd capy/samples/mcp-widgets
capy run lib.capy script.capy        # writes widget.html
open widget.html
```

---

## See also

- [Capy for AI agents](ai-agents.md) — the broader agent-codegen
  story (the patterns that make widget DSLs work apply to other
  agent-emitted content too).
- [Compile cookbook](cookbook-compile.md) — how to ship the
  library as a native binary, WASM blob, or Docker image.
- [Group types](types.md#group-types) — the primitive that makes
  delimited inline DSL syntax (`[link](url)`, `**bold**`) feel
  native.
- [`samples/mcp-widgets/`](https://github.com/olivierdevelops/capy/tree/main/samples/mcp-widgets) —
  the complete library + script driving the live preview at the
  top of this page.
