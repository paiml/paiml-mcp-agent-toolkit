#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for models/template.rs
    //! These tests ensure comprehensive coverage of all template model types.

    use super::*;

    /// Test all TemplateCategory variants can be Debug-printed
    #[test]
    fn test_template_category_debug() {
        let categories = [
            TemplateCategory::Makefile,
            TemplateCategory::Readme,
            TemplateCategory::Gitignore,
            TemplateCategory::Context,
        ];

        for cat in &categories {
            let debug_str = format!("{:?}", cat);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test all ParameterType variants can be Debug-printed
    #[test]
    fn test_parameter_type_debug() {
        let types = [
            ParameterType::ProjectName,
            ParameterType::SemVer,
            ParameterType::GitHubUsername,
            ParameterType::LicenseIdentifier,
            ParameterType::Boolean,
            ParameterType::String,
        ];

        for pt in &types {
            let debug_str = format!("{:?}", pt);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test Toolchain Debug implementation
    #[test]
    fn test_toolchain_debug() {
        let toolchains = [
            Toolchain::RustCli {
                cargo_features: vec!["serde".to_string()],
            },
            Toolchain::DenoTypescript {
                deno_version: "1.38".to_string(),
            },
            Toolchain::PythonUv {
                python_version: "3.11".to_string(),
            },
        ];

        for tc in &toolchains {
            let debug_str = format!("{:?}", tc);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test ParameterSpec Debug implementation
    #[test]
    fn test_parameter_spec_debug() {
        let spec = ParameterSpec {
            name: "test".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            validation_pattern: None,
            description: "A test parameter".to_string(),
        };

        let debug_str = format!("{:?}", spec);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("String"));
    }

    /// Test TemplateResource Debug implementation
    #[test]
    fn test_template_resource_debug() {
        let resource = TemplateResource {
            uri: "test://uri".to_string(),
            name: "Test Template".to_string(),
            description: "A test".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
            category: TemplateCategory::Makefile,
            parameters: vec![],
            s3_object_key: "key".to_string(),
            content_hash: "hash".to_string(),
            semantic_version: Version::new(1, 0, 0),
            dependency_graph: vec![],
        };

        let debug_str = format!("{:?}", resource);
        assert!(debug_str.contains("Test Template"));
    }

    /// Test GeneratedTemplate Debug implementation
    #[test]
    fn test_generated_template_debug() {
        let template = GeneratedTemplate {
            content: "content".to_string(),
            filename: "file.txt".to_string(),
            checksum: "abc123".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
        };

        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("file.txt"));
    }

    /// Test TemplateResponse Debug implementation
    #[test]
    fn test_template_response_debug() {
        let response = TemplateResponse {
            content: "response content".to_string(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("response content"));
    }

    /// Test deserialization with invalid JSON
    #[test]
    fn test_invalid_json_deserialization() {
        let invalid_json = r#"{"type": "invalid_toolchain"}"#;
        let result: Result<Toolchain, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    /// Test deserialization with missing required fields
    #[test]
    fn test_missing_field_deserialization() {
        let incomplete_json = r#"{"type": "rust"}"#;
        let result: Result<Toolchain, _> = serde_json::from_str(incomplete_json);
        assert!(result.is_err());
    }

    /// Test TemplateCategory deserialization with invalid value
    #[test]
    fn test_template_category_invalid_deserialization() {
        let invalid = r#""invalid_category""#;
        let result: Result<TemplateCategory, _> = serde_json::from_str(invalid);
        assert!(result.is_err());
    }

    /// Test ParameterType deserialization with invalid value
    #[test]
    fn test_parameter_type_invalid_deserialization() {
        let invalid = r#""unknown_type""#;
        let result: Result<ParameterType, _> = serde_json::from_str(invalid);
        assert!(result.is_err());
    }

    /// Test complex nested serialization
    #[test]
    fn test_complex_nested_serialization() {
        let resource = TemplateResource {
            uri: "template://complex".to_string(),
            name: "Complex Template".to_string(),
            description: "A complex template with nested structures".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![
                    "serde".to_string(),
                    "tokio".to_string(),
                    "async-trait".to_string(),
                ],
            },
            category: TemplateCategory::Makefile,
            parameters: vec![
                ParameterSpec {
                    name: "param1".to_string(),
                    param_type: ParameterType::ProjectName,
                    required: true,
                    default_value: None,
                    validation_pattern: Some(r"^[a-z]+$".to_string()),
                    description: "First parameter".to_string(),
                },
                ParameterSpec {
                    name: "param2".to_string(),
                    param_type: ParameterType::SemVer,
                    required: false,
                    default_value: Some("1.0.0".to_string()),
                    validation_pattern: None,
                    description: "Second parameter".to_string(),
                },
            ],
            s3_object_key: "templates/complex.tpl".to_string(),
            content_hash: "sha256:complex_hash".to_string(),
            semantic_version: Version::new(2, 3, 4),
            dependency_graph: vec!["dep1".to_string(), "dep2".to_string()],
        };

        let json = serde_json::to_string_pretty(&resource).unwrap();
        let roundtrip: TemplateResource = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.uri, resource.uri);
        assert_eq!(roundtrip.parameters.len(), 2);
        assert_eq!(roundtrip.dependency_graph.len(), 2);
        assert_eq!(roundtrip.semantic_version, Version::new(2, 3, 4));
    }
}
