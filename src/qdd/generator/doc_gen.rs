//! Documentation generation logic

use crate::qdd::{CreateSpec, QualityProfile};
use anyhow::Result;

/// Documentation generator
pub struct DocGenerator {
    pub(crate) profile: QualityProfile,
}

impl DocGenerator {
    #[must_use]
    pub fn new(profile: QualityProfile) -> Self {
        Self { profile }
    }

    /// Generate documentation for a function
    pub fn generate_for_function(&self, _code: &str, spec: &CreateSpec) -> Result<String> {
        debug_assert!(!_code.is_empty(), "_code must not be empty");
        let mut docs = String::new();

        docs.push_str(&format!("# {}\n\n", spec.name));
        docs.push_str(&format!("{}\n\n", spec.purpose));

        docs.push_str("## Parameters\n\n");
        for param in &spec.inputs {
            docs.push_str(&format!(
                "- `{}` ({}): {}\n",
                param.name,
                param.param_type,
                param
                    .description
                    .as_deref()
                    .unwrap_or("Parameter description")
            ));
        }

        docs.push_str(&format!(
            "\n## Returns\n\n{} ({})\n",
            spec.outputs
                .description
                .as_deref()
                .unwrap_or("Return value"),
            spec.outputs.param_type
        ));

        docs.push_str("\n## Example\n\n```rust\n");
        docs.push_str(&format!("let result = {}(", spec.name));

        for (i, param) in spec.inputs.iter().enumerate() {
            if i > 0 {
                docs.push_str(", ");
            }
            docs.push_str(&self.generate_example_value(&param.param_type).clone());
        }

        docs.push_str(");\n");
        docs.push_str("assert!(result.is_ok());\n");
        docs.push_str("```\n");

        Ok(docs)
    }

    pub(crate) fn generate_example_value(&self, param_type: &str) -> String {
        debug_assert!(!param_type.is_empty(), "param_type must not be empty");
        match param_type {
            "u32" | "i32" => "42".to_string(),
            "f32" | "f64" => "3.14".to_string(),
            "String" => "\"example\".to_string()".to_string(),
            "&str" => "\"example\"".to_string(),
            "bool" => "true".to_string(),
            _ => "Default::default()".to_string(),
        }
    }

    /// Generate documentation for any code
    pub fn generate_documentation(&self, code: &str) -> Result<String> {
        debug_assert!(!code.is_empty(), "code must not be empty");
        let mut docs = String::new();

        docs.push_str("# Generated Code Documentation\n\n");
        docs.push_str("This code was generated with quality standards.\n\n");

        // Extract function names from code
        let functions: Vec<&str> = code
            .lines()
            .filter_map(|line| {
                if line.contains("pub fn ") || line.contains("fn ") {
                    line.split("fn ").nth(1)?.split('(').next()
                } else {
                    None
                }
            })
            .collect();

        if !functions.is_empty() {
            docs.push_str("## Functions\n\n");
            for func in functions {
                docs.push_str(&format!("### {func}\n\n"));
                docs.push_str("Generated function with quality guarantees.\n\n");
            }
        }

        docs.push_str("## Quality Standards\n\n");
        docs.push_str(&format!(
            "- Maximum complexity: {}\n",
            self.profile.thresholds.max_complexity
        ));
        docs.push_str(&format!(
            "- Minimum coverage: {}%\n",
            self.profile.thresholds.min_coverage
        ));
        docs.push_str(&format!(
            "- Zero technical debt: {}\n",
            self.profile.thresholds.zero_satd
        ));

        Ok(docs)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::qdd::{CodeType, CreateSpec, Parameter};

    #[test]
    fn test_doc_generator_for_function() {
        let profile = QualityProfile::standard();
        let generator = DocGenerator::new(profile);

        let spec = create_multi_param_spec("doc_func");
        let docs = generator.generate_for_function("", &spec).unwrap();

        // Verify documentation structure
        assert!(docs.contains("# doc_func"));
        assert!(docs.contains("Multi-parameter function"));
        assert!(docs.contains("## Parameters"));
        assert!(docs.contains("`x` (u32): First parameter"));
        assert!(docs.contains("`y` (i32): Second parameter"));
        assert!(docs.contains("`z` (f64): Parameter description")); // Default for None
        assert!(docs.contains("## Returns"));
        assert!(docs.contains("Computed result (String)"));
        assert!(docs.contains("## Example"));
        assert!(docs.contains("```rust"));
    }

    #[test]
    fn test_doc_generator_example_values() {
        let profile = QualityProfile::standard();
        let generator = DocGenerator::new(profile);

        assert_eq!(generator.generate_example_value("u32"), "42");
        assert_eq!(generator.generate_example_value("i32"), "42");
        assert_eq!(generator.generate_example_value("f32"), "3.14");
        assert_eq!(generator.generate_example_value("f64"), "3.14");
        assert_eq!(
            generator.generate_example_value("String"),
            "\"example\".to_string()"
        );
        assert_eq!(generator.generate_example_value("&str"), "\"example\"");
        assert_eq!(generator.generate_example_value("bool"), "true");
        assert_eq!(
            generator.generate_example_value("Option<i32>"),
            "Default::default()"
        );
    }

    #[test]
    fn test_doc_generator_documentation_code_extraction() {
        let profile = QualityProfile::standard();
        let generator = DocGenerator::new(profile);

        let code = r#"
        pub fn first() {}
        fn private() {}
        pub fn second() {}
        pub fn third() {}
        "#;

        let docs = generator.generate_documentation(code).unwrap();

        assert!(docs.contains("# Generated Code Documentation"));
        assert!(docs.contains("## Functions"));
        assert!(docs.contains("### first"));
        assert!(docs.contains("### second"));
        assert!(docs.contains("### third"));
        // private() should also be detected (it has "fn ")
        assert!(docs.contains("### private"));
    }

    #[test]
    fn test_doc_generator_quality_standards_section() {
        let profile = QualityProfile::extreme();
        let generator = DocGenerator::new(profile);

        let code = "pub fn example() {}";
        let docs = generator.generate_documentation(code).unwrap();

        assert!(docs.contains("## Quality Standards"));
        assert!(docs.contains("Maximum complexity: 5"));
        assert!(docs.contains("Minimum coverage: 90%"));
        assert!(docs.contains("Zero technical debt: true"));
    }

    #[test]
    fn test_doc_generator_no_functions() {
        let profile = QualityProfile::standard();
        let generator = DocGenerator::new(profile);

        let code = "const X: i32 = 42;";
        let docs = generator.generate_documentation(code).unwrap();

        // Should still have structure but no Functions section content
        assert!(docs.contains("# Generated Code Documentation"));
        assert!(!docs.contains("## Functions")); // No functions detected
    }

    // Test helpers
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
        fn documentation_contains_purpose(
            purpose in "[A-Za-z][A-Za-z0-9 ]{0,50}"
        ) {
            let profile = QualityProfile::standard();
            let generator = DocGenerator::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: "test_fn".to_string(),
                purpose: purpose.clone(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let docs = generator.generate_for_function("", &spec).unwrap();
            prop_assert!(
                docs.contains(&purpose),
                "Documentation should contain purpose: {}",
                purpose
            );
        }
    }
}
