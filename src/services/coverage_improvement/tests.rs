#![cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);
        assert_eq!(service.config.target_coverage, 95.0);
    }

    #[test]
    fn test_config_default_values() {
        let config = CoverageImprovementConfig::default();
        assert_eq!(config.project_path, PathBuf::from("."));
        assert_eq!(config.target_coverage, 95.0);
        assert_eq!(config.max_iterations, 10);
        assert!(!config.fast_mode);
        assert_eq!(config.mutation_threshold, 80.0);
        assert!(config.focus_patterns.is_empty());
        assert!(config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_config_custom_values() {
        let config = CoverageImprovementConfig {
            project_path: PathBuf::from("/custom/path"),
            target_coverage: 85.0,
            max_iterations: 5,
            fast_mode: true,
            mutation_threshold: 70.0,
            focus_patterns: vec!["src/**/*.rs".to_string()],
            exclude_patterns: vec!["**/tests/**".to_string()],
            max_targets: 10,
        };

        assert_eq!(config.project_path, PathBuf::from("/custom/path"));
        assert_eq!(config.target_coverage, 85.0);
        assert_eq!(config.max_iterations, 5);
        assert!(config.fast_mode);
        assert_eq!(config.mutation_threshold, 70.0);
        assert_eq!(config.focus_patterns.len(), 1);
        assert_eq!(config.exclude_patterns.len(), 1);
    }

    #[test]
    fn test_iteration_report_serialization() {
        let report = IterationReport {
            iteration: 1,
            files_targeted: vec![PathBuf::from("src/lib.rs")],
            tests_generated: 5,
            coverage_gain: 2.5,
            mutation_score: 85.0,
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: IterationReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.iteration, 1);
        assert_eq!(deserialized.files_targeted.len(), 1);
        assert_eq!(deserialized.tests_generated, 5);
        assert!((deserialized.coverage_gain - 2.5).abs() < f64::EPSILON);
        assert!((deserialized.mutation_score - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coverage_improvement_report_serialization() {
        let report = CoverageImprovementReport {
            baseline_coverage: 50.0,
            target_coverage: 95.0,
            final_coverage: 75.0,
            iterations: vec![
                IterationReport {
                    iteration: 1,
                    files_targeted: vec![PathBuf::from("src/lib.rs")],
                    tests_generated: 5,
                    coverage_gain: 12.5,
                    mutation_score: 85.0,
                },
                IterationReport {
                    iteration: 2,
                    files_targeted: vec![PathBuf::from("src/main.rs")],
                    tests_generated: 3,
                    coverage_gain: 12.5,
                    mutation_score: 90.0,
                },
            ],
            success: false,
            stop_reason: "Max iterations reached".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: CoverageImprovementReport = serde_json::from_str(&json).unwrap();

        assert!((deserialized.baseline_coverage - 50.0).abs() < f64::EPSILON);
        assert!((deserialized.target_coverage - 95.0).abs() < f64::EPSILON);
        assert!((deserialized.final_coverage - 75.0).abs() < f64::EPSILON);
        assert_eq!(deserialized.iterations.len(), 2);
        assert!(!deserialized.success);
        assert_eq!(deserialized.stop_reason, "Max iterations reached");
    }

    #[test]
    fn test_extract_file_path_from_line_rs_extension() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let result = service.extract_file_path_from_line("src/lib.rs 10 errors");
        assert_eq!(result, Some(PathBuf::from("src/lib.rs")));
    }

    #[test]
    fn test_extract_file_path_from_line_toml_extension() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let result = service.extract_file_path_from_line("Checking Cargo.toml for updates");
        assert_eq!(result, Some(PathBuf::from("Cargo.toml")));
    }

    #[test]
    fn test_extract_file_path_from_line_md_extension() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let result = service.extract_file_path_from_line("Updated README.md with new docs");
        assert_eq!(result, Some(PathBuf::from("README.md")));
    }

    #[test]
    fn test_extract_file_path_from_line_no_match() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let result = service.extract_file_path_from_line("No file path here");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_file_path_from_line_empty() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let result = service.extract_file_path_from_line("");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_files_from_json_with_file_key() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!({
            "file": "src/lib.rs",
            "complexity": 10
        });

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.5);

        assert_eq!(file_scores.len(), 1);
        assert!(
            (file_scores.get(&PathBuf::from("src/lib.rs")).unwrap() - 0.5).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_extract_files_from_json_with_path_key() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!({
            "path": "src/main.rs",
            "lines": 100
        });

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.3);

        assert_eq!(file_scores.len(), 1);
        assert!(
            (file_scores.get(&PathBuf::from("src/main.rs")).unwrap() - 0.3).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_extract_files_from_json_with_file_path_key() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!({
            "file_path": "src/utils.rs",
            "errors": 5
        });

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.2);

        assert_eq!(file_scores.len(), 1);
        assert!(
            (file_scores.get(&PathBuf::from("src/utils.rs")).unwrap() - 0.2).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_extract_files_from_json_nested_objects() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!({
            "results": {
                "file": "src/nested.rs",
                "inner": {
                    "path": "src/deeper.rs"
                }
            }
        });

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 1.0);

        assert_eq!(file_scores.len(), 2);
        assert!(file_scores.contains_key(&PathBuf::from("src/nested.rs")));
        assert!(file_scores.contains_key(&PathBuf::from("src/deeper.rs")));
    }

    #[test]
    fn test_extract_files_from_json_array() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!([
            {"file": "src/a.rs"},
            {"file": "src/b.rs"},
            {"file": "src/c.rs"}
        ]);

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.4);

        assert_eq!(file_scores.len(), 3);
        for (_, score) in &file_scores {
            assert!((*score - 0.4).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_extract_files_from_json_accumulates_scores() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!([
            {"file": "src/lib.rs"},
            {"path": "src/lib.rs"},
            {"file_path": "src/lib.rs"}
        ]);

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.2);

        assert_eq!(file_scores.len(), 1);
        // Score should be 0.2 * 3 = 0.6
        assert!(
            (*file_scores.get(&PathBuf::from("src/lib.rs")).unwrap() - 0.6).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_extract_files_from_json_empty_object() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json: serde_json::Value = serde_json::json!({});

        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.5);

        assert!(file_scores.is_empty());
    }

    #[test]
    fn test_extract_files_from_json_primitive_values() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        // Test with string value (not in object)
        let json: serde_json::Value = serde_json::json!("src/lib.rs");
        let mut file_scores = std::collections::HashMap::new();
        service.extract_files_from_json(&json, &mut file_scores, 0.5);
        assert!(file_scores.is_empty());

        // Test with number
        let json: serde_json::Value = serde_json::json!(42);
        service.extract_files_from_json(&json, &mut file_scores, 0.5);
        assert!(file_scores.is_empty());

        // Test with boolean
        let json: serde_json::Value = serde_json::json!(true);
        service.extract_files_from_json(&json, &mut file_scores, 0.5);
        assert!(file_scores.is_empty());

        // Test with null
        let json: serde_json::Value = serde_json::json!(null);
        service.extract_files_from_json(&json, &mut file_scores, 0.5);
        assert!(file_scores.is_empty());
    }

    #[test]
    fn test_parse_and_score_json_input() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let json_output = r#"{"files": [{"file": "src/lib.rs"}, {"file": "src/main.rs"}]}"#;
        let mut file_scores = std::collections::HashMap::new();

        let result = service.parse_and_score(json_output, &mut file_scores, 0.5);
        assert!(result.is_ok());
        assert_eq!(file_scores.len(), 2);
    }

    #[test]
    fn test_parse_and_score_text_input() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let text_output = "Analyzing src/lib.rs\nFound issues in Cargo.toml\nProcessed README.md";
        let mut file_scores = std::collections::HashMap::new();

        let result = service.parse_and_score(text_output, &mut file_scores, 0.3);
        assert!(result.is_ok());
        assert_eq!(file_scores.len(), 3);
    }

    #[test]
    fn test_parse_and_score_empty_input() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let mut file_scores = std::collections::HashMap::new();

        let result = service.parse_and_score("", &mut file_scores, 0.5);
        assert!(result.is_ok());
        assert!(file_scores.is_empty());
    }

    #[test]
    fn test_generate_strategy_for_type_integers() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        // Parse type i32
        let ty: syn::Type = syn::parse_str("i32").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i32>()");

        // Parse type i64
        let ty: syn::Type = syn::parse_str("i64").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i64>()");

        // Parse type u32
        let ty: syn::Type = syn::parse_str("u32").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<u32>()");

        // Parse type usize
        let ty: syn::Type = syn::parse_str("usize").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<usize>()");
    }

    #[test]
    fn test_generate_strategy_for_type_floats() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("f32").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<f32>()");

        let ty: syn::Type = syn::parse_str("f64").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<f64>()");
    }

    #[test]
    fn test_generate_strategy_for_type_bool_char() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("bool").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<bool>()");

        let ty: syn::Type = syn::parse_str("char").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<char>()");
    }

    #[test]
    fn test_generate_strategy_for_type_string() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("String").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), r#"".*""#);
    }

    #[test]
    fn test_generate_strategy_for_type_vec_option() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("Vec<i32>").unwrap();
        assert_eq!(
            service.generate_strategy_for_type(&ty),
            "prop::collection::vec(any::<i32>(), 0..100)"
        );

        let ty: syn::Type = syn::parse_str("Option<String>").unwrap();
        assert_eq!(
            service.generate_strategy_for_type(&ty),
            "prop::option::of(any::<i32>())"
        );
    }

    #[test]
    fn test_generate_strategy_for_type_pathbuf() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("PathBuf").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), r#""[a-z0-9/]+""#);
    }

    #[test]
    fn test_generate_strategy_for_type_reference() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("&str").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), r#"".*""#);

        let ty: syn::Type = syn::parse_str("&i32").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i32>()");
    }

    #[test]
    fn test_generate_strategy_for_type_unknown() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("CustomType").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i32>()");
    }

    #[test]
    fn test_extract_public_functions() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            /// Public fn.
            pub fn public_fn() {}
            fn private_fn() {}
            /// Another public.
            pub fn another_public() -> i32 { 42 }
            pub(crate) fn crate_public() {}
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        // Should find 2 public functions (pub fn, not pub(crate))
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].sig.ident.to_string(), "public_fn");
        assert_eq!(functions[1].sig.ident.to_string(), "another_public");
    }

    #[test]
    fn test_extract_public_functions_empty() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            fn private_fn() {}
            fn another_private() {}
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        assert!(functions.is_empty());
    }

    #[test]
    fn test_generate_proptest_module_no_params() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = "pub fn no_args() {}";
        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);
        let target = PathBuf::from("src/test.rs");

        let module = service
            .generate_proptest_module(&target, &functions)
            .unwrap();

        assert!(module.contains("use proptest::prelude::*;"));
        assert!(module.contains("fn proptest_no_args()"));
        assert!(module.contains("let _result = no_args();"));
    }

    #[test]
    fn test_generate_proptest_module_with_params() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = "pub fn with_args(x: i32, y: String) -> bool { true }";
        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);
        let target = PathBuf::from("src/test.rs");

        let module = service
            .generate_proptest_module(&target, &functions)
            .unwrap();

        assert!(module.contains("proptest!"));
        assert!(module.contains("fn proptest_with_args"));
        assert!(module.contains("x in any::<i32>()"));
        assert!(module.contains("let _result = with_args(x, y);"));
    }

    #[test]
    fn test_find_makefile_directory_current() {
        // Create a temporary directory with a Makefile
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join("Makefile"), "all:\n\techo test").unwrap();

        let config = CoverageImprovementConfig {
            project_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let service = CoverageImprovementService::new(config);

        let result = service.find_makefile_directory();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_makefile_directory_parent() {
        // Create parent with Makefile and child without
        let parent_dir = tempfile::tempdir().unwrap();
        std::fs::write(parent_dir.path().join("Makefile"), "all:\n\techo test").unwrap();

        let child_dir = parent_dir.path().join("child");
        std::fs::create_dir(&child_dir).unwrap();

        let config = CoverageImprovementConfig {
            project_path: child_dir.clone(),
            ..Default::default()
        };
        let service = CoverageImprovementService::new(config);

        let result = service.find_makefile_directory();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_makefile_directory_not_found() {
        // Create directory without Makefile anywhere in hierarchy
        let temp_dir = tempfile::tempdir().unwrap();
        let deep_child = temp_dir
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("e")
            .join("f");
        std::fs::create_dir_all(&deep_child).unwrap();

        let config = CoverageImprovementConfig {
            project_path: deep_child,
            ..Default::default()
        };
        let service = CoverageImprovementService::new(config);

        let result = service.find_makefile_directory();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find Makefile"));
    }

    #[tokio::test]
    #[ignore = "Integration test - requires Makefile and make coverage"]
    async fn test_already_at_target_coverage() {
        let config = CoverageImprovementConfig {
            target_coverage: 45.0, // Lower than baseline (49.87%)
            ..Default::default()
        };
        let service = CoverageImprovementService::new(config);
        let report = service.improve_coverage().await.expect("internal error");

        assert!(report.success);
        assert_eq!(report.iterations.len(), 0);
        assert!(report.stop_reason.contains("Already at target"));
    }

    #[tokio::test]
    #[ignore = "Integration test - requires Makefile and make coverage"]
    async fn test_improvement_iterations() {
        let config = CoverageImprovementConfig {
            target_coverage: 95.0,
            max_iterations: 3,
            ..Default::default()
        };
        let service = CoverageImprovementService::new(config);
        let report = service.improve_coverage().await.expect("internal error");

        // Should run some iterations
        assert!(!report.iterations.is_empty());
        assert!(report.iterations.len() <= 3);
    }
}
