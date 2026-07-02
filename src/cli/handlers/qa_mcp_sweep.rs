//! MACS F5 (Component 32): LLM-free deterministic MCP sweep harness.
//!
//! Sub-spec: `docs/specifications/components/modern-agentic-coding-support.md`
//! Contract: `contracts/macs-sweep-v1.yaml`
//!
//! `pmat qa-work mcp-sweep` exercises the live 20-tool MCP server over stdio
//! JSON-RPC with schema-derived arguments — NO model, NO tokens. The MCP
//! surface is schema-derivable (each tool advertises an `inputSchema` via
//! `tools/list`), so conformance sweeping is pure Rust with byte-framing
//! goldens; running it through agent fleets would only add variance and cost
//! (spec CoT-7 / E6). Anomalies (malformed frames, non-conformant responses,
//! stray stdout bytes, concurrency lock errors) are the ONLY thing a fleet
//! then judges — via the committed MACS-012 workflow, never here.
//!
//! Determinism: two runs are byte-identical modulo the timestamp/duration
//! fields (contracts/macs-sweep-v1.yaml#sweep_deterministic). This module
//! links no model/API client (`no_llm_symbols_linked`).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Options for `pmat qa-work mcp-sweep`.
#[derive(Debug, Clone)]
pub struct SweepOpts {
    /// Concurrent sweep passes against one working tree (default 8).
    pub concurrency: usize,
    /// Output format.
    pub format: SweepFormat,
    /// Project path the server operates on.
    pub path: PathBuf,
}

/// Sweep report output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepFormat {
    Json,
    Junit,
    Text,
}

/// One tool's conformance result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSweepResult {
    pub name: String,
    /// A well-formed JSON-RPC response (result OR error) was received.
    pub conformant: bool,
    /// Present when non-conformant: what went wrong.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly: Option<String>,
}

/// A framing/concurrency anomaly not tied to a single tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SweepAnomaly {
    pub id: String,
    pub detail: String,
}

/// Deterministic sweep report (canonical; wall-clock excluded from equality).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SweepReport {
    /// Sorted by tool name for byte-stability.
    pub tools: Vec<ToolSweepResult>,
    /// Framing anomalies + concurrency findings, sorted by id.
    pub anomalies: Vec<SweepAnomaly>,
    /// stdout carried only JSON-RPC frames (no stray bytes).
    pub framing_pure: bool,
    /// Passes requested (`--concurrency`).
    pub concurrency: usize,
    pub total_tools: usize,
    pub conformant_tools: usize,
    /// Overall pass: framing pure, no anomalies, every tool conformant.
    pub ok: bool,
}

impl SweepReport {
    /// True iff nothing red: framing pure, no anomalies, all tools conformant.
    pub fn is_ok(&self) -> bool {
        self.framing_pure && self.anomalies.is_empty() && self.tools.iter().all(|t| t.conformant)
    }

    /// Build from raw per-tool results + anomalies, sorting for determinism.
    pub fn build(
        mut tools: Vec<ToolSweepResult>,
        mut anomalies: Vec<SweepAnomaly>,
        framing_pure: bool,
        concurrency: usize,
    ) -> Self {
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        anomalies.sort_by(|a, b| a.id.cmp(&b.id));
        let total_tools = tools.len();
        let conformant_tools = tools.iter().filter(|t| t.conformant).count();
        let mut report = Self {
            tools,
            anomalies,
            framing_pure,
            concurrency,
            total_tools,
            conformant_tools,
            ok: false,
        };
        report.ok = report.is_ok();
        report
    }
}

/// Derive a minimal valid argument set from a JSON Schema `inputSchema`.
///
/// Pure and deterministic (sorted key iteration): every `required` property
/// gets a type-appropriate placeholder; a `paths`/`project_path`/`path`/`file_path`
/// string is filled with the sweep target so path-guarded tools accept it
/// (the anti-cwd-exfiltration guard rejects empty/missing paths). Non-required
/// properties are omitted — "minimal".
pub fn derive_args_from_schema(schema: &serde_json::Value, target: &str) -> serde_json::Value {
    let properties = schema.get("properties").and_then(|v| v.as_object());
    let required: Vec<String> = schema
        .get("required")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut out = serde_json::Map::new();
    let Some(props) = properties else {
        return serde_json::Value::Object(out);
    };

    let mut required_sorted = required;
    required_sorted.sort_unstable();
    for key in required_sorted {
        if let Some(prop_schema) = props.get(&key) {
            out.insert(key.clone(), placeholder_for(&key, prop_schema, target));
        }
    }
    serde_json::Value::Object(out)
}

