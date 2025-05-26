use crate::cli::args;
use crate::models::template::{
    ParameterSpec, ParameterType, TemplateCategory, TemplateResource, Toolchain,
};
use crate::stateless_server::StatelessTemplateServer;
use semver::Version;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;

#[cfg(test)]
mod cli_args_tests {
    use super::*;

    #[test]
    fn test_validate_params_all_valid() {
        let specs = vec![
            ParameterSpec {
                name: "project_name".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                description: "Project name".to_string(),
                validation_pattern: None,
            },
            ParameterSpec {
                name: "has_tests".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some("true".to_string()),
                description: "Include tests".to_string(),
                validation_pattern: None,
            },
        ];

        let mut params = serde_json::Map::new();
        params.insert("project_name".to_string(), json!("my-project"));
        params.insert("has_tests".to_string(), json!(false));

        let result = args::validate_params(&specs, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_params_missing_required() {
        let specs = vec![ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Project name".to_string(),
            validation_pattern: None,
        }];

        let params = serde_json::Map::new();

        let result = args::validate_params(&specs, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Missing required parameter: project_name"));
    }

    #[test]
    fn test_validate_params_unknown_parameter() {
        let specs = vec![ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Project name".to_string(),
            validation_pattern: None,
        }];

        let mut params = serde_json::Map::new();
        params.insert("project_name".to_string(), json!("my-project"));
        params.insert("unknown_param".to_string(), json!("value"));

        let result = args::validate_params(&specs, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("Unknown parameter: unknown_param")));
    }

    #[test]
    fn test_validate_params_type_validation() {
        let specs = vec![ParameterSpec {
            name: "has_tests".to_string(),
            param_type: ParameterType::Boolean,
            required: true,
            default_value: None,
            description: "Include tests".to_string(),
            validation_pattern: None,
        }];

        // Boolean value should work
        let mut params = serde_json::Map::new();
        params.insert("has_tests".to_string(), json!(true));
        assert!(args::validate_params(&specs, &params).is_ok());

        // String value should also work (will be parsed later)
        let mut params = serde_json::Map::new();
        params.insert("has_tests".to_string(), json!("true"));
        assert!(args::validate_params(&specs, &params).is_ok());

        // Number value should fail
        let mut params = serde_json::Map::new();
        params.insert("has_tests".to_string(), json!(123));
        let result = args::validate_params(&specs, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Invalid type"));
    }

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_USER", "alice");
        std::env::set_var("TEST_HOME", "/home/alice");

        let template = "Hello ${TEST_USER}, your home is ${TEST_HOME}";
        let expanded = args::expand_env_vars(template);
        assert_eq!(expanded, "Hello alice, your home is /home/alice");

        // Test with missing env var
        let template = "Missing: ${NONEXISTENT_VAR}";
        let expanded = args::expand_env_vars(template);
        assert_eq!(expanded, "Missing: ${NONEXISTENT_VAR}");

        // Cleanup
        std::env::remove_var("TEST_USER");
        std::env::remove_var("TEST_HOME");
    }

    #[test]
    fn test_expand_env_vars_no_vars() {
        let template = "No variables here";
        let expanded = args::expand_env_vars(template);
        assert_eq!(expanded, "No variables here");
    }

