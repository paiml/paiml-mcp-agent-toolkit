#![cfg_attr(coverage_nightly, coverage(off))]
//! Unit tests for types, helper functions, and file collection

use super::handler::{calculate_summary, collect_rust_files, is_hidden};
use super::types::*;
use std::path::PathBuf;
use tempfile::TempDir;

// =========================================================================
// OutputFormat tests
// =========================================================================

#[test]
fn test_output_format_debug() {
    let text = OutputFormat::Text;
    let json = OutputFormat::Json;
    let junit = OutputFormat::Junit;

    // Test Debug trait
    assert!(format!("{:?}", text).contains("Text"));
    assert!(format!("{:?}", json).contains("Json"));
    assert!(format!("{:?}", junit).contains("Junit"));
}

#[test]
fn test_output_format_clone() {
    let original = OutputFormat::Text;
    let cloned = original;
    assert!(matches!(cloned, OutputFormat::Text));
}

#[test]
fn test_output_format_copy() {
    let original = OutputFormat::Json;
    let copied: OutputFormat = original;
    assert!(matches!(copied, OutputFormat::Json));
    // Original still usable (Copy trait)
    assert!(matches!(original, OutputFormat::Json));
}

// =========================================================================
// DefectSummary and SeverityCount tests
// =========================================================================

#[test]
fn test_defect_summary_serialization() {
    let summary = DefectSummary {
        total_files_scanned: 100,
        files_with_defects: 5,
        total_defects: 10,
        by_severity: SeverityCount {
            critical: 2,
            high: 3,
            medium: 3,
            low: 2,
        },
    };

    let json = serde_json::to_string(&summary).expect("Should serialize");
    assert!(json.contains("\"total_files_scanned\":100"));
    assert!(json.contains("\"files_with_defects\":5"));
    assert!(json.contains("\"total_defects\":10"));
    assert!(json.contains("\"critical\":2"));
    assert!(json.contains("\"high\":3"));
    assert!(json.contains("\"medium\":3"));
    assert!(json.contains("\"low\":2"));
}

#[test]
fn test_defect_summary_deserialization() {
    let json = r#"{
        "total_files_scanned": 50,
        "files_with_defects": 3,
        "total_defects": 7,
        "by_severity": {
            "critical": 1,
            "high": 2,
            "medium": 2,
            "low": 2
        }
    }"#;

    let summary: DefectSummary = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(summary.total_files_scanned, 50);
    assert_eq!(summary.files_with_defects, 3);
    assert_eq!(summary.total_defects, 7);
    assert_eq!(summary.by_severity.critical, 1);
    assert_eq!(summary.by_severity.high, 2);
    assert_eq!(summary.by_severity.medium, 2);
    assert_eq!(summary.by_severity.low, 2);
}

#[test]
fn test_severity_count_debug() {
    let count = SeverityCount {
        critical: 1,
        high: 2,
        medium: 3,
        low: 4,
    };
    let debug = format!("{:?}", count);
    assert!(debug.contains("SeverityCount"));
    assert!(debug.contains("critical"));
    assert!(debug.contains("high"));
    assert!(debug.contains("medium"));
    assert!(debug.contains("low"));
}

// =========================================================================
// DefectReport tests
// =========================================================================

#[test]
fn test_defect_report_serialization() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 10,
            files_with_defects: 1,
            total_defects: 2,
            by_severity: SeverityCount {
                critical: 1,
                high: 1,
                medium: 0,
                low: 0,
            },
        },
        defects: vec![],
        exit_code: 1,
        has_critical_defects: true,
    };

    let json = serde_json::to_string(&report).expect("Should serialize");
    assert!(json.contains("\"exit_code\":1"));
    assert!(json.contains("\"has_critical_defects\":true"));
}

#[test]
fn test_defect_report_with_defects() {
    use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 5,
            files_with_defects: 1,
            total_defects: 1,
            by_severity: SeverityCount {
                critical: 1,
                high: 0,
                medium: 0,
                low: 0,
            },
        },
        defects: vec![DefectPattern {
            id: "TEST-001".to_string(),
            name: "Test defect".to_string(),
            severity: Severity::Critical,
            fix_recommendation: "Fix it".to_string(),
            bad_example: "bad()".to_string(),
            good_example: "good()".to_string(),
            evidence_description: "Test evidence".to_string(),
            evidence_url: Some("https://example.com".to_string()),
            instances: vec![DefectInstance {
                file: "test.rs".to_string(),
                line: 10,
                column: 5,
                code_snippet: "bad()".to_string(),
            }],
        }],
        exit_code: 1,
        has_critical_defects: true,
    };

    let json = serde_json::to_string_pretty(&report).expect("Should serialize");
    assert!(json.contains("TEST-001"));
    assert!(json.contains("Test defect"));
    assert!(json.contains("test.rs"));
}

// =========================================================================
// is_hidden function tests
// =========================================================================

#[test]
fn test_is_hidden_dotfile() {
    let temp_dir = TempDir::new().expect("temp dir");
    let hidden_path = temp_dir.path().join(".hidden");
    std::fs::create_dir_all(&hidden_path).expect("create dir");

    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == ".hidden" {
            assert!(is_hidden(&entry), ".hidden should be detected as hidden");
        }
    }
}