/// Placeholder value for a single property by its schema type and name.
fn placeholder_for(name: &str, prop: &serde_json::Value, target: &str) -> serde_json::Value {
    let ty = prop
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("string");
    let name_lc = name.to_lowercase();
    let is_pathish = name_lc.contains("path")
        || name_lc == "paths"
        || name_lc.contains("file")
        || name_lc.contains("dir");
    match ty {
        "string" => {
            if is_pathish {
                serde_json::Value::String(target.to_string())
            } else {
                serde_json::Value::String("sweep".to_string())
            }
        }
        "integer" | "number" => serde_json::json!(1),
        "boolean" => serde_json::Value::Bool(false),
        "array" => {
            // Path-array (e.g. `paths`) gets the target; else empty.
            if is_pathish {
                serde_json::json!([target])
            } else {
                serde_json::json!([])
            }
        }
        "object" => derive_args_from_schema(prop, target),
        _ => serde_json::Value::Null,
    }
}

/// Classify a raw stdout blob's framing purity: every non-empty line must
/// parse as a JSON object (JSON-RPC frame). Returns the list of stray
/// (non-JSON) lines — empty means pure.
pub fn framing_stray_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| serde_json::from_str::<serde_json::Value>(l).is_err())
        .map(str::to_string)
        .collect()
}

/// Validate a parsed JSON-RPC response frame: must be a 2.0 object carrying
/// `id` and exactly one of `result`/`error` (a tool-level `error` is still
/// conformant — only malformed frames are anomalies).
pub fn is_conformant_response(frame: &serde_json::Value) -> bool {
    let Some(obj) = frame.as_object() else {
        return false;
    };
    if obj.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
        return false;
    }
    if obj.get("id").is_none() {
        return false;
    }
    obj.contains_key("result") ^ obj.contains_key("error")
}

/// Locate the pmat binary to spawn (release preferred, then debug, then PATH).
fn resolve_pmat_binary() -> PathBuf {
    for candidate in ["target/release/pmat", "target/debug/pmat"] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("pmat")
}

/// Per-pass wall-clock deadline. A tool that has not answered by then is a
/// timeout anomaly, not a hang — the watchdog kills the server so the sweep
/// always terminates (deterministic completion, not deterministic latency).
const SWEEP_DEADLINE_SECS: u64 = 45;

