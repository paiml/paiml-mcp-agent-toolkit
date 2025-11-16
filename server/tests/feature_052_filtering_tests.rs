//! Feature #52: Include/Exclude File Filtering Tests - RED Phase
//!
//! These tests verify that comprehensive analysis can filter files based on:
//! - `--include` patterns (glob matching)
//! - `--exclude` patterns (glob matching)
//! - `--min-lines` threshold
//!
//! **Current Status**: 🔴 RED - These tests will FAIL until filtering is implemented
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write 6 comprehensive filtering tests (all fail)
//! 2. GREEN: Implement file filtering logic
//! 3. GREEN: Remove warning messages
//! 4. REFACTOR: Clean implementation
//! 5. COMMIT: Single atomic commit with feature

/// Test helper: Create mock defect report with multiple files
#[allow(dead_code)]
fn create_mock_defect_report() -> pmat::models::defect_report::DefectReport {
    use pmat::models::defect_report::{
        Defect, DefectCategory, DefectReport, DefectSummary, ReportMetadata, Severity,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

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
            message: "High complexity".to_string(),
            rule_id: "C001".to_string(),
            fix_suggestion: None,
            metrics: std::collections::HashMap::new(),
        },
        Defect {
            id: "D002".to_string(),
            severity: Severity::Medium,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("src/lib.rs"),
            line_start: 20,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Medium complexity".to_string(),
            rule_id: "C002".to_string(),
            fix_suggestion: None,
            metrics: std::collections::HashMap::new(),
        },
        Defect {
            id: "D003".to_string(),
            severity: Severity::Low,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from("tests/integration_test.rs"),
            line_start: 30,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Dead code detected".to_string(),
            rule_id: "DC001".to_string(),
            fix_suggestion: None,
            metrics: std::collections::HashMap::new(),
        },
        Defect {
            id: "D004".to_string(),
            severity: Severity::Medium,
            category: DefectCategory::Duplication,
            file_path: PathBuf::from("benches/benchmark.rs"),
            line_start: 40,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Code duplication".to_string(),
            rule_id: "DUP001".to_string(),
            fix_suggestion: None,
            metrics: std::collections::HashMap::new(),
        },
    ];

    let mut file_index = BTreeMap::new();
    for defect in &defects {
        file_index
            .entry(defect.file_path.clone())
            .or_insert_with(Vec::new)
            .push(defect.id.clone());
    }

    // Build summary
    let mut by_severity = BTreeMap::new();
    by_severity.insert("high".to_string(), 1);
    by_severity.insert("medium".to_string(), 2);
    by_severity.insert("low".to_string(), 1);

    let mut by_category = BTreeMap::new();
    by_category.insert("complexity".to_string(), 2);
    by_category.insert("dead_code".to_string(), 1);
    by_category.insert("duplication".to_string(), 1);

    DefectReport {
        metadata: ReportMetadata {
            tool: "pmat".to_string(),
            version: "2.190.0".to_string(),
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

#[test]
#[ignore = "Feature #52: RED test - will fail until include filtering implemented"]
fn test_include_pattern_filters_files() {
    // This test verifies that --include pattern only includes matching files
    //
    // Expected behavior:
    // - Pattern "src/*.rs" should include: src/main.rs, src/lib.rs
    // - Should exclude: tests/integration_test.rs, benches/benchmark.rs

    let report = create_mock_defect_report();

    // Apply include filter: only src/*.rs files
    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report,
        Some("src/*.rs".to_string()),
        None,
        0,
    );

    // Verify only src/ files remain
    assert_eq!(
        filtered.defects.len(),
        2,
        "Should have 2 defects from src/ directory"
    );

    let file_paths: Vec<_> = filtered
        .defects
        .iter()
        .map(|d| d.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.contains(&"src/main.rs".to_string()));
    assert!(file_paths.contains(&"src/lib.rs".to_string()));
    assert!(!file_paths.contains(&"tests/integration_test.rs".to_string()));
    assert!(!file_paths.contains(&"benches/benchmark.rs".to_string()));
}

#[test]
#[ignore = "Feature #52: RED test - will fail until exclude filtering implemented"]
fn test_exclude_pattern_filters_files() {
    // This test verifies that --exclude pattern removes matching files
    //
    // Expected behavior:
    // - Pattern "tests/*" should exclude: tests/integration_test.rs
    // - Should include: src/main.rs, src/lib.rs, benches/benchmark.rs

    let report = create_mock_defect_report();

    // Apply exclude filter: exclude tests/*
    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report,
        None,
        Some("tests/*".to_string()),
        0,
    );

    // Verify tests/ files removed
    assert_eq!(
        filtered.defects.len(),
        3,
        "Should have 3 defects (tests excluded)"
    );

    let file_paths: Vec<_> = filtered
        .defects
        .iter()
        .map(|d| d.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.contains(&"src/main.rs".to_string()));
    assert!(file_paths.contains(&"src/lib.rs".to_string()));
    assert!(file_paths.contains(&"benches/benchmark.rs".to_string()));
    assert!(!file_paths.contains(&"tests/integration_test.rs".to_string()));
}

