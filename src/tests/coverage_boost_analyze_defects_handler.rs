#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for cli/handlers/analyze_defects_handler.rs
//! Tests: OutputFormat, DefectSummary, SeverityCount, DefectReport structs
//! and report formatting functions

use crate::cli::handlers::analyze_defects_handler::{
    DefectReport, DefectSummary, OutputFormat, SeverityCount,
};
use crate::services::defect_detector::{DefectInstance, DefectPattern, Severity};

// ============================================================================
// OutputFormat Tests
// ============================================================================

#[test]
fn test_output_format_text_debug() {
    let format = OutputFormat::Text;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("Text"));
}

#[test]
fn test_output_format_json_debug() {
    let format = OutputFormat::Json;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("Json"));
}

#[test]
fn test_output_format_junit_debug() {
    let format = OutputFormat::Junit;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("Junit"));
}

#[test]
fn test_output_format_clone_text() {
    let original = OutputFormat::Text;
    let cloned = original;
    assert!(matches!(cloned, OutputFormat::Text));
}

#[test]
fn test_output_format_clone_json() {
    let original = OutputFormat::Json;
    let cloned = original;
    assert!(matches!(cloned, OutputFormat::Json));
}

#[test]
fn test_output_format_clone_junit() {
    let original = OutputFormat::Junit;
    let cloned = original;
    assert!(matches!(cloned, OutputFormat::Junit));
}

#[test]
fn test_output_format_copy_preserves_original() {
    let original = OutputFormat::Text;
    let copied: OutputFormat = original;
    // Both should be usable after copy (Copy trait)
    assert!(matches!(original, OutputFormat::Text));
    assert!(matches!(copied, OutputFormat::Text));
}

#[test]
fn test_output_format_all_variants_distinct() {
    let text = OutputFormat::Text;
    let json = OutputFormat::Json;
    let junit = OutputFormat::Junit;

    // Each variant has unique debug representation
    let text_debug = format!("{:?}", text);
    let json_debug = format!("{:?}", json);
    let junit_debug = format!("{:?}", junit);

    assert_ne!(text_debug, json_debug);
    assert_ne!(json_debug, junit_debug);
    assert_ne!(text_debug, junit_debug);
}

// ============================================================================
// SeverityCount Tests
// ============================================================================

#[test]
fn test_severity_count_zero_values() {
    let count = SeverityCount {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
    };
    assert_eq!(count.critical, 0);
    assert_eq!(count.high, 0);
    assert_eq!(count.medium, 0);
    assert_eq!(count.low, 0);
}

#[test]
fn test_severity_count_max_values() {
    let count = SeverityCount {
        critical: usize::MAX,
        high: usize::MAX,
        medium: usize::MAX,
        low: usize::MAX,
    };
    assert_eq!(count.critical, usize::MAX);
    assert_eq!(count.high, usize::MAX);
    assert_eq!(count.medium, usize::MAX);
    assert_eq!(count.low, usize::MAX);
}

#[test]
fn test_severity_count_mixed_values() {
    let count = SeverityCount {
        critical: 5,
        high: 10,
        medium: 20,
        low: 100,
    };
    assert_eq!(count.critical, 5);
    assert_eq!(count.high, 10);
    assert_eq!(count.medium, 20);
    assert_eq!(count.low, 100);
}

#[test]
fn test_severity_count_serialization_roundtrip() {
    let count = SeverityCount {
        critical: 3,
        high: 7,
        medium: 15,
        low: 42,
    };
    let json = serde_json::to_string(&count).expect("serialize");
    let deserialized: SeverityCount = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.critical, 3);
    assert_eq!(deserialized.high, 7);
    assert_eq!(deserialized.medium, 15);
    assert_eq!(deserialized.low, 42);
}

#[test]
fn test_severity_count_json_field_names() {
    let count = SeverityCount {
        critical: 1,
        high: 2,
        medium: 3,
        low: 4,
    };
    let json = serde_json::to_string(&count).expect("serialize");
    assert!(json.contains("\"critical\":1"));
    assert!(json.contains("\"high\":2"));
    assert!(json.contains("\"medium\":3"));
    assert!(json.contains("\"low\":4"));
}

