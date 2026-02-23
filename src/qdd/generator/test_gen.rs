//! Test generation logic

use crate::qdd::{CreateSpec, QualityProfile};
use anyhow::Result;

/// Test generator for comprehensive test coverage
pub struct TestGenerator {
    pub(crate) profile: QualityProfile,
}

impl TestGenerator {
    #[must_use]
    pub fn new(profile: QualityProfile) -> Self {
        Self { profile }
    }

    /// Generate comprehensive tests for a function
    pub fn generate_for_function(&self, _code: &str, spec: &CreateSpec) -> Result<String> {
        let mut tests = String::new();

        tests.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");

        // Generate basic unit test
        tests.push_str(&format!(
            "    #[test]\n    fn test_{}_basic() {{\n",
            spec.name
        ));
        tests.push_str("        // Test basic functionality\n");
        tests.push_str(&format!("        let result = {}(", spec.name));

        for (i, param) in spec.inputs.iter().enumerate() {
            if i > 0 {
                tests.push_str(", ");
            }
            tests.push_str(&self.generate_test_value(&param.param_type));
        }

        tests.push_str(");\n");
        tests.push_str("        assert!(result.is_ok());\n");
        tests.push_str("    }\n\n");

        // Generate error test
        tests.push_str(&format!(
            "    #[test]\n    fn test_{}_edge_cases() {{\n",
            spec.name
        ));
        tests.push_str("        // Test edge cases and error conditions\n");
        tests.push_str("        // Add specific edge case tests here\n");
        tests.push_str("    }\n\n");

        // Generate property test if required
        if self.profile.thresholds.require_property_tests {
            tests.push_str("    use proptest::prelude::*;\n\n");
            tests.push_str(&format!(
                "    proptest! {{\n        #[test]\n        fn prop_{}_always_valid(\n",
                spec.name
            ));

            for param in &spec.inputs {
                tests.push_str(&format!(
                    "            {} in any::<{}>(),\n",
                    param.name, param.param_type
                ));
            }

            tests.push_str("        ) {\n");
            tests.push_str(&format!("            let result = {}(", spec.name));

            for (i, param) in spec.inputs.iter().enumerate() {
                if i > 0 {
                    tests.push_str(", ");
                }
                tests.push_str(&param.name);
            }

            tests.push_str(");\n");
            tests.push_str(
                "            // Property: function should handle all inputs gracefully\n",
            );
            tests.push_str("            // Add property assertions here\n");
            tests.push_str("        }\n    }\n\n");
        }

        tests.push_str("}\n");

        Ok(tests)
    }

    /// Generate test value for parameter type
    pub(crate) fn generate_test_value(&self, param_type: &str) -> String {
        match param_type {
            "u32" | "i32" => "42".to_string(),
            "f32" | "f64" => "3.14".to_string(),
            "String" => "\"test\".to_string()".to_string(),
            "&str" => "\"test\"".to_string(),
            "bool" => "true".to_string(),
            _ => "Default::default()".to_string(),
        }
    }

    /// Generate tests for any code
    pub fn generate_tests(&self, _code: &str) -> Result<String> {
        let mut tests = String::new();

        tests.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
        tests.push_str("    #[test]\n    fn test_generated_code() {\n");
        tests.push_str("        // Auto-generated test for code quality\n");
        tests.push_str("        assert!(true); // Placeholder\n");
        tests.push_str("    }\n");

        // Add property tests if required
        if self.profile.thresholds.require_property_tests {
            tests.push_str("\n    #[cfg(feature = \"property-testing\")]\n");
            tests.push_str("    mod property_tests {\n");
            tests.push_str("        use super::*;\n");
            tests.push_str("        use proptest::prelude::*;\n\n");
            tests.push_str("        proptest! {\n");
            tests.push_str("            #[test]\n");
            tests.push_str("            fn property_test_code(input in any::<u32>()) {\n");
            tests.push_str("                // Property-based test\n");
            tests.push_str("                assert!(input == input);\n");
            tests.push_str("            }\n");
            tests.push_str("        }\n");
            tests.push_str("    }\n");
        }

        tests.push_str("}\n");
        Ok(tests)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::qdd::{CodeType, CreateSpec, Parameter};

    #[test]
    fn test_test_generator_basic_tests() {
        let profile = QualityProfile::standard();
        let generator = TestGenerator::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "test_func".to_string(),
            purpose: "Test function".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "String".to_string(),
                description: None,
            },
        };

        let tests = generator.generate_for_function("", &spec).unwrap();