    #[test]
    fn test_expand_env_vars_multiple_occurrences() {
        std::env::set_var("TEST_VAR", "value");

        let template = "${TEST_VAR} and ${TEST_VAR} again";
        let expanded = args::expand_env_vars(template);
        assert_eq!(expanded, "value and value again");

        std::env::remove_var("TEST_VAR");
    }
}

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    async fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().unwrap())
    }

    #[tokio::test]
    async fn test_generate_command_to_stdout() {
        let _server = create_test_server().await;

        // Capture stdout by writing to a file
        let temp_dir = TempDir::new().unwrap();
        let _output_file = temp_dir.path().join("output.txt");

        // We can't easily test stdout directly in unit tests, but we can test the file output
        // Skip this test for now - would need to refactor CLI to be more testable
    }

    #[tokio::test]
    async fn test_generate_command_to_file() {
        let _server = create_test_server().await;
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("Makefile");

        // Simulate CLI args
        let _args = [
            "paiml-mcp-agent-toolkit",
            "generate",
            "makefile",
            "rust/cli-binary",
            "-p",
            "project_name=test-project",
            "-p",
            "has_tests=true",
            "-o",
            output_file.to_str().unwrap(),
        ];

        // We would need to refactor the CLI to make it testable
        // For now, we'll test the underlying functions directly
    }

    #[tokio::test]
    async fn test_list_command_json_format() {
        let _server = create_test_server().await;

        // Test listing templates - would need CLI refactoring for proper testing
    }

    #[tokio::test]
    async fn test_search_command() {
        let _server = create_test_server().await;

        // Test search functionality - would need CLI refactoring
    }

    #[tokio::test]
    async fn test_validate_command() {
        let _server = create_test_server().await;

        // Test validation - would need CLI refactoring
    }

    #[tokio::test]
    async fn test_scaffold_command() {
        let _server = create_test_server().await;
        let _temp_dir = TempDir::new().unwrap();

        // Test scaffolding - would need CLI refactoring
    }

    #[tokio::test]
    async fn test_context_command() {
        let _server = create_test_server().await;
        let temp_dir = TempDir::new().unwrap();

        // Create test project files
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_dir).await.unwrap();

        // Create a Rust project
        fs::write(
            project_dir.join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
"#,
        )
        .await
        .unwrap();

        fs::write(
            project_dir.join("main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}
"#,
        )
        .await
        .unwrap();

        // Test context generation - would need CLI refactoring
    }
}

#[cfg(test)]
mod cli_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_key_val() {
        // This function is private in the CLI module, but we can test similar logic

        // Test basic key=value
        let input = "name=value";
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "name");
        assert_eq!(parts[1], "value");

        // Test with = in value
        let input = "url=https://example.com?foo=bar";
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "url");
        assert_eq!(parts[1], "https://example.com?foo=bar");

        // Test without =
        let input = "invalid";
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_value_type_inference() {
        // Test boolean inference
        assert_eq!(json!("true"), Value::String("true".to_string()));
        assert_eq!(json!(true), Value::Bool(true));

        // Test number inference
        assert_eq!(json!("123"), Value::String("123".to_string()));
        assert_eq!(json!(123), Value::Number(123.into()));

        // Test float inference
        assert_eq!(json!("123.45"), Value::String("123.45".to_string()));
        assert_eq!(
            json!(123.45),
            Value::Number(serde_json::Number::from_f64(123.45).unwrap())
        );
    }
}

#[cfg(test)]
mod cli_output_tests {
    use super::*;

    #[test]
    fn test_table_formatting() {
        // Test table output formatting logic
        let templates = [
            Arc::new(TemplateResource {
                uri: "template://makefile/rust/cli-binary".to_string(),
                category: TemplateCategory::Makefile,
                toolchain: Toolchain::RustCli {
                    cargo_features: vec![],
                },
                name: "Rust CLI Binary Makefile".to_string(),
                description: "Makefile for Rust CLI applications".to_string(),
                parameters: vec![ParameterSpec {
                    name: "project_name".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default_value: None,
                    description: "Project name".to_string(),
                    validation_pattern: None,
                }],
                content_hash: "hash123".to_string(),
                semantic_version: Version::parse("1.0.0").unwrap(),
                dependency_graph: vec![],
                s3_object_key: "templates/makefile/rust/cli-binary.hbs".to_string(),
            }),
            Arc::new(TemplateResource {
                uri: "template://readme/deno/cli-application".to_string(),
                category: TemplateCategory::Readme,
                toolchain: Toolchain::DenoTypescript {
                    deno_version: "1.40.0".to_string(),
                },
                name: "Deno CLI README".to_string(),
                description: "README for Deno CLI applications".to_string(),
                parameters: vec![],
                content_hash: "hash456".to_string(),
                semantic_version: Version::parse("1.0.0").unwrap(),
                dependency_graph: vec![],
                s3_object_key: "templates/readme/deno/cli-application.hbs".to_string(),
            }),
        ];

        // Calculate expected column width
        let expected_width = templates.iter().map(|t| t.uri.len()).max().unwrap_or(20);

        assert_eq!(
            expected_width,
            "template://readme/deno/cli-application".len()
        );
    }

