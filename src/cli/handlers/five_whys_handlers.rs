// Handler for Five Whys debug command
//
// REFACTOR PHASE: CLI integration

use crate::cli::DebugOutputFormat;
use crate::services::debug_formatters::{format_json, format_markdown, format_text};
use crate::services::five_whys_analyzer::FiveWhysAnalyzer;
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    #[tokio::test]
    async fn test_handle_debug_depth_1() {
        let result = handle_debug(
            "Simple issue",
            1,
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
    async fn test_handle_debug_depth_10() {
        let result = handle_debug(
            "Complex issue requiring deep analysis",
            10,
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
    async fn test_handle_debug_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("analysis.txt");

        let result = handle_debug(
            "Test issue",
            3,
            DebugOutputFormat::Text,
            Some(&output_path),
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_handle_debug_json_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("analysis.json");

        let result = handle_debug(
            "Test issue",
            3,
            DebugOutputFormat::Json,
            Some(&output_path),
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify JSON is valid
        let content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn test_handle_debug_markdown_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("analysis.md");

        let result = handle_debug(
            "Test issue",
            3,
            DebugOutputFormat::Markdown,
            Some(&output_path),
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify markdown contains expected sections
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Five Whys") || content.contains("#"));
    }

    #[tokio::test]
    async fn test_handle_debug_with_different_issues() {
        let issues = vec![
            "Stack overflow in parser",
            "Memory leak detected",
            "Test failures on CI",
            "Performance regression",
            "Security vulnerability found",
        ];

        for issue in issues {
            let result = handle_debug(
                issue,
                3,
                DebugOutputFormat::Text,
                None,
                Path::new("."),
                None,
                false,
            )
            .await;

            assert!(result.is_ok(), "Failed for issue: {}", issue);
        }
    }

    #[tokio::test]
    async fn test_handle_debug_empty_issue() {
        let result = handle_debug(
            "",
            3,
            DebugOutputFormat::Text,
            None,
            Path::new("."),
            None,
            false,
        )
        .await;

        // Empty issue should be rejected - Five Whys requires an actual issue to analyze
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_debug_long_issue() {
        let long_issue = "This is a very long issue description that goes on and on and on and contains many details about the problem at hand including symptoms, context, and potential theories about what might be causing the issue in the first place. It also includes some technical details like error messages, stack traces, and system configuration information that might be relevant to understanding the root cause.";

        let result = handle_debug(
            long_issue,
            3,
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
    async fn test_handle_debug_special_characters() {
        let result = handle_debug(
            "Issue with special chars: <>&'\"!@#$%^*()[]{}",
            3,
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
    async fn test_handle_debug_unicode_issue() {
        let result = handle_debug(
            "Bug with emoji: 🐛 and unicode: こんにちは",
            3,
            DebugOutputFormat::Text,
            None,
            Path::new("."),
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }
}
