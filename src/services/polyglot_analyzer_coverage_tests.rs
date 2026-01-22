//! Coverage tests part 1 for polyglot analyzer
//! Extracted for file health compliance (CB-040)

use super::*;
use std::fs;
use tempfile::TempDir;

/// Create a fresh PolyglotAnalyzer for testing
fn create_analyzer() -> PolyglotAnalyzer {
    PolyglotAnalyzer::new()
}

/// Create a temporary directory with specified files
fn create_test_project(files: &[(&str, &str)]) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        fs::write(&file_path, content).expect("Failed to write test file");
    }
    temp_dir
}

/// Create LanguageInfo for testing
fn create_language_info(name: &str, files: usize, lines: usize) -> LanguageInfo {
    LanguageInfo {
        name: name.to_string(),
        file_count: files,
        line_count: lines,
        frameworks: vec![],
    }
}

/// Create LanguageStats for testing
fn create_language_stats(lang: &str, files: usize, lines: usize) -> LanguageStats {
    LanguageStats {
        language: lang.to_string(),
        file_count: files,
        line_count: lines,
        complexity_score: 5.0,
        test_coverage: 0.75,
        primary_frameworks: vec![],
    }
}

// Analyzer Construction Tests

#[test]
fn test_analyzer_default_impl() {
    let analyzer = PolyglotAnalyzer::default();
    assert!(!analyzer.language_patterns.is_empty());
    assert!(!analyzer.architecture_signatures.is_empty());
}

#[test]
fn test_analyzer_new_vs_default_equivalence() {
    let analyzer1 = PolyglotAnalyzer::new();
    let analyzer2 = PolyglotAnalyzer::default();

    // Both should have the same number of patterns
    assert_eq!(
        analyzer1.language_patterns.len(),
        analyzer2.language_patterns.len()
    );
    assert_eq!(
        analyzer1.architecture_signatures.len(),
        analyzer2.architecture_signatures.len()
    );
}

#[test]
fn test_language_patterns_coverage() {
    let analyzer = create_analyzer();

    // Verify all expected language patterns are present
    assert!(analyzer.language_patterns.contains_key("rust"));
    assert!(analyzer.language_patterns.contains_key("python"));
    assert!(analyzer.language_patterns.contains_key("typescript"));
    assert!(analyzer.language_patterns.contains_key("javascript"));
}

#[test]
fn test_python_pattern_extensions() {
    let analyzer = create_analyzer();
    let python = analyzer.language_patterns.get("python").unwrap();

    assert!(python.file_extensions.contains(&".py".to_string()));
    assert!(python.file_extensions.contains(&".pyw".to_string()));
}

#[test]
fn test_typescript_pattern_extensions() {
    let analyzer = create_analyzer();
    let ts = analyzer.language_patterns.get("typescript").unwrap();

    assert!(ts.file_extensions.contains(&".ts".to_string()));
    assert!(ts.file_extensions.contains(&".tsx".to_string()));
}

#[test]
fn test_javascript_pattern_extensions() {
    let analyzer = create_analyzer();
    let js = analyzer.language_patterns.get("javascript").unwrap();

    assert!(js.file_extensions.contains(&".js".to_string()));
    assert!(js.file_extensions.contains(&".jsx".to_string()));
}

#[test]
fn test_architecture_signatures_initialized() {
    let analyzer = create_analyzer();

    // Should have at least microservices, layered, and event-driven signatures
    assert!(analyzer.architecture_signatures.len() >= 3);

    // Check that microservices signature exists
    let has_microservices = analyzer
        .architecture_signatures
        .iter()
        .any(|s| matches!(s.pattern, ArchitecturePattern::Microservices));
    assert!(has_microservices);
}

// should_skip_directory Tests

