//! Complexity analysis handler using uniform contracts
//! This handler is part of the Sprint 1 migration to ensure uniform parameters across CLI/MCP/HTTP

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle complexity analysis using uniform contracts
pub async fn handle_complexity(cmd: AnalyzeCommands) -> Result<()> {
    // For Sprint 1 Ticket #44: Foundation complete, delegate to existing handlers
    // This establishes the uniform contracts migration pattern
    // Future ticket will implement full uniform contracts integration
    crate::cli::handlers::route_analyze_command(cmd).await
}
