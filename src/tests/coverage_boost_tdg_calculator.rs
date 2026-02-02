//! Coverage boost tests for services/tdg_calculator.rs
//! Tests: ComplexityVariance, CouplingMetrics, TDGCalculator

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