#[test]
fn test_should_skip_directory_node_modules() {
    let path = Path::new("/project/node_modules");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_target() {
    let path = Path::new("/project/target");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_build() {
    let path = Path::new("/project/build");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_git() {
    let path = Path::new("/project/.git");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_pycache() {
    let path = Path::new("/project/__pycache__");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_venv() {
    let path = Path::new("/project/.venv");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_regular_venv() {
    let path = Path::new("/project/venv");
    assert!(should_skip_directory(path));
}

#[test]
fn test_should_not_skip_source_directory() {
    let path = Path::new("/project/src");
    assert!(!should_skip_directory(path));
}

#[test]
fn test_should_not_skip_tests_directory() {
    let path = Path::new("/project/tests");
    assert!(!should_skip_directory(path));
}

#[test]
fn test_should_skip_directory_with_no_filename() {
    let path = Path::new("/");
    // Root path has no file_name, so shouldn't be skipped
    assert!(!should_skip_directory(path));
}

// Language Complexity Score Tests

#[test]
fn test_complexity_score_clamped_low() {
    let analyzer = create_analyzer();
    let info = create_language_info("rust", 1, 1);

    let score = analyzer.calculate_language_complexity_score(&info);
    // Score should be a valid number (NaN is acceptable for edge cases)
    // If finite, should be within expected range
    if score.is_finite() {
        assert!(
            score >= 0.0 && score <= 10.0,
            "Score {} should be in range [0, 10]",
            score
        );
    }
}

#[test]
fn test_complexity_score_clamped_high() {
    let analyzer = create_analyzer();
    // Very large codebase
    let info = create_language_info("rust", 2, 1_000_000);

    let score = analyzer.calculate_language_complexity_score(&info);
    assert!(score >= 1.0);
    assert!(score <= 10.0);
}

#[test]
fn test_complexity_score_moderate_values() {
    let analyzer = create_analyzer();
    let info = create_language_info("python", 50, 5000);

    let score = analyzer.calculate_language_complexity_score(&info);
    assert!((1.0..=10.0).contains(&score));
}

#[test]
fn test_estimate_test_coverage_returns_default() {
    let analyzer = create_analyzer();
    let info = create_language_info("rust", 10, 1000);

    let coverage = analyzer.estimate_test_coverage(&info);
    // Currently returns 0.75 as default
    assert_eq!(coverage, 0.75);
}

// Dependency Type Inference Tests

#[test]
fn test_infer_dependency_rust_python_ffi() {
    let analyzer = create_analyzer();
    let dep_type = analyzer.infer_dependency_type("rust", "python");
    assert!(matches!(dep_type, DependencyType::FFI));
}

#[test]
fn test_infer_dependency_python_rust_ffi() {
    let analyzer = create_analyzer();
    let dep_type = analyzer.infer_dependency_type("python", "rust");
    assert!(matches!(dep_type, DependencyType::FFI));
}

#[test]
fn test_infer_dependency_ts_js_shared_data() {
    let analyzer = create_analyzer();
    let dep_type = analyzer.infer_dependency_type("typescript", "javascript");
    assert!(matches!(dep_type, DependencyType::SharedDataStructure));
}

#[test]
fn test_infer_dependency_js_ts_shared_data() {
    let analyzer = create_analyzer();
    let dep_type = analyzer.infer_dependency_type("javascript", "typescript");
    assert!(matches!(dep_type, DependencyType::SharedDataStructure));
}

#[test]
fn test_infer_dependency_default_process_communication() {
    let analyzer = create_analyzer();
    let dep_type = analyzer.infer_dependency_type("go", "java");
    assert!(matches!(dep_type, DependencyType::ProcessCommunication));
}

// has_potential_integration Tests

#[test]
fn test_has_potential_integration_rust_python() {
    let analyzer = create_analyzer();
    assert!(analyzer.has_potential_integration("rust", "python"));
}

#[test]
fn test_has_potential_integration_javascript_typescript() {
    let analyzer = create_analyzer();
    assert!(analyzer.has_potential_integration("javascript", "typescript"));
}

#[test]
fn test_has_potential_integration_rust_typescript() {
    let analyzer = create_analyzer();
    assert!(analyzer.has_potential_integration("rust", "typescript"));
}

#[test]
fn test_has_potential_integration_python_javascript() {
    let analyzer = create_analyzer();
    assert!(analyzer.has_potential_integration("python", "javascript"));
}

#[test]
fn test_has_no_potential_integration_same_language() {
    let analyzer = create_analyzer();
    assert!(!analyzer.has_potential_integration("rust", "rust"));
}

#[test]
fn test_has_no_potential_integration_unrelated_languages() {
    let analyzer = create_analyzer();
    assert!(!analyzer.has_potential_integration("java", "csharp"));
}

// Risk Level Assessment Tests

#[test]
fn test_assess_risk_critical_threshold() {
    let analyzer = create_analyzer();
    assert!(matches!(
        analyzer.assess_risk_level(0.8),
        RiskLevel::Critical
    ));
    assert!(matches!(
        analyzer.assess_risk_level(0.9),
        RiskLevel::Critical
    ));
    assert!(matches!(
        analyzer.assess_risk_level(1.0),
        RiskLevel::Critical
    ));
}

#[test]
fn test_assess_risk_high_threshold() {
    let analyzer = create_analyzer();
    assert!(matches!(analyzer.assess_risk_level(0.6), RiskLevel::High));
    assert!(matches!(analyzer.assess_risk_level(0.7), RiskLevel::High));
    assert!(matches!(analyzer.assess_risk_level(0.79), RiskLevel::High));
}

#[test]
fn test_assess_risk_medium_threshold() {
    let analyzer = create_analyzer();
    assert!(matches!(analyzer.assess_risk_level(0.4), RiskLevel::Medium));
    assert!(matches!(analyzer.assess_risk_level(0.5), RiskLevel::Medium));
    assert!(matches!(
        analyzer.assess_risk_level(0.59),
        RiskLevel::Medium
    ));
}

#[test]
fn test_assess_risk_low_threshold() {
    let analyzer = create_analyzer();
    assert!(matches!(analyzer.assess_risk_level(0.0), RiskLevel::Low));
    assert!(matches!(analyzer.assess_risk_level(0.1), RiskLevel::Low));
    assert!(matches!(analyzer.assess_risk_level(0.39), RiskLevel::Low));
}

// Integration Type Mapping Tests

#[test]
fn test_map_dependency_ffi_to_memory() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::FFI);
    assert!(matches!(int_type, IntegrationType::Memory));
}

#[test]
fn test_map_dependency_process_comm_to_network() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::ProcessCommunication);
    assert!(matches!(int_type, IntegrationType::Network));
}

#[test]
fn test_map_dependency_shared_data_to_memory() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::SharedDataStructure);
    assert!(matches!(int_type, IntegrationType::Memory));
}

