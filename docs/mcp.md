---
title: MCP server
---

# Capy via MCP

Capy ships a Model Context Protocol server (`capy-mcp`) that lets any
MCP-aware AI agent — Claude Desktop, Claude Code, Cursor, Zed, your
own client — call Capy as a tool.

```
                              ┌───────────────────────────┐
   ┌──────────────────┐       │ capy-mcp (stdio MCP)      │
   │  AI agent        │──────▶│                           │
   │  (Claude / etc.) │       │   tools/list              │
   │                  │◀──────│   capy_check              │
   └──────────────────┘       │   capy_run                │
                              │   capy_run_file           │
                              └───────────────────────────┘
```

## What the agent gets

Three tools, advertised via the standard `tools/list` MCP method:

| Tool             | Inputs                                  | Use for                                              |
|------------------|-----------------------------------------|------------------------------------------------------|
| `capy_check`     | `library` (string)                      | Validate a library and discover its functions/types. |
| `capy_run`       | `library`, `script`                     | One-shot generation from inline strings.             |
| `capy_run_file`  | `library_path`, `script_path`           | Run files from the user's workspace.                 |

Libraries are `.capy` files. Tool calls return either the transpiled
text (success) or the parser/type error message (`isError: true`).

## Install

```sh
# Via Cargo (the simplest)
cargo install --git https://github.com/olivierdevelops/capy capy-mcp

# Or extract from a release tarball
curl -sL https://github.com/olivierdevelops/capy/releases/download/v0.3.0/capy_0.3.0_darwin_arm64.tar.gz \
  | tar -xz capy-mcp
sudo mv capy-mcp /usr/local/bin/
```

Verify:

```sh
which capy-mcp
# /Users/you/go/bin/capy-mcp   (or /usr/local/bin/capy-mcp)
```

## Wire it up

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "capy": {
      "command": "capy-mcp"
    }
  }
}
```

Restart Claude Desktop. You should see `capy` in the connected-servers
list and the three tools (`capy_check`, `capy_run`, `capy_run_file`)
should be invokable.

### Claude Code

```sh
claude mcp add capy capy-mcp
```

That writes a project-local entry to `.claude/settings.json`. Or do it
globally:

```sh
claude mcp add --scope user capy capy-mcp
```

### Cursor / Zed / other MCP clients

Any client that supports MCP stdio servers takes the same `command: "capy-mcp"`
form. Refer to your client's docs for the exact config-file location.

### Custom Rust agent

```rust
use capy_core::capy::Library;

let lib = Library::new(lib_src)?;
let out = lib.run(script_src)?;
```

Sometimes you don't need MCP at all — embed the engine directly. See
[embedding.md](embedding.md).

## Verify the connection

After wiring up, ask the agent something like:

> Use the capy MCP server to validate this library and tell me what
> functions and types it declares:
>
> ```
> extension html
>
> function button
>     arg literal "button"
>     arg capture label any
>     write `<button>${label}</button>
> `
> end
> ```

A connected agent will call `capy_check` and report `{ "valid": true,
"functions": ["button"], "extension": "html" }`.

## Talking to the server by hand

The server speaks JSON-RPC 2.0 over stdio, newline-delimited per the
MCP stdio transport. You can drive it yourself:

```sh
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | capy-mcp
```

```json
{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"capy","version":"0.3.0"}}}
```

This is the same protocol every MCP client uses. Useful when debugging
why a client isn't seeing your tools.

## Why Capy + MCP is a strong fit

- **Sandboxing.** Capy can't execute arbitrary code. The library
  defines a finite vocabulary; the agent can't escape it. When you
  give an agent the `capy_run` tool, you've also given it the schema
  of what it can produce — no more, no less.
- **Token compression.** Instead of asking the model to emit 200 lines
  of Unity C#, ask it to emit 8 lines of Capy DSL. The library —
  loaded *once* per session — does the rest. Across many calls in a
  loop this saves 5–10× tokens.
- **Reproducibility.** Same library + same source = byte-identical
  output. No "regenerate the same thing slightly differently" drift.
- **Reviewable.** A human reads the `lib.capy` once and
  knows every shape of output the agent can ever produce.

## Skill files

If your agent uses Claude Code skills:

- [`skills/capy-mcp/SKILL.md`](https://github.com/olivierdevelops/capy/tree/main/skills/capy-mcp/SKILL.md) —
  when to reach for Capy via MCP and how to operate the tools.
- [`skills/capy-author/SKILL.md`](https://github.com/olivierdevelops/capy/tree/main/skills/capy-author/SKILL.md) —
  when designing or extending a library (with or without MCP).

Drop those directories into `~/.claude/skills/` (user-scope) or your
project's `.claude/skills/`. Claude Code picks them up automatically.

## Troubleshooting

**Agent says "no tools available."** The MCP client never spawned
`capy-mcp`. Confirm `which capy-mcp` resolves; restart the client;
check the client's MCP-server log.

**Tool calls hang.** Some clients buffer stdio with a tighter timeout
than expected. Try running `capy-mcp` directly with a JSON test message
to confirm the binary is healthy.

**Library "doesn't parse."** Call `capy_check` first. The error
message names the function, arg, and rule that failed.

**Output has extra quotes around captured strings.** Strings captured
with type `string` keep their quotes (Capy is a transpiler — input
syntax is preserved into output). Switch to `arg capture x any` if
you want the bare value.
