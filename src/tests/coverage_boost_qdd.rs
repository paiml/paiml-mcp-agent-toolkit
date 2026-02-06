#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for qdd module
//! Tests: core types, patterns, profiles, refactor engine, code analyzer, pattern engine

use crate::qdd::core::{QualityMetrics, QualityProfile, QualityThresholds};
use crate::qdd::patterns::{
    DesignPattern, DependencyInjectionPattern, DryPattern, KissPattern,
    SingleResponsibilityPattern, ViolationSeverity, YagniPattern,
};
use crate::qdd::profiles::{ProfileComparator, ProfileValidator, QualityProfiles};
use crate::qdd::refactor::{CodeAnalyzer, PatternEngine, QualityRefactoringEngine};
use crate::qdd::{
    CodeType, CreateSpec, EnhanceSpec, MigrateSpec, Parameter, QddOperation, QddTool,
    QualityScore, RefactorSpec,
};
use std::path::PathBuf;

// --- QualityProfile constructor tests ---

#[test]
fn test_quality_profile_extreme() {
    let p = QualityProfile::extreme();
    assert_eq!(p.name, "extreme");
    assert_eq!(p.thresholds.max_complexity, 5);
    assert!(p.patterns.enforce_solid);
    assert!(p.thresholds.zero_satd);
}

#[test]
fn test_quality_profile_standard() {
    let p = QualityProfile::standard();
    assert_eq!(p.name, "standard");
    assert_eq!(p.thresholds.max_complexity, 10);
    assert!(p.thresholds.min_coverage >= 80);
}

#[test]
fn test_quality_profile_relaxed() {
    let p = QualityProfile::relaxed();
    assert_eq!(p.name, "relaxed");
    assert!(p.thresholds.max_complexity > 10);
    assert!(!p.thresholds.zero_satd);
}

// --- QualityMetrics tests ---

#[test]
fn test_quality_metrics_default() {
    let m = QualityMetrics::default();
    assert_eq!(m.complexity, 0);
    assert_eq!(m.coverage, 0);
    assert!(!m.has_doctests);
}

#[test]
fn test_quality_metrics_calculate_score_perfect() {
    let m = QualityMetrics {
        complexity: 0,
        cognitive_complexity: 0,
        coverage: 100,
        tdg: 0,
        satd_count: 0,
        dead_code_percentage: 0,
        has_doctests: true,
        has_property_tests: true,
    };
    let score = m.calculate_score();
    assert!(score >= 99.0); // Should be 100 or close
}

#[test]
fn test_quality_metrics_calculate_score_worst() {
    let m = QualityMetrics {
        complexity: 50,
        cognitive_complexity: 50,
        coverage: 0,
        tdg: 20,
        satd_count: 10,
        dead_code_percentage: 50,
        has_doctests: false,
        has_property_tests: false,
    };
    let score = m.calculate_score();
    assert!(score >= 0.0);
    assert!(score < 10.0);
}

#[test]
fn test_quality_metrics_calculate_score_medium() {
    let m = QualityMetrics {
        complexity: 10,
        cognitive_complexity: 15,
        coverage: 80,
        tdg: 5,
        satd_count: 0,
        dead_code_percentage: 5,
        has_doctests: true,
        has_property_tests: false,
    };
    let score = m.calculate_score();
    assert!(score > 40.0);
    assert!(score < 90.0);
}

// --- meets_thresholds tests ---

#[test]
fn test_meets_thresholds_passing() {
    let profile = QualityProfile::standard();
    let metrics = QualityMetrics {
        complexity: 5,
        cognitive_complexity: 8,
        coverage: 90,
        tdg: 3,
        ..Default::default()
    };
    assert!(profile.meets_thresholds(&metrics));
}

#[test]
fn test_meets_thresholds_failing_complexity() {
    let profile = QualityProfile::extreme();
    let metrics = QualityMetrics {
        complexity: 20, // exceeds extreme threshold of 5
        cognitive_complexity: 1,
        coverage: 100,
        tdg: 0,
        ..Default::default()
    };
    assert!(!profile.meets_thresholds(&metrics));
}

