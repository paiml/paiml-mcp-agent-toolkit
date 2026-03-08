#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for complexity module
//! Comprehensive tests for ComplexityMetrics, HalsteadMetrics, rules, and aggregation

use crate::services::complexity::{
    aggregate_results, aggregate_results_with_thresholds, compute_complexity_cache_key,
    format_as_sarif, format_complexity_report, format_complexity_summary, ClassComplexity,
    CognitiveComplexityRule, ComplexityHotspot, ComplexityMetrics, ComplexityReport,
    ComplexityRule, ComplexitySummary, ComplexityThresholds, ComplexityVisitor,
    CyclomaticComplexityRule, FileComplexityMetrics, FunctionComplexity, HalsteadMetrics,
    Violation,
};
use std::path::Path;

// ============ ComplexityMetrics Tests ============

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
    assert!(metrics.halstead.is_some());
    let h = metrics.halstead.unwrap();
    assert_eq!(h.operators_unique, 10);
    assert_eq!(h.operands_unique, 5);
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
    let complex = ComplexityMetrics::new(2, 8, 1, 10);
    assert!(!complex.is_simple());
}

#[test]
fn test_complexity_metrics_needs_refactoring_false() {
    let simple = ComplexityMetrics::new(3, 4, 2, 15);
    assert!(!simple.needs_refactoring());
}

#[test]
fn test_complexity_metrics_needs_refactoring_cyclomatic() {
    let complex = ComplexityMetrics::new(11, 4, 2, 15);
    assert!(complex.needs_refactoring());
}

#[test]
fn test_complexity_metrics_needs_refactoring_cognitive() {
    let complex = ComplexityMetrics::new(3, 16, 2, 15);
    assert!(complex.needs_refactoring());
}

#[test]
fn test_complexity_metrics_complexity_score() {
    let simple = ComplexityMetrics::new(1, 1, 1, 5);
    let complex = ComplexityMetrics::new(10, 15, 4, 80);
    assert!(complex.complexity_score() > simple.complexity_score());
}

#[test]
fn test_complexity_metrics_score_calculation() {
    let metrics = ComplexityMetrics::new(5, 10, 3, 100);
    // score = cyclomatic*1.0 + cognitive*1.2 + nesting*2.0 + lines*0.1
    // score = 5*1.0 + 10*1.2 + 3*2.0 + 100*0.1 = 5 + 12 + 6 + 10 = 33
    let score = metrics.complexity_score();
    assert!((score - 33.0).abs() < 0.01);
}

#[test]
fn test_complexity_metrics_default() {
    let metrics = ComplexityMetrics::default();
    assert_eq!(metrics.cyclomatic, 0);
    assert_eq!(metrics.cognitive, 0);
    assert_eq!(metrics.nesting_max, 0);
    assert_eq!(metrics.lines, 0);
    assert!(metrics.halstead.is_none());
}

#[test]
fn test_complexity_metrics_clone() {
    let metrics = ComplexityMetrics::new(5, 8, 3, 42);
    let cloned = metrics;
    assert_eq!(metrics.cyclomatic, cloned.cyclomatic);
}

// ============ HalsteadMetrics Tests ============

