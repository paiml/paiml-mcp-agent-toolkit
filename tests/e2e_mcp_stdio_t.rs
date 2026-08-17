//! Transport gate: the **MCP stdio** interface, exercised against the shipped binary.
//!
//! Declared by `[package.metadata.transports] mcp = { e2e = "e2e_mcp_stdio_t" }`
//! in `Cargo.toml`. Naming the target is the point of the table: a bare
//! `cargo test` cannot prove a transport is covered, so the release protocol
//! runs `cargo test --test e2e_mcp_stdio_t` and gets a hard exit 101 if this
//! file ever disappears.
//!
//! Everything here drives `env!("CARGO_BIN_EXE_pmat")` as a real child process
//! with `MCP_VERSION=1`, which is the only route that actually serves the tool
//! surface over stdio (`src/bin/pmat.rs` gates on that variable). Calling
//! `SimpleUnifiedServer` from the library would assert the server works while
//! saying nothing about whether `main` can reach it — the exact way a sibling
//! repo kept a green parity suite over two transports with no caller at all.
//!
//! Two invariants, and the second is the one that bites:
//!
//! 1. **Round trip.** `initialize` and `tools/list` are both answered, with
//!    well-formed JSON-RPC and a non-empty tool list.
//! 2. **stdout is protocol-only.** MCP frames JSON-RPC on stdout, so a single
//!    stray `println!` anywhere in the process corrupts the stream for every
//!    client. That bug shipped in a sibling crate. Diagnostics belong on stderr.
//!
//! `tests/modules/mcp_stdio_no_truncation.rs` pins the related EOF-truncation
//! race across repeated trials; this target is the transport gate proper and
//! additionally asserts stream purity, which that file does not check.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A session must complete well inside this. It exists so a regression that
/// hangs the server fails with a readable message instead of wedging the suite
/// until the outer harness kills it.
const SESSION_TIMEOUT: Duration = Duration::from_secs(120);

/// Captured output of one piped MCP session.
struct Session {
    stdout: String,
    stderr: String,
}

/// The standard opening exchange, written in one shot; stdin is then closed.
fn handshake_payload() -> String {
    [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"transport-gate","version":"1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]
    .join("\n")
        + "\n"
}

/// Spawn the shipped binary in MCP stdio mode, feed it `requests`, close stdin
/// and collect both streams.
fn run_session(requests: &str) -> Session {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .env("MCP_VERSION", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn the pmat binary in MCP stdio mode");

    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(requests.as_bytes())
        .expect("write MCP requests");
    // EOF on stdin is how a one-shot client signals it is done.
    drop(child.stdin.take());

    let mut out_pipe = child.stdout.take().expect("stdout is piped");
    let mut err_pipe = child.stderr.take().expect("stderr is piped");
    let (tx, rx) = mpsc::channel::<(&'static str, String)>();
    let tx_err = tx.clone();
    std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = out_pipe.read_to_string(&mut buf);
        let _ = tx.send(("stdout", buf));
    });
    std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = err_pipe.read_to_string(&mut buf);
        let _ = tx_err.send(("stderr", buf));
    });

    let deadline = Instant::now() + SESSION_TIMEOUT;
    let (mut stdout, mut stderr) = (None, None);
    for _ in 0..2 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok(("stdout", s)) => stdout = Some(s),
            Ok((_, s)) => stderr = Some(s),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!(
                    "the MCP stdio server did not finish within {SESSION_TIMEOUT:?} after stdin \
                     EOF; a one-shot piped session must drain and exit, not hang"
                );
            }
        }
    }
    let _ = child.wait();

    Session {
        stdout: stdout.expect("stdout collected"),
        stderr: stderr.expect("stderr collected"),
    }
}

/// Non-empty stdout lines, which on this transport must all be protocol frames.
fn stdout_lines(session: &Session) -> Vec<&str> {
    session
        .stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

/// Parse stdout as a JSON-RPC stream, panicking with the offending line if any
/// line is not a frame.
fn protocol_frames(session: &Session) -> Vec<serde_json::Value> {
    stdout_lines(session)
        .into_iter()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
                panic!(
                    "stdout carries a non-JSON line, which corrupts the MCP stream for every \
                     client: {e}\n  line: {line}\n  (diagnostics must go to stderr)"
                )
            });
            assert_eq!(
                value.get("jsonrpc").and_then(|v| v.as_str()),
                Some("2.0"),
                "every stdout line must be a JSON-RPC 2.0 frame, got: {line}"
            );
            value
        })
        .collect()
}

