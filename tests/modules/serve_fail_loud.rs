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
    // This asserted on `PMAT_PMCP_MCP=1` for as long as the hint named it —
    // an environment variable no code reads. The test did not catch that
    // because it pinned the string rather than the behaviour, and because
    // `tests/all.rs` is not part of the `--lib` suite CI runs, so it went on
    // asserting the bug for two releases after the hint was corrected.
    assert!(
        stderr.contains("MCP_VERSION=1"),
        "stderr must point users to the working stdio transport, got: {stderr}"
    );
    for dead in ["PMAT_PMCP_MCP", "agent mcp-server"] {
        assert!(
            !stderr.contains(dead),
            "stderr must not name `{dead}`, which does not start this server, got: {stderr}"
        );
    }
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
fn pmat_serve_help_does_not_advertise_a_dead_route() {
    // `pmat serve --help` renders the doc comment on the `Serve` variant. That
    // comment recommended `pmat agent mcp-server` while the runtime hint two
    // files away had already been corrected, so the help text and the error
    // text disagreed about how to start an MCP server.
    let out = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["serve", "--help"])
        .output()
        .expect("failed to spawn pmat binary");
    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        help.contains("MCP_VERSION=1"),
        "serve help must name the working route, got: {help}"
    );
    for dead in ["PMAT_PMCP_MCP", "agent mcp-server"] {
        assert!(
            !help.contains(dead),
            "serve help must not name `{dead}`, got: {help}"
        );
    }
}

/// A non-zero exit must always be accompanied by a diagnostic.
///
/// `pmat agent mcp-server` sets `EarlyCliArgs::is_mcp_server`, which installs
/// `EnvFilter::new("off")`. The fatal error was logged with `tracing::error!`,
/// so the filter discarded it and the command exited 1 with both streams
/// empty — the user got a failure with no way to learn why.
#[test]
fn fatal_errors_are_never_silent() {
    let out = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args(["agent", "mcp-server"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to spawn pmat binary");

    // Built without `--features agent-daemon` (not in `default`), this must
    // fail. If the feature *is* enabled the command is a long-running server,
    // so only assert the contract when it actually exited non-zero.
    if out.status.success() {
        return;
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "a non-zero exit must explain itself on stderr; got an empty stream, \
         which is the silent-failure regression this test exists to catch"
    );
    assert!(
        stderr.contains("Error:"),
        "fatal diagnostic must be present, got: {stderr}"
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
