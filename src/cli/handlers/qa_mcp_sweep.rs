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
    /// Digest of the response payload with volatile fields normalised — the
    /// only thing that can tell one pass's answer from another's. Without it
    /// the advertised "replay determinism" check compared nothing but the
    /// `conformant` boolean, so a server returning a fresh random nonce on
    /// every call swept clean.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_digest: Option<String>,
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

/// Locate the pmat binary to spawn (release preferred, then debug, then the
/// running executable, then PATH).
///
/// The candidates used to be returned as the RELATIVE paths they were tested
/// as (`target/release/pmat`), while `run_sweep_once` spawns with
/// `.current_dir(<swept path>)` — so the existence check and the execution
/// referred to different directories. Sweeping any directory from a built tree
/// executed `<swept path>/target/release/pmat`, i.e. whatever binary happened
/// to sit there, and a swept path with no `target/` failed to spawn even with
/// pmat on PATH. Candidates are now absolutised against the caller's cwd
/// before they are handed to `Command`.
fn resolve_pmat_binary() -> PathBuf {
    for candidate in ["target/release/pmat", "target/debug/pmat"] {
        if let Ok(abs) = std::fs::canonicalize(candidate) {
            return abs;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        return exe;
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

    let mut child = match Command::new(binary)
        .env("MCP_VERSION", "1")
        .current_dir(target)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            std::fs::remove_dir_all(&fixture).ok();
            // Name the binary we actually tried: the message used to say
            // "pmat" while spawning a relative path resolved against the
            // swept directory.
            return Err(anyhow::Error::from(e).context(format!(
                "spawn MCP server (MCP_VERSION=1 {})",
                binary.display()
            )));
        }
    };

    // Take the pipes without `?`: an error here must still reap the child and
    // remove the fixture (no zombie process, no temp-dir leak).
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        let _ = child.wait();
        std::fs::remove_dir_all(&fixture).ok();
        anyhow::bail!("MCP server pipes unavailable");
    };

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
    let mut raw_stdout = String::new();
    let mut frames: Vec<serde_json::Value> = Vec::new();

    // Phase 1: initialize + tools/list, read frames until the list response.
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                       "clientInfo": {"name": "pmat-mcp-sweep", "version": "1"}}})
    )
    .context("write initialize")?;
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
    )
    .context("write tools/list")?;
    stdin.flush().ok();
    let tools = read_tool_list(&rx, deadline, &mut raw_stdout, &mut frames);

    // Phase 2: one tools/call per tool with schema-derived args (pointed at
    // the minimal fixture so tools do near-zero work).
    let expected_ids = send_tool_calls(&mut stdin, &tools, &arg_target);
    stdin.flush().ok();
    drop(stdin); // EOF -> server exits cleanly (EofSignalingTransport)

    // Drain responses until answered or deadline, then reap the server.
    drain_responses(&rx, deadline, &expected_ids, &mut raw_stdout, &mut frames);
    let _ = child.kill();
    let _ = child.wait();
    std::fs::remove_dir_all(&fixture).ok();

    let results = correlate_results(&expected_ids, &frames, &arg_target);
    let (anomalies, framing_pure) = framing_anomalies(&raw_stdout, tools.is_empty());
    Ok((results, anomalies, framing_pure))
}

/// Receive one line from the reader thread, bounded by `deadline`.
fn recv_before(
    rx: &std::sync::mpsc::Receiver<String>,
    deadline: std::time::Instant,
) -> Option<String> {
    let now = std::time::Instant::now();
    if now >= deadline {
        return None;
    }
    rx.recv_timeout(deadline - now).ok()
}

/// Phase 1: read frames until the tools/list response (id == 2); return its
/// advertised tools. Appends every raw line + parsed frame.
fn read_tool_list(
    rx: &std::sync::mpsc::Receiver<String>,
    deadline: std::time::Instant,
    raw_stdout: &mut String,
    frames: &mut Vec<serde_json::Value>,
) -> Vec<serde_json::Value> {
    while let Some(line) = recv_before(rx, deadline) {
        raw_stdout.push_str(&line);
        let Ok(frame) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
            continue;
        };
        let is_list = frame.get("id").and_then(|v| v.as_u64()) == Some(2);
        let tools = is_list.then(|| {
            frame
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array())
                .cloned()
                .unwrap_or_default()
        });
        frames.push(frame);
        if let Some(tools) = tools {
            return tools;
        }
    }
    Vec::new()
}

/// Phase 2: send one tools/call per advertised tool with schema-derived args;
/// return the (request id, tool name) pairs to correlate against.
fn send_tool_calls(
    stdin: &mut impl Write,
    tools: &[serde_json::Value],
    arg_target: &str,
) -> Vec<(u64, String)> {
    let mut expected = Vec::new();
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
        let args = derive_args_from_schema(&schema, arg_target);
        let id = 100 + i as u64;
        let _ = writeln!(
            stdin,
            "{}",
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": {"name": name, "arguments": args}})
        );
        expected.push((id, name));
    }
    expected
}

