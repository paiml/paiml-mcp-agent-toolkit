#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use std::path::Path;

// ===================
// ComplexityMetrics Tests
// ===================

#[test]
fn test_complexity_metrics_new() {
    let metrics = ComplexityMetrics::new(5, 8, 3, 42);
    assert_eq!(metrics.cyclomatic, 5);
    assert_eq!(metrics.cognitive, 8);
    assert_eq!(metrics.nesting_max, 3);
    assert_eq!(metrics.lines, 42);
    assert!(metrics.halstead.is_none());
}

#[test]
fn test_complexity_metrics_with_halstead() {
    let halstead = HalsteadMetrics::new(10, 5, 20, 8);
    let metrics = ComplexityMetrics::with_halstead(5, 8, 3, 42, halstead);

    assert_eq!(metrics.cyclomatic, 5);
    assert_eq!(metrics.cognitive, 8);
    assert!(metrics.halstead.is_some());
    assert_eq!(metrics.halstead.unwrap().operators_unique, 10);
}

#[test]
fn test_complexity_metrics_is_simple_true() {
    let simple = ComplexityMetrics::new(2, 3, 1, 10);
    assert!(simple.is_simple());
}

#[test]
fn test_complexity_metrics_is_simple_false_cyclomatic() {
    let complex = ComplexityMetrics::new(6, 3, 1, 10);
    assert!(!complex.is_simple());
}

#[test]
fn test_complexity_metrics_is_simple_false_cognitive() {
    let complex = ComplexityMetrics::new(3, 8, 1, 10);
    assert!(!complex.is_simple());
}

#[test]
fn test_complexity_metrics_is_simple_boundary() {
    // Exactly at boundary should be simple
    let boundary = ComplexityMetrics::new(5, 7, 2, 20);
    assert!(boundary.is_simple());
}

#[test]
fn test_complexity_metrics_needs_refactoring_false() {
    let simple = ComplexityMetrics::new(3, 4, 2, 15);
    assert!(!simple.needs_refactoring());
}

#[test]
fn test_complexity_metrics_needs_refactoring_cyclomatic() {
    let complex = ComplexityMetrics::new(15, 4, 2, 15);
    assert!(complex.needs_refactoring());
}

#[test]
fn test_complexity_metrics_needs_refactoring_cognitive() {
    let complex = ComplexityMetrics::new(5, 20, 2, 15);
    assert!(complex.needs_refactoring());
}

#[test]
fn test_complexity_metrics_needs_refactoring_boundary() {
    // Exactly at boundary should NOT need refactoring
    let boundary = ComplexityMetrics::new(10, 15, 3, 30);
    assert!(!boundary.needs_refactoring());
}

#[test]
fn test_complexity_metrics_complexity_score() {
    let simple = ComplexityMetrics::new(1, 1, 1, 5);
    let complex = ComplexityMetrics::new(10, 15, 4, 80);

    assert!(complex.complexity_score() > simple.complexity_score());
}

#[test]
fn test_complexity_metrics_complexity_score_calculation() {
    let metrics = ComplexityMetrics::new(10, 20, 3, 100);
    // Score = (10*1.0) + (20*1.2) + (3*2.0) + (100*0.1) = 10 + 24 + 6 + 10 = 50
    let expected = 50.0;
    assert!((metrics.complexity_score() - expected).abs() < 0.001);
}

#[test]
fn test_complexity_metrics_default() {
    let default = ComplexityMetrics::default();
    assert_eq!(default.cyclomatic, 0);
    assert_eq!(default.cognitive, 0);
    assert_eq!(default.nesting_max, 0);
    assert_eq!(default.lines, 0);
    assert!(default.halstead.is_none());
}

// ===================
// HalsteadMetrics Tests
// ===================

#[test]
fn test_halstead_metrics_new() {
    let metrics = HalsteadMetrics::new(8, 6, 20, 15);
    assert_eq!(metrics.operators_unique, 8);
    assert_eq!(metrics.operands_unique, 6);
    assert_eq!(metrics.operators_total, 20);
    assert_eq!(metrics.operands_total, 15);
    assert_eq!(metrics.volume, 0.0);
    assert_eq!(metrics.difficulty, 0.0);
}

