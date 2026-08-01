#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/defect_report_service/mod.rs
//! Targets: compute_summary, format_json, format_csv, format_markdown, format_text,
//! generate_filename, filter_by_pattern

use crate::models::defect_report::{
    Defect, DefectCategory, DefectReport, ReportMetadata, Severity,
};
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

fn make_defect(id: &str, sev: Severity, cat: DefectCategory, file: &str, line: u32) -> Defect {
    Defect {
        id: id.to_string(),
        severity: sev,
        category: cat,
        file_path: PathBuf::from(file),
        line_start: line,
        line_end: Some(line + 5),
        column_start: Some(1),
        column_end: Some(80),
        message: format!("Test defect {id}"),
        rule_id: format!("RULE-{id}"),
        fix_suggestion: Some(format!("Fix for {id}")),
        metrics: {
            let mut m = HashMap::new();
            m.insert("cyclomatic".to_string(), 15.0);
            m.insert("cognitive".to_string(), 12.0);
            m
        },
    }
}

fn make_report(defects: Vec<Defect>) -> DefectReport {
    let svc = DefectReportService::new();
    let summary = svc.compute_summary(&defects);
    let mut file_index = BTreeMap::new();
    for d in &defects {
        file_index
            .entry(d.file_path.clone())
            .or_insert_with(Vec::new)
            .push(d.id.clone());
    }
    DefectReport {
        metadata: ReportMetadata {
            tool: "pmat".to_string(),
            version: "2.0.0".to_string(),
            generated_at: Utc::now(),
            project_root: PathBuf::from("/test/project"),
            total_files_analyzed: file_index.len(),
            analysis_duration_ms: 1234,
        },
        defects,
        summary,
        file_index,
    }
}

fn sample_defects() -> Vec<Defect> {
    vec![
        make_defect(
            "D001",
            Severity::Critical,
            DefectCategory::Complexity,
            "src/main.rs",
            10,
        ),
        make_defect(
            "D002",
            Severity::High,
            DefectCategory::TechnicalDebt,
            "src/main.rs",
            50,
        ),
        make_defect(
            "D003",
            Severity::Medium,
            DefectCategory::DeadCode,
            "src/lib.rs",
            20,
        ),
        make_defect(
            "D004",
            Severity::Low,
            DefectCategory::Duplication,
            "src/utils.rs",
            30,
        ),
        make_defect(
            "D005",
            Severity::High,
            DefectCategory::Performance,
            "src/main.rs",
            100,
        ),
        make_defect(
            "D006",
            Severity::Medium,
            DefectCategory::Architecture,
            "src/mod.rs",
            5,
        ),
        make_defect(
            "D007",
            Severity::Critical,
            DefectCategory::Complexity,
            "tests/test.rs",
            1,
        ),
        make_defect(
            "D008",
            Severity::Low,
            DefectCategory::TestCoverage,
            "src/lib.rs",
            200,
        ),
    ]
}

// ==================== DefectReportService construction ====================

#[test]
fn test_defect_report_service_new() {
    let svc = DefectReportService::new();
    // Should not panic
    let _ = svc;
}

#[test]
fn test_defect_report_service_default() {
    let svc = DefectReportService::default();
    let _ = svc;
}

// ==================== compute_summary ====================

#[test]
fn test_compute_summary_empty() {
    let svc = DefectReportService::new();
    let summary = svc.compute_summary(&[]);
    assert_eq!(summary.total_defects, 0);
    assert!(summary.by_severity.is_empty());
    assert!(summary.by_category.is_empty());
    assert!(summary.hotspot_files.is_empty());
}

#[test]
fn test_compute_summary_single_defect() {
    let svc = DefectReportService::new();
    let defects = vec![make_defect(
        "D001",
        Severity::High,
        DefectCategory::Complexity,
        "src/main.rs",
        10,
    )];
    let summary = svc.compute_summary(&defects);
    assert_eq!(summary.total_defects, 1);
    assert_eq!(summary.hotspot_files.len(), 1);
    assert_eq!(summary.hotspot_files[0].defect_count, 1);
}

#[test]
fn test_compute_summary_multiple_defects() {
    let svc = DefectReportService::new();
    let defects = sample_defects();
    let summary = svc.compute_summary(&defects);
    assert_eq!(summary.total_defects, 8);
    assert!(!summary.by_severity.is_empty());
    assert!(!summary.by_category.is_empty());
    // Hotspots should be sorted by severity score descending
    if summary.hotspot_files.len() >= 2 {
        assert!(summary.hotspot_files[0].severity_score >= summary.hotspot_files[1].severity_score);
    }
}

