//! EXTREME TDD: Deep Context Quality Fixes
//! Implements user-requested improvements with failing tests first (RED-GREEN-REFACTOR)

use tempfile::TempDir;
use std::fs;

/// RED TEST: SATD should only appear when technical debt is detected
#[tokio::test]
async fn test_satd_only_appears_when_debt_found() {
    // ARRANGE: Create file with NO technical debt
    let temp_dir = TempDir::new().unwrap();
    let clean_file = temp_dir.path().join("clean.rs");
    fs::write(&clean_file, r#"
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
"#).unwrap();

    // ACT: Generate context for clean file
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: SATD annotation should NOT appear for clean code
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();
    assert!(!output.contains("[SATD:"),
        "Clean code should not have SATD annotations. Found in output: {}",
        output.lines().filter(|line| line.contains("SATD")).collect::<Vec<_>>().join("\n"));
}

/// RED TEST: SATD should appear with count when technical debt exists
#[tokio::test]
async fn test_satd_appears_with_count_when_debt_exists() {
    // ARRANGE: Create file WITH technical debt
    let temp_dir = TempDir::new().unwrap();
    let debt_file = temp_dir.path().join("debt.rs");
    fs::write(&debt_file, r#"
// TODO: Optimize this algorithm
// FIXME: Handle edge cases properly
fn bad_function() -> i32 {
    // HACK: This is a temporary solution
    42
}
"#).unwrap();

    // ACT: Generate context for file with debt
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: SATD annotation should appear with specific count
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();
    assert!(output.contains("[SATD: 3 items]") || output.contains("[satd: 3]"),
        "Should show SATD count of 3. Output: {}", output);
}

/// RED TEST: Churn annotations must appear when git history exists
#[tokio::test]
async fn test_churn_annotations_appear() {
    // ARRANGE: Create git repository with commits
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn test() {}").unwrap();

    // Initialize git repo and make commits
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["commit", "-m", "initial"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Churn annotations must be present
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();
    assert!(output.contains("[churn:") || output.contains("[CHURN:"),
        "Churn annotations missing from output");
}

/// RED TEST: Overall Health should be normalized 0-100 TDG score, not meaningless percentage
#[tokio::test]
async fn test_overall_health_is_normalized_tdg_score() {
    // ARRANGE: Create test project
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn simple() -> i32 { 42 }").unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Overall Health should be 0-100 range, not thousands of percent
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();

    // Find Overall Health line
    let health_line = output.lines()
        .find(|line| line.contains("Overall Health"))
        .expect("Overall Health line not found");

    // Extract percentage value
    let health_value = extract_percentage(health_line)
        .expect("Could not extract percentage from health line");

    assert!(health_value >= 0.0 && health_value <= 100.0,
        "Overall Health should be 0-100, got: {}%. Line: {}", health_value, health_line);
}

/// RED TEST: Test Coverage should be meaningful percentage or absent, not >100%
#[tokio::test]
async fn test_coverage_is_meaningful_or_absent() {
    // ARRANGE: Create test project
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn simple() -> i32 { 42 }").unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Test Coverage should be realistic or absent
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();

    if let Some(coverage_line) = output.lines().find(|line| line.contains("Test Coverage")) {
        let coverage_value = extract_percentage(coverage_line);
        if let Some(value) = coverage_value {
            assert!(value >= 0.0 && value <= 100.0,
                "Test Coverage should be 0-100% if present, got: {}%. Line: {}",
                value, coverage_line);
        }
    }
    // If Test Coverage line is absent, that's acceptable
}

/// RED TEST: PageRank should only appear for highly connected files (threshold based)
#[tokio::test]
async fn test_pagerank_only_for_highly_connected() {
    // ARRANGE: Create project with varying connectivity
    let temp_dir = TempDir::new().unwrap();

    // Low connectivity file (should not show PageRank)
    let isolated_file = temp_dir.path().join("isolated.rs");
    fs::write(&isolated_file, "fn isolated() -> i32 { 42 }").unwrap();

    // High connectivity file (should show PageRank)
    let connected_file = temp_dir.path().join("connected.rs");
    fs::write(&connected_file, r#"
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn highly_connected() -> Vec<String> {
    vec![]
}
"#).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: PageRank should be selective, not appear for every file
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();

    // Should not have PageRank for every single function
    let pagerank_count = output.matches("[pagerank:").count() + output.matches("[PageRank:").count();
    let total_functions = output.matches("**Function**").count();

    if pagerank_count > 0 {
        let pagerank_ratio = pagerank_count as f64 / total_functions as f64;
        assert!(pagerank_ratio < 0.3, // Less than 30% of functions should have PageRank
            "PageRank should be selective, not appear for every function. Ratio: {:.1}%",
            pagerank_ratio * 100.0);
    }
}

/// RED TEST: Auto-scaling concurrency should detect and use system capabilities
#[tokio::test]
async fn test_auto_scaling_concurrency() {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

    // ARRANGE: Create config with auto-scaling
    let config = DeepContextConfig::with_auto_scaling();
    let analyzer = DeepContextAnalyzer::new(config.clone());

    // ASSERT: Should use more than 1 core on multi-core systems
    let logical_cores = num_cpus::get();
    if logical_cores > 1 {
        assert!(config.parallel > 1,
            "Auto-scaling should use multiple cores. Detected {} cores, using {} threads",
            logical_cores, config.parallel);

        // Should use reasonable scaling (not more than logical cores + 1)
        assert!(config.parallel <= logical_cores + 1,
            "Should not exceed logical cores + 1. Got {} for {} cores",
            config.parallel, logical_cores);
    }
}

/// RED TEST: Shell scripts should be analyzed per specification
#[tokio::test]
async fn test_shell_script_analysis() {
    // ARRANGE: Create shell script with various constructs
    let temp_dir = TempDir::new().unwrap();
    let shell_script = temp_dir.path().join("script.sh");
    fs::write(&shell_script, r#"#!/bin/bash
set -euo pipefail

# Function definition
process_files() {
    local dir="$1"

    # Pipeline with complexity
    find "$dir" -name "*.log" | grep -v error | head -10 | while read -r file; do
        # Subshell complexity
        (
            wc -l "$file"
            if [ -f "$file.backup" ]; then
                echo "Backup exists"
            fi
        )
    done
}

main() {
    process_files "/var/log"
}

main "$@"
"#).unwrap();

    // ACT: Generate context including shell script
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        None, // Auto-detect
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Shell script should be analyzed with appropriate metrics
    assert!(result.is_ok());
    let output = fs::read_to_string(output_file).unwrap();

    assert!(output.contains("script.sh"), "Shell script should be included in analysis");

    // Should detect shell functions
    assert!(output.contains("process_files") || output.contains("main"),
        "Shell functions should be detected");

    // Should have appropriate complexity metrics for shell
    let script_section = extract_file_section(&output, "script.sh");
    if let Some(section) = script_section {
        // Should have some form of analysis - either complexity annotations or functions detected
        assert!(section.contains("[complexity:") || section.contains("Functions:") || section.contains("process_files") || section.contains("main"),
            "Shell script should have complexity analysis or function detection. Section: {}", section);
    }
}

// Helper functions for tests

fn extract_percentage(line: &str) -> Option<f64> {
    // Extract percentage value from lines like "Overall Health: 6833.3%"
    if let Some(percent_pos) = line.find('%') {
        let before_percent = &line[..percent_pos];
        if let Some(colon_pos) = before_percent.rfind(':') {
            let value_str = before_percent[colon_pos + 1..].trim();
            return value_str.parse::<f64>().ok();
        }
        // Try finding space-separated value
        if let Some(space_pos) = before_percent.rfind(' ') {
            let value_str = before_percent[space_pos + 1..].trim();
            return value_str.parse::<f64>().ok();
        }
    }
    None
}

fn extract_file_section(output: &str, filename: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    let mut start_idx = None;

    // Find section header
    for (i, line) in lines.iter().enumerate() {
        if line.contains(filename) && (line.starts_with("### ") || line.starts_with("##")) {
            start_idx = Some(i);
            break;
        }
    }

    if let Some(start) = start_idx {
        // Find end of section (next file header or end of document)
        let mut end_idx = lines.len();
        for i in (start + 1)..lines.len() {
            if lines[i].starts_with("### ") && !lines[i].contains(filename) {
                end_idx = i;
                break;
            }
        }

        let section_lines = &lines[start..end_idx];
        return Some(section_lines.join("\n"));
    }

    None
}