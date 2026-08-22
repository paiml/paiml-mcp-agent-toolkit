//! R17-4 / KAIZEN-0191: `pmat serve` must fail loudly, not hang silently.
//!
//! Prior to this fix the command printed a misleading "Server ready!" banner
//! and then blocked on `ctrl_c().await` without binding a socket, returning
//! HTTP 000 for every request while exiting 0. These tests pin the honest-
//! failure contract so we never regress back to the silent footgun.

use std::process::Command;

/// Every invocation here is expected to exit, not to serve: an unimplemented
/// transport exits 2 synchronously, and `--transport http` refuses to start
/// without a token. If the handler ever regresses back to the old
/// `ctrl_c().await` stub, this call blocks and the outer cargo-test timeout
/// makes the regression loud.
fn run_serve(args: &[&str]) -> std::process::Output {
    run_serve_with_env(args, &[])
}

/// `run_serve`, plus environment the CALLER wants the child to start from.
///
/// The scrubs below are applied AFTER `extra`, deliberately: they are the
/// harness's invariants and no caller may defeat them. Applying them first
/// would let the overlay put back exactly the variable being removed — a
/// mistake worth naming, because the flag-efficacy harness made it and the
/// resulting fix compiled, ran, and changed nothing.
fn run_serve_with_env(args: &[&str], extra: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pmat"));
    cmd.arg("serve").args(args).env("RUST_LOG", "error");
    for (k, v) in extra {
        cmd.env(k, v);
    }
    cmd
        // `--transport http` is implemented: with a token in the environment it
        // would BIND A SOCKET and `output()` would block until killed. The test
        // must not depend on the developer's environment not having one.
        .env_remove("PMAT_MCP_HTTP_TOKEN")
        // MCP_VERSION makes pmat ignore the subcommand entirely and start the
        // stdio MCP server (src/bin/pmat.rs:41 — "Explicit MCP opt-in via env
        // var always wins", for Claude Desktop). `output()` closes stdin, that
        // server reads EOF and exits 0, and every assertion in this file about
        // an exit code then compares against the wrong process.
        //
        // This is not hypothetical. `pmat_serve_websocket_fails_loudly` failed
        // in the full-feature run with `left: Some(0), right: Some(2)` and
        // passed when run alone. Cargo runs a binary's tests as parallel THREADS
        // in one process, and two siblings set the variable process-wide:
        //   tests/modules/execution_mode.rs:24
        //   tests/modules/services_integration.rs:340
        // Both remove it afterwards, but a child spawned inside that window
        // inherits it. Measured directly:
        //   env -u MCP_VERSION  pmat serve --transport web-socket -> exit 2
        //   MCP_VERSION=1.0.0   pmat serve --transport web-socket -> exit 0, 0 bytes
        .env_remove("MCP_VERSION")
        .output()
        .expect("failed to spawn pmat binary")
}

/// Bare `pmat serve` means `--transport http`, which is IMPLEMENTED — so the
/// contract it must keep is no longer "exit 2, unimplemented" but "refuse to
/// start unauthenticated, and say why".
///
/// This test asserted exit code 2 and the words "not yet implemented" long
/// after the streamable-HTTP transport shipped (EV-6, #999): the binary was
/// exiting 4 with a token diagnostic, and the test went on failing unnoticed
/// because `tests/all.rs` is not part of the `--lib` suite CI runs. Pinning
/// "there is no HTTP server" is the same defect as `serve --help` saying so.
///
/// The *value* 2 was the wrong part, not the fact that a value was checked.
/// Replacing `assert_eq!(code, Some(2))` with a bare `!success()` threw away a
/// contract callers depend on, so the exit code is asserted again below — at
/// the code this path actually has.
#[test]
fn pmat_serve_without_a_token_refuses_to_start_and_says_why() {
    let out = run_serve(&[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // A missing token is a misconfiguration, and pmat has a documented code for
    // that: `ExitCode::ConfigurationError` = 4 (src/bin/pmat.rs). Measured 4 in
    // both the default and the `--features mcp-http` build. `!success()` alone
    // would keep passing if the code drifted to 1 (GeneralError) — which is
    // precisely what happens if the diagnostic is ever reworded past the
    // "config" substring `categorize_error` matches on.
    assert_eq!(
        out.status.code(),
        Some(4),
        "refusing to start over a missing token is a configuration error \
         (exit 4), got {:?}\nstderr: {stderr}",
        out.status
    );
    // The security property, not an exit-code literal: pmcp serves every
    // request when no auth provider is wired, so "no token" must mean "no
    // server", and the diagnostic must name the variable to set.
    assert!(
        stderr.contains("PMAT_MCP_HTTP_TOKEN"),
        "stderr must name the token variable, got: {stderr}"
    );
    // This asserted on `PMAT_PMCP_MCP=1` for as long as the hint named it —
    // an environment variable no code reads. The test did not catch that
    // because it pinned the string rather than the behaviour.
    for dead in ["PMAT_PMCP_MCP", "agent mcp-server"] {
        assert!(
            !stderr.contains(dead),
            "stderr must not name `{dead}`, which does not start this server, got: {stderr}"
        );
    }
}

/// An ambient `MCP_VERSION` must not be able to answer for `pmat serve`.
///
/// This is the flake that failed the 3.32.0 release dogfood: the full-feature
/// run reported `left: Some(0), right: Some(2)` for the websocket test, and the
/// same test passed when run alone. Cargo runs a binary's tests as parallel
/// THREADS in one process, so a sibling's process-wide `set_var` is visible to
/// any child spawned in that window — `execution_mode.rs:24` and
/// `services_integration.rs:340` both set `MCP_VERSION`, and both remove it
/// afterwards, which closes the window without closing the hole.
///
/// With the variable set, pmat ignores the subcommand and starts the stdio MCP
/// server; `output()` closes stdin; the server reads EOF and exits 0. Every
/// exit-code assertion in this file is then comparing against a different
/// program. Measured:
///
/// ```text
/// env -u MCP_VERSION  pmat serve --transport web-socket  -> exit 2
/// MCP_VERSION=1.0.0   pmat serve --transport web-socket  -> exit 0, 0 bytes
/// ```
///
/// The variable is passed HERE as explicit child environment rather than set on
/// this process, because setting it process-wide would recreate for every other
/// test the exact race this test exists to close.
#[test]
fn an_ambient_mcp_version_cannot_defeat_the_serve_contract() {
    let out = run_serve_with_env(&["--transport", "web-socket"], &[("MCP_VERSION", "1.0.0")]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "the harness must scrub MCP_VERSION so an unimplemented transport still \
         fails loudly; exit 0 means pmat ran the MCP stdio server instead and \
         hit EOF\nstderr: {stderr}"
    );
    assert!(
        stderr.contains("websocket"),
        "the refusal must still name the transport, got: {stderr}"
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

/// Echoing the request back is a property of the *unimplemented* path, so it
/// is exercised on a transport that is unimplemented. With the default
/// (`http`) transport this asserted exit 2 against a binary that now starts a
/// server — or, without a token, exits 4 with the auth refusal.
#[test]
fn pmat_serve_echoes_host_and_port() {
    let out = run_serve(&[
        "--transport",
        "web-socket",
        "--host",
        "0.0.0.0",
        "--port",
        "12345",
    ]);
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
