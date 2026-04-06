//! Types and utilities for AI-Powered Automated Refactoring
//!
//! Extracted from refactor_auto_handlers.rs for file health compliance (CB-040).
//! Contains public types and markdown analysis utilities.
#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cli::RefactorAutoOutputFormat;

/// File rewrite plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRewritePlan {
    pub file_path: PathBuf,
    pub violations: Vec<ViolationWithContext>,
    pub ast_metadata: AstMetadata,
    pub new_content: String,
}

/// Violation with AST context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationWithContext {
    pub lint_name: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub ast_node_id: Option<String>,
    pub fix_strategy: FixStrategy,
}

/// AST metadata for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMetadata {
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<String>,
    pub structure_hash: String,
}

/// Function information from AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub complexity: u32,
    pub is_test: bool,
}

/// Fix strategy for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixStrategy {
    ExtractFunction,
    SimplifyCondition,
    RemoveDeadCode,
    AddTest,
    ApplySuggestion(String),
}

// ============================================================================
// Markdown Analysis Utilities
// ============================================================================

/// Check if file is a markdown file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn is_markdown_file(file_path: &Path) -> bool {
    file_path.extension().and_then(|s| s.to_str()) == Some("md")
}

/// Handle markdown file analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_markdown_analysis(
    file_path: &Path,
    format: RefactorAutoOutputFormat,
) -> Result<()> {
    use anyhow::Context;

    eprintln!("📝 Detected markdown file - analyzing for quality issues...");

    let content = tokio::fs::read_to_string(file_path)
        .await
        .context("Failed to read markdown file")?;

    let issues = analyze_markdown_issues(file_path, &content)?;
    eprintln!("📊 Found {} quality issues in markdown", issues.len());

    let refactor_request = create_markdown_refactor_request(file_path, &issues, &content);
    // For now, just print the results since the function signature changed
    match format {
        RefactorAutoOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&refactor_request)?);
        }
        _ => {
            eprintln!("📝 Markdown refactor request created");
        }
    }

    Ok(())
}

/// Analyze markdown content for issues
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn analyze_markdown_issues(file_path: &Path, content: &str) -> Result<Vec<&'static str>> {
    let mut issues = Vec::new();

    if !has_proper_headers(content) {
        issues.push("Missing proper header structure");
    }

    if has_unspecified_code_blocks(content) {
        issues.push("Code blocks without language specification");
    }

    if has_broken_relative_links(file_path, content)? {
        issues.push("Contains broken relative links");
    }

    Ok(issues)
}

/// Check if content has proper header structure
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn has_proper_headers(content: &str) -> bool {
    content.contains("# ") || content.contains("## ")
}

/// Check if content has code blocks without language specification
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn has_unspecified_code_blocks(content: &str) -> bool {
    content.contains("```\n") && !content.contains("```rust") && !content.contains("```bash")
}

/// Check if content has broken relative links
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn has_broken_relative_links(file_path: &Path, content: &str) -> Result<bool> {
    for line in content.lines() {
        if line.contains("](../") || line.contains("](./") {
            if let Some(path) = extract_link_path(line) {
                let full_path = file_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(path);
                if !full_path.exists() {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// Extract link path from markdown line
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_link_path(line: &str) -> Option<&str> {
    line.split("](").nth(1).and_then(|s| s.split(')').next())
}

/// Create markdown refactor request
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn create_markdown_refactor_request(
    file_path: &Path,
    issues: &[&str],
    content: &str,
) -> serde_json::Value {
    serde_json::json!({
        "file_path": file_path,
        "file_type": "markdown",
        "issues": issues,
        "content": content,
        "instructions": "Analyze and fix this markdown file. Ensure proper formatting, clear structure, accurate technical details, and working links.",
    })
}

/// Print markdown analysis summary
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_markdown_summary(refactor_request: &serde_json::Value) {
    eprintln!("📄 Markdown Analysis:");
    if let Some(issues) = refactor_request["issues"].as_array() {
        for issue in issues {
            if let Some(issue_str) = issue.as_str() {
                eprintln!("  ⚠️  {issue_str}");
            }
        }
    }

    eprintln!("\n💡 Suggested fixes:");
    eprintln!("  • Add proper header hierarchy");
    eprintln!("  • Specify languages for all code blocks");
    eprintln!("  • Fix any broken links");
    eprintln!("  • Ensure consistent formatting");
}

// ============================================================================
// Quality Metrics (extracted for file health compliance)
// ============================================================================

/// Quality metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    pub total_violations: usize,
    pub coverage_percent: f64,
    pub max_complexity: u32,
    pub satd_count: usize,
    pub files_with_issues: usize,
    pub total_files: usize,
    pub functions_with_high_complexity: usize,
    pub total_functions: usize,
}

/// Refactor progress tracking with percentage completion
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefactorProgress {
    pub overall_completion_percent: f64,
    pub lint_completion_percent: f64,
    pub complexity_completion_percent: f64,
    pub satd_completion_percent: f64,
    pub coverage_completion_percent: f64,
    pub files_completed: usize,
    pub files_remaining: usize,
    pub estimated_time_remaining_minutes: u32,
    pub quality_gates_passed: Vec<String>,
    pub quality_gates_remaining: Vec<String>,
    pub current_phase: RefactorPhase,
}

/// Current phase of refactoring
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum RefactorPhase {
    #[default]
    Initialization,
    LintFixes,
    BuildFixes,
    ComplexityReduction,
    SatdCleanup,
    CoverageDriven,
    QualityValidation,
    Complete,
}
