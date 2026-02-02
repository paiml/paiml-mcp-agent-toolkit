//! Coverage boost tests for models/defect_report.rs
//! Tests: Severity, DefectCategory, Defect, DefectSummary, FileHotspot, ReportMetadata

use crate::models::defect_report::{
    Defect, DefectCategory, DefectSummary, FileHotspot, ReportMetadata, Severity,
};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

// ============ Severity Tests ============

#[test]
fn test_severity_serde() {
    let severities = vec![
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];
    for sev in &severities {
        let json = serde_json::to_string(sev).unwrap();
        let back: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(*sev, back);
    }
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Low);
}

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Low), "Low");
    assert_eq!(format!("{}", Severity::Medium), "Medium");
    assert_eq!(format!("{}", Severity::High), "High");
    assert_eq!(format!("{}", Severity::Critical), "Critical");
}

#[test]
fn test_severity_rename_all_lowercase() {
    let json = serde_json::to_string(&Severity::Critical).unwrap();
    assert!(json.contains("critical"));
}

// ============ DefectCategory Tests ============

#[test]
fn test_defect_category_all() {
    let categories = DefectCategory::all();
    assert_eq!(categories.len(), 7);
    assert!(categories.contains(&DefectCategory::Complexity));
    assert!(categories.contains(&DefectCategory::TechnicalDebt));
    assert!(categories.contains(&DefectCategory::DeadCode));
    assert!(categories.contains(&DefectCategory::Duplication));
    assert!(categories.contains(&DefectCategory::Performance));
    assert!(categories.contains(&DefectCategory::Architecture));
    assert!(categories.contains(&DefectCategory::TestCoverage));
}

#[test]
fn test_defect_category_serde() {
    for cat in DefectCategory::all() {
        let json = serde_json::to_string(&cat).unwrap();
        let back: DefectCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, back);
    }
}

#[test]
fn test_defect_category_display() {
    assert_eq!(format!("{}", DefectCategory::Complexity), "Complexity");
    assert_eq!(format!("{}", DefectCategory::TechnicalDebt), "Technical Debt");
    assert_eq!(format!("{}", DefectCategory::DeadCode), "Dead Code");
    assert_eq!(format!("{}", DefectCategory::Duplication), "Duplication");
    assert_eq!(format!("{}", DefectCategory::Performance), "Performance");
    assert_eq!(format!("{}", DefectCategory::Architecture), "Architecture");
    assert_eq!(format!("{}", DefectCategory::TestCoverage), "Test Coverage");
}

#[test]
fn test_defect_category_ordering() {
    assert!(DefectCategory::Complexity < DefectCategory::TechnicalDebt);
}

#[test]
fn test_defect_category_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(DefectCategory::Complexity);
    set.insert(DefectCategory::DeadCode);
    set.insert(DefectCategory::Complexity); // duplicate
    assert_eq!(set.len(), 2);
}

// ============ Defect Tests ============

#[test]
fn test_defect_generate_id() {
    assert_eq!(Defect::generate_id("TEST", 0), "TEST-001");
    assert_eq!(Defect::generate_id("BUG", 99), "BUG-100");
    assert_eq!(Defect::generate_id("ERR", 999), "ERR-1000");
}

#[test]
fn test_defect_severity_weight() {
    let make_defect = |severity: Severity| Defect {
        id: "T-001".to_string(),
        severity,
        category: DefectCategory::Complexity,
        file_path: PathBuf::from("test.rs"),
        line_start: 1,
        line_end: None,
        column_start: None,
        column_end: None,
        message: "test".to_string(),
        rule_id: "test".to_string(),
        fix_suggestion: None,
        metrics: HashMap::new(),
    };

    assert_eq!(make_defect(Severity::Critical).severity_weight(), 10.0);
    assert_eq!(make_defect(Severity::High).severity_weight(), 5.0);
    assert_eq!(make_defect(Severity::Medium).severity_weight(), 3.0);
    assert_eq!(make_defect(Severity::Low).severity_weight(), 1.0);
}

#[test]
fn test_defect_serde() {
    let defect = Defect {
        id: "CPLX-001".to_string(),
        severity: Severity::High,
        category: DefectCategory::Complexity,
        file_path: PathBuf::from("src/main.rs"),
        line_start: 42,
        line_end: Some(60),
        column_start: Some(1),
        column_end: Some(80),
        message: "High complexity".to_string(),
        rule_id: "cyclomatic".to_string(),
        fix_suggestion: Some("Extract method".to_string()),
        metrics: HashMap::from([("complexity".to_string(), 25.0)]),
    };
    let json = serde_json::to_string(&defect).unwrap();
    let back: Defect = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "CPLX-001");
    assert_eq!(back.severity, Severity::High);
    assert_eq!(back.line_start, 42);
}

#[test]
fn test_defect_clone_debug() {
    let defect = Defect {
        id: "T-001".to_string(),
        severity: Severity::Low,
        category: DefectCategory::DeadCode,
        file_path: PathBuf::from("test.rs"),
        line_start: 1,
        line_end: None,
        column_start: None,
        column_end: None,
        message: "Unused function".to_string(),
        rule_id: "dead-code".to_string(),
        fix_suggestion: None,
        metrics: HashMap::new(),
    };
    let cloned = defect.clone();
    assert_eq!(cloned.id, defect.id);
    let _ = format!("{:?}", defect);
}

// ============ ReportMetadata Tests ============

#[test]
fn test_report_metadata_serde() {
    let meta = ReportMetadata {
        tool: "pmat".to_string(),
        version: "1.0.0".to_string(),
        generated_at: chrono::Utc::now(),
        project_root: PathBuf::from("/project"),
        total_files_analyzed: 100,
        analysis_duration_ms: 5000,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: ReportMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool, "pmat");
    assert_eq!(back.total_files_analyzed, 100);
}

// ============ DefectSummary Tests ============

#[test]
fn test_defect_summary_serde() {
    let summary = DefectSummary {
        total_defects: 42,
        by_severity: BTreeMap::from([
            ("critical".to_string(), 2),
            ("high".to_string(), 10),
        ]),
        by_category: BTreeMap::from([
            ("complexity".to_string(), 15),
            ("dead_code".to_string(), 5),
        ]),
        hotspot_files: vec![],
    };
    let json = serde_json::to_string(&summary).unwrap();
    let back: DefectSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_defects, 42);
    assert_eq!(back.by_severity.len(), 2);
}

// ============ FileHotspot Tests ============

#[test]
fn test_file_hotspot_serde() {
    let hotspot = FileHotspot {
        path: PathBuf::from("src/complex.rs"),
        defect_count: 15,
        severity_score: 75.5,
    };
    let json = serde_json::to_string(&hotspot).unwrap();
    let back: FileHotspot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.defect_count, 15);
    assert!((back.severity_score - 75.5).abs() < f64::EPSILON);
}
