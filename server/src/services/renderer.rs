use crate::models::error::TemplateError;
use crate::utils::helpers;
use handlebars::Handlebars;
use serde_json::Value;

pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register custom helpers
        handlebars.register_helper("snake_case", Box::new(helpers::snake_case_helper));
        handlebars.register_helper("kebab_case", Box::new(helpers::kebab_case_helper));
        handlebars.register_helper("pascal_case", Box::new(helpers::pascal_case_helper));
        handlebars.register_helper("current_year", Box::new(helpers::current_year_helper));
        handlebars.register_helper("current_date", Box::new(helpers::current_date_helper));

        Ok(Self { handlebars })
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

    renderer
        .handlebars
        .render_template(template, &Value::Object(full_context))
        .map_err(|e| TemplateError::RenderError {
            line: e.line_no.unwrap_or(0) as u32,
            message: e.to_string(),
        })
}

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
        let template = "Hello, {{name}}!";
        let mut context = serde_json::Map::new();
        context.insert("name".to_string(), Value::String("World".to_string()));

        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_render_template_with_current_timestamp() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "Generated at: {{current_timestamp}}";
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
        let template = "Project: {{pascal_case project_name}}";

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
        let template = "Hello, {{name}}! Your age is {{age}}.";
        let mut context = serde_json::Map::new();
        context.insert("name".to_string(), Value::String("Alice".to_string()));

        // In non-strict mode, missing variables render as empty
        let result = render_template(&renderer, template, context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Alice! Your age is .");
    }

    #[test]
    fn test_render_template_error() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "{{#if}}Missing condition{{/if}}"; // Invalid template
        let context = serde_json::Map::new();

        let result = render_template(&renderer, template, context);
        assert!(result.is_err());

        match result.unwrap_err() {
            TemplateError::RenderError { line: _, message } => {
                assert!(message.contains("if") || message.contains("param"));
                // Line number can vary based on handlebars version
            }
            _ => panic!("Expected RenderError"),
        }
    }

    #[test]
    fn test_render_template_with_conditionals() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "{{#if enabled}}Feature is enabled{{else}}Feature is disabled{{/if}}";

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
        let template = "Value: {{value}}, Timestamp: {{current_timestamp}}";

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