#[test]
fn test_severity_count_debug_format() {
    let count = SeverityCount {
        critical: 1,
        high: 2,
        medium: 3,
        low: 4,
    };
    let debug = format!("{:?}", count);
    assert!(debug.contains("SeverityCount"));
    assert!(debug.contains("1"));
    assert!(debug.contains("2"));
    assert!(debug.contains("3"));
    assert!(debug.contains("4"));
}

#[test]
fn test_severity_count_from_json() {
    let json = r#"{"critical":10,"high":20,"medium":30,"low":40}"#;
    let count: SeverityCount = serde_json::from_str(json).expect("deserialize");
    assert_eq!(count.critical, 10);
    assert_eq!(count.high, 20);
    assert_eq!(count.medium, 30);
    assert_eq!(count.low, 40);
}

// ============================================================================
// DefectSummary Tests
// ============================================================================

#[test]
fn test_defect_summary_empty() {
    let summary = DefectSummary {
        total_files_scanned: 0,
        files_with_defects: 0,
        total_defects: 0,
        by_severity: SeverityCount {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        },
    };
    assert_eq!(summary.total_files_scanned, 0);
    assert_eq!(summary.files_with_defects, 0);
    assert_eq!(summary.total_defects, 0);
}

#[test]
fn test_defect_summary_large_project() {
    let summary = DefectSummary {
        total_files_scanned: 10000,
        files_with_defects: 500,
        total_defects: 2500,
        by_severity: SeverityCount {
            critical: 100,
            high: 400,
            medium: 1000,
            low: 1000,
        },
    };
    assert_eq!(summary.total_files_scanned, 10000);
    assert_eq!(summary.files_with_defects, 500);
    assert_eq!(summary.total_defects, 2500);
    // Verify severity totals add up correctly
    let severity_total = summary.by_severity.critical
        + summary.by_severity.high
        + summary.by_severity.medium
        + summary.by_severity.low;
    assert_eq!(severity_total, summary.total_defects);
}

#[test]
fn test_defect_summary_serialization() {
    let summary = DefectSummary {
        total_files_scanned: 100,
        files_with_defects: 10,
        total_defects: 25,
        by_severity: SeverityCount {
            critical: 5,
            high: 5,
            medium: 10,
            low: 5,
        },
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    assert!(json.contains("\"total_files_scanned\":100"));
    assert!(json.contains("\"files_with_defects\":10"));
    assert!(json.contains("\"total_defects\":25"));
}

#[test]
fn test_defect_summary_deserialization() {
    let json = r#"{
        "total_files_scanned": 50,
        "files_with_defects": 5,
        "total_defects": 10,
        "by_severity": {
            "critical": 2,
            "high": 3,
            "medium": 3,
            "low": 2
        }
    }"#;
    let summary: DefectSummary = serde_json::from_str(json).expect("deserialize");
    assert_eq!(summary.total_files_scanned, 50);
    assert_eq!(summary.files_with_defects, 5);
    assert_eq!(summary.total_defects, 10);
    assert_eq!(summary.by_severity.critical, 2);
}

#[test]
fn test_defect_summary_debug_format() {
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
    assert!(debug.contains("files_with_defects"));
    assert!(debug.contains("total_defects"));
}

#[test]
fn test_defect_summary_only_critical() {
    let summary = DefectSummary {
        total_files_scanned: 10,
        files_with_defects: 5,
        total_defects: 5,
        by_severity: SeverityCount {
            critical: 5,
            high: 0,
            medium: 0,
            low: 0,
        },
    };
    assert_eq!(summary.by_severity.critical, 5);
    assert_eq!(summary.by_severity.high, 0);
    assert_eq!(summary.by_severity.medium, 0);
    assert_eq!(summary.by_severity.low, 0);
}

#[test]
fn test_defect_summary_only_low() {
    let summary = DefectSummary {
        total_files_scanned: 100,
        files_with_defects: 50,
        total_defects: 200,
        by_severity: SeverityCount {
            critical: 0,
            high: 0,
            medium: 0,
            low: 200,
        },
    };
    assert_eq!(summary.by_severity.low, 200);
    assert_eq!(summary.total_defects, 200);
}

