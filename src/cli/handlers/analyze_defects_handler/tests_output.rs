#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for output formatting (text, JSON, JUnit) and calculate_summary with defects

use super::handler::calculate_summary;
use super::output::{print_json_report, print_junit_report, print_text_report};
use super::types::*;
use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};
use std::path::PathBuf;

// =========================================================================
// calculate_summary with defects
// =========================================================================

#[test]
fn test_calculate_summary_with_defects() {
    let files = [
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

    let summary = calculate_summary(files.len(), &defects);

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
// print_defect_pattern boundary tests (via print_text_report)
// =========================================================================

#[test]
fn test_print_defect_pattern_with_url() {
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
// Round-5 dogfood regression: the summary must describe the list it heads
// =========================================================================

fn one_defect(severity: Severity, file: &str) -> DefectPattern {
    DefectPattern {
        id: format!("X-{file}"),
        name: "defect".to_string(),
        severity,
        fix_recommendation: "Fix".to_string(),
        bad_example: "bad".to_string(),
        good_example: "good".to_string(),
        evidence_description: "Evidence".to_string(),
        evidence_url: None,
        instances: vec![DefectInstance {
            file: file.to_string(),
            line: 1,
            column: 1,
            code_snippet: "bad".to_string(),
        }],
    }
}

/// `analyze defects --severity low --format json` on this repo printed
/// `files_with_defects: 22` next to `total_defects: 0` and an empty `defects`
/// array: the count was tallied while scanning, BEFORE the severity filter ran,
/// and nothing recomputed it afterwards. `calculate_summary` now derives the
/// count from the defects it is given, so a summary cannot outlive its list.
#[test]
fn files_with_defects_is_zero_once_the_severity_filter_empties_the_list() {
    let files = [PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let scanned = vec![
        one_defect(Severity::Critical, "a.rs"),
        one_defect(Severity::High, "b.rs"),
    ];

    // What `--severity low` does to the scan results.
    let reported: Vec<DefectPattern> = scanned
        .into_iter()
        .filter(|d| d.severity == Severity::Low)
        .collect();

    let summary = calculate_summary(files.len(), &reported);

    assert_eq!(summary.total_defects, 0);
    assert_eq!(
        summary.files_with_defects, 0,
        "no defect is reported, so no file can have one"
    );
    assert_eq!(summary.total_files_scanned, 2, "the scan still happened");
}

/// The same derivation must not double-count a file that several patterns hit.
#[test]
fn files_with_defects_counts_distinct_files_not_patterns() {
    let files = [PathBuf::from("a.rs")];
    let reported = vec![
        one_defect(Severity::Critical, "a.rs"),
        one_defect(Severity::Low, "a.rs"),
    ];

    let summary = calculate_summary(files.len(), &reported);

    assert_eq!(summary.total_defects, 2);
    assert_eq!(summary.files_with_defects, 1);
}
