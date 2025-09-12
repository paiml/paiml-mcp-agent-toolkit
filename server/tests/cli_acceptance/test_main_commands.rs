//! CLI Acceptance Tests - Main Commands
//!
//! Tests for top-level pmat CLI commands following the cli-acceptance-testing.md specification.
//! Ensures 100% coverage of main commands with proper error handling and performance validation.

use crate::cli_acceptance::helpers::cli_test_runner::{
    CliTestRunner, TestValidators,
};
use anyhow::Result;
use std::time::Duration;

/// Test the --version flag
#[tokio::test]
async fn test_version_flag() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test short version
    let result = runner.run_success(&["--version"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(1))?;
    assert!(result.stdout_text.contains("pmat"));
    assert!(result.stdout_text.contains("2.79.0"));

    // Test long version
    let result = runner.run_success(&["-V"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(1))?;
    assert!(result.stdout_text.contains("pmat"));

    Ok(())
}

/// Test the --help flag
#[tokio::test]
async fn test_help_flag() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test main help
    let result = runner.run_success(&["--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result
        .stdout_text
        .contains("Professional project quantitative"));
    assert!(result.stdout_text.contains("Commands:"));
    assert!(result.stdout_text.contains("generate"));
    assert!(result.stdout_text.contains("analyze"));

    // Test short help
    let result = runner.run_success(&["-h"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Usage:"));

    Ok(())
}

/// Test generate command basic functionality
#[tokio::test]
async fn test_generate_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test generate help
    let result = runner.run_success(&["generate", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Generate a single template"));

    // Test generate without arguments (should show help or error)
    let result = runner.run_command(&["generate"])?;
    // Either succeeds with help or fails with usage message
    if result.exit_code != 0 {
        assert!(result.stderr_text.contains("Usage") || result.stderr_text.contains("required"));
    }

    Ok(())
}

/// Test scaffold command basic functionality  
#[tokio::test]
async fn test_scaffold_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test scaffold help
    let result = runner.run_success(&["scaffold", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Scaffold complete project"));

    Ok(())
}

/// Test list command functionality
#[tokio::test]
async fn test_list_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test list help
    let result = runner.run_success(&["list", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("List available templates"));

    // Test actual list execution
    let result = runner.run_success(&["list"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    // Should produce some output (templates or empty list)

    Ok(())
}

/// Test search command functionality
#[tokio::test]
async fn test_search_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test search help
    let result = runner.run_success(&["search", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Search templates"));

    Ok(())
}

/// Test validate command functionality
#[tokio::test]
async fn test_validate_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test validate help
    let result = runner.run_success(&["validate", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Validate template parameters"));

    Ok(())
}

/// Test diagnose command functionality
#[tokio::test]
async fn test_diagnose_command() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test diagnose help
    let result = runner.run_success(&["diagnose", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Run self-diagnostics"));

    // Test actual diagnose execution
    let result = runner.run_success(&["diagnose"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    // Diagnose should complete and show system status

    Ok(())
}

/// Test global flags with commands
#[tokio::test]
async fn test_global_flags() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test verbose flag
    let result = runner.run_success(&["diagnose", "--verbose"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;

    // Test debug flag
    let result = runner.run_success(&["diagnose", "--debug"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;

    // Test mode flag
    let result = runner.run_success(&["diagnose", "--mode", "cli"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;

    Ok(())
}

/// Test invalid command handling
#[tokio::test]
async fn test_invalid_commands() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test completely invalid command
    let result = runner.run_failure(&["nonexistent-command"])?;
    TestValidators::assert_exit_code(&result, 1)?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stderr_text.contains("error") || result.stderr_text.contains("invalid"));

    // Test invalid flag
    let result = runner.run_failure(&["--invalid-flag"])?;
    TestValidators::assert_exit_code(&result, 1)?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;

    Ok(())
}

/// Test command help consistency
#[tokio::test]
async fn test_help_system_consistency() -> Result<()> {
    let runner = CliTestRunner::new()?;

    let commands = [
        "generate",
        "scaffold",
        "list",
        "search",
        "validate",
        "context",
        "analyze",
        "qdd",
        "demo",
        "quality-gate",
        "report",
        "serve",
        "diagnose",
        "enforce",
        "refactor",
        "roadmap",
        "test",
        "memory",
        "cache",
        "telemetry",
        "config",
        "agent",
        "tdg",
    ];

    for command in &commands {
        // Each command should have help available
        let result = runner.run_success(&[command, "--help"])?;
        TestValidators::assert_performance(&result, Duration::from_secs(5))?;

        // Help should contain usage information
        assert!(
            result.stdout_text.contains("Usage:")
                || result.stdout_text.contains("USAGE:")
                || result.stdout_text.to_lowercase().contains("usage"),
            "Command '{}' help does not contain usage information",
            command
        );
    }

    Ok(())
}

/// Test error message quality
#[tokio::test]
async fn test_error_message_quality() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Test missing required argument
    let result = runner.run_failure(&["analyze", "complexity"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    // Error should be user-friendly and actionable
    assert!(
        result.stderr_text.contains("required")
            || result.stderr_text.contains("missing")
            || result.stderr_text.contains("Usage"),
        "Error message should be user-friendly: {}",
        result.stderr_text
    );

    Ok(())
}

/// Test performance requirements for quick commands
#[tokio::test]
async fn test_performance_quick_commands() -> Result<()> {
    let runner = CliTestRunner::new()?;

    // Version and help should be very fast
    let result = runner.run_success(&["--version"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(1))?;

    let result = runner.run_success(&["--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(1))?;

    // Help for subcommands should be reasonably fast
    let result = runner.run_success(&["analyze", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test command chaining and workflow
    #[tokio::test]
    async fn test_command_workflow() -> Result<()> {
        let runner = CliTestRunner::new()?;
        let project_path = runner.create_sample_project()?;

        // Change to project directory for analysis
        std::env::set_current_dir(&project_path)?;

        // Run a simple analysis workflow
        let result = runner.run_success(&["diagnose"])?;
        TestValidators::assert_performance(&result, Duration::from_secs(30))?;

        // The diagnose should complete successfully
        assert!(result.exit_code == 0);

        Ok(())
    }

    /// Test with various project structures
    #[tokio::test]
    async fn test_different_project_types() -> Result<()> {
        let runner = CliTestRunner::new()?;

        // Test with empty directory
        let empty_dir = runner.workspace_path().join("empty_project");
        std::fs::create_dir_all(&empty_dir)?;

        std::env::set_current_dir(&empty_dir)?;

        // Commands should handle empty directories gracefully
        let result = runner.run_command(&["diagnose"])?;
        // Should either succeed or fail gracefully
        TestValidators::assert_performance(&result, Duration::from_secs(30))?;

        Ok(())
    }
}
