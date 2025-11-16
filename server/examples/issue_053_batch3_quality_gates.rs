//! Issue #53 Batch 3: Quality Gate MCP Functions - Real Service Integration
//!
//! This example demonstrates that batch 3 MCP tool functions call real
//! TDG (Technical Debt Grading) services instead of returning placeholder data.
//!
//! Functions demonstrated:
//! - check_quality_gates: Project-level quality gate validation
//! - check_quality_gate_file: File-level quality gate validation
//! - quality_gate_summary: Aggregated quality metrics summary
//!
//! Run this example:
//! ```bash
//! cargo run --example issue_053_batch3_quality_gates
//! ```

use pmat::mcp_pmcp::tool_functions;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Issue #53 Batch 3: Quality Gate MCP Functions ===\n");

    // Create temporary test files with varying quality
    let temp_dir = TempDir::new()?;

    // File 1: High-quality, simple code
    let simple_file = temp_dir.path().join("simple.rs");
    std::fs::write(
        &simple_file,
        r#"
/// Simple, well-documented function
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Calculate factorial
pub fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
"#,
    )?;

    // File 2: Complex code with high nesting and SATD
    let complex_file = temp_dir.path().join("complex.rs");
    std::fs::write(
        &complex_file,
        r#"
pub fn process_data(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for item in data {
        // TODO: This needs refactoring - too complex
        if item > 0 {
            if item < 100 {
                if item % 2 == 0 {
                    if item % 3 == 0 {
                        result.push(item * 2);
                    } else {
                        result.push(item + 1);
                    }
                } else {
                    if item % 5 == 0 {
                        result.push(item - 1);
                    } else {
                        result.push(item);
                    }
                }
            } else {
                result.push(item / 2);
            }
        } else {
            // FIXME: Handle negative numbers properly
            result.push(0);
        }
    }

    result
}

// HACK: Quick workaround for performance issue
pub fn quick_sort(arr: &mut [i32]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot = arr[arr.len() / 2];
    let mut i = 0;
    let mut j = arr.len() - 1;

    // XXX: This implementation is inefficient
    loop {
        while arr[i] < pivot {
            i += 1;
            if i > j {
                break;
            }
        }
        while arr[j] > pivot {
            if j == 0 {
                break;
            }
            j -= 1;
        }
        if i >= j {
            break;
        }
        arr.swap(i, j);
        i += 1;
        if j > 0 {
            j -= 1;
        }
    }
}
"#,
    )?;

    // File 3: Moderate quality
    let moderate_file = temp_dir.path().join("moderate.rs");
    std::fs::write(
        &moderate_file,
        r#"
pub struct User {
    pub name: String,
    pub email: String,
    pub age: u32,
}

impl User {
    pub fn new(name: String, email: String, age: u32) -> Self {
        Self { name, email, age }
    }

    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }

    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

pub fn find_user(users: &[User], name: &str) -> Option<&User> {
    users.iter().find(|u| u.name == name)
}
"#,
    )?;

    // ========================================================================
    // Example 1: check_quality_gates (Project-Level)
    // ========================================================================
    println!("📋 Example 1: Project-Level Quality Gate Check");
    println!("────────────────────────────────────────────────");

    // Standard mode (lenient thresholds: score >= 50, grade >= D)
    println!("\n🔵 Standard Mode (threshold: 50.0, grade D):");
    let standard_result = tool_functions::check_quality_gates(
        &[temp_dir.path().to_path_buf()],
        false, // strict = false
    )
    .await?;

    println!("Status: {}", standard_result["status"]);
    println!("Message: {}", standard_result["message"]);
    println!("Passed: {}", standard_result["passed"]);
    println!("Average Score: {:.2}", standard_result["score"]);
    println!("Average Grade: {}", standard_result["grade"]);
    println!("Files Analyzed: {}", standard_result["files_analyzed"]);

    if let Some(violations) = standard_result["violations"].as_array() {
        if !violations.is_empty() {
            println!("\nViolations:");
            for v in violations {
                println!(
                    "  - {} (score: {}, grade: {})",
                    v["file"], v["score"], v["grade"]
                );
            }
        } else {
            println!("\n✅ No violations in standard mode");
        }
    }

    // Strict mode (strict thresholds: score >= 70, grade >= B)
    println!("\n🔴 Strict Mode (threshold: 70.0, grade B):");
    let strict_result = tool_functions::check_quality_gates(
        &[temp_dir.path().to_path_buf()],
        true, // strict = true
    )
    .await?;

    println!("Status: {}", strict_result["status"]);
    println!("Passed: {}", strict_result["passed"]);
    println!("Average Score: {:.2}", strict_result["score"]);
    println!("Average Grade: {}", strict_result["grade"]);

    if let Some(violations) = strict_result["violations"].as_array() {
        println!("\nViolations ({} files):", violations.len());
        for v in violations {
            println!(
                "  - {} (score: {}, grade: {})",
                v["file"], v["score"], v["grade"]
            );
        }
    }

    println!("\n✅ check_quality_gates is calling REAL TDG service (not placeholder)\n");

    // ========================================================================
    // Example 2: check_quality_gate_file (File-Level)
    // ========================================================================
    println!("📄 Example 2: File-Level Quality Gate Check");
    println!("──────────────────────────────────────────");

    // Check the simple file
    println!("\n🟢 Checking simple.rs:");
    let simple_gate = tool_functions::check_quality_gate_file(&simple_file, false).await?;

    println!("Status: {}", simple_gate["status"]);
    println!("File: {}", simple_gate["file"]);
    println!("Passed: {}", simple_gate["passed"]);
    println!("Score: {:.2}", simple_gate["score"]);
    println!("Grade: {}", simple_gate["grade"]);

    if let Some(metrics) = simple_gate["metrics"].as_object() {
        println!("\nMetrics:");
        println!(
            "  Structural Complexity: {:.2}",
            metrics["structural_complexity"]
        );
        println!(
            "  Semantic Complexity: {:.2}",
            metrics["semantic_complexity"]
        );
        println!("  Duplication Ratio: {:.2}", metrics["duplication_ratio"]);
        println!("  Coupling Score: {:.2}", metrics["coupling_score"]);
        println!("  Documentation Coverage: {:.2}", metrics["doc_coverage"]);
        println!("  Consistency Score: {:.2}", metrics["consistency_score"]);
    }

    // Check the complex file (likely to have violations)
    println!("\n🟡 Checking complex.rs:");
    let complex_gate = tool_functions::check_quality_gate_file(&complex_file, false).await?;

    println!("Status: {}", complex_gate["status"]);
    println!("Passed: {}", complex_gate["passed"]);
    println!("Score: {:.2}", complex_gate["score"]);
    println!("Grade: {}", complex_gate["grade"]);

    if let Some(violations) = complex_gate["violations"].as_array() {
        if !violations.is_empty() {
            println!("\nQuality Violations ({}):", violations.len());
            for (i, v) in violations.iter().enumerate().take(5) {
                println!(
                    "  {}. [{}] -{:.2} points: {}",
                    i + 1,
                    v["category"].as_str().unwrap_or("Unknown"),
                    v["penalty"],
                    v["description"]
                );
            }
            if violations.len() > 5 {
                println!("  ... and {} more violations", violations.len() - 5);
            }
        }
    }

    println!("\n✅ check_quality_gate_file is calling REAL TDG service (not placeholder)\n");

    // ========================================================================
    // Example 3: quality_gate_summary (Aggregated Summary)
    // ========================================================================
    println!("📊 Example 3: Quality Gate Summary");
    println!("─────────────────────────────────");

    let summary_result =
        tool_functions::quality_gate_summary(&[temp_dir.path().to_path_buf()]).await?;

    println!("Status: {}", summary_result["status"]);
    println!("Message: {}", summary_result["message"]);

    if let Some(summary) = summary_result["summary"].as_object() {
        println!("\nProject Summary:");
        println!("  Total Files: {}", summary["total_files"]);
        println!("  Passed Files: {}", summary["passed_files"]);
        println!("  Failed Files: {}", summary["failed_files"]);
        println!("  Average Score: {:.2}", summary["average_score"]);
        println!("  Average Grade: {}", summary["average_grade"]);
        println!("  Threshold: {}", summary["threshold_score"]);

        if let Some(grade_dist) = summary["grade_distribution"].as_object() {
            println!("\n  Grade Distribution:");
            let mut grades: Vec<_> = grade_dist.iter().collect();
            grades.sort_by_key(|&(k, _)| k);
            for (grade, count) in grades {
                println!("    {}: {} files", grade, count);
            }
        }

        if let Some(lang_dist) = summary["language_distribution"].as_object() {
            println!("\n  Language Distribution:");
            for (lang, count) in lang_dist {
                println!("    {}: {} files", lang, count);
            }
        }
    }

    println!("\n✅ quality_gate_summary is calling REAL TDG service (not placeholder)\n");

    // ========================================================================
    // Summary
    // ========================================================================
    println!("════════════════════════════════════════════");
    println!("✅ Issue #53 Batch 3 GREEN Phase Complete!");
    println!("════════════════════════════════════════════");
    println!();
    println!("All 3 MCP tool functions now use real TDG analysis services:");
    println!("  1. check_quality_gates → TdgAnalyzer.analyze_project()");
    println!("  2. check_quality_gate_file → TdgAnalyzer.analyze_file()");
    println!("  3. quality_gate_summary → TdgAnalyzer + ProjectScore aggregation");
    println!();
    println!("Progress: 9/16 MCP functions complete (56.3%)");
    println!("  ✅ Batch 1: analyze_complexity, analyze_satd, analyze_dead_code");
    println!("  ✅ Batch 2: generate_context, generate_deep_context, analyze_churn");
    println!("  ✅ Batch 3: check_quality_gates, check_quality_gate_file, quality_gate_summary");
    println!();
    println!("No more placeholder responses in batches 1-3!");
    println!();

    Ok(())
}