#[test]
fn test_compute_summary_hotspot_limit() {
    let svc = DefectReportService::new();
    // Create defects across 15 different files
    let defects: Vec<Defect> = (0..15)
        .map(|i| {
            make_defect(
                &format!("D{:03}", i),
                Severity::Medium,
                DefectCategory::Complexity,
                &format!("src/file{i}.rs"),
                10,
            )
        })
        .collect();
    let summary = svc.compute_summary(&defects);
    // Should truncate to top 10
    assert!(summary.hotspot_files.len() <= 10);
}

#[test]
fn test_compute_summary_severity_counting() {
    let svc = DefectReportService::new();
    let defects = vec![
        make_defect(
            "D001",
            Severity::Critical,
            DefectCategory::Complexity,
            "a.rs",
            1,
        ),
        make_defect(
            "D002",
            Severity::Critical,
            DefectCategory::Complexity,
            "b.rs",
            1,
        ),
        make_defect("D003", Severity::Low, DefectCategory::DeadCode, "c.rs", 1),
    ];
    let summary = svc.compute_summary(&defects);
    assert_eq!(summary.total_defects, 3);
    assert_eq!(*summary.by_severity.get("critical").unwrap_or(&0), 2);
    assert_eq!(*summary.by_severity.get("low").unwrap_or(&0), 1);
}

// ==================== format_json ====================

#[test]
fn test_format_json_valid() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let json_str = svc.format_json(&report).unwrap();
    assert!(json_str.contains("D001"));
    assert!(json_str.contains("pmat"));
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_format_json_empty_report() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let json_str = svc.format_json(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["defects"].as_array().unwrap().len(), 0);
}

// ==================== format_csv ====================

#[test]
fn test_format_csv_valid() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let csv_str = svc.format_csv(&report).unwrap();
    // Should have header row
    assert!(csv_str.starts_with("id,severity,category,file_path,line_start"));
    // Should have data rows
    assert!(csv_str.contains("D001"));
    assert!(csv_str.contains("src/main.rs"));
}

#[test]
fn test_format_csv_empty() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let csv_str = svc.format_csv(&report).unwrap();
    // Should still have header
    assert!(csv_str.contains("id,severity"));
}

// Issue #672 regression: replaced a test that asserted `format_csv` errors
// without the non-default `reporting` feature. That was the shipped default,
// so `pmat report --format csv` failed for every installed user.
#[test]
fn test_format_csv_always_available_no_feature_gate() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let csv = svc
        .format_csv(&report)
        .expect("CSV must work in the default feature set (issue #672)");
    assert!(csv.contains("D001"));
}

// ==================== format_markdown ====================

#[test]
fn test_format_markdown_structure() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("# Code Quality Report"));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("**Total Defects**"));
    assert!(md.contains("### Severity Distribution"));
    assert!(md.contains("## Detailed Findings"));
}

#[test]
fn test_format_markdown_hotspots() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("Top 10 Hotspot Files"));
    assert!(md.contains("| Rank |"));
}

#[test]
fn test_format_markdown_fix_suggestions() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("Suggestion"));
}

#[test]
fn test_format_markdown_empty() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("# Code Quality Report"));
    assert!(md.contains("**Total Defects**: 0"));
}

// ==================== format_text ====================

#[test]
fn test_format_text_structure() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("CODE QUALITY REPORT"));
    assert!(txt.contains("SEVERITY BREAKDOWN"));
    assert!(txt.contains("CATEGORY BREAKDOWN"));
    assert!(txt.contains("DEFECTS"));
}

#[test]
fn test_format_text_hotspots() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("TOP HOTSPOT FILES"));
}

#[test]
fn test_format_text_defect_lines() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    // Check defect formatting: [Severity] Category - file:line
    assert!(txt.contains("src/main.rs"));
    assert!(txt.contains("Fix:"));
}

#[test]
fn test_format_text_empty() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("Total Defects: 0"));
}

// ==================== generate_filename ====================

#[test]
fn test_generate_filename_json() {
    let svc = DefectReportService::new();
    let name = svc.generate_filename(ReportFormat::Json);
    assert!(name.starts_with("defect-report-"));
    assert!(name.ends_with(".json"));
}

#[test]
fn test_generate_filename_csv() {
    let svc = DefectReportService::new();
    let name = svc.generate_filename(ReportFormat::Csv);
    assert!(name.ends_with(".csv"));
}

