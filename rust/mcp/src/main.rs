//! Port of `cmd/capy-mcp/main.go`.
//!
//! A Model Context Protocol server that exposes Capy as a tool to AI agents
//! (Claude Desktop, Claude Code, any MCP client).
//!
//! Speaks JSON-RPC 2.0 over stdio, line-delimited per the MCP stdio transport.
//! Tools advertised:
//!
//!   * `capy_run`      — transpile a script through an inline library (string)
//!   * `capy_run_file` — transpile a script through a library file on disk
//!   * `capy_check`    — validate a library (no script); returns function names
//!
//! Wire it into Claude Desktop via `claude_desktop_config.json`:
//!
//! ```json
//! { "mcpServers": { "capy": { "command": "capy-mcp" } } }
//! ```

use capy_core::capy::Library;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "capy";
const SERVER_VERSION: &str = "0.3.0";

// --- JSON-RPC 2.0 envelopes -------------------------------------------------

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    #[serde(default)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// --- tools ------------------------------------------------------------------

/// Port of `toolDef`. A derived struct so the `name` / `description` /
/// `inputSchema` key order matches Go's struct order; the schema itself stays a
/// `Value` because Go builds it from `map[string]any`, which `json.Marshal`
/// sorts — exactly what `serde_json::Value` does.
#[derive(Serialize)]
struct ToolDef {
    name: &'static str,
    description: &'static str,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// Port of `textContent`.
#[derive(Serialize)]
struct TextContent {
    #[serde(rename = "type")]
    kind: &'static str,
    text: String,
}

/// Port of `toolResult`.
#[derive(Serialize)]
struct ToolResult {
    content: Vec<TextContent>,
    #[serde(rename = "isError", skip_serializing_if = "is_false")]
    is_error: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Port of the `tools` table.
///
/// Inner schema objects are written in ALPHABETICAL key order because Go builds
/// them from `map[string]any`, and `json.Marshal` sorts map keys. The outer
/// `ToolDef` is a struct, so its keys follow declaration order.
fn tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "capy_run",
            description: "Transpile a Capy source script through an inline library and return the generated output. Use this when you have both a library definition and source text in memory — no disk needed.",
            input_schema: json!({
                "properties": {
                    "library": {
                        "description": "Full contents of the library file (.capy syntax).",
                        "type": "string"
                    },
                    "script": {
                        "description": "The source code to transpile (Capy DSL declared by the library).",
                        "type": "string"
                    }
                },
                "required": ["library", "script"],
                "type": "object"
            }),
        },
        ToolDef {
            name: "capy_run_file",
            description: "Transpile a script file through a .capy library file on disk and return the generated output.",
            input_schema: json!({
                "properties": {
                    "library_path": {
                        "description": "Absolute or working-directory-relative path to the .capy library file.",
                        "type": "string"
                    },
                    "script_path": {
                        "description": "Path to the source script (typically .capy).",
                        "type": "string"
                    }
                },
                "required": ["library_path", "script_path"],
                "type": "object"
            }),
        },
        ToolDef {
            name: "capy_check",
            description: "Validate a Capy library (no script run). Returns the declared function names and type names if it parses, or a precise error message if it doesn't. Use before calling capy_run to give the user actionable feedback about library bugs.",
            input_schema: json!({
                "properties": {
                    "library": {
                        "description": "Full contents of the .capy library file.",
                        "type": "string"
                    }
                },
                "required": ["library"],
                "type": "object"
            }),
        },
    ]
}

// --- tool implementations ---------------------------------------------------

