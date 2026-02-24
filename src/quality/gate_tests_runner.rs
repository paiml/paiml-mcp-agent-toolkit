// ============================================================
// QualityGateRunner Tests
// ============================================================

#[test]
fn test_quality_gate_runner_new() {
    let thresholds = QualityThresholds::default();
    let runner = QualityGateRunner::new(thresholds);
    assert_eq!(runner.thresholds.max_cyclomatic, 10);
}

#[test]
fn test_quality_gate_runner_strict() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.thresholds.max_cyclomatic, 10);
    assert_eq!(runner.thresholds.satd_tolerance, 0);
}

#[test]
fn test_quality_gate_runner_custom_thresholds() {
    let thresholds = QualityThresholds {
        max_cyclomatic: 50,
        max_cognitive: 30,
        max_nesting: 10,
        max_params: 10,
        max_lines: 200,
        satd_tolerance: 10,
        max_big_o: "O(n^3)".to_string(),
        min_entropy: 1.0,
    };
    let runner = QualityGateRunner::new(thresholds);
    assert_eq!(runner.thresholds.max_cyclomatic, 50);
    assert_eq!(runner.thresholds.satd_tolerance, 10);
}

// ============================================================
// parse_big_o Tests
// ============================================================

#[test]
fn test_parse_big_o_constant() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(1)"), 1);
}

#[test]
fn test_parse_big_o_logarithmic() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(log n)"), 2);
}

#[test]
fn test_parse_big_o_linear() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(n)"), 3);
}

#[test]
fn test_parse_big_o_linearithmic() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(n log n)"), 4);
}

#[test]
fn test_parse_big_o_quadratic() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(n^2)"), 5);
}

#[test]
fn test_parse_big_o_cubic() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(n^3)"), 6);
}

#[test]
fn test_parse_big_o_unknown() {
    let runner = QualityGateRunner::strict();
    assert_eq!(runner.parse_big_o("O(2^n)"), 10);
    assert_eq!(runner.parse_big_o("unknown"), 10);
    assert_eq!(runner.parse_big_o(""), 10);
}

// ============================================================
// is_efficiency_acceptable Tests
// ============================================================

#[test]
fn test_is_efficiency_acceptable_within_threshold() {
    let runner = QualityGateRunner::strict(); // max is O(n log n)
    assert!(runner.is_efficiency_acceptable("O(1)"));
    assert!(runner.is_efficiency_acceptable("O(log n)"));
    assert!(runner.is_efficiency_acceptable("O(n)"));
    assert!(runner.is_efficiency_acceptable("O(n log n)"));
}

#[test]
fn test_is_efficiency_acceptable_exceeds_threshold() {
    let runner = QualityGateRunner::strict(); // max is O(n log n)
    assert!(!runner.is_efficiency_acceptable("O(n^2)"));
    assert!(!runner.is_efficiency_acceptable("O(n^3)"));
}

#[test]
fn test_is_efficiency_acceptable_custom_threshold() {
    let thresholds = QualityThresholds {
        max_big_o: "O(n^2)".to_string(),
        ..Default::default()
    };
    let runner = QualityGateRunner::new(thresholds);
    assert!(runner.is_efficiency_acceptable("O(n^2)"));
    assert!(!runner.is_efficiency_acceptable("O(n^3)"));
}
