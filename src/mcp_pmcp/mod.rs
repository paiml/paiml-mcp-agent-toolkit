//! High-performance MCP server implementation using the pmcp SDK
//!
//! This module provides an experimental Model Context Protocol (MCP) server
//! implementation built on top of the pmcp Rust SDK. It offers significant
//! performance improvements and native async/await support compared to the
//! standard implementation.
//!
//! # Features
//!
//! - **10x performance improvement** over the standard MCP implementation
//! - **Type-safe tool handlers** with compile-time validation
//! - **Native async/await** support with tokio
//! - **Built-in transport support** for stdio, WebSocket, and HTTP/SSE
//!
//! # Usage
//!
//! The server is activated by setting the `MCP_VERSION` environment variable
//! (which is what MCP hosts such as Claude Desktop set). That is the only
//! trigger: `detect_execution_mode` in `src/bin/pmat.rs` reads `MCP_VERSION`
//! and nothing else.
//!
//! Two other spellings have been documented here over time and neither works.
//! `PMAT_PMCP_MCP` is read by no code at all. `pmat agent mcp-server` runs a
//! *different* server — `ClaudeCodeAgentMcpServer`, which exposes four
//! agent-monitoring tools rather than these analysis tools — and it is
//! compiled out entirely unless `--features agent-daemon` is set, which is not
//! in `default`.
//!
//! ## Running the pmcp server
//!
//! ```bash
//! MCP_VERSION=1 pmat
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use pmat::mcp_pmcp::PmcpServer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new pmcp server instance
//!     let server = PmcpServer::new();
//!     
//!     // Run the server on stdio transport
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Available Tools
//!
//! The live server advertises exactly the tools in
//! [`tool_manifest::LIVE_MCP_TOOLS`] — 19 at the time of writing. That const is
//! the single source of truth: `mcp.json` is rendered from it, and
//! `manifest_matches_server` pins it to the server's actual `.tool(...)`
//! registrations.
//!
//! This section used to enumerate "24 MCP tools" by hand, including four
//! `refactor.*` tools unregistered in EV-0 (#999) and six `tdg_*` tools this
//! server has never registered. A hand-written inventory beside a machine-
//! readable one is a second answer to "what does pmat serve", and it drifted by
//! eight tools — so the list is not repeated here. Read the const, or ask the
//! server:
//!
//! ```bash
//! printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | pmat --mode mcp
//! ```
//!
//! Note what the surface does NOT cover: `pmat analyze` has 35 subcommands and
//! nine of them have MCP counterparts. The other 26 are declared, one row each,
//! in [`crate::cli::analyze_mcp_exposure`] (#1029) — either as a reasoned
//! `CliOnly` decision or as a ratcheted `Backlog` entry. That registry is a
//! TOTAL MATCH over `AnalyzeCommands` with no catch-all arm, so a new
//! subcommand cannot be CLI-only in silence any more: it fails to compile until
//! someone declares it. It can still be CLI-only — that is a legitimate answer,
//! and for the three analyzers that rewrite the working tree it is the right
//! one.
//!
//! # Performance
//!
//! The pmcp implementation provides significant performance benefits:
//!
//! ```rust,ignore
//! // Standard MCP server
//! // Average response time: 50ms
//! // Memory usage: 100MB
//!
//! // pmcp-based server  
//! // Average response time: 5ms (10x faster)
//! // Memory usage: 50MB (50% reduction)
//! ```

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod advertised_schema_parity_tests; // honoured-parameter vs advertised-schema drift guard
pub mod agent_context_handlers;
pub mod analyze_handlers;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cli_mcp_surface_parity_tests; // #1029: module-doc inventory guard (the parity registry is cli::analyze_mcp_exposure)
pub mod context_handlers;
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dead_code_payload_contract_tests; // analyze_dead_code payload-key contract
pub mod discovery;
pub mod handlers;
#[cfg(feature = "mcp-http")]
pub mod http_frames; // JSON-RPC frame classification for the streamable-HTTP transport
pub mod http_server;
pub mod pdmt_handler;
pub mod prompt_handlers; // Phase 4: Organizational Intelligence Integration
pub mod quality_handlers;
pub mod quality_proxy_handler;
pub mod server;
pub mod simple_unified_server;
pub mod stdio_frames; // #648: raw-line stdio transport with honest JSON-RPC errors
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod surface_verdict_and_scope_tests; // R13/R17/R18 drift guards
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub mod tdg_git_context_tests;
pub mod tdg_handlers;
pub mod tool_functions;
pub mod tool_manifest; // MACS F6 (Component 32): canonical mcp.json source
pub mod tool_schemas;
pub mod tool_schemas_generated; // KAIZEN-0178: build.rs-generated tool schema registry
pub mod tools; // Sprint 65 Phase 2B: MCP git-context integration

