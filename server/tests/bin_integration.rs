use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_binary_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "pmat", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Command failed with status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pmat"));
    // Just verify it outputs a version number in semver format
    assert!(
        stdout.contains('.'),
        "Output should contain a version number with dots"
    );
}

#[test]
fn test_binary_json_rpc_initialize() {
    // Skip in CI to avoid timeout issues with MCP server that runs forever
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").is_ok() {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }
    
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "pmat"])
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
    
    // Kill the process after sending request since MCP servers don't exit
    std::thread::sleep(std::time::Duration::from_secs(2));
    child.kill().expect("Failed to kill process");
    
    let output = child.wait_with_output().expect("Failed to wait for output");
    // Don't check exit status since we killed it
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"jsonrpc\":\"2.0\""));
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"result\""));
}

#[test]
fn test_binary_invalid_json() {
    // Skip in CI to avoid timeout issues with MCP server that runs forever
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").is_ok() {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }
    
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "pmat"])
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
    
    // Kill the process after sending request since MCP servers don't exit
    std::thread::sleep(std::time::Duration::from_secs(2));
    child.kill().expect("Failed to kill process");
    
    let output = child.wait_with_output().expect("Failed to wait for output");
    // Don't check exit status since we killed it

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"error\""));
    assert!(stdout.contains("Parse error"));
}

#[test]
fn test_binary_multiple_requests() {
    // Skip in CI to avoid timeout issues with MCP server that runs forever
    if std::env::var("CI").is_ok() || std::env::var("SKIP_SLOW_TESTS").is_ok() {
        eprintln!("Skipping MCP server test in CI environment");
        return;
    }
    
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "pmat"])
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
    
    // Kill the process after sending requests since MCP servers don't exit
    std::thread::sleep(std::time::Duration::from_secs(2));
    child.kill().expect("Failed to kill process");
    
    let output = child.wait_with_output().expect("Failed to wait for output");
    // Don't check exit status since we killed it

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":1"));
    assert!(stdout.contains("\"id\":2"));
    assert!(stdout.contains("\"tools\""));
    assert!(stdout.contains("\"prompts\""));
}