// ============================================================================
// DefectReport Tests
// ============================================================================

#[test]
fn test_defect_report_no_critical() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 100,
            files_with_defects: 10,
            total_defects: 20,
            by_severity: SeverityCount {
                critical: 0,
                high: 10,
                medium: 5,
                low: 5,
            },
        },
        defects: vec![],
        exit_code: 0,
        has_critical_defects: false,
    };
    assert_eq!(report.exit_code, 0);
    assert!(!report.has_critical_defects);
}

#[test]
fn test_defect_report_with_critical() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 50,
            files_with_defects: 5,
            total_defects: 10,
            by_severity: SeverityCount {
                critical: 3,
                high: 3,
                medium: 2,
                low: 2,
            },
        },
        defects: vec![],
        exit_code: 1,
        has_critical_defects: true,
    };
    assert_eq!(report.exit_code, 1);
    assert!(report.has_critical_defects);
}

#[test]
fn test_defect_report_serialization() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 10,
            files_with_defects: 1,
            total_defects: 1,
            by_severity: SeverityCount {
                critical: 1,
                high: 0,
                medium: 0,
                low: 0,
            },
        },
        defects: vec![],
        exit_code: 1,
        has_critical_defects: true,
    };
    let json = serde_json::to_string(&report).expect("serialize");
    assert!(json.contains("\"exit_code\":1"));
    assert!(json.contains("\"has_critical_defects\":true"));
}

#[test]
fn test_defect_report_with_defect_patterns() {
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
            name: "Test Pattern".to_string(),
            severity: Severity::Critical,
            fix_recommendation: "Fix this".to_string(),
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
    assert_eq!(report.defects.len(), 1);
    assert_eq!(report.defects[0].id, "TEST-001");
}

#[test]
fn test_defect_report_multiple_defect_patterns() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 20,
            files_with_defects: 5,
            total_defects: 10,
            by_severity: SeverityCount {
                critical: 3,
                high: 3,
                medium: 2,
                low: 2,
            },
        },
        defects: vec![
            DefectPattern {
                id: "CRIT-001".to_string(),
                name: "Critical Issue".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Fix immediately".to_string(),
                bad_example: "panic!()".to_string(),
                good_example: "return Err(...)".to_string(),
                evidence_description: "Production outage".to_string(),
                evidence_url: None,
                instances: vec![
                    DefectInstance {
                        file: "src/a.rs".to_string(),
                        line: 1,
                        column: 1,
                        code_snippet: "panic!()".to_string(),
                    },
                    DefectInstance {
                        file: "src/b.rs".to_string(),
                        line: 2,
                        column: 1,
                        code_snippet: "panic!()".to_string(),
                    },
                ],
            },
            DefectPattern {
                id: "HIGH-001".to_string(),
                name: "High Issue".to_string(),
                severity: Severity::High,
                fix_recommendation: "Fix soon".to_string(),
                bad_example: "unwrap()".to_string(),
                good_example: "expect()".to_string(),
                evidence_description: "Performance degradation".to_string(),
                evidence_url: Some("https://example.com/high".to_string()),
                instances: vec![DefectInstance {
                    file: "src/c.rs".to_string(),
                    line: 5,
                    column: 10,
                    code_snippet: "foo.unwrap()".to_string(),
                }],
            },
        ],
        exit_code: 1,
        has_critical_defects: true,
    };
    assert_eq!(report.defects.len(), 2);
    assert_eq!(report.defects[0].instances.len(), 2);
    assert_eq!(report.defects[1].instances.len(), 1);
}

#[test]
fn test_defect_report_debug_format() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 1,
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
    assert!(debug.contains("has_critical_defects"));
}

#[test]
fn test_defect_report_empty_defects_vec() {
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 50,
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
    assert!(report.defects.is_empty());
}

