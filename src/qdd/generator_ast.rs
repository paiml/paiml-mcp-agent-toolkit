//! AST builder for creating code structures

use super::{CreateSpec, QualityProfile};
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
    pub fn generate_example_value(&self, param_type: &str) -> String {
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