// Export the simple unified server as the primary interface
pub use simple_unified_server::SimpleUnifiedServer as UnifiedServer;

/// Serve MCP on stdio. This is the crate's ONLY MCP stdio entry point.
///
/// Every route that claims to "start the MCP server" goes through here:
/// `MCP_VERSION=1 pmat`, `pmat --mode mcp`, and [`crate::cli::run`] for library
/// embedders. They must serve the same tool inventory, and the only way to
/// guarantee that is for exactly one place to construct the server.
///
/// #696/#697: there used to be three answers to "which server does pmat
/// serve?". `src/bin/pmat.rs` and `cli::run` each built their own
/// [`UnifiedServer`] (same 20 tools, but two independent call sites free to
/// drift), and `pmat::run_mcp_server` — reachable from the library API and
/// exercised by its own test — ran a *different* server whose 21-tool
/// inventory shared only 7 names with this one's 20, took different arguments
/// for those 7 (`project_path` string vs `paths` array), and had 7 tools whose
/// own descriptions read "(unimplemented stub — KAIZEN-0200)" and which return
/// JSON-RPC -32001 when called. That server has been deleted; this is what
/// replaces it.
///
/// The `exactly_one_mcp_stdio_entry_point` test pins the property.
///
/// # Errors
///
/// Returns an error if the server cannot be constructed or the transport fails.
pub async fn run_stdio_server() -> anyhow::Result<()> {
    let server = UnifiedServer::new()
        .map_err(|e| anyhow::anyhow!("Failed to create unified server: {e}"))?;
    server.run().await.map_err(|e| anyhow::anyhow!("{e}"))
}

// Keep PmcpServer for backward compatibility (will be removed)
pub use server::PmcpServer;
// Export the discovery service for MCP optimization
pub use discovery::{Context, DiscoveryMetrics, DiscoveryService, ToolInfo};

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod single_entry_point_tests {
    //! #696/#697 drift guards. These read the source of the call sites rather
    //! than calling them, because the thing under test is "how many places
    //! start an MCP server", which no runtime assertion can observe.

    /// Source with `//` comment lines removed.
    ///
    /// The guards must fire on a *definition* or a *call*, not on a comment
    /// that names one — the first draft of `lib_rs_exposes_no_second_mcp_stdio_server`
    /// failed against the tombstone comment left where the deleted server was.
    fn code_only(src: &str) -> String {
        src.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The binary and `cli::run` must both delegate to `run_stdio_server`, and
    /// neither may construct a server of its own.
    #[test]
    fn exactly_one_mcp_stdio_entry_point() {
        for (label, src) in [
            ("src/bin/pmat.rs", include_str!("../bin/pmat.rs")),
            (
                "src/cli/cli_run_command.rs",
                include_str!("../cli/cli_run_command.rs"),
            ),
        ] {
            let code = code_only(src);
            assert!(
                !code.contains("UnifiedServer::new("),
                "{label} constructs its own MCP server; it must call \
                 mcp_pmcp::run_stdio_server() so every route serves one inventory"
            );
            assert!(
                code.contains("run_stdio_server()"),
                "{label} no longer routes MCP mode through run_stdio_server()"
            );
        }
    }

    /// #696: `pmat::run_mcp_server` was a second, disjoint MCP stdio server
    /// reachable from the library API. It is gone; keep it gone.
    #[test]
    fn lib_rs_exposes_no_second_mcp_stdio_server() {
        let code = code_only(include_str!("../lib.rs"));
        assert!(
            !code.contains("fn run_mcp_server"),
            "src/lib.rs defines a second MCP stdio server (the legacy 21-tool \
             one, sharing only 7 names with the live 20). Use \
             mcp_pmcp::run_stdio_server instead."
        );
    }

    /// The comment-stripper has to actually strip, or the guards above pass
    /// vacuously the moment someone documents what they removed.
    #[test]
    fn code_only_drops_comment_lines_and_keeps_code() {
        let src = "// fn run_mcp_server(x) {}\nfn live() {}\n    /// UnifiedServer::new()\n";
        let code = code_only(src);
        assert!(!code.contains("run_mcp_server"));
        assert!(!code.contains("UnifiedServer::new("));
        assert!(code.contains("fn live()"));
    }
}