#[test]
fn test_halstead_metrics_calculate_derived() {
    let base = HalsteadMetrics::new(10, 8, 25, 20);
    let calculated = base.calculate_derived();

    assert!(calculated.volume > 0.0);
    assert!(calculated.difficulty > 0.0);
    assert!(calculated.effort > 0.0);
    assert!(calculated.time > 0.0);
    assert!(calculated.bugs >= 0.0);
}

#[test]
fn test_halstead_metrics_calculate_derived_zero_operators() {
    let base = HalsteadMetrics::new(0, 5, 10, 8);
    let calculated = base.calculate_derived();

    // Should return early without calculating
    assert_eq!(calculated.volume, 0.0);
    assert_eq!(calculated.difficulty, 0.0);
}

#[test]
fn test_halstead_metrics_calculate_derived_zero_operands() {
    let base = HalsteadMetrics::new(5, 0, 10, 8);
    let calculated = base.calculate_derived();

    // Should return early without calculating
    assert_eq!(calculated.volume, 0.0);
}

#[test]
fn test_halstead_metrics_default() {
    let default = HalsteadMetrics::default();
    assert_eq!(default.operators_unique, 0);
    assert_eq!(default.operands_unique, 0);
    assert_eq!(default.volume, 0.0);
}

// ===================
// ComplexityThresholds Tests
// ===================

#[test]
fn test_complexity_thresholds_default() {
    let thresholds = ComplexityThresholds::default();
    assert_eq!(thresholds.cyclomatic_warn, 10);
    assert_eq!(thresholds.cyclomatic_error, 20);
    assert_eq!(thresholds.cognitive_warn, 15);
    assert_eq!(thresholds.cognitive_error, 30);
    assert_eq!(thresholds.nesting_max, 5);
    assert_eq!(thresholds.method_length, 50);
}

// ===================
// ComplexityVisitor Tests
// ===================

#[test]
fn test_complexity_visitor_new() {
    let mut metrics = ComplexityMetrics::default();
    let visitor = ComplexityVisitor::new(&mut metrics);

    assert_eq!(visitor.nesting_level, 0);
    assert!(visitor.current_function.is_none());
    assert!(visitor.functions.is_empty());
    assert!(visitor.classes.is_empty());
}

#[test]
fn test_complexity_visitor_calculate_cognitive_increment_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    // At nesting level 0, nesting constructs add 1
    assert_eq!(visitor.calculate_cognitive_increment(true), 1);

    // At nesting level 1, nesting constructs add 1
    visitor.nesting_level = 1;
    assert_eq!(visitor.calculate_cognitive_increment(true), 1);

    // At nesting level 2, nesting constructs add 1 + (2-1) = 2
    visitor.nesting_level = 2;
    assert_eq!(visitor.calculate_cognitive_increment(true), 2);

    // At nesting level 3, nesting constructs add 1 + (3-1) = 3
    visitor.nesting_level = 3;
    assert_eq!(visitor.calculate_cognitive_increment(true), 3);
}

#[test]
fn test_complexity_visitor_calculate_cognitive_increment_non_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    // Non-nesting constructs always add 1
    visitor.nesting_level = 0;
    assert_eq!(visitor.calculate_cognitive_increment(false), 1);

    visitor.nesting_level = 5;
    assert_eq!(visitor.calculate_cognitive_increment(false), 1);
}

#[test]
fn test_complexity_visitor_enter_exit_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    assert_eq!(visitor.nesting_level, 0);
    assert_eq!(visitor.complexity.nesting_max, 0);

    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, 1);
    assert_eq!(visitor.complexity.nesting_max, 1);

    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, 2);
    assert_eq!(visitor.complexity.nesting_max, 2);

    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 1);
    assert_eq!(visitor.complexity.nesting_max, 2); // Max preserved

    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 0);
}

#[test]
fn test_complexity_visitor_exit_nesting_saturating() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    // Exit at 0 should stay at 0 (saturating sub)
    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 0);
}

#[test]
fn test_complexity_visitor_enter_nesting_saturating() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    // Set to max value
    visitor.nesting_level = u8::MAX;
    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, u8::MAX); // Saturating add
}

// ===================
// compute_complexity_cache_key Tests
// ===================

#[test]
fn test_compute_complexity_cache_key() {
    let path = Path::new("src/main.rs");
    let content = b"fn main() { println!(\"Hello\"); }";

    let key = compute_complexity_cache_key(path, content);
    assert!(key.starts_with("cx:"));
    assert!(key.len() > 3);
}