#[test]
fn test_meets_thresholds_failing_coverage() {
    let profile = QualityProfile::standard();
    let metrics = QualityMetrics {
        complexity: 5,
        cognitive_complexity: 5,
        coverage: 50, // below minimum
        tdg: 3,
        ..Default::default()
    };
    assert!(!profile.meets_thresholds(&metrics));
}

// --- QualityProfiles factory tests ---

#[test]
fn test_available_profiles() {
    let profiles = QualityProfiles::available_profiles();
    assert!(profiles.len() >= 5);
    assert!(profiles.contains(&"extreme"));
    assert!(profiles.contains(&"standard"));
    assert!(profiles.contains(&"relaxed"));
}

#[test]
fn test_by_name_valid() {
    assert!(QualityProfiles::by_name("extreme").is_some());
    assert!(QualityProfiles::by_name("standard").is_some());
    assert!(QualityProfiles::by_name("relaxed").is_some());
    assert!(QualityProfiles::by_name("enterprise").is_some());
    assert!(QualityProfiles::by_name("startup").is_some());
    assert!(QualityProfiles::by_name("legacy").is_some());
}

#[test]
fn test_by_name_invalid() {
    assert!(QualityProfiles::by_name("nonexistent").is_none());
    assert!(QualityProfiles::by_name("").is_none());
}

#[test]
fn test_enterprise_profile() {
    let p = QualityProfiles::enterprise();
    assert_eq!(p.name, "enterprise");
    assert!(p.patterns.enforce_solid);
}

#[test]
fn test_startup_profile() {
    let p = QualityProfiles::startup();
    assert_eq!(p.name, "startup");
}

#[test]
fn test_legacy_profile() {
    let p = QualityProfiles::legacy();
    assert_eq!(p.name, "legacy");
    assert!(p.thresholds.max_complexity > 15);
}

// --- ProfileValidator tests ---

#[test]
fn test_profile_validator_good_metrics() {
    let profile = QualityProfile::relaxed();
    let metrics = QualityMetrics {
        complexity: 5,
        cognitive_complexity: 5,
        coverage: 90,
        tdg: 2,
        satd_count: 0,
        dead_code_percentage: 1,
        has_doctests: true,
        has_property_tests: true,
    };
    let validation = ProfileValidator::validate_profile_for_codebase(&profile, &metrics);
    assert!(validation.is_valid);
}

#[test]
fn test_profile_validator_poor_metrics() {
    let profile = QualityProfile::extreme();
    let metrics = QualityMetrics {
        complexity: 30,
        cognitive_complexity: 40,
        coverage: 20,
        tdg: 15,
        satd_count: 50,
        dead_code_percentage: 30,
        has_doctests: false,
        has_property_tests: false,
    };
    let validation = ProfileValidator::validate_profile_for_codebase(&profile, &metrics);
    assert!(!validation.is_valid);
    assert!(!validation.issues.is_empty());
}

// --- ProfileComparator tests ---

#[test]
fn test_profile_comparator_same() {
    let p1 = QualityProfile::standard();
    let p2 = QualityProfile::standard();
    let comparison = ProfileComparator::compare(&p1, &p2);
    assert!(comparison.differences.is_empty() || comparison.is_stricter.is_none());
}

#[test]
fn test_profile_comparator_different() {
    let extreme = QualityProfile::extreme();
    let relaxed = QualityProfile::relaxed();
    let comparison = ProfileComparator::compare(&extreme, &relaxed);
    assert!(!comparison.differences.is_empty());
    // extreme should be stricter
    if let Some(ref stricter) = comparison.is_stricter {
        assert!(stricter.contains("extreme"));
    }
}

// --- Pattern detection tests ---

#[test]
fn test_srp_name_description() {
    let p = SingleResponsibilityPattern;
    assert_eq!(p.name(), "Single Responsibility Principle");
    assert!(!p.description().is_empty());
}

#[test]
fn test_srp_apply_short_code() {
    let p = SingleResponsibilityPattern;
    let result = p.apply("fn hello() { println!(\"hi\"); }").unwrap();
    assert!(result.contains("fn hello"));
}

