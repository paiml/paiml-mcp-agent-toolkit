//! MCP Agent Context Demo — Drive the 4 `pmat_*` tools over stdio JSON-RPC.
//!
//! Spawns `PMAT_PMCP_MCP=1 pmat` as a subprocess and exercises the 4
//! AgentContextTools added in KAIZEN-0165 (PR #349):
//!   - `pmat_query_code`    — semantic code search
//!   - `pmat_get_function`  — full function metadata by id
//!   - `pmat_find_similar`  — refactor-friendly similarity search
//!   - `pmat_index_stats`   — index health + manifest
//!
//! This is the MCP-stdio counterpart to `agent_context_query_demo.rs`
//! (which drives the `pmat query` CLI, not the MCP server).
//!
//! # Requirements
//! - `pmat` binary on PATH (or `target/{release,debug}/pmat`).
//! - A prebuilt `.pmat/context.idx` in the cwd OR a cold build (slow).
//!   First-call index build can take 60–90s on large projects; the
//!   10s warmup sleep only covers server boot, not index construction.
//!
//! # Run
//! ```bash
//! cargo run --example mcp_agent_context_demo
//! ```
//!
//! Reference: PR #349, KAIZEN-0165, issue #337.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PMAT MCP Agent Context Demo (stdio JSON-RPC) ===\n");

    let pmat = find_pmat_binary()?;
    println!("Using pmat: {}", pmat);

    // Spawn pmat in MCP mode over stdio.
    let mut child = Command::new(&pmat)
        .env("PMAT_PMCP_MCP", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().ok_or("no child stdin")?;
    let stdout = child.stdout.take().ok_or("no child stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut writer = stdin;

    // Give the server a moment to boot. First `tools/call` still triggers
    // cold index build if `.pmat/context.idx` is missing.
    thread::sleep(Duration::from_secs(10));

    // 1. initialize handshake
    let resp = rpc(&mut writer, &mut reader, 1, "initialize", json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": { "name": "mcp_agent_context_demo", "version": "1.0" }
    }))?;
    let server_info = resp.pointer("/result/serverInfo/name").cloned().unwrap_or(json!("?"));
    println!("[initialize] server={}", server_info);

    // 2. tools/list — sanity check that the 4 pmat_* tools are registered
    let resp = rpc(&mut writer, &mut reader, 2, "tools/list", json!({}))?;
    let tools = resp.pointer("/result/tools").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let pmat_tools: Vec<&str> = tools.iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .filter(|n| n.starts_with("pmat_"))
        .collect();
    println!("[tools/list] total={} pmat_*={} ({:?})", tools.len(), pmat_tools.len(), pmat_tools);

    // 3. pmat_index_stats — pick up a real function_id we can reuse below
    let resp = call_tool(&mut writer, &mut reader, 3, "pmat_index_stats",
        json!({ "rebuild": false }))?;
    let index_result = extract_tool_result(&resp);
    let fn_count = index_result.pointer("/manifest/function_count").cloned().unwrap_or(json!(0));
    let file_count = index_result.pointer("/manifest/file_count").cloned().unwrap_or(json!(0));
    println!("[pmat_index_stats] ok: functions={} files={}", fn_count, file_count);

    // 4. pmat_query_code — realistic semantic query
    let resp = call_tool(&mut writer, &mut reader, 4, "pmat_query_code",
        json!({ "query": "error handling", "limit": 3 }))?;
    let query_result = extract_tool_result(&resp);
    let result_count = query_result.get("results")
        .or_else(|| query_result.get("functions"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let first_id = query_result.pointer("/results/0/id")
        .or_else(|| query_result.pointer("/functions/0/id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    println!("[pmat_query_code] ok: {} results; first_id={:?}", result_count, first_id);

    // 5. pmat_get_function — use the first id from query_code, or fall back to "main"
    let function_id = first_id.clone().unwrap_or_else(|| "src/bin/pmat.rs::main".to_string());
    let resp = call_tool(&mut writer, &mut reader, 5, "pmat_get_function",
        json!({ "function_id": function_id, "include_source": false }))?;
    match resp.get("error") {
        Some(err) => println!("[pmat_get_function] graceful miss for {:?}: {}", function_id,
            err.get("message").and_then(|m| m.as_str()).unwrap_or("?")),
        None => {
            let gf = extract_tool_result(&resp);
            println!("[pmat_get_function] ok: name={} grade={}",
                gf.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
                gf.pointer("/quality/grade").and_then(|v| v.as_str()).unwrap_or("?"));
        }
    }

    // 6. pmat_find_similar — only meaningful if we got a real id from the query
    if let Some(id) = first_id {
        let resp = call_tool(&mut writer, &mut reader, 6, "pmat_find_similar",
            json!({ "function_id": id, "limit": 3, "min_similarity": 0.3 }))?;
        match resp.get("error") {
            Some(err) => println!("[pmat_find_similar] error: {}",
                err.get("message").and_then(|m| m.as_str()).unwrap_or("?")),
            None => {
                let sim = extract_tool_result(&resp);
                let total = sim.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("[pmat_find_similar] ok: {} similar functions", total);
            }
        }
    } else {
        println!("[pmat_find_similar] skipped: no function_id available from pmat_query_code");
    }

    // Cleanly shut down the server subprocess.
    drop(writer);
    let _ = child.kill();
    let _ = child.wait();
    println!("\nDone.");
    Ok(())
}

// ---- helpers ----

/// Send one JSON-RPC request, parse one response line. Newline-delimited.
fn rpc(
    writer: &mut ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    id: i32,
    method: &str,
    params: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let req = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
    writeln!(writer, "{}", req)?;
    writer.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.trim().is_empty() {
        return Err(format!("empty response for {}", method).into());
    }
    Ok(serde_json::from_str(&line)?)
}

fn call_tool(
    writer: &mut ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    id: i32,
    name: &str,
    args: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    rpc(writer, reader, id, "tools/call", json!({ "name": name, "arguments": args }))
}

/// MCP `tools/call` wraps results in `result.content[0].text` (JSON string) or
/// `result.structuredContent` depending on server impl. This extracts either.
fn extract_tool_result(resp: &Value) -> Value {
    if let Some(sc) = resp.pointer("/result/structuredContent") {
        return sc.clone();
    }
    if let Some(text) = resp.pointer("/result/content/0/text").and_then(|v| v.as_str()) {
        if let Ok(parsed) = serde_json::from_str::<Value>(text) {
            return parsed;
        }
    }
    resp.pointer("/result").cloned().unwrap_or(Value::Null)
}

fn find_pmat_binary() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(output) = Command::new("pmat").arg("--version").output() {
        if output.status.success() {
            return Ok("pmat".to_string());
        }
    }
    for candidate in ["target/release/pmat", "target/debug/pmat"] {
        if Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }
    Err("pmat binary not found. Install via 'cargo install --path .' or 'cargo build'.".into())
}
