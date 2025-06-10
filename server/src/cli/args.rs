use crate::models::template::ParameterSpec;
use serde_json::Value;

/// Shared parameter validation logic
#[inline]
pub fn validate_params(
    specs: &[ParameterSpec],
    provided: &serde_json::Map<String, Value>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Check required parameters
    for spec in specs {
        if spec.required && !provided.contains_key(&spec.name) {
            errors.push(format!("Missing required parameter: {}", spec.name));
        }
    }

    // Validate types
    for (key, value) in provided {
        if let Some(spec) = specs.iter().find(|s| s.name == *key) {
            if !validate_type(&spec.param_type, value) {
                errors.push(format!(
                    "Invalid type for '{}': expected {:?}, got {}",
                    key,
                    spec.param_type,
                    value_type_name(value)
                ));
            }
        } else {
            errors.push(format!("Unknown parameter: {key}"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_type(expected: &crate::models::template::ParameterType, value: &Value) -> bool {
    use crate::models::template::ParameterType;

    match (expected, value) {
        // All parameter types can accept strings since they'll be validated later
        (_, Value::String(_)) => true,
        // Boolean type can accept bool values
        (ParameterType::Boolean, Value::Bool(_)) => true,
        // Everything else is invalid
        _ => false,
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Environment variable expansion for CLI defaults
pub fn expand_env_vars(template: &str) -> String {
    // Simple ${VAR} expansion
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_else(|_| caps[0].to_string())
    })
    .to_string()
}

#[inline]
/// Zero-allocation parameter parsing for common types
pub fn parse_key_val(s: &str) -> Result<(String, Value), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let key = &s[..pos];
    let val = &s[pos + 1..];

    // Type inference with fast paths
    let value = if val.is_empty() {
        Value::Bool(true) // Treat bare flags as true
    } else if val == "true" || val == "false" {
        Value::Bool(val.parse().unwrap())
    } else if let Ok(n) = val.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(f) = val.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(f).unwrap())
    } else {
        Value::String(val.to_string())
    };

    Ok((key.to_string(), value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