fn arg_str(args: &Value, key: &str) -> String {
    args.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// Port of `toolCapyRun`.
fn tool_capy_run(args: &Value) -> Result<String, String> {
    let lib_src = arg_str(args, "library");
    let script_src = arg_str(args, "script");
    if lib_src.is_empty() {
        return Err("library is required".to_string());
    }
    if script_src.is_empty() {
        return Err("script is required".to_string());
    }
    let lib = Library::new(&lib_src).map_err(|e| format!("library: {}", e))?;
    lib.run(&script_src).map_err(|e| format!("transpile: {}", e))
}

/// Port of `toolCapyRunFile`.
fn tool_capy_run_file(args: &Value) -> Result<String, String> {
    let lib_path = arg_str(args, "library_path");
    let script_path = arg_str(args, "script_path");
    if lib_path.is_empty() || script_path.is_empty() {
        return Err("library_path and script_path are required".to_string());
    }
    let lib = Library::from_file(&lib_path).map_err(|e| format!("load {}: {}", lib_path, e))?;
    let script = std::fs::read_to_string(&script_path).map_err(|e| {
        format!(
            "read {}: {}",
            script_path,
            capy_core::gopath::io_error("open", &script_path, &e)
        )
    })?;
    lib.run(&script).map_err(|e| format!("transpile: {}", e))
}

/// Port of `checkResult`.
///
/// A derived struct rather than a `json!` literal, because serde preserves field
/// DECLARATION order while `serde_json::Value` sorts keys — and Go emits these in
/// struct order (`valid`, `functions`, `extension`, `error`). The payload is a
/// string shown to the agent, so key order is part of the observable output.
#[derive(Serialize)]
struct CheckResult {
    valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    functions: Vec<String>,
    #[serde(skip_serializing_if = "str::is_empty")]
    extension: String,
    #[serde(skip_serializing_if = "str::is_empty")]
    error: String,
}

/// Port of `toolCapyCheck`.
///
/// NOTE: Go calls `lib.FunctionNames()`, which its own doc comment claims is
/// sorted but isn't — so the Go server's `functions` array order varies per run.
/// This port is deterministic (alphabetical).
fn tool_capy_check(args: &Value) -> Result<String, String> {
    let lib_src = arg_str(args, "library");
    if lib_src.is_empty() {
        return Err("library is required".to_string());
    }
    let payload = match Library::new(&lib_src) {
        Err(e) => CheckResult {
            valid: false,
            functions: Vec::new(),
            extension: String::new(),
            error: e.to_string(),
        },
        Ok(lib) => CheckResult {
            valid: true,
            functions: lib.function_names(),
            extension: lib.extension().to_string(),
            error: String::new(),
        },
    };
    Ok(serde_json::to_string_pretty(&payload).unwrap_or_default())
}

/// Port of `sniffCapy`.
///
/// Decides whether a source looks like Capy-native syntax rather than YAML: the
/// first non-comment line of a `.capy` library starts with a bareword directive,
/// while YAML almost always leads with `key:` mapping syntax.
pub fn sniff_capy(src: &str) -> bool {
    for line in src.split('\n') {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        return s.starts_with("function ")
            || s.starts_with("type ")
            || s.starts_with("extension ")
            || s.starts_with("output_file ")
            || s.starts_with("file_template:");
    }
    false
}

/// Port of `dispatchTool`.
fn dispatch_tool(name: &str, args: &Value) -> Result<String, String> {
    match name {
        "capy_run" => tool_capy_run(args),
        "capy_run_file" => tool_capy_run_file(args),
        "capy_check" => tool_capy_check(args),
        other => Err(format!("unknown tool {:?}", other)),
    }
}

/// A tool call's textual result.
fn tool_result(text: &str, is_error: bool) -> Value {
    serde_json::to_value(ToolResult {
        content: vec![TextContent { kind: "text", text: text.to_string() }],
        is_error,
    })
    .unwrap_or(Value::Null)
}

// --- dispatcher -------------------------------------------------------------

/// Port of `handle`. Returns `None` for notifications that produce no response.
fn handle(req: &RpcRequest) -> Option<RpcResponse> {
    let mut resp = RpcResponse {
        jsonrpc: "2.0",
        id: req.id.clone(),
        result: None,
        error: None,
    };
    match req.method.as_str() {
        "initialize" => {
            // Go builds this from map[string]any, so keys emit sorted.
            resp.result = Some(json!({
                "capabilities": { "tools": {} },
                "protocolVersion": PROTOCOL_VERSION,
                "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
            }));
        }
        // A notification — no response.
        "notifications/initialized" => return None,
        "tools/list" => {
            resp.result = Some(json!({ "tools": serde_json::to_value(tools()).unwrap() }));
        }
        "tools/call" => {
            let params = req.params.clone().unwrap_or(Value::Null);
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            if name.is_empty() && params.get("name").is_none() {
                resp.error = Some(RpcError {
                    code: -32602,
                    message: "invalid params: missing name".to_string(),
                    data: None,
                });
                return Some(resp);
            }
            match dispatch_tool(name, &args) {
                Err(e) => resp.result = Some(tool_result(&e, true)),
                Ok(text) => resp.result = Some(tool_result(&text, false)),
            }
        }
        "ping" => resp.result = Some(json!({})),
        other => {
            resp.error = Some(RpcError {
                code: -32601,
                message: format!("method not found: {}", other),
                data: None,
            });
        }
    }
    Some(resp)
}

// --- stdio loop -------------------------------------------------------------

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    // Some clients send long single-line JSON; `read_line` grows as needed, so the
    // 64 KiB-default concern that made the Go version bump its scanner buffer
    // doesn't apply here.
    for line in stdin.lock().lines() {
        let line = match line {
            Err(e) => {
                eprintln!("capy-mcp: stdin error: {}", e);
                std::process::exit(1);
            }
            Ok(l) => l,
        };
        if line.trim().is_empty() {
            continue;
        }
        let req: RpcRequest = match serde_json::from_str(&line) {
            Err(e) => {
                eprintln!("capy-mcp: parse error: {}", e);
                continue;
            }
            Ok(r) => r,
        };
        // Notifications (no id) produce no output.
        let has_id = req.id.is_some() && !req.id.as_ref().unwrap().is_null();
        let resp = match handle(&req) {
            None => continue,
            Some(r) => r,
        };
        if !has_id {
            continue;
        }
        // Go sets SetEscapeHTML(false); serde_json does not HTML-escape either.
        match serde_json::to_string(&resp) {
            Err(e) => {
                eprintln!("capy-mcp: encode error: {}", e);
                continue;
            }
            Ok(s) => {
                let _ = writeln!(out, "{}", s);
                let _ = out.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Port of `TestSniffCapy`.
    #[test]
    fn sniff_capy_distinguishes_native_from_yaml() {
        assert!(sniff_capy("extension html\n"));
        assert!(sniff_capy("# comment\n\nfunction x\n"));
        assert!(sniff_capy("type Port\n"));
        assert!(sniff_capy("output_file \"a\"\n"));
        assert!(sniff_capy("file_template:\n"));
        // YAML-style leads with `key:` mapping syntax.
        assert!(!sniff_capy("extension: html\n"));
        assert!(!sniff_capy("functions:\n  say:\n"));
        assert!(!sniff_capy(""));
    }
}
