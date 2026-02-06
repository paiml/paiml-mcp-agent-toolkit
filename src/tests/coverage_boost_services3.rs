#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services - batch 3
//! Tests for defect_report models and enhanced_reporting

use crate::models::defect_report::{DefectCategory, Severity};
use crate::services::enhanced_reporting::{ReportFormat, RiskLevel, SectionType};

#[test]
fn test_defect_category_variants() {
    let categories = vec![
        DefectCategory::Complexity,
        DefectCategory::TechnicalDebt,
        DefectCategory::DeadCode,
        DefectCategory::Duplication,
        DefectCategory::Performance,
        DefectCategory::Architecture,
        DefectCategory::TestCoverage,
    ];
    for c in categories {
        let _ = format!("{:?}", c);
    }
}

#[test]
fn test_severity_variants() {
    let severities = vec![
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ];
    for s in severities {
        let _ = format!("{:?}", s);
        let _ = format!("{}", s);
    }
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Critical, Severity::Critical);
    assert_ne!(Severity::Critical, Severity::Low);
}

#[test]
fn test_defect_category_equality() {
    assert_eq!(DefectCategory::Complexity, DefectCategory::Complexity);
    assert_ne!(DefectCategory::Complexity, DefectCategory::DeadCode);
}

#[test]
fn test_report_format_variants() {
    let formats = vec![
        ReportFormat::Html,
        ReportFormat::Markdown,
        ReportFormat::Json,
        ReportFormat::Pdf,
        ReportFormat::Dashboard,
    ];
    for f in formats {
        let _ = format!("{:?}", f);
    }
}

#[test]
fn test_report_format_equality() {
    assert_eq!(ReportFormat::Json, ReportFormat::Json);
    assert_ne!(ReportFormat::Json, ReportFormat::Html);
}

#[test]
fn test_risk_level_variants() {
    let levels = vec![
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];
    for l in levels {
        let _ = format!("{:?}", l);
    }
}

#[test]
fn test_section_type_variants() {
    let types = vec![
        SectionType::Complexity,
        SectionType::DeadCode,
        SectionType::Duplication,
        SectionType::TechnicalDebt,
        SectionType::Security,
        SectionType::Performance,
        SectionType::BigOAnalysis,
        SectionType::Dependencies,
        SectionType::TestCoverage,
        SectionType::CodeSmells,
    ];
    for t in types {
        let _ = format!("{:?}", t);
    }
}

#[test]
fn test_severity_clone() {
    let s = Severity::High;
    let cloned = s.clone();
    assert_eq!(s, cloned);
}

#[test]
fn test_defect_category_clone() {
    let c = DefectCategory::Complexity;
    let cloned = c.clone();
    assert_eq!(c, cloned);
}

#[test]
fn test_report_format_clone() {
    let f = ReportFormat::Markdown;
    let cloned = f.clone();
    assert_eq!(f, cloned);
}

#[test]
fn test_defect_category_all() {
    let all = DefectCategory::all();
    assert!(!all.is_empty());
    assert!(all.contains(&DefectCategory::Complexity));
}
