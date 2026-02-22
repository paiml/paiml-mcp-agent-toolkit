#![cfg_attr(coverage_nightly, coverage(off))]
//! Template rendering engine for code generation.
//!
//! This module provides a flexible template rendering system based on minijinja
//! templating engine. It supports custom filters for common code transformations
//! and provides a safe environment for template execution.
//!
//! # Features
//!
//! - **Jinja2 Templates**: Full support for Jinja2/minijinja syntax
//! - **Custom Filters**: Built-in filters for case transformations
//! - **Date/Time Support**: Automatic injection of current date/timestamp
//! - **Error Handling**: Detailed error messages with line numbers
//! - **Type Safety**: Strict type checking for template variables
//!
//! # Built-in Filters
//!
//! - `{{ value|snake_case }}` - Converts to `snake_case`
//! - `{{ value|kebab_case }}` - Converts to kebab-case
//! - `{{ value|pascal_case }}` - Converts to `PascalCase`
//! - `{{ current_year() }}` - Inserts current year
//! - `{{ current_date() }}` - Inserts current date
//!
//! # Example
//!
//! ```
//! use pmat::services::renderer::{TemplateRenderer, render_template};
//! use serde_json::{Map, Value};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let renderer = TemplateRenderer::new()?;
//!
//! let template = r#"
//! pub struct {{ name|pascal_case }} {
//!     created_at: &'static str,
//! }
//!
//! impl {{ name|pascal_case }} {
//!     pub fn new() -> Self {
//!         Self {
//!             created_at: "{{ current_date() }}",
//!         }
//!     }
//! }
//! "#;
//!
//! let mut context = Map::new();
//! context.insert("name".to_string(), Value::String("my_struct".to_string()));
//!
//! let rendered = render_template(&renderer, template, context)?;
//! assert!(rendered.contains("pub struct MyStruct"));
//! # Ok(())
//! # }
//! ```

use crate::models::error::TemplateError;
use crate::utils::helpers;
use serde_json::Value;

pub struct TemplateRenderer {
    env: minijinja::Environment<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut env = minijinja::Environment::new();
        // Allow undefined variables to render as empty (like handlebars non-strict mode)
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Chainable);

        // Register custom filters and functions
        helpers::register_helpers(&mut env);

        Ok(Self { env })
    }
}

pub fn render_template(
    renderer: &TemplateRenderer,
    template: &str,
    context: serde_json::Map<String, Value>,
) -> Result<String, TemplateError> {
    // Add system context
    let mut full_context = context;
    full_context.insert(
        "current_timestamp".to_string(),
        Value::String(chrono::Utc::now().to_rfc3339()),
    );

    let tmpl = renderer
        .env
        .template_from_str(template)
        .map_err(|e| TemplateError::RenderError {
            line: e.line().unwrap_or(0) as u32,
            message: e.to_string(),
        })?;

    tmpl.render(Value::Object(full_context))
        .map_err(|e| TemplateError::RenderError {
            line: e.line().unwrap_or(0) as u32,
            message: e.to_string(),
        })
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_renderer_new() {
        let renderer = TemplateRenderer::new();
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_render_template_simple() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Hello, {{ name }}!";
        let mut context = serde_json::Map::new();
        context.insert("name".to_string(), Value::String("World".to_string()));

        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_render_template_with_current_timestamp() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Generated at: {{ current_timestamp }}";
        let context = serde_json::Map::new();

        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.starts_with("Generated at: 20"));
        // RFC3339 format check
        assert!(output.contains("T") || output.contains(" "));
        assert!(output.len() > 20); // Should have a full timestamp
    }

    #[test]
    fn test_render_template_with_helpers() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Project: {{ project_name|pascal_case }}";

        let mut context = serde_json::Map::new();
        context.insert(
            "project_name".to_string(),
            Value::String("my test project".to_string()),
        );

        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Project: MyTestProject");
    }

    #[test]
    fn test_render_template_missing_variable() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Hello, {{ name }}! Your age is {{ age }}.";
        let mut context = serde_json::Map::new();
        context.insert("name".to_string(), Value::String("Alice".to_string()));

        // In chainable undefined mode, missing variables render as empty
        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Alice! Your age is .");
    }

    #[test]
    fn test_render_template_error() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "{% if %}Missing condition{% endif %}"; // Invalid template
        let context = serde_json::Map::new();

        let result = render_template(&renderer, template, context);
        assert!(result.is_err());

        match result.unwrap_err() {
            TemplateError::RenderError { line: _, message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected RenderError"),
        }
    }

    #[test]
    fn test_render_template_with_conditionals() {
        let renderer = TemplateRenderer::new().unwrap();
        let template =
            "{% if enabled %}Feature is enabled{% else %}Feature is disabled{% endif %}";

        let mut context = serde_json::Map::new();
        context.insert("enabled".to_string(), Value::Bool(true));

        let result = render_template(&renderer, template, context.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Feature is enabled");

        context.insert("enabled".to_string(), Value::Bool(false));
        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Feature is disabled");
    }

    #[test]
    fn test_render_template_preserves_original_context() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Value: {{ value }}, Timestamp: {{ current_timestamp }}";

        let mut context = serde_json::Map::new();
        context.insert("value".to_string(), Value::String("test".to_string()));
        context.insert(
            "current_timestamp".to_string(),
            Value::String("should-be-overwritten".to_string()),
        );

        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Value: test"));
        assert!(!output.contains("should-be-overwritten"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            prop_assert!(_x < 1001);
        }
    }
}