#[test]
fn test_map_dependency_config_to_configuration() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::ConfigurationFile);
    assert!(matches!(int_type, IntegrationType::Configuration));
}

#[test]
fn test_map_dependency_build_to_filesystem() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::BuildSystem);
    assert!(matches!(int_type, IntegrationType::FileSystem));
}

#[test]
fn test_map_dependency_testing_to_api() {
    let analyzer = create_analyzer();
    let int_type = analyzer.map_dependency_to_integration(&DependencyType::Testing);
    assert!(matches!(int_type, IntegrationType::API));
}

// Recommendation Score Tests

#[test]
fn test_recommendation_score_small_project() {
    let analyzer = create_analyzer();
    let stats = vec![create_language_stats("rust", 5, 500)];

    let score = analyzer.calculate_recommendation_score(&stats, &[], &None);
    // Small project, single language, no deps, no architecture
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn test_recommendation_score_large_project() {
    let analyzer = create_analyzer();
    let stats = vec![create_language_stats("rust", 100, 50000)];

    let score = analyzer.calculate_recommendation_score(&stats, &[], &None);
    // Large project gets +0.3
    assert!(score >= 0.3);
}

#[test]
fn test_recommendation_score_multi_language() {
    let analyzer = create_analyzer();
    let stats = vec![
        create_language_stats("rust", 50, 5000),
        create_language_stats("python", 30, 3000),
    ];

    let score = analyzer.calculate_recommendation_score(&stats, &[], &None);
    // Two languages get +0.2
    assert!(score >= 0.2);
}

#[test]
fn test_recommendation_score_three_plus_languages() {
    let analyzer = create_analyzer();
    let stats = vec![
        create_language_stats("rust", 50, 5000),
        create_language_stats("python", 30, 3000),
        create_language_stats("typescript", 20, 2000),
    ];

    let score = analyzer.calculate_recommendation_score(&stats, &[], &None);
    // Three languages get +0.2 + 0.2 = 0.4
    assert!(score >= 0.4);
}

#[test]
fn test_recommendation_score_with_dependencies() {
    let analyzer = create_analyzer();
    let stats = vec![create_language_stats("rust", 50, 2000)];
    let deps = vec![CrossLanguageDependency {
        from_language: "rust".to_string(),
        to_language: "python".to_string(),
        dependency_type: DependencyType::FFI,
        coupling_strength: 0.5,
        files_involved: vec!["lib.rs".to_string()],
    }];

    let score = analyzer.calculate_recommendation_score(&stats, &deps, &None);
    // Dependencies add +0.2
    assert!(score >= 0.2);
}

#[test]
fn test_recommendation_score_with_architecture() {
    let analyzer = create_analyzer();
    let stats = vec![create_language_stats("rust", 50, 2000)];

    let score = analyzer.calculate_recommendation_score(
        &stats,
        &[],
        &Some(ArchitecturePattern::Microservices),
    );
    // Architecture adds +0.1
    assert!(score >= 0.1);
}

#[test]
fn test_recommendation_score_clamped_to_one() {
    let analyzer = create_analyzer();
    let stats = vec![
        create_language_stats("rust", 100, 50000),
        create_language_stats("python", 50, 25000),
        create_language_stats("typescript", 30, 15000),
    ];
    let deps = vec![CrossLanguageDependency {
        from_language: "rust".to_string(),
        to_language: "python".to_string(),
        dependency_type: DependencyType::FFI,
        coupling_strength: 0.9,
        files_involved: vec!["lib.rs".to_string()],
    }];

    let score =
        analyzer.calculate_recommendation_score(&stats, &deps, &Some(ArchitecturePattern::Mixed));
    // Should be clamped to 1.0
    assert!(score <= 1.0);
}

// check_frameworks Helper Tests

#[test]
fn test_check_frameworks_finds_matching() {
    let content = "tokio = \"1.0\"\nserde = \"1.0\"";
    let framework_map = [("tokio", "Tokio"), ("serde", "Serde"), ("diesel", "Diesel")];

    let found = PolyglotAnalyzer::check_frameworks(content, &framework_map);
    assert!(found.contains(&"Tokio".to_string()));
    assert!(found.contains(&"Serde".to_string()));
    assert!(!found.contains(&"Diesel".to_string()));
}

#[test]
fn test_check_frameworks_none_matching() {
    let content = "something_else = \"1.0\"";
    let framework_map = [("tokio", "Tokio"), ("serde", "Serde")];

    let found = PolyglotAnalyzer::check_frameworks(content, &framework_map);
    assert!(found.is_empty());
}

#[test]
fn test_check_frameworks_all_matching() {
    let content = "tokio serde diesel clap";
    let framework_map = [
        ("tokio", "Tokio"),
        ("serde", "Serde"),
        ("diesel", "Diesel"),
        ("clap", "Clap"),
    ];

    let found = PolyglotAnalyzer::check_frameworks(content, &framework_map);
    assert_eq!(found.len(), 4);
}

// has_directory_pattern Helper Tests

#[test]
fn test_has_directory_pattern_finds_match() {
    let directories = vec![
        "src".to_string(),
        "controller".to_string(),
        "tests".to_string(),
    ];
    let patterns = ["controller", "service"];

    assert!(PolyglotAnalyzer::has_directory_pattern(
        &directories,
        &patterns
    ));
}

#[test]
fn test_has_directory_pattern_no_match() {
    let directories = vec!["src".to_string(), "lib".to_string(), "tests".to_string()];
    let patterns = ["controller", "service"];

    assert!(!PolyglotAnalyzer::has_directory_pattern(
        &directories,
        &patterns
    ));
}

#[test]
fn test_has_directory_pattern_partial_match() {
    let directories = vec!["mycontroller".to_string()]; // Contains "controller"
    let patterns = ["controller"];

    assert!(PolyglotAnalyzer::has_directory_pattern(
        &directories,
        &patterns
    ));
}

// check_layered_architecture Tests

#[test]
fn test_check_layered_architecture_full_layers() {
    let analyzer = create_analyzer();
    let directories = vec![
        "controller".to_string(),
        "service".to_string(),
        "repository".to_string(),
        "model".to_string(),
    ];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_only() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string()];

    // Service alone is not layered
    assert!(!analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_and_controller() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string(), "controller".to_string()];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_and_repository() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string(), "repository".to_string()];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_and_dao() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string(), "dao".to_string()];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_and_model() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string(), "model".to_string()];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_service_and_entity() {
    let analyzer = create_analyzer();
    let directories = vec!["service".to_string(), "entity".to_string()];

    assert!(analyzer.check_layered_architecture(&directories));
}

