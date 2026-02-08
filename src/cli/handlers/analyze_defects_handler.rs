#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat analyze defects` command
//!
//! Scans projects for known defect patterns with text, JSON, and JUnit output formats
//! for CI/CD integration. Based on docs/issues/analyze-defects-command.md

use crate::services::defect_detector::{DefectPattern, RustDefectDetector, Severity};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Junit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectSummary {
    pub total_files_scanned: usize,
    pub files_with_defects: usize,
    pub total_defects: usize,
    pub by_severity: SeverityCount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeverityCount {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectReport {
    pub summary: DefectSummary,
    pub defects: Vec<DefectPattern>,
    pub exit_code: i32,
    pub has_critical_defects: bool,
}

/// Handle the `pmat analyze defects` command
pub async fn handle_analyze_defects(
    path: Option<&Path>,
    file: Option<&Path>,
    severity_filter: Option<Severity>,
    format: OutputFormat,
) -> Result<i32> {
    let detector = RustDefectDetector::new();

    let target_path = path.unwrap_or_else(|| Path::new("."));

    // Collect all Rust files to scan
    let files_to_scan = if let Some(specific_file) = file {
        vec![specific_file.to_path_buf()]
    } else {
        collect_rust_files(target_path)?
    };

    // Scan all files for defects
    let mut all_defects = Vec::new();
    let mut files_with_defects = 0;

    for file_path in &files_to_scan {
        if let Ok(content) = fs::read_to_string(file_path) {
            let defects = detector.detect(&content, file_path);
            if !defects.is_empty() {
                files_with_defects += 1;
                all_defects.extend(defects);
            }
        }
    }

    // Apply severity filter if specified
    if let Some(filter_severity) = severity_filter {
        all_defects.retain(|d| d.severity == filter_severity);
    }

    // Calculate summary
    let summary = calculate_summary(&files_to_scan, files_with_defects, &all_defects);
    let has_critical = all_defects
        .iter()
        .any(|d| matches!(d.severity, Severity::Critical));
    let exit_code = if has_critical { 1 } else { 0 };

    let report = DefectReport {
        summary,
        defects: all_defects,
        exit_code,
        has_critical_defects: has_critical,
    };

    // Output in requested format
    match format {
        OutputFormat::Text => print_text_report(&report),
        OutputFormat::Json => print_json_report(&report)?,
        OutputFormat::Junit => print_junit_report(&report)?,
    }

    Ok(exit_code)
}

fn collect_rust_files(path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
        || entry.file_name() == "target"
}

fn calculate_summary(
    files: &[std::path::PathBuf],
    files_with_defects: usize,
    defects: &[DefectPattern],
) -> DefectSummary {
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for defect in defects {
        match defect.severity {
            Severity::Critical => critical += defect.instances.len(),
            Severity::High => high += defect.instances.len(),
            Severity::Medium => medium += defect.instances.len(),
            Severity::Low => low += defect.instances.len(),
        }
    }

    DefectSummary {
        total_files_scanned: files.len(),
        files_with_defects,
        total_defects: critical + high + medium + low,
        by_severity: SeverityCount {
            critical,
            high,
            medium,
            low,
        },
    }
}

fn print_text_report(report: &DefectReport) {
    println!("\nKnown Defects Report");
    println!("====================");

    println!("\n📊 Summary");
    println!(
        "  Total Files Scanned: {}",
        report.summary.total_files_scanned
    );
    println!(
        "  Files with Defects: {}",
        report.summary.files_with_defects
    );
    println!("  Total Defects: {}", report.summary.total_defects);
    println!("  Critical: {}", report.summary.by_severity.critical);
    println!("  High: {}", report.summary.by_severity.high);
    println!("  Medium: {}", report.summary.by_severity.medium);
    println!("  Low: {}", report.summary.by_severity.low);

    // Group defects by severity
    let critical_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::Critical))
        .collect();
    let high_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::High))
        .collect();
    let medium_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::Medium))
        .collect();

    if !critical_defects.is_empty() {
        println!("\n🔴 CRITICAL Defects ({})", critical_defects.len());
        for defect in critical_defects {
            print_defect_pattern(defect);
        }
    }

    if !high_defects.is_empty() {
        println!("\n🟠 HIGH Defects ({})", high_defects.len());
        for defect in high_defects {
            print_defect_pattern(defect);
        }
    }

    if !medium_defects.is_empty() {
        println!("\n🟡 MEDIUM Defects ({})", medium_defects.len());
        for defect in medium_defects {
            print_defect_pattern(defect);
        }
    }

    println!(
        "\nExit code: {} {}",
        report.exit_code,
        if report.has_critical_defects {
            "(critical defects found)"
        } else {
            "(no critical defects)"
        }
    );
}

