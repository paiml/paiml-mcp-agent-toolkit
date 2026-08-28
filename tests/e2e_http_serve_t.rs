//! Transport gate: the **streamable-HTTP MCP** interface, exercised against the
//! shipped binary.
//!
//! Declared by
//! `[package.metadata.transports] http = { e2e = "e2e_http_serve_t", features = ["mcp-http"] }`
//! in `Cargo.toml`. This target carries `required-features = ["mcp-http"]`, and
//! that is precisely why the table has to name it: `cargo test` **silently
//! skips** targets whose required features are off, so a bare run reports
//! success while compiling none of this. Running
//! `cargo test --test e2e_http_serve_t --features mcp-http` turns "absent" into
//! a hard exit 101.
//!
//! `src/mcp_pmcp/http_server.rs` already has an in-crate `http_e2e_tests` module
//! that binds a socket and checks 401-vs-200. It calls `serve()` directly, so it
//! proves the *library* is correct and nothing about whether `main.rs` can reach
//! it. That is the failure this gate exists for: in a sibling repo a four-way
//! parity suite stayed green for months while the mcp and http servers had no
//! caller from the process entry point at all. Here the whole path
//! `main → CLI dispatch → handle_serve → serve_streamable_http → serve` is
//! exercised by spawning `env!("CARGO_BIN_EXE_pmat")`.
//!
//! Port selection: `--port 0` lets the kernel pick, and the handler prints the
//! address it actually bound on stderr, which this file parses. Nothing here
//! assumes 8080 (or any other port) is free.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Long enough for a cold debug binary to boot and bind, short enough that a
/// regression fails with a message instead of wedging the suite.
const BIND_TIMEOUT: Duration = Duration::from_secs(90);

/// Meets `BearerToken`'s 16-character minimum. A shorter one is rejected at
/// construction, which would fail this test for the wrong reason.
const TOKEN: &str = "0123456789abcdef0123";

/// Streamable HTTP negotiates content *before* it authenticates. Without this
/// header the server answers 406 and the 401 assertions below would be measuring
/// content negotiation rather than auth.
const ACCEPT: &str = "application/json, text/event-stream";

/// A spawned `pmat serve` process plus the address it reported binding.
struct ServerUnderTest {
    child: Child,
    base_url: String,
    stderr: Arc<Mutex<Vec<String>>>,
}

impl ServerUnderTest {
    /// Spawn the shipped binary, wait for it to announce a bound address.
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_pmat"))
            .args([
                "serve",
                "--transport",
                "http",
                "--host",
                "127.0.0.1",
                // Kernel-assigned: never assume a fixed port is free.
                "--port",
                "0",
            ])
            .env("PMAT_MCP_HTTP_TOKEN", TOKEN)
            .env("RUST_LOG", "error")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn the pmat binary for `serve --transport http`");