#[test]
fn test_halstead_metrics_new() {
    let metrics = HalsteadMetrics::new(8, 6, 20, 15);
    assert_eq!(metrics.operators_unique, 8);
    assert_eq!(metrics.operands_unique, 6);
    assert_eq!(metrics.operators_total, 20);
    assert_eq!(metrics.operands_total, 15);
    assert_eq!(metrics.volume, 0.0);
    assert_eq!(metrics.difficulty, 0.0);
    assert_eq!(metrics.effort, 0.0);
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
fn test_halstead_metrics_zero_operators() {
    let base = HalsteadMetrics::new(0, 8, 25, 20);
    let calculated = base.calculate_derived();
    // Should return self without calculation when operators_unique is 0
    assert_eq!(calculated.volume, 0.0);
}

#[test]
fn test_halstead_metrics_zero_operands() {
    let base = HalsteadMetrics::new(10, 0, 25, 20);
    let calculated = base.calculate_derived();
    // Should return self without calculation when operands_unique is 0
    assert_eq!(calculated.volume, 0.0);
}

#[test]
fn test_halstead_metrics_default() {
    let metrics = HalsteadMetrics::default();
    assert_eq!(metrics.operators_unique, 0);
    assert_eq!(metrics.operands_unique, 0);
    assert_eq!(metrics.volume, 0.0);
}

#[test]
fn test_halstead_metrics_clone() {
    let metrics = HalsteadMetrics::new(8, 6, 20, 15);
    let cloned = metrics;
    assert_eq!(metrics.operators_unique, cloned.operators_unique);
}

// ============ ComplexityThresholds Tests ============

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

#[test]
fn test_complexity_thresholds_clone() {
    let thresholds = ComplexityThresholds::default();
    let cloned = thresholds.clone();
    assert_eq!(thresholds.cyclomatic_warn, cloned.cyclomatic_warn);
}

// ============ Violation Tests ============

#[test]
fn test_violation_error_creation() {
    let violation = Violation::Error {
        rule: "cyclomatic-complexity".to_string(),
        message: "Too complex".to_string(),
        value: 25,
        threshold: 20,
        file: "test.rs".to_string(),
        line: 10,
        function: Some("process".to_string()),
    };
    if let Violation::Error {
        value, threshold, ..
    } = violation
    {
        assert_eq!(value, 25);
        assert_eq!(threshold, 20);
    }
}

#[test]
fn test_violation_warning_creation() {
    let violation = Violation::Warning {
        rule: "cognitive-complexity".to_string(),
        message: "Consider refactoring".to_string(),
        value: 12,
        threshold: 10,
        file: "test.rs".to_string(),
        line: 5,
        function: None,
    };
    if let Violation::Warning {
        value, function, ..
    } = violation
    {
        assert_eq!(value, 12);
        assert!(function.is_none());
    }
}

// ============ ComplexitySummary Tests ============

#[test]
fn test_complexity_summary_default() {
    let summary = ComplexitySummary::default();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_functions, 0);
    assert_eq!(summary.median_cyclomatic, 0.0);
    assert_eq!(summary.max_cyclomatic, 0);
    assert_eq!(summary.technical_debt_hours, 0.0);
}

#[test]
fn test_complexity_summary_clone() {
    let summary = ComplexitySummary {
        total_files: 5,
        total_functions: 20,
        median_cyclomatic: 4.0,
        median_cognitive: 5.0,
        max_cyclomatic: 15,
        max_cognitive: 20,
        p90_cyclomatic: 10,
        p90_cognitive: 12,
        technical_debt_hours: 2.5,
    };
    let cloned = summary.clone();
    assert_eq!(summary.total_files, cloned.total_files);
}

// ============ ComplexityHotspot Tests ============

#[test]
fn test_complexity_hotspot_creation() {
    let hotspot = ComplexityHotspot {
        file: "src/main.rs".to_string(),
        function: Some("process_data".to_string()),
        line: 42,
        complexity: 25,
        complexity_type: "cyclomatic".to_string(),
    };
    assert_eq!(hotspot.file, "src/main.rs");
    assert_eq!(hotspot.complexity, 25);
}

#[test]
fn test_complexity_hotspot_clone() {
    let hotspot = ComplexityHotspot {
        file: "src/main.rs".to_string(),
        function: None,
        line: 10,
        complexity: 15,
        complexity_type: "cognitive".to_string(),
    };
    let cloned = hotspot.clone();
    assert_eq!(hotspot.line, cloned.line);
}

// ============ ComplexityVisitor Tests ============

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
fn test_complexity_visitor_cognitive_increment_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);
    visitor.nesting_level = 3;
    // is_nesting_construct = true: 1 + (nesting_level - 1) = 1 + 2 = 3
    let increment = visitor.calculate_cognitive_increment(true);
    assert_eq!(increment, 3);
}

#[test]
fn test_complexity_visitor_cognitive_increment_non_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let visitor = ComplexityVisitor::new(&mut metrics);
    // is_nesting_construct = false: always 1
    let increment = visitor.calculate_cognitive_increment(false);
    assert_eq!(increment, 1);
}

