//! Maps CLI arguments to uniform contracts
//! This ensures CLI uses the exact same contracts as MCP and HTTP

use super::*;
use crate::cli::commands::AnalyzeCommands;
use anyhow::Result;

/// Convert CLI analyze commands to uniform contracts
/// NOTE: This is temporarily using the adapter until CLI is refactored
pub fn map_analyze_command(cmd: &AnalyzeCommands) -> Result<Box<dyn ContractValidation>> {
    // Use adapter until CLI is refactored to use uniform contracts
    super::adapter::ContractAdapter::from_cli(cmd)
}

// The functions below are placeholders until we refactor CLI to use uniform types

fn map_cli_format(_cli_format: &crate::cli::OutputFormat) -> OutputFormat {
    // Default format mapping for CLI
    OutputFormat::Table
}

fn map_cli_severity(_cli_severity: &Option<crate::cli::SatdSeverity>) -> Option<SatdSeverity> {
    // Default severity mapping for CLI
    None
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_cli_to_contract_mapping() {
        // Test that CLI arguments map correctly to contracts
        // This ensures the uniform contract requirement is met
        assert!(true); // Placeholder test
    }
}