#[test]
fn test_generate_filename_markdown() {
    let svc = DefectReportService::new();
    let name = svc.generate_filename(ReportFormat::Markdown);
    assert!(name.ends_with(".md"));
}

#[test]
fn test_generate_filename_text() {
    let svc = DefectReportService::new();
    let name = svc.generate_filename(ReportFormat::Text);
    assert!(name.ends_with(".txt"));
}

// ==================== filter_by_pattern ====================

#[test]
fn test_filter_by_pattern_include() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, Some("src/main.rs".to_string()), None, 0);
    // Only main.rs defects should remain
    for d in &filtered.defects {
        assert_eq!(d.file_path, PathBuf::from("src/main.rs"));
    }
    assert!(filtered.defects.len() < report.defects.len());
}

#[test]
fn test_filter_by_pattern_exclude() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, None, Some("tests/*".to_string()), 0);
    for d in &filtered.defects {
        assert!(!d.file_path.starts_with("tests/"));
    }
}

#[test]
fn test_filter_by_pattern_both() {
    let report = make_report(sample_defects());
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("src/**".to_string()),
        Some("src/utils.rs".to_string()),
        0,
    );
    for d in &filtered.defects {
        assert!(d.file_path.starts_with("src/"));
        assert_ne!(d.file_path, PathBuf::from("src/utils.rs"));
    }
}

#[test]
fn test_filter_by_pattern_none() {
    let report = make_report(sample_defects());
    let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);
    assert_eq!(filtered.defects.len(), report.defects.len());
}

#[test]
fn test_filter_by_pattern_rebuilds_index() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, Some("src/main.rs".to_string()), None, 0);
    // file_index should only contain src/main.rs
    assert!(filtered
        .file_index
        .contains_key(&PathBuf::from("src/main.rs")));
    assert_eq!(
        filtered.metadata.total_files_analyzed,
        filtered.file_index.len()
    );
}

// ==================== ReportFormat ====================

#[test]
fn test_report_format_debug() {
    let fmt = ReportFormat::Json;
    let debug_str = format!("{:?}", fmt);
    assert_eq!(debug_str, "Json");
}

#[test]
fn test_report_format_clone() {
    let fmt = ReportFormat::Markdown;
    let cloned = fmt;
    assert!(matches!(cloned, ReportFormat::Markdown));
}

// ==================== Cross-format consistency ====================

#[test]
fn test_all_formats_contain_defect_data() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());

    let json = svc.format_json(&report).unwrap();
    let md = svc.format_markdown(&report).unwrap();
    let txt = svc.format_text(&report).unwrap();

    // All formats should reference the defects
    assert!(json.contains("D001"));
    assert!(md.contains("src/main.rs"));
    assert!(txt.contains("src/main.rs"));

    // Issue #672: CSV is unconditional, no feature gate.
    let csv = svc.format_csv(&report).unwrap();
    assert!(csv.contains("D001"));
}

// ==================== Edge cases ====================

#[test]
fn test_defect_without_fix_suggestion() {
    let mut defect = make_defect("D100", Severity::Low, DefectCategory::DeadCode, "a.rs", 1);
    defect.fix_suggestion = None;
    let svc = DefectReportService::new();
    let report = make_report(vec![defect]);

    // All format methods should handle None fix_suggestion
    let _ = svc.format_json(&report).unwrap();
    let _ = svc.format_markdown(&report).unwrap();
    let _ = svc.format_text(&report).unwrap();

    // Issue #672: CSV is unconditional, no feature gate.
    let _ = svc.format_csv(&report).unwrap();
}

#[test]
fn test_defect_without_line_end() {
    let mut defect = make_defect(
        "D101",
        Severity::Medium,
        DefectCategory::Complexity,
        "b.rs",
        42,
    );
    defect.line_end = None;
    let svc = DefectReportService::new();
    let report = make_report(vec![defect]);

    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("b.rs:42"));
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("b.rs:42"));
}

#[test]
fn test_many_defects_in_same_file() {
    let defects: Vec<Defect> = (0..50)
        .map(|i| {
            make_defect(
                &format!("D{:03}", i),
                Severity::Medium,
                DefectCategory::Complexity,
                "src/big_file.rs",
                i * 10,
            )
        })
        .collect();
    let svc = DefectReportService::new();
    let summary = svc.compute_summary(&defects);
    assert_eq!(summary.total_defects, 50);
    // Should have exactly 1 hotspot file
    assert_eq!(summary.hotspot_files.len(), 1);
    assert_eq!(summary.hotspot_files[0].defect_count, 50);
}
