// Unit tests and property tests for template_service.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_service_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_parse_template_uri_valid() {
        let result = parse_template_uri("template://project/rust/cli");
        assert!(result.is_ok());
        let (category, toolchain, variant) = result.unwrap();
        assert_eq!(category, "project");
        assert_eq!(toolchain, "rust");
        assert_eq!(variant, "cli");
    }

    #[test]
    fn test_parse_template_uri_no_prefix() {
        let result = parse_template_uri("project/rust/cli");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_template_uri_too_few_parts() {
        let result = parse_template_uri("template://project/rust");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_template_uri_too_many_parts() {
        let result = parse_template_uri("template://a/b/c/d");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_template_prefix_none_none() {
        let prefix = build_template_prefix(None, None);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_build_template_prefix_category_only() {
        let prefix = build_template_prefix(Some("project"), None);
        assert_eq!(prefix, "project/");
    }

    #[test]
    fn test_build_template_prefix_both() {
        let prefix = build_template_prefix(Some("project"), Some("rust"));
        assert_eq!(prefix, "project/rust/");
    }

    #[test]
    fn test_build_template_prefix_toolchain_only() {
        let prefix = build_template_prefix(None, Some("rust"));
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_extract_filename_makefile() {
        assert_eq!(extract_filename("makefile"), "Makefile");
    }

    #[test]
    fn test_extract_filename_readme() {
        assert_eq!(extract_filename("readme"), "README.md");
    }

    #[test]
    fn test_extract_filename_gitignore() {
        assert_eq!(extract_filename("gitignore"), ".gitignore");
    }

    #[test]
    fn test_extract_filename_other() {
        assert_eq!(extract_filename("config"), "config.txt");
        assert_eq!(extract_filename("setup"), "setup.txt");
    }

    #[test]
    fn test_validate_parameters_empty() {
        let specs = vec![];
        let params = Map::new();
        let result = validate_parameters(&specs, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_required_parameter_missing() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::String,
            description: "Name of the project".to_string(),
            required: true,
            default_value: None,
            validation_pattern: None,
        };
        let params = Map::new();
        let result = check_required_parameter(&spec, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_required_parameter_present() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::String,
            description: "Name of the project".to_string(),
            required: true,
            default_value: None,
            validation_pattern: None,
        };
        let mut params = Map::new();
        params.insert(
            "project_name".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        let result = check_required_parameter(&spec, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_required_parameter_optional() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "description".to_string(),
            param_type: ParameterType::String,
            description: "Optional description".to_string(),
            required: false,
            default_value: None,
            validation_pattern: None,
        };
        let params = Map::new();
        let result = check_required_parameter(&spec, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_parameter_value_no_pattern() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "name".to_string(),
            param_type: ParameterType::String,
            description: "test".to_string(),
            required: false,
            default_value: None,
            validation_pattern: None,
        };
        let mut params = Map::new();
        params.insert(
            "name".to_string(),
            serde_json::Value::String("anything".to_string()),
        );
        let result = validate_parameter_value(&spec, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_match_valid() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "name".to_string(),
            param_type: ParameterType::String,
            description: "test".to_string(),
            required: true,
            default_value: None,
            validation_pattern: Some("^[a-z]+$".to_string()),
        };
        let value = serde_json::Value::String("hello".to_string());
        let result = validate_pattern_match(&spec, "^[a-z]+$", &value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_match_invalid() {
        use crate::models::template::{ParameterSpec, ParameterType};
        let spec = ParameterSpec {
            name: "name".to_string(),
            param_type: ParameterType::String,
            description: "test".to_string(),
            required: true,
            default_value: None,
            validation_pattern: Some("^[a-z]+$".to_string()),
        };
        let value = serde_json::Value::String("Hello123".to_string());
        let result = validate_pattern_match(&spec, "^[a-z]+$", &value);
        assert!(result.is_err());
    }
}