#[test]
fn test_srp_apply_long_code() {
    let p = SingleResponsibilityPattern;
    let long_code = (0..25).map(|i| format!("    let x{i} = {i};")).collect::<Vec<_>>().join("\n");
    let code = format!("fn big_func() {{\n{long_code}\n}}");
    let result = p.apply(&code).unwrap();
    assert!(result.contains("SRP") || result.contains("single responsibility") || result.contains("helper"));
}

#[test]
fn test_srp_detect_violations_clean() {
    let p = SingleResponsibilityPattern;
    let violations = p.detect_violations("fn small() { 1 }");
    assert!(violations.is_empty());
}

#[test]
fn test_srp_detect_violations_complex() {
    let p = SingleResponsibilityPattern;
    let long_code = (0..35).map(|i| format!("    let x{i} = {i};")).collect::<Vec<_>>().join("\n");
    let code = format!("fn big_func() {{\n{long_code}\n}}");
    let violations = p.detect_violations(&code);
    assert!(!violations.is_empty());
}

#[test]
fn test_dry_pattern() {
    let p = DryPattern;
    assert_eq!(p.name(), "Don't Repeat Yourself");
    let _ = p.description();
}

#[test]
fn test_kiss_pattern() {
    let p = KissPattern;
    assert_eq!(p.name(), "Keep It Simple Stupid");
    let _ = p.apply("fn simple() {}").unwrap();
}

#[test]
fn test_yagni_pattern() {
    let p = YagniPattern;
    assert_eq!(p.name(), "You Aren't Gonna Need It");
    let _ = p.detect_violations("fn used() {}");
}

#[test]
fn test_di_pattern() {
    let p = DependencyInjectionPattern;
    assert_eq!(p.name(), "Dependency Injection");
    let _ = p.apply("fn new() -> Self { Self {} }").unwrap();
}

#[test]
fn test_violation_severity_debug() {
    let _ = format!("{:?}", ViolationSeverity::Critical);
    let _ = format!("{:?}", ViolationSeverity::Major);
    let _ = format!("{:?}", ViolationSeverity::Minor);
}

// --- CodeAnalyzer tests ---

#[test]
fn test_code_analyzer_simple() {
    let analyzer = CodeAnalyzer::new(QualityProfile::standard());
    let analysis = analyzer.analyze("fn main() { println!(\"hello\"); }").unwrap();
    assert_eq!(analysis.function_count, 1);
    assert!(analysis.complexity >= 1);
}

#[test]
fn test_code_analyzer_complex() {
    let code = r#"
fn complex() {
    if true {
        for i in 0..10 {
            if i > 5 {
                match i {
                    6 => println!("six"),
                    _ => println!("other"),
                }
            }
        }
    }
    while false {
        // never runs
    }
}
"#;
    let analyzer = CodeAnalyzer::new(QualityProfile::standard());
    let analysis = analyzer.analyze(code).unwrap();
    assert!(analysis.complexity > 3); // if + for + if + match + while
}

#[test]
fn test_code_analyzer_satd_detection() {
    let code = "// TODO: fix this\n// FIXME: broken\n// HACK: workaround\n";
    let analyzer = CodeAnalyzer::new(QualityProfile::standard());
    let analysis = analyzer.analyze(code).unwrap();
    assert_eq!(analysis.satd_count, 3);
}

#[test]
fn test_code_analyzer_empty_code() {
    let analyzer = CodeAnalyzer::new(QualityProfile::standard());
    let analysis = analyzer.analyze("").unwrap();
    assert_eq!(analysis.function_count, 0);
    assert_eq!(analysis.satd_count, 0);
}

// --- PatternEngine tests ---

#[test]
fn test_pattern_engine_new() {
    let engine = PatternEngine::new();
    let _ = format!("{:p}", &engine);
}

#[test]
fn test_pattern_engine_default() {
    let engine = PatternEngine::default();
    let _ = format!("{:p}", &engine);
}