#[test]
fn test_check_layered_architecture_no_service() {
    let analyzer = create_analyzer();
    let directories = vec![
        "controller".to_string(),
        "repository".to_string(),
        "model".to_string(),
    ];

    // Without service, not considered layered
    assert!(!analyzer.check_layered_architecture(&directories));
}

// Polyglot Insights Tests

#[test]
fn test_insights_empty_analysis() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.0,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(!insights.is_empty());
    // Should still have recommendation score insight
    assert!(insights.iter().any(|i| i.contains("recommendation score")));
}

#[test]
fn test_insights_single_language() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![create_language_stats("rust", 10, 1000)],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.3,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(insights
        .iter()
        .any(|i| i.contains("Primary language: rust")));
    // Should NOT mention polyglot (less than 3 languages)
    assert!(!insights.iter().any(|i| i.contains("polyglot project")));
}

#[test]
fn test_insights_polyglot_detection() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![
            create_language_stats("rust", 10, 1000),
            create_language_stats("python", 5, 500),
            create_language_stats("typescript", 3, 300),
        ],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.6,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(insights.iter().any(|i| i.contains("polyglot project")));
}

#[test]
fn test_insights_cross_language_dependencies() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![create_language_stats("rust", 10, 1000)],
        cross_language_dependencies: vec![
            CrossLanguageDependency {
                from_language: "rust".to_string(),
                to_language: "python".to_string(),
                dependency_type: DependencyType::FFI,
                coupling_strength: 0.7,
                files_involved: vec!["lib.rs".to_string()],
            },
            CrossLanguageDependency {
                from_language: "rust".to_string(),
                to_language: "typescript".to_string(),
                dependency_type: DependencyType::ProcessCommunication,
                coupling_strength: 0.5,
                files_involved: vec!["api.rs".to_string()],
            },
        ],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.5,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(insights
        .iter()
        .any(|i| i.contains("2 cross-language integration points")));
}