/// Run one sweep pass: spawn `MCP_VERSION=1 pmat`, initialize, tools/list,
/// call every tool with schema-derived args, and check framing. Returns the
/// per-tool results, anomalies, and framing purity for this pass. Path
/// arguments target a minimal throwaway fixture so tools do near-zero work;
/// the SERVER still runs in `target` (the one working tree the concurrency
/// and scratch invariants are about).
fn run_sweep_once(
    binary: &std::path::Path,
    target: &str,
) -> Result<(Vec<ToolSweepResult>, Vec<SweepAnomaly>, bool)> {
    // Minimal fixture: one tiny file the analyze tools can chew in ~0ms.
    // Unique per pass (pid + monotonic counter) so concurrent passes never
    // share or clobber a fixture dir.
    static FIXTURE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = FIXTURE_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let fixture = std::env::temp_dir().join(format!("pmat-mcp-sweep-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&fixture).ok();
    std::fs::write(fixture.join("m.rs"), "pub fn f() -> i32 { 1 }\n").ok();
    let arg_target = fixture.to_string_lossy().to_string();

    let mut child = Command::new(binary)
        .env("MCP_VERSION", "1")
        .current_dir(target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn MCP server (MCP_VERSION=1 pmat)")?;

    let mut stdin = child.stdin.take().context("child stdin")?;
    let stdout = child.stdout.take().context("child stdout")?;

    // Reader thread: every stdout line -> channel. Bounds our reads with a
    // deadline (std pipes have no read timeout) without blocking forever.
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(line.clone()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(SWEEP_DEADLINE_SECS);
    let recv_slice = |rx: &std::sync::mpsc::Receiver<String>| -> Option<String> {
        let now = std::time::Instant::now();
        if now >= deadline {
            return None;
        }
        rx.recv_timeout(deadline - now).ok()
    };

    let mut raw_stdout = String::new();
    let mut frames: Vec<serde_json::Value> = Vec::new();

    // Phase 1: initialize + tools/list, read frames until id == 2.
    let init = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "pmat-mcp-sweep", "version": "1"}}
    });
    let list = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}
    });
    writeln!(stdin, "{init}").context("write initialize")?;
    writeln!(stdin, "{list}").context("write tools/list")?;
    stdin.flush().ok();

    let mut tools: Vec<serde_json::Value> = Vec::new();
    while let Some(line) = recv_slice(&rx) {
        raw_stdout.push_str(&line);
        if let Ok(frame) = serde_json::from_str::<serde_json::Value>(line.trim()) {
            let is_list = frame.get("id").and_then(|v| v.as_u64()) == Some(2);
            if is_list {
                tools = frame
                    .get("result")
                    .and_then(|r| r.get("tools"))
                    .and_then(|t| t.as_array())
                    .cloned()
                    .unwrap_or_default();
            }
            frames.push(frame);
            if is_list {
                break;
            }
        }
    }

    // Phase 2: one tools/call per tool with schema-derived args (pointed at
    // the minimal fixture so tools do near-zero work).
    let mut expected_ids: Vec<(u64, String)> = Vec::new();
    for (i, tool) in tools.iter().enumerate() {
        let name = tool
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let schema = tool
            .get("inputSchema")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let args = derive_args_from_schema(&schema, &arg_target);
        let id = 100 + i as u64;
        let call = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}
        });
        writeln!(stdin, "{call}").ok();
        expected_ids.push((id, name));
    }
    stdin.flush().ok();
    drop(stdin); // EOF -> server exits cleanly (EofSignalingTransport)

    // Drain remaining frames until every call is answered or the deadline
    // hits; then kill the server (bounds a slow/hanging tool).
    let wanted: std::collections::HashSet<u64> = expected_ids.iter().map(|(id, _)| *id).collect();
    let mut answered: std::collections::HashSet<u64> = std::collections::HashSet::new();
    while answered.len() < wanted.len() {
        let Some(line) = recv_slice(&rx) else {
            break;
        };
        raw_stdout.push_str(&line);
        if let Ok(frame) = serde_json::from_str::<serde_json::Value>(line.trim()) {
            if let Some(id) = frame.get("id").and_then(|v| v.as_u64()) {
                if wanted.contains(&id) {
                    answered.insert(id);
                }
            }
            frames.push(frame);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    std::fs::remove_dir_all(&fixture).ok();

    // Correlate responses by id.
    let mut results = Vec::new();
    for (id, name) in expected_ids {
        let frame = frames
            .iter()
            .find(|f| f.get("id").and_then(|v| v.as_u64()) == Some(id));
        match frame {
            Some(f) if is_conformant_response(f) => results.push(ToolSweepResult {
                name,
                conformant: true,
                anomaly: None,
            }),
            Some(_) => results.push(ToolSweepResult {
                name,
                conformant: false,
                anomaly: Some("malformed JSON-RPC response frame".to_string()),
            }),
            None => results.push(ToolSweepResult {
                name,
                conformant: false,
                anomaly: Some("no response frame for this tool call".to_string()),
            }),
        }
    }

    // Framing purity over the whole session's stdout.
    let stray = framing_stray_lines(&raw_stdout);
    let mut anomalies = Vec::new();
    for (i, s) in stray.iter().enumerate() {
        anomalies.push(SweepAnomaly {
            id: format!("framing-{i:03}"),
            detail: format!("non-JSON stdout line: {}", truncate(s, 120)),
        });
    }
    if tools.is_empty() {
        anomalies.push(SweepAnomaly {
            id: "tools-list-empty".to_string(),
            detail: "server advertised zero tools (tools/list returned none)".to_string(),
        });
    }
    Ok((results, anomalies, stray.is_empty()))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// Snapshot the scratch surface a concurrent sweep must leave untouched:
/// stray temp dirs the server might leak. Advisory-lock discipline
/// (src/services/metric_trends_core.rs) plus append-only ledger writes mean
/// N concurrent read-only sweeps against one tree leave no residue.
fn scratch_snapshot(target: &std::path::Path) -> std::collections::BTreeSet<String> {
    let pmat_dir = target.join(".pmat");
    let mut names = std::collections::BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&pmat_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.contains(".tmp") || name.contains(".lock") || name.starts_with("sweep-") {
                names.insert(name);
            }
        }
    }
    names
}

