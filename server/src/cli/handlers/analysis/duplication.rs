//! Code duplication analysis handlers

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle duplicates analysis
pub async fn handle_duplicates(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}

/// Handle name similarity analysis
pub async fn handle_name_similarity(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}
