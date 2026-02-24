    use super::*;

    // ==================== FuzzConfig Tests ====================

    #[test]
    fn test_fuzz_config_default() {
        let config = FuzzConfig::default();
        assert_eq!(config.iterations, 1000);
        assert_eq!(config.input_generator, InputGeneratorType::Random);
        assert!(config.crash_detection);
        assert_eq!(config.iteration_timeout.as_millis(), 100);
    }

    #[test]
    fn test_fuzz_config_validate_valid() {
        let config = FuzzConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fuzz_config_validate_zero_iterations() {
        let config = FuzzConfig {
            iterations: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("iterations must be > 0"));
    }

    #[test]
    fn test_fuzz_config_validate_zero_timeout() {
        let config = FuzzConfig {
            iteration_timeout: Duration::from_millis(0),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("iteration_timeout must be > 0"));
    }

    #[test]
    fn test_fuzz_config_custom_values() {
        let config = FuzzConfig {
            iterations: 500,
            input_generator: InputGeneratorType::CoverageGuided,
            crash_detection: false,
            iteration_timeout: Duration::from_millis(200),
        };
        assert_eq!(config.iterations, 500);
        assert_eq!(config.input_generator, InputGeneratorType::CoverageGuided);
        assert!(!config.crash_detection);
        assert_eq!(config.iteration_timeout.as_millis(), 200);
    }

    #[test]
    fn test_fuzz_config_serialization() {
        let config = FuzzConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"iterations\":1000"));
        assert!(json.contains("\"crash_detection\":true"));
    }

    #[test]
    fn test_fuzz_config_deserialization() {
        let json = r#"{
            "iterations": 500,
            "input_generator": "GrammarBased",
            "crash_detection": false,
            "iteration_timeout": {"secs": 0, "nanos": 50000000}
        }"#;
        let config: FuzzConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.iterations, 500);
        assert_eq!(config.input_generator, InputGeneratorType::GrammarBased);
        assert!(!config.crash_detection);
    }

    #[test]
    fn test_fuzz_config_clone() {
        let config = FuzzConfig::default();
        let cloned = config.clone();
        assert_eq!(config.iterations, cloned.iterations);
        assert_eq!(config.input_generator, cloned.input_generator);
    }

    // ==================== InputGeneratorType Tests ====================

    #[test]
    fn test_input_generator_type_random() {
        let gen_type = InputGeneratorType::Random;
        assert_eq!(gen_type, InputGeneratorType::Random);
    }

    #[test]
    fn test_input_generator_type_grammar_based() {
        let gen_type = InputGeneratorType::GrammarBased;
        assert_eq!(gen_type, InputGeneratorType::GrammarBased);
    }

    #[test]
    fn test_input_generator_type_mutation_based() {
        let gen_type = InputGeneratorType::MutationBased;
        assert_eq!(gen_type, InputGeneratorType::MutationBased);
    }

    #[test]
    fn test_input_generator_type_coverage_guided() {
        let gen_type = InputGeneratorType::CoverageGuided;
        assert_eq!(gen_type, InputGeneratorType::CoverageGuided);
    }

    #[test]
    fn test_input_generator_type_serialization() {
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::Random).unwrap(),
            "\"Random\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::GrammarBased).unwrap(),
            "\"GrammarBased\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::MutationBased).unwrap(),
            "\"MutationBased\""
        );
        assert_eq!(
            serde_json::to_string(&InputGeneratorType::CoverageGuided).unwrap(),
            "\"CoverageGuided\""
        );
    }

    #[test]
    fn test_input_generator_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<InputGeneratorType>("\"Random\"").unwrap(),
            InputGeneratorType::Random
        );
        assert_eq!(
            serde_json::from_str::<InputGeneratorType>("\"CoverageGuided\"").unwrap(),
            InputGeneratorType::CoverageGuided
        );
    }

    #[test]
    fn test_input_generator_type_copy() {
        let gen_type = InputGeneratorType::Random;
        let copied = gen_type; // Copy
        assert_eq!(gen_type, copied);
    }

    #[test]
    fn test_input_generator_type_debug() {
        let gen_type = InputGeneratorType::GrammarBased;
        let debug = format!("{:?}", gen_type);
        assert_eq!(debug, "GrammarBased");
    }

    // ==================== FuzzResult Tests ====================

    #[test]
    fn test_fuzz_result_empty() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!(!result.has_crashes());
        assert!(!result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_crashes() {
        let result = FuzzResult {
            crashes: vec!["crash1".to_string(), "crash2".to_string()],
            hangs: vec![],
            coverage_increase: 0.0,
        };
        assert!(result.has_crashes());
        assert!(!result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_hangs() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![vec![1, 2, 3], vec![4, 5, 6]],
            coverage_increase: 0.0,
        };
        assert!(!result.has_crashes());
        assert!(result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_with_both() {
        let result = FuzzResult {
            crashes: vec!["crash".to_string()],
            hangs: vec![vec![1, 2]],
            coverage_increase: 0.5,
        };
        assert!(result.has_crashes());
        assert!(result.has_hangs());
    }

    #[test]
    fn test_fuzz_result_coverage_increase() {
        let result = FuzzResult {
            crashes: vec![],
            hangs: vec![],
            coverage_increase: 0.75,
        };
        assert!((result.coverage_increase - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzz_result_serialization() {
        let result = FuzzResult {
            crashes: vec!["crash1".to_string()],
            hangs: vec![vec![1, 2, 3]],
            coverage_increase: 0.25,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"crashes\":[\"crash1\"]"));
        assert!(json.contains("\"coverage_increase\":0.25"));
    }

    #[test]
    fn test_fuzz_result_deserialization() {
        let json = r#"{"crashes":["c1","c2"],"hangs":[[1,2]],"coverage_increase":0.5}"#;
        let result: FuzzResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.crashes.len(), 2);
        assert_eq!(result.hangs.len(), 1);
        assert!((result.coverage_increase - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fuzz_result_clone() {
        let result = FuzzResult {
            crashes: vec!["crash".to_string()],
            hangs: vec![vec![1]],
            coverage_increase: 0.1,
        };
        let cloned = result.clone();
        assert_eq!(result.crashes, cloned.crashes);
        assert_eq!(result.hangs, cloned.hangs);
    }

    // ==================== FuzzMutationReport Tests ====================

    #[test]
    fn test_fuzz_mutation_report_empty() {
        let report = FuzzMutationReport {
            total_mutants: 0,
            mutants_with_crashes: 0,
            mutants_with_hangs: 0,
            execution_time: Duration::from_secs(0),
            results: vec![],
        };
        assert_eq!(report.total_mutants, 0);
        assert_eq!(report.results.len(), 0);
    }

    #[test]
    fn test_fuzz_mutation_report_with_results() {
        let report = FuzzMutationReport {
            total_mutants: 5,
            mutants_with_crashes: 2,
            mutants_with_hangs: 1,
            execution_time: Duration::from_secs(10),
            results: vec![],
        };
        assert_eq!(report.total_mutants, 5);
        assert_eq!(report.mutants_with_crashes, 2);
        assert_eq!(report.mutants_with_hangs, 1);
        assert_eq!(report.execution_time.as_secs(), 10);
    }

    #[test]
    fn test_fuzz_mutation_report_serialization() {
        let report = FuzzMutationReport {
            total_mutants: 3,
            mutants_with_crashes: 1,
            mutants_with_hangs: 0,
            execution_time: Duration::from_millis(500),
            results: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"total_mutants\":3"));
        assert!(json.contains("\"mutants_with_crashes\":1"));
    }

    #[test]
    fn test_fuzz_mutation_report_clone() {
        let report = FuzzMutationReport {
            total_mutants: 10,
            mutants_with_crashes: 2,
            mutants_with_hangs: 1,
            execution_time: Duration::from_secs(5),
            results: vec![],
        };
        let cloned = report.clone();
        assert_eq!(report.total_mutants, cloned.total_mutants);
        assert_eq!(report.mutants_with_crashes, cloned.mutants_with_crashes);
    }

