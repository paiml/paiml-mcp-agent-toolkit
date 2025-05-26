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
    let mut result = String::new();
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
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
