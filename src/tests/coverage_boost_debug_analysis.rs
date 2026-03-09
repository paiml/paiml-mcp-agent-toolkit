#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for models/debug_analysis.rs
//! Tests: DebugAnalysis, WhyIteration, Evidence, EvidenceSource, Recommendation,
//! Priority, EvidenceSummary

use crate::models::debug_analysis::{
    DebugAnalysis, Evidence, EvidenceSource, EvidenceSummary, Priority, Recommendation,
    WhyIteration,
};
use std::path::PathBuf;

// ============ DebugAnalysis Tests ============

#[test]
fn test_debug_analysis_new() {
    let analysis = DebugAnalysis::new("Stack overflow".to_string());
    assert_eq!(analysis.issue, "Stack overflow");
    assert!(analysis.whys.is_empty());
    assert!(analysis.root_cause.is_none());
    assert!(analysis.recommendations.is_empty());
}

#[test]
fn test_debug_analysis_serde() {
    let analysis = DebugAnalysis::new("Test issue".to_string());
    let json = serde_json::to_string(&analysis).unwrap();
    let back: DebugAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.issue, "Test issue");
}

#[test]
fn test_debug_analysis_clone_debug() {
    let analysis = DebugAnalysis::new("Clone test".to_string());
    let cloned = analysis.clone();
    assert_eq!(cloned.issue, "Clone test");
    let _ = format!("{:?}", analysis);
}

// ============ WhyIteration Tests ============