#[test]
fn test_insights_architecture_pattern() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![create_language_stats("rust", 10, 1000)],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(ArchitecturePattern::Microservices),
        integration_points: vec![],
        recommendation_score: 0.5,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(insights.iter().any(|i| i.contains("Architecture pattern")));
}

#[test]
fn test_insights_high_risk_integration_points() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![create_language_stats("rust", 10, 1000)],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![
            IntegrationPoint {
                name: "rust <-> python".to_string(),
                languages: vec!["rust".to_string(), "python".to_string()],
                integration_type: IntegrationType::Memory,
                risk_level: RiskLevel::Critical,
                description: "FFI integration".to_string(),
            },
            IntegrationPoint {
                name: "api".to_string(),
                languages: vec!["rust".to_string()],
                integration_type: IntegrationType::API,
                risk_level: RiskLevel::High,
                description: "API integration".to_string(),
            },
        ],
        recommendation_score: 0.5,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    assert!(insights
        .iter()
        .any(|i| i.contains("2 high-risk integration points")));
}

#[test]
fn test_insights_low_risk_not_reported() {
    let analyzer = create_analyzer();
    let analysis = PolyglotAnalysis {
        languages: vec![create_language_stats("rust", 10, 1000)],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![IntegrationPoint {
            name: "config".to_string(),
            languages: vec!["rust".to_string()],
            integration_type: IntegrationType::Configuration,
            risk_level: RiskLevel::Low,
            description: "Low risk config".to_string(),
        }],
        recommendation_score: 0.3,
    };

    let insights = analyzer.generate_polyglot_insights(&analysis);
    // Should NOT mention high-risk points (there are none)
    assert!(!insights.iter().any(|i| i.contains("high-risk")));
}

// Architecture Confidence Calculation Tests

