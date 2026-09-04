//! Example: Quality Gate with Violation Details (Issue #129, #226 Fix)
//!
//! This example demonstrates that the quality-gate command now shows
//! detailed violation information instead of just counts. With #226,
//! violations now carry optional `ViolationDetails` for explainability.
//!
//! # Usage
//! ```bash
//! cargo run --example quality_gate_violations
//! ```
//!
//! # Related
//! - GitHub Issue #129: quality-gate sub-command doesn't report violations
//! - GitHub Issue #226: entropy violations lack explainability
//! - Fix: ViolationDetails struct with affected_files, example_code, fix_suggestion, score_factors

use pmat::cli::analysis_utilities::{
    format_quality_gate_output, QualityGateResults, QualityViolation, ViolationDetails,
};
use pmat::cli::QualityGateOutputFormat;

fn main() {
    println!("=== Quality Gate Violation Reporting Demo ===\n");

    // Create sample results with violations
    let results = QualityGateResults {
        files_examined: 0,
        // #1035: every check declares what it declined to read. This demo runs no
        // check, so the map is empty — an empty map is "nothing was skipped", which
        // is a different claim from an absent key.
        files_not_read: std::collections::BTreeMap::new(),
        checks_run: Vec::new(),
        not_measured: Vec::new(),
        not_applicable: Vec::new(),
        passed: false,
        total_violations: 5,
        // Of the 5 findings, 4 are verdict-bearing (warning/error) and one is
        // advisory (info). `passed` follows this count, not total_violations —
        // an informational finding must not decide a gate verdict.
        blocking_violations: 4,
        complexity_violations: 2,
        dead_code_violations: 1,
        satd_violations: 1,
        entropy_violations: 1,
        security_violations: 0,
        identical_files: 0,
        coverage_violations: 0,
        section_violations: 0,
        provability_violations: 0,
        file_size_violations: 0,
        churn_violations: 0,
        lint_violations: 0,
        provability_score: None,
        violations: vec![], // Simplified for backwards compat
    };

    // Create sample violations — note the new `details` field (#226)
    let violations = vec![
        QualityViolation::new(
            "complexity",
            "error",
            "src/parser.rs",
            Some(42),
            "Cyclomatic complexity 25 exceeds threshold 20",
        ),
        QualityViolation::new(
            "complexity",
            "warning",
            "src/analyzer.rs",
            Some(100),
            "Cyclomatic complexity 18 approaching threshold",
        ),
        QualityViolation::new(
            "dead_code",
            "warning",
            "src/utils.rs",
            Some(55),
            "Function 'unused_helper' is never called",
        ),
        QualityViolation::new(
            "satd",
            "info",
            "src/main.rs",
            Some(10),
            "TODO: Refactor this function",
        ),
        // Entropy violation WITH explainability details (#226)
        // Note: structural code hashing groups only structurally identical patterns,
        // so variation_score is 0.0 (all matches are identical after normalization).
        QualityViolation::new(
            "entropy",
            "warning",
            "src/config.rs",
            None,
            "ResourceManagement pattern repeated 8 times (saves 120 lines)",
        )
        .with_details(ViolationDetails {
            affected_files: vec!["src/config.rs".to_string(), "src/settings.rs".to_string()],
            example_code: Some("let guard = mutex.lock()".to_string()),
            fix_suggestion: Some("Extract shared resource guard helper".to_string()),
            score_factors: vec![
                "pattern_type: ResourceManagement".to_string(),
                "repetitions: 8".to_string(),
                "variation_score: 0.0 (structurally identical)".to_string(),
            ],
        }),
    ];

    // Demo: Summary format (now shows violations)
    println!("--- Summary Format (default) ---\n");
    let summary =
        format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Summary)
            .expect("formatting should work");
    println!("{}", summary);

    // Demo: JSON format (clean output with ViolationDetails)
    println!("\n--- JSON Format (clean, no progress lines - #230) ---\n");
    let json = format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Json)
        .expect("formatting should work");
    println!("{}", json);

    // Demo: Detailed format (full violation list)
    println!("\n--- Detailed Format ---\n");
    let detailed =
        format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Detailed)
            .expect("formatting should work");
    println!("{}", detailed);

    println!("\n=== End of Demo ===");
    println!("\nDemonstrates fixes for #129 (violation reporting), #226 (entropy explainability),");
    println!("and #230 (clean JSON output).");
}
