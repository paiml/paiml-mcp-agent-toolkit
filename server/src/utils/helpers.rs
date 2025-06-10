use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
    RenderErrorReason,
};

pub fn snake_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| -> RenderError {
        RenderErrorReason::ParamNotFoundForIndex("snake_case", 0).into()
    })?;

    let value = param.value().as_str().ok_or_else(|| -> RenderError {
        RenderErrorReason::Other("snake_case expects string parameter".to_string()).into()
    })?;

    let snake = to_snake_case(value);
    out.write(&snake)?;
    Ok(())
}

pub fn kebab_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| -> RenderError {
        RenderErrorReason::ParamNotFoundForIndex("kebab_case", 0).into()
    })?;

    let value = param.value().as_str().ok_or_else(|| -> RenderError {
        RenderErrorReason::Other("kebab_case expects string parameter".to_string()).into()
    })?;

    let kebab = to_kebab_case(value);
    out.write(&kebab)?;
    Ok(())
}

pub fn pascal_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).ok_or_else(|| -> RenderError {
        RenderErrorReason::ParamNotFoundForIndex("pascal_case", 0).into()
    })?;

    let value = param.value().as_str().ok_or_else(|| -> RenderError {
        RenderErrorReason::Other("pascal_case expects string parameter".to_string()).into()
    })?;

    let pascal = to_pascal_case(value);
    out.write(&pascal)?;
    Ok(())
}

pub fn current_year_helper(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let year = chrono::Utc::now().format("%Y").to_string();
    out.write(&year)?;
    Ok(())
}

pub fn current_date_helper(
    _: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    out.write(&date)?;
    Ok(())
}

// Case conversion utilities
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(1024);
    let mut prev_is_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 && !prev_is_upper {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
        prev_is_upper = ch.is_uppercase();
    }

    result.replace(['-', ' '], "_")
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-', ' '])
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::with_capacity(1024),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;
    use serde_json::json;

    #[test]
    fn test_to_snake_case_basic() {
        assert_eq!(to_snake_case("MyProjectName"), "my_project_name");
        assert_eq!(to_snake_case("camelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("snake_case"), "snake_case");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
        assert_eq!(to_snake_case("With Spaces"), "with__spaces"); // Double underscore for spaces
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_to_snake_case_edge_cases() {
        assert_eq!(to_snake_case("XMLHttpRequest"), "xmlhttp_request");
        assert_eq!(to_snake_case("IOError"), "ioerror");
        assert_eq!(to_snake_case("123Numbers"), "123_numbers");
        assert_eq!(to_snake_case("UPPERCASE"), "uppercase");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
    }

    #[test]
    fn test_to_kebab_case_basic() {
        assert_eq!(to_kebab_case("MyProjectName"), "my-project-name");
        assert_eq!(to_kebab_case("snake_case"), "snake-case");
        assert_eq!(to_kebab_case("With Spaces"), "with--spaces"); // Double dash for spaces
    }

    #[test]
    fn test_to_pascal_case_basic() {
        assert_eq!(to_pascal_case("my_project_name"), "MyProjectName");
        assert_eq!(to_pascal_case("kebab-case-name"), "KebabCaseName");
        assert_eq!(to_pascal_case("with spaces here"), "WithSpacesHere");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_pascal_case("single"), "Single");
    }

    #[test]
    fn test_snake_case_helper_with_handlebars() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        let template = "{{snake_case name}}";
        let data = json!({"name": "MyProjectName"});
        let result = handlebars.render_template(template, &data).unwrap();
        assert_eq!(result, "my_project_name");
    }

    #[test]
    fn test_kebab_case_helper_with_handlebars() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("kebab_case", Box::new(kebab_case_helper));

        let template = "{{kebab_case name}}";
        let data = json!({"name": "MyProjectName"});
        let result = handlebars.render_template(template, &data).unwrap();
        assert_eq!(result, "my-project-name");
    }

    #[test]
    fn test_pascal_case_helper_with_handlebars() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("pascal_case", Box::new(pascal_case_helper));

        let template = "{{pascal_case name}}";
        let data = json!({"name": "my_project_name"});
        let result = handlebars.render_template(template, &data).unwrap();
        assert_eq!(result, "MyProjectName");
    }

    #[test]
    fn test_current_year_helper_with_handlebars() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("current_year", Box::new(current_year_helper));

        let template = "{{current_year}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data).unwrap();

        // Verify it's a valid year
        let year: u32 = result.parse().expect("Should be a valid year");
        assert!((2024..=2100).contains(&year));
    }

    #[test]
    fn test_current_date_helper_with_handlebars() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("current_date", Box::new(current_date_helper));

        let template = "{{current_date}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data).unwrap();

        // Check format YYYY-MM-DD
        assert_eq!(result.len(), 10);
        assert_eq!(result.chars().nth(4), Some('-'));
        assert_eq!(result.chars().nth(7), Some('-'));
    }

    #[test]
    fn test_helper_error_cases() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));

        // Missing parameter
        let template = "{{snake_case}}";
        let data = json!({});
        let result = handlebars.render_template(template, &data);
        assert!(result.is_err());

        // Non-string parameter
        let template = "{{snake_case number}}";
        let data = json!({"number": 123});
        let result = handlebars.render_template(template, &data);
        assert!(result.is_err());
    }
}