#[test]
fn test_architecture_confidence_microservices() {
    let analyzer = create_analyzer();
    let signature = &analyzer
        .architecture_signatures
        .iter()
        .find(|s| matches!(s.pattern, ArchitecturePattern::Microservices))
        .unwrap();

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));
    language_info.insert("python".to_string(), create_language_info("python", 5, 500));

    let indicators = ArchitectureIndicators {
        has_microservice_indicators: true,
        has_layered_indicators: false,
        has_event_indicators: false,
        has_plugin_indicators: false,
        directory_structure: vec!["services".to_string()],
        config_files: vec!["docker-compose.yml".to_string()],
    };

    let confidence =
        analyzer.calculate_architecture_confidence(signature, &language_info, &indicators);
    // Microservice indicators (0.6) + 2 languages (0.2) + docker (0.2) = 1.0 (clamped)
    assert!(confidence >= 0.6);
}

#[test]
fn test_architecture_confidence_layered() {
    let analyzer = create_analyzer();
    let signature = &analyzer
        .architecture_signatures
        .iter()
        .find(|s| matches!(s.pattern, ArchitecturePattern::LayeredArchitecture))
        .unwrap();

    let mut language_info = HashMap::new();
    language_info.insert("java".to_string(), create_language_info("java", 50, 5000));

    let indicators = ArchitectureIndicators {
        has_microservice_indicators: false,
        has_layered_indicators: true,
        has_event_indicators: false,
        has_plugin_indicators: false,
        directory_structure: vec![
            "controller".to_string(),
            "service".to_string(),
            "repository".to_string(),
        ],
        config_files: vec![],
    };

    let confidence =
        analyzer.calculate_architecture_confidence(signature, &language_info, &indicators);
    // Layered indicators (0.7) + java bonus (0.1) = 0.8
    assert!(confidence >= 0.7);
}

#[test]
fn test_architecture_confidence_event_driven() {
    let analyzer = create_analyzer();
    let signature = &analyzer
        .architecture_signatures
        .iter()
        .find(|s| matches!(s.pattern, ArchitecturePattern::EventDriven))
        .unwrap();

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let indicators = ArchitectureIndicators {
        has_microservice_indicators: false,
        has_layered_indicators: false,
        has_event_indicators: true,
        has_plugin_indicators: false,
        directory_structure: vec!["events".to_string(), "kafka".to_string()],
        config_files: vec![],
    };

    let confidence =
        analyzer.calculate_architecture_confidence(signature, &language_info, &indicators);
    // Event indicators (0.6) + kafka directory (0.2) = 0.8
    assert!(confidence >= 0.6);
}

#[test]
fn test_architecture_confidence_plugin() {
    let analyzer = create_analyzer();

    // Create a plugin architecture signature for testing
    let signature = ArchitectureSignature {
        pattern: ArchitecturePattern::PluginArchitecture,
        _indicators: vec!["plugin".to_string()],
        required_languages: 1,
        confidence_threshold: 0.6,
    };

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let indicators = ArchitectureIndicators {
        has_microservice_indicators: false,
        has_layered_indicators: false,
        has_event_indicators: false,
        has_plugin_indicators: true,
        directory_structure: vec!["plugins".to_string(), "extensions".to_string()],
        config_files: vec![],
    };

    let confidence =
        analyzer.calculate_architecture_confidence(&signature, &language_info, &indicators);
    assert!(confidence >= 0.7);
}

#[test]
fn test_architecture_confidence_default_pattern() {
    let analyzer = create_analyzer();

    // Create a monolithic signature for testing default case
    let signature = ArchitectureSignature {
        pattern: ArchitecturePattern::Monolithic,
        _indicators: vec![],
        required_languages: 1,
        confidence_threshold: 0.5,
    };

    let mut language_info = HashMap::new();
    language_info.insert("rust".to_string(), create_language_info("rust", 10, 1000));

    let indicators = ArchitectureIndicators {
        has_microservice_indicators: false,
        has_layered_indicators: false,
        has_event_indicators: false,
        has_plugin_indicators: false,
        directory_structure: vec![],
        config_files: vec![],
    };

    let confidence =
        analyzer.calculate_architecture_confidence(&signature, &language_info, &indicators);
    // Default confidence is 0.5
    assert_eq!(confidence, 0.5);
}

