// Template URI parsing, parameter validation, and template validation functions.
// Includes parse_template_uri, build_template_prefix, extract_filename,
// validate_parameters, validate_template, and all related helpers.

fn parse_template_uri(uri: &str) -> Result<(&str, &str, &str), TemplateError> {
    debug_assert!(!uri.is_empty(), "uri must not be empty");
    let parts: Vec<&str> = uri
        .strip_prefix("template://")
        .ok_or_else(|| TemplateError::InvalidUri {
            uri: uri.to_string(),
        })?
        .split('/')
        .collect();

    if parts.len() != 3 {
        return Err(TemplateError::InvalidUri {
            uri: uri.to_string(),
        });
    }

    Ok((parts[0], parts[1], parts[2]))
}

fn build_template_prefix(category: Option<&str>, toolchain: Option<&str>) -> String {
    match (category, toolchain) {
        (None, None) => String::new(), // Empty prefix to match all
        (Some(cat), None) => format!("{cat}/"),
        (Some(cat), Some(tc)) => format!("{cat}/{tc}/"),
        (None, Some(_)) => String::new(), // Invalid case, return empty
    }
}

fn extract_filename(category: &str) -> String {
    debug_assert!(!category.is_empty(), "category must not be empty");
    match category {
        "makefile" => "Makefile".to_string(),
        "readme" => "README.md".to_string(),
        "gitignore" => ".gitignore".to_string(),
        _ => format!("{category}.txt"),
    }
}

fn validate_parameters(
    specs: &[crate::models::template::ParameterSpec],
    provided: &Map<String, serde_json::Value>,
) -> Result<(), TemplateError> {
    debug_assert!(!specs.is_empty(), "specs must not be empty");
    // Collect all missing required params for a single helpful error
    let missing: Vec<_> = specs
        .iter()
        .filter(|s| s.required && !provided.contains_key(&s.name))
        .collect();
    if !missing.is_empty() {
        let params_help: Vec<String> = missing
            .iter()
            .map(|s| format!("  -p {}=<value>  ({})", s.name, s.description))
            .collect();
        return Err(TemplateError::ValidationError {
            parameter: missing.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "),
            reason: format!(
                "required parameter(s) missing. Add:\n{}",
                params_help.join("\n")
            ),
        });
    }
    for spec in specs {
        validate_parameter_value(spec, provided)?;
    }
    Ok(())
}

#[cfg(test)]
fn check_required_parameter(
    spec: &crate::models::template::ParameterSpec,
    provided: &Map<String, serde_json::Value>,
) -> Result<(), TemplateError> {
    if spec.required && !provided.contains_key(&spec.name) {
        return Err(TemplateError::ValidationError {
            parameter: spec.name.clone(),
            reason: format!(
                "required parameter missing. Use: -p {}=<value> ({})",
                spec.name, spec.description
            ),
        });
    }
    Ok(())
}

fn validate_parameter_value(
    spec: &crate::models::template::ParameterSpec,
    provided: &Map<String, serde_json::Value>,
) -> Result<(), TemplateError> {
    if let Some(value) = provided.get(&spec.name) {
        if let Some(pattern) = &spec.validation_pattern {
            validate_pattern_match(spec, pattern, value)?;
        }
    }
    Ok(())
}

fn validate_pattern_match(
    spec: &crate::models::template::ParameterSpec,
    pattern: &str,
    value: &serde_json::Value,
) -> Result<(), TemplateError> {
    let regex = regex::Regex::new(pattern).map_err(|_| TemplateError::ValidationError {
        parameter: spec.name.clone(),
        reason: "invalid validation pattern".to_string(),
    })?;

    if let Some(str_value) = value.as_str() {
        if !regex.is_match(str_value) {
            return Err(TemplateError::ValidationError {
                parameter: spec.name.clone(),
                reason: format!("value does not match pattern: {pattern}"),
            });
        }
    }
    Ok(())
}

pub async fn validate_template<T: TemplateServerTrait>(
    server: Arc<T>,
    uri: &str,
    parameters: &serde_json::Value,
) -> Result<ValidationResult, TemplateError> {
    debug_assert!(!uri.is_empty(), "uri must not be empty");
    let metadata = get_template_metadata(server, uri).await?;
    let params_map = match extract_params_map(parameters) {
        Ok(map) => map,
        Err(validation_result) => return Ok(validation_result),
    };

    let mut errors = Vec::new();
    validate_required_parameters(&metadata.parameters, params_map, &mut errors);
    validate_parameter_values(&metadata.parameters, params_map, &mut errors);

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
    })
}

async fn get_template_metadata<T: TemplateServerTrait>(
    server: Arc<T>,
    uri: &str,
) -> Result<crate::models::template::TemplateResource, TemplateError> {
    debug_assert!(!uri.is_empty(), "uri must not be empty");
    server
        .get_template_metadata(uri)
        .await
        .map(|arc_resource| (*arc_resource).clone())
        .map_err(|_| TemplateError::TemplateNotFound {
            uri: uri.to_string(),
        })
}

fn extract_params_map(
    parameters: &serde_json::Value,
) -> Result<&Map<String, serde_json::Value>, ValidationResult> {
    if let serde_json::Value::Object(map) = parameters {
        Ok(map)
    } else {
        let error = ValidationError {
            field: "parameters".to_string(),
            message: "Parameters must be an object".to_string(),
        };
        Err(ValidationResult {
            valid: false,
            errors: vec![error],
        })
    }
}

fn validate_required_parameters(
    param_specs: &[crate::models::template::ParameterSpec],
    params_map: &Map<String, serde_json::Value>,
    errors: &mut Vec<ValidationError>,
) {
    debug_assert!(!param_specs.is_empty(), "param_specs must not be empty");
    for param in param_specs {
        if param.required && !params_map.contains_key(&param.name) {
            errors.push(ValidationError {
                field: param.name.clone(),
                message: "Required parameter missing".to_string(),
            });
        }
    }
}

fn validate_parameter_values(
    param_specs: &[crate::models::template::ParameterSpec],
    params_map: &Map<String, serde_json::Value>,
    errors: &mut Vec<ValidationError>,
) {
    debug_assert!(!param_specs.is_empty(), "param_specs must not be empty");
    for (key, value) in params_map {
        if let Some(param_spec) = param_specs.iter().find(|p| p.name == *key) {
            validate_parameter_pattern(param_spec, key, value, errors);
        } else {
            errors.push(ValidationError {
                field: key.clone(),
                message: "Unknown parameter".to_string(),
            });
        }
    }
}

fn validate_parameter_pattern(
    param_spec: &crate::models::template::ParameterSpec,
    key: &str,
    value: &serde_json::Value,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(pattern) = &param_spec.validation_pattern {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(str_val) = value.as_str() {
                if !regex.is_match(str_val) {
                    errors.push(ValidationError {
                        field: key.to_string(),
                        message: format!("Does not match pattern: {pattern}"),
                    });
                }
            }
        }
    }
}