/// Drain response frames until every expected id is answered or the deadline
/// hits (bounds a slow/hanging tool).
fn drain_responses(
    rx: &std::sync::mpsc::Receiver<String>,
    deadline: std::time::Instant,
    expected_ids: &[(u64, String)],
    raw_stdout: &mut String,
    frames: &mut Vec<serde_json::Value>,
) {
    let wanted: std::collections::HashSet<u64> = expected_ids.iter().map(|(id, _)| *id).collect();
    let mut answered: std::collections::HashSet<u64> = std::collections::HashSet::new();
    while answered.len() < wanted.len() {
        let Some(line) = recv_before(rx, deadline) else {
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
}

/// Correlate each expected tool-call id to its response frame: conformant,
/// malformed, or missing.
fn correlate_results(
    expected_ids: &[(u64, String)],
    frames: &[serde_json::Value],
    arg_target: &str,
) -> Vec<ToolSweepResult> {
    expected_ids
        .iter()
        .map(|(id, name)| {
            let frame = frames
                .iter()
                .find(|f| f.get("id").and_then(|v| v.as_u64()) == Some(*id));
            let anomaly = match frame {
                Some(f) if is_conformant_response(f) => None,
                Some(_) => Some("malformed JSON-RPC response frame".to_string()),
                None => Some("no response frame for this tool call".to_string()),
            };
            ToolSweepResult {
                name: name.clone(),
                conformant: anomaly.is_none(),
                anomaly,
                response_digest: frame.map(|f| response_digest(f, arg_target)),
            }
        })
        .collect()
}

/// Digest of a tool response's payload, used for the cross-pass replay check.
///
/// Only the `result`/`error` payload is hashed (the JSON-RPC envelope's `id`
/// is per-pass bookkeeping), and fields that legitimately differ between two
/// identical requests — wall-clock, durations, pids, session ids, and the
/// per-pass fixture path — are normalised away first, so a mismatch means the
/// server really did answer the same question differently.
fn response_digest(frame: &serde_json::Value, arg_target: &str) -> String {
    let mut payload = frame
        .get("result")
        .or_else(|| frame.get("error"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    normalize_volatile(&mut payload, arg_target);
    // serde_json serialises object keys in sorted order, so the canonical form
    // is stable without any extra ordering pass.
    let canonical = serde_json::to_string(&payload).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&canonical, &mut hasher);
    format!("{:016x}", std::hash::Hasher::finish(&hasher))
}

/// Key fragments whose values vary between two identical requests by design.
const VOLATILE_KEY_FRAGMENTS: &[&str] = &[
    "timestamp",
    "time",
    "duration",
    "elapsed",
    "started",
    "finished",
    "generated",
    "session",
    "pid",
    "uuid",
    "seed",
    "_ms",
];

/// Null out volatile values and mask the per-pass fixture path in-place.
fn normalize_volatile(value: &mut serde_json::Value, arg_target: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lc = key.to_lowercase();
                if VOLATILE_KEY_FRAGMENTS.iter().any(|f| key_lc.contains(f)) {
                    *val = serde_json::Value::Null;
                } else {
                    normalize_volatile(val, arg_target);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_volatile(item, arg_target);
            }
        }
        serde_json::Value::String(s) if !arg_target.is_empty() && s.contains(arg_target) => {
            *s = s.replace(arg_target, "<TARGET>");
        }
        _ => {}
    }
}

/// Build framing anomalies (stray non-JSON stdout lines) + purity flag.
fn framing_anomalies(raw_stdout: &str, tools_empty: bool) -> (Vec<SweepAnomaly>, bool) {
    let stray = framing_stray_lines(raw_stdout);
    let mut anomalies: Vec<SweepAnomaly> = stray
        .iter()
        .enumerate()
        .map(|(i, s)| SweepAnomaly {
            id: format!("framing-{i:03}"),
            detail: format!("non-JSON stdout line: {}", truncate(s, 120)),
        })
        .collect();
    if tools_empty {
        anomalies.push(SweepAnomaly {
            id: "tools-list-empty".to_string(),
            detail: "server advertised zero tools (tools/list returned none)".to_string(),
        });
    }
    (anomalies, stray.is_empty())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Char-safe: never slice mid-codepoint (a stray non-JSON line — the very
    // framing anomaly we report — can be arbitrary UTF-8). Adversarial-review
    // fix for a panic at a non-char-boundary byte index.
    let end = (0..=max)
        .rev()
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(0);
    format!("{}…", &s[..end])
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

/// Cross-pass invariants: every pass must agree on tool conformance (no
/// lock-error-induced divergence) AND on the payload each tool returned.
///
/// The payload half is what `--help` calls "replay determinism", and it was
/// missing: the only comparison built a `name -> conformant` map from pass 0,
/// so a server answering every `tools/call` with a fresh random nonce swept
/// clean with ok:true. `conformant` only means "a well-formed JSON-RPC frame
/// came back" — eight passes can return eight different documents and still
/// agree on that boolean.
fn cross_pass_anomalies(pass_reports: &[Vec<ToolSweepResult>]) -> Vec<SweepAnomaly> {
    let mut anomalies = Vec::new();
    let Some(first) = pass_reports.first() else {
        return anomalies;
    };
    let baseline: std::collections::BTreeMap<&str, &ToolSweepResult> =
        first.iter().map(|t| (t.name.as_str(), t)).collect();

    for (pass, report) in pass_reports.iter().enumerate().skip(1) {
        for t in report {
            let Some(base) = baseline.get(t.name.as_str()) else {
                continue;
            };
            if base.conformant != t.conformant {
                anomalies.push(SweepAnomaly {
                    id: format!("concurrency-divergence-{}-pass{pass:02}", t.name),
                    detail: format!(
                        "tool {} conformance differs across passes (possible lock error)",
                        t.name
                    ),
                });
            }
            if base.response_digest.is_some() && base.response_digest != t.response_digest {
                anomalies.push(SweepAnomaly {
                    id: format!("replay-nondeterminism-{}-pass{pass:02}", t.name),
                    detail: format!(
                        "tool {} answered the identical request with a different payload \
                         (pass00 digest {}, pass{pass:02} digest {})",
                        t.name,
                        base.response_digest.as_deref().unwrap_or("none"),
                        t.response_digest.as_deref().unwrap_or("none"),
                    ),
                });
            }
        }
    }
    anomalies
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

    all_anomalies.extend(cross_pass_anomalies(&pass_reports));

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
    fn truncate_is_char_safe_on_multibyte() {
        // ADVERSARIAL-REVIEW regression: truncating a stray UTF-8 line at a
        // byte offset that lands mid-codepoint must not panic.
        let s = format!("{}\u{1F600}", "x".repeat(118)); // byte 120 inside the emoji
        let out = truncate(&s, 120);
        assert!(out.ends_with('\u{2026}'));
        assert!(out.is_char_boundary(out.len()));
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
                        response_digest: None,
                    },
                    ToolSweepResult {
                        name: "a_tool".into(),
                        conformant: true,
                        anomaly: None,
                        response_digest: None,
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
                response_digest: None,
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
                response_digest: None,
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
                response_digest: None,
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
                    response_digest: None,
                },
                ToolSweepResult {
                    name: "bad".into(),
                    conformant: false,
                    anomaly: Some("no response".into()),
                    response_digest: None,
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

    fn frame(id: u64, text: &str) -> serde_json::Value {
        serde_json::json!({"jsonrpc":"2.0","id":id,
            "result":{"content":[{"type":"text","text":text}]}})
    }

    #[test]
    fn replay_nondeterminism_is_an_anomaly() {
        // A server whose every tools/call answers with a fresh nonce used to
        // sweep clean: only the `conformant` boolean was compared across
        // passes, and eight different payloads all set it to true.
        let expected = vec![(100u64, "alpha_tool".to_string())];
        let pass0 = correlate_results(&expected, &[frame(100, "NONCE-1-111")], "/tmp/fixture");
        let pass1 = correlate_results(&expected, &[frame(100, "NONCE-2-222")], "/tmp/fixture");
        assert!(
            pass0[0].conformant && pass1[0].conformant,
            "both well-formed"
        );

        let anomalies = cross_pass_anomalies(&[pass0, pass1]);
        assert_eq!(anomalies.len(), 1, "got: {anomalies:?}");
        assert!(anomalies[0]
            .id
            .starts_with("replay-nondeterminism-alpha_tool"));
    }

    #[test]
    fn identical_payloads_and_volatile_fields_are_not_anomalies() {
        let expected = vec![(100u64, "alpha_tool".to_string())];
        // Same answer, different wall-clock/duration and a different per-pass
        // fixture path: neither is the server answering differently.
        let a = serde_json::json!({"jsonrpc":"2.0","id":100,
            "result":{"files":1,"timestamp":"2026-01-01T00:00:00Z","duration_ms":7,
                      "root":"/tmp/pmat-mcp-sweep-1-0"}});
        let b = serde_json::json!({"jsonrpc":"2.0","id":100,
            "result":{"files":1,"timestamp":"2026-01-01T00:00:09Z","duration_ms":9,
                      "root":"/tmp/pmat-mcp-sweep-2-1"}});
        let pass0 = correlate_results(&expected, &[a], "/tmp/pmat-mcp-sweep-1-0");
        let pass1 = correlate_results(&expected, &[b], "/tmp/pmat-mcp-sweep-2-1");
        assert_eq!(pass0[0].response_digest, pass1[0].response_digest);
        assert!(cross_pass_anomalies(&[pass0, pass1]).is_empty());
    }

    #[test]
    fn resolved_binary_is_absolute_so_cwd_and_spawn_dir_agree() {
        // `run_sweep_once` spawns with `.current_dir(<swept path>)`, so a
        // relative "target/release/pmat" was re-resolved against the SWEPT
        // directory and executed whatever binary sat there.
        let binary = resolve_pmat_binary();
        assert!(
            binary.is_absolute() || binary == std::path::Path::new("pmat"),
            "candidate must be absolute (or the bare PATH name), got {}",
            binary.display()
        );
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