#[test]
fn test_pattern_engine_srp() {
    let engine = PatternEngine::new();
    let result = engine.apply_pattern("fn test() {}", "single_responsibility");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Single Responsibility"));
}

#[test]
fn test_pattern_engine_di() {
    let engine = PatternEngine::new();
    let result = engine.apply_pattern("fn test() {}", "dependency_injection");
    assert!(result.is_ok());
}

#[test]
fn test_pattern_engine_unknown() {
    let engine = PatternEngine::new();
    let result = engine.apply_pattern("fn test() {}", "nonexistent");
    assert!(result.is_err());
}

// --- QddTool tests ---

#[test]
fn test_qdd_tool_with_profile() {
    let tool = QddTool::with_profile(QualityProfile::standard());
    let _ = format!("{:p}", &tool);
}

// --- QDD type construction tests ---

#[test]
fn test_create_spec() {
    let spec = CreateSpec {
        code_type: CodeType::Function,
        name: "calculate_sum".to_string(),
        purpose: "Add two numbers".to_string(),
        inputs: vec![Parameter {
            name: "a".to_string(),
            param_type: "i32".to_string(),
            description: Some("First number".to_string()),
        }],
        outputs: Parameter {
            name: "result".to_string(),
            param_type: "i32".to_string(),
            description: None,
        },
    };
    assert_eq!(spec.name, "calculate_sum");
    assert_eq!(spec.inputs.len(), 1);
}

#[test]
fn test_code_type_variants() {
    let types = [
        CodeType::Function,
        CodeType::Module,
        CodeType::Service,
        CodeType::Test,
    ];
    for t in &types {
        let _ = format!("{:?}", t);
    }
}

#[test]
fn test_refactor_spec() {
    let spec = RefactorSpec {
        file_path: PathBuf::from("src/main.rs"),
        function_name: Some("complex_fn".to_string()),
        target_metrics: QualityThresholds {
            max_complexity: 10,
            max_cognitive: 15,
            min_coverage: 80,
            max_tdg: 5,
            max_entropy_violations: 0,
            zero_satd: true,
            zero_dead_code: true,
            require_doctests: false,
            require_property_tests: false,
        },
    };
    assert_eq!(spec.file_path, PathBuf::from("src/main.rs"));
}

#[test]
fn test_enhance_spec() {
    let spec = EnhanceSpec {
        base_file: PathBuf::from("src/lib.rs"),
        features: vec!["caching".to_string(), "logging".to_string()],
        maintain_api: true,
    };
    assert_eq!(spec.features.len(), 2);
    assert!(spec.maintain_api);
}

#[test]
fn test_migrate_spec() {
    let spec = MigrateSpec {
        from_pattern: "unwrap".to_string(),
        to_pattern: "expect".to_string(),
        files: vec![PathBuf::from("src/main.rs")],
    };
    assert_eq!(spec.from_pattern, "unwrap");
}

#[test]
fn test_qdd_operation_variants() {
    let _create = QddOperation::Create(CreateSpec {
        code_type: CodeType::Function,
        name: "test".to_string(),
        purpose: "test".to_string(),
        inputs: vec![],
        outputs: Parameter {
            name: "out".to_string(),
            param_type: "()".to_string(),
            description: None,
        },
    });
    let _ = format!("{:?}", _create);
}

#[test]
fn test_quality_score_construction() {
    let score = QualityScore {
        overall: 85.0,
        complexity: 5,
        coverage: 90.0,
        tdg: 3,
    };
    assert_eq!(score.overall, 85.0);
    let _ = format!("{:?}", score);
}

// --- QualityRefactoringEngine tests ---

#[test]
fn test_refactoring_engine_migrate_pattern() {
    let engine = QualityRefactoringEngine::new(QualityProfile::standard());
    let code = "fn test() { let x = foo.unwrap(); }";
    let result = engine.migrate_pattern(code, "unwrap", "expect");
    assert!(result.is_ok());
}

#[test]
fn test_refactoring_engine_new() {
    let _engine = QualityRefactoringEngine::new(QualityProfile::extreme());
}
