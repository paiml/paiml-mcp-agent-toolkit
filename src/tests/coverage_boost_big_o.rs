#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/big_o_analyzer.rs and models/complexity_bound.rs
//! Tests: BigOClass, ComplexityBound, BigOAnalyzer, ComplexityDistribution,
//! FunctionComplexity, PatternMatch, BigOAnalysisConfig, BigOAnalysisReport

use crate::models::complexity_bound::{BigOClass, ComplexityBound, ComplexityFlags, InputVariable};
use crate::services::big_o_analyzer::{
    BigOAnalysisConfig, BigOAnalysisReport, BigOAnalyzer, ComplexityDistribution,
    FunctionComplexity, PatternMatch,
};
use std::path::PathBuf;

// ============ BigOClass Tests ============

#[test]
fn test_big_o_class_notation() {
    assert_eq!(BigOClass::Constant.notation(), "O(1)");
    assert_eq!(BigOClass::Logarithmic.notation(), "O(log n)");
    assert_eq!(BigOClass::Linear.notation(), "O(n)");
    assert_eq!(BigOClass::Linearithmic.notation(), "O(n log n)");
    assert_eq!(BigOClass::Quadratic.notation(), "O(n²)");
    assert_eq!(BigOClass::Cubic.notation(), "O(n³)");
    assert_eq!(BigOClass::Exponential.notation(), "O(2^n)");
    assert_eq!(BigOClass::Factorial.notation(), "O(n!)");
    assert_eq!(BigOClass::Unknown.notation(), "O(?)");
}

#[test]
fn test_big_o_class_is_better_than() {
    assert!(BigOClass::Constant.is_better_than(&BigOClass::Linear));
    assert!(BigOClass::Linear.is_better_than(&BigOClass::Quadratic));
    assert!(BigOClass::Quadratic.is_better_than(&BigOClass::Exponential));
    assert!(!BigOClass::Exponential.is_better_than(&BigOClass::Linear));
    assert!(!BigOClass::Constant.is_better_than(&BigOClass::Constant));
}

#[test]
fn test_big_o_class_growth_factor() {
    assert!((BigOClass::Constant.growth_factor(100.0) - 1.0).abs() < f64::EPSILON);
    assert!((BigOClass::Linear.growth_factor(100.0) - 100.0).abs() < f64::EPSILON);
    assert!((BigOClass::Quadratic.growth_factor(10.0) - 100.0).abs() < f64::EPSILON);
    assert!(BigOClass::Logarithmic.growth_factor(8.0) > 0.0);
    assert!(BigOClass::Cubic.growth_factor(5.0) > 0.0);
    assert!(BigOClass::Exponential.growth_factor(3.0) > 0.0);
}

#[test]
fn test_big_o_class_growth_factor_factorial() {
    // Small n uses direct calculation
    let f5 = BigOClass::Factorial.growth_factor(5.0);
    assert!((f5 - 120.0).abs() < f64::EPSILON);
}

#[test]
fn test_big_o_class_serde() {
    let classes = vec![
        BigOClass::Constant,
        BigOClass::Logarithmic,
        BigOClass::Linear,
        BigOClass::Linearithmic,
        BigOClass::Quadratic,
        BigOClass::Cubic,
        BigOClass::Exponential,
        BigOClass::Factorial,
        BigOClass::Unknown,
    ];
    for c in &classes {
        let json = serde_json::to_string(c).unwrap();
        let back: BigOClass = serde_json::from_str(&json).unwrap();
        assert_eq!(*c, back);
    }
}

#[test]
fn test_big_o_class_copy() {
    let c = BigOClass::Linear;
    let copied = c;
    assert_eq!(c, copied);
}

// ============ ComplexityBound Tests ============

#[test]
fn test_complexity_bound_new() {
    let bound = ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N);
    assert_eq!(bound.class, BigOClass::Quadratic);
    assert_eq!(bound.confidence, 50); // default
    assert_eq!(bound.coefficient, 1);
    assert_eq!(bound.input_var, InputVariable::N);
}

#[test]
fn test_complexity_bound_constant() {
    let bound = ComplexityBound::constant();
    assert_eq!(bound.class, BigOClass::Constant);
    assert_eq!(bound.confidence, 100);
}

