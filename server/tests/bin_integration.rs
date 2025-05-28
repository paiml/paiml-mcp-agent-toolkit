use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_binary_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("paiml-mcp-agent-toolkit"));
    assert!(stdout.contains("0.2.")); // Version should start with 0.2
}

#[test]
fn test_binary_json_rpc_initialize() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");

    // Send a valid JSON-RPC request
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    stdin
        .write_all(request.as_bytes())
        .expect("Failed to write to stdin");
    stdin.write_all(b"\n").expect("Failed to write newline");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to wait for output");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"jsonrpc\":\"2.0\""));
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"result\""));
}

#[test]
fn test_binary_invalid_json() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");

    // Send invalid JSON
    stdin
        .write_all(b"invalid json\n")
        .expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to wait for output");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"error\""));
    assert!(stdout.contains("Parse error"));
}

#[test]
fn test_binary_multiple_requests() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");

    // Send multiple requests
    let req1 = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let req2 = r#"{"jsonrpc":"2.0","id":2,"method":"prompts/list"}"#;

    stdin
        .write_all(req1.as_bytes())
        .expect("Failed to write request 1");
    stdin.write_all(b"\n").expect("Failed to write newline");
    stdin
        .write_all(req2.as_bytes())
        .expect("Failed to write request 2");
    stdin.write_all(b"\n").expect("Failed to write newline");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to wait for output");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"id\":2"));
    assert!(stdout.contains("\"tools\""));
    assert!(stdout.contains("\"prompts\""));
}