        let stderr_pipe = child.stderr.take().expect("stderr is piped");
        let lines = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = Arc::clone(&lines);
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            // Keep draining after the address is found: a full pipe would block
            // the server mid-test and look like a hang in the transport.
            for line in BufReader::new(stderr_pipe).lines().map_while(Result::ok) {
                if let Some(addr) = parse_bound_addr(&line) {
                    let _ = tx.send(addr);
                }
                sink.lock().expect("stderr buffer").push(line);
            }
        });

        match rx.recv_timeout(BIND_TIMEOUT) {
            Ok(base_url) => ServerUnderTest {
                child,
                base_url,
                stderr: lines,
            },
            // `Disconnected` means the stderr reader thread ended, i.e. the
            // process died without ever binding — the "no caller from main.rs"
            // shape. `Timeout` means it is still alive but silent. Naming which
            // one happened is the difference between a 90-second mystery and a
            // one-line diagnosis.
            Err(why) => {
                let _ = child.kill();
                let _ = child.wait();
                let cause = match why {
                    std::sync::mpsc::RecvTimeoutError::Disconnected => {
                        "the process exited without binding"
                    }
                    std::sync::mpsc::RecvTimeoutError::Timeout => {
                        "the process stayed alive but never announced an address"
                    }
                };
                panic!(
                    "`pmat serve --transport http` never announced a bound address \
                     ({cause}; waited up to {BIND_TIMEOUT:?}). stderr so far:\n{}",
                    lines.lock().expect("stderr buffer").join("\n")
                );
            }
        }
    }

    fn stderr_text(&self) -> String {
        self.stderr.lock().expect("stderr buffer").join("\n")
    }

    /// The server must still be running, then must exit when asked.
    ///
    /// Checking liveness first is the load-bearing half: a process that crashed
    /// after answering one request would otherwise be indistinguishable from a
    /// healthy one that we then killed.
    fn assert_alive_then_shut_down(&mut self) {
        let alive = self.child.try_wait().expect("poll the server process");
        assert!(
            alive.is_none(),
            "the server exited on its own while still under test ({alive:?}); \
             a transport that dies after one request is not serving.\nstderr:\n{}",
            self.stderr_text()
        );

        self.child.kill().expect("signal the server to stop");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match self.child.try_wait().expect("poll the server process") {
                Some(_) => break,
                None if Instant::now() >= deadline => {
                    panic!("the server did not exit after being signalled; it is leaking a process")
                }
                None => std::thread::sleep(Duration::from_millis(25)),
            }
        }
    }
}

impl Drop for ServerUnderTest {
    fn drop(&mut self) {
        // Panicking tests must not leave a listener behind. Both calls are
        // no-ops once `assert_alive_then_shut_down` has already reaped the
        // child, so running this after an explicit shutdown is harmless.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Pull `http://127.0.0.1:PORT` out of the handler's startup banner.
fn parse_bound_addr(line: &str) -> Option<String> {
    let rest = line.split("listening on ").nth(1)?;
    let url = rest.trim().trim_end_matches('/');
    url.starts_with("http://").then(|| url.to_string())
}

fn tools_list_body() -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}})
}

/// The whole contract in one session: it binds, it refuses the unauthenticated,
/// it serves the authenticated, and it stays up.
#[tokio::test]
async fn http_transport_round_trip() {
    let mut server = ServerUnderTest::start();
    let url = format!("{}/", server.base_url);
    let client = reqwest::Client::new();

    // pmcp mounts the MCP endpoint at the ROOT of the server it starts
    // (`build_mcp_router` routes POST/GET/DELETE on "/"), not at /mcp. An
    // earlier version of the in-crate test used /mcp, got 404, and would have
    // "passed" a 401 assertion against a path that does not exist.
    let unauthenticated = client
        .post(&url)
        .header("Accept", ACCEPT)
        .json(&tools_list_body())
        .send()
        .await
        .expect("the server must answer an unauthenticated request, not drop it");
    assert_eq!(
        unauthenticated.status().as_u16(),
        401,
        "an unauthenticated request must be refused, not served.\nstderr:\n{}",
        server.stderr_text()
    );

    let wrong_token = client
        .post(&url)
        .header("Accept", ACCEPT)
        .header("Authorization", "Bearer wrongwrongwrongwrong")
        .json(&tools_list_body())
        .send()
        .await
        .expect("request with a wrong token");
    assert_eq!(
        wrong_token.status().as_u16(),
        401,
        "a wrong bearer token must be refused"
    );

    let authenticated = client
        .post(&url)
        .header("Accept", ACCEPT)
        .header("Authorization", format!("Bearer {TOKEN}"))
        .json(&tools_list_body())
        .send()
        .await
        .expect("request with the correct token");
    assert_eq!(
        authenticated.status().as_u16(),
        200,
        "the correct bearer token must be served.\nstderr:\n{}",
        server.stderr_text()
    );

    let payload: serde_json::Value = authenticated
        .json()
        .await
        .expect("an authenticated tools/list must return JSON");
    assert_eq!(
        payload.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "the HTTP transport must speak JSON-RPC 2.0, got: {payload}"
    );
    assert!(
        payload.get("error").is_none(),
        "tools/list must not return a JSON-RPC error, got: {payload}"
    );
    let tools = payload
        .pointer("/result/tools")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("tools/list must return result.tools as an array: {payload}"));
    assert!(
        !tools.is_empty(),
        "the HTTP transport must advertise at least one tool; an endpoint that is \
         reachable but wired to nothing is the failure this gate exists to catch"
    );
    for tool in tools {
        assert!(
            tool.get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|n| !n.is_empty()),
            "every advertised tool needs a name, got: {tool}"
        );
    }

    server.assert_alive_then_shut_down();
}

