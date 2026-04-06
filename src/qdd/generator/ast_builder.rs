//! AST building logic for code generation

use crate::qdd::{CreateSpec, QualityProfile};
use anyhow::Result;

/// AST builder for creating code structures
pub struct AstBuilder {
    pub(crate) profile: QualityProfile,
}

impl AstBuilder {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(profile: QualityProfile) -> Self {
        Self { profile }
    }

    /// Build a function from specification
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build_function(&self, spec: &CreateSpec) -> Result<String> {
        let mut code = String::new();

        // Generate documentation comment
        code.push_str(&format!("/// {}\n", spec.purpose));
        code.push_str("///\n");

        // Document parameters
        for param in &spec.inputs {
            code.push_str(&format!(
                "/// * `{}` - {}\n",
                param.name,
                param
                    .description
                    .as_deref()
                    .unwrap_or("Parameter description")
            ));
        }

        code.push_str("///\n");
        code.push_str(&format!(
            "/// # Returns\n/// {}\n",
            spec.outputs
                .description
                .as_deref()
                .unwrap_or("Function return value")
        ));

        // Add doctest if required
        if self.profile.thresholds.require_doctests {
            code.push_str("///\n/// # Example\n/// ```\n");
            code.push_str(&format!("/// let result = {}(", spec.name));

            for (i, param) in spec.inputs.iter().enumerate() {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&self.generate_example_value(&param.param_type));
            }

            code.push_str(");\n/// assert!(result.is_ok());\n/// ```\n");
        }

        // Generate function signature
        code.push_str(&format!("pub fn {}(", spec.name));

        for (i, param) in spec.inputs.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&format!("{}: {}", param.name, param.param_type));
        }

        code.push_str(&format!(") -> Result<{}> {{\n", spec.outputs.param_type));

        // Generate simple implementation
        code.push_str("    // Implementation placeholder\n");
        code.push_str("    todo!(\"Implementation needed\")\n");
        code.push_str("}\n");

        Ok(code)
    }

    /// Generate example value for doctest
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn generate_example_value(&self, param_type: &str) -> String {
        debug_assert!(!param_type.is_empty(), "param_type must not be empty");
        match param_type {
            "u32" | "i32" => "42".to_string(),
            "f32" | "f64" => "3.14".to_string(),
            "String" => "\"test\".to_string()".to_string(),
            "&str" => "\"test\"".to_string(),
            "bool" => "true".to_string(),
            _ => "Default::default()".to_string(),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::qdd::{CodeType, CreateSpec, Parameter};

    #[test]
    fn test_ast_builder_function_generation() {
        let profile = QualityProfile::standard();
        let builder = AstBuilder::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "test_func".to_string(),
            purpose: "Test function".to_string(),
            inputs: vec![Parameter {
                name: "input".to_string(),
                param_type: "String".to_string(),
                description: None,
            }],
            outputs: Parameter {
                name: "output".to_string(),
                param_type: "String".to_string(),
                description: None,
            },
        };

        let code = builder.build_function(&spec).unwrap();

        assert!(code.contains("/// Test function"));
        assert!(code.contains("pub fn test_func(input: String) -> Result<String>"));
        assert!(code.contains("todo!"));
    }

    #[test]
    fn test_ast_builder_with_doctests() {
        let mut profile = QualityProfile::standard();
        profile.thresholds.require_doctests = true;
        let builder = AstBuilder::new(profile);

        let spec = create_multi_param_spec("my_func");
        let code = builder.build_function(&spec).unwrap();

        // Should contain doctest example
        assert!(code.contains("# Example"));
        assert!(code.contains("/// ```"));
        assert!(code.contains("let result = my_func("));
        assert!(code.contains("assert!(result.is_ok());"));
    }

    #[test]
    fn test_ast_builder_without_doctests() {
        let mut profile = QualityProfile::standard();
        profile.thresholds.require_doctests = false;
        let builder = AstBuilder::new(profile);

        let spec = create_minimal_spec("simple");
        let code = builder.build_function(&spec).unwrap();

        // Should not contain doctest example
        assert!(!code.contains("# Example"));
    }

    #[test]
    fn test_ast_builder_parameter_documentation() {
        let profile = QualityProfile::standard();
        let builder = AstBuilder::new(profile);

        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "documented_fn".to_string(),
            purpose: "A well-documented function".to_string(),
            inputs: vec![
                Parameter {
                    name: "input".to_string(),
                    param_type: "String".to_string(),
                    description: Some("The input string".to_string()),
                },
                Parameter {
                    name: "flag".to_string(),
                    param_type: "bool".to_string(),
                    description: None, // No description
                },
            ],
            outputs: Parameter {
                name: "output".to_string(),
                param_type: "i32".to_string(),
                description: Some("The computed result".to_string()),
            },
        };

        let code = builder.build_function(&spec).unwrap();

        // Check documentation
        assert!(code.contains("/// A well-documented function"));
        assert!(code.contains("/// * `input` - The input string"));
        assert!(code.contains("/// * `flag` - Parameter description")); // Default description
        assert!(code.contains("/// # Returns"));
        assert!(code.contains("/// The computed result"));
    }

    #[test]
    fn test_generate_example_value_all_types() {
        let profile = QualityProfile::standard();
        let builder = AstBuilder::new(profile);

        assert_eq!(builder.generate_example_value("u32"), "42");
        assert_eq!(builder.generate_example_value("i32"), "42");
        assert_eq!(builder.generate_example_value("f32"), "3.14");
        assert_eq!(builder.generate_example_value("f64"), "3.14");
        assert_eq!(
            builder.generate_example_value("String"),
            "\"test\".to_string()"
        );
        assert_eq!(builder.generate_example_value("&str"), "\"test\"");
        assert_eq!(builder.generate_example_value("bool"), "true");
        assert_eq!(
            builder.generate_example_value("CustomType"),
            "Default::default()"
        );
        assert_eq!(
            builder.generate_example_value("Vec<u8>"),
            "Default::default()"
        );
    }

    // Test helpers
    fn create_minimal_spec(name: &str) -> CreateSpec {
        debug_assert!(!name.is_empty(), "name must not be empty");
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

    fn create_multi_param_spec(name: &str) -> CreateSpec {
        debug_assert!(!name.is_empty(), "name must not be empty");
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
        fn generated_function_contains_function_name(
            name in "[a-z][a-z0-9_]{0,20}"
        ) {
            let profile = QualityProfile::standard();
            let builder = AstBuilder::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: name.clone(),
                purpose: "Test purpose".to_string(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let code = builder.build_function(&spec).unwrap();
            prop_assert!(
                code.contains(&format!("pub fn {}", name)),
                "Generated code should contain function name: {}",
                name
            );
        }
    }
}