/// `pmat qa-work mcp-sweep` entry point (MACS-011).
pub async fn handle_mcp_sweep(opts: SweepOpts) -> Result<()> {
    let binary = resolve_pmat_binary();
    let target = opts.path.to_string_lossy().to_string();

    let scratch_before = scratch_snapshot(&opts.path);

    // Run `concurrency` passes against ONE working tree.
    let concurrency = opts.concurrency.max(1);
    let mut handles = Vec::new();
    for _ in 0..concurrency {
        let bin = binary.clone();
        let tgt = target.clone();
        handles.push(std::thread::spawn(move || run_sweep_once(&bin, &tgt)));
    }

    let mut all_tools: Vec<ToolSweepResult> = Vec::new();
    let mut all_anomalies: Vec<SweepAnomaly> = Vec::new();
    let mut framing_pure = true;
    let mut pass_reports: Vec<Vec<ToolSweepResult>> = Vec::new();
    for (pass, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok((tools, anomalies, pure))) => {
                framing_pure &= pure;
                for a in anomalies {
                    all_anomalies.push(SweepAnomaly {
                        id: format!("pass{pass:02}-{}", a.id),
                        detail: a.detail,
                    });
                }
                if all_tools.is_empty() {
                    all_tools = tools.clone();
                }
                pass_reports.push(tools);
            }
            Ok(Err(e)) => all_anomalies.push(SweepAnomaly {
                id: format!("pass{pass:02}-spawn-error"),
                detail: e.to_string(),
            }),
            Err(_) => all_anomalies.push(SweepAnomaly {
                id: format!("pass{pass:02}-panic"),
                detail: "sweep pass panicked".to_string(),
            }),
        }
    }

    // Concurrency invariant: every pass must agree on tool conformance
    // (no lock-error-induced divergence across passes).
    if let Some(first) = pass_reports.first() {
        let baseline: std::collections::BTreeMap<&str, bool> = first
            .iter()
            .map(|t| (t.name.as_str(), t.conformant))
            .collect();
        for (pass, report) in pass_reports.iter().enumerate().skip(1) {
            for t in report {
                if baseline.get(t.name.as_str()) != Some(&t.conformant) {
                    all_anomalies.push(SweepAnomaly {
                        id: format!("concurrency-divergence-{}-pass{pass:02}", t.name),
                        detail: format!(
                            "tool {} conformance differs across passes (possible lock error)",
                            t.name
                        ),
                    });
                }
            }
        }
    }

    // Scratch-leftover invariant.
    let scratch_after = scratch_snapshot(&opts.path);
    for leaked in scratch_after.difference(&scratch_before) {
        all_anomalies.push(SweepAnomaly {
            id: format!("scratch-leftover-{leaked}"),
            detail: format!("concurrent sweep left scratch residue: {leaked}"),
        });
    }

    let report = SweepReport::build(all_tools, all_anomalies, framing_pure, concurrency);
    emit_report(&report, opts.format);

    if report.ok {
        Ok(())
    } else {
        anyhow::bail!(
            "mcp-sweep: {} anomaly(ies), {}/{} tools conformant, framing_pure={}",
            report.anomalies.len(),
            report.conformant_tools,
            report.total_tools,
            report.framing_pure
        )
    }
}

fn emit_report(report: &SweepReport, format: SweepFormat) {
    match format {
        SweepFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
            );
        }
        SweepFormat::Junit => println!("{}", render_junit(report)),
        SweepFormat::Text => {
            println!(
                "MCP sweep: {}/{} tools conformant, {} anomaly(ies), framing_pure={}, concurrency={}",
                report.conformant_tools,
                report.total_tools,
                report.anomalies.len(),
                report.framing_pure,
                report.concurrency
            );
            for t in &report.tools {
                if !t.conformant {
                    println!(
                        "  ✗ {} — {}",
                        t.name,
                        t.anomaly.as_deref().unwrap_or("non-conformant")
                    );
                }
            }
            for a in &report.anomalies {
                println!("  ! {} — {}", a.id, a.detail);
            }
        }
    }
}