/// The banner reports the port the kernel actually assigned, not the `0` asked
/// for. Without this the round trip above could be talking to something else.
#[tokio::test]
async fn bound_address_is_reported_and_is_not_the_requested_placeholder() {
    let mut server = ServerUnderTest::start();
    let port: u16 = server
        .base_url
        .rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| panic!("could not read a port out of `{}`", server.base_url));
    assert_ne!(
        port, 0,
        "`--port 0` must be resolved to the kernel-assigned port in the banner"
    );
    assert!(
        server.base_url.starts_with("http://127.0.0.1:"),
        "the server must bind the requested host, got `{}`",
        server.base_url
    );
    server.assert_alive_then_shut_down();
}

/// It fails closed. Without a token, pmcp would serve every request, so a
/// "working" endpoint with no token publishes the entire tool surface to anyone
/// who can reach the port. Starting must be impossible, not merely discouraged.
#[tokio::test]
async fn refuses_to_start_without_a_token() {
    let out = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args([
            "serve",
            "--transport",
            "http",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
        ])
        .env_remove("PMAT_MCP_HTTP_TOKEN")
        .env("RUST_LOG", "error")
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn the pmat binary");

    assert!(
        !out.status.success(),
        "an unauthenticated MCP endpoint must not start"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "a non-zero exit must explain itself on stderr"
    );
    assert!(
        stderr.contains("PMAT_MCP_HTTP_TOKEN"),
        "the diagnostic must name the variable to set, got: {stderr}"
    );
}

/// One malformed-or-odd frame and the JSON-RPC answer the transport owes it.
struct BadFrame {
    /// What is wrong with it, for the assertion message.
    what: &'static str,
    /// Raw bytes, sent verbatim — several of these are not valid JSON, so they
    /// cannot go through `reqwest`'s `.json()`.
    body: &'static str,
    /// The JSON-RPC error code JSON-RPC 2.0 assigns this fault.
    code: i64,
    /// The id the client sent, which must come back so it can resolve its
    /// pending promise. `null` only where there is no id to recover.
    id: serde_json::Value,
    /// A phrase the message must NOT contain, where the old answer contained it.
    never_says: Option<&'static str>,
}

/// The contract, stated here rather than derived, because `classify_bad_frame`
/// and `FrameVerdict` are `pub(crate)` and an integration test lives outside the
/// crate. These rows are the same ones the in-crate table in
/// `src/mcp_pmcp/http_frames.rs` drives, and that module asserts its verdicts
/// are byte-for-byte what the stdio classifier gives — so agreement here is
/// agreement between the two transports.
fn bad_frames() -> Vec<BadFrame> {
    vec![
        BadFrame {
            what: "a frame declaring JSON-RPC 1.0",
            body: r#"{"jsonrpc":"1.0","id":1,"method":"tools/list","params":{}}"#,
            code: -32600,
            id: serde_json::json!(1),
            // This one was answered 200 with the full tool listing: an answer in
            // a protocol the client never asked for.
            never_says: None,
        },
        BadFrame {
            what: "a frame declaring an invented JSON-RPC version",
            body: r#"{"jsonrpc":"9.9","id":"a","method":"tools/list","params":{}}"#,
            code: -32600,
            id: serde_json::json!("a"),
            never_says: None,
        },
        BadFrame {
            what: "a frame with no jsonrpc member at all",
            body: r#"{"id":2,"method":"tools/list","params":{}}"#,
            code: -32600,
            id: serde_json::json!(2),
            never_says: None,
        },
        BadFrame {
            what: "an unknown method",
            body: r#"{"jsonrpc":"2.0","id":3,"method":"no/such/method","params":{}}"#,
            code: -32601,
            id: serde_json::json!(3),
            never_says: None,
        },
        BadFrame {
            what: "a known method whose params are rejected",
            body: r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":null}"#,
            code: -32602,
            id: serde_json::json!(4),
            // `tools/call` exists and works. Reporting it missing points the
            // operator at the wrong defect, which is what the old answer did.
            never_says: Some("Method not found"),
        },
        BadFrame {
            what: "an object with an id but no method",
            body: r#"{"jsonrpc":"2.0","id":5}"#,
            code: -32600,
            id: serde_json::json!(5),
            never_says: None,
        },
        BadFrame {
            what: "bytes that are not JSON",
            body: r#"{"jsonrpc":"2.0","id":6,"method":"tools/list""#,
            code: -32700,
            // The ONLY row where a null id is correct: there is no id to echo
            // out of bytes that do not parse.
            id: serde_json::Value::Null,
            never_says: None,
        },
    ]
}

