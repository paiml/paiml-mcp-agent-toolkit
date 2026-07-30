//! `MCP_VERSION=1 pmat` must answer every request it consumed, even when the
//! client closes stdin immediately after writing.
//!
//! Regression guard for the EOF truncation race: `EofSignalingTransport`
//! signalled session end on the *first* receive error, so EOF observed while a
//! request was still being handled made `run`'s `select!` exit before that
//! response was written. Measured on a release build, `tools/list` was answered
//! in only 11 of 30 one-shot piped sessions.
//!
//! This test spawns the binary via `CARGO_BIN_EXE_pmat`, which cargo resolves
//! to the artifact it just built. That matters: this repo's committed
//! `.cargo/config.toml` redirects `target-dir` to an absolute path, so a
//! hand-written binary path can silently read a stale build — which is exactly
//! how an earlier "improved to 8/12" measurement was produced against code
//! that did not contain the fix.

use std::io::Write;
use std::process::{Command, Stdio};

/// Requests written in one shot, then stdin closed.
fn one_shot_payload() -> String {
    [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"regression","version":"1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]
    .join("\n")
        + "\n"
}

/// Run one piped session; return the set of JSON-RPC ids that got a response.
fn answered_ids() -> Vec<i64> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .env("MCP_VERSION", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn pmat");

    child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(one_shot_payload().as_bytes())
        .expect("write requests");
    // Drop stdin -> EOF, which is what triggered the race.
    drop(child.stdin.take());

    let out = child.wait_with_output().expect("wait for pmat");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('{') {
                return None;
            }
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            v.get("id")?.as_i64()
        })
        .collect()
}

#[test]
fn every_consumed_request_is_answered_before_exit() {
    // The race was probabilistic (11/30), so a single trial proves little.
    // Ten consecutive clean runs is a tight enough bound to fail loudly if the
    // truncation returns, while staying cheap.
    const TRIALS: usize = 10;
    let mut truncated = Vec::new();

    for trial in 0..TRIALS {
        let ids = answered_ids();
        if !ids.contains(&1) {
            truncated.push(format!("trial {trial}: initialize (id=1) unanswered"));
        }
        if !ids.contains(&2) {
            truncated.push(format!("trial {trial}: tools/list (id=2) unanswered"));
        }
    }

    assert!(
        truncated.is_empty(),
        "the server exited before answering requests it had already consumed \
         ({} of {TRIALS} trials affected):\n  {}",
        truncated.len(),
        truncated.join("\n  ")
    );
}

#[test]
fn one_shot_session_exits_without_hanging() {
    // The wrapper being fixed exists because pmcp's keep-alive future never
    // completes, so a piped session used to live until externally killed.
    // Draining in-flight work must not reintroduce that.
    let ids = answered_ids();
    assert!(
        ids.contains(&1),
        "initialize must be answered in a one-shot piped session, got ids {ids:?}"
    );
}
