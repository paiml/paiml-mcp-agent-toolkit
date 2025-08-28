//! Technical debt analysis handlers

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle TDG analysis
pub async fn handle_tdg(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}