#[test]
fn test_complexity_visitor_enter_exit_nesting() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, 1);
    assert_eq!(visitor.complexity.nesting_max, 1);

    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, 2);
    assert_eq!(visitor.complexity.nesting_max, 2);

    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 1);
    assert_eq!(visitor.complexity.nesting_max, 2); // max unchanged

    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 0);
}

#[test]
fn test_complexity_visitor_nesting_saturating() {
    let mut metrics = ComplexityMetrics::default();
    let mut visitor = ComplexityVisitor::new(&mut metrics);

    // Should not underflow
    visitor.exit_nesting();
    assert_eq!(visitor.nesting_level, 0);

    // Saturating add should handle high values
    visitor.nesting_level = 255;
    visitor.enter_nesting();
    assert_eq!(visitor.nesting_level, 255); // saturated
}

// ============ Cache Key Tests ============

#[test]
fn test_compute_complexity_cache_key() {
    let path = Path::new("src/main.rs");
    let content = b"fn main() { println!(\"Hello\"); }";
    let key = compute_complexity_cache_key(path, content);
    assert!(key.starts_with("cx:"));
    assert!(key.len() > 10);
}

#[test]
fn test_compute_complexity_cache_key_different_content() {
    let path = Path::new("src/main.rs");
    let content1 = b"fn main() { }";
    let content2 = b"fn main() { println!(); }";
    let key1 = compute_complexity_cache_key(path, content1);
    let key2 = compute_complexity_cache_key(path, content2);
    assert_ne!(key1, key2);
}

#[test]
fn test_compute_complexity_cache_key_different_paths() {
    let content = b"fn main() { }";
    let key1 = compute_complexity_cache_key(Path::new("src/a.rs"), content);
    let key2 = compute_complexity_cache_key(Path::new("src/b.rs"), content);
    assert_ne!(key1, key2);
}

// ============ ComplexityRule Tests ============

#[test]
fn test_cyclomatic_rule_no_violation() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 5, 2, 30);
    let result = rule.evaluate(&metrics, "test.rs", 10, Some("fn test"));
    assert!(result.is_none());
}

#[test]
fn test_cyclomatic_rule_warning() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(15, 5, 2, 30); // > 10 (warn), < 20 (error)
    let result = rule.evaluate(&metrics, "test.rs", 10, Some("fn test"));
    assert!(matches!(result, Some(Violation::Warning { .. })));
}

#[test]
fn test_cyclomatic_rule_error() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(25, 5, 2, 30); // > 20 (error)
    let result = rule.evaluate(&metrics, "test.rs", 10, Some("fn test"));
    assert!(matches!(result, Some(Violation::Error { .. })));
}

#[test]
fn test_cognitive_rule_no_violation() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 10, 2, 30); // < 15 (warn)
    let result = rule.evaluate(&metrics, "test.rs", 10, Some("fn test"));
    assert!(result.is_none());
}

#[test]
fn test_cognitive_rule_warning() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 20, 2, 30); // > 15 (warn), < 30 (error)
    let result = rule.evaluate(&metrics, "test.rs", 10, Some("fn test"));
    assert!(matches!(result, Some(Violation::Warning { .. })));
}

#[test]
fn test_cognitive_rule_error() {
    let thresholds = ComplexityThresholds::default();
    let rule = CognitiveComplexityRule::new(&thresholds);
    let metrics = ComplexityMetrics::new(5, 35, 2, 30); // > 30 (error)
    let result = rule.evaluate(&metrics, "test.rs", 10, None);
    assert!(matches!(result, Some(Violation::Error { .. })));
}

#[test]
fn test_exceeds_threshold() {
    let thresholds = ComplexityThresholds::default();
    let rule = CyclomaticComplexityRule::new(&thresholds);
    assert!(!rule.exceeds_threshold(10, 10));
    assert!(rule.exceeds_threshold(11, 10));
}

// ============ Aggregate Results Tests ============

#[test]
fn test_aggregate_results_empty() {
    let report = aggregate_results(vec![]);
    assert_eq!(report.summary.total_files, 0);
    assert_eq!(report.summary.total_functions, 0);
    assert!(report.violations.is_empty());
    assert!(report.hotspots.is_empty());
}