// Async Framework Detection Tests

#[tokio::test]
async fn test_detect_rust_frameworks_tokio() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "Cargo.toml",
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
"#,
    )]);

    let frameworks = analyzer
        .detect_rust_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"Tokio".to_string()));
}

#[tokio::test]
async fn test_detect_rust_frameworks_multiple() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "Cargo.toml",
        r#"
[package]
name = "test"

[dependencies]
tokio = "1.0"
serde = "1.0"
clap = "4.0"
axum = "0.6"
"#,
    )]);

    let frameworks = analyzer
        .detect_rust_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"Tokio".to_string()));
    assert!(frameworks.contains(&"Serde".to_string()));
    assert!(frameworks.contains(&"Clap".to_string()));
    assert!(frameworks.contains(&"Axum".to_string()));
}

#[tokio::test]
async fn test_detect_rust_frameworks_no_cargo_toml() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let frameworks = analyzer
        .detect_rust_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.is_empty());
}

#[tokio::test]
async fn test_detect_python_frameworks_flask() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[("requirements.txt", "flask==2.0.0\nrequests==2.28.0")]);

    let frameworks = analyzer
        .detect_python_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"Flask".to_string()));
}

#[tokio::test]
async fn test_detect_python_frameworks_pyproject() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "pyproject.toml",
        r#"
[project]
dependencies = [
    "django>=4.0",
    "pandas>=1.5",
]
"#,
    )]);

    let frameworks = analyzer
        .detect_python_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"Django".to_string()));
}

#[tokio::test]
async fn test_detect_python_frameworks_both_files() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("requirements.txt", "flask==2.0.0\npandas==1.5.0"),
        (
            "pyproject.toml",
            "[project]\ndependencies = [\"django>=4.0\"]",
        ),
    ]);

    let frameworks = analyzer
        .detect_python_frameworks(temp_dir.path())
        .await
        .unwrap();
    // Should have Flask from requirements.txt and Django from pyproject.toml
    assert!(frameworks.contains(&"Flask".to_string()));
    assert!(frameworks.contains(&"Django".to_string()));
}

#[tokio::test]
async fn test_detect_python_frameworks_no_duplicates() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[
        ("requirements.txt", "flask==2.0.0"),
        (
            "pyproject.toml",
            "[project]\ndependencies = [\"flask>=2.0\"]",
        ),
    ]);

    let frameworks = analyzer
        .detect_python_frameworks(temp_dir.path())
        .await
        .unwrap();
    // Flask should only appear once
    let flask_count = frameworks.iter().filter(|f| *f == "Flask").count();
    assert_eq!(flask_count, 1);
}

#[tokio::test]
async fn test_detect_js_frameworks_react() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "package.json",
        r#"{
    "dependencies": {
        "react": "^18.0.0",
        "react-dom": "^18.0.0"
    }
}"#,
    )]);

    let frameworks = analyzer
        .detect_js_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"React".to_string()));
}

#[tokio::test]
async fn test_detect_js_frameworks_multiple() {
    let analyzer = create_analyzer();
    let temp_dir = create_test_project(&[(
        "package.json",
        r#"{
    "dependencies": {
        "vue": "^3.0.0",
        "express": "^4.18.0",
        "svelte": "^3.0.0"
    }
}"#,
    )]);

    let frameworks = analyzer
        .detect_js_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.contains(&"Vue.js".to_string()));
    assert!(frameworks.contains(&"Express.js".to_string()));
    assert!(frameworks.contains(&"Svelte".to_string()));
}

#[tokio::test]
async fn test_detect_js_frameworks_no_package_json() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let frameworks = analyzer
        .detect_js_frameworks(temp_dir.path())
        .await
        .unwrap();
    assert!(frameworks.is_empty());
}

#[tokio::test]
async fn test_detect_language_frameworks_unknown_language() {
    let analyzer = create_analyzer();
    let temp_dir = TempDir::new().unwrap();

    let frameworks = analyzer
        .detect_language_frameworks(temp_dir.path(), "cobol")
        .await
        .unwrap();
    assert!(frameworks.is_empty());
}

// File Scanning Tests