#[test]
fn test_compute_complexity_cache_key_different_content() {
    let path = Path::new("src/main.rs");
    let content1 = b"fn main() {}";
    let content2 = b"fn main() { todo!(); }";

    let key1 = compute_complexity_cache_key(path, content1);
    let key2 = compute_complexity_cache_key(path, content2);

    assert_ne!(key1, key2);
}

#[test]
fn test_compute_complexity_cache_key_different_path() {
    let path1 = Path::new("src/main.rs");
    let path2 = Path::new("src/lib.rs");
    let content = b"fn test() {}";

    let key1 = compute_complexity_cache_key(path1, content);
    let key2 = compute_complexity_cache_key(path2, content);

    assert_ne!(key1, key2);
}

// ===================
// CyclomaticComplexityRule Tests
// ===================

#[test]
fn test_cyclomatic_complexity_rule_new() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);

    assert_eq!(rule.warn_threshold, 10);
    assert_eq!(rule.error_threshold, 20);
}

#[test]
fn test_cyclomatic_complexity_rule_evaluate_ok() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 5, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 1, Some("test_fn"));
    assert!(violation.is_none());
}

#[test]
fn test_cyclomatic_complexity_rule_evaluate_warning() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(15, 5, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 10, Some("complex_fn"));
    assert!(violation.is_some());

    if let Some(Violation::Warning {
        rule: r,
        value,
        threshold,
        ..
    }) = violation
    {
        assert_eq!(r, "cyclomatic-complexity");
        assert_eq!(value, 15);
        assert_eq!(threshold, 10);
    } else {
        panic!("Expected Warning violation");
    }
}

#[test]
fn test_cyclomatic_complexity_rule_evaluate_error() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(25, 5, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 10, None);
    assert!(violation.is_some());

    if let Some(Violation::Error {
        rule: r,
        value,
        threshold,
        function,
        ..
    }) = violation
    {
        assert_eq!(r, "cyclomatic-complexity");
        assert_eq!(value, 25);
        assert_eq!(threshold, 20);
        assert!(function.is_none());
    } else {
        panic!("Expected Error violation");
    }
}

// ===================
// CognitiveComplexityRule Tests
// ===================

#[test]
fn test_cognitive_complexity_rule_new() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);

    assert_eq!(rule.warn_threshold, 15);
    assert_eq!(rule.error_threshold, 30);
}

#[test]
fn test_cognitive_complexity_rule_evaluate_ok() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 10, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 1, Some("simple_fn"));
    assert!(violation.is_none());
}

#[test]
fn test_cognitive_complexity_rule_evaluate_warning() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 20, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 10, Some("complex_fn"));
    assert!(violation.is_some());

    if let Some(Violation::Warning {
        rule: r,
        value,
        threshold,
        ..
    }) = violation
    {
        assert_eq!(r, "cognitive-complexity");
        assert_eq!(value, 20);
        assert_eq!(threshold, 15);
    } else {
        panic!("Expected Warning violation");
    }
}

#[test]
fn test_cognitive_complexity_rule_evaluate_error() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 35, 2, 20);

    let violation = rule.evaluate(&metrics, "test.rs", 10, None);
    assert!(violation.is_some());

    if let Some(Violation::Error {
        rule: r,
        value,
        threshold,
        ..
    }) = violation
    {
        assert_eq!(r, "cognitive-complexity");
        assert_eq!(value, 35);
        assert_eq!(threshold, 30);
    } else {
        panic!("Expected Error violation");
    }
}

// ===================
// Struct Construction Tests
// ===================

#[test]
fn test_function_complexity_struct() {
    let fc = FunctionComplexity {
        name: "test_fn".to_string(),
        line_start: 10,
        line_end: 25,
        metrics: ComplexityMetrics::new(3, 4, 2, 15),
    };

    assert_eq!(fc.name, "test_fn");
    assert_eq!(fc.line_start, 10);
    assert_eq!(fc.line_end, 25);
    assert_eq!(fc.metrics.cyclomatic, 3);
}

#[test]
fn test_class_complexity_struct() {
    let cc = ClassComplexity {
        name: "TestClass".to_string(),
        line_start: 1,
        line_end: 100,
        metrics: ComplexityMetrics::new(10, 15, 4, 80),
        methods: vec![],
    };

    assert_eq!(cc.name, "TestClass");
    assert!(cc.methods.is_empty());
}