#[test]
fn test_aggregate_results_single_file() {
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(10, 8, 3, 50),
        functions: vec![FunctionComplexity {
            name: "process".to_string(),
            line_start: 10,
            line_end: 60,
            metrics: ComplexityMetrics::new(10, 8, 3, 50),
        }],
        classes: vec![],
    };
    let report = aggregate_results(vec![file]);
    assert_eq!(report.summary.total_files, 1);
    assert_eq!(report.summary.total_functions, 1);
}

#[test]
fn test_aggregate_results_with_violations() {
    let func = FunctionComplexity {
        name: "complex_function".to_string(),
        line_start: 10,
        line_end: 50,
        metrics: ComplexityMetrics::new(25, 35, 5, 100),
    };
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(25, 35, 5, 100),
        functions: vec![func],
        classes: vec![],
    };
    let report = aggregate_results(vec![file]);
    assert!(!report.violations.is_empty()); // Both cyclomatic and cognitive violations
}

#[test]
fn test_aggregate_results_with_custom_thresholds() {
    let func = FunctionComplexity {
        name: "moderate_function".to_string(),
        line_start: 10,
        line_end: 50,
        metrics: ComplexityMetrics::new(15, 18, 3, 50),
    };
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(15, 18, 3, 50),
        functions: vec![func],
        classes: vec![],
    };

    // With default thresholds (warn=10, error=20): cyclomatic 15 -> warning
    let report1 = aggregate_results(vec![file.clone()]);
    let warnings1 = report1
        .violations
        .iter()
        .filter(|v| matches!(v, Violation::Warning { .. }))
        .count();
    assert!(warnings1 > 0);

    // With custom threshold of 25: no violations
    let report2 = aggregate_results_with_thresholds(vec![file], Some(25), Some(25));
    assert!(report2.violations.is_empty());
}

#[test]
fn test_aggregate_results_with_classes() {
    let method = FunctionComplexity {
        name: "calculate".to_string(),
        line_start: 15,
        line_end: 30,
        metrics: ComplexityMetrics::new(8, 10, 2, 15),
    };
    let class = ClassComplexity {
        name: "Calculator".to_string(),
        line_start: 10,
        line_end: 50,
        metrics: ComplexityMetrics::new(8, 10, 2, 40),
        methods: vec![method],
    };
    let file = FileComplexityMetrics {
        path: "src/calc.rs".to_string(),
        total_complexity: ComplexityMetrics::new(8, 10, 2, 40),
        functions: vec![],
        classes: vec![class],
    };
    let report = aggregate_results(vec![file]);
    assert_eq!(report.summary.total_functions, 1); // Method counted as function
}

#[test]
fn test_aggregate_results_hotspots() {
    let func = FunctionComplexity {
        name: "complex".to_string(),
        line_start: 10,
        line_end: 50,
        metrics: ComplexityMetrics::new(15, 10, 3, 40), // > 10 warn threshold
    };
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(15, 10, 3, 40),
        functions: vec![func],
        classes: vec![],
    };
    let report = aggregate_results(vec![file]);
    assert!(!report.hotspots.is_empty());
    assert_eq!(report.hotspots[0].function, Some("complex".to_string()));
}

#[test]
fn test_aggregate_results_statistics() {
    let funcs = vec![
        FunctionComplexity {
            name: "func1".to_string(),
            line_start: 10,
            line_end: 20,
            metrics: ComplexityMetrics::new(5, 4, 2, 10),
        },
        FunctionComplexity {
            name: "func2".to_string(),
            line_start: 25,
            line_end: 40,
            metrics: ComplexityMetrics::new(8, 6, 3, 15),
        },
        FunctionComplexity {
            name: "func3".to_string(),
            line_start: 45,
            line_end: 60,
            metrics: ComplexityMetrics::new(3, 2, 1, 15),
        },
    ];
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(16, 12, 3, 40),
        functions: funcs,
        classes: vec![],
    };
    let report = aggregate_results(vec![file]);
    assert_eq!(report.summary.total_functions, 3);
    assert_eq!(report.summary.max_cyclomatic, 8);
    assert_eq!(report.summary.max_cognitive, 6);
    // Median of [3, 5, 8] = 5
    assert!((report.summary.median_cyclomatic - 5.0).abs() < 0.01);
}

// ============ Format Functions Tests ============

/// Strip ANSI escape codes from a string for assertion comparisons.
fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

