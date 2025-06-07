//! Complexity analysis handlers

use anyhow::Result;
use std::path::PathBuf;
use crate::cli::AnalyzeCommands;

/// Handle complexity analysis command
pub async fn handle_complexity(cmd: AnalyzeCommands) -> Result<()> {
    if let AnalyzeCommands::Complexity {
        project_path,
        toolchain,
        format,
        output,
        max_cyclomatic,
        max_cognitive,
        include,
        watch,
        top_files,
    } = cmd {
        // Delegate to the original implementation for now
        // This will be refactored to reduce complexity
        super::super::super::handle_analyze_complexity(
            project_path,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
        ).await
    } else {
        unreachable!("handle_complexity called with non-complexity command")
    }
}