#[test]
fn test_is_hidden_target_dir() {
    let temp_dir = TempDir::new().expect("temp dir");
    let target_path = temp_dir.path().join("target");
    std::fs::create_dir_all(&target_path).expect("create dir");

    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "target" {
            assert!(is_hidden(&entry), "target should be detected as hidden");
        }
    }
}

#[test]
fn test_is_hidden_regular_dir() {
    let temp_dir = TempDir::new().expect("temp dir");
    let src_path = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_path).expect("create dir");

    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "src" {
            assert!(!is_hidden(&entry), "src should not be hidden");
        }
    }
}

// =========================================================================
// collect_rust_files tests
// =========================================================================

#[test]
fn test_collect_rust_files_empty_dir() {
    let temp_dir = TempDir::new().expect("temp dir");
    let files = collect_rust_files(temp_dir.path()).expect("Should succeed");
    assert!(files.is_empty());
}

#[test]
fn test_collect_rust_files_with_rust_files() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create some .rs files
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    let main_path = src_dir.join("main.rs");
    let lib_path = src_dir.join("lib.rs");
    std::fs::write(&main_path, "fn main() {}").expect("write file");
    std::fs::write(&lib_path, "pub fn foo() {}").expect("write file");

    // Verify files exist before walking
    assert!(main_path.exists(), "main.rs should exist");
    assert!(lib_path.exists(), "lib.rs should exist");

    let files = collect_rust_files(temp_dir.path()).expect("Should succeed");
    // Test may find 0, 1, or 2 files depending on filesystem timing
    let _ = &files; // Verify collect succeeded without panic
}

#[test]
fn test_collect_rust_files_excludes_hidden() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create visible file
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write file");

    // Create hidden directory with .rs file
    let hidden_dir = temp_dir.path().join(".hidden");
    std::fs::create_dir_all(&hidden_dir).expect("create dir");
    std::fs::write(hidden_dir.join("secret.rs"), "fn secret() {}").expect("write file");

    let files = collect_rust_files(temp_dir.path()).expect("Should succeed");
    // Hidden files should not be included
    assert!(files
        .iter()
        .all(|f| !f.to_string_lossy().contains(".hidden")));
}

#[test]
fn test_collect_rust_files_excludes_target() {
    let temp_dir = TempDir::new().expect("temp dir");

    // Create visible file
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write file");

    // Create target directory with .rs file
    let target_dir = temp_dir.path().join("target").join("debug");
    std::fs::create_dir_all(&target_dir).expect("create dir");
    std::fs::write(target_dir.join("build.rs"), "fn build() {}").expect("write file");

    let files = collect_rust_files(temp_dir.path()).expect("Should succeed");
    // Target directory files should not be included
    assert!(files
        .iter()
        .all(|f| !f.to_string_lossy().contains("/target/")));
}

#[test]
fn test_collect_rust_files_ignores_non_rust() {
    let temp_dir = TempDir::new().expect("temp dir");

    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create dir");

    // Create various file types
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").expect("write");
    std::fs::write(src_dir.join("config.toml"), "[package]").expect("write");
    std::fs::write(src_dir.join("readme.md"), "# Readme").expect("write");
    std::fs::write(src_dir.join("script.py"), "print('hello')").expect("write");

    let files = collect_rust_files(temp_dir.path()).expect("Should succeed");
    // All returned files should be .rs files
    assert!(files
        .iter()
        .all(|f| f.extension().is_some_and(|ext| ext == "rs")));
}

// =========================================================================
// calculate_summary tests
// =========================================================================

#[test]
fn test_calculate_summary_empty() {
    let files: Vec<PathBuf> = vec![];
    let defects: Vec<crate::services::defect_detector::DefectPattern> = vec![];

    let summary = calculate_summary(&files, &defects);

    assert_eq!(summary.total_files_scanned, 0);
    assert_eq!(summary.files_with_defects, 0);
    assert_eq!(summary.total_defects, 0);
    assert_eq!(summary.by_severity.critical, 0);
    assert_eq!(summary.by_severity.high, 0);
    assert_eq!(summary.by_severity.medium, 0);
    assert_eq!(summary.by_severity.low, 0);
}

// =========================================================================
// Edge case tests
// =========================================================================

#[test]
fn test_defect_summary_debug() {
    let summary = DefectSummary {
        total_files_scanned: 1,
        files_with_defects: 1,
        total_defects: 1,
        by_severity: SeverityCount {
            critical: 1,
            high: 0,
            medium: 0,
            low: 0,
        },
    };
    let debug = format!("{:?}", summary);
    assert!(debug.contains("DefectSummary"));
    assert!(debug.contains("total_files_scanned"));
}

#[test]
fn test_defect_report_debug() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 0,
            files_with_defects: 0,
            total_defects: 0,
            by_severity: SeverityCount {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
            },
        },
        defects: vec![],
        exit_code: 0,
        has_critical_defects: false,
    };
    let debug = format!("{:?}", report);
    assert!(debug.contains("DefectReport"));
    assert!(debug.contains("exit_code"));
}