#[test]
fn test_defect_report_json_roundtrip() {
    let original = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 100,
            files_with_defects: 10,
            total_defects: 20,
            by_severity: SeverityCount {
                critical: 5,
                high: 5,
                medium: 5,
                low: 5,
            },
        },
        defects: vec![DefectPattern {
            id: "ROUND-001".to_string(),
            name: "Roundtrip Test".to_string(),
            severity: Severity::Medium,
            fix_recommendation: "Test fix".to_string(),
            bad_example: "old()".to_string(),
            good_example: "new()".to_string(),
            evidence_description: "Testing".to_string(),
            evidence_url: None,
            instances: vec![DefectInstance {
                file: "round.rs".to_string(),
                line: 42,
                column: 13,
                code_snippet: "old()".to_string(),
            }],
        }],
        exit_code: 0,
        has_critical_defects: false,
    };

    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: DefectReport = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.summary.total_files_scanned, 100);
    assert_eq!(deserialized.defects.len(), 1);
    assert_eq!(deserialized.defects[0].id, "ROUND-001");
    assert_eq!(deserialized.exit_code, 0);
}

// ============================================================================
// DefectInstance Tests (via DefectPattern)
// ============================================================================

#[test]
fn test_defect_instance_fields() {
    let instance = DefectInstance {
        file: "/path/to/file.rs".to_string(),
        line: 100,
        column: 15,
        code_snippet: "let x = foo.unwrap();".to_string(),
    };
    assert_eq!(instance.file, "/path/to/file.rs");
    assert_eq!(instance.line, 100);
    assert_eq!(instance.column, 15);
    assert_eq!(instance.code_snippet, "let x = foo.unwrap();");
}

#[test]
fn test_defect_instance_serialization() {
    let instance = DefectInstance {
        file: "test.rs".to_string(),
        line: 1,
        column: 1,
        code_snippet: "code".to_string(),
    };
    let json = serde_json::to_string(&instance).expect("serialize");
    assert!(json.contains("\"file\":\"test.rs\""));
    assert!(json.contains("\"line\":1"));
    assert!(json.contains("\"column\":1"));
    assert!(json.contains("\"code_snippet\":\"code\""));
}

#[test]
fn test_defect_instance_clone() {
    let original = DefectInstance {
        file: "original.rs".to_string(),
        line: 50,
        column: 10,
        code_snippet: "snippet".to_string(),
    };
    let cloned = original.clone();
    assert_eq!(cloned.file, "original.rs");
    assert_eq!(cloned.line, 50);
}

// ============================================================================
// DefectPattern Tests
// ============================================================================

#[test]
fn test_defect_pattern_all_severities() {
    let severities = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ];

    for severity in severities {
        let pattern = DefectPattern {
            id: format!("{:?}-001", severity),
            name: format!("{:?} test", severity),
            severity,
            fix_recommendation: "Fix it".to_string(),
            bad_example: "bad".to_string(),
            good_example: "good".to_string(),
            evidence_description: "Evidence".to_string(),
            evidence_url: None,
            instances: vec![],
        };
        assert_eq!(pattern.severity, severity);
    }
}

#[test]
fn test_defect_pattern_with_url() {
    let pattern = DefectPattern {
        id: "URL-001".to_string(),
        name: "URL Test".to_string(),
        severity: Severity::High,
        fix_recommendation: "Check URL".to_string(),
        bad_example: "bad()".to_string(),
        good_example: "good()".to_string(),
        evidence_description: "See link".to_string(),
        evidence_url: Some("https://example.com/evidence".to_string()),
        instances: vec![],
    };
    assert!(pattern.evidence_url.is_some());
    assert_eq!(
        pattern.evidence_url.unwrap(),
        "https://example.com/evidence"
    );
}

#[test]
fn test_defect_pattern_without_url() {
    let pattern = DefectPattern {
        id: "NO-URL-001".to_string(),
        name: "No URL Test".to_string(),
        severity: Severity::Low,
        fix_recommendation: "Fix".to_string(),
        bad_example: "bad".to_string(),
        good_example: "good".to_string(),
        evidence_description: "Local evidence".to_string(),
        evidence_url: None,
        instances: vec![],
    };
    assert!(pattern.evidence_url.is_none());
}

