#![cfg_attr(coverage_nightly, coverage(off))]
//! Analysis-related command definitions (complexity, churn, dead-code, etc.)

use crate::cli::AnalyzeCommands;
use anyhow::Result;

/// Command group for analysis operations (complexity, churn, dead-code, etc.)
pub struct AnalyzeCommandGroup;

impl Default for AnalyzeCommandGroup {
    fn default() -> Self {
        Self
    }
}

impl AnalyzeCommandGroup {
    /// Execute analysis command using modular handlers
    pub async fn execute(&self, cmd: AnalyzeCommands) -> Result<()> {
        // Delegate to analysis handlers which further delegate to specific modules
        crate::cli::handlers::analysis_handlers::route_analyze_command(cmd).await
    }
}
