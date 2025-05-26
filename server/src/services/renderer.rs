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
