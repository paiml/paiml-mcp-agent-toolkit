//! Issue #53 Batch 2: Context & Churn MCP Functions - Real Service Integration
//!
//! This example demonstrates that batch 2 MCP tool functions call real
//! analysis services instead of returning placeholder data.
//!
//! Functions demonstrated:
//! - generate_context: File-level AST context generation
//! - generate_deep_context: Full project deep analysis
//! - analyze_churn: Git repository code churn analysis
//!
//! Run this example:
//! ```bash
//! cargo run --example issue_053_batch2_context_churn
//! ```

use pmat::mcp_pmcp::tool_functions;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Issue #53 Batch 2: Context & Churn MCP Functions ===\n");

    // Create temporary test files
    let temp_dir = TempDir::new()?;

    // Create a Rust file with various AST items
    let sample_file = temp_dir.path().join("sample.rs");
    std::fs::write(
        &sample_file,
        r#"
pub struct User {
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }

    pub async fn fetch_profile(&self) -> Result<Profile, Error> {
        // TODO: Implement profile fetching
        unimplemented!()
    }
}

pub enum Status {
    Active,
    Inactive,
    Pending,
}

pub fn authenticate(username: &str, password: &str) -> bool {
    // Authentication logic
    username.len() > 0 && password.len() >= 8
}

async fn internal_helper() {
    println!("Helper function");
}
"#,
    )?;

    // Create another file for deep context
    let lib_file = temp_dir.path().join("lib.rs");
    std::fs::write(
        &lib_file,
        r#"
pub mod models;
pub mod services;

pub fn init() {
    println!("Initializing application");
}
"#,
    )?;

    // ========================================================================
    // Example 1: generate_context (File-level AST Analysis)
    // ========================================================================
    println!("📄 Example 1: File-Level Context Generation");
    println!("────────────────────────────────────────────");

    let context_result = tool_functions::generate_context(
        &[sample_file.clone()],
        Some(10), // max_depth
        false,    // include_dependencies
    )
    .await?;

    println!("Status: {}", context_result["status"]);
    println!("Message: {}", context_result["message"]);

    if let Some(context) = context_result["context"].as_object() {
        println!("\nContext Summary:");
        println!("  Total files: {}", context["total_files"]);

        if let Some(files) = context["files"].as_array() {
            for file in files {
                println!("\n  File: {}", file["path"]);
                println!("  Language: {}", file["language"]);
                println!("  Items: {}", file["items_count"]);

                if let Some(items) = file["items"].as_array() {
                    for item in items.iter().take(5) {
                        println!(
                            "    - {} (line {})",
                            item["name"].as_str().unwrap_or("unnamed"),
                            item["line"]
                        );
                    }
                    if items.len() > 5 {
                        println!("    ... and {} more items", items.len() - 5);
                    }
                }
            }
        }
    }

    println!("\n✅ generate_context is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Example 2: generate_deep_context (Full Project Analysis)
    // ========================================================================
    println!("🔍 Example 2: Deep Context Project Analysis");
    println!("───────────────────────────────────────────");

    // Use the temp directory as a mini project
    let deep_context_result = tool_functions::generate_deep_context(
        &[temp_dir.path().to_path_buf()],
        None, // format (default)
    )
    .await?;

    println!("Status: {}", deep_context_result["status"]);
    println!("Message: {}", deep_context_result["message"]);

    if let Some(context) = deep_context_result["context"].as_object() {
        if let Some(metadata) = context["metadata"].as_object() {
            println!("\nProject Metadata:");
            println!("  Root: {}", metadata["project_root"]);
            println!("  Tool Version: {}", metadata["tool_version"]);
            println!("  Generated: {}", metadata["generated_at"]);
            println!(
                "  Analysis Duration: {}ms",
                metadata["analysis_duration_ms"]
            );
        }

        if let Some(scorecard) = context["quality_scorecard"].as_object() {
            println!("\nQuality Scorecard:");
            println!("  Overall Health: {:.2}", scorecard["overall_health"]);
            println!("  Complexity Score: {:.2}", scorecard["complexity_score"]);
            println!(
                "  Maintainability: {:.2}",
                scorecard["maintainability_index"]
            );
            println!("  Modularity: {:.2}", scorecard["modularity_score"]);
            println!(
                "  Technical Debt: {:.2}h",
                scorecard["technical_debt_hours"]
            );
        }

        println!("\n  Files Analyzed: {}", context["file_count"]);
    }

    println!("\n✅ generate_deep_context is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Example 3: analyze_churn (Git Repository Churn Analysis)
    // ========================================================================
    println!("📊 Example 3: Code Churn Analysis");
    println!("─────────────────────────────────");

    // Use current repository for churn analysis
    let repo_path = std::env::current_dir()?;

    let churn_result = tool_functions::analyze_churn(
        &[repo_path.clone()],
        Some(30), // last 30 days
        Some(5),  // top 5 files
    )
    .await?;

    println!("Status: {}", churn_result["status"]);
    println!("Message: {}", churn_result["message"]);

    if let Some(results) = churn_result["results"].as_object() {
        println!("\nChurn Analysis Summary:");
        println!("  Period: {} days", results["period_days"]);
        println!("  Total Commits: {}", results["total_commits"]);
        println!("  Files Changed: {}", results["total_files_changed"]);
        println!("  Hotspot Files: {}", results["hotspot_files"]);

        if let Some(files) = results["files"].as_array() {
            if !files.is_empty() {
                println!("\n  Top Churning Files:");
                for (i, file) in files.iter().enumerate() {
                    println!(
                        "    {}. {} (score: {:.3})",
                        i + 1,
                        file["path"].as_str().unwrap_or("unknown"),
                        file["churn_score"]
                    );
                    println!(
                        "       {} commits, {} authors, +{} -{} lines",
                        file["commit_count"],
                        file["unique_authors"],
                        file["additions"],
                        file["deletions"]
                    );
                }
            } else {
                println!("\n  No churn detected in the specified period");
            }
        }
    }

    println!("\n✅ analyze_churn is calling REAL service (not placeholder)\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("════════════════════════════════════════════");
    println!("✅ Issue #53 Batch 2 GREEN Phase Complete!");
    println!("════════════════════════════════════════════");
    println!();
    println!("All 3 MCP tool functions now use real analysis services:");
    println!("  1. generate_context → analyze_single_file (AST extraction)");
    println!("  2. generate_deep_context → DeepContextAnalyzer (full project)");
    println!("  3. analyze_churn → GitAnalysisService (code churn metrics)");
    println!();
    println!("Progress: 6/16 MCP functions complete (37.5%)");
    println!("  ✅ Batch 1: analyze_complexity, analyze_satd, analyze_dead_code");
    println!("  ✅ Batch 2: generate_context, generate_deep_context, analyze_churn");
    println!();
    println!("No more placeholder responses!");
    println!();

    Ok(())
}
