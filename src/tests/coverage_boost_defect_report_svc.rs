#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/defect_report_service/mod.rs
//! Tests: compute_summary, format_json, format_csv, format_markdown, format_text,
//! generate_filename, filter_by_pattern, Default trait

use crate::models::defect_report::{
    Defect, DefectCategory, DefectReport, DefectSummary, FileHotspot, ReportMetadata, Severity,
};
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

// ============ Helpers ============

fn make_defect(id: &str, sev: Severity, cat: DefectCategory, file: &str) -> Defect {
    Defect {
        id: id.to_string(),
        severity: sev,
        category: cat,
        file_path: PathBuf::from(file),
        line_start: 10,
        line_end: Some(20),
        column_start: Some(1),
        column_end: Some(40),
        message: format!("{id} message"),
        rule_id: format!("{id}-rule"),
        fix_suggestion: Some(format!("Fix {id}")),
        metrics: {
            let mut m = HashMap::new();
            m.insert("cyclomatic".to_string(), 15.0);
            m.insert("cognitive".to_string(), 8.0);
            m
        },
    }
}

fn make_report(defects: Vec<Defect>) -> DefectReport {
    let mut by_severity = BTreeMap::new();
    let mut by_category = BTreeMap::new();
    for d in &defects {
        *by_severity
            .entry(format!("{:?}", d.severity).to_lowercase())
            .or_insert(0) += 1;
        *by_category
            .entry(format!("{:?}", d.category))
            .or_insert(0) += 1;
    }

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
            version: "2.200.0".to_string(),
            generated_at: Utc::now(),
            project_root: PathBuf::from("/test/project"),
            total_files_analyzed: 50,
            analysis_duration_ms: 3000,
        },
        summary: DefectSummary {
            total_defects: defects.len(),
            by_severity,
            by_category,
            hotspot_files: vec![FileHotspot {
                path: PathBuf::from("src/hot.rs"),
                defect_count: 5,
                severity_score: 25.0,
            }],
        },
        defects,
        file_index,
    }
}

fn sample_defects() -> Vec<Defect> {
    vec![
        make_defect("CPLX-001", Severity::High, DefectCategory::Complexity, "src/a.rs"),
        make_defect("CPLX-002", Severity::Medium, DefectCategory::Complexity, "src/b.rs"),
        make_defect("SATD-001", Severity::Low, DefectCategory::TechnicalDebt, "src/a.rs"),
        make_defect("DEAD-001", Severity::Critical, DefectCategory::DeadCode, "tests/t.rs"),
        make_defect("PERF-001", Severity::High, DefectCategory::Performance, "src/c.rs"),
    ]
}

// ============ DefectReportService::new & Default ============

#[test]
fn test_defect_report_service_new() {
    let svc = DefectReportService::new();
    let _ = svc.generate_filename(ReportFormat::Json);
}

#[test]
fn test_defect_report_service_default() {
    let svc = DefectReportService::default();
    let _ = svc.generate_filename(ReportFormat::Csv);
}

// ============ compute_summary ============

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
        "X-001",
        Severity::High,
        DefectCategory::Complexity,
        "src/x.rs",
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
    assert_eq!(summary.total_defects, 5);
    // Should have severity entries
    assert!(summary.by_severity.values().sum::<usize>() == 5);
    // Should have category entries
    assert!(summary.by_category.values().sum::<usize>() == 5);
    // Hotspots sorted by severity score descending
    assert!(!summary.hotspot_files.is_empty());
}

#[test]
fn test_compute_summary_hotspot_sorting() {
    let svc = DefectReportService::new();
    // Create multiple defects in same file to produce a high-score hotspot
    let defects = vec![
        make_defect("A-001", Severity::Critical, DefectCategory::Complexity, "src/hot.rs"),
        make_defect("A-002", Severity::Critical, DefectCategory::Performance, "src/hot.rs"),
        make_defect("B-001", Severity::Low, DefectCategory::DeadCode, "src/cold.rs"),
    ];
    let summary = svc.compute_summary(&defects);
    assert_eq!(summary.hotspot_files[0].path, PathBuf::from("src/hot.rs"));
    assert!(summary.hotspot_files[0].severity_score > summary.hotspot_files[1].severity_score);
}

#[test]
fn test_compute_summary_hotspot_truncation() {
    let svc = DefectReportService::new();
    // Create defects across 15 different files
    let defects: Vec<Defect> = (0..15)
        .map(|i| {
            make_defect(
                &format!("D-{:03}", i),
                Severity::Medium,
                DefectCategory::Complexity,
                &format!("src/file_{}.rs", i),
            )
        })
        .collect();
    let summary = svc.compute_summary(&defects);
    // Should be truncated to top 10
    assert!(summary.hotspot_files.len() <= 10);
}

// ============ format_json ============

#[test]
fn test_format_json_valid() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let json = svc.format_json(&report).unwrap();
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
    assert!(parsed["defects"].is_array());
    assert_eq!(parsed["defects"].as_array().unwrap().len(), 5);
}

#[test]
fn test_format_json_empty_report() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let json = svc.format_json(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["defects"].as_array().unwrap().len(), 0);
}

// ============ format_csv ============

