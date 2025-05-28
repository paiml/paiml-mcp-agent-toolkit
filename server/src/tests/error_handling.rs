#[cfg(test)]
mod error_handling_tests {
    use crate::models::error::TemplateError;
    use crate::models::template::{ParameterSpec, ParameterType};

    #[test]
    fn test_template_error_display() {
        let error = TemplateError::InvalidUri {
            uri: "bad://uri".to_string(),
        };
        assert!(error.to_string().contains("Invalid template URI"));
        assert!(error.to_string().contains("bad://uri"));
    }

    #[test]
    fn test_template_not_found_error() {
        let error = TemplateError::TemplateNotFound {
            uri: "template://missing/not/found".to_string(),
        };
        assert!(error.to_string().contains("Template not found"));
        assert!(error.to_string().contains("template://missing/not/found"));
    }

    #[test]
    fn test_not_found_error() {
        let error = TemplateError::NotFound("Resource not found".to_string());
        assert!(error.to_string().contains("Not found"));
        assert!(error.to_string().contains("Resource not found"));
    }

    #[test]
    fn test_render_error() {
        let error = TemplateError::RenderError {
            line: 42,
            message: "Failed to render template".to_string(),
        };
        assert!(error.to_string().contains("Template rendering failed"));
        assert!(error.to_string().contains("line 42"));
        assert!(error.to_string().contains("Failed to render template"));
    }

    #[test]
    fn test_validation_error() {
        let error = TemplateError::ValidationError {
            parameter: "port".to_string(),
            reason: "Port must be between 1 and 65535".to_string(),
        };
        assert!(error.to_string().contains("Parameter validation failed"));
        assert!(error.to_string().contains("port"));
        assert!(error
            .to_string()
            .contains("Port must be between 1 and 65535"));
    }

    #[test]
    fn test_invalid_utf8_error() {
        let error = TemplateError::InvalidUtf8("Bad encoding in template".to_string());
        assert!(error.to_string().contains("Invalid UTF-8"));
    }

    #[test]
    fn test_error_to_mcp_code() {
        assert_eq!(
            TemplateError::TemplateNotFound {
                uri: "test".to_string()
            }
            .to_mcp_code(),
            -32001
        );
        assert_eq!(
            TemplateError::InvalidUri {
                uri: "test".to_string()
            }
            .to_mcp_code(),
            -32002
        );
        assert_eq!(
            TemplateError::ValidationError {
                parameter: "p".to_string(),
                reason: "r".to_string()
            }
            .to_mcp_code(),
            -32003
        );
        assert_eq!(
            TemplateError::RenderError {
                line: 1,
                message: "msg".to_string()
            }
            .to_mcp_code(),
            -32004
        );
        assert_eq!(
            TemplateError::NotFound("test".to_string()).to_mcp_code(),
            -32000
        );
        assert_eq!(
            TemplateError::InvalidUtf8("test".to_string()).to_mcp_code(),
            -32000
        );
    }

    #[test]
    fn test_parameter_spec_creation() {
        let spec = ParameterSpec {
            name: "test_param".to_string(),
            description: "Test parameter".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            validation_pattern: Some(r"^\w+$".to_string()),
        };

        assert_eq!(spec.name, "test_param");
        assert_eq!(spec.description, "Test parameter");
        assert!(matches!(spec.param_type, ParameterType::String));
        assert!(spec.required);
        assert!(spec.default_value.is_none());
        assert!(spec.validation_pattern.is_some());
    }

    #[test]
    fn test_parameter_spec_with_default() {
        let spec = ParameterSpec {
            name: "optional_param".to_string(),
            description: "Optional parameter".to_string(),
            param_type: ParameterType::Boolean,
            required: false,
            default_value: Some("true".to_string()),
            validation_pattern: None,
        };

        assert!(!spec.required);
        assert!(spec.default_value.is_some());
        assert_eq!(spec.default_value.unwrap(), "true");
    }

    #[test]
    fn test_error_debug_representation() {
        let error = TemplateError::InvalidUri {
            uri: "bad://uri".to_string(),
        };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidUri"));
        assert!(debug_str.contains("bad://uri"));
    }

    #[test]
    fn test_multiple_error_types() {
        let errors = vec![
            TemplateError::InvalidUri {
                uri: "test".to_string(),
            },
            TemplateError::TemplateNotFound {
                uri: "test".to_string(),
            },
            TemplateError::NotFound("not found".to_string()),
            TemplateError::RenderError {
                line: 1,
                message: "error".to_string(),
            },
            TemplateError::ValidationError {
                parameter: "param".to_string(),
                reason: "invalid".to_string(),
            },
            TemplateError::InvalidUtf8("bad utf8".to_string()),
            TemplateError::S3Error {
                operation: "GetObject".to_string(),
                source: anyhow::anyhow!("S3 error"),
            },
        ];

        // Ensure all errors have distinct string representations
        let error_strings: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        for i in 0..error_strings.len() {
            for j in (i + 1)..error_strings.len() {
                assert_ne!(
                    error_strings[i], error_strings[j],
                    "Error strings should be unique: '{}' vs '{}'",
                    error_strings[i], error_strings[j]
                );
            }
        }
    }

    #[test]
    fn test_cache_error_from_anyhow() {
        let anyhow_error = anyhow::anyhow!("Cache operation failed");
        let error = TemplateError::CacheError(anyhow_error);
        assert!(error.to_string().contains("Cache operation failed"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{invalid json}";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        if let Err(json_err) = result {
            let error = TemplateError::JsonError(json_err);
            assert!(error.to_string().contains("JSON serialization error"));
        }
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io;
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = TemplateError::Io(io_error);
        assert!(error.to_string().contains("IO operation failed"));
    }

    #[test]
    fn test_s3_error() {
        let error = TemplateError::S3Error {
            operation: "PutObject".to_string(),
            source: anyhow::anyhow!("Failed to upload to S3"),
        };
        assert!(error.to_string().contains("S3 operation failed"));
        assert!(error.to_string().contains("PutObject"));
    }
}