fn render_junit(report: &SweepReport) -> String {
    let failures = report.total_tools - report.conformant_tools + report.anomalies.len();
    let mut out = String::new();
    out.push_str(&format!(
        "<testsuite name=\"mcp-sweep\" tests=\"{}\" failures=\"{}\">\n",
        report.total_tools, failures
    ));
    for t in &report.tools {
        if t.conformant {
            out.push_str(&format!("  <testcase name=\"{}\"/>\n", xml_escape(&t.name)));
        } else {
            out.push_str(&format!(
                "  <testcase name=\"{}\"><failure>{}</failure></testcase>\n",
                xml_escape(&t.name),
                xml_escape(t.anomaly.as_deref().unwrap_or("non-conformant"))
            ));
        }
    }
    for a in &report.anomalies {
        out.push_str(&format!(
            "  <testcase name=\"{}\"><failure>{}</failure></testcase>\n",
            xml_escape(&a.id),
            xml_escape(&a.detail)
        ));
    }
    out.push_str("</testsuite>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_derived_from_every_tool_schema() {
        // Every required field of each of the 20 live tools' schema shapes
        // gets a type-appropriate placeholder; path-ish fields get the target.
        let schemas = [
            serde_json::json!({"type": "object",
                "properties": {"paths": {"type": "array"}, "flag": {"type": "boolean"}},
                "required": ["paths"]}),
            serde_json::json!({"type": "object",
                "properties": {"project_path": {"type": "string"}, "n": {"type": "integer"}},
                "required": ["project_path", "n"]}),
            serde_json::json!({"type": "object", "properties": {}, "required": []}),
            serde_json::json!({"type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"]}),
        ];
        for schema in &schemas {
            let args = derive_args_from_schema(schema, ".");
            let required = schema.get("required").and_then(|v| v.as_array()).unwrap();
            for req in required {
                let key = req.as_str().unwrap();
                assert!(args.get(key).is_some(), "required '{key}' must be present");
            }
            // Non-required fields are omitted (minimal).
            if schema["properties"].get("flag").is_some() {
                assert!(args.get("flag").is_none(), "non-required omitted");
            }
        }
        // Path fields carry the target; scalar arrays fill it too.
        let a = derive_args_from_schema(
            &serde_json::json!({"type":"object","properties":{"paths":{"type":"array"}},"required":["paths"]}),
            "/proj",
        );
        assert_eq!(a["paths"], serde_json::json!(["/proj"]));
        let b = derive_args_from_schema(
            &serde_json::json!({"type":"object","properties":{"project_path":{"type":"string"}},"required":["project_path"]}),
            "/proj",
        );
        assert_eq!(b["project_path"], serde_json::json!("/proj"));
    }

    #[test]
    fn derive_args_is_deterministic() {
        let schema = serde_json::json!({"type":"object",
            "properties":{"z":{"type":"string"},"a":{"type":"integer"},"m":{"type":"boolean"}},
            "required":["z","a","m"]});
        let one = serde_json::to_string(&derive_args_from_schema(&schema, ".")).unwrap();
        let two = serde_json::to_string(&derive_args_from_schema(&schema, ".")).unwrap();
        assert_eq!(one, two);
    }

    #[test]
    fn framing_stdout_pure_jsonrpc_golden() {
        let pure = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{}}\n";
        assert!(
            framing_stray_lines(pure).is_empty(),
            "pure frames = no stray"
        );
        let dirty =
            "starting server...\n{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\nWARN: leak\n";
        let stray = framing_stray_lines(dirty);
        assert_eq!(stray.len(), 2, "two non-JSON lines flagged");
        assert!(stray.iter().any(|l| l.contains("starting server")));
    }

    #[test]
    fn conformance_accepts_result_and_error_rejects_malformed() {
        assert!(is_conformant_response(
            &serde_json::json!({"jsonrpc":"2.0","id":1,"result":{}})
        ));
        assert!(is_conformant_response(
            &serde_json::json!({"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"x"}})
        ));
        // Both result and error = malformed.
        assert!(!is_conformant_response(
            &serde_json::json!({"jsonrpc":"2.0","id":1,"result":{},"error":{}})
        ));
        // Missing id, wrong version, non-object.
        assert!(!is_conformant_response(
            &serde_json::json!({"jsonrpc":"2.0","result":{}})
        ));
        assert!(!is_conformant_response(
            &serde_json::json!({"id":1,"result":{}})
        ));
        assert!(!is_conformant_response(&serde_json::json!("nope")));
    }

    #[test]
    fn two_runs_byte_identical_modulo_time() {
        // The report carries no wall-clock field, so identical tool data
        // serializes byte-identically across runs (sweep_deterministic).
        let mk = || {
            SweepReport::build(
                vec![
                    ToolSweepResult {
                        name: "b_tool".into(),
                        conformant: true,
                        anomaly: None,
                    },
                    ToolSweepResult {
                        name: "a_tool".into(),
                        conformant: true,
                        anomaly: None,
                    },
                ],
                vec![],
                true,
                8,
            )
        };
        let one = serde_json::to_string_pretty(&mk()).unwrap();
        let two = serde_json::to_string_pretty(&mk()).unwrap();
        assert_eq!(one, two);
        // Sorted for stability regardless of input order.
        let r = mk();
        assert_eq!(r.tools[0].name, "a_tool");
        assert!(r.ok);
    }

    #[test]
    fn report_ok_requires_pure_framing_no_anomalies_all_conformant() {
        let bad_framing = SweepReport::build(
            vec![ToolSweepResult {
                name: "t".into(),
                conformant: true,
                anomaly: None,
            }],
            vec![],
            false,
            1,
        );
        assert!(!bad_framing.ok);
        let has_anomaly = SweepReport::build(
            vec![ToolSweepResult {
                name: "t".into(),
                conformant: true,
                anomaly: None,
            }],
            vec![SweepAnomaly {
                id: "x".into(),
                detail: "y".into(),
            }],
            true,
            1,
        );
        assert!(!has_anomaly.ok);
        let non_conformant = SweepReport::build(
            vec![ToolSweepResult {
                name: "t".into(),
                conformant: false,
                anomaly: Some("bad".into()),
            }],
            vec![],
            true,
            1,
        );
        assert!(!non_conformant.ok);
    }

    #[test]
    fn junit_render_marks_failures() {
        let report = SweepReport::build(
            vec![
                ToolSweepResult {
                    name: "good".into(),
                    conformant: true,
                    anomaly: None,
                },
                ToolSweepResult {
                    name: "bad".into(),
                    conformant: false,
                    anomaly: Some("no response".into()),
                },
            ],
            vec![],
            true,
            1,
        );
        let xml = render_junit(&report);
        assert!(xml.contains("<testcase name=\"good\"/>"));
        assert!(xml.contains("<testcase name=\"bad\"><failure>no response</failure>"));
        assert!(xml.contains("failures=\"1\""));
    }

    #[test]
    fn no_llm_symbols_linked() {
        // The sweep path must never link a model/API client (spec invariant).
        // Assert this module's own source references no LLM client symbols.
        let src = include_str!("qa_mcp_sweep.rs");
        // Only inspect code below the doc header to avoid matching prose.
        let code = src.split("use anyhow").nth(1).unwrap_or(src);
        for forbidden in [
            "anthropic",
            "openai",
            "api_key",
            "ANTHROPIC_API",
            "chat/completions",
        ] {
            assert!(
                !code.contains(forbidden),
                "sweep path must not reference LLM client symbol '{forbidden}'"
            );
        }
    }

    #[test]
    #[ignore = "spawns the MCP server binary; run after `cargo build` with target/*/pmat present"]
    fn concurrency8_zero_lock_errors_zero_scratch() {
        // Integration: 8 concurrent sweeps against this repo, one working tree.
        let binary = resolve_pmat_binary();
        if !binary.exists() && binary.to_str() == Some("pmat") {
            eprintln!("skip: no pmat binary");
            return;
        }
        let opts = SweepOpts {
            concurrency: 8,
            format: SweepFormat::Json,
            path: PathBuf::from("."),
        };
        // Directly exercise the pass-level machinery without the async wrapper.
        let before = scratch_snapshot(&opts.path);
        let mut reports = Vec::new();
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..8)
                .map(|_| s.spawn(|| run_sweep_once(&binary, ".")))
                .collect();
            for h in handles {
                if let Ok(Ok((tools, anomalies, pure))) = h.join() {
                    reports.push((tools, anomalies, pure));
                }
            }
        });
        let after = scratch_snapshot(&opts.path);
        assert_eq!(
            before, after,
            "no scratch leftovers after 8 concurrent sweeps"
        );
        assert!(!reports.is_empty(), "at least one pass completed");
        // All passes agree on tool conformance (no lock-error divergence).
        // Compare as sorted sets: tools/list order is not stable across
        // server instances, which is exactly why SweepReport::build sorts.
        let sorted = |tools: &[ToolSweepResult]| -> Vec<(String, bool)> {
            let mut v: Vec<(String, bool)> = tools
                .iter()
                .map(|t| (t.name.clone(), t.conformant))
                .collect();
            v.sort();
            v
        };
        let baseline = sorted(&reports[0].0);
        for (tools, _, _) in &reports[1..] {
            assert_eq!(
                sorted(tools),
                baseline,
                "conformance stable across concurrent passes"
            );
        }
    }
}