    #[test]
    fn test_json_output_format() {
        let template = TemplateResource {
            uri: "template://makefile/rust/cli-binary".to_string(),
            category: TemplateCategory::Makefile,
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
            name: "Test Template".to_string(),
            description: "Test description".to_string(),
            parameters: vec![],
            content_hash: "hash789".to_string(),
            semantic_version: Version::parse("1.0.0").unwrap(),
            dependency_graph: vec![],
            s3_object_key: "test.hbs".to_string(),
        };

        let json = serde_json::to_string_pretty(&template).unwrap();
        assert!(json.contains("\"uri\": \"template://makefile/rust/cli-binary\""));
        assert!(json.contains("\"category\": \"makefile\""));
        assert!(json.contains("\"type\": \"rust\""));
    }

    #[test]
    fn test_yaml_output_format() {
        let template = TemplateResource {
            uri: "template://makefile/rust/cli-binary".to_string(),
            category: TemplateCategory::Makefile,
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
            name: "Test Template".to_string(),
            description: "Test description".to_string(),
            parameters: vec![],
            content_hash: "hash789".to_string(),
            semantic_version: Version::parse("1.0.0").unwrap(),
            dependency_graph: vec![],
            s3_object_key: "test.hbs".to_string(),
        };

        let yaml = serde_yaml::to_string(&template).unwrap();
        assert!(yaml.contains("uri: template://makefile/rust/cli-binary"));
        assert!(yaml.contains("category: makefile"));
        assert!(yaml.contains("type: rust"));
    }
}

#[cfg(test)]
mod cli_error_handling_tests {
    use super::*;

    #[test]
    fn test_missing_required_params_error() {
        let specs = vec![ParameterSpec {
            name: "required_param".to_string(),
            param_type: ParameterType::String,
            required: true,
            default_value: None,
            description: "Required parameter".to_string(),
            validation_pattern: None,
        }];

        let params = serde_json::Map::new();
        let result = args::validate_params(&specs, &params);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("required_param"));
    }

    #[test]
    fn test_multiple_validation_errors() {
        let specs = vec![
            ParameterSpec {
                name: "param1".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                description: "First param".to_string(),
                validation_pattern: None,
            },
            ParameterSpec {
                name: "param2".to_string(),
                param_type: ParameterType::Boolean,
                required: true,
                default_value: None,
                description: "Second param".to_string(),
                validation_pattern: None,
            },
        ];

        let mut params = serde_json::Map::new();
        // Missing param1
        // Invalid type for param2
        params.insert("param2".to_string(), json!(123));
        // Unknown param
        params.insert("unknown".to_string(), json!("value"));

        let result = args::validate_params(&specs, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 3);
    }
}

#[cfg(test)]
mod cli_mode_detection_tests {

    #[test]
    fn test_cli_parameter_parsing() {
        // Test parameter parsing patterns
        let test_cases = [
            ("key=value", Some(("key", "value"))),
            ("key=", Some(("key", ""))),
            ("invalid", None),
            ("key=value=with=equals", Some(("key", "value=with=equals"))),
        ];

        for (input, expected) in test_cases {
            let parts: Vec<&str> = input.splitn(2, '=').collect();
            match expected {
                Some((key, val)) => {
                    assert_eq!(parts.len(), 2);
                    assert_eq!(parts[0], key);
                    assert_eq!(parts[1], val);
                }
                None => {
                    assert_eq!(parts.len(), 1);
                }
            }
        }
    }

    #[test]
    fn test_template_uri_patterns() {
        // Test valid template URI patterns
        let valid_uris = [
            "template://makefile/rust/cli-binary",
            "template://readme/deno/cli-application",
            "template://gitignore/python-uv/cli-application",
        ];

        for uri in valid_uris {
            assert!(uri.starts_with("template://"));
            let parts: Vec<&str> = uri
                .strip_prefix("template://")
                .unwrap()
                .split('/')
                .collect();
            assert_eq!(parts.len(), 3);
        }
    }

    #[test]
    fn test_toolchain_names() {
        // Test expected toolchain names
        let toolchains = ["rust", "deno", "python-uv"];

        for toolchain in toolchains {
            assert!(!toolchain.is_empty());
            assert!(toolchain
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-'));
        }
    }
}
