---
title: AI integration cookbook
---

# AI integration cookbook

Recipes for wiring Capy into AI workflows — MCP servers, agent
skills, embedded Rust agents, prompt-side patterns. Each recipe is
self-contained and copy-pasteable.

## Recipe 1 — Drop-in MCP server for Claude Desktop / Claude Code

**Goal:** any tool-capable AI agent on this machine can call Capy.

```sh
cargo install --git https://github.com/olivierdevelops/capy capy-mcp
```

Add to your MCP config (Claude Desktop, Claude Code, Cursor, Zed all
share this shape):

```json
{
  "mcpServers": {
    "capy": { "command": "capy-mcp" }
  }
}
```

Three tools become available: `capy_check`, `capy_run`, `capy_run_file`.
See [docs/mcp.md](mcp.md) for the full reference.

## Recipe 2 — Sandboxed code generation in an agent loop

**Problem:** an agent generates Python / SQL / HCL / Unity C# in a loop.
Every iteration's output is slightly different even with `temperature: 0`.
Reviewers complain that they can't audit what the agent might emit.

**Capy answer:** generate **Capy DSL** (10× shorter, finite vocabulary)
and run it through a pinned library. Same DSL → same output, every time.

Before:

```
Agent prompt: "Generate a Unity MonoBehaviour that spawns a red cube
at (0,0,0), a blue sphere at (4,0,0), and a camera at (7,7,3)."

→ 200 lines of C# that occasionally hallucinates UnityEngine method names
```

After:

```
Agent prompt: "Generate a Capy scene declaration using these primitives:
cube color x y z size, sphere color x y z radius, camera x y z."

→ 4 lines of DSL:
  cube   red    0 0 0   2
  sphere blue   4 0 0   1
  camera        7 7 3
```

The 4 lines go through `lib_unity.capy` (a library the human audited
once). The output is byte-identical every time and uses only the
Unity API calls the library explicitly enumerates.

```rust
// In your agent loop:
let lib = Library::from_file("lib_unity.capy")?;
for source in agent_emissions {
    let result = lib.run(&source);
    // Ok(out) is guaranteed-shape Unity C#; Err(e) is a precise parse failure.
}
```

## Recipe 3 — Token compression: 200 tokens of DSL beats 2000 of target

**Problem:** the agent is regenerating boilerplate-heavy targets
(Express routes, Prisma schemas, Kubernetes manifests, Terraform).
Every call burns thousands of tokens on the scaffolding.

**Capy answer:** load the library scaffold *once* per session (or bake
it into the system prompt). The agent emits only the *intent*.

Cost math for a typical agent loop emitting 50 K8s manifests:

| Approach          | Tokens per call | 50 calls       |
|-------------------|-----------------|----------------|
| Raw YAML          | ~600            | 30,000         |
| Capy DSL          | ~50             | 2,500 + library cost (~400 once) |
| **Savings**       |                 | **27,000 tokens (~90%)** |

The library itself goes in the system prompt or — better — never
appears in tokens at all because Capy runs in-process via the MCP
tool.

## Recipe 4 — Let the AI build the library, the human use it

**Pattern:** the AI is involved at *build time* (designing the DSL
shape), then disappears at *runtime*. The library is a one-time
asset, not a per-call dependency.

Step 1, with the AI:

```
You: I have 200 product cards in this CSV. I want to generate HTML
for each one with title, price, photo, and CTA. Design a Capy library.

Agent: [writes lib.capy with `card` / `field` / `cta` patterns,
        validates with capy_check, shows you a sample run]

You: Looks good — commit it.
```

Step 2, every weekday morning, no AI:

```sh
./generate.sh   # csv → capy DSL → capy run → 200 HTML files
```

The agent's work is in `lib.capy`. Reproducible, auditable, fast.

## Recipe 5 — A typed config DSL as your agent's safe surface

**Goal:** let users (or downstream agents) declare config with type
validation. Bad input is a clean error, not a runtime surprise.

```capy
# typed_config.capy library snippet
type Email
    pattern "^[^@]+@[^@]+\\.[^@]+$"
end

type LogLevel
    options "debug" "info" "warn" "error"
end

type Port
    base int
end

function service
    arg literal "service"
    arg capture name ident
    block_closer end
    write `service ${name} {
${indent 2 body}
}
`
end

function owner
    arg literal "owner"
    arg capture who Email                 # ← validated
    write `owner = ${who}
`
end

function log_level
    arg literal "log_level"
    arg capture lvl LogLevel              # ← enum
    write `log_level = ${lvl}
`
end

function port
    arg literal "port"
    arg capture n Port                    # ← typed int
    write `port = ${n}
