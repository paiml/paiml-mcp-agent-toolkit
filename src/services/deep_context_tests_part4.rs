//! Tests for deep context - Part 4: Annotation indicators and advanced coverage
//! Extracted for file health compliance (CB-040)

use super::*;

mod annotation_indicator_tests {
    use super::*;
    use std::path::PathBuf;

    // ANNOTATION INDICATOR TESTS

    #[test]
    fn test_add_defect_indicator_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_defect_indicator(&mut result, 0.8);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🔴"));
    }

    #[test]
    fn test_add_defect_indicator_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_defect_indicator(&mut result, 0.5);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🟡"));
    }

    #[test]
    fn test_add_defect_indicator_low() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_defect_indicator(&mut result, 0.3);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_add_cognitive_complexity_indicator_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_cognitive_complexity_indicator(&mut result, 35);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🧠"));
    }

    #[test]
    fn test_add_cognitive_complexity_indicator_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_cognitive_complexity_indicator(&mut result, 20);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🧪"));
    }

    #[test]
    fn test_add_cognitive_complexity_indicator_low() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_cognitive_complexity_indicator(&mut result, 10);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_add_coverage_indicator_low() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_coverage_indicator(&mut result, 0.3);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🚨"));
    }

    #[test]
    fn test_add_coverage_indicator_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_coverage_indicator(&mut result, 0.6);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("⚠️"));
    }

    #[test]
    fn test_add_coverage_indicator_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_coverage_indicator(&mut result, 0.9);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("✅"));
    }

    #[test]
    fn test_add_churn_indicator_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_churn_indicator(&mut result, 0.9);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🔥"));
    }

    #[test]
    fn test_add_churn_indicator_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_churn_indicator(&mut result, 0.6);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🌡️"));
    }

    #[test]
    fn test_add_churn_indicator_low() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_churn_indicator(&mut result, 0.3);
        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("🌊"));
    }

    #[test]
    fn test_add_churn_indicator_none() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_churn_indicator(&mut result, 0.1);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_add_memory_complexity_indicator_o1() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_memory_complexity_indicator(&mut result, "O(1)");
        assert!(result[0].contains("💎"));
    }

    #[test]
    fn test_add_memory_complexity_indicator_linear() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_memory_complexity_indicator(&mut result, "O(n)");
        assert!(result[0].contains("💙"));
    }

    #[test]
    fn test_add_duplication_indicator_high() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_duplication_indicator(&mut result, 0.4);
        assert!(result[0].contains("📑"));
    }

    #[test]
    fn test_add_duplication_indicator_medium() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_duplication_indicator(&mut result, 0.15);
        assert!(result[0].contains("📄"));
    }

    #[test]
    fn test_add_duplication_indicator_low() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let mut result = Vec::new();
        analyzer.add_duplication_indicator(&mut result, 0.05);
        assert_eq!(result.len(), 0);
    }

    // CONFIG AND AUTO-SCALING TESTS

    #[test]
    fn test_deep_context_config_with_auto_scaling() {
        let config = DeepContextConfig::with_auto_scaling();
        assert!(config.parallel >= 2);
        assert!(config.parallel <= num_cpus::get());
    }

    #[test]
    fn test_should_exclude_path_node_modules() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert!(analyzer.should_exclude_path(Path::new("/project/node_modules/package/file.js")));
    }

    #[test]
    fn test_should_exclude_path_target() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert!(analyzer.should_exclude_path(Path::new("/project/target/debug/file")));
    }

    #[test]
    fn test_should_exclude_path_git() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert!(analyzer.should_exclude_path(Path::new("/project/.git/objects/abc")));
    }

    #[test]
    fn test_should_not_exclude_src() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        assert!(!analyzer.should_exclude_path(Path::new("/project/src/main.rs")));
    }

    // FORMAT IMPORT PATH TESTS

    #[test]
    fn test_format_import_path_with_alias() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let result = analyzer.format_import_path("std", &[], &Some("s".to_string()));
        assert_eq!(result, "std as s");
    }

    #[test]
    fn test_format_import_path_with_items() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let items = vec!["println".to_string(), "eprintln".to_string()];
        let result = analyzer.format_import_path("std::io", &items, &None);
        assert_eq!(result, "std::io (println, eprintln)");
    }

    #[test]
    fn test_format_import_path_simple() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let result = analyzer.format_import_path("std::io", &[], &None);
        assert_eq!(result, "std::io");
    }

    // DEFECT DENSITY CALCULATION TESTS

    #[test]
    fn test_calculate_defect_density_with_loc() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let density = analyzer.calculate_defect_density(10, 5000);
        assert!((density - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_defect_density_zero_loc() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let density = analyzer.calculate_defect_density(10, 0);
        assert_eq!(density, 0.0);
    }

    // SATD SEVERITY CONVERSION TESTS

    #[test]
    fn test_satd_severity_to_level() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        use crate::services::satd_detector::Severity;

        assert_eq!(
            analyzer.satd_severity_to_level(&Severity::Critical),
            "error"
        );
        assert_eq!(analyzer.satd_severity_to_level(&Severity::High), "warning");
        assert_eq!(analyzer.satd_severity_to_level(&Severity::Medium), "note");
        assert_eq!(analyzer.satd_severity_to_level(&Severity::Low), "note");
    }

    // COLLECT FILE PATHS TESTS

    #[test]
    fn test_collect_file_paths_single_file() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let node = AnnotatedNode {
            name: "test.rs".to_string(),
            path: PathBuf::from("/project/test.rs"),
            node_type: NodeType::File,
            children: vec![],
            annotations: NodeAnnotations::default(),
        };

        let paths = analyzer.collect_file_paths(&node);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/project/test.rs");
    }

    #[test]
    fn test_collect_file_paths_directory_with_children() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let file1 = AnnotatedNode {
            name: "file1.rs".to_string(),
            path: PathBuf::from("/project/src/file1.rs"),
            node_type: NodeType::File,
            children: vec![],
            annotations: NodeAnnotations::default(),
        };

        let file2 = AnnotatedNode {
            name: "file2.rs".to_string(),
            path: PathBuf::from("/project/src/file2.rs"),
            node_type: NodeType::File,
            children: vec![],
            annotations: NodeAnnotations::default(),
        };

        let dir = AnnotatedNode {
            name: "src".to_string(),
            path: PathBuf::from("/project/src"),
            node_type: NodeType::Directory,
            children: vec![file1, file2],
            annotations: NodeAnnotations::default(),
        };

        let paths = analyzer.collect_file_paths(&dir);
        assert_eq!(paths.len(), 2);
    }

    // NODE DISPLAY FORMAT TESTS

    #[test]
    fn test_format_node_display_file() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let node = AnnotatedNode {
            name: "test.rs".to_string(),
            path: PathBuf::from("/project/test.rs"),
            node_type: NodeType::File,
            children: vec![],
            annotations: NodeAnnotations::default(),
        };

        let display = analyzer.format_node_display(&node).unwrap();
        assert_eq!(display, "test.rs");
    }

    #[test]
    fn test_format_node_display_directory() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let node = AnnotatedNode {
            name: "src".to_string(),
            path: PathBuf::from("/project/src"),
            node_type: NodeType::Directory,
            children: vec![],
            annotations: NodeAnnotations::default(),
        };

        let display = analyzer.format_node_display(&node).unwrap();
        assert_eq!(display, "src/");
    }

    #[test]
    fn test_format_node_display_with_annotations() {
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);

        let node = AnnotatedNode {
            name: "test.rs".to_string(),
            path: PathBuf::from("/project/test.rs"),
            node_type: NodeType::File,
            children: vec![],
            annotations: NodeAnnotations {
                defect_score: Some(0.8),
                satd_items: 2,
                dead_code_items: 3,
                ..Default::default()
            },
        };

        let display = analyzer.format_node_display(&node).unwrap();
        assert!(display.contains("🔴"));
        assert!(display.contains("📝2"));
        assert!(display.contains("💀3"));
    }

    // EXTENSION TO LANGUAGE MATCHING TESTS

    #[test]
    fn test_match_extension_to_language_rust() {
        let result = match_extension_to_language("rs").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::Rust)
        ));
    }

    #[test]
    fn test_match_extension_to_language_typescript() {
        let result = match_extension_to_language("ts").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::TypeScript)
        ));

        let result = match_extension_to_language("tsx").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::TypeScript)
        ));
    }

    #[test]
    fn test_match_extension_to_language_javascript() {
        let result = match_extension_to_language("js").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::JavaScript)
        ));

        let result = match_extension_to_language("jsx").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::JavaScript)
        ));
    }

    #[test]
    fn test_match_extension_to_language_python() {
        let result = match_extension_to_language("py").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::Python)
        ));
    }

    #[test]
    fn test_match_extension_to_language_cpp() {
        let result = match_extension_to_language("cpp").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::Cpp)
        ));

        let result = match_extension_to_language("cc").unwrap();
        assert!(matches!(
            result,
            Some(crate::services::duplicate_detector::Language::Cpp)
        ));
    }

    #[test]
    fn test_match_extension_to_language_unknown() {
        let result = match_extension_to_language("xyz").unwrap();
        assert!(result.is_none());
    }
}