/// The transport-parity gate. Before this existed,
/// `grep -cE '32700|32600|32602|32601'` over this file returned 0, while the
/// same predicate over `stdio_frames.rs`'s test modules returned 24 — which is
/// how a compliance audit came to rate the JSON-RPC transport compliant having
/// measured only one of its two halves.
///
/// Two failures are pinned here, and they are the two a host actually feels:
///
/// 1. The `jsonrpc` member was never checked, so `{"jsonrpc":"1.0",…}` was
///    served HTTP 200 with the whole tool listing.
/// 2. Every client-side fault collapsed to one parse error with `"id":null` on
///    HTTP 400 — the true code surviving only as prose inside the message. A
///    client correlating by id gets nothing back to correlate and waits out its
///    own timeout.
///
/// All cases run against ONE spawned server: binding a debug binary is the
/// expensive part, and a per-case server would turn a fast gate into a slow one.
#[tokio::test]
async fn malformed_frames_answer_with_the_right_code_and_the_clients_own_id() {
    let mut server = ServerUnderTest::start();
    let url = format!("{}/", server.base_url);
    let client = reqwest::Client::new();

    for case in bad_frames() {
        let response = client
            .post(&url)
            .header("Accept", ACCEPT)
            // Required by the transport before it parses anything; without it
            // every row below would be measuring content negotiation.
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {TOKEN}"))
            .body(case.body)
            .send()
            .await
            .unwrap_or_else(|e| panic!("{}: the server must answer, got {e}", case.what));

        let status = response.status().as_u16();
        let payload: serde_json::Value = response.json().await.unwrap_or_else(|e| {
            panic!(
                "{}: the answer must be JSON, got {e}.\nstderr:\n{}",
                case.what,
                server.stderr_text()
            )
        });

        assert_eq!(
            status, 200,
            "{}: a JSON-RPC error is an application result, not an HTTP-level \
             rejection; answering {status} makes hosts discard a well-formed \
             reply unread. got: {payload}",
            case.what
        );
        assert_eq!(
            payload.get("jsonrpc").and_then(|v| v.as_str()),
            Some("2.0"),
            "{}: the reply itself must be JSON-RPC 2.0, got {payload}",
            case.what
        );
        assert!(
            payload.get("result").is_none(),
            "{}: a refused frame must not be served a result, got {payload}",
            case.what
        );
        assert_eq!(
            payload.pointer("/error/code").and_then(|v| v.as_i64()),
            Some(case.code),
            "{}: the code must say what is actually wrong, not be smuggled \
             into the message text. got {payload}",
            case.what
        );
        assert_eq!(
            payload
                .get("id")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
            case.id,
            "{}: the client's id must come back or its promise never resolves. \
             got {payload}",
            case.what
        );
        if let Some(phrase) = case.never_says {
            let message = payload
                .pointer("/error/message")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            assert!(
                !message.contains(phrase),
                "{}: the diagnostic must not claim `{phrase}`, got {message:?}",
                case.what
            );
        }
    }

    server.assert_alive_then_shut_down();
}