        assert!(tests.contains("#[cfg(test)]"));
        assert!(tests.contains("test_test_func_basic"));
        assert!(tests.contains("test_test_func_edge_cases"));
    }

    #[test]
    fn test_test_generator_property_tests() {
        let mut profile = QualityProfile::standard();
        profile.thresholds.require_property_tests = true;
        let generator = TestGenerator::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "test_func".to_string(),
            purpose: "Test function".to_string(),
            inputs: vec![Parameter {
                name: "x".to_string(),
                param_type: "u32".to_string(),
                description: None,
            }],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "u32".to_string(),
                description: None,
            },
        };

        let tests = generator.generate_for_function("", &spec).unwrap();

        assert!(tests.contains("proptest!"));
        assert!(tests.contains("prop_test_func_always_valid"));
    }

    #[test]
    fn test_test_generator_multiple_params() {
        let profile = QualityProfile::standard();
        let generator = TestGenerator::new(profile);

        let spec = create_multi_param_spec("multi_func");
        let tests = generator.generate_for_function("", &spec).unwrap();

        // Check parameter usage in generated tests
        assert!(tests.contains("test_multi_func_basic"));
        assert!(tests.contains("42")); // u32 example value
    }

    #[test]
    fn test_test_generator_with_multiple_property_params() {
        let mut profile = QualityProfile::standard();
        profile.thresholds.require_property_tests = true;
        let generator = TestGenerator::new(profile);

        let spec = create_multi_param_spec("prop_func");
        let tests = generator.generate_for_function("", &spec).unwrap();

        // Check property test generation
        assert!(tests.contains("proptest!"));
        assert!(tests.contains("x in any::<u32>()"));
        assert!(tests.contains("y in any::<i32>()"));
        assert!(tests.contains("z in any::<f64>()"));
    }

    #[test]
    fn test_test_generator_generic_tests() {
        let profile = QualityProfile::standard();
        let generator = TestGenerator::new(profile);

        let code = "pub struct MyStruct {}";
        let tests = generator.generate_tests(code).unwrap();

        assert!(tests.contains("#[cfg(test)]"));
        assert!(tests.contains("test_generated_code"));
    }

    #[test]
    fn test_test_generator_with_property_feature() {
        let mut profile = QualityProfile::standard();
        profile.thresholds.require_property_tests = true;
        let generator = TestGenerator::new(profile);

        let code = "pub fn example() {}";
        let tests = generator.generate_tests(code).unwrap();

        assert!(tests.contains("#[cfg(feature = \"property-testing\")]"));
        assert!(tests.contains("mod property_tests"));
        assert!(tests.contains("proptest!"));
    }

    #[test]
    fn test_generate_test_value_all_types() {
        let profile = QualityProfile::standard();
        let generator = TestGenerator::new(profile);

        assert_eq!(generator.generate_test_value("u32"), "42");
        assert_eq!(generator.generate_test_value("i32"), "42");
        assert_eq!(generator.generate_test_value("f32"), "3.14");
        assert_eq!(generator.generate_test_value("f64"), "3.14");
        assert_eq!(
            generator.generate_test_value("String"),
            "\"test\".to_string()"
        );
        assert_eq!(generator.generate_test_value("&str"), "\"test\"");
        assert_eq!(generator.generate_test_value("bool"), "true");
        assert_eq!(
            generator.generate_test_value("UnknownType"),
            "Default::default()"
        );
    }

    // Test helpers
    fn create_multi_param_spec(name: &str) -> CreateSpec {
        CreateSpec {
            code_type: CodeType::Function,
            name: name.to_string(),
            purpose: "Multi-parameter function".to_string(),
            inputs: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: "u32".to_string(),
                    description: Some("First parameter".to_string()),
                },
                Parameter {
                    name: "y".to_string(),
                    param_type: "i32".to_string(),
                    description: Some("Second parameter".to_string()),
                },
                Parameter {
                    name: "z".to_string(),
                    param_type: "f64".to_string(),
                    description: None,
                },
            ],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "String".to_string(),
                description: Some("Computed result".to_string()),
            },
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::qdd::{CodeType, CreateSpec, Parameter};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn generated_tests_contain_test_function_name(
            name in "[a-z][a-z0-9_]{0,15}"
        ) {
            let profile = QualityProfile::standard();
            let generator = TestGenerator::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: name.clone(),
                purpose: "Test".to_string(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let tests = generator.generate_for_function("", &spec).unwrap();
            prop_assert!(
                tests.contains(&format!("test_{}", name)),
                "Generated tests should contain test function name: test_{}",
                name
            );
        }
    }
}