#[cfg(feature = "reporting")]
#[test]
fn test_format_csv_headers() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let csv = svc.format_csv(&report).unwrap();
    let first_line = csv.lines().next().unwrap();
    assert!(first_line.contains("id"));
    assert!(first_line.contains("severity"));
    assert!(first_line.contains("category"));
    assert!(first_line.contains("file_path"));
    assert!(first_line.contains("message"));
}

#[cfg(feature = "reporting")]
#[test]
fn test_format_csv_row_count() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let csv = svc.format_csv(&report).unwrap();
    // header + 5 data rows
    let line_count = csv.lines().count();
    assert_eq!(line_count, 6);
}

#[cfg(feature = "reporting")]
#[test]
fn test_format_csv_empty_report() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let csv = svc.format_csv(&report).unwrap();
    // Just the header
    assert_eq!(csv.lines().count(), 1);
}

#[cfg(feature = "reporting")]
#[test]
fn test_format_csv_metrics_included() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let csv = svc.format_csv(&report).unwrap();
    // Second line should have cyclomatic and cognitive values
    let second_line = csv.lines().nth(1).unwrap();
    assert!(second_line.contains("15")); // cyclomatic
}

#[cfg(not(feature = "reporting"))]
#[test]
fn test_format_csv_requires_feature() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    assert!(svc.format_csv(&report).is_err());
}

// ============ format_markdown ============

#[test]
fn test_format_markdown_structure() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("# Code Quality Report"));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("### Severity Distribution"));
    assert!(md.contains("## Detailed Findings"));
}

#[test]
fn test_format_markdown_hotspots() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("### Top 10 Hotspot Files"));
    assert!(md.contains("| Rank | File | Defects | Severity Score |"));
}

#[test]
fn test_format_markdown_severity_bars() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    // Should contain severity distribution with bar characters
    assert!(md.contains("█") || md.contains("░"));
}

#[test]
fn test_format_markdown_fix_suggestions() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("Suggestion"));
}

#[test]
fn test_format_markdown_empty_report() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let md = svc.format_markdown(&report).unwrap();
    assert!(md.contains("# Code Quality Report"));
    assert!(md.contains("**Total Defects**: 0"));
}

// ============ format_text ============

#[test]
fn test_format_text_structure() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("CODE QUALITY REPORT"));
    assert!(txt.contains("==================="));
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
fn test_format_text_defect_details() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("CPLX-001 message"));
    assert!(txt.contains("Fix CPLX-001"));
}

#[test]
fn test_format_text_line_range() {
    let svc = DefectReportService::new();
    let report = make_report(sample_defects());
    let txt = svc.format_text(&report).unwrap();
    // Should display line range "10-20"
    assert!(txt.contains("10-20") || txt.contains(":10"));
}

#[test]
fn test_format_text_empty_report() {
    let svc = DefectReportService::new();
    let report = make_report(vec![]);
    let txt = svc.format_text(&report).unwrap();
    assert!(txt.contains("Total Defects: 0"));
}

// ============ generate_filename ============

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

// ============ filter_by_pattern ============

#[test]
fn test_filter_by_pattern_no_filters() {
    let report = make_report(sample_defects());
    let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);
    assert_eq!(filtered.defects.len(), 5);
}

#[test]
fn test_filter_by_pattern_include() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, Some("src/**".to_string()), None, 0);
    // Should exclude tests/t.rs
    assert_eq!(filtered.defects.len(), 4);
    assert!(filtered.defects.iter().all(|d| d.file_path.starts_with("src")));
}

#[test]
fn test_filter_by_pattern_exclude() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, None, Some("tests/**".to_string()), 0);
    // Should exclude tests/t.rs
    assert_eq!(filtered.defects.len(), 4);
}

#[test]
fn test_filter_by_pattern_include_and_exclude() {
    let report = make_report(sample_defects());
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("src/**".to_string()),
        Some("src/c.rs".to_string()),
        0,
    );
    // Include src/**, exclude src/c.rs -> should be 3
    assert_eq!(filtered.defects.len(), 3);
}

#[test]
fn test_filter_by_pattern_recomputes_summary() {
    let report = make_report(sample_defects());
    let filtered =
        DefectReportService::filter_by_pattern(&report, Some("src/a.rs".to_string()), None, 0);
    assert_eq!(filtered.summary.total_defects, 2); // CPLX-001 and SATD-001
    assert_eq!(filtered.file_index.len(), 1);
}

#[test]
fn test_filter_by_pattern_no_matches() {
    let report = make_report(sample_defects());
    let filtered = DefectReportService::filter_by_pattern(
        &report,
        Some("nonexistent/**".to_string()),
        None,
        0,
    );
    assert_eq!(filtered.defects.len(), 0);
    assert_eq!(filtered.summary.total_defects, 0);
}

// ============ ReportFormat tests ============

#[test]
fn test_report_format_all_filename_extensions() {
    let svc = DefectReportService::new();
    let json_name = svc.generate_filename(ReportFormat::Json);
    let csv_name = svc.generate_filename(ReportFormat::Csv);
    let md_name = svc.generate_filename(ReportFormat::Markdown);
    let txt_name = svc.generate_filename(ReportFormat::Text);
    assert!(json_name.ends_with(".json"));
    assert!(csv_name.ends_with(".csv"));
    assert!(md_name.ends_with(".md"));
    assert!(txt_name.ends_with(".txt"));
}
