#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    // === Toolchain Tests ===

    #[test]
    fn test_toolchain_rust_cli_priority() {
        let rust = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string(), "tokio".to_string()],
        };
        assert_eq!(rust.priority(), 1);
        assert_eq!(rust.as_str(), "rust");
    }

    #[test]
    fn test_toolchain_deno_typescript_priority() {
        let deno = Toolchain::DenoTypescript {
            deno_version: "1.38.0".to_string(),
        };
        assert_eq!(deno.priority(), 2);
        assert_eq!(deno.as_str(), "deno");
    }

    #[test]
    fn test_toolchain_python_uv_priority() {
        let python = Toolchain::PythonUv {
            python_version: "3.11".to_string(),
        };
        assert_eq!(python.priority(), 3);
        assert_eq!(python.as_str(), "python-uv");
    }

    #[test]
    fn test_toolchain_priority_ordering() {
        let rust = Toolchain::RustCli {
            cargo_features: vec![],
        };
        let deno = Toolchain::DenoTypescript {
            deno_version: "1.0".to_string(),
        };
        let python = Toolchain::PythonUv {
            python_version: "3.10".to_string(),
        };

        // Rust has highest priority (lowest number)
        assert!(rust.priority() < deno.priority());
        assert!(deno.priority() < python.priority());
    }

    #[test]
    fn test_toolchain_empty_features() {
        let rust = Toolchain::RustCli {
            cargo_features: vec![],
        };
        assert_eq!(rust.priority(), 1);
        assert_eq!(rust.as_str(), "rust");
    }

    #[test]
    fn test_toolchain_equality() {
        let rust1 = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string()],
        };
        let rust2 = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string()],
        };
        let rust3 = Toolchain::RustCli {
            cargo_features: vec!["tokio".to_string()],
        };

        assert_eq!(rust1, rust2);
        assert_ne!(rust1, rust3);
    }

    #[test]
    fn test_toolchain_clone() {
        let original = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string()],
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // === TemplateCategory Tests ===

    #[test]
    fn test_template_category_variants() {
        let makefile = TemplateCategory::Makefile;
        let readme = TemplateCategory::Readme;
        let gitignore = TemplateCategory::Gitignore;
        let context = TemplateCategory::Context;

        // All variants should be distinct
        assert_ne!(makefile, readme);
        assert_ne!(readme, gitignore);
        assert_ne!(gitignore, context);
        assert_ne!(context, makefile);
    }

    #[test]
    fn test_template_category_equality() {
        let cat1 = TemplateCategory::Makefile;
        let cat2 = TemplateCategory::Makefile;
        assert_eq!(cat1, cat2);
    }

    #[test]
    fn test_template_category_clone() {
        let original = TemplateCategory::Readme;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // === ParameterType Tests ===

    #[test]
    fn test_parameter_type_variants() {
        let types = [
            ParameterType::ProjectName,
            ParameterType::SemVer,
            ParameterType::GitHubUsername,
            ParameterType::LicenseIdentifier,
            ParameterType::Boolean,
            ParameterType::String,
        ];

        // Ensure all variants are distinct
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert_ne!(types[i], types[j], "Types at {} and {} should differ", i, j);
            }
        }
    }

    #[test]
    fn test_parameter_type_equality() {
        let pt1 = ParameterType::ProjectName;
        let pt2 = ParameterType::ProjectName;
        assert_eq!(pt1, pt2);
    }

    #[test]
    fn test_parameter_type_clone() {
        let original = ParameterType::SemVer;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // === ParameterSpec Tests ===

    #[test]
    fn test_parameter_spec_required() {
        let spec = ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::ProjectName,
            required: true,
            default_value: None,
            validation_pattern: Some(r"^[a-z][a-z0-9_-]*$".to_string()),
            description: "The name of the project".to_string(),
        };

        assert_eq!(spec.name, "project_name");
        assert!(spec.required);
        assert!(spec.default_value.is_none());
        assert!(spec.validation_pattern.is_some());
    }

    #[test]
    fn test_parameter_spec_optional_with_default() {
        let spec = ParameterSpec {
            name: "version".to_string(),
            param_type: ParameterType::SemVer,
            required: false,
            default_value: Some("0.1.0".to_string()),
            validation_pattern: Some(r"^\d+\.\d+\.\d+$".to_string()),
            description: "Semantic version".to_string(),
        };

        assert!(!spec.required);
        assert_eq!(spec.default_value, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_parameter_spec_clone() {
        let spec = ParameterSpec {
            name: "test".to_string(),
            param_type: ParameterType::Boolean,
            required: false,
            default_value: Some("true".to_string()),
            validation_pattern: None,
            description: "A boolean flag".to_string(),
        };

        let cloned = spec.clone();
        assert_eq!(cloned.name, spec.name);
        assert_eq!(cloned.param_type, spec.param_type);
        assert_eq!(cloned.required, spec.required);
        assert_eq!(cloned.default_value, spec.default_value);
    }

    // === TemplateResource Tests ===

    #[test]
    fn test_template_resource_creation() {
        let resource = TemplateResource {
            uri: "template://makefile/rust".to_string(),
            name: "Rust Makefile".to_string(),
            description: "Standard Makefile for Rust projects".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec!["serde".to_string()],
            },
            category: TemplateCategory::Makefile,
            parameters: vec![ParameterSpec {
                name: "project_name".to_string(),
                param_type: ParameterType::ProjectName,
                required: true,
                default_value: None,
                validation_pattern: None,
                description: "Project name".to_string(),
            }],
            s3_object_key: "templates/makefile/rust.mk".to_string(),
            content_hash: "abc123".to_string(),
            semantic_version: Version::new(1, 0, 0),
            dependency_graph: vec!["cargo".to_string()],
        };

        assert_eq!(resource.uri, "template://makefile/rust");
        assert_eq!(resource.name, "Rust Makefile");
        assert_eq!(resource.category, TemplateCategory::Makefile);
        assert_eq!(resource.parameters.len(), 1);
        assert_eq!(resource.semantic_version, Version::new(1, 0, 0));
    }

    #[test]
    fn test_template_resource_clone() {
        let resource = TemplateResource {
            uri: "template://readme/python".to_string(),
            name: "Python README".to_string(),
            description: "README template for Python projects".to_string(),
            toolchain: Toolchain::PythonUv {
                python_version: "3.11".to_string(),
            },
            category: TemplateCategory::Readme,
            parameters: vec![],
            s3_object_key: "templates/readme/python.md".to_string(),
            content_hash: "def456".to_string(),
            semantic_version: Version::new(2, 1, 0),
            dependency_graph: vec![],
        };

        let cloned = resource.clone();
        assert_eq!(cloned.uri, resource.uri);
        assert_eq!(cloned.name, resource.name);
        assert_eq!(cloned.semantic_version, resource.semantic_version);
    }

    // === GeneratedTemplate Tests ===

    #[test]
    fn test_generated_template_creation() {
        let template = GeneratedTemplate {
            content: "# My Project\n\nThis is a README.".to_string(),
            filename: "README.md".to_string(),
            checksum: "sha256:abc123def456".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
        };

        assert_eq!(template.filename, "README.md");
        assert!(template.content.contains("My Project"));
        assert!(template.checksum.starts_with("sha256:"));
    }

    #[test]
    fn test_generated_template_empty_content() {
        let template = GeneratedTemplate {
            content: "".to_string(),
            filename: "empty.txt".to_string(),
            checksum: "sha256:e3b0c44".to_string(),
            toolchain: Toolchain::DenoTypescript {
                deno_version: "1.38".to_string(),
            },
        };

        assert!(template.content.is_empty());
        assert!(!template.checksum.is_empty());
    }

    // === TemplateResponse Tests ===

    #[test]
    fn test_template_response_creation() {
        let response = TemplateResponse {
            content: "Generated content here".to_string(),
        };

        assert_eq!(response.content, "Generated content here");
    }

    #[test]
    fn test_template_response_empty() {
        let response = TemplateResponse {
            content: "".to_string(),
        };

        assert!(response.content.is_empty());
    }

    // === Serialization Tests ===

    #[test]
    fn test_toolchain_serialization_rust() {
        let rust = Toolchain::RustCli {
            cargo_features: vec!["serde".to_string(), "tokio".to_string()],
        };

        let json = serde_json::to_string(&rust).unwrap();
        assert!(json.contains(r#""type":"rust""#));
        assert!(json.contains("cargo_features"));
        assert!(json.contains("serde"));
        assert!(json.contains("tokio"));

        let deserialized: Toolchain = serde_json::from_str(&json).unwrap();
        assert_eq!(rust, deserialized);
    }

    #[test]
    fn test_toolchain_serialization_deno() {
        let deno = Toolchain::DenoTypescript {
            deno_version: "1.38.0".to_string(),
        };

        let json = serde_json::to_string(&deno).unwrap();
        assert!(json.contains(r#""type":"deno""#));
        assert!(json.contains("deno_version"));
        assert!(json.contains("1.38.0"));

        let deserialized: Toolchain = serde_json::from_str(&json).unwrap();
        assert_eq!(deno, deserialized);
    }

    #[test]
    fn test_toolchain_serialization_python() {
        let python = Toolchain::PythonUv {
            python_version: "3.11".to_string(),
        };

        let json = serde_json::to_string(&python).unwrap();
        assert!(json.contains(r#""type":"python-uv""#));
        assert!(json.contains("python_version"));

        let deserialized: Toolchain = serde_json::from_str(&json).unwrap();
        assert_eq!(python, deserialized);
    }

    #[test]
    fn test_template_category_serialization() {
        let categories = [
            (TemplateCategory::Makefile, "makefile"),
            (TemplateCategory::Readme, "readme"),
            (TemplateCategory::Gitignore, "gitignore"),
            (TemplateCategory::Context, "context"),
        ];

        for (cat, expected) in categories {
            let json = serde_json::to_string(&cat).unwrap();
            assert!(
                json.contains(expected),
                "Expected '{}' in JSON for {:?}",
                expected,
                cat
            );

            let deserialized: TemplateCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(cat, deserialized);
        }
    }

    #[test]
    fn test_parameter_type_serialization() {
        let types = [
            (ParameterType::ProjectName, "project_name"),
            (ParameterType::SemVer, "sem_ver"),
            (ParameterType::GitHubUsername, "git_hub_username"),
            (ParameterType::LicenseIdentifier, "license_identifier"),
            (ParameterType::Boolean, "boolean"),
            (ParameterType::String, "string"),
        ];

        for (pt, expected) in types {
            let json = serde_json::to_string(&pt).unwrap();
            assert!(
                json.contains(expected),
                "Expected '{}' in JSON for {:?}",
                expected,
                pt
            );

            let deserialized: ParameterType = serde_json::from_str(&json).unwrap();
            assert_eq!(pt, deserialized);
        }
    }

    #[test]
    fn test_parameter_spec_serialization() {
        let spec = ParameterSpec {
            name: "test_param".to_string(),
            param_type: ParameterType::Boolean,
            required: true,
            default_value: Some("false".to_string()),
            validation_pattern: Some(r"^(true|false)$".to_string()),
            description: "A test parameter".to_string(),
        };

        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("test_param"));
        assert!(json.contains("boolean"));
        assert!(json.contains(r#""required":true"#));

        let deserialized: ParameterSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, spec.name);
        assert_eq!(deserialized.param_type, spec.param_type);
        assert_eq!(deserialized.required, spec.required);
    }

    #[test]
    fn test_generated_template_serialization() {
        let template = GeneratedTemplate {
            content: "Hello, World!".to_string(),
            filename: "hello.txt".to_string(),
            checksum: "abc123".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
        };

        let json = serde_json::to_string(&template).unwrap();
        let deserialized: GeneratedTemplate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, template.content);
        assert_eq!(deserialized.filename, template.filename);
        assert_eq!(deserialized.checksum, template.checksum);
    }

    #[test]
    fn test_template_response_serialization() {
        let response = TemplateResponse {
            content: "Some generated content".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TemplateResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, response.content);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_toolchain_with_many_features() {
        let rust = Toolchain::RustCli {
            cargo_features: vec![
                "feature1".to_string(),
                "feature2".to_string(),
                "feature3".to_string(),
                "feature4".to_string(),
                "feature5".to_string(),
            ],
        };

        let json = serde_json::to_string(&rust).unwrap();
        let deserialized: Toolchain = serde_json::from_str(&json).unwrap();

        if let Toolchain::RustCli { cargo_features } = deserialized {
            assert_eq!(cargo_features.len(), 5);
        } else {
            panic!("Expected RustCli variant");
        }
    }

    #[test]
    fn test_parameter_spec_without_validation() {
        let spec = ParameterSpec {
            name: "simple".to_string(),
            param_type: ParameterType::String,
            required: false,
            default_value: None,
            validation_pattern: None,
            description: "A simple parameter".to_string(),
        };

        assert!(spec.validation_pattern.is_none());
        assert!(spec.default_value.is_none());
    }

    #[test]
    fn test_template_resource_empty_dependencies() {
        let resource = TemplateResource {
            uri: "template://test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
            category: TemplateCategory::Context,
            parameters: vec![],
            s3_object_key: "test.tpl".to_string(),
            content_hash: "hash".to_string(),
            semantic_version: Version::new(0, 0, 1),
            dependency_graph: vec![],
        };

        assert!(resource.dependency_graph.is_empty());
        assert!(resource.parameters.is_empty());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
}
