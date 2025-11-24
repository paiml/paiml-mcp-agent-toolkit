// Handler for Five Whys debug command
//
// REFACTOR PHASE: CLI integration

use crate::cli::DebugOutputFormat;
use crate::services::five_whys_analyzer::FiveWhysAnalyzer;
use crate::services::debug_formatters::{format_text, format_json, format_markdown};
use anyhow::Result;
use std::path::Path;

/// Handle pmat debug command - Five Whys root cause analysis
pub async fn handle_debug(
    issue: &str,
    depth: u8,
    format: DebugOutputFormat,
    output: Option<&Path>,
    path: &Path,
    _context: Option<&Path>,
    _auto_analyze: bool,
) -> Result<()> {
    // Create analyzer
    let analyzer = FiveWhysAnalyzer::new();

    // Run Five Whys analysis
    println!("🔍 Analyzing: {}", issue);
    println!("   Depth: {} iterations", depth);
    println!("   Path: {}", path.display());
    println!();

    let analysis = analyzer.analyze(issue, path, depth).await?;

    // Format output
    let formatted = match format {
        DebugOutputFormat::Text => format_text(&analysis)?,
        DebugOutputFormat::Json => format_json(&analysis)?,
        DebugOutputFormat::Markdown => format_markdown(&analysis)?,
    };

    // Write to file or stdout
    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        println!("✅ Analysis written to: {}", output_path.display());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_handle_debug_basic() {
        let result = handle_debug(
            "Test issue",
            5,
            DebugOutputFormat::Text,
            None,
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_debug_json_format() {
        let result = handle_debug(
            "Test issue",
            3,
            DebugOutputFormat::Json,
            None,
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_debug_markdown_format() {
        let result = handle_debug(
            "Test issue",
            3,
            DebugOutputFormat::Markdown,
            None,
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }
}
