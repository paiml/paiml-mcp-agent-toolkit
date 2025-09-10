//! CLI Acceptance Tests - Analyze Commands
//!
//! Tests for all analyze subcommands following the cli-acceptance-testing.md specification.
//! Ensures 100% coverage of analyze functionality with proper error handling and performance validation.

use crate::cli_acceptance::helpers::cli_test_runner::{CliTestRunner, TestValidators, OutputFormat};
use std::time::Duration;
use anyhow::Result;

/// Test analyze complexity command
#[tokio::test]
async fn test_analyze_complexity() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test complexity help
    let result = runner.run_success(&["analyze", "complexity", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Analyze code complexity"));
    
    // Test basic complexity analysis
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    TestValidators::assert_output_format(&result, OutputFormat::Human)?;
    
    // Test JSON output format
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--format", "json"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    TestValidators::assert_output_format(&result, OutputFormat::Json)?;
    
    // Test top files option
    let result = runner.run_success(&["analyze", "complexity", "--top-files", "5"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    Ok(())
}

/// Test analyze dead-code command
#[tokio::test]
async fn test_analyze_dead_code() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test dead-code help
    let result = runner.run_success(&["analyze", "dead-code", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Analyze dead code"));
    
    // Test basic dead code analysis
    let result = runner.run_success(&["analyze", "dead-code", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    // Test with different formats
    let result = runner.run_success(&["analyze", "dead-code", "src/", "--format", "json"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    TestValidators::assert_output_format(&result, OutputFormat::Json)?;
    
    Ok(())
}

/// Test analyze satd command
#[tokio::test]
async fn test_analyze_satd() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test satd help
    let result = runner.run_success(&["analyze", "satd", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Self-Admitted Technical Debt"));
    
    // Test basic SATD analysis
    let result = runner.run_success(&["analyze", "satd", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test with file-specific analysis
    let result = runner.run_success(&["analyze", "satd", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    
    Ok(())
}

/// Test analyze deep-context command
#[tokio::test]
async fn test_analyze_deep_context() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test deep-context help
    let result = runner.run_success(&["analyze", "deep-context", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Deep context analysis"));
    
    // Test basic deep context analysis
    let result = runner.run_success(&["analyze", "deep-context", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    Ok(())
}

/// Test analyze tdg command
#[tokio::test]
async fn test_analyze_tdg() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test tdg help
    let result = runner.run_success(&["analyze", "tdg", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Technical Debt Grading"));
    
    // Test basic TDG analysis
    let result = runner.run_success(&["analyze", "tdg", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test with include-components flag
    let result = runner.run_success(&["analyze", "tdg", "src/main.rs", "--include-components"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    Ok(())
}

/// Test analyze entropy command
#[tokio::test]
async fn test_analyze_entropy() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test entropy help
    let result = runner.run_success(&["analyze", "entropy", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Entropy analysis"));
    
    // Test basic entropy analysis
    let result = runner.run_success(&["analyze", "entropy", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    // Test with severity filtering
    let result = runner.run_success(&["analyze", "entropy", "src/", "--min-severity", "medium"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    // Test with top violations
    let result = runner.run_success(&["analyze", "entropy", "src/", "--top-violations", "10"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test analyze duplicates command
#[tokio::test]
async fn test_analyze_duplicates() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test duplicates help
    let result = runner.run_success(&["analyze", "duplicates", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Duplicate code analysis"));
    
    // Test basic duplicates analysis
    let result = runner.run_success(&["analyze", "duplicates", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    Ok(())
}

/// Test analyze lint-hotspot command
#[tokio::test]
async fn test_analyze_lint_hotspot() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test lint-hotspot help
    let result = runner.run_success(&["analyze", "lint-hotspot", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Lint hotspot analysis"));
    
    // Test basic lint hotspot analysis
    let result = runner.run_success(&["analyze", "lint-hotspot", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    // Test with top files option
    let result = runner.run_success(&["analyze", "lint-hotspot", "--top-files", "5"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test analyze big-o command
#[tokio::test]
async fn test_analyze_big_o() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test big-o help
    let result = runner.run_success(&["analyze", "big-o", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Big O complexity analysis"));
    
    // Test basic Big O analysis
    let result = runner.run_success(&["analyze", "big-o", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    Ok(())
}

/// Test analyze defect-prediction command
#[tokio::test]
async fn test_analyze_defect_prediction() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test defect-prediction help
    let result = runner.run_success(&["analyze", "defect-prediction", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Defect prediction analysis"));
    
    // Test basic defect prediction analysis
    let result = runner.run_success(&["analyze", "defect-prediction", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test analyze ml-analysis command
#[tokio::test]
async fn test_analyze_ml_analysis() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test ml-analysis help
    let result = runner.run_success(&["analyze", "ml-analysis", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Machine learning analysis"));
    
    // Test basic ML analysis
    let result = runner.run_success(&["analyze", "ml-analysis", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(45))?;
    
    Ok(())
}

/// Test analyze dependencies command
#[tokio::test]
async fn test_analyze_dependencies() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test dependencies help
    let result = runner.run_success(&["analyze", "dependencies", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Dependency analysis"));
    
    // Test basic dependencies analysis
    let result = runner.run_success(&["analyze", "dependencies", "."])?;
    TestValidators::assert_performance(&result, Duration::from_secs(30))?;
    
    Ok(())
}

/// Test analyze graph-metrics command
#[tokio::test]
async fn test_analyze_graph_metrics() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test graph-metrics help
    let result = runner.run_success(&["analyze", "graph-metrics", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Graph metrics analysis"));
    
    // Test basic graph metrics analysis
    let result = runner.run_success(&["analyze", "graph-metrics", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(25))?;
    
    Ok(())
}

/// Test analyze name-similarity command
#[tokio::test]
async fn test_analyze_name_similarity() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test name-similarity help
    let result = runner.run_success(&["analyze", "name-similarity", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Name similarity analysis"));
    
    // Test basic name similarity analysis
    let result = runner.run_success(&["analyze", "name-similarity", "src/"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    
    Ok(())
}

/// Test analyze symbol-table command
#[tokio::test]
async fn test_analyze_symbol_table() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test symbol-table help
    let result = runner.run_success(&["analyze", "symbol-table", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Symbol table analysis"));
    
    // Test basic symbol table analysis
    let result = runner.run_success(&["analyze", "symbol-table", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(15))?;
    
    Ok(())
}

/// Test analyze comprehensive command
#[tokio::test]
async fn test_analyze_comprehensive() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test comprehensive help
    let result = runner.run_success(&["analyze", "comprehensive", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("Comprehensive analysis"));
    
    // Test basic comprehensive analysis
    let result = runner.run_success(&["analyze", "comprehensive", "src/main.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(60))?;
    
    Ok(())
}

/// Test analyze wasm command
#[tokio::test]
async fn test_analyze_wasm() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test wasm help
    let result = runner.run_success(&["analyze", "wasm", "--help"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stdout_text.contains("WebAssembly analysis"));
    
    // Note: WASM analysis requires actual .wasm files, so we test help and error handling
    let result = runner.run_failure(&["analyze", "wasm", "nonexistent.wasm"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    assert!(result.stderr_text.contains("file") || result.stderr_text.contains("found"));
    
    Ok(())
}

/// Test analyze command error handling
#[tokio::test]
async fn test_analyze_error_handling() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    // Test analyze without subcommand
    let result = runner.run_failure(&["analyze"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stderr_text.contains("required") || result.stderr_text.contains("subcommand"));
    
    // Test invalid subcommand
    let result = runner.run_failure(&["analyze", "invalid-subcommand"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    assert!(result.stderr_text.contains("invalid") || result.stderr_text.contains("unrecognized"));
    
    // Test missing file argument
    let result = runner.run_failure(&["analyze", "complexity"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(2))?;
    
    // Test nonexistent file
    let result = runner.run_failure(&["analyze", "complexity", "nonexistent.rs"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(5))?;
    
    Ok(())
}

/// Test analyze command consistency
#[tokio::test]
async fn test_analyze_help_consistency() -> Result<()> {
    let runner = CliTestRunner::new()?;
    
    let subcommands = [
        "complexity", "dead-code", "satd", "deep-context", "tdg", "entropy",
        "duplicates", "lint-hotspot", "big-o", "defect-prediction", "ml-analysis",
        "dependencies", "graph-metrics", "name-similarity", "symbol-table",
        "comprehensive", "wasm"
    ];
    
    for subcommand in &subcommands {
        // Each subcommand should have help available
        let result = runner.run_success(&["analyze", subcommand, "--help"])?;
        TestValidators::assert_performance(&result, Duration::from_secs(5))?;
        
        // Help should contain usage information
        assert!(
            result.stdout_text.contains("Usage:") || 
            result.stdout_text.contains("USAGE:") ||
            result.stdout_text.to_lowercase().contains("usage"),
            "Analyze subcommand '{}' help does not contain usage information", 
            subcommand
        );
    }
    
    Ok(())
}

/// Test analyze command format options
#[tokio::test]
async fn test_analyze_format_options() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Test JSON format
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--format", "json"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    TestValidators::assert_output_format(&result, OutputFormat::Json)?;
    
    // Test CSV format (where supported)
    let result = runner.run_command(&["analyze", "complexity", "src/main.rs", "--format", "csv"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    if result.exit_code == 0 {
        TestValidators::assert_output_format(&result, OutputFormat::Csv)?;
    }
    
    // Test human format (default)
    let result = runner.run_success(&["analyze", "complexity", "src/main.rs", "--format", "human"])?;
    TestValidators::assert_performance(&result, Duration::from_secs(10))?;
    TestValidators::assert_output_format(&result, OutputFormat::Human)?;
    
    Ok(())
}

/// Test analyze command performance requirements
#[tokio::test]
async fn test_analyze_performance() -> Result<()> {
    let runner = CliTestRunner::new()?;
    let project_path = runner.create_sample_project()?;
    std::env::set_current_dir(&project_path)?;
    
    // Fast commands should complete quickly
    let fast_commands = ["satd", "complexity", "tdg"];
    for cmd in &fast_commands {
        let result = runner.run_success(&["analyze", cmd, "src/main.rs"])?;
        TestValidators::assert_performance(&result, Duration::from_secs(20))?;
    }
    
    // Complex commands can take longer
    let slow_commands = ["comprehensive", "ml-analysis", "dead-code"];
    for cmd in &slow_commands {
        let result = runner.run_success(&["analyze", cmd, "src/"])?;
        TestValidators::assert_performance(&result, Duration::from_secs(60))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test analyze command workflow with multiple subcommands
    #[tokio::test]
    async fn test_analyze_workflow() -> Result<()> {
        let runner = CliTestRunner::new()?;
        let project_path = runner.create_sample_project()?;
        std::env::set_current_dir(&project_path)?;
        
        // Run sequence of analyze commands
        let result = runner.run_success(&["analyze", "complexity", "src/main.rs"])?;
        assert!(result.exit_code == 0);
        
        let result = runner.run_success(&["analyze", "satd", "src/"])?;
        assert!(result.exit_code == 0);
        
        let result = runner.run_success(&["analyze", "tdg", "src/main.rs"])?;
        assert!(result.exit_code == 0);
        
        Ok(())
    }
}