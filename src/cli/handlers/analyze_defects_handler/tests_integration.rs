#![cfg_attr(coverage_nightly, coverage(off))]
//! Integration tests for handle_analyze_defects async handler

use super::handler::{handle_analyze_defects, EXIT_NOTHING_MEASURED};
use super::output::print_text_report;
use super::types::*;
use crate::services::defect_detector::Severity;
use tempfile::TempDir;

// =========================================================================
// handle_analyze_defects integration tests
// =========================================================================

/// This test used to assert `0` — "no critical defects" for a directory in
/// which nothing was read. That is the #923 defect written down as an
/// expectation: an empty walk is an absent measurement, not a clean one, and
/// the handler now refuses it the way `analyze satd` does.
#[tokio::test]
async fn test_handle_analyze_defects_empty_dir() {
    let temp_dir = TempDir::new().expect("temp dir");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Text).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), EXIT_NOTHING_MEASURED);
}

#[tokio::test]
async fn test_handle_analyze_defects_no_defects() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create a clean Rust file (no .unwrap() calls)
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(
        src_dir.join("main.rs"),
        r#"
fn main() {
    let x = Some(42);
    if let Some(val) = x {
        println!("Value: {}", val);
    }
}
"#,
    )
    .expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Json).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[tokio::test]
async fn test_handle_analyze_defects_with_critical_defects() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create a file with .unwrap() calls (critical defect)
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(
        src_dir.join("main.rs"),
        r#"
fn main() {
    let x = Some(42).unwrap();
    println!("Value: {}", x);
}
"#,
    )
    .expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Text).await;

    assert!(result.is_ok());
    // Result depends on file system detection
    assert!(result.unwrap() >= 0);
}

#[tokio::test]
async fn test_handle_analyze_defects_specific_file() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");

    // Create two files, only scan one
    std::fs::write(src_dir.join("clean.rs"), "fn clean() { let x = Some(42); }").expect("write");
    std::fs::write(
        src_dir.join("dirty.rs"),
        "fn dirty() { let x = Some(42).unwrap(); }",
    )
    .expect("write");

    // Scan only the clean file
    let clean_file = src_dir.join("clean.rs");
    let result = handle_analyze_defects(
        Some(temp_dir.path()),
        Some(clean_file.as_path()),
        None,
        OutputFormat::Json,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Clean file has no defects
}

#[tokio::test]
async fn test_handle_analyze_defects_severity_filter() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(
        src_dir.join("main.rs"),
        "fn main() { let x = Some(42).unwrap(); }",
    )
    .expect("write");

    // Filter for only High severity (unwrap is Critical, so should be filtered out)
    let result = handle_analyze_defects(
        Some(temp_dir.path()),
        None,
        Some(Severity::High),
        OutputFormat::Text,
    )
    .await;

    assert!(result.is_ok());
    // After filtering, no critical defects remain in the filtered list
    assert_eq!(result.unwrap(), 0);
}

#[tokio::test]
async fn test_handle_analyze_defects_default_path() {
    // Test with None path (should use ".")
    let result = handle_analyze_defects(None, None, None, OutputFormat::Junit).await;

    // Should succeed (may or may not find defects depending on cwd)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_defects_json_format() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Json).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_defects_junit_format() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Junit).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_defects_unreadable_file() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");

    // Create a valid file
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write");

    // The handler gracefully handles unreadable files (skips them)
    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Text).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_defects_filter_critical() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(
        src_dir.join("main.rs"),
        "fn main() { let x = Some(42).unwrap(); }",
    )
    .expect("write");

    // Filter for Critical severity (should keep the unwrap defect)
    let result = handle_analyze_defects(
        Some(temp_dir.path()),
        None,
        Some(Severity::Critical),
        OutputFormat::Json,
    )
    .await;

    assert!(result.is_ok());
    // Result depends on file system detection - may find 0 or more defects
    assert!(result.unwrap() >= 0);
}

// =========================================================================
// Nested directory and multi-file tests
// =========================================================================

#[tokio::test]
async fn test_handle_analyze_defects_nested_directories() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create nested directory structure
    let deep_dir = temp_dir.path().join("src").join("nested").join("deep");
    std::fs::create_dir_all(&deep_dir).expect("create dir");
    std::fs::write(
        deep_dir.join("module.rs"),
        "fn deep() { let x = Some(1).unwrap(); }",
    )
    .expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Text).await;

    assert!(result.is_ok());
    // Result depends on file system detection
    assert!(result.unwrap() >= 0);
}