#[test]
fn test_complexity_bound_serde() {
    let bound = ComplexityBound::new(BigOClass::Linear, 2, InputVariable::N);
    let json = serde_json::to_string(&bound).unwrap();
    let back: ComplexityBound = serde_json::from_str(&json).unwrap();
    assert_eq!(back.class, BigOClass::Linear);
    assert_eq!(back.coefficient, 2);
}

#[test]
fn test_complexity_bound_copy() {
    let bound = ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N);
    let copied = bound;
    assert_eq!(copied.class, BigOClass::Linear);
}

// ============ InputVariable Tests ============

#[test]
fn test_input_variable_display() {
    assert_eq!(format!("{}", InputVariable::N), "n");
    assert_eq!(format!("{}", InputVariable::M), "m");
    assert_eq!(format!("{}", InputVariable::K), "k");
    assert_eq!(format!("{}", InputVariable::D), "d");
    assert_eq!(format!("{}", InputVariable::Custom), "x");
}

#[test]
fn test_input_variable_serde() {
    let vars = vec![
        InputVariable::N,
        InputVariable::M,
        InputVariable::K,
        InputVariable::D,
        InputVariable::Custom,
    ];
    for v in &vars {
        let json = serde_json::to_string(v).unwrap();
        let back: InputVariable = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, back);
    }
}

// ============ ComplexityFlags Tests ============

#[test]
fn test_complexity_flags_new() {
    let flags = ComplexityFlags::new();
    assert!(!flags.has(ComplexityFlags::WORST_CASE));
    assert!(!flags.has(ComplexityFlags::PROVEN));
}

#[test]
fn test_complexity_flags_with() {
    let flags = ComplexityFlags::new()
        .with(ComplexityFlags::WORST_CASE)
        .with(ComplexityFlags::PROVEN);
    assert!(flags.has(ComplexityFlags::WORST_CASE));
    assert!(flags.has(ComplexityFlags::PROVEN));
    assert!(!flags.has(ComplexityFlags::AMORTIZED));
}

#[test]
fn test_complexity_flags_is_worst_case() {
    let wc = ComplexityFlags::new().with(ComplexityFlags::WORST_CASE);
    assert!(wc.is_worst_case());
    let avg = ComplexityFlags::new().with(ComplexityFlags::AVERAGE_CASE);
    assert!(!avg.is_worst_case());
}

#[test]
fn test_complexity_flags_is_proven() {
    let proven = ComplexityFlags::new().with(ComplexityFlags::PROVEN);
    assert!(proven.is_proven());
    let empirical = ComplexityFlags::new().with(ComplexityFlags::EMPIRICAL);
    assert!(!empirical.is_proven());
}

#[test]
fn test_complexity_flags_all_flags() {
    let all = ComplexityFlags::new()
        .with(ComplexityFlags::AMORTIZED)
        .with(ComplexityFlags::WORST_CASE)
        .with(ComplexityFlags::AVERAGE_CASE)
        .with(ComplexityFlags::BEST_CASE)
        .with(ComplexityFlags::TIGHT_BOUND)
        .with(ComplexityFlags::EMPIRICAL)
        .with(ComplexityFlags::PROVEN)
        .with(ComplexityFlags::RECURSIVE);
    assert!(all.has(ComplexityFlags::AMORTIZED));
    assert!(all.has(ComplexityFlags::RECURSIVE));
}

// ============ BigOAnalyzer Tests ============

#[test]
fn test_big_o_analyzer_new() {
    let _analyzer = BigOAnalyzer::new();
}

// ============ BigOAnalysisConfig Tests ============

#[test]
fn test_big_o_analysis_config_clone() {
    let config = BigOAnalysisConfig {
        project_path: PathBuf::from("./src"),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["test_*.rs".to_string()],
        confidence_threshold: 70,
        analyze_space_complexity: true,
    };
    let cloned = config.clone();
    assert_eq!(cloned.project_path, PathBuf::from("./src"));
    assert_eq!(cloned.confidence_threshold, 70);
    assert!(cloned.analyze_space_complexity);
}

// ============ ComplexityDistribution Tests ============

#[test]
fn test_complexity_distribution_serde() {
    let dist = ComplexityDistribution {
        constant: 50,
        logarithmic: 10,
        linear: 30,
        linearithmic: 5,
        quadratic: 3,
        cubic: 1,
        exponential: 0,
        unknown: 1,
    };
    let json = serde_json::to_string(&dist).unwrap();
    let back: ComplexityDistribution = serde_json::from_str(&json).unwrap();
    assert_eq!(back.constant, 50);
    assert_eq!(back.linear, 30);
    assert_eq!(back.unknown, 1);
}

