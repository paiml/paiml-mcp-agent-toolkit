//! Example: Quality Gate with Violation Details (Issue #129 Fix)
//!
//! This example demonstrates that the quality-gate command now shows
//! detailed violation information instead of just counts.
//!
//! # Usage
//! ```bash
//! cargo run --example quality_gate_violations
//! ```
//!
//! # Related
//! - GitHub Issue #129: quality-gate sub-command doesn't report violations
//! - Fix: Updated format_qg_as_markdown and format_qg_as_summary to include violations

use pmat::cli::analysis_utilities::{
    format_quality_gate_output, QualityGateResults, QualityViolation,
};
use pmat::cli::QualityGateOutputFormat;

fn main() {
    println!("=== Quality Gate Violation Reporting Demo ===\n");

    // Create sample results with violations
    let results = QualityGateResults {
        passed: false,
        total_violations: 5,
        complexity_violations: 2,
        dead_code_violations: 1,
        satd_violations: 1,
        entropy_violations: 1,
        security_violations: 0,
        duplicate_violations: 0,
        coverage_violations: 0,
        section_violations: 0,
        provability_violations: 0,
        provability_score: None,
        violations: vec![], // Simplified for backwards compat
    };

    // Create sample violations
    let violations = vec![
        QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            file: "src/parser.rs".to_string(),
            line: Some(42),
            message: "Cyclomatic complexity 25 exceeds threshold 20".to_string(),
        },
        QualityViolation {
            check_type: "complexity".to_string(),
            severity: "warning".to_string(),
            file: "src/analyzer.rs".to_string(),
            line: Some(100),
            message: "Cyclomatic complexity 18 approaching threshold".to_string(),
        },
        QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "warning".to_string(),
            file: "src/utils.rs".to_string(),
            line: Some(55),
            message: "Function 'unused_helper' is never called".to_string(),
        },
        QualityViolation {
            check_type: "satd".to_string(),
            severity: "info".to_string(),
            file: "src/main.rs".to_string(),
            line: Some(10),
            message: "TODO: Refactor this function".to_string(),
        },
        QualityViolation {
            check_type: "entropy".to_string(),
            severity: "warning".to_string(),
            file: "src/config.rs".to_string(),
            line: None,
            message: "Low entropy score 2.1 (threshold: 3.0)".to_string(),
        },
    ];

    // Demo: Summary format (now shows violations)
    println!("--- Summary Format (default) ---\n");
    let summary =
        format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Summary)
            .expect("formatting should work");
    println!("{}", summary);

    // Demo: Markdown format (now shows violations table)
    println!("\n--- Markdown Format ---\n");
    let markdown =
        format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Markdown)
            .expect("formatting should work");
    println!("{}", markdown);

    // Demo: Detailed format (full violation list)
    println!("\n--- Detailed Format ---\n");
    let detailed =
        format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Detailed)
            .expect("formatting should work");
    println!("{}", detailed);

    println!("\n=== End of Demo ===");
    println!("\nThis demonstrates the fix for GitHub Issue #129.");
    println!("Previously, these formats only showed counts without details.");
}
