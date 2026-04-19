//! R17-4 / KAIZEN-0191: `pmat serve` must fail loudly, not hang silently.
//!
//! Prior to this fix the command printed a misleading "Server ready!" banner
//! and then blocked on `ctrl_c().await` without binding a socket, returning
//! HTTP 000 for every request while exiting 0. These tests pin the honest-
//! failure contract so we never regress back to the silent footgun.

use std::process::Command;

fn run_serve(args: &[&str]) -> std::process::Output {
    // The `serve` handler calls `std::process::exit(2)` synchronously before
    // awaiting anything, so this call returns almost immediately. If the
    // handler ever regresses back to the old `ctrl_c().await` stub, this test
    // will hang and be killed by the outer cargo-test timeout, making the
    // regression loud.
    Command::new(env!("CARGO_BIN_EXE_pmat"))
        .arg("serve")
        .args(args)
        .env("RUST_LOG", "error")
        .output()
        .expect("failed to spawn pmat binary")
}

#[test]
fn pmat_serve_default_fails_with_exit_code_two() {
    let out = run_serve(&[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "pmat serve must exit with code 2 (misuse), got {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not yet implemented"),
        "stderr must clearly say HTTP is not implemented, got: {stderr}"
    );
    assert!(
        stderr.contains("PMAT_PMCP_MCP=1"),
        "stderr must point users to the working stdio transport, got: {stderr}"
    );
}

#[test]
fn pmat_serve_websocket_fails_loudly() {
    let out = run_serve(&["--transport", "web-socket"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("websocket"),
        "stderr must echo the requested transport, got: {stderr}"
    );
}

#[test]
fn pmat_serve_echoes_host_and_port() {
    let out = run_serve(&["--host", "0.0.0.0", "--port", "12345"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("0.0.0.0"),
        "stderr must echo requested host, got: {stderr}"
    );
    assert!(
        stderr.contains("12345"),
        "stderr must echo requested port, got: {stderr}"
    );
}

#[test]
fn pmat_serve_does_not_print_dishonest_ready_banner() {
    // Regression guard: the old stub printed "Server ready!" / "ready for
    // implementation" — both are prohibited now because they lie about state.
    let out = run_serve(&[]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("Server ready!"),
        "must not print 'Server ready!' — that was the dishonest stub banner"
    );
    assert!(
        !combined.contains("ready for implementation"),
        "must not print 'ready for implementation' — that was the dishonest stub banner"
    );
}
