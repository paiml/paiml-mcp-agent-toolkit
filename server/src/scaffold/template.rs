// Template rendering system - TICKET-PMAT-5002
// Simple variable substitution for code generation

use std::collections::HashMap;
use crate::scaffold::errors::{ScaffoldError, Result};

/// Template with variable substitution
#[derive(Debug, Clone)]
pub struct Template {
    content: String,
    name: String,
}

impl Template {
    /// Create template from string content
    ///
    /// # Complexity
    /// - Time: O(1) - simple construction
    /// - Cyclomatic: 1
    pub fn from_string(content: impl Into<String>) -> Result<Self> {
        Ok(Self {
            content: content.into(),
            name: String::new(),
        })
    }

    /// Create named template
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Result<Self> {
        Ok(Self {
            content: content.into(),
            name: name.into(),
        })
    }

    /// Render template with variable substitution
    ///
    /// Variables use {{name}} syntax
    ///
    /// # Complexity
    /// - Time: O(n*m) where n=template size, m=number of variables
    /// - Cyclomatic: 3 (iteration, substitution, validation)
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Check for unreplaced variables
        if result.contains("{{") && result.contains("}}") {
            // Extract unreplaced variable for error message
            if let Some(start) = result.find("{{") {
                if let Some(end) = result[start..].find("}}") {
                    let var_name = &result[start+2..start+end];
                    return Err(ScaffoldError::InvalidProjectName(
                        format!("Unreplaced template variable: {}", var_name)
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Get template name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// pforge.yaml template
    pub fn pforge_yaml() -> Self {
        Self::new(
            "pforge.yaml",
            include_str!("templates/pforge.yaml.tmpl")
        ).expect("pforge.yaml template should be valid")
    }

    /// Cargo.toml template
    pub fn cargo_toml() -> Self {
        Self::new(
            "Cargo.toml",
            include_str!("templates/Cargo.toml.tmpl")
        ).expect("Cargo.toml template should be valid")
    }

    /// Handler template
    pub fn handler_rs() -> Self {
        Self::new(
            "handler.rs",
            include_str!("templates/handler.rs.tmpl")
        ).expect("handler.rs template should be valid")
    }

    /// README template
    pub fn readme_md() -> Self {
        Self::new(
            "README.md",
            include_str!("templates/README.md.tmpl")
        ).expect("README.md template should be valid")
    }
}

/// Template registry for managing templates
#[derive(Debug)]
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    /// Create new template registry
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    ///
    /// # Complexity
    /// - Time: O(1) average
    /// - Cyclomatic: 1
    pub fn register(&mut self, name: String, template: Template) {
        self.templates.insert(name, template);
    }

    /// Get template by name
    ///
    /// # Complexity
    /// - Time: O(1) average
    /// - Cyclomatic: 2 (lookup + error)
    pub fn get(&self, name: &str) -> Result<&Template> {
        self.templates.get(name)
            .ok_or_else(|| ScaffoldError::InvalidProjectName(
                format!("Template not found: {}", name)
            ))
    }

    /// List all template names
    ///
    /// # Complexity
    /// - Time: O(n) where n=number of templates
    /// - Cyclomatic: 1
    pub fn list(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Create registry with default pforge templates
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Cyclomatic: 1
    pub fn with_pforge_templates() -> Self {
        let mut registry = Self::new();
        registry.register("pforge.yaml".into(), Template::pforge_yaml());
        registry.register("Cargo.toml".into(), Template::cargo_toml());
        registry.register("handler.rs".into(), Template::handler_rs());
        registry.register("README.md".into(), Template::readme_md());
        registry
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;
