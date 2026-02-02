//! Coverage boost tests for models/tdg.rs
//! Tests: TDGScore, TDGComponents, TDGSeverity, TDGConfig, TDGSummary,
//! TDGHotspot, TDGAnalysis, TDGRecommendation, RecommendationType, TDGDistribution

use crate::models::tdg::{
    RecommendationType, TDGAnalysis, TDGBucket, TDGComponents, TDGConfig, TDGDistribution,
    TDGHotspot, TDGRecommendation, TDGScore, TDGSeverity, TDGSummary,
};

// ============ TDGSeverity Tests ============

#[test]
fn test_tdg_severity_from_low() {
    let severity = TDGSeverity::from(0.5);
    assert_eq!(severity, TDGSeverity::Normal);
}

#[test]
fn test_tdg_severity_from_warning() {
    let severity = TDGSeverity::from(2.0);
    assert_eq!(severity, TDGSeverity::Warning);
}

#[test]
fn test_tdg_severity_from_critical() {
    let severity = TDGSeverity::from(3.0);
    assert_eq!(severity, TDGSeverity::Critical);
}

#[test]
fn test_tdg_severity_boundary_normal_warning() {
    assert_eq!(TDGSeverity::from(1.5), TDGSeverity::Normal);
    assert_eq!(TDGSeverity::from(1.501), TDGSeverity::Warning);
}

#[test]
fn test_tdg_severity_boundary_warning_critical() {
    assert_eq!(TDGSeverity::from(2.5), TDGSeverity::Warning);
    assert_eq!(TDGSeverity::from(2.501), TDGSeverity::Critical);
}

#[test]
fn test_tdg_severity_as_str() {
    assert_eq!(TDGSeverity::Normal.as_str(), "normal");
    assert_eq!(TDGSeverity::Warning.as_str(), "warning");
    assert_eq!(TDGSeverity::Critical.as_str(), "critical");
}

#[test]
fn test_tdg_severity_serde() {
    let severities = vec![TDGSeverity::Normal, TDGSeverity::Warning, TDGSeverity::Critical];
    for sev in &severities {
        let json = serde_json::to_string(sev).unwrap();
        let back: TDGSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(*sev, back);
    }
}

#[test]
fn test_tdg_severity_rename_all_lowercase() {
    let json = serde_json::to_string(&TDGSeverity::Critical).unwrap();
    assert!(json.contains("critical"));
}

// ============ TDGComponents Tests ============

#[test]
fn test_tdg_components_default() {
    let comp = TDGComponents::default();
    assert_eq!(comp.complexity, 0.0);
    assert_eq!(comp.churn, 0.0);
    assert_eq!(comp.coupling, 0.0);
    assert_eq!(comp.domain_risk, 0.0);
    assert_eq!(comp.duplication, 0.0);
    assert_eq!(comp.dead_code, 0.0);
}

#[test]
fn test_tdg_components_serde() {
    let comp = TDGComponents {
        complexity: 1.5,
        churn: 2.0,
        coupling: 0.8,
        domain_risk: 0.3,
        duplication: 0.1,
        dead_code: 0.5,
    };
    let json = serde_json::to_string(&comp).unwrap();
    let back: TDGComponents = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity, 1.5);
    assert_eq!(back.dead_code, 0.5);
}

#[test]
fn test_tdg_components_copy() {
    let comp = TDGComponents { complexity: 1.0, ..Default::default() };
    let copied = comp;
    assert_eq!(copied.complexity, 1.0);
}

// ============ TDGConfig Tests ============

#[test]
fn test_tdg_config_default() {
    let config = TDGConfig::default();
    assert_eq!(config.complexity_weight, 0.25);
    assert_eq!(config.churn_weight, 0.20);
    assert_eq!(config.coupling_weight, 0.15);
    assert_eq!(config.domain_risk_weight, 0.10);
    assert_eq!(config.duplication_weight, 0.10);
    assert_eq!(config.dead_code_weight, 0.20);
    assert_eq!(config.critical_threshold, 2.5);
    assert_eq!(config.warning_threshold, 1.5);
}

#[test]
fn test_tdg_config_weights_sum_to_one() {
    let config = TDGConfig::default();
    let total = config.complexity_weight
        + config.churn_weight
        + config.coupling_weight
        + config.domain_risk_weight
        + config.duplication_weight
        + config.dead_code_weight;
    assert!((total - 1.0).abs() < 0.001);
}