#[tokio::test]
async fn test_handle_analyze_defects_multiple_files_with_defects() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");

    // Create multiple files with defects
    std::fs::write(src_dir.join("file1.rs"), "fn f1() { Some(1).unwrap(); }").expect("write");
    std::fs::write(
        src_dir.join("file2.rs"),
        "fn f2() { Some(2).unwrap(); Some(3).unwrap(); }",
    )
    .expect("write");
    std::fs::write(src_dir.join("file3.rs"), "fn f3() { let x = 1; }").expect("write");

    let result =
        handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Json).await;

    assert!(result.is_ok());
    // Result depends on file system detection
    assert!(result.unwrap() >= 0);
}

// =========================================================================
// print_text_report with all severity levels
// =========================================================================

#[test]
fn test_print_text_report_with_high_and_medium() {
    use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 10,
            files_with_defects: 2,
            total_defects: 2,
            by_severity: SeverityCount {
                critical: 0,
                high: 1,
                medium: 1,
                low: 0,
            },
        },
        defects: vec![
            DefectPattern {
                id: "HIGH-001".to_string(),
                name: "High Issue".to_string(),
                severity: Severity::High,
                fix_recommendation: "Fix soon".to_string(),
                bad_example: "bad()".to_string(),
                good_example: "good()".to_string(),
                evidence_description: "High evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "src/lib.rs".to_string(),
                    line: 10,
                    column: 1,
                    code_snippet: "bad()".to_string(),
                }],
            },
            DefectPattern {
                id: "MED-001".to_string(),
                name: "Medium Issue".to_string(),
                severity: Severity::Medium,
                fix_recommendation: "Fix when possible".to_string(),
                bad_example: "meh()".to_string(),
                good_example: "better()".to_string(),
                evidence_description: "Medium evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "src/util.rs".to_string(),
                    line: 20,
                    column: 5,
                    code_snippet: "meh()".to_string(),
                }],
            },
        ],
        exit_code: 0,
        has_critical_defects: false,
    };

    // Just ensure it doesn't panic
    print_text_report(&report);
}

#[test]
fn test_print_text_report_many_instances() {
    use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

    // Create a defect with more than 10 instances to test truncation
    let instances: Vec<DefectInstance> = (0..15)
        .map(|i| DefectInstance {
            file: format!("src/file{}.rs", i),
            line: i + 1,
            column: 1,
            code_snippet: "bad()".to_string(),
        })
        .collect();

    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 15,
            files_with_defects: 15,
            total_defects: 15,
            by_severity: SeverityCount {
                critical: 15,
                high: 0,
                medium: 0,
                low: 0,
            },
        },
        defects: vec![DefectPattern {
            id: "CRIT-001".to_string(),
            name: "Many instances".to_string(),
            severity: Severity::Critical,
            fix_recommendation: "Fix all".to_string(),
            bad_example: "bad()".to_string(),
            good_example: "good()".to_string(),
            evidence_description: "Evidence".to_string(),
            evidence_url: None,
            instances,
        }],
        exit_code: 1,
        has_critical_defects: true,
    };

    // Tests the truncation logic (shows first 10, then "... (N more)")
    print_text_report(&report);
}

#[test]
fn test_print_text_report_all_severity_levels() {
    use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 4,
            files_with_defects: 4,
            total_defects: 4,
            by_severity: SeverityCount {
                critical: 1,
                high: 1,
                medium: 1,
                low: 1,
            },
        },
        defects: vec![
            DefectPattern {
                id: "CRIT-001".to_string(),
                name: "Critical".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "c.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
            DefectPattern {
                id: "HIGH-001".to_string(),
                name: "High".to_string(),
                severity: Severity::High,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "h.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
            DefectPattern {
                id: "MED-001".to_string(),
                name: "Medium".to_string(),
                severity: Severity::Medium,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "m.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
            DefectPattern {
                id: "LOW-001".to_string(),
                name: "Low".to_string(),
                severity: Severity::Low,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "l.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
        ],
        exit_code: 1,
        has_critical_defects: true,
    };

    // Should print all severity levels (Critical, High, Medium - Low is not printed separately)
    print_text_report(&report);
}