fn response_with_id(frames: &[serde_json::Value], id: i64) -> &serde_json::Value {
    frames
        .iter()
        .find(|f| f.get("id").and_then(|v| v.as_i64()) == Some(id))
        .unwrap_or_else(|| panic!("no JSON-RPC response for id {id}; frames: {frames:?}"))
}

/// `initialize` is answered with a protocol version and this crate's identity.
#[test]
fn initialize_round_trip() {
    let session = run_session(&handshake_payload());
    let frames = protocol_frames(&session);
    let init = response_with_id(&frames, 1);

    let result = init
        .get("result")
        .unwrap_or_else(|| panic!("initialize must return a result, got: {init}"));
    assert!(
        result
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .is_some(),
        "initialize result must carry a protocolVersion, got: {result}"
    );
    let server_version = result
        .pointer("/serverInfo/version")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("initialize result must carry serverInfo.version: {result}"));
    assert_eq!(
        server_version,
        env!("CARGO_PKG_VERSION"),
        "the MCP server must report the version of the crate being released"
    );
}

/// `tools/list` returns a real, non-empty, well-formed tool surface.
#[test]
fn tools_list_round_trip() {
    let session = run_session(&handshake_payload());
    let frames = protocol_frames(&session);
    let listed = response_with_id(&frames, 2);

    assert!(
        listed.get("error").is_none(),
        "tools/list must not return a JSON-RPC error, got: {listed}"
    );
    let tools = listed
        .pointer("/result/tools")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("tools/list must return result.tools as an array: {listed}"));
    assert!(
        !tools.is_empty(),
        "tools/list must advertise at least one tool; an empty surface means the \
         server is reachable but wired to nothing"
    );
    for tool in tools {
        assert!(
            tool.get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|n| !n.is_empty()),
            "every advertised tool needs a name, got: {tool}"
        );
        assert!(
            tool.get("inputSchema").is_some(),
            "every advertised tool needs an inputSchema, got: {tool}"
        );
    }
}

/// stdout carries protocol frames and nothing else.
///
/// This is the assertion that catches the class of bug that shipped in a sibling
/// crate: one `println!` reaching stdout desynchronises every MCP client, while
/// the server itself still looks perfectly healthy from the inside.
#[test]
fn stdout_carries_only_protocol_lines() {
    let session = run_session(&handshake_payload());
    let lines = stdout_lines(&session);
    assert!(
        !lines.is_empty(),
        "the server answered nothing on stdout; stderr was:\n{}",
        session.stderr
    );

    let mut contaminants = Vec::new();
    for line in &lines {
        let is_frame = match serde_json::from_str::<serde_json::Value>(line) {
            Ok(value) => value.get("jsonrpc").and_then(|j| j.as_str()) == Some("2.0"),
            Err(_) => false,
        };
        if !is_frame {
            contaminants.push(*line);
        }
    }
    assert!(
        contaminants.is_empty(),
        "{} line(s) on stdout are not JSON-RPC frames. On MCP stdio this corrupts the \
         stream for every client — route these to stderr:\n  {}",
        contaminants.len(),
        contaminants.join("\n  ")
    );

    // Every response must answer a request we actually sent. Unsolicited frames
    // on this transport are as damaging as plain text.
    let frames = protocol_frames(&session);
    for frame in &frames {
        if let Some(id) = frame.get("id").and_then(|v| v.as_i64()) {
            assert!(
                id == 1 || id == 2,
                "unexpected JSON-RPC id {id} on stdout; only ids 1 and 2 were requested"
            );
        }
    }
}

/// Banner text belongs on stderr, so a session that prints nothing but frames on
/// stdout is not merely a session that printed nothing at all.
#[test]
fn every_request_is_answered_before_exit() {
    let session = run_session(&handshake_payload());
    let frames = protocol_frames(&session);
    let answered: Vec<i64> = frames
        .iter()
        .filter_map(|f| f.get("id").and_then(|v| v.as_i64()))
        .collect();
    for id in [1, 2] {
        assert!(
            answered.contains(&id),
            "request id {id} was consumed but never answered; got ids {answered:?}\nstderr:\n{}",
            session.stderr
        );
    }
}