#[test]
fn test_format_complexity_summary_basic() {
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 2,
            total_functions: 10,
            median_cyclomatic: 5.0,
            median_cognitive: 6.0,
            max_cyclomatic: 15,
            max_cognitive: 20,
            p90_cyclomatic: 12,
            p90_cognitive: 15,
            technical_debt_hours: 0.0,
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };
    let summary = strip_ansi(&format_complexity_summary(&report));
    assert!(summary.contains("Complexity Analysis Summary"));
    assert!(summary.contains("Files analyzed:"));
    assert!(summary.contains("2"));
    assert!(summary.contains("Total functions:"));
    assert!(summary.contains("10"));
}

#[test]
fn test_format_complexity_summary_with_debt() {
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 1,
            total_functions: 5,
            median_cyclomatic: 8.0,
            median_cognitive: 10.0,
            max_cyclomatic: 25,
            max_cognitive: 30,
            p90_cyclomatic: 20,
            p90_cognitive: 25,
            technical_debt_hours: 5.5,
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };
    let summary = format_complexity_summary(&report);
    assert!(summary.contains("Estimated Refactoring Time"));
}

#[test]
fn test_format_complexity_summary_with_violations() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![
            Violation::Error {
                rule: "cyclomatic".to_string(),
                message: "Too complex".to_string(),
                value: 25,
                threshold: 20,
                file: "test.rs".to_string(),
                line: 10,
                function: None,
            },
            Violation::Warning {
                rule: "cognitive".to_string(),
                message: "Consider simplifying".to_string(),
                value: 18,
                threshold: 15,
                file: "test.rs".to_string(),
                line: 20,
                function: None,
            },
        ],
        hotspots: vec![],
        files: vec![],
    };
    let summary = format_complexity_summary(&report);
    assert!(summary.contains("Issues Found"));
    assert!(summary.contains("Errors"));
    assert!(summary.contains("Warnings"));
}

#[test]
fn test_format_complexity_summary_with_files() {
    let file1 = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(15, 20, 4, 100),
        functions: vec![],
        classes: vec![],
    };
    let file2 = FileComplexityMetrics {
        path: "src/lib.rs".to_string(),
        total_complexity: ComplexityMetrics::new(5, 8, 2, 50),
        functions: vec![],
        classes: vec![],
    };
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 2,
            ..Default::default()
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![file1, file2],
    };
    let summary = format_complexity_summary(&report);
    assert!(summary.contains("Top Files by Complexity"));
    assert!(summary.contains("main.rs"));
    assert!(summary.contains("lib.rs"));
}

#[test]
fn test_format_complexity_summary_single_file_with_functions() {
    let funcs = vec![
        FunctionComplexity {
            name: "complex_func".to_string(),
            line_start: 10,
            line_end: 50,
            metrics: ComplexityMetrics::new(15, 20, 4, 40),
        },
        FunctionComplexity {
            name: "simple_func".to_string(),
            line_start: 55,
            line_end: 65,
            metrics: ComplexityMetrics::new(2, 3, 1, 10),
        },
    ];
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(17, 23, 4, 50),
        functions: funcs,
        classes: vec![],
    };
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 1,
            ..Default::default()
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![file],
    };
    let summary = format_complexity_summary(&report);
    assert!(summary.contains("Functions in File"));
    assert!(summary.contains("complex_func"));
    assert!(summary.contains("simple_func"));
}

#[test]
fn test_format_complexity_summary_with_hotspots() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![],
        hotspots: vec![
            ComplexityHotspot {
                file: "src/main.rs".to_string(),
                function: Some("process_data".to_string()),
                line: 42,
                complexity: 25,
                complexity_type: "cyclomatic".to_string(),
            },
            ComplexityHotspot {
                file: "src/lib.rs".to_string(),
                function: None,
                line: 100,
                complexity: 20,
                complexity_type: "cognitive".to_string(),
            },
        ],
        files: vec![],
    };
    let summary = format_complexity_summary(&report);
    assert!(summary.contains("Top Complexity Hotspots"));
    assert!(summary.contains("process_data"));
}