mod dead_code_and_metrics_tests {
    use super::*;
    use std::path::PathBuf;

    // DEAD CODE ANALYSIS TESTS

    #[test]
    fn test_extract_function_name_if_unused_not_called() {
        let lines = vec!["fn unused_fn() {}", "}"];
        let result = extract_function_name_if_unused(&lines, "fn unused_fn() {");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_function_name_if_unused_called() {
        let lines = vec!["fn used_fn() {}", "used_fn();"];
        let result = extract_function_name_if_unused(&lines, "fn used_fn() {");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_struct_name_if_unused_not_used() {
        let lines = vec!["struct UnusedStruct {}", "let x = 5;"];
        let result = extract_struct_name_if_unused(&lines, "struct UnusedStruct {}");
        assert_eq!(result, Some("UnusedStruct".to_string()));
    }

    #[test]
    fn test_extract_struct_name_if_unused_used() {
        let lines = vec!["struct UsedStruct {}", "let x: UsedStruct = UsedStruct {};"];
        let result = extract_struct_name_if_unused(&lines, "struct UsedStruct {}");
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_rust_dead_functions() {
        let lines = vec!["fn private_fn() {}", "pub fn public_fn() {}"];
        let mut dead_functions = 0;
        let mut dead_items = Vec::new();

        analyze_rust_dead_functions(&lines, &mut dead_functions, &mut dead_items);
        assert_eq!(dead_functions, 0);
        assert_eq!(dead_items.len(), 0);
    }

    #[test]
    fn test_analyze_rust_dead_structs() {
        let lines = vec!["struct PrivateStruct {}", "pub struct PublicStruct {}"];
        let mut dead_classes = 0;
        let mut dead_items = Vec::new();

        analyze_rust_dead_structs(&lines, &mut dead_classes, &mut dead_items);
        assert_eq!(dead_classes, 1);
        assert_eq!(dead_items.len(), 1);
    }

    #[test]
    fn test_analyze_typescript_dead_functions() {
        let lines = vec![
            "function privateFunc() {}",
            "export function publicFunc() {}",
        ];
        let mut dead_functions = 0;
        let mut dead_items = Vec::new();

        analyze_typescript_dead_functions(&lines, &mut dead_functions, &mut dead_items);
        assert_eq!(dead_functions, 0);
    }

    #[test]
    fn test_analyze_python_dead_functions() {
        let lines = vec!["def _private_func():", "def public_func():"];
        let mut dead_functions = 0;
        let mut dead_items = Vec::new();

        analyze_python_dead_functions(&lines, &mut dead_functions, &mut dead_items);
        assert_eq!(dead_functions, 0);
    }

    // COMPLEXITY METRICS TESTS

    #[test]
    fn test_complexity_metrics_for_qa_default() {
        let metrics = ComplexityMetricsForQA::default();
        assert_eq!(metrics.files.len(), 0);
        assert_eq!(metrics.summary.total_files, 0);
        assert_eq!(metrics.summary.total_functions, 0);
    }

    #[test]
    fn test_file_complexity_metrics_for_qa_creation() {
        let metrics = FileComplexityMetricsForQA {
            path: PathBuf::from("/test.rs"),
            functions: vec![],
            total_cyclomatic: 10,
            total_cognitive: 15,
            total_lines: 100,
        };

        assert_eq!(metrics.total_cyclomatic, 10);
        assert_eq!(metrics.total_cognitive, 15);
        assert_eq!(metrics.total_lines, 100);
    }

    #[test]
    fn test_function_complexity_for_qa_creation() {
        let func = FunctionComplexityForQA {
            name: "test_fn".to_string(),
            cyclomatic: 5,
            cognitive: 3,
            nesting_depth: 2,
            start_line: 10,
            end_line: 20,
        };

        assert_eq!(func.name, "test_fn");
        assert_eq!(func.cyclomatic, 5);
        assert_eq!(func.cognitive, 3);
    }

    // DEFECT FACTOR AND PRIORITY TESTS

    #[test]
    fn test_priority_variants() {
        assert!(Priority::Critical != Priority::High);
        assert!(Priority::High != Priority::Medium);
        assert!(Priority::Medium != Priority::Low);
    }

    #[test]
    fn test_impact_variants() {
        let high = Impact::High;
        let medium = Impact::Medium;
        let low = Impact::Low;

        assert!(matches!(high, Impact::High));
        assert!(matches!(medium, Impact::Medium));
        assert!(matches!(low, Impact::Low));
    }

    #[test]
    fn test_technical_debt_category_variants() {
        let design = TechnicalDebtCategory::Design;
        let requirements = TechnicalDebtCategory::Requirements;
        let implementation = TechnicalDebtCategory::Implementation;
        let testing = TechnicalDebtCategory::Testing;
        let documentation = TechnicalDebtCategory::Documentation;

        assert!(matches!(design, TechnicalDebtCategory::Design));
        assert!(matches!(requirements, TechnicalDebtCategory::Requirements));
        assert!(matches!(
            implementation,
            TechnicalDebtCategory::Implementation
        ));
        assert!(matches!(testing, TechnicalDebtCategory::Testing));
        assert!(matches!(
            documentation,
            TechnicalDebtCategory::Documentation
        ));
    }

    #[test]
    fn test_confidence_level_variants() {
        let high = ConfidenceLevel::High;
        let medium = ConfidenceLevel::Medium;
        let low = ConfidenceLevel::Low;

        assert!(matches!(high, ConfidenceLevel::High));
        assert!(matches!(medium, ConfidenceLevel::Medium));
        assert!(matches!(low, ConfidenceLevel::Low));
    }

    #[test]
    fn test_cross_lang_reference_type_variants() {
        let wasm = CrossLangReferenceType::WasmBinding;
        let ffi = CrossLangReferenceType::FfiCall;
        let python = CrossLangReferenceType::PythonBinding;
        let typedef = CrossLangReferenceType::TypeDefinition;

        assert!(matches!(wasm, CrossLangReferenceType::WasmBinding));
        assert!(matches!(ffi, CrossLangReferenceType::FfiCall));
        assert!(matches!(python, CrossLangReferenceType::PythonBinding));
        assert!(matches!(typedef, CrossLangReferenceType::TypeDefinition));
    }

    // QUALITY SCORECARD TESTS

    #[test]
    fn test_quality_scorecard_creation() {
        let scorecard = QualityScorecard {
            overall_health: 85.0,
            complexity_score: 90.0,
            maintainability_index: 80.0,
            modularity_score: 75.0,
            test_coverage: Some(70.0),
            technical_debt_hours: 40.0,
        };

        assert_eq!(scorecard.overall_health, 85.0);
        assert_eq!(scorecard.test_coverage, Some(70.0));
    }

    #[test]
    fn test_quality_scorecard_default() {
        let scorecard = QualityScorecard::default();
        assert_eq!(scorecard.overall_health, 0.0);
        assert_eq!(scorecard.test_coverage, None);
    }

    // ASYNC LANGUAGE ANALYSIS TESTS

    #[tokio::test]
    async fn test_analyze_elixir_language_empty() {
        let result = analyze_elixir_language(Path::new("test.ex")).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_erlang_language_empty() {
        let result = analyze_erlang_language(Path::new("test.erl"))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_haskell_language_empty() {
        let result = analyze_haskell_language(Path::new("test.hs"))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_ocaml_language_empty() {
        let result = analyze_ocaml_language(Path::new("test.ml")).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_file_by_language_unknown() {
        let result = analyze_file_by_language(Path::new("test.xyz"), "unknown")
            .await
            .unwrap();
        assert!(result.is_empty());
    }
}
