//! Dependency analysis handlers using uniform contracts

use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Handle DAG analysis
pub async fn handle_dag(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}

/// Handle graph metrics analysis
pub async fn handle_graph_metrics(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}

/// Handle symbol table analysis
pub async fn handle_symbol_table(cmd: AnalyzeCommands) -> Result<()> {
    // Route to existing working handler
    crate::cli::handlers::route_analyze_command(cmd).await
}