#[test]
fn test_file_complexity_metrics_struct() {
    let fcm = FileComplexityMetrics {
        path: "src/test.rs".to_string(),
        total_complexity: ComplexityMetrics::new(5, 8, 3, 50),
        functions: vec![],
        classes: vec![],
    };

    assert_eq!(fcm.path, "src/test.rs");
    assert_eq!(fcm.total_complexity.cyclomatic, 5);
}

#[test]
fn test_complexity_hotspot_struct() {
    let hotspot = ComplexityHotspot {
        file: "src/complex.rs".to_string(),
        function: Some("complex_fn".to_string()),
        line: 42,
        complexity: 25,
        complexity_type: "cyclomatic".to_string(),
    };

    assert_eq!(hotspot.file, "src/complex.rs");
    assert_eq!(hotspot.complexity, 25);
}

#[test]
fn test_complexity_summary_default() {
    let summary = ComplexitySummary::default();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_functions, 0);
    assert_eq!(summary.median_cyclomatic, 0.0);
}

#[test]
fn test_complexity_report_struct() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };

    assert!(report.violations.is_empty());
    assert!(report.hotspots.is_empty());
}

// ===================
// format_as_sarif Tests
// ===================

#[test]
fn test_format_as_sarif_empty_report() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };

    let sarif = format_as_sarif(&report).unwrap();
    assert!(sarif.contains("\"version\": \"2.1.0\""));
    assert!(sarif.contains("pmat"));
}

#[test]
fn test_format_as_sarif_with_warning() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![Violation::Warning {
            rule: "cyclomatic-complexity".to_string(),
            message: "Too complex".to_string(),
            value: 15,
            threshold: 10,
            file: "test.rs".to_string(),
            line: 42,
            function: Some("complex_fn".to_string()),
        }],
        hotspots: vec![],
        files: vec![],
    };

    let sarif = format_as_sarif(&report).unwrap();
    assert!(sarif.contains("cyclomatic-complexity"));
    assert!(sarif.contains("warning"));
    assert!(sarif.contains("test.rs"));
}

#[test]
fn test_format_as_sarif_with_error() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![Violation::Error {
            rule: "cognitive-complexity".to_string(),
            message: "Very complex".to_string(),
            value: 35,
            threshold: 30,
            file: "complex.rs".to_string(),
            line: 100,
            function: None,
        }],
        hotspots: vec![],
        files: vec![],
    };

    let sarif = format_as_sarif(&report).unwrap();
    assert!(sarif.contains("cognitive-complexity"));
    assert!(sarif.contains("error"));
    assert!(sarif.contains("complex.rs"));
}

// ===================
// technical_debt_hours sign (#719)
// ===================

/// A project with no complexity violations has *zero* debt, not negative debt.
///
/// `Iterator::sum::<f32>()` seeds with `-0.0`, so the previous implementation
/// serialized `"technical_debt_hours": -0.0` for every clean project — including
/// an empty directory. This asserts the sign, which is what the JSON consumer
/// and the `{:.1} hours` renderer actually see.
#[test]
fn test_technical_debt_is_positive_zero_when_there_are_no_violations() {
    let debt = analysis::calculate_technical_debt(&[]);

    assert_eq!(debt, 0.0);
    assert!(
        !debt.is_sign_negative(),
        "no violations must yield +0.0, got {debt:?} (renders as -0.0 in JSON)"
    );
    assert_eq!(format!("{:?}", f64::from(debt)), "0.0");
}

/// The seed change must not perturb a real, non-empty sum.
#[test]
fn test_technical_debt_still_sums_violations() {
    let violations = vec![
        Violation::Error {
            rule: "cyclomatic-complexity".to_string(),
            message: "too complex".to_string(),
            value: 30,
            threshold: 20,
            file: "a.rs".to_string(),
            line: 1,
            function: None,
        },
        Violation::Warning {
            rule: "cognitive-complexity".to_string(),
            message: "getting complex".to_string(),
            value: 14,
            threshold: 10,
            file: "b.rs".to_string(),
            line: 2,
            function: None,
        },
    ];

    // (30-20)*30 + (14-10)*15 = 300 + 60 = 360 minutes = 6 hours
    assert!((analysis::calculate_technical_debt(&violations) - 6.0).abs() < f32::EPSILON);
}