#[test]
fn test_why_iteration_new() {
    let why = WhyIteration::new(
        1,
        "Why does it crash?".to_string(),
        "Memory overflow".to_string(),
    );
    assert_eq!(why.depth, 1);
    assert_eq!(why.question, "Why does it crash?");
    assert_eq!(why.hypothesis, "Memory overflow");
    assert!(why.evidence.is_empty());
    assert!((why.confidence - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_why_iteration_with_confidence() {
    let why = WhyIteration::new(1, "Q".to_string(), "H".to_string()).with_confidence(0.9);
    assert!((why.confidence - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_why_iteration_with_confidence_clamped_high() {
    let why = WhyIteration::new(1, "Q".to_string(), "H".to_string()).with_confidence(1.5);
    assert!((why.confidence - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_why_iteration_with_confidence_clamped_low() {
    let why = WhyIteration::new(1, "Q".to_string(), "H".to_string()).with_confidence(-0.5);
    assert!((why.confidence - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_why_iteration_add_evidence() {
    let mut why = WhyIteration::new(1, "Q".to_string(), "H".to_string());
    assert!(why.evidence.is_empty());

    why.add_evidence(Evidence::new(
        EvidenceSource::Complexity,
        PathBuf::from("test.rs"),
        "cyclomatic".to_string(),
        serde_json::json!({"value": 25}),
        "High complexity".to_string(),
    ));
    assert_eq!(why.evidence.len(), 1);
}

#[test]
fn test_why_iteration_serde() {
    let why = WhyIteration::new(3, "Why slow?".to_string(), "O(n^2) loop".to_string())
        .with_confidence(0.8);
    let json = serde_json::to_string(&why).unwrap();
    let back: WhyIteration = serde_json::from_str(&json).unwrap();
    assert_eq!(back.depth, 3);
    assert!((back.confidence - 0.8).abs() < f64::EPSILON);
}

// ============ Evidence Tests ============

#[test]
fn test_evidence_new() {
    let evidence = Evidence::new(
        EvidenceSource::TDG,
        PathBuf::from("src/main.rs"),
        "tdg_score".to_string(),
        serde_json::json!(2.5),
        "High TDG".to_string(),
    );
    assert_eq!(evidence.source, EvidenceSource::TDG);
    assert_eq!(evidence.metric, "tdg_score");
}

#[test]
fn test_evidence_serde() {
    let evidence = Evidence::new(
        EvidenceSource::SATD,
        PathBuf::from("lib.rs"),
        "count".to_string(),
        serde_json::json!({"count": 5}),
        "Multiple TODOs".to_string(),
    );
    let json = serde_json::to_string(&evidence).unwrap();
    let back: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source, EvidenceSource::SATD);
}

// ============ EvidenceSource Tests ============

#[test]
fn test_evidence_source_all_variants_serde() {
    let sources = vec![
        EvidenceSource::Complexity,
        EvidenceSource::SATD,
        EvidenceSource::DeadCode,
        EvidenceSource::GitChurn,
        EvidenceSource::TDG,
        EvidenceSource::ManualInspection,
    ];
    for src in &sources {
        let json = serde_json::to_string(src).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(*src, back);
    }
}

#[test]
fn test_evidence_source_eq() {
    assert_eq!(EvidenceSource::Complexity, EvidenceSource::Complexity);
    assert_ne!(EvidenceSource::Complexity, EvidenceSource::SATD);
}

#[test]
fn test_evidence_source_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(EvidenceSource::Complexity);
    set.insert(EvidenceSource::TDG);
    set.insert(EvidenceSource::Complexity);
    assert_eq!(set.len(), 2);
}

// ============ Recommendation Tests ============

#[test]
fn test_recommendation_new() {
    let rec = Recommendation::new(
        Priority::High,
        "Fix memory leak".to_string(),
        Some(PathBuf::from("src/cache.rs")),
    );
    assert_eq!(rec.priority, Priority::High);
    assert_eq!(rec.action, "Fix memory leak");
    assert!(rec.file.is_some());
}

#[test]
fn test_recommendation_high() {
    let rec = Recommendation::high("Critical fix".to_string(), Some(PathBuf::from("main.rs")));
    assert_eq!(rec.priority, Priority::High);
}

#[test]
fn test_recommendation_medium() {
    let rec = Recommendation::medium("Refactor".to_string(), None);
    assert_eq!(rec.priority, Priority::Medium);
    assert!(rec.file.is_none());
}

#[test]
fn test_recommendation_low() {
    let rec = Recommendation::low("Nice to have".to_string(), None);
    assert_eq!(rec.priority, Priority::Low);
}

#[test]
fn test_recommendation_serde() {
    let rec = Recommendation::high("Fix it".to_string(), None);
    let json = serde_json::to_string(&rec).unwrap();
    let back: Recommendation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.priority, Priority::High);
}

// ============ Priority Tests ============

#[test]
fn test_priority_serde() {
    let priorities = vec![Priority::High, Priority::Medium, Priority::Low];
    for p in &priorities {
        let json = serde_json::to_string(p).unwrap();
        let back: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(*p, back);
    }
}

#[test]
fn test_priority_eq() {
    assert_eq!(Priority::High, Priority::High);
    assert_ne!(Priority::High, Priority::Low);
}

// ============ EvidenceSummary Tests ============

#[test]
fn test_evidence_summary_default() {
    let summary = EvidenceSummary::default();
    assert_eq!(summary.complexity_violations, 0);
    assert_eq!(summary.satd_markers, 0);
    assert_eq!(summary.tdg_score, 0.0);
    assert!(!summary.git_churn_high);
}

#[test]
fn test_evidence_summary_serde() {
    let summary = EvidenceSummary {
        complexity_violations: 3,
        satd_markers: 5,
        tdg_score: 2.1,
        git_churn_high: true,
        evoscore_trajectory: 0.0,
        coverage_delta: 0.0,
    };
    let json = serde_json::to_string(&summary).unwrap();
    let back: EvidenceSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity_violations, 3);
    assert!(back.git_churn_high);
}

#[test]
fn test_evidence_summary_from_whys_empty() {
    let summary = EvidenceSummary::from_whys(&[]);
    assert_eq!(summary.complexity_violations, 0);
    assert_eq!(summary.satd_markers, 0);
}

#[test]
fn test_evidence_summary_from_whys_with_satd() {
    let mut why = WhyIteration::new(1, "Q".to_string(), "H".to_string());
    why.add_evidence(Evidence::new(
        EvidenceSource::SATD,
        PathBuf::from("test.rs"),
        "count".to_string(),
        serde_json::json!({"count": 3}),
        "Found TODOs".to_string(),
    ));
    let summary = EvidenceSummary::from_whys(&[why]);
    assert_eq!(summary.satd_markers, 3);
}

#[test]
fn test_evidence_summary_from_whys_with_complexity() {
    let mut why = WhyIteration::new(1, "Q".to_string(), "H".to_string());
    why.add_evidence(Evidence::new(
        EvidenceSource::Complexity,
        PathBuf::from("test.rs"),
        "complexity".to_string(),
        serde_json::json!({"value": 25.0, "threshold": 20.0}),
        "High complexity".to_string(),
    ));
    let summary = EvidenceSummary::from_whys(&[why]);
    assert_eq!(summary.complexity_violations, 1);
}
