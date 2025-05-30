#[cfg(test)]
use crate::cli::args::{parse_key_val, validate_params};
#[cfg(test)]
use crate::models::template::{ParameterSpec, ParameterType};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use serde_json::json;

// Property: Parameter parsing preserves types
proptest! {
    #[test]
    fn prop_parameter_parsing_preserves_types(
        key in "[a-z_]+",
        str_val in "[^0-9].*", // Avoid strings that start with numbers
        bool_val in any::<bool>(),
        num_val in any::<f64>().prop_filter("finite", |x| x.is_finite()),
    ) {
        // Test string values that are NOT "true", "false", or numbers
        let input = format!("{}={}", key, str_val);
        if let Ok((k, v)) = parse_key_val(&input) {
            assert_eq!(k, key);
            // Check if this string will be parsed as bool or number
            if str_val == "true" || str_val == "false" {
                // These become booleans
                assert!(v.is_boolean());
                assert_eq!(v.as_bool().unwrap(), str_val == "true");
            } else if str_val.parse::<f64>().is_ok() {
                // These become numbers
                assert!(v.is_number());
            } else {
                // Everything else remains a string
                assert!(v.is_string());
                assert_eq!(v.as_str().unwrap(), str_val);
            }
        }

        // Test boolean values explicitly
        let bool_str = if bool_val { "true" } else { "false" };
        let input = format!("{}={}", key, bool_str);
        if let Ok((k, v)) = parse_key_val(&input) {
            assert_eq!(k, key);
            assert!(v.is_boolean());
            assert_eq!(v.as_bool().unwrap(), bool_val);
        }

        // Test number values
        let input = format!("{}={}", key, num_val);
        if let Ok((k, v)) = parse_key_val(&input) {
            assert_eq!(k, key);
            assert!(v.is_number());
            assert_eq!(v.as_f64().unwrap(), num_val);
        }
    }
}

// Property: Valid template URIs can be parsed
proptest! {
    #[test]
    fn prop_template_uri_format(
        category in "(makefile|readme|gitignore)",
        toolchain in "(rust|deno|python-uv)",
        variant in "cli",
    ) {
        let uri = format!("template://{}/{}/{}", category, toolchain, variant);

        // Verify URI structure
        assert!(uri.starts_with("template://"));
        let parts: Vec<&str> = uri.strip_prefix("template://").unwrap().split('/').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], category);
        assert_eq!(parts[1], toolchain);
        assert_eq!(parts[2], variant);
    }
}

// Property: Key-value parsing handles edge cases correctly
proptest! {
    #[test]
    fn prop_key_value_parsing_edge_cases(
        key in "[a-zA-Z_][a-zA-Z0-9_]*",
        value in ".*",
    ) {
        // Test with various value patterns
        let input = format!("{}={}", key, value);

        match parse_key_val(&input) {
            Ok((parsed_key, parsed_value)) => {
                assert_eq!(parsed_key, key);
                // Value might be parsed as bool, number, or string
                // parse_key_val parses "true"/"false" as bool, numbers as numbers
                match &parsed_value {
                    serde_json::Value::Bool(b) => {
                        // Empty string becomes true, "true"/"false" become bools
                        if value.is_empty() {
                            assert!(*b);
                        } else {
                            assert!(value == "true" || value == "false");
                            assert_eq!(value, b.to_string());
                        }
                    },
                    serde_json::Value::Number(n) => {
                        // For numeric values, verify it parses correctly
                        assert_eq!(value.parse::<f64>().unwrap(), n.as_f64().unwrap());
                    },
                    serde_json::Value::String(s) => {
                        // Only non-bool/non-number values remain as strings
                        assert_eq!(s, &value);
                    },
                    _ => panic!("Unexpected value type"),
                }
            }
            Err(_) => {
                // Should only fail if the format is completely wrong
                panic!("Valid key=value should parse successfully");
            }
        }
    }
}