#[test]
fn test_defect_pattern_many_instances() {
    let instances: Vec<DefectInstance> = (0..100)
        .map(|i| DefectInstance {
            file: format!("file{}.rs", i),
            line: i + 1,
            column: 1,
            code_snippet: format!("bad code {}", i),
        })
        .collect();

    let pattern = DefectPattern {
        id: "MANY-001".to_string(),
        name: "Many Instances".to_string(),
        severity: Severity::Critical,
        fix_recommendation: "Fix all".to_string(),
        bad_example: "bad".to_string(),
        good_example: "good".to_string(),
        evidence_description: "Mass defect".to_string(),
        evidence_url: None,
        instances,
    };
    assert_eq!(pattern.instances.len(), 100);
}

#[test]
fn test_defect_pattern_serialization() {
    let pattern = DefectPattern {
        id: "SER-001".to_string(),
        name: "Serialization Test".to_string(),
        severity: Severity::Medium,
        fix_recommendation: "Serialize properly".to_string(),
        bad_example: "old_serialize()".to_string(),
        good_example: "new_serialize()".to_string(),
        evidence_description: "Testing serialization".to_string(),
        evidence_url: Some("https://serde.rs".to_string()),
        instances: vec![DefectInstance {
            file: "ser.rs".to_string(),
            line: 25,
            column: 5,
            code_snippet: "old_serialize()".to_string(),
        }],
    };

    let json = serde_json::to_string(&pattern).expect("serialize");
    assert!(json.contains("\"id\":\"SER-001\""));
    assert!(json.contains("\"severity\":"));
    assert!(json.contains("\"instances\":["));
}

// ============================================================================
// Severity Tests (from defect_detector)
// ============================================================================

#[test]
fn test_severity_as_str_critical() {
    assert_eq!(Severity::Critical.as_str(), "CRITICAL");
}

#[test]
fn test_severity_as_str_high() {
    assert_eq!(Severity::High.as_str(), "HIGH");
}

#[test]
fn test_severity_as_str_medium() {
    assert_eq!(Severity::Medium.as_str(), "MEDIUM");
}

#[test]
fn test_severity_as_str_low() {
    assert_eq!(Severity::Low.as_str(), "LOW");
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Critical, Severity::Critical);
    assert_eq!(Severity::High, Severity::High);
    assert_eq!(Severity::Medium, Severity::Medium);
    assert_eq!(Severity::Low, Severity::Low);
}

#[test]
fn test_severity_inequality() {
    assert_ne!(Severity::Critical, Severity::High);
    assert_ne!(Severity::High, Severity::Medium);
    assert_ne!(Severity::Medium, Severity::Low);
    assert_ne!(Severity::Critical, Severity::Low);
}

