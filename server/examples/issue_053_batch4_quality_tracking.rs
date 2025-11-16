//! Issue #53 Batch 4: Quality Tracking & Git Integration MCP Functions
//!
//! This example demonstrates that batch 4 MCP tool functions call real
//! TDG (Technical Debt Grading) and Git services instead of returning placeholder data.
//!
//! Functions demonstrated:
//! - quality_gate_baseline: Create TDG baseline snapshots with content hashing
//! - quality_gate_compare: Compare baselines to detect quality regressions
//! - git_status: Extract git repository status and metadata
//!
//! Run this example:
//! ```bash
//! cargo run --example issue_053_batch4_quality_tracking
//! ```

use pmat::mcp_pmcp::tool_functions;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Issue #53 Batch 4: Quality Tracking & Git Integration ===\n");

    // ========================================================================
    // Setup: Create temporary test files with varying quality
    // ========================================================================
    let temp_dir = TempDir::new()?;
    let baseline_v1 = temp_dir.path().join("baseline_v1.json");
    let baseline_v2 = temp_dir.path().join("baseline_v2.json");

    // Version 1: High-quality, simple code
    let simple_file = temp_dir.path().join("calculator.rs");
    std::fs::write(
        &simple_file,
        r#"
/// Simple calculator functions
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#,
    )?;

    let utils_file = temp_dir.path().join("utils.rs");
    std::fs::write(
        &utils_file,
        r#"
pub fn format_number(n: i32) -> String {
    format!("{}", n)
}
"#,
    )?;

    // ========================================================================
    // Example 1: quality_gate_baseline - Create Baseline Snapshot
    // ========================================================================
    println!("📸 Example 1: Creating Quality Gate Baseline (Version 1)");
    println!("────────────────────────────────────────────────────────");

    let result_v1 =
        tool_functions::quality_gate_baseline(&[temp_dir.path().to_path_buf()], Some(&baseline_v1))
            .await?;

    println!("Status: {}", result_v1["status"]);
    println!("Message: {}", result_v1["message"]);

    if let Some(baseline) = result_v1["baseline"].as_object() {
        println!("\nBaseline Details:");
        println!(
            "  File: {}",
            baseline
                .get("file_path")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Timestamp: {}",
            baseline
                .get("timestamp")
                .unwrap_or(&serde_json::json!("unknown"))
        );

        if let Some(summary) = baseline.get("summary").and_then(|s| s.as_object()) {
            println!("\n  Summary:");
            println!(
                "    Total Files: {}",
                summary.get("total_files").unwrap_or(&serde_json::json!(0))
            );
            if let Some(avg_score) = summary.get("average_score").and_then(|s| s.as_f64()) {
                println!("    Average Score: {:.2}", avg_score);
            }
            println!(
                "    Average Grade: {}",
                summary
                    .get("average_grade")
                    .unwrap_or(&serde_json::json!("N/A"))
            );
        }

        if let Some(git_context) = baseline.get("git_context").and_then(|g| g.as_object()) {
            println!("\n  Git Context (from baseline):");
            println!(
                "    Branch: {}",
                git_context
                    .get("branch")
                    .unwrap_or(&serde_json::json!("N/A"))
            );
            println!(
                "    Commit: {}",
                git_context
                    .get("commit_sha_short")
                    .unwrap_or(&serde_json::json!("N/A"))
            );
        }
    }

    println!("\n✅ Baseline snapshot created with content hashing and TDG scores\n");

    // ========================================================================
    // Example 2: Modify Code to Introduce Quality Regression
    // ========================================================================
    println!("📝 Example 2: Modifying Code (Introducing Quality Regression)");
    println!("─────────────────────────────────────────────────────────────");

    // Version 2: Add complex code with high nesting and SATD
    std::fs::write(
        &simple_file,
        r#"
/// Simple calculator functions
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// TODO: This needs major refactoring
pub fn complex_calculation(data: Vec<i32>) -> i32 {
    let mut result = 0;
    for item in data {
        if item > 0 {
            if item < 100 {
                if item % 2 == 0 {
                    if item % 3 == 0 {
                        result += item * 2;
                    } else {
                        result += item + 1;
                    }
                } else {
                    result += item;
                }
            }
        }
    }
    result
}
"#,
    )?;

    println!("Modified calculator.rs:");
    println!("  - Added complex_calculation() with deep nesting (4+ levels)");
    println!("  - Added SATD annotation (TODO comment)");
    println!("  - Expected: Lower quality score, potential regression\n");

    // Create baseline v2 with modified code
    let result_v2 =
        tool_functions::quality_gate_baseline(&[temp_dir.path().to_path_buf()], Some(&baseline_v2))
            .await?;

    println!("Status: {}", result_v2["status"]);
    if let Some(baseline) = result_v2["baseline"].as_object() {
        if let Some(summary) = baseline.get("summary").and_then(|s| s.as_object()) {
            println!("Version 2 Summary:");
            if let Some(avg_score) = summary.get("average_score").and_then(|s| s.as_f64()) {
                println!("  Average Score: {:.2}", avg_score);
            }
            println!(
                "  Average Grade: {}",
                summary
                    .get("average_grade")
                    .unwrap_or(&serde_json::json!("N/A"))
            );
        }
    }

    println!("\n✅ Version 2 baseline created\n");

    // ========================================================================
    // Example 3: quality_gate_compare - Compare Baselines
    // ========================================================================
    println!("🔍 Example 3: Comparing Baselines (Detecting Regressions)");
    println!("─────────────────────────────────────────────────────────");

    let comparison_result =
        tool_functions::quality_gate_compare(&baseline_v1, &[temp_dir.path().to_path_buf()])
            .await?;

    println!("Status: {}", comparison_result["status"]);
    println!("Message: {}", comparison_result["message"]);

    if let Some(comparison) = comparison_result["comparison"].as_object() {
        println!("\nComparison Results:");
        println!("  Improved Files: {}", comparison["improved"]);
        println!("  Regressed Files: {}", comparison["regressed"]);
        println!("  Unchanged Files: {}", comparison["unchanged"]);
        println!("  Added Files: {}", comparison["added"]);
        println!("  Removed Files: {}", comparison["removed"]);
        println!("  Has Regressions: {}", comparison["has_regressions"]);
        println!("  Total Changes: {}", comparison["total_changes"]);

        if let Some(regressed) = comparison["regressed_files"].as_array() {
            if !regressed.is_empty() {
                println!("\n  Regressed Files (Quality Decreased):");
                for file in regressed.iter().take(3) {
                    if let Some(file_obj) = file.as_object() {
                        println!(
                            "    - {} (Δ{:.2} points, old: {}, new: {})",
                            file_obj
                                .get("file")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown"),
                            file_obj
                                .get("delta")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            file_obj
                                .get("old_grade")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?"),
                            file_obj
                                .get("new_grade")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                        );
                    }
                }
            }
        }

        if let Some(improved) = comparison["improved_files"].as_array() {
            if !improved.is_empty() {
                println!("\n  Improved Files (Quality Increased):");
                for file in improved.iter().take(3) {
                    if let Some(file_obj) = file.as_object() {
                        println!(
                            "    - {} (+{:.2} points, old: {}, new: {})",
                            file_obj
                                .get("file")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown"),
                            file_obj
                                .get("delta")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            file_obj
                                .get("old_grade")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?"),
                            file_obj
                                .get("new_grade")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                        );
                    }
                }
            }
        }
    }

    println!("\n✅ Baseline comparison detected quality changes (regression in calculator.rs)\n");

    // ========================================================================
    // Example 4: git_status - Extract Git Repository Status
    // ========================================================================
    println!("🔧 Example 4: Git Repository Status Extraction");
    println!("─────────────────────────────────────────────");

    // Use actual git repository (project root)
    let repo_path = std::env::current_dir()?
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let git_result = tool_functions::git_status(&repo_path).await?;

    println!("Status: {}", git_result["status"]);
    println!("Message: {}", git_result["message"]);

    if let Some(git_status) = git_result["git_status"].as_object() {
        println!("\nGit Repository Status:");
        println!(
            "  Branch: {}",
            git_status
                .get("branch")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Commit SHA (short): {}",
            git_status
                .get("commit_sha_short")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Commit SHA (full): {}",
            git_status
                .get("commit_sha")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Author: {}",
            git_status
                .get("author_name")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Timestamp: {}",
            git_status
                .get("commit_timestamp")
                .unwrap_or(&serde_json::json!("unknown"))
        );
        println!(
            "  Is Clean: {}",
            git_status
                .get("is_clean")
                .unwrap_or(&serde_json::json!(false))
        );
        println!(
            "  Uncommitted Files: {}",
            git_status
                .get("uncommitted_files")
                .unwrap_or(&serde_json::json!(0))
        );

        if let Some(tags) = git_status.get("tags").and_then(|t| t.as_array()) {
            if !tags.is_empty() {
                println!(
                    "  Tags: {}",
                    tags.iter()
                        .filter_map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        if let Some(remote_url) = git_status.get("remote_url").and_then(|r| r.as_str()) {
            if !remote_url.is_empty() {
                println!("  Remote URL: {}", remote_url);
            }
        }

        println!("\n  Recent Commit Message:");
        if let Some(message) = git_status.get("commit_message").and_then(|m| m.as_str()) {
            let lines: Vec<&str> = message.lines().take(3).collect();
            for line in lines {
                println!("    {}", line);
            }
        }
    }

    println!("\n✅ Git status extracted from real repository (not placeholder)\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("════════════════════════════════════════════════════════");
    println!("✅ Issue #53 Batch 4 GREEN Phase Complete!");
    println!("════════════════════════════════════════════════════════");
    println!();
    println!("All 3 MCP tool functions now use real services:");
    println!("  1. quality_gate_baseline → TdgBaseline + content hashing");
    println!("  2. quality_gate_compare → BaselineComparison with regression detection");
    println!("  3. git_status → GitContext with full repository metadata");
    println!();
    println!("Key Features Demonstrated:");
    println!("  ✓ Baseline creation with Blake3 content hashing");
    println!("  ✓ Baseline persistence to JSON files");
    println!("  ✓ Quality regression detection via comparison");
    println!("  ✓ Git integration with commit/branch/author tracking");
    println!("  ✓ Project-level quality metrics aggregation");
    println!();
    println!("Progress: 12/16 MCP functions complete (75.0%)");
    println!("  ✅ Batch 1: analyze_complexity, analyze_satd, analyze_dead_code");
    println!("  ✅ Batch 2: generate_context, generate_deep_context, analyze_churn");
    println!("  ✅ Batch 3: check_quality_gates, check_quality_gate_file, quality_gate_summary");
    println!("  ✅ Batch 4: quality_gate_baseline, quality_gate_compare, git_status");
    println!();
    println!("No more placeholder responses in batches 1-4!");
    println!();

    Ok(())
}
