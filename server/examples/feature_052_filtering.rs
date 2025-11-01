//! Feature #52: Include/Exclude File Filtering Example
//!
//! This example demonstrates how to filter defect reports using glob patterns:
//! - `--include` patterns (only include matching files)
//! - `--exclude` patterns (exclude matching files)
//! - `--min-lines` threshold (exclude small files)
//!
//! Run this example:
//! ```bash
//! cargo run --example feature_052_filtering
//! ```

use pmat::models::defect_report::{
    Defect, DefectCategory, DefectReport, DefectSummary, ReportMetadata, Severity,
};
use pmat::services::defect_report_service::DefectReportService;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

fn main() {
    println!("=== Feature #52: File Filtering with Glob Patterns ===\n");

    // Create a sample defect report with multiple files
    let report = create_sample_report();

    println!("📊 Original Report:");
    print_report_summary(&report);

    // Example 1: Include only src/*.rs files
    println!("\n=== Example 1: Include Pattern ===");
    println!("Pattern: --include 'src/*.rs'");
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("src/*.rs".to_string()),
        None,
        0,
    );
    print_report_summary(&filtered);

    // Example 2: Exclude tests/* files
    println!("\n=== Example 2: Exclude Pattern ===");
    println!("Pattern: --exclude 'tests/*'");
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        None,
        Some("tests/*".to_string()),
        0,
    );
    print_report_summary(&filtered);

    // Example 3: Combined include + exclude
    println!("\n=== Example 3: Combined Include + Exclude ===");
    println!("Pattern: --include '**/*.rs' --exclude 'tests/*'");
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("**/*.rs".to_string()),
        Some("tests/*".to_string()),
        0,
    );
    print_report_summary(&filtered);

    // Example 4: Recursive glob pattern
    println!("\n=== Example 4: Recursive Glob Pattern ===");
    println!("Pattern: --include 'src/**/*.rs'");
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("src/**/*.rs".to_string()),
        None,
        0,
    );
    print_report_summary(&filtered);

    println!("\n✅ Feature #52 demonstration complete!");
    println!("\n💡 Use in CLI:");
    println!("   pmat analyze comprehensive --include 'src/*.rs' --exclude 'tests/*' --min-lines 50");
}

/// Create a sample defect report for demonstration
fn create_sample_report() -> DefectReport {
    let defects = vec![
        Defect {
            id: "D001".to_string(),
            severity: Severity::High,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("src/main.rs"),
            line_start: 10,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "High cyclomatic complexity (CC=15)".to_string(),
            rule_id: "COMPLEXITY001".to_string(),
            fix_suggestion: Some("Consider refactoring into smaller functions".to_string()),
            metrics: HashMap::new(),
        },
        Defect {
            id: "D002".to_string(),
            severity: Severity::Medium,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("src/lib.rs"),
            line_start: 42,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Medium cyclomatic complexity (CC=8)".to_string(),
            rule_id: "COMPLEXITY002".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        },
        Defect {
            id: "D003".to_string(),
            severity: Severity::Low,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from("tests/integration_test.rs"),
            line_start: 100,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Unused function detected".to_string(),
            rule_id: "DEADCODE001".to_string(),
            fix_suggestion: Some("Remove unused function".to_string()),
            metrics: HashMap::new(),
        },
        Defect {
            id: "D004".to_string(),
            severity: Severity::Medium,
            category: DefectCategory::Duplication,
            file_path: PathBuf::from("benches/benchmark.rs"),
            line_start: 50,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Duplicate code block (20 lines)".to_string(),
            rule_id: "DUPLICATION001".to_string(),
            fix_suggestion: Some("Extract common code into helper function".to_string()),
            metrics: HashMap::new(),
        },
        Defect {
            id: "D005".to_string(),
            severity: Severity::High,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("src/utils/helpers.rs"),
            line_start: 75,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Very high cyclomatic complexity (CC=22)".to_string(),
            rule_id: "COMPLEXITY003".to_string(),
            fix_suggestion: Some("Refactor function using Strategy pattern".to_string()),
            metrics: HashMap::new(),
        },
    ];

    // Build file index
    let mut file_index = BTreeMap::new();
    for defect in &defects {
        file_index
            .entry(defect.file_path.clone())
            .or_insert_with(Vec::new)
            .push(defect.id.clone());
    }

    // Build summary
    let mut by_severity = BTreeMap::new();
    by_severity.insert("high".to_string(), 2);
    by_severity.insert("medium".to_string(), 2);
    by_severity.insert("low".to_string(), 1);

    let mut by_category = BTreeMap::new();
    by_category.insert("complexity".to_string(), 3);
    by_category.insert("dead_code".to_string(), 1);
    by_category.insert("duplication".to_string(), 1);

    DefectReport {
        metadata: ReportMetadata {
            tool: "pmat".to_string(),
            version: "2.191.0".to_string(),
            generated_at: chrono::Utc::now(),
            project_root: PathBuf::from("."),
            total_files_analyzed: file_index.len(),
            analysis_duration_ms: 100,
        },
        summary: DefectSummary {
            total_defects: defects.len(),
            by_severity,
            by_category,
            hotspot_files: vec![],
        },
        defects,
        file_index,
    }
}

/// Print a summary of the defect report
fn print_report_summary(report: &DefectReport) {
    println!("  Total defects: {}", report.summary.total_defects);
    println!("  Files analyzed: {}", report.metadata.total_files_analyzed);
    println!("  Files with defects:");
    for (file_path, defect_ids) in &report.file_index {
        println!(
            "    - {} ({} defects)",
            file_path.display(),
            defect_ids.len()
        );
    }
}