#[test]
fn test_severity_clone() {
    let original = Severity::Critical;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn test_severity_serialization() {
    let json = serde_json::to_string(&Severity::Critical).expect("serialize");
    assert!(json.contains("Critical"));
}

#[test]
fn test_severity_deserialization() {
    let critical: Severity = serde_json::from_str("\"Critical\"").expect("deserialize");
    assert_eq!(critical, Severity::Critical);

    let high: Severity = serde_json::from_str("\"High\"").expect("deserialize");
    assert_eq!(high, Severity::High);

    let medium: Severity = serde_json::from_str("\"Medium\"").expect("deserialize");
    assert_eq!(medium, Severity::Medium);

    let low: Severity = serde_json::from_str("\"Low\"").expect("deserialize");
    assert_eq!(low, Severity::Low);
}

// ============================================================================
// Integration-style Tests
// ============================================================================

#[test]
fn test_complete_report_structure() {
    // Test a complete report with all components
    let report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 1000,
            files_with_defects: 50,
            total_defects: 150,
            by_severity: SeverityCount {
                critical: 10,
                high: 30,
                medium: 60,
                low: 50,
            },
        },
        defects: vec![
            DefectPattern {
                id: "RUST-UNWRAP-001".to_string(),
                name: ".unwrap() calls".to_string(),
                severity: Severity::Critical,
                fix_recommendation: "Use .expect() with descriptive messages".to_string(),
                bad_example: "result.unwrap()".to_string(),
                good_example: "result.expect(\"Expected value to be present\")".to_string(),
                evidence_description: "Cloudflare outage".to_string(),
                evidence_url: Some("https://blog.cloudflare.com".to_string()),
                instances: (1..=10)
                    .map(|i| DefectInstance {
                        file: format!("src/module{}.rs", i),
                        line: i * 10,
                        column: 5,
                        code_snippet: "result.unwrap()".to_string(),
                    })
                    .collect(),
            },
            DefectPattern {
                id: "RUST-PANIC-001".to_string(),
                name: "panic! macro usage".to_string(),
                severity: Severity::High,
                fix_recommendation: "Use Result or Option".to_string(),
                bad_example: "panic!(\"error\")".to_string(),
                good_example: "return Err(Error::new(...)".to_string(),
                evidence_description: "Unexpected termination".to_string(),
                evidence_url: None,
                instances: vec![DefectInstance {
                    file: "src/lib.rs".to_string(),
                    line: 42,
                    column: 1,
                    code_snippet: "panic!(\"This should not happen\")".to_string(),
                }],
            },
        ],
        exit_code: 1,
        has_critical_defects: true,
    };

    // Verify structure
    assert_eq!(report.summary.total_files_scanned, 1000);
    assert_eq!(report.summary.files_with_defects, 50);
    assert_eq!(report.summary.total_defects, 150);

    // Verify severity counts
    assert_eq!(report.summary.by_severity.critical, 10);
    assert_eq!(report.summary.by_severity.high, 30);
    assert_eq!(report.summary.by_severity.medium, 60);
    assert_eq!(report.summary.by_severity.low, 50);

    // Verify defects
    assert_eq!(report.defects.len(), 2);
    assert_eq!(report.defects[0].instances.len(), 10);
    assert_eq!(report.defects[1].instances.len(), 1);

    // Verify exit code
    assert_eq!(report.exit_code, 1);
    assert!(report.has_critical_defects);
}

#[test]
fn test_report_json_pretty_print() {
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
        defects: vec![],
        exit_code: 1,
        has_critical_defects: true,
    };

    let json = serde_json::to_string_pretty(&report).expect("pretty serialize");
    // Pretty print should have newlines
    assert!(json.contains('\n'));
    // Should be valid JSON
    let _: DefectReport = serde_json::from_str(&json).expect("deserialize pretty");
}

#[test]
fn test_severity_count_total_calculation() {
    let count = SeverityCount {
        critical: 5,
        high: 10,
        medium: 20,
        low: 15,
    };
    let total = count.critical + count.high + count.medium + count.low;
    assert_eq!(total, 50);
}

#[test]
fn test_defect_summary_consistency() {
    // Test that summary can have more files scanned than files with defects
    let summary = DefectSummary {
        total_files_scanned: 1000,
        files_with_defects: 100,
        total_defects: 250,
        by_severity: SeverityCount {
            critical: 0,
            high: 50,
            medium: 100,
            low: 100,
        },
    };
    assert!(summary.total_files_scanned >= summary.files_with_defects);
    assert!(summary.total_defects >= summary.files_with_defects);
}

#[test]
fn test_report_exit_code_semantics() {
    // Exit code 0 = no critical defects
    let clean_report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 10,
            files_with_defects: 5,
            total_defects: 10,
            by_severity: SeverityCount {
                critical: 0,
                high: 5,
                medium: 3,
                low: 2,
            },
        },
        defects: vec![],
        exit_code: 0,
        has_critical_defects: false,
    };
    assert_eq!(clean_report.exit_code, 0);
    assert!(!clean_report.has_critical_defects);

    // Exit code 1 = critical defects found
    let critical_report = DefectReport {
        summary: DefectSummary {
            total_files_scanned: 10,
            files_with_defects: 5,
            total_defects: 10,
            by_severity: SeverityCount {
                critical: 1,
                high: 4,
                medium: 3,
                low: 2,
            },
        },
        defects: vec![],
        exit_code: 1,
        has_critical_defects: true,
    };
    assert_eq!(critical_report.exit_code, 1);
    assert!(critical_report.has_critical_defects);
}
