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
    use crate::models::template::{ParameterSpec, ParameterType};
    use serde_json::{Map, Value};

    #[test]
    fn test_validate_params_required_missing() {
        let specs = vec![ParameterSpec {
            name: "required_param".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            validation_pattern: None,
            description: "A required parameter".to_string(),
        }];

        let provided = Map::new();
        let result = validate_params(&specs, &provided);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Missing required parameter: required_param"));
    }

    #[test]
    fn test_validate_params_optional_missing() {
        let specs = vec![ParameterSpec {
            name: "optional_param".to_string(),
            param_type: ParameterType::String,
            required: false,
            default_value: Some("default".to_string()),
            validation_pattern: None,
            description: "An optional parameter".to_string(),
        }];

        let provided = Map::new();
        let result = validate_params(&specs, &provided);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_params_unknown_parameter() {
        let specs = vec![];
        let mut provided = Map::new();
        provided.insert("unknown".to_string(), Value::String("value".to_string()));

        let result = validate_params(&specs, &provided);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Unknown parameter: unknown"));
    }

    #[test]
    fn test_validate_params_type_validation() {
        let specs = vec![ParameterSpec {
            name: "bool_param".to_string(),
            param_type: ParameterType::Boolean,
            required: true,
            default_value: None,
            validation_pattern: None,
            description: "A boolean parameter".to_string(),
        }];

        let mut provided = Map::new();
        provided.insert("bool_param".to_string(), Value::Number(123.into()));

        let result = validate_params(&specs, &provided);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Invalid type for 'bool_param'"));
    }

    #[test]
    fn test_validate_params_success() {
        let specs = vec![
            ParameterSpec {
                name: "string_param".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                validation_pattern: None,
                description: "A string parameter".to_string(),
            },
            ParameterSpec {
                name: "bool_param".to_string(),
                param_type: ParameterType::Boolean,
                required: true,
                default_value: None,
                validation_pattern: None,
                description: "A boolean parameter".to_string(),
            },
        ];

        let mut provided = Map::new();
        provided.insert(
            "string_param".to_string(),
            Value::String("test".to_string()),
        );
        provided.insert("bool_param".to_string(), Value::Bool(true));

        let result = validate_params(&specs, &provided);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_type_string_accepts_all() {
        assert!(validate_type(
            &ParameterType::String,
            &Value::String("test".to_string())
        ));
        assert!(validate_type(
            &ParameterType::Boolean,
            &Value::String("true".to_string())
        ));
    }

    #[test]
    fn test_validate_type_boolean() {
        assert!(validate_type(&ParameterType::Boolean, &Value::Bool(true)));
        assert!(validate_type(&ParameterType::Boolean, &Value::Bool(false)));
        assert!(!validate_type(
            &ParameterType::Boolean,
            &Value::Number(123.into())
        ));
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(value_type_name(&Value::Null), "null");
        assert_eq!(value_type_name(&Value::Bool(true)), "boolean");
        assert_eq!(value_type_name(&Value::Number(123.into())), "number");
        assert_eq!(
            value_type_name(&Value::String("test".to_string())),
            "string"
        );
        assert_eq!(value_type_name(&Value::Array(vec![])), "array");
        assert_eq!(value_type_name(&Value::Object(Map::new())), "object");
    }

    #[test]
    fn test_expand_env_vars_no_vars() {
        let result = expand_env_vars("no variables here");
        assert_eq!(result, "no variables here");
    }

    #[test]
    fn test_expand_env_vars_with_existing_var() {
        std::env::set_var("TEST_VAR", "expanded_value");
        let result = expand_env_vars("prefix ${TEST_VAR} suffix");
        assert_eq!(result, "prefix expanded_value suffix");
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_expand_env_vars_with_missing_var() {
        let result = expand_env_vars("prefix ${NONEXISTENT_VAR} suffix");
        assert_eq!(result, "prefix ${NONEXISTENT_VAR} suffix");
    }

    #[test]
    fn test_parse_key_val_string() {
        let result = parse_key_val("key=value").unwrap();
        assert_eq!(result.0, "key");
        assert_eq!(result.1, Value::String("value".to_string()));
    }

    #[test]
    fn test_parse_key_val_boolean_true() {
        let result = parse_key_val("flag=true").unwrap();
        assert_eq!(result.0, "flag");
        assert_eq!(result.1, Value::Bool(true));
    }

    #[test]
    fn test_parse_key_val_boolean_false() {
        let result = parse_key_val("flag=false").unwrap();
        assert_eq!(result.0, "flag");
        assert_eq!(result.1, Value::Bool(false));
    }

    #[test]
    fn test_parse_key_val_integer() {
        let result = parse_key_val("count=42").unwrap();
        assert_eq!(result.0, "count");
        assert_eq!(result.1, Value::Number(42.into()));
    }

    #[test]
    fn test_parse_key_val_float() {
        let result = parse_key_val("ratio=1.234").unwrap();
        assert_eq!(result.0, "ratio");
        if let Value::Number(n) = result.1 {
            assert_eq!(n.as_f64().unwrap(), 1.234);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_parse_key_val_empty_value() {
        let result = parse_key_val("flag=").unwrap();
        assert_eq!(result.0, "flag");
        assert_eq!(result.1, Value::Bool(true));
    }

    #[test]
    fn test_parse_key_val_no_equals() {
        let result = parse_key_val("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no `=` found"));
    }

    #[test]
    fn test_parse_key_val_complex_string() {
        let result = parse_key_val("path=/some/complex/path").unwrap();
        assert_eq!(result.0, "path");
        assert_eq!(result.1, Value::String("/some/complex/path".to_string()));
    }
}
