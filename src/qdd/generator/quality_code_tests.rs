#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::qdd::Parameter;

    #[tokio::test]
    async fn test_create_simple_function() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "add_numbers".to_string(),
            purpose: "Adds two numbers together".to_string(),
            inputs: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: "u32".to_string(),
                    description: Some("First number".to_string()),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: "u32".to_string(),
                    description: Some("Second number".to_string()),
                },
            ],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "u32".to_string(),
                description: Some("Sum of the numbers".to_string()),
            },
        };

        let result = generator.create(&spec).await.unwrap();

        // Verify code was generated
        assert!(!result.code.is_empty());
        assert!(result.code.contains("pub fn add_numbers"));
        assert!(result.code.contains("a: u32, b: u32"));

        // Verify tests were generated
        assert!(!result.tests.is_empty());
        assert!(result.tests.contains("#[test]"));
        assert!(result.tests.contains("test_add_numbers_basic"));

        // Verify documentation was generated
        assert!(!result.documentation.is_empty());
        assert!(result.documentation.contains("# add_numbers"));

        // Verify quality metrics
        assert!(result.quality_score.overall > 80.0);
        // Complexity is always non-negative for u32
        // Doctest flag is valid in either state
    }

    #[test]
    fn test_complexity_estimation() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let simple_code = "fn simple() { println!(\"hello\"); }";
        assert_eq!(generator.estimate_complexity(simple_code), 1);

        let complex_code = r#"
        fn complex(x: i32) -> i32 {
            if x > 0 {
                if x < 10 {
                    for i in 0..x {
                        match i {
                            0 => return 1,
                            _ => continue,
                        }
                    }
                }
            }
            while x > 5 {
                break;
            }
            x
        }"#;

        let complexity = generator.estimate_complexity(complex_code);
        assert!(complexity > 5); // Should detect multiple branches
    }

    #[test]
    fn test_quality_code_generator_new_with_all_profiles() {
        // Test with extreme profile
        let extreme = QualityProfile::extreme();
        let gen_extreme = QualityCodeGenerator::new(extreme);
        assert!(gen_extreme.profile.thresholds.require_property_tests);

        // Test with standard profile
        let standard = QualityProfile::standard();
        let gen_standard = QualityCodeGenerator::new(standard);
        assert!(!gen_standard.profile.thresholds.require_property_tests);

        // Test with relaxed profile
        let relaxed = QualityProfile::relaxed();
        let gen_relaxed = QualityCodeGenerator::new(relaxed);
        assert!(!gen_relaxed.profile.thresholds.zero_satd);
    }

    #[tokio::test]
    async fn test_create_module() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let spec = create_module_spec("test_module");
        let result = generator.create(&spec).await.unwrap();

        // Verify module structure
        assert!(result.code.contains("//!"));
        assert!(result.code.contains("pub mod test_module"));
        assert!(result.code.contains("initialize()"));

        // Verify tests and documentation exist
        assert!(!result.tests.is_empty());
        assert!(!result.documentation.is_empty());
    }

    #[tokio::test]
    async fn test_create_service() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let spec = create_service_spec("MyTest");
        let result = generator.create(&spec).await.unwrap();

        // Verify service structure
        assert!(result.code.contains("pub struct MyTestService"));
        assert!(result.code.contains("ServiceConfig"));
        assert!(result.code.contains("pub fn new(config: ServiceConfig)"));
        assert!(result.code.contains("pub async fn start(&self)"));
    }

    #[tokio::test]
    async fn test_create_test() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let spec = create_test_spec("my_feature");
        let result = generator.create(&spec).await.unwrap();

        // Verify test module structure
        assert!(result.code.contains("#[cfg(test)]"));
        assert!(result.code.contains("mod my_feature"));
        assert!(result.code.contains("#[test]"));
        assert!(result.code.contains("fn test_my_feature()"));
        assert!(result.code.contains("property-testing"));
    }

    #[test]
    fn test_estimate_complexity_edge_cases() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        // Empty code
        let empty = "";
        assert_eq!(generator.estimate_complexity(empty), 1);

        // Code with just ifs
        let many_ifs = "if x {} if y {} if z {} if a {} if b {}";
        let complexity = generator.estimate_complexity(many_ifs);
        assert_eq!(complexity, 6); // 1 base + 5 ifs

        // Code with just loops
        let loops_only = "for x {} while y {} for z {} while a {}";
        let complexity = generator.estimate_complexity(loops_only);
        assert_eq!(complexity, 5); // 1 base + 4 loops

        // Code with just matches
        let matches_only = "match x {} match y {} match z {}";
        let complexity = generator.estimate_complexity(matches_only);
        assert_eq!(complexity, 4); // 1 base + 3 matches

        // Mixed constructs
        let mixed = "if x { match y { _ => for z in {} } while true {} }";
        let complexity = generator.estimate_complexity(mixed);
        assert!(complexity >= 4); // At least 1 base + 1 if + 1 match + 1 for + 1 while
    }

    #[test]
    fn test_calculate_quality_score_various_complexities() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        // Simple code - high score
        let simple = "fn foo() {}";
        let score = generator.calculate_quality_score(simple).unwrap();
        assert!(score.overall >= 90.0);
        assert_eq!(score.coverage, 100.0);

        // Complex code - lower score
        let complex =
            "if a {} if b {} if c {} if d {} if e {} match x {} for i in {} while true {}";
        let score = generator.calculate_quality_score(complex).unwrap();
        assert!(score.overall < 90.0);
        assert!(score.complexity > 5);
    }

    #[test]
    fn test_calculate_metrics() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        // Test with TODO markers
        let code_with_todos = "fn foo() { // TODO: fix this\n// TODO: and this }";
        let tests_with_proptest = "proptest! { #[test] fn prop() {} }";

        let metrics = generator
            .calculate_metrics(code_with_todos, tests_with_proptest)
            .unwrap();
        assert_eq!(metrics.satd_count, 2); // 2 TODOs
        assert!(metrics.has_property_tests);

        // Test without TODOs
        let clean_code = "fn foo() { println!(\"hello\"); }";
        let simple_tests = "#[test] fn test() {}";

        let metrics = generator
            .calculate_metrics(clean_code, simple_tests)
            .unwrap();
        assert_eq!(metrics.satd_count, 0);
        assert!(!metrics.has_property_tests);
    }

    #[test]
    fn test_needs_decomposition() {
        let profile = QualityProfile::extreme(); // max_complexity = 5
        let generator = QualityCodeGenerator::new(profile);

        // Simple code - no decomposition needed
        let simple = "fn foo() {}";
        assert!(!generator.needs_decomposition(simple).unwrap());

        // Complex code - decomposition needed
        let complex = "if a {} if b {} if c {} if d {} if e {} match x {}";
        assert!(generator.needs_decomposition(complex).unwrap());
    }

    #[test]
    fn test_decompose_function() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let code = "fn complex() { /* lots of code */ }".to_string();
        let result = generator.decompose_function(code.clone()).unwrap();

        // Current implementation returns original code
        assert_eq!(result, code);
    }

    #[test]
    fn test_enhance_with_features() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let base_code = "pub struct Foo {}";
        let features = vec!["validate".to_string(), "serialize".to_string()];

        let enhanced = generator
            .enhance_with_features(base_code, &features)
            .unwrap();

        assert!(enhanced.contains("pub struct Foo {}"));
        assert!(enhanced.contains("// Feature: validate"));
        assert!(enhanced.contains("// Feature: serialize"));
        assert!(enhanced.contains("pub fn validate(&self)"));
        assert!(enhanced.contains("pub fn serialize(&self)"));
    }

    #[test]
    fn test_generate_tests_wrapper() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let code = "fn foo() {}";
        let tests = generator.generate_tests(code).unwrap();

        assert!(tests.contains("#[cfg(test)]"));
        assert!(tests.contains("mod tests"));
        assert!(tests.contains("#[test]"));
    }

    #[test]
    fn test_generate_documentation_wrapper() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let code = "fn foo() {} fn bar() {}";
        let docs = generator.generate_documentation(code).unwrap();

        assert!(docs.contains("# Generated Code Documentation"));
        assert!(docs.contains("## Functions"));
    }

    #[test]
    fn test_generate_feature_code() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let feature_code = generator.generate_feature_code("My Feature").unwrap();
        assert!(feature_code.contains("pub fn my_feature(&self)"));
        assert!(feature_code.contains("// Implementation for My Feature"));
    }

    #[tokio::test]
    async fn test_full_workflow_function_creation() {
        let profile = QualityProfile::standard();
        let generator = QualityCodeGenerator::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "process_data".to_string(),
            purpose: "Processes input data and returns result".to_string(),
            inputs: vec![Parameter {
                name: "data".to_string(),
                param_type: "String".to_string(),
                description: Some("Input data to process".to_string()),
            }],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "String".to_string(),
                description: Some("Processed output".to_string()),
            },
        };

        let result = generator.create(&spec).await.unwrap();

        // Verify complete output
        assert!(!result.code.is_empty());
        assert!(!result.tests.is_empty());
        assert!(!result.documentation.is_empty());
        assert!(!result.rollback_plan.checkpoints.is_empty());

        // Verify rollback plan has checkpoints
        assert!(result
            .rollback_plan
            .checkpoints
            .iter()
            .any(|c| c.step == "initial_generation"));
    }

    #[tokio::test]
    async fn test_quality_thresholds_enforcement() {
        let mut profile = QualityProfile::extreme();
        // Set impossibly strict thresholds
        profile.thresholds.max_complexity = 0;
        profile.thresholds.min_coverage = 100;

        let generator = QualityCodeGenerator::new(profile);
        let spec = create_minimal_spec("strict_test");

        let result = generator.create(&spec).await;

        // Should fail because metrics cannot meet thresholds
        assert!(result.is_err());
    }

    // Test helpers
    fn create_minimal_spec(name: &str) -> CreateSpec {
        CreateSpec {
            code_type: CodeType::Function,
            name: name.to_string(),
            purpose: "Minimal test function".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "()".to_string(),
                description: None,
            },
        }
    }

    fn create_module_spec(name: &str) -> CreateSpec {
        CreateSpec {
            code_type: CodeType::Module,
            name: name.to_string(),
            purpose: "Test module".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "".to_string(),
                param_type: "()".to_string(),
                description: None,
            },
        }
    }

    fn create_service_spec(name: &str) -> CreateSpec {
        CreateSpec {
            code_type: CodeType::Service,
            name: name.to_string(),
            purpose: "Test service".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "".to_string(),
                param_type: "()".to_string(),
                description: None,
            },
        }
    }

    fn create_test_spec(name: &str) -> CreateSpec {
        CreateSpec {
            code_type: CodeType::Test,
            name: name.to_string(),
            purpose: "Test suite for validation".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "".to_string(),
                param_type: "()".to_string(),
                description: None,
            },
        }
    }
}