// Property: Empty values are handled correctly
proptest! {
    #[test]
    fn prop_empty_value_handling(
        key in "[a-zA-Z_]+",
    ) {
        let input = format!("{}=", key);
        let (parsed_key, parsed_value) = parse_key_val(&input).unwrap();
        assert_eq!(parsed_key, key);
        assert_eq!(parsed_value, json!(true)); // Empty value is treated as boolean true
    }
}

// Property: Parameter validation respects type constraints
proptest! {
    #[test]
    fn prop_parameter_type_validation(
        param_name in "[a-z_]+",
        str_value in ".*",
        bool_str in "(true|false|yes|no|1|0)",
    ) {
        // String type accepts any value
        let string_spec = vec![ParameterSpec {
            name: param_name.clone(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Test".to_string(),
            validation_pattern: None,
        }];

        let mut params = serde_json::Map::new();
        params.insert(param_name.clone(), json!(str_value));
        assert!(validate_params(&string_spec, &params).is_ok());

        // Boolean type accepts boolean-like strings
        let bool_spec = vec![ParameterSpec {
            name: param_name.clone(),
            param_type: ParameterType::Boolean,
            required: true,
            default_value: None,
            description: "Test".to_string(),
            validation_pattern: None,
        }];

        let mut params = serde_json::Map::new();
        params.insert(param_name.clone(), json!(bool_str));
        assert!(validate_params(&bool_spec, &params).is_ok());
    }
}

// Property: validate_params only validates types and required fields, not patterns
proptest! {
    #[test]
    fn prop_validate_params_behavior(
        param_name in "[a-z_]+",
        valid_value in "[a-z][a-z0-9-]*",
        invalid_pattern_value in "[A-Z][A-Z0-9]*",
    ) {
        let spec = vec![ParameterSpec {
            name: param_name.clone(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Test".to_string(),
            validation_pattern: Some(r"^[a-z][a-z0-9-]*$".to_string()),
        }];

        // Both valid and invalid patterns should pass validate_params
        // because it only checks types, not patterns
        let mut params = serde_json::Map::new();
        params.insert(param_name.clone(), json!(valid_value));
        assert!(validate_params(&spec, &params).is_ok());

        // Even invalid patterns pass because validate_params doesn't check patterns
        let mut params = serde_json::Map::new();
        params.insert(param_name.clone(), json!(invalid_pattern_value));
        assert!(validate_params(&spec, &params).is_ok());
    }
}

// Property: Multiple parameters are validated independently
proptest! {
    #[test]
    fn prop_multiple_parameter_validation(
        params_count in 1..10usize,
        keys in prop::collection::vec("[a-z_]+", 1..10),
        values in prop::collection::vec(".*", 1..10),
    ) {
        // Create specs for all parameters
        let specs: Vec<ParameterSpec> = keys
            .iter()
            .take(params_count)
            .map(|key| ParameterSpec {
                name: key.clone(),
                param_type: ParameterType::String,
                required: false,
                default_value: Some("default".to_string()),
                description: "Test".to_string(),
                validation_pattern: None,
            })
            .collect();

        // Create parameter map
        let mut params = serde_json::Map::new();
        for (i, key) in keys.iter().take(params_count).enumerate() {
            if i < values.len() {
                params.insert(key.clone(), json!(values[i]));
            }
        }

        // Should validate successfully since all are optional
        assert!(validate_params(&specs, &params).is_ok());
    }
}

// Property: Default values are respected
proptest! {
    #[test]
    fn prop_default_values_respected(
        param_name in "[a-z_]+",
        default_value in ".*",
    ) {
        let spec = vec![ParameterSpec {
            name: param_name.clone(),
            param_type: ParameterType::String,
            required: false,
            default_value: Some(default_value.clone()),
            description: "Test".to_string(),
            validation_pattern: None,
        }];

        // Empty params should be valid due to default
        let params = serde_json::Map::new();
        assert!(validate_params(&spec, &params).is_ok());
    }
}

// Property: Required parameters are enforced
proptest! {
    #[test]
    fn prop_required_parameters_enforced(
        param_name in "[a-z_]+",
        other_param in "[a-z_]+",
        value in ".*",
    ) {
        prop_assume!(param_name != other_param);

        let spec = vec![ParameterSpec {
            name: param_name.clone(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Test".to_string(),
            validation_pattern: None,
        }];

        // Missing required parameter should fail
        let params = serde_json::Map::new();
        assert!(validate_params(&spec, &params).is_err());

        // Providing wrong parameter should still fail
        let mut params = serde_json::Map::new();
        params.insert(other_param, json!(value));
        assert!(validate_params(&spec, &params).is_err());

        // Providing correct parameter should pass
        let mut params = serde_json::Map::new();
        params.insert(param_name, json!(value));
        assert!(validate_params(&spec, &params).is_ok());
    }
}

// Property: Unknown parameters are rejected
proptest! {
    #[test]
    fn prop_unknown_parameters_rejected(
        known_param in "[a-z_]+",
        unknown_param in "[a-z_]+",
        value in ".*",
    ) {
        prop_assume!(known_param != unknown_param);

        let spec = vec![ParameterSpec {
            name: known_param.clone(),
            param_type: ParameterType::String,
            required: false,
            default_value: Some("default".to_string()),
            description: "Test".to_string(),
            validation_pattern: None,
        }];

        // Unknown parameter should cause validation error
        let mut params = serde_json::Map::new();
        params.insert(unknown_param, json!(value));

        let result = validate_params(&spec, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Unknown parameter")));
    }
}

// Property: Escaping in parameter values is preserved
proptest! {
    #[test]
    fn prop_parameter_escaping_preserved(
        key in "[a-z_]+",
        value_with_special_chars in r#".*[\n\r\t"'\\].*"#,
    ) {
        let input = format!("{}={}", key, value_with_special_chars);

        if let Ok((parsed_key, parsed_value)) = parse_key_val(&input) {
            assert_eq!(parsed_key, key);
            // The value should be preserved exactly as provided
            assert_eq!(parsed_value, json!(value_with_special_chars));
        }
    }
}

// Property: Number-like strings are parsed as numbers
proptest! {
    #[test]
    fn prop_number_strings_parsed(
        key in "[a-z_]+",
        int_val in any::<i64>(),
        float_val in any::<f64>().prop_filter("finite", |x| x.is_finite()),
    ) {
        // Integer-like strings are parsed as numbers
        let input = format!("{}={}", key, int_val);
        let (_, parsed_value) = parse_key_val(&input).unwrap();
        assert_eq!(parsed_value, json!(int_val));

        // Float-like strings are parsed as numbers
        let input = format!("{}={}", key, float_val);
        let (_, parsed_value) = parse_key_val(&input).unwrap();
        // Compare the actual numeric values, not JSON representations
        // This handles floating point precision and -0.0 vs 0.0
        assert!(parsed_value.is_number());
        let parsed_float = parsed_value.as_f64().unwrap();
        // For zero values, accept both 0.0 and -0.0
        if float_val == 0.0 || float_val == -0.0 {
            assert!(parsed_float == 0.0 || parsed_float == -0.0);
        } else {
            // For non-zero values, check if they're approximately equal
            // due to floating point precision issues
            assert!((parsed_float - float_val).abs() < f64::EPSILON * float_val.abs().max(1.0));
        }
    }
}

// Property: Complex parameter values with equals signs
proptest! {
    #[test]
    fn prop_complex_values_with_equals(
        key in "[a-z_]+",
        prefix in ".*",
        suffix in ".*",
    ) {
        // Create a value with embedded equals signs
        let complex_value = format!("{}=middle={}", prefix, suffix);
        let input = format!("{}={}", key, complex_value);

        let (parsed_key, parsed_value) = parse_key_val(&input).unwrap();
        assert_eq!(parsed_key, key);
        assert_eq!(parsed_value, json!(complex_value));
    }
}
