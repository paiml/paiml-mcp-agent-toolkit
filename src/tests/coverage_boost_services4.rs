#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services - batch 4
//! Tests for SATD detector

use crate::services::satd_detector::{DebtCategory, DebtClassifier, SATDDetector, Severity};
use std::path::Path;

#[test]
fn test_debt_category_variants() {
    let categories = vec![
        DebtCategory::Design,
        DebtCategory::Test,
        DebtCategory::Defect,
        DebtCategory::Requirement,
        DebtCategory::Performance,
        DebtCategory::Security,
    ];
    for c in categories {
        let _ = format!("{:?}", c);
    }
}

#[test]
fn test_satd_severity_variants() {
    let severities = vec![
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ];
    for s in severities {
        let _ = format!("{:?}", s);
    }
}

#[test]
fn test_debt_classifier_new() {
    let _classifier = DebtClassifier::new();
}

#[test]
fn test_debt_classifier_classify_todo() {
    let classifier = DebtClassifier::new();
    let result = classifier.classify_comment("TODO: implement feature");
    assert!(result.is_some());
}

#[test]
fn test_debt_classifier_classify_fixme() {
    let classifier = DebtClassifier::new();
    let result = classifier.classify_comment("FIXME: this is broken");
    assert!(result.is_some());
}

#[test]
fn test_debt_classifier_classify_normal() {
    let classifier = DebtClassifier::new();
    let result = classifier.classify_comment("This is a normal comment");
    assert!(result.is_none());
}

#[test]
fn test_satd_detector_new() {
    let _detector = SATDDetector::new();
}

#[test]
fn test_satd_detector_extract_empty() {
    let detector = SATDDetector::new();
    let result = detector.extract_from_content("", Path::new("test.rs"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_satd_detector_extract_no_debt() {
    let detector = SATDDetector::new();
    let content = "fn main() {\n    println!(\"Hello\");\n}";
    let result = detector.extract_from_content(content, Path::new("test.rs"));
    assert!(result.is_ok());
}

#[test]
fn test_satd_detector_extract_with_todo() {
    let detector = SATDDetector::new();
    let content = "// TODO: implement this\nfn main() {}";
    let result = detector.extract_from_content(content, Path::new("test.rs"));
    assert!(result.is_ok());
}

#[test]
fn test_debt_category_equality() {
    assert_eq!(DebtCategory::Design, DebtCategory::Design);
    assert_ne!(DebtCategory::Design, DebtCategory::Test);
}

#[test]
fn test_satd_severity_equality() {
    assert_eq!(Severity::High, Severity::High);
    assert_ne!(Severity::High, Severity::Low);
}

#[test]
fn test_severity_escalate() {
    assert_eq!(Severity::Low.escalate(), Severity::Medium);
    assert_eq!(Severity::Medium.escalate(), Severity::High);
    assert_eq!(Severity::High.escalate(), Severity::Critical);
    assert_eq!(Severity::Critical.escalate(), Severity::Critical);
}

#[test]
fn test_severity_reduce() {
    assert_eq!(Severity::Critical.reduce(), Severity::High);
    assert_eq!(Severity::High.reduce(), Severity::Medium);
    assert_eq!(Severity::Medium.reduce(), Severity::Low);
    assert_eq!(Severity::Low.reduce(), Severity::Low);
}

#[test]
fn test_debt_classifier_strict() {
    let classifier = DebtClassifier::new_strict();
    // Strict mode should only detect explicit markers with colon format
    let result = classifier.classify_comment("technical debt");
    assert!(result.is_none()); // "technical debt" without TODO/FIXME prefix
}

#[test]
fn test_satd_detector_default() {
    let detector = SATDDetector::default();
    let result = detector.extract_from_content("fn test() {}", Path::new("test.rs"));
    assert!(result.is_ok());
}

#[test]
fn test_satd_detector_new_strict() {
    let detector = SATDDetector::new_strict();
    let result = detector.extract_from_content("fn test() {}", Path::new("test.rs"));
    assert!(result.is_ok());
}
