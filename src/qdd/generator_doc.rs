//! Documentation generator

use super::{CreateSpec, QualityProfile};
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

    pub fn generate_example_value(&self, param_type: &str) -> String {
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
