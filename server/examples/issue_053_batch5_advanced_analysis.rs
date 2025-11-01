//! Issue #53 Batch 5: Advanced Analysis MCP Functions
//!
//! This example demonstrates that batch 5 MCP tool functions call real services
//! instead of returning placeholder data.
//!
//! Functions demonstrated:
//! - analyze_lint_hotspots: Find files with high quality violation density using TDG
//! - analyze_coupling: Detect structural coupling using dependency analysis
//! - analyze_context: Multi-type context analysis with DeepContext
//! - context_summary: Aggregate codebase summary with language detection
//!
//! Run this example:
//! ```bash
//! cargo run --example issue_053_batch5_advanced_analysis
//! ```

use pmat::mcp_pmcp::tool_functions;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Issue #53 Batch 5: Advanced Analysis MCP Functions ===\n");

    // ========================================================================
    // Setup: Create temporary test files with varying quality
    // ========================================================================
    let temp_dir = TempDir::new()?;
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir)?;

    // High-quality file
    let simple_file = src_dir.join("simple.rs");
    std::fs::write(
        &simple_file,
        r#"
/// Simple, clean functions
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#,
    )?;

    // Low-quality file (hotspot candidate)
    let complex_file = src_dir.join("complex.rs");
    std::fs::write(
        &complex_file,
        r#"
use std::collections::HashMap;

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

    // File with coupling
    let utils_file = src_dir.join("utils.rs");
    std::fs::write(
        &utils_file,
        r#"
use super::simple;
use super::complex;

pub fn format_number(n: i32) -> String {
    format!("{}", n)
}
"#,
    )?;

    // ========================================================================
    // Example 1: analyze_lint_hotspots - Find quality hotspots
    // ========================================================================
    println!("📍 Example 1: Analyzing Lint Hotspots (TDG-based)");
    println!("────────────────────────────────────────────────");

    let hotspots_result = tool_functions::analyze_lint_hotspots(
        &[temp_dir.path().to_path_buf()],
        Some(3),
    )
    .await?;

    println!("Status: {}", hotspots_result["status"]);
    println!("Message: {}", hotspots_result["message"]);

    if let Some(results) = hotspots_result["results"].as_object() {
        println!("\nResults:");
        println!("  Total Files Analyzed: {}", results["total_files_analyzed"]);
        println!("  Top Files Limit: {}", results["top_files_limit"]);

        if let Some(hotspots) = results["hotspots"].as_array() {
            println!("\n  Hotspots (lowest quality scores):");
            for hotspot in hotspots {
                println!("    - {}", hotspot["file"]);
                println!("      Score: {}", hotspot["score"]);
                println!("      Grade: {}", hotspot["grade"]);
                println!("      Violations: {}", hotspot["violation_count"]);
                println!("      SATD: {}", hotspot["satd_count"]);
                println!("      Complexity: {}", hotspot["complexity"]);
                println!();
            }
        }
    }

    println!("✅ Lint hotspots detected using real TDG analysis\n");

    // ========================================================================
    // Example 2: analyze_coupling - Detect structural coupling
    // ========================================================================
    println!("🔗 Example 2: Analyzing Coupling");
    println!("──────────────────────────────");

    let coupling_result = tool_functions::analyze_coupling(
        &[temp_dir.path().to_path_buf()],
        Some(0.3), // Lower threshold to catch more files
    )
    .await?;

    println!("Status: {}", coupling_result["status"]);
    println!("Message: {}", coupling_result["message"]);

    if let Some(results) = coupling_result["results"].as_object() {
        println!("\nResults:");
        println!("  Total Files: {}", results["total_files"]);
        println!("  Threshold: {}", results["threshold"]);

        if let Some(project_metrics) = results["project_metrics"].as_object() {
            println!("\n  Project Metrics:");
            println!("    Average Afferent: {}", project_metrics["avg_afferent"]);
            println!("    Average Efferent: {}", project_metrics["avg_efferent"]);
            println!("    Max Afferent: {}", project_metrics["max_afferent"]);
            println!("    Max Efferent: {}", project_metrics["max_efferent"]);
        }

        if let Some(couplings) = results["couplings"].as_array() {
            println!("\n  Files Above Threshold:");
            for coupling in couplings {
                println!("    - {}", coupling["file"]);
                println!("      Afferent: {}", coupling["afferent_coupling"]);
                println!("      Efferent: {}", coupling["efferent_coupling"]);
                println!("      Instability: {:.2}", coupling["instability"]);
                println!("      Strength: {}", coupling["strength"]);
                println!();
            }
        }
    }

    println!("✅ Coupling analysis completed using real dependency analysis\n");

    // ========================================================================
    // Example 3: analyze_context - Multi-type context analysis
    // ========================================================================
    println!("🔍 Example 3: Analyzing Context (DeepContext)");
    println!("────────────────────────────────────────────");

    let context_result = tool_functions::analyze_context(
        &[temp_dir.path().to_path_buf()],
        &["structure".to_string(), "dependencies".to_string()],
    )
    .await?;

    println!("Status: {}", context_result["status"]);
    println!("Message: {}", context_result["message"]);
    println!("Context: {}", context_result["context"]);

    if let Some(analyses) = context_result["analyses"].as_object() {
        println!("\nAnalyses:");
        if let Some(structure) = analyses["structure"].as_object() {
            println!("  Structure:");
            println!("    Total Files: {}", structure["total_files"]);
            println!("    Total Functions: {}", structure["total_functions"]);
        }
        if let Some(dependencies) = analyses["dependencies"].as_object() {
            println!("  Dependencies:");
            println!("    Total Imports: {}", dependencies["total_imports"]);
        }
    }

    println!("\n✅ Context analysis completed using real DeepContext analyzer\n");

    // ========================================================================
    // Example 4: context_summary - Aggregate codebase summary
    // ========================================================================
    println!("📊 Example 4: Generating Context Summary");
    println!("───────────────────────────────────────");

    let summary_result = tool_functions::context_summary(
        &[temp_dir.path().to_path_buf()],
        Some("detailed"),
    )
    .await?;

    println!("Status: {}", summary_result["status"]);
    println!("Message: {}", summary_result["message"]);

    if let Some(summary) = summary_result["summary"].as_object() {
        println!("\nSummary:");
        println!("  Total Files: {}", summary["total_files"]);
        println!("  Total Lines: {}", summary["total_lines"]);
        if let Some(languages) = summary["languages"].as_array() {
            println!("  Languages Detected:");
            for lang in languages {
                println!("    - {}", lang);
            }
        }
    }

    println!("\n✅ Context summary generated with real file system analysis\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("════════════════════════════════════════════════════════");
    println!("✅ Issue #53 Batch 5 GREEN Phase Complete!");
    println!("════════════════════════════════════════════════════════");
    println!();
    println!("All 4 MCP tool functions now use real services:");
    println!("  1. analyze_lint_hotspots → TdgAnalyzer (quality hotspot detection)");
    println!("  2. analyze_coupling → DeepContext (dependency/coupling analysis)");
    println!("  3. analyze_context → DeepContextAnalyzer (multi-type analysis)");
    println!("  4. context_summary → File system traversal (language detection)");
    println!();
    println!("Key Features Demonstrated:");
    println!("  ✓ TDG-based quality scoring and hotspot identification");
    println!("  ✓ Afferent/efferent coupling metrics with instability calculation");
    println!("  ✓ Multi-type context analysis (structure, dependencies)");
    println!("  ✓ Language detection across 13 supported languages");
    println!("  ✓ Project-level aggregated metrics and summaries");
    println!();
    println!("Progress: 16/16 MCP functions complete (100%)");
    println!("  ✅ Batch 1: analyze_complexity, analyze_satd, analyze_dead_code");
    println!("  ✅ Batch 2: generate_context, generate_deep_context, analyze_churn");
    println!("  ✅ Batch 3: check_quality_gates, check_quality_gate_file, quality_gate_summary");
    println!("  ✅ Batch 4: quality_gate_baseline, quality_gate_compare, git_status");
    println!("  ✅ Batch 5: analyze_lint_hotspots, analyze_coupling, analyze_context, context_summary");
    println!();
    println!("No more placeholder responses - all functions use real services!");
    println!();

    Ok(())
}
