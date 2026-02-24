// ============================================================
// QualityViolation Tests
// ============================================================

#[test]
fn test_quality_violation_excessive_complexity_display() {
    let violation = QualityViolation::ExcessiveComplexity {
        found: 25,
        max: 10,
        location: std::path::PathBuf::from("/path/to/file.rs"),
    };
    let display = format!("{}", violation);
    assert!(display.contains("Excessive complexity"));
    assert!(display.contains("25"));
    assert!(display.contains("10"));
    assert!(display.contains("file.rs"));
}

#[test]
fn test_quality_violation_satd_detected_display() {
    let violation = QualityViolation::SatdDetected {
        count: 3,
        patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        location: std::path::PathBuf::from("/path/to/module.rs"),
    };
    let display = format!("{}", violation);
    assert!(display.contains("SATD detected"));
    assert!(display.contains("3"));
    assert!(display.contains("TODO"));
    assert!(display.contains("FIXME"));
}

#[test]
fn test_quality_violation_inefficient_algorithm_display() {
    let violation = QualityViolation::InefficientAlgorithm {
        function: "bubble_sort".to_string(),
        complexity: "O(n^2)".to_string(),
        required: "O(n log n)".to_string(),
    };
    let display = format!("{}", violation);
    assert!(display.contains("Inefficient algorithm"));
    assert!(display.contains("bubble_sort"));
    assert!(display.contains("O(n^2)"));
    assert!(display.contains("O(n log n)"));
}

#[test]
fn test_quality_violation_insufficient_diversity_display() {
    let violation = QualityViolation::InsufficientDiversity {
        entropy: 2.5,
        required: 3.5,
    };
    let display = format!("{}", violation);
    assert!(display.contains("Insufficient diversity"));
    assert!(display.contains("2.5"));
    assert!(display.contains("3.5"));
}

#[test]
fn test_quality_violation_parse_error_display() {
    let violation = QualityViolation::ParseError("unexpected token".to_string());
    let display = format!("{}", violation);
    assert!(display.contains("Parse error"));
    assert!(display.contains("unexpected token"));
}

#[test]
fn test_quality_violation_debug() {
    let violation = QualityViolation::ParseError("test error".to_string());
    let debug = format!("{:?}", violation);
    assert!(debug.contains("ParseError"));
}

// ============================================================
// QualityThresholds Tests
// ============================================================

#[test]
fn test_quality_thresholds_default() {
    let thresholds = QualityThresholds::default();
    assert_eq!(thresholds.max_cyclomatic, 10);
    assert_eq!(thresholds.max_cognitive, 7);
    assert_eq!(thresholds.max_nesting, 3);
    assert_eq!(thresholds.max_params, 4);
    assert_eq!(thresholds.max_lines, 50);
    assert_eq!(thresholds.satd_tolerance, 0);
    assert_eq!(thresholds.max_big_o, "O(n log n)");
    assert!((thresholds.min_entropy - 3.5).abs() < 0.001);
}

#[test]
fn test_quality_thresholds_clone() {
    let original = QualityThresholds::default();
    let cloned = original.clone();
    assert_eq!(original.max_cyclomatic, cloned.max_cyclomatic);
    assert_eq!(original.max_cognitive, cloned.max_cognitive);
    assert_eq!(original.max_big_o, cloned.max_big_o);
}

#[test]
fn test_quality_thresholds_serialization() {
    let thresholds = QualityThresholds::default();
    let json = serde_json::to_string(&thresholds).unwrap();
    let deserialized: QualityThresholds = serde_json::from_str(&json).unwrap();
    assert_eq!(thresholds.max_cyclomatic, deserialized.max_cyclomatic);
    assert_eq!(thresholds.max_cognitive, deserialized.max_cognitive);
    assert_eq!(thresholds.max_nesting, deserialized.max_nesting);
    assert_eq!(thresholds.max_params, deserialized.max_params);
    assert_eq!(thresholds.max_lines, deserialized.max_lines);
    assert_eq!(thresholds.satd_tolerance, deserialized.satd_tolerance);
    assert_eq!(thresholds.max_big_o, deserialized.max_big_o);
    assert!((thresholds.min_entropy - deserialized.min_entropy).abs() < 0.001);
}

