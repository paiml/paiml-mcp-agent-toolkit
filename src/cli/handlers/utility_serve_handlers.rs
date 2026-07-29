//! Server command handlers for PMAT
//!
//! Extracted from utility_handlers.rs for file health compliance (CB-040).
//! Contains handle_serve and related server transport implementations.
//!
//! # Honest-failure policy (R17-4 / KAIZEN-0191)
//!
//! The `pmat serve` command previously printed a misleading "Server ready!"
//! banner and then hung on `ctrl_c().await` without binding any socket. This
//! produced HTTP 000 on every request while reporting exit code 0, which is a
//! classic D75 "exit 0 on error" regression.
//!
//! Per the remediation policy, the command now fails LOUD: it prints a clear
//! error to stderr and exits with code 2 (misuse) until real HTTP/WebSocket/SSE
//! transports are wired up. Honest failure is strictly better than a lying
//! success.
//!
//! If you need an MCP server today, use stdio transport via `MCP_VERSION=1
//! pmat` — see the `SimpleUnifiedServer` in `mcp_pmcp`. Do not add
//! `pmat agent mcp-server` back here: it is compiled out unless
//! `--features agent-daemon` is set, and it starts the agent-monitoring
//! server, not the 20 analysis tools.
#![cfg_attr(coverage_nightly, coverage(off))]

use anyhow::Result;

/// Exit code returned when `pmat serve` is invoked while HTTP transports are
/// not yet implemented. `2` is the conventional "misuse" code and matches the
/// D75 remediation guidance.
pub const SERVE_UNIMPLEMENTED_EXIT_CODE: i32 = 2;

/// Emit the honest-failure diagnostic to the given writer.
///
/// Extracted so tests can capture the exact bytes that would be printed to
/// stderr without having to shell out to the real binary.
pub fn write_serve_unimplemented_message<W: std::io::Write>(
    mut out: W,
    host: &str,
    port: u16,
    transport: &str,
) -> std::io::Result<()> {
    writeln!(out, "error: pmat serve HTTP transport not yet implemented")?;
    writeln!(
        out,
        "  requested: transport={transport} host={host} port={port}"
    )?;
    // Names only the route verified to serve the 20 analysis tools.
    //
    // Two earlier versions of this hint did not work. It first named
    // `PMAT_PMCP_MCP=1`, an environment variable nothing in the binary reads
    // (MCP mode is gated on `MCP_VERSION`, src/bin/pmat.rs). The fix for that
    // led with `pmat agent mcp-server`, which dogfooding then showed exits 1
    // with no output at all — it starts the separate agent-monitoring server,
    // not this one. A hint is only worth printing if it has been run.
    writeln!(
        out,
        "hint: use stdio MCP today — `MCP_VERSION=1 pmat` (serves the analysis tools over stdio)"
    )?;
    writeln!(
        out,
        "hint: follow KAIZEN-0191 for the HTTP/WebSocket/SSE wiring"
    )?;
    Ok(())
}

/// Handle serve command.
///
/// Prints a clear "not yet implemented" diagnostic to stderr and exits the
/// process with [`SERVE_UNIMPLEMENTED_EXIT_CODE`]. Never returns.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_serve(
    host: String,
    port: u16,
    _cors: bool,
    transport: crate::cli::commands::ServeTransport,
) -> Result<()> {
    let transport_label = match transport {
        crate::cli::commands::ServeTransport::Http => "http",
        crate::cli::commands::ServeTransport::WebSocket => "websocket",
        crate::cli::commands::ServeTransport::HttpSse => "http-sse",
        crate::cli::commands::ServeTransport::Both => "http+websocket",
        crate::cli::commands::ServeTransport::All => "http+websocket+sse",
    };

    let _ = write_serve_unimplemented_message(std::io::stderr(), &host, port, transport_label);
    std::process::exit(SERVE_UNIMPLEMENTED_EXIT_CODE);
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_contains_not_yet_implemented() {
        let mut buf = Vec::new();
        write_serve_unimplemented_message(&mut buf, "127.0.0.1", 8080, "http").unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.contains("not yet implemented"),
            "stderr message must clearly state the feature is unimplemented, got: {s}"
        );
    }

    #[test]
    fn message_includes_requested_parameters() {
        let mut buf = Vec::new();
        write_serve_unimplemented_message(&mut buf, "0.0.0.0", 9000, "http-sse").unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("0.0.0.0"), "must echo requested host, got: {s}");
        assert!(s.contains("9000"), "must echo requested port, got: {s}");
        assert!(
            s.contains("http-sse"),
            "must echo requested transport, got: {s}"
        );
    }

    #[test]
    fn message_points_to_stdio_workaround() {
        let mut buf = Vec::new();
        write_serve_unimplemented_message(&mut buf, "127.0.0.1", 8080, "http").unwrap();
        let s = String::from_utf8(buf).unwrap();
        // Must name a route that actually works. This previously asserted on
        // `PMAT_PMCP_MCP=1`, an environment variable nothing in the binary
        // reads — so the test pinned advice that could not work.
        assert!(
            s.contains("MCP_VERSION=1"),
            "must point users at the working stdio transport, got: {s}"
        );
        assert!(
            !s.contains("PMAT_PMCP_MCP"),
            "must not name an environment variable no code reads, got: {s}"
        );
        assert!(
            !s.contains("agent mcp-server"),
            "must not name `pmat agent mcp-server`, which exits 1 without \
             serving these tools, got: {s}"
        );
    }

    #[test]
    fn exit_code_is_misuse_convention() {
        assert_eq!(SERVE_UNIMPLEMENTED_EXIT_CODE, 2);
    }
}