`
end
```

When the agent (or human) writes `log_level verbose`, Capy returns:

```
error: function "log_level" arg "lvl": value "verbose" is not in
       options for type "LogLevel"
```

The agent sees the error, picks `info` instead, retries. The library
*is* the schema; the agent doesn't need a separate spec doc.

[Full sample → `samples/typed-config-dsl/`](https://github.com/olivierdevelops/capy/tree/main/samples/typed-config-dsl)

## Recipe 6 — One DSL, many target tools (game engines, 3D, web)

**Goal:** the agent helps the user design 3D scenes / game mechanics
/ web layouts that need to run in multiple tools (Blender + Unity,
Three.js + Unreal, React + Vue).

**Capy answer:** keep one source-of-truth DSL; ship a library per
target tool. The agent edits the source; you swap libraries to
re-target.

```
samples/3d-tools-demo/
├── script.capy           ← single source describing a scene
├── lib_blender.capy      ← → Blender Python
├── lib_sketchup.capy     ← → SketchUp Ruby
├── lib_rhino.capy        ← → Rhino C# (RhinoCommon)
├── lib_unity.capy        ← → Unity C# MonoBehaviour
└── lib_unreal.capy       ← → Unreal Python
```

When the user says "now make this work in Blender too," the agent
runs `capy_run_file` with `lib_blender.capy` and emits exactly the
right Python — no manual translation.

## Recipe 7 — Bake Capy into your Rust-based AI tool

If you're building an agent in Rust, skip MCP entirely — embed the
engine:

```rust
use capy_core::capy::Library;

const GRAMMAR: &str = r#"
extension html

function card
    arg literal "card"
    arg capture title string
    arg capture price any
    write `<div class="card"><h3>${title}</h3><p>\$${price}</p></div>
`
end
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = Library::new(GRAMMAR)?;

    // Inside your agent loop:
    for source in model_emissions {
        match lib.run(&source) {
            Ok(html) => write_file(&html),
            // Surface the precise parse error back to the model.
            Err(e) => agent.tool_error(&e.to_string()),
        }
    }
    Ok(())
}
```

No subprocess, no MCP framing — just function calls. The library is
a string constant compiled into your binary. See
[docs/embedding.md](embedding.md) for the full guide.

## Recipe 8 — Skills + MCP together (Claude Code)

For Claude Code specifically, you want both:

- The **MCP server** so the agent can call Capy tools.
- A **skill** so Claude knows *when* to reach for those tools.

```sh
# 1. Install the MCP server
cargo install --git https://github.com/olivierdevelops/capy capy-mcp

# 2. Register it with Claude Code
claude mcp add --scope user capy capy-mcp

# 3. Install the skills (clone the repo or download)
mkdir -p ~/.claude/skills
cp -r ~/code/capy/skills/capy-mcp     ~/.claude/skills/
cp -r ~/code/capy/skills/capy-author  ~/.claude/skills/
```

Restart Claude Code. Now when the user asks "build me a DSL for
service configs" or "generate the same data in JSON and SQL," Claude
picks up the right skill and uses the MCP tools.

## Recipe 9 — Self-correcting agent via `capy_check`

When the agent authors a library on the fly, it can validate before
running:

```
1. Agent drafts lib.capy
2. Agent calls capy_check → "valid: false, error: function port arg n
   has unknown type Port"
3. Agent adds the missing type, retries capy_check → "valid: true"
4. Agent calls capy_run with confidence
```

This is the same loop you'd use with a linter or type-checker, but
the language *is the user's*.

## Recipe 10 — Prompt-side: telling the model about Capy

Include this in the system prompt of any agent that should use Capy:

```
You have a `capy` MCP tool that transpiles a tiny DSL into target
code. When the user wants to generate Python, JSON, SQL, YAML, HCL,
C#, HTML, or similar boilerplate-heavy targets:

1. Prefer authoring a 30-50 line `lib.capy` library plus a much
   shorter `script.capy`, then calling `capy_run`. Don't write the
   target output by hand.
2. Use `capy_check` first when authoring a library inline — the
   error messages name the function, arg, and rule that failed.
3. The library's `type:` blocks (`pattern:`, `options:`, `base:`)
   give you free input validation. Reach for them for any field
   with a constrained shape.
4. Same DSL through different libraries → different targets. If
   the user wants the same logic in two languages, write one source
   and two libraries.
```

The shorter version: see [`docs/CAPY_FOR_LLMS.md`](CAPY_FOR_LLMS.md)
for a one-page brief designed to be embedded in any agent's prompt.
