//! MCP Client Walkthrough — Talking to pmat over stdio
//!
//! Demonstrates how an MCP (Model Context Protocol) client drives `pmat`
//! through its stdio transport. Covers the three canonical JSON-RPC flows:
//!
//!   1. `initialize`    — client/server handshake + capability negotiation
//!   2. `tools/list`    — advertise available tools and input schemas
//!   3. `tools/call`    — invoke `analyze_complexity` with `{"paths":["."]}`
//!
//! Run with: `cargo run --example mcp_client_walkthrough`
//!
//! NOTE: Every `tools/call` payload must use the PLURAL field name `paths`.
//! The MCP dispatcher rejects the singular `path` silently when the schema
//! demands an array — the dispatcher never coerces.
//!
//! ## Defect D14 / D32 — Empty Tool Schemas
//!
//! At the time this example was written, `tools/list` returned schemas where
//! `inputSchema.properties` was an empty object for several tools. Clients
//! that validate parameters up-front would emit a "no parameters accepted"
//! error even though the dispatcher accepts `paths`, `format`, `top_files`,
//! and `include`. Treat any empty schema as the D14/D32 gap — fall back to
//! the documented parameter shapes in `docs/mcp-protocol.md` until the
//! regression is fixed.
//!
//! This example is docs-first: if `pmat` is not on PATH we still print the
//! walkthrough so the file is useful as a protocol cheat-sheet.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;

fn main() {
    println!("=== PMAT MCP Client Walkthrough ===\n");
    print_protocol_notes();

    let pmat = option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat");
    println!("Launching: {pmat} mcp\n");

    let mut child = match Command::new(pmat)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            println!("  (pmat not found on PATH: {e})");
            println!("  Install with `cargo install --path .` from the pmat checkout.");
            println!("\nWalkthrough above is the primary documentation payload.");
            return;
        }
    };

    let mut stdin = child.stdin.take().expect("child stdin");
    let stdout = child.stdout.take().expect("child stdout");
    let mut reader = BufReader::new(stdout);

    // ---- Phase 1: initialize handshake -----------------------------------
    println!("--- Phase 1: initialize ---");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "clientInfo": { "name": "pmat-walkthrough", "version": "0.1.0" }
        }
    });
    send(&mut stdin, &init_req);
    let resp = recv(&mut reader);
    summarize_initialize(&resp);

    // MCP requires a "notifications/initialized" after the init response.
    let notif = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    send(&mut stdin, &notif);

    // ---- Phase 2: tools/list ---------------------------------------------
    println!("\n--- Phase 2: tools/list ---");
    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    send(&mut stdin, &list_req);
    let resp = recv(&mut reader);
    summarize_tools_list(&resp);

    // ---- Phase 3: tools/call analyze_complexity --------------------------
    println!("\n--- Phase 3: tools/call analyze_complexity ---");
    println!("  Note: plural `paths` is required; singular `path` is silently rejected.");
    let call_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "analyze_complexity",
            "arguments": {
                "paths": ["."],
                "format": "summary",
                "top_files": 3
            }
        }
    });
    send(&mut stdin, &call_req);
    let resp = recv(&mut reader);
    summarize_tool_call(&resp);

    // ---- Teardown --------------------------------------------------------
    drop(stdin);
    let _ = child.wait_timeout(Duration::from_secs(2));
    let _ = child.kill();
    let _ = child.wait();
    println!("\n=== Walkthrough complete ===");
}