#[test]
fn test_quality_thresholds_custom() {
    let thresholds = QualityThresholds {
        max_cyclomatic: 20,
        max_cognitive: 15,
        max_nesting: 5,
        max_params: 6,
        max_lines: 100,
        satd_tolerance: 5,
        max_big_o: "O(n^2)".to_string(),
        min_entropy: 2.0,
    };
    assert_eq!(thresholds.max_cyclomatic, 20);
    assert_eq!(thresholds.max_cognitive, 15);
    assert_eq!(thresholds.max_nesting, 5);
}

#[test]
fn test_quality_thresholds_debug() {
    let thresholds = QualityThresholds::default();
    let debug = format!("{:?}", thresholds);
    assert!(debug.contains("QualityThresholds"));
    assert!(debug.contains("max_cyclomatic"));
}

// ============================================================
// QualityReport Tests
// ============================================================

#[test]
fn test_quality_report_passed() {
    let report = QualityReport::passed();
    assert!(report.passed);
    assert!(report.violations.is_empty());
}

#[test]
fn test_quality_report_clone() {
    let report = QualityReport {
        passed: false,
        metrics: QualityMetrics::default(),
        violations: vec!["violation1".to_string(), "violation2".to_string()],
    };
    let cloned = report.clone();
    assert_eq!(report.passed, cloned.passed);
    assert_eq!(report.violations.len(), cloned.violations.len());
}

#[test]
fn test_quality_report_serialization() {
    let report = QualityReport {
        passed: true,
        metrics: QualityMetrics::default(),
        violations: vec!["test violation".to_string()],
    };
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: QualityReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report.passed, deserialized.passed);
    assert_eq!(report.violations.len(), deserialized.violations.len());
}

#[test]
fn test_quality_report_debug() {
    let report = QualityReport::passed();
    let debug = format!("{:?}", report);
    assert!(debug.contains("QualityReport"));
    assert!(debug.contains("passed"));
}

// ============================================================
// QualityMetrics Tests
// ============================================================

#[test]
fn test_quality_metrics_default() {
    let metrics = QualityMetrics::default();
    assert_eq!(metrics.cyclomatic_complexity, 1);
    assert_eq!(metrics.cognitive_complexity, 0);
    assert_eq!(metrics.nesting_depth, 0);
    assert_eq!(metrics.satd_count, 0);
    assert!((metrics.entropy - 0.0).abs() < 0.001);
    assert_eq!(metrics.efficiency, "O(1)");
}

#[test]
fn test_quality_metrics_clone() {
    let metrics = QualityMetrics {
        cyclomatic_complexity: 15,
        cognitive_complexity: 10,
        nesting_depth: 4,
        satd_count: 2,
        entropy: 4.5,
        efficiency: "O(n)".to_string(),
    };
    let cloned = metrics.clone();
    assert_eq!(metrics.cyclomatic_complexity, cloned.cyclomatic_complexity);
    assert_eq!(metrics.cognitive_complexity, cloned.cognitive_complexity);
    assert_eq!(metrics.efficiency, cloned.efficiency);
}

#[test]
fn test_quality_metrics_serialization() {
    let metrics = QualityMetrics {
        cyclomatic_complexity: 5,
        cognitive_complexity: 3,
        nesting_depth: 2,
        satd_count: 1,
        entropy: 3.8,
        efficiency: "O(n log n)".to_string(),
    };
    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(
        metrics.cyclomatic_complexity,
        deserialized.cyclomatic_complexity
    );
    assert_eq!(
        metrics.cognitive_complexity,
        deserialized.cognitive_complexity
    );
    assert_eq!(metrics.efficiency, deserialized.efficiency);
}

#[test]
fn test_quality_metrics_debug() {
    let metrics = QualityMetrics::default();
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("QualityMetrics"));
    assert!(debug.contains("cyclomatic_complexity"));
}
