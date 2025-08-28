//! ML and predictive analysis handlers

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle defect prediction analysis
pub async fn handle_defect_prediction(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}