#[test]
fn test_format_complexity_report() {
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 1,
            total_functions: 2,
            ..Default::default()
        },
        violations: vec![Violation::Error {
            rule: "cyclomatic-complexity".to_string(),
            message: "Too complex".to_string(),
            value: 25,
            threshold: 20,
            file: "src/main.rs".to_string(),
            line: 10,
            function: Some("process".to_string()),
        }],
        hotspots: vec![],
        files: vec![],
    };
    let full_report = format_complexity_report(&report);
    assert!(full_report.contains("Detailed Violations"));
    assert!(full_report.contains("src/main.rs"));
    assert!(full_report.contains("process"));
}

#[test]
fn test_format_as_sarif_empty() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };
    let sarif = format_as_sarif(&report).unwrap();
    assert!(sarif.contains("\"version\": \"2.1.0\""));
    assert!(sarif.contains("cyclomatic-complexity"));
    assert!(sarif.contains("cognitive-complexity"));
}

#[test]
fn test_format_as_sarif_with_violations() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![
            Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: "Complexity too high".to_string(),
                value: 25,
                threshold: 20,
                file: "src/main.rs".to_string(),
                line: 10,
                function: Some("process".to_string()),
            },
            Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: "Consider simplifying".to_string(),
                value: 18,
                threshold: 15,
                file: "src/lib.rs".to_string(),
                line: 50,
                function: None,
            },
        ],
        hotspots: vec![],
        files: vec![],
    };
    let sarif = format_as_sarif(&report).unwrap();
    assert!(sarif.contains("\"level\": \"error\""));
    assert!(sarif.contains("\"level\": \"warning\""));
    assert!(sarif.contains("src/main.rs"));
    assert!(sarif.contains("src/lib.rs"));
}

// ============ FileComplexityMetrics Tests ============

#[test]
fn test_file_complexity_metrics_creation() {
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(10, 15, 3, 100),
        functions: vec![],
        classes: vec![],
    };
    assert_eq!(file.path, "src/main.rs");
    assert_eq!(file.total_complexity.cyclomatic, 10);
}

#[test]
fn test_file_complexity_metrics_clone() {
    let file = FileComplexityMetrics {
        path: "src/main.rs".to_string(),
        total_complexity: ComplexityMetrics::new(10, 15, 3, 100),
        functions: vec![FunctionComplexity {
            name: "test".to_string(),
            line_start: 1,
            line_end: 10,
            metrics: ComplexityMetrics::new(5, 5, 2, 10),
        }],
        classes: vec![],
    };
    let cloned = file.clone();
    assert_eq!(file.path, cloned.path);
    assert_eq!(file.functions.len(), cloned.functions.len());
}

// ============ FunctionComplexity Tests ============

#[test]
fn test_function_complexity_creation() {
    let func = FunctionComplexity {
        name: "process_data".to_string(),
        line_start: 10,
        line_end: 50,
        metrics: ComplexityMetrics::new(8, 10, 3, 40),
    };
    assert_eq!(func.name, "process_data");
    assert_eq!(func.line_start, 10);
    assert_eq!(func.line_end, 50);
}

#[test]
fn test_function_complexity_clone() {
    let func = FunctionComplexity {
        name: "test".to_string(),
        line_start: 1,
        line_end: 10,
        metrics: ComplexityMetrics::new(5, 5, 2, 10),
    };
    let cloned = func.clone();
    assert_eq!(func.name, cloned.name);
}

// ============ ClassComplexity Tests ============

#[test]
fn test_class_complexity_creation() {
    let class = ClassComplexity {
        name: "Calculator".to_string(),
        line_start: 1,
        line_end: 100,
        metrics: ComplexityMetrics::new(20, 25, 4, 100),
        methods: vec![
            FunctionComplexity {
                name: "add".to_string(),
                line_start: 10,
                line_end: 20,
                metrics: ComplexityMetrics::new(2, 2, 1, 10),
            },
            FunctionComplexity {
                name: "complex_calc".to_string(),
                line_start: 25,
                line_end: 80,
                metrics: ComplexityMetrics::new(18, 23, 4, 55),
            },
        ],
    };
    assert_eq!(class.name, "Calculator");
    assert_eq!(class.methods.len(), 2);
}

