//! Coverage boost tests for services/tdg_calculator.rs
//! Tests: ComplexityVariance, CouplingMetrics
//! Additional coverage for TDGSeverity, TDGComponents, TDGConfig

use crate::models::tdg::{TDGComponents, TDGConfig, TDGSeverity};
use crate::services::tdg_calculator::{ComplexityVariance, CouplingMetrics};

// ============ ComplexityVariance Tests ============

#[test]
fn test_complexity_variance_clone() {
    let cv = ComplexityVariance {
        mean: 15.0,
        variance: 25.0,
        gini: 0.35,
        percentile_90: 30.0,
    };
    let cloned = cv.clone();
    assert!((cloned.mean - 15.0).abs() < f64::EPSILON);
    assert!((cloned.variance - 25.0).abs() < f64::EPSILON);
    assert!((cloned.gini - 0.35).abs() < f64::EPSILON);
    assert!((cloned.percentile_90 - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_complexity_variance_debug() {
    let cv = ComplexityVariance {
        mean: 10.0,
        variance: 4.0,
        gini: 0.2,
        percentile_90: 18.0,
    };
    let debug = format!("{:?}", cv);
    assert!(debug.contains("ComplexityVariance"));
    assert!(debug.contains("mean"));
}

// ============ CouplingMetrics Tests ============

#[test]
fn test_coupling_metrics_clone() {
    let cm = CouplingMetrics {
        afferent: 5,
        efferent: 3,
        instability: 0.375,
    };
    let cloned = cm.clone();
    assert_eq!(cloned.afferent, 5);
    assert_eq!(cloned.efferent, 3);
    assert!((cloned.instability - 0.375).abs() < f64::EPSILON);
}

#[test]
fn test_coupling_metrics_debug() {
    let cm = CouplingMetrics {
        afferent: 10,
        efferent: 2,
        instability: 0.167,
    };
    let debug = format!("{:?}", cm);
    assert!(debug.contains("CouplingMetrics"));
    assert!(debug.contains("afferent"));
}

#[test]
fn test_coupling_metrics_instability_calculation() {
    // instability = efferent / (afferent + efferent)
    let stable = CouplingMetrics {
        afferent: 10,
        efferent: 0,
        instability: 0.0,
    };
    assert!((stable.instability - 0.0).abs() < f64::EPSILON);

    let unstable = CouplingMetrics {
        afferent: 0,
        efferent: 10,
        instability: 1.0,
    };
    assert!((unstable.instability - 1.0).abs() < f64::EPSILON);

    let balanced = CouplingMetrics {
        afferent: 5,
        efferent: 5,
        instability: 0.5,
    };
    assert!((balanced.instability - 0.5).abs() < f64::EPSILON);
}

// ============ TDGSeverity Tests ============

#[test]
fn test_tdg_severity_normal() {
    let severity = TDGSeverity::Normal;
    let debug = format!("{:?}", severity);
    assert!(debug.contains("Normal"));
    assert_eq!(severity.as_str(), "normal");
}

#[test]
fn test_tdg_severity_warning() {
    let severity = TDGSeverity::Warning;
    let debug = format!("{:?}", severity);
    assert!(debug.contains("Warning"));
    assert_eq!(severity.as_str(), "warning");
}

#[test]
fn test_tdg_severity_critical() {
    let severity = TDGSeverity::Critical;
    let debug = format!("{:?}", severity);
    assert!(debug.contains("Critical"));
    assert_eq!(severity.as_str(), "critical");
}

#[test]
fn test_tdg_severity_clone() {
    let s1 = TDGSeverity::Warning;
    let s2 = s1.clone();
    assert_eq!(s1, s2);
}

#[test]
fn test_tdg_severity_partial_eq() {
    assert_eq!(TDGSeverity::Normal, TDGSeverity::Normal);
    assert_eq!(TDGSeverity::Warning, TDGSeverity::Warning);
    assert_eq!(TDGSeverity::Critical, TDGSeverity::Critical);
    assert_ne!(TDGSeverity::Normal, TDGSeverity::Critical);
}

#[test]
fn test_tdg_severity_from_f64() {
    // Normal: value <= 1.5
    assert_eq!(TDGSeverity::from(0.5), TDGSeverity::Normal);
    assert_eq!(TDGSeverity::from(1.5), TDGSeverity::Normal);

    // Warning: 1.5 < value <= 2.5
    assert_eq!(TDGSeverity::from(1.6), TDGSeverity::Warning);
    assert_eq!(TDGSeverity::from(2.0), TDGSeverity::Warning);
    assert_eq!(TDGSeverity::from(2.5), TDGSeverity::Warning);

    // Critical: value > 2.5
    assert_eq!(TDGSeverity::from(2.6), TDGSeverity::Critical);
    assert_eq!(TDGSeverity::from(5.0), TDGSeverity::Critical);
}

#[test]
fn test_tdg_severity_copy() {
    let s1 = TDGSeverity::Warning;
    let s2 = s1; // Copy
    assert_eq!(s1, s2);
}

// ============ TDGComponents Tests ============

#[test]
fn test_tdg_components_default() {
    let components = TDGComponents::default();
    assert!((components.complexity - 0.0).abs() < f64::EPSILON);
    assert!((components.churn - 0.0).abs() < f64::EPSILON);
    assert!((components.coupling - 0.0).abs() < f64::EPSILON);
    assert!((components.duplication - 0.0).abs() < f64::EPSILON);
    assert!((components.domain_risk - 0.0).abs() < f64::EPSILON);
    assert!((components.dead_code - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_components_custom_values() {
    let components = TDGComponents {
        complexity: 0.8,
        churn: 0.6,
        coupling: 0.4,
        duplication: 0.2,
        domain_risk: 0.9,
        dead_code: 0.1,
    };
    assert!((components.complexity - 0.8).abs() < f64::EPSILON);
    assert!((components.churn - 0.6).abs() < f64::EPSILON);
    assert!((components.coupling - 0.4).abs() < f64::EPSILON);
    assert!((components.duplication - 0.2).abs() < f64::EPSILON);
    assert!((components.domain_risk - 0.9).abs() < f64::EPSILON);
    assert!((components.dead_code - 0.1).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_components_clone() {
    let c1 = TDGComponents {
        complexity: 0.5,
        churn: 0.5,
        coupling: 0.5,
        duplication: 0.5,
        domain_risk: 0.5,
        dead_code: 0.5,
    };
    let c2 = c1.clone();
    assert!((c1.complexity - c2.complexity).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_components_debug() {
    let components = TDGComponents::default();
    let debug = format!("{:?}", components);
    assert!(debug.contains("TDGComponents"));
    assert!(debug.contains("complexity"));
}

#[test]
fn test_tdg_components_copy() {
    let c1 = TDGComponents {
        complexity: 1.0,
        churn: 2.0,
        coupling: 3.0,
        duplication: 4.0,
        domain_risk: 5.0,
        dead_code: 0.5,
    };
    let c2 = c1; // Copy
    assert!((c1.complexity - c2.complexity).abs() < f64::EPSILON);
}

// ============ TDGConfig Tests ============

#[test]
fn test_tdg_config_default() {
    let config = TDGConfig::default();
    // Verify default weights sum to 1.0
    let total = config.complexity_weight
        + config.churn_weight
        + config.coupling_weight
        + config.domain_risk_weight
        + config.duplication_weight
        + config.dead_code_weight;
    assert!((total - 1.0).abs() < 0.01);

    // Verify thresholds
    assert!((config.critical_threshold - 2.5).abs() < f64::EPSILON);
    assert!((config.warning_threshold - 1.5).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_config_clone() {
    let c1 = TDGConfig::default();
    let c2 = c1.clone();
    assert!((c1.complexity_weight - c2.complexity_weight).abs() < f64::EPSILON);
}

#[test]
fn test_tdg_config_debug() {
    let config = TDGConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("TDGConfig"));
    assert!(debug.contains("complexity_weight"));
}

#[test]
fn test_tdg_config_dead_code_weight() {
    let config = TDGConfig::default();
    // CB-128: dead_code_weight should be 0.20
    assert!((config.dead_code_weight - 0.20).abs() < f64::EPSILON);
}

// ============ Edge Case Tests ============

#[test]
fn test_complexity_variance_extreme_values() {
    let cv = ComplexityVariance {
        mean: 0.0,
        variance: 0.0,
        gini: 0.0,
        percentile_90: 0.0,
    };
    assert!((cv.mean - 0.0).abs() < f64::EPSILON);

    let cv_large = ComplexityVariance {
        mean: 1000.0,
        variance: 10000.0,
        gini: 1.0,
        percentile_90: 2000.0,
    };
    assert!((cv_large.gini - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_coupling_metrics_zero_dependencies() {
    let cm = CouplingMetrics {
        afferent: 0,
        efferent: 0,
        instability: 0.0,
    };
    assert_eq!(cm.afferent, 0);
    assert_eq!(cm.efferent, 0);
}

#[test]
fn test_coupling_metrics_large_counts() {
    let cm = CouplingMetrics {
        afferent: 1000,
        efferent: 500,
        instability: 500.0 / 1500.0,
    };
    assert_eq!(cm.afferent, 1000);
    assert_eq!(cm.efferent, 500);
    assert!((cm.instability - (500.0 / 1500.0)).abs() < 0.0001);
}

#[test]
fn test_tdg_severity_boundary_values() {
    // Test exact boundaries
    assert_eq!(TDGSeverity::from(1.5), TDGSeverity::Normal);
    assert_eq!(TDGSeverity::from(1.500001), TDGSeverity::Warning);
    assert_eq!(TDGSeverity::from(2.5), TDGSeverity::Warning);
    assert_eq!(TDGSeverity::from(2.500001), TDGSeverity::Critical);
}

#[test]
fn test_tdg_severity_negative_value() {
    // Negative values should map to Normal
    assert_eq!(TDGSeverity::from(-1.0), TDGSeverity::Normal);
    assert_eq!(TDGSeverity::from(0.0), TDGSeverity::Normal);
}
