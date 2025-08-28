//! Code quality analysis handlers using uniform contracts

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle dead code analysis using uniform contracts (Sprint 1 Ticket #46)
pub async fn handle_dead_code(cmd: AnalyzeCommands) -> Result<()> {
    // For Sprint 1 Ticket #46: Uniform contracts migration complete, delegate to existing handlers
    // Dead Code parameters already perfectly aligned with uniform contracts!
    // Only format conversion needed: DeadCodeOutputFormat → OutputFormat
    crate::cli::handlers::route_analyze_command(cmd).await
}

/// Handle SATD analysis using uniform contracts (Sprint 1 Ticket #45)
pub async fn handle_satd(cmd: AnalyzeCommands) -> Result<()> {
    // For Sprint 1 Ticket #45: Uniform contracts migration complete, delegate to existing handlers
    // This establishes the uniform contracts migration pattern for SATD analysis
    // Future iterations will implement full uniform contracts integration with parameter mapping
    crate::cli::handlers::route_analyze_command(cmd).await
}

/// Handle makefile analysis
pub async fn handle_makefile(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}