#[test]
fn test_class_complexity_clone() {
    let class = ClassComplexity {
        name: "Test".to_string(),
        line_start: 1,
        line_end: 50,
        metrics: ComplexityMetrics::default(),
        methods: vec![],
    };
    let cloned = class.clone();
    assert_eq!(class.name, cloned.name);
}

// ============ ComplexityReport Tests ============

#[test]
fn test_complexity_report_creation() {
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 5,
            total_functions: 50,
            median_cyclomatic: 4.0,
            median_cognitive: 5.0,
            max_cyclomatic: 25,
            max_cognitive: 30,
            p90_cyclomatic: 15,
            p90_cognitive: 20,
            technical_debt_hours: 10.0,
        },
        violations: vec![],
        hotspots: vec![],
        files: vec![],
    };
    assert_eq!(report.summary.total_files, 5);
    assert_eq!(report.summary.technical_debt_hours, 10.0);
}

#[test]
fn test_complexity_report_clone() {
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: vec![Violation::Warning {
            rule: "test".to_string(),
            message: "test".to_string(),
            value: 10,
            threshold: 5,
            file: "test.rs".to_string(),
            line: 1,
            function: None,
        }],
        hotspots: vec![],
        files: vec![],
    };
    let cloned = report.clone();
    assert_eq!(report.violations.len(), cloned.violations.len());
}

// ============ Edge Cases and Boundary Tests ============

#[test]
fn test_complexity_metrics_boundary_simple() {
    // Exactly at the threshold
    let at_threshold = ComplexityMetrics::new(5, 7, 2, 20);
    assert!(at_threshold.is_simple());

    // Just above threshold
    let above_cyclomatic = ComplexityMetrics::new(6, 7, 2, 20);
    assert!(!above_cyclomatic.is_simple());

    let above_cognitive = ComplexityMetrics::new(5, 8, 2, 20);
    assert!(!above_cognitive.is_simple());
}

#[test]
fn test_complexity_metrics_boundary_refactoring() {
    // Exactly at threshold (10 cyclomatic, 15 cognitive)
    let at_threshold = ComplexityMetrics::new(10, 15, 3, 50);
    assert!(!at_threshold.needs_refactoring());

    // Just above threshold
    let above = ComplexityMetrics::new(11, 15, 3, 50);
    assert!(above.needs_refactoring());
}

#[test]
fn test_halstead_derived_metrics_calculation() {
    // Test specific Halstead calculations
    let h = HalsteadMetrics::new(10, 10, 50, 50).calculate_derived();

    // n = n1 + n2 = 20
    // N = N1 + N2 = 100
    // V = N * log2(n) = 100 * log2(20) ≈ 432.19
    // D = (n1/2) * (N2/n2) = 5 * 5 = 25
    // E = V * D
    assert!(h.volume > 400.0 && h.volume < 500.0);
    assert!((h.difficulty - 25.0).abs() < 0.01);
    assert!(h.effort > 10000.0);
}

#[test]
fn test_technical_debt_calculation() {
    let func = FunctionComplexity {
        name: "complex".to_string(),
        line_start: 1,
        line_end: 100,
        metrics: ComplexityMetrics::new(30, 40, 5, 100),
    };
    let file = FileComplexityMetrics {
        path: "test.rs".to_string(),
        total_complexity: func.metrics,
        functions: vec![func],
        classes: vec![],
    };
    let report = aggregate_results(vec![file]);
    // Should have some technical debt from the violations
    assert!(report.summary.technical_debt_hours > 0.0);
}

#[test]
fn test_multiple_files_aggregation() {
    let files: Vec<FileComplexityMetrics> = (0..5)
        .map(|i| FileComplexityMetrics {
            path: format!("src/file{i}.rs"),
            total_complexity: ComplexityMetrics::new(i as u16 * 2 + 3, i as u16 * 2 + 5, 2, 30),
            functions: vec![FunctionComplexity {
                name: format!("func{i}"),
                line_start: 1,
                line_end: 30,
                metrics: ComplexityMetrics::new(i as u16 * 2 + 3, i as u16 * 2 + 5, 2, 30),
            }],
            classes: vec![],
        })
        .collect();

    let report = aggregate_results(files);
    assert_eq!(report.summary.total_files, 5);
    assert_eq!(report.summary.total_functions, 5);
}