// ============ FunctionComplexity Tests ============

#[test]
fn test_function_complexity_serde() {
    let fc = FunctionComplexity {
        file_path: PathBuf::from("src/main.rs"),
        function_name: "process_data".to_string(),
        line_number: 42,
        time_complexity: ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N)
            .with_confidence(80),
        space_complexity: ComplexityBound::new(BigOClass::Linear, 1, InputVariable::N)
            .with_confidence(90),
        confidence: 85,
        notes: vec!["Consider using HashMap".to_string()],
    };
    let json = serde_json::to_string(&fc).unwrap();
    let back: FunctionComplexity = serde_json::from_str(&json).unwrap();
    assert_eq!(back.function_name, "process_data");
    assert_eq!(back.line_number, 42);
    assert_eq!(back.time_complexity.class, BigOClass::Quadratic);
}

// ============ PatternMatch Tests ============

#[test]
fn test_pattern_match_serde() {
    let pm = PatternMatch {
        pattern_name: "nested_loops".to_string(),
        occurrences: 3,
        typical_complexity: BigOClass::Quadratic,
    };
    let json = serde_json::to_string(&pm).unwrap();
    let back: PatternMatch = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pattern_name, "nested_loops");
    assert_eq!(back.occurrences, 3);
}

// ============ BigOAnalysisReport Tests ============

#[test]
fn test_big_o_analysis_report_serde() {
    let report = BigOAnalysisReport {
        analyzed_functions: 100,
        complexity_distribution: ComplexityDistribution {
            constant: 50,
            logarithmic: 10,
            linear: 30,
            linearithmic: 5,
            quadratic: 3,
            cubic: 1,
            exponential: 0,
            unknown: 1,
        },
        high_complexity_functions: vec![],
        pattern_matches: vec![],
        recommendations: vec!["Optimize quadratic functions".to_string()],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: BigOAnalysisReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.analyzed_functions, 100);
    assert_eq!(back.recommendations.len(), 1);
}

// ============ Display Implementation Tests ============

#[test]
fn test_big_o_class_display() {
    // Display uses notation() internally
    assert_eq!(format!("{}", BigOClass::Constant), "O(1)");
    assert_eq!(format!("{}", BigOClass::Logarithmic), "O(log n)");
    assert_eq!(format!("{}", BigOClass::Linear), "O(n)");
    assert_eq!(format!("{}", BigOClass::Linearithmic), "O(n log n)");
    assert_eq!(format!("{}", BigOClass::Quadratic), "O(n²)");
    assert_eq!(format!("{}", BigOClass::Cubic), "O(n³)");
    assert_eq!(format!("{}", BigOClass::Exponential), "O(2^n)");
    assert_eq!(format!("{}", BigOClass::Factorial), "O(n!)");
    assert_eq!(format!("{}", BigOClass::Unknown), "O(?)");
}

#[test]
fn test_complexity_bound_display() {
    let bound = ComplexityBound::new(BigOClass::Quadratic, 1, InputVariable::N).with_confidence(85);
    let display = format!("{}", bound);
    // Display format: "O(n²) (85% confidence)"
    assert!(display.contains("O(n²)"));
    assert!(display.contains("85% confidence"));
}

#[test]
fn test_complexity_bound_display_constant() {
    let bound = ComplexityBound::constant();
    let display = format!("{}", bound);
    assert!(display.contains("O(1)"));
    assert!(display.contains("100% confidence"));
}

#[test]
fn test_complexity_bound_display_various() {
    let cases = vec![
        (BigOClass::Linear, 75, "O(n)", "75%"),
        (BigOClass::Logarithmic, 90, "O(log n)", "90%"),
        (BigOClass::Exponential, 50, "O(2^n)", "50%"),
    ];
    for (class, conf, expected_class, expected_conf) in cases {
        let bound = ComplexityBound::new(class, 1, InputVariable::N).with_confidence(conf);
        let display = format!("{}", bound);
        assert!(display.contains(expected_class), "Expected {} in {}", expected_class, display);
        assert!(display.contains(expected_conf), "Expected {} in {}", expected_conf, display);
    }
}