/// One-line JSON-RPC summary you can paste into a client README.
fn print_protocol_notes() {
    println!(
        "MCP stdio wire format (newline-delimited JSON-RPC 2.0):

  client -> server: {{\"jsonrpc\":\"2.0\",\"id\":N,\"method\":\"...\",\"params\":...}}
  server -> client: {{\"jsonrpc\":\"2.0\",\"id\":N,\"result\":...}} OR error

Required order:
  1. initialize (request/response)
  2. notifications/initialized (one-way notification)
  3. tools/list, tools/call, resources/list, ... (any order)
"
    );
}

fn send(stdin: &mut impl Write, payload: &Value) {
    let line = format!("{}\n", payload);
    if stdin.write_all(line.as_bytes()).is_err() {
        println!("  (write to pmat stdin failed)");
    }
    let _ = stdin.flush();
    let preview = truncate(&payload.to_string(), 120);
    println!("  -> {preview}");
}

fn recv(reader: &mut BufReader<impl std::io::Read>) -> Option<Value> {
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.is_empty() {
        println!("  <- (no response)");
        return None;
    }
    let parsed: Option<Value> = serde_json::from_str(line.trim()).ok();
    match &parsed {
        Some(v) => println!("  <- {}", truncate(&v.to_string(), 120)),
        None => println!("  <- (non-JSON line: {})", truncate(line.trim(), 120)),
    }
    parsed
}

fn summarize_initialize(resp: &Option<Value>) {
    let Some(resp) = resp else {
        return;
    };
    let result = resp.get("result");
    let proto = result
        .and_then(|r| r.get("protocolVersion"))
        .and_then(Value::as_str)
        .unwrap_or("?");
    let server_name = result
        .and_then(|r| r.get("serverInfo"))
        .and_then(|s| s.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("?");
    let server_version = result
        .and_then(|r| r.get("serverInfo"))
        .and_then(|s| s.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("?");
    println!("  server: {server_name} v{server_version}, protocol {proto}");
}

fn summarize_tools_list(resp: &Option<Value>) {
    let Some(resp) = resp else {
        return;
    };
    let Some(tools) = resp.pointer("/result/tools").and_then(Value::as_array) else {
        println!("  (no tools array in response)");
        return;
    };
    println!("  {} tools advertised:", tools.len());
    let mut empty_schema_count = 0usize;
    for (i, tool) in tools.iter().take(5).enumerate() {
        let name = tool.get("name").and_then(Value::as_str).unwrap_or("?");
        let props = tool
            .pointer("/inputSchema/properties")
            .and_then(Value::as_object)
            .map(|m| m.len())
            .unwrap_or(0);
        if props == 0 {
            empty_schema_count += 1;
        }
        println!("    [{i}] {name} — {props} declared parameters");
    }
    if tools.len() > 5 {
        println!("    ... ({} more)", tools.len() - 5);
    }
    // Tally across all tools, not just the first 5.
    let total_empty = tools
        .iter()
        .filter(|t| {
            t.pointer("/inputSchema/properties")
                .and_then(Value::as_object)
                .map(|m| m.is_empty())
                .unwrap_or(true)
        })
        .count();
    if total_empty > 0 {
        println!(
            "  D14/D32: {} of {} tools advertise an empty inputSchema.properties.",
            total_empty,
            tools.len()
        );
        println!("    Dispatcher still accepts documented params (paths/format/…).");
    } else {
        let _ = empty_schema_count;
    }
}

fn summarize_tool_call(resp: &Option<Value>) {
    let Some(resp) = resp else {
        return;
    };
    if let Some(err) = resp.get("error") {
        println!("  error: {}", truncate(&err.to_string(), 200));
        return;
    }
    let content = resp.pointer("/result/content").and_then(Value::as_array);
    match content {
        Some(items) if !items.is_empty() => {
            let text = items[0].get("text").and_then(Value::as_str).unwrap_or("");
            println!("  result preview: {}", truncate(text, 240));
        }
        _ => {
            println!(
                "  (no content array — full result: {})",
                truncate(&resp.to_string(), 200)
            );
        }
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut cut = n;
        while !s.is_char_boundary(cut) && cut > 0 {
            cut -= 1;
        }
        format!("{}…", &s[..cut])
    }
}

// Tiny local wait_timeout shim so we don't add a new crate for teardown.
trait WaitTimeout {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<Option<std::process::ExitStatus>>;
}
impl WaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<Option<std::process::ExitStatus>> {
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(Some(status));
            }
            if start.elapsed() >= dur {
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