#[test]
fn test_tdg_config_serde() {
    let config = TDGConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let back: TDGConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity_weight, config.complexity_weight);
}

// ============ TDGScore Tests ============

#[test]
fn test_tdg_score_serde() {
    let score = TDGScore {
        value: 1.8,
        components: TDGComponents {
            complexity: 1.0,
            churn: 0.5,
            coupling: 0.2,
            domain_risk: 0.1,
            duplication: 0.0,
            dead_code: 0.0,
        },
        severity: TDGSeverity::Warning,
        percentile: 75.0,
        confidence: 0.9,
    };
    let json = serde_json::to_string(&score).unwrap();
    let back: TDGScore = serde_json::from_str(&json).unwrap();
    assert_eq!(back.value, 1.8);
    assert_eq!(back.severity, TDGSeverity::Warning);
}

// ============ TDGSummary Tests ============

#[test]
fn test_tdg_summary_serde() {
    let summary = TDGSummary {
        total_files: 100,
        critical_files: 5,
        warning_files: 15,
        average_tdg: 1.2,
        p95_tdg: 2.8,
        p99_tdg: 3.5,
        estimated_debt_hours: 120.0,
        hotspots: vec![],
    };
    let json = serde_json::to_string(&summary).unwrap();
    let back: TDGSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_files, 100);
    assert_eq!(back.critical_files, 5);
}

// ============ TDGHotspot Tests ============

#[test]
fn test_tdg_hotspot_serde() {
    let hotspot = TDGHotspot {
        path: "src/complex.rs".to_string(),
        tdg_score: 3.2,
        primary_factor: "complexity".to_string(),
        estimated_hours: 8.0,
    };
    let json = serde_json::to_string(&hotspot).unwrap();
    let back: TDGHotspot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/complex.rs");
}

// ============ TDGRecommendation Tests ============

#[test]
fn test_tdg_recommendation_serde() {
    let rec = TDGRecommendation {
        recommendation_type: RecommendationType::ReduceComplexity,
        action: "Extract method from process_data()".to_string(),
        expected_reduction: 0.5,
        estimated_hours: 4.0,
        priority: 5,
    };
    let json = serde_json::to_string(&rec).unwrap();
    let back: TDGRecommendation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.priority, 5);
}

// ============ RecommendationType Tests ============

#[test]
fn test_recommendation_type_all_variants_serde() {
    let types = vec![
        RecommendationType::ReduceComplexity,
        RecommendationType::StabilizeChurn,
        RecommendationType::ReduceCoupling,
        RecommendationType::AddressDomainRisk,
        RecommendationType::RemoveDuplication,
        RecommendationType::SplitFile,
        RecommendationType::AddTests,
    ];
    for rt in &types {
        let json = serde_json::to_string(rt).unwrap();
        let back: RecommendationType = serde_json::from_str(&json).unwrap();
        assert_eq!(*rt, back);
    }
}

// ============ TDGAnalysis Tests ============

#[test]
fn test_tdg_analysis_serde() {
    let analysis = TDGAnalysis {
        score: TDGScore {
            value: 1.0,
            components: TDGComponents::default(),
            severity: TDGSeverity::Normal,
            percentile: 50.0,
            confidence: 0.8,
        },
        explanation: "Low complexity".to_string(),
        recommendations: vec![],
    };
    let json = serde_json::to_string(&analysis).unwrap();
    let back: TDGAnalysis = serde_json::from_str(&json).unwrap();
    assert!(back.recommendations.is_empty());
}

// ============ TDGDistribution Tests ============

#[test]
fn test_tdg_distribution_serde() {
    let dist = TDGDistribution {
        buckets: vec![
            TDGBucket { min: 0.0, max: 1.0, count: 50, percentage: 50.0 },
            TDGBucket { min: 1.0, max: 2.0, count: 30, percentage: 30.0 },
            TDGBucket { min: 2.0, max: 5.0, count: 20, percentage: 20.0 },
        ],
        total_files: 100,
    };
    let json = serde_json::to_string(&dist).unwrap();
    let back: TDGDistribution = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_files, 100);
    assert_eq!(back.buckets.len(), 3);
}

#[test]
fn test_tdg_bucket_serde() {
    let bucket = TDGBucket {
        min: 0.0,
        max: 1.0,
        count: 42,
        percentage: 42.0,
    };
    let json = serde_json::to_string(&bucket).unwrap();
    let back: TDGBucket = serde_json::from_str(&json).unwrap();
    assert_eq!(back.count, 42);
}