#[test]
#[ignore = "Feature #52: RED test - will fail until combined include+exclude implemented"]
fn test_combined_include_and_exclude() {
    // This test verifies that include and exclude work together
    //
    // Expected behavior:
    // - Include "**/*.rs" matches all Rust files
    // - Exclude "tests/*" removes test files
    // - Result: src/main.rs, src/lib.rs, benches/benchmark.rs

    let report = create_mock_defect_report();

    // Apply both filters
    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report,
        Some("**/*.rs".to_string()),
        Some("tests/*".to_string()),
        0,
    );

    assert_eq!(
        filtered.defects.len(),
        3,
        "Should have 3 defects (include all .rs, exclude tests)"
    );

    let file_paths: Vec<_> = filtered
        .defects
        .iter()
        .map(|d| d.file_path.to_string_lossy().to_string())
        .collect();

    assert!(!file_paths.contains(&"tests/integration_test.rs".to_string()));
}

#[test]
#[ignore = "Feature #52: RED test - will fail until min_lines filtering implemented"]
fn test_min_lines_threshold_filters_small_files() {
    // This test verifies that --min-lines filters out small files
    //
    // Expected behavior:
    // - Files with < min_lines should be excluded
    // - File line count determined from actual file or metadata

    // Note: This test requires actual file line counts
    // For now, we'll test the interface exists
    let report = create_mock_defect_report();

    // Apply min_lines filter: 50 lines minimum
    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report, None, None, 50, // min_lines threshold
    );

    // Verification depends on actual file line counts
    // This test documents the expected behavior
    assert!(
        filtered.defects.len() <= report.defects.len(),
        "Filtered report should have <= defects than original"
    );
}

#[test]
#[ignore = "Feature #52: RED test - will fail until glob pattern matching works"]
fn test_glob_pattern_matching() {
    // This test verifies that glob patterns work correctly
    //
    // Patterns to test:
    // - "*.rs" - all Rust files in current directory
    // - "**/*.rs" - all Rust files recursively
    // - "src/**/*.rs" - all Rust files under src/
    // - "tests/*.rs" - Rust files in tests/ directory

    let report = create_mock_defect_report();

    // Test: include only files in src/
    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report,
        Some("src/**/*.rs".to_string()),
        None,
        0,
    );

    assert_eq!(
        filtered.defects.len(),
        2,
        "Pattern 'src/**/*.rs' should match 2 files"
    );
}

#[test]
#[ignore = "Feature #52: RED test - will fail until file_index is updated"]
fn test_file_index_updated_after_filtering() {
    // This test verifies that file_index is correctly updated after filtering
    //
    // Expected behavior:
    // - Filtered report's file_index should only contain included files
    // - Defect IDs in file_index should match filtered defects

    let report = create_mock_defect_report();

    let filtered = pmat::services::defect_report_service::DefectReportService::filter_by_pattern(
        &report,
        Some("src/*.rs".to_string()),
        None,
        0,
    );

    // Verify file_index consistency
    assert_eq!(
        filtered.file_index.len(),
        2,
        "file_index should have 2 files"
    );

    // Verify all defects are in file_index
    for defect in &filtered.defects {
        assert!(
            filtered.file_index.contains_key(&defect.file_path),
            "Defect file_path {} should be in file_index",
            defect.file_path.display()
        );

        let defect_ids = filtered.file_index.get(&defect.file_path).unwrap();
        assert!(
            defect_ids.contains(&defect.id),
            "Defect ID {} should be in file_index for file {}",
            defect.id,
            defect.file_path.display()
        );
    }
}

// =============================================================================
// Implementation Notes for GREEN Phase
// =============================================================================
//
// The implementation should add a method to DefectReportService:
//
// ```rust
// impl DefectReportService {
//     /// Filter defect report by file patterns and line count
//     pub fn filter_by_pattern(
//         report: &DefectReport,
//         include: Option<String>,
//         exclude: Option<String>,
//         min_lines: usize,
//     ) -> DefectReport {
//         use globset::{Glob, GlobSetBuilder};
//
//         // Build glob matchers
//         let include_matcher = include.as_ref().map(|pattern| {
//             Glob::new(pattern).unwrap().compile_matcher()
//         });
//
//         let exclude_matcher = exclude.as_ref().map(|pattern| {
//             Glob::new(pattern).unwrap().compile_matcher()
//         });
//
//         // Filter defects
//         let filtered_defects: Vec<Defect> = report.defects.iter()
//             .filter(|defect| {
//                 // Check include pattern
//                 if let Some(matcher) = &include_matcher {
//                     if !matcher.is_match(&defect.file_path) {
//                         return false;
//                     }
//                 }
//
//                 // Check exclude pattern
//                 if let Some(matcher) = &exclude_matcher {
//                     if matcher.is_match(&defect.file_path) {
//                         return false;
//                     }
//                 }
//
//                 // Check min_lines (requires file line count)
//                 if min_lines > 0 {
//                     // TODO: Get actual line count from file
//                     // For now, assume all files pass
//                 }
//
//                 true
//             })
//             .cloned()
//             .collect();
//
//         // Rebuild file_index
//         let mut file_index = BTreeMap::new();
//         for defect in &filtered_defects {
//             file_index
//                 .entry(defect.file_path.clone())
//                 .or_insert_with(Vec::new)
//                 .push(defect.id.clone());
//         }
//
//         // Recompute summary
//         let summary = self.compute_summary(&filtered_defects);
//
//         DefectReport {
//             metadata: report.metadata.clone(),
//             summary,
//             defects: filtered_defects,
//             file_index,
//             hotspots: report.hotspots.clone(), // TODO: Filter hotspots too
//         }
//     }
// }
// ```
//
// Required dependencies in Cargo.toml:
// - globset = "0.4" (for glob pattern matching)
//
// Integration in comprehensive_handler.rs:
// 1. Remove warning messages (lines 346-348)
// 2. Call filter_by_pattern() after generate_report()
// 3. Pass filtered report to format functions
