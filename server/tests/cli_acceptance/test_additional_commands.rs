//! CLI Acceptance Tests - Additional Commands
//!
//! Tests for additional pmat CLI commands following the cli-acceptance-testing.md specification.
//! Covers refactor, quality-gate, tdg, qdd, report, serve, and other specialized commands.

use crate::cli_acceptance::helpers::cli_test_runner::{CliTestRunner, TestValidators, OutputFormat};
use std::time::Duration;
use anyhow::Result;

/// Test refactor command functionality
#[tokio::test]
async fn test_refactor_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test refactor help
    let result = runner.run_success(&["refactor", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Refactor code"));
    
    // Test refactor auto subcommand help
    let result = runner.run_success(&["refactor", "auto", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Automatic refactoring"));
    
    // Test refactor auto on a file
    let result = runner.run_success(&["refactor", "auto", "--file", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    Ok(())
}

/// Test quality-gate command functionality
#[tokio::test]
async fn test_quality_gate_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test quality-gate help
    let result = runner.run_success(&["quality-gate", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Quality gate"));
    
    // Test quality gate on a file
    let result = runner.run_success(&["quality-gate", "--file", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    // Test quality gate with profile
    let result = runner.run_success(&["quality-gate", "--file", "src/main.rs", "--profile", "standard"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test tdg command functionality
#[tokio::test]
async fn test_tdg_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test tdg help
    let result = runner.run_success(&["tdg", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Technical Debt Grading"));
    
    // Test basic TDG analysis
    let result = runner.run_success(&["tdg", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test TDG with components
    let result = runner.run_success(&["tdg", "src/main.rs", "--include-components"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test TDG dashboard help
    let result = runner.run_success(&["tdg", "dashboard", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("dashboard"));
    
    // Test TDG storage help
    let result = runner.run_success(&["tdg", "storage", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("storage"));
    
    Ok(())
}

/// Test qdd command functionality
#[tokio::test]
async fn test_qdd_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test qdd help
    let result = runner.run_success(&["qdd", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Quality-Driven Development"));
    
    // Test qdd create help
    let result = runner.run_success(&["qdd", "create", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Create"));
    
    // Test qdd validate help
    let result = runner.run_success(&["qdd", "validate", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Validate"));
    
    // Test qdd refactor help
    let result = runner.run_success(&["qdd", "refactor", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Refactor"));
    
    Ok(())
}

/// Test report command functionality
#[tokio::test]
async fn test_report_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test report help
    let result = runner.run_success(&["report", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Generate reports"));
    
    // Test basic report generation
    let result = runner.run_success(&["report", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    // Test report with format
    let result = runner.run_success(&["report", "src/", "--format", "json"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    TestValidators::assert_output_format(&result, OutputFormat::Json)?;
    
    Ok(())
}

/// Test serve command functionality
#[tokio::test]
async fn test_serve_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test serve help
    let result = runner.run_success(&["serve", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Start server"));
    
    // Note: Cannot easily test actual server startup without blocking, so test help and validation
    
    Ok(())
}

/// Test context command functionality
#[tokio::test]
async fn test_context_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test context help
    let result = runner.run_success(&["context", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Context"));
    
    // Test context generation
    let result = runner.run_success(&["context", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test demo command functionality
#[tokio::test]
async fn test_demo_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test demo help
    let result = runner.run_success(&["demo", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Demo"));
    
    // Note: Demo command may require special setup, so primarily test help
    
    Ok(())
}

/// Test enforce command functionality
#[tokio::test]
async fn test_enforce_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test enforce help
    let result = runner.run_success(&["enforce", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Enforce"));
    
    // Test enforce complexity
    let result = runner.run_success(&["enforce", "complexity", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    Ok(())
}

/// Test roadmap command functionality
#[tokio::test]
async fn test_roadmap_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test roadmap help
    let result = runner.run_success(&["roadmap", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Roadmap"));
    
    // Test roadmap generation
    let result = runner.run_success(&["roadmap", "generate"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    Ok(())
}

/// Test test command functionality
#[tokio::test]
async fn test_test_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test test help
    let result = runner.run_success(&["test", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Test"));
    
    // Test test execution (may fail if no tests, but should handle gracefully)
    let result = runner.run_command(&["test", "."])?;
    TestValidators::assert_performance(&result, Duration::from_secs(60))?;
    
    Ok(())
}

/// Test memory command functionality
#[tokio::test]
async fn test_memory_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test memory help
    let result = runner.run_success(&["memory", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Memory"));
    
    // Test memory analysis
    let result = runner.run_success(&["memory", "status"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test cache command functionality
#[tokio::test]
async fn test_cache_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test cache help
    let result = runner.run_success(&["cache", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Cache"));
    
    // Test cache status
    let result = runner.run_success(&["cache", "status"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    // Test cache clear
    let result = runner.run_success(&["cache", "clear"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test telemetry command functionality
#[tokio::test]
async fn test_telemetry_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test telemetry help
    let result = runner.run_success(&["telemetry", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Telemetry"));
    
    // Test telemetry status
    let result = runner.run_success(&["telemetry", "status"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test config command functionality
#[tokio::test]
async fn test_config_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test config help
    let result = runner.run_success(&["config", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Configuration"));
    
    // Test config show
    let result = runner.run_success(&["config", "show"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test agent command functionality
#[tokio::test]
async fn test_agent_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test agent help
    let result = runner.run_success(&["agent", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Agent"));
    
    // Test agent start help
    let result = runner.run_success(&["agent", "start", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("start"));
    
    Ok(())
}

/// Test mcp command functionality
#[tokio::test]
async fn test_mcp_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test mcp help
    let result = runner.run_success(&["mcp", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("MCP"));
    
    // Test mcp serve help
    let result = runner.run_success(&["mcp", "serve", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("serve"));
    
    Ok(())
}

/// Test pdmt-todos command functionality
#[tokio::test]
async fn test_pdmt_todos_command() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test pdmt-todos help
    let result = runner.run_success(&["pdmt-todos", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("PDMT"));
    
    // Test pdmt-todos with simple requirement
    let result = runner.run_success(&["pdmt-todos", "Test requirement", "--granularity", "medium", "--seed", "42"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    
    Ok(())
}

/// Test command flag combinations
#[tokio::test]
async fn test_command_flag_combinations() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test verbose flag with various commands
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--verbose"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test debug flag
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--debug"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test mode flag
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--mode", "cli"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test format flag combinations
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--format", "json", "--verbose"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    TestValidators::assert_output_format(&result, OutputFormat::Json)?;
    
    Ok(())
}

/// Test command output consistency
#[tokio::test]
async fn test_command_output_consistency() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // All commands should produce consistent output format for JSON
    let json_commands = [
        vec!["analyze", "complexity", "src/main.rs", "--format", "json"],
        vec!["report", "src/", "--format", "json"],
        vec!["tdg", "src/main.rs", "--format", "json"],
    ];
    
    for cmd in &json_commands {
        let result = runner.run_success(cmd)?;
        TestValidators::assert_performance(&result, Duration::from_secs(30))?;
        // Should be valid JSON (if supported)
        if result.stdout_text.trim().starts_with('{') || result.stdout_text.trim().starts_with('[') {
            TestValidators::assert_output_format(&result, OutputFormat::Json)?;
        }
    }
    
    Ok(())
}

/// Test error handling across commands
#[tokio::test]
async fn test_cross_command_error_handling() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    let commands_requiring_files = [
        vec!["analyze", "complexity"],
        vec!["quality-gate", "--file"],
        vec!["refactor", "auto", "--file"],
        vec!["tdg"],
    ];
    
    for cmd in &commands_requiring_files {
        // Test with nonexistent file
        let mut test_cmd = cmd.clone();
        test_cmd.push("nonexistent_file.rs");
        
        let result = runner.run_failure(&test_cmd)?;
        TestValidators::assert_performance(&result, Duration::from_secs(5))?;
        // Should have meaningful error message
        assert!(
            result.stderr_text.contains("file") || 
            result.stderr_text.contains("found") ||
            result.stderr_text.contains("exist"),
            "Command {:?} should provide meaningful error for nonexistent file", cmd
        );
    }
    
    Ok(())
}

/// Test performance across command categories
#[tokio::test]
async fn test_cross_command_performance() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Quick commands should be very fast
    let quick_commands = [
        vec!["--version"],
        vec!["--help"],
        vec!["analyze", "complexity", "--help"],
        vec!["tdg", "--help"],
    ];
    
    for cmd in &quick_commands {
        let result = runner.run_success(cmd)?;
        TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    }
    
    // Analysis commands should be reasonably fast
    let analysis_commands = [
        vec!["analyze", "complexity", "src/main.rs"],
        vec!["tdg", "src/main.rs"],
        vec!["quality-gate", "--file", "src/main.rs"],
    ];
    
    for cmd in &analysis_commands {
        let result = runner.run_success(cmd)?;
        TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test full workflow with multiple commands
    #[tokio::test]
    async fn test_full_command_workflow() -> Result<()> {
        let runner = CliTestRunner::new()?;
        let project_path = runner.create_sample_project()?;
        std::env::set_current_dir(&project_path)?;
        
        // Run a complete analysis workflow
        let result = runner.run_success(&["analyze", "complexity", "src/main.rs"])?;
        assert!(result.exit_code == 0);
        
        let result = runner.run_success(&["tdg", "src/main.rs"])?;
        assert!(result.exit_code == 0);
        
        let result = runner.run_success(&["quality-gate", "--file", "src/main.rs"])?;
        assert!(result.exit_code == 0);
        
        let result = runner.run_success(&["report", "src/"])?;
        assert!(result.exit_code == 0);
        
        Ok(())
    }
}