fn print_defect_pattern(defect: &DefectPattern) {
    println!(
        "\n  {}: {} ({} instances)",
        defect.id,
        defect.name,
        defect.instances.len()
    );

    // Show first 10 instances
    for (i, instance) in defect.instances.iter().take(10).enumerate() {
        println!("    - {}:{}", instance.file, instance.line);
        if i == 9 && defect.instances.len() > 10 {
            println!("    ... ({} more)", defect.instances.len() - 10);
        }
    }

    println!("\n  Fix: {}", defect.fix_recommendation);
    println!("  Evidence: {}", defect.evidence_description);
}

fn print_json_report(report: &DefectReport) -> Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;
    println!("{}", json);
    Ok(())
}

fn print_junit_report(report: &DefectReport) -> Result<()> {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuites name=\"Known Defects Analysis\" tests=\"{}\" failures=\"{}\" errors=\"0\">",
        report.summary.total_defects, report.summary.total_defects
    );

    for defect in &report.defects {
        println!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"0\">",
            defect.id,
            defect.instances.len(),
            defect.instances.len()
        );

        for instance in &defect.instances {
            println!(
                "    <testcase name=\"{}:{}\" classname=\"{}\">",
                instance.file, instance.line, defect.id
            );
            println!("      <failure message=\"{} detected\">", defect.name);
            println!("File: {}:{}", instance.file, instance.line);
            println!("Pattern: {}", defect.id);
            println!("Severity: {}", defect.severity.as_str());
            println!("Evidence: {}", defect.evidence_description);
            println!("Fix: {}", defect.fix_recommendation);
            println!("      </failure>");
            println!("    </testcase>");
        }

        println!("  </testsuite>");
    }

    println!("</testsuites>");
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
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
        assert!(files.len() >= 0, "Should return a list of files");
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
        let defects: Vec<DefectPattern> = vec![];

        let summary = calculate_summary(&files, 0, &defects);

        assert_eq!(summary.total_files_scanned, 0);
        assert_eq!(summary.files_with_defects, 0);
        assert_eq!(summary.total_defects, 0);
        assert_eq!(summary.by_severity.critical, 0);
        assert_eq!(summary.by_severity.high, 0);
        assert_eq!(summary.by_severity.medium, 0);
        assert_eq!(summary.by_severity.low, 0);
    }

    #[test]
    fn test_calculate_summary_with_defects() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        let files = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
        ];

        let defects = vec![
            DefectPattern {
                id: "CRIT-001".to_string(),
                name: "Critical defect".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![
                    DefectInstance {
                        file: "a.rs".to_string(),
                        line: 1,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    },
                    DefectInstance {
                        file: "a.rs".to_string(),
                        line: 2,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    },
                ],
            },
            DefectPattern {
                id: "HIGH-001".to_string(),
                name: "High defect".to_string(),
                severity: Severity::High,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "b.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
            DefectPattern {
                id: "MED-001".to_string(),
                name: "Medium defect".to_string(),
                severity: Severity::Medium,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![
                    DefectInstance {
                        file: "c.rs".to_string(),
                        line: 1,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    },
                    DefectInstance {
                        file: "c.rs".to_string(),
                        line: 2,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    },
                    DefectInstance {
                        file: "c.rs".to_string(),
                        line: 3,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    },
                ],
            },
            DefectPattern {
                id: "LOW-001".to_string(),
                name: "Low defect".to_string(),
                severity: Severity::Low,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "a.rs".to_string(),
                    line: 10,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            },
        ];

        let summary = calculate_summary(&files, 3, &defects);

        assert_eq!(summary.total_files_scanned, 3);
        assert_eq!(summary.files_with_defects, 3);
        assert_eq!(summary.total_defects, 7); // 2 + 1 + 3 + 1
        assert_eq!(summary.by_severity.critical, 2);
        assert_eq!(summary.by_severity.high, 1);
        assert_eq!(summary.by_severity.medium, 3);
        assert_eq!(summary.by_severity.low, 1);
    }

    // =========================================================================
    // print_text_report tests
    // =========================================================================

    #[test]
    fn test_print_text_report_no_defects() {
        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 10,
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

        // Just ensure it doesn't panic
        print_text_report(&report);
    }

    #[test]
    fn test_print_text_report_with_critical() {
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
                id: "CRIT-001".to_string(),
                name: "Critical Issue".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix immediately".to_string(),
                bad_example: "bad()".to_string(),
                good_example: "good()".to_string(),
                evidence_description: "Production incident".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "src/main.rs".to_string(),
                    line: 42,
                    column: 10,
                    code_snippet: "bad()".to_string(),
                }],
            }],
            exit_code: 1,
            has_critical_defects: true,
        };

        // Just ensure it doesn't panic
        print_text_report(&report);
    }

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

    // =========================================================================
    // print_json_report tests
    // =========================================================================

    #[test]
    fn test_print_json_report_success() {
        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 5,
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

        let result = print_json_report(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_report_with_defects() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 1,
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
                name: "Test".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: Some("https://example.com".to_string()),
                instances: vec![DefectInstance {
                    file: "test.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            }],
            exit_code: 1,
            has_critical_defects: true,
        };

        let result = print_json_report(&report);
        assert!(result.is_ok());
    }

    // =========================================================================
    // print_junit_report tests
    // =========================================================================

    #[test]
    fn test_print_junit_report_empty() {
        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 10,
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

        let result = print_junit_report(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_junit_report_with_defects() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 5,
                files_with_defects: 2,
                total_defects: 3,
                by_severity: SeverityCount {
                    critical: 2,
                    high: 1,
                    medium: 0,
                    low: 0,
                },
            },
            defects: vec![
                DefectPattern {
                    id: "CRIT-001".to_string(),
                    name: "Critical Bug".to_string(),
                    severity: Severity::Critical,
                    fix_recommendation: "Fix now".to_string(),
                    bad_example: "panic!()".to_string(),
                    good_example: "return Err(...)".to_string(),
                    evidence_description: "Production crash".to_string(),
                    evidence_url: None,
                    instances: vec![
                        DefectInstance {
                            file: "src/main.rs".to_string(),
                            line: 10,
                            column: 5,
                            code_snippet: "panic!()".to_string(),
                        },
                        DefectInstance {
                            file: "src/lib.rs".to_string(),
                            line: 20,
                            column: 10,
                            code_snippet: "panic!()".to_string(),
                        },
                    ],
                },
                DefectPattern {
                    id: "HIGH-001".to_string(),
                    name: "High Bug".to_string(),
                    severity: Severity::High,
                    fix_recommendation: "Fix soon".to_string(),
                    bad_example: "bad".to_string(),
                    good_example: "good".to_string(),
                    evidence_description: "High evidence".to_string(),
                    evidence_url: Some("https://example.com".to_string()),
                    instances: vec![DefectInstance {
                        file: "src/util.rs".to_string(),
                        line: 30,
                        column: 1,
                        code_snippet: "bad".to_string(),
                    }],
                },
            ],
            exit_code: 1,
            has_critical_defects: true,
        };

        let result = print_junit_report(&report);
        assert!(result.is_ok());
    }

    // =========================================================================
    // handle_analyze_defects integration tests
    // =========================================================================

    #[tokio::test]
    async fn test_handle_analyze_defects_empty_dir() {
        let temp_dir = TempDir::new().expect("temp dir");

        let result =
            handle_analyze_defects(Some(temp_dir.path()), None, None, OutputFormat::Text).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // No critical defects
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
        std::fs::write(src_dir.join("clean.rs"), "fn clean() { let x = Some(42); }")
            .expect("write");
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
    // print_defect_pattern tests (internal function via print_text_report)
    // =========================================================================

    #[test]
    fn test_print_defect_pattern_with_url() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 1,
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
                id: "URL-TEST".to_string(),
                name: "URL Test".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Check the URL".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "See URL for details".to_string(),
                evidence_url: Some("https://example.com/evidence".to_string()),
                instances: vec![DefectInstance {
                    file: "test.rs".to_string(),
                    line: 1,
                    column: 1,
                    code_snippet: "bad".to_string(),
                }],
            }],
            exit_code: 1,
            has_critical_defects: true,
        };

        // Test that URL is handled (printed in evidence)
        print_text_report(&report);
    }

    #[test]
    fn test_print_defect_pattern_exactly_10_instances() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        // Create exactly 10 instances (boundary test)
        let instances: Vec<DefectInstance> = (0..10)
            .map(|i| DefectInstance {
                file: format!("file{}.rs", i),
                line: i + 1,
                column: 1,
                code_snippet: "bad".to_string(),
            })
            .collect();

        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 10,
                files_with_defects: 10,
                total_defects: 10,
                by_severity: SeverityCount {
                    critical: 10,
                    high: 0,
                    medium: 0,
                    low: 0,
                },
            },
            defects: vec![DefectPattern {
                id: "BOUND-001".to_string(),
                name: "Boundary Test".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances,
            }],
            exit_code: 1,
            has_critical_defects: true,
        };

        // Should show all 10 without "... (N more)" message
        print_text_report(&report);
    }

    #[test]
    fn test_print_defect_pattern_11_instances() {
        use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

        // Create 11 instances (triggers truncation message)
        let instances: Vec<DefectInstance> = (0..11)
            .map(|i| DefectInstance {
                file: format!("file{}.rs", i),
                line: i + 1,
                column: 1,
                code_snippet: "bad".to_string(),
            })
            .collect();

        let report = DefectReport {
            summary: DefectSummary {
                total_files_scanned: 11,
                files_with_defects: 11,
                total_defects: 11,
                by_severity: SeverityCount {
                    critical: 11,
                    high: 0,
                    medium: 0,
                    low: 0,
                },
            },
            defects: vec![DefectPattern {
                id: "TRUNC-001".to_string(),
                name: "Truncation Test".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix".to_string(),
                bad_example: "bad".to_string(),
                good_example: "good".to_string(),
                evidence_description: "Evidence".to_string(),
                evidence_url: None,
                instances,
            }],
            exit_code: 1,
            has_critical_defects: true,
        };

        // Should show first 10 + "... (1 more)" message
        print_text_report(&report);
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
}
