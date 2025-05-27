#[cfg(test)]
mod tests {
    use crate::cli::args::{expand_env_vars, validate_params};
    use crate::models::template::{ParameterSpec, ParameterType};
    use serde_json::json;

    #[test]
    fn test_validate_params_basic() {
        let specs = vec![ParameterSpec {
            name: "project_name".to_string(),
            description: "Name of the project".to_string(),
            required: true,
            default_value: None,
            validation_pattern: None,
            param_type: ParameterType::String,
        }];

        let mut params = serde_json::Map::new();
        params.insert("project_name".to_string(), json!("my-project"));

        assert!(validate_params(&specs, &params).is_ok());
    }

    #[test]
    fn test_expand_env_vars_basic() {
        std::env::set_var("TEST_CLI_VAR", "test_value");
        assert_eq!(expand_env_vars("${TEST_CLI_VAR}"), "test_value");
        std::env::remove_var("TEST_CLI_VAR");

        // Test non-existent var
        assert_eq!(expand_env_vars("${NONEXISTENT}"), "${NONEXISTENT}");
    }
}

#[cfg(test)]
mod cli_command_enums {
    use crate::cli::{Commands, ContextFormat, OutputFormat};

    #[test]
    fn test_output_format_enum() {
        // Test that enum variants exist
        let _table = OutputFormat::Table;
        let _json = OutputFormat::Json;
        let _yaml = OutputFormat::Yaml;
    }

    #[test]
    fn test_context_format_enum() {
        // Test that enum variants exist
        let _markdown = ContextFormat::Markdown;
        let _json = ContextFormat::Json;
    }

    #[test]
    fn test_commands_construction() {
        // Just test that we can construct the enum variants
        let _cmd = Commands::List {
            toolchain: None,
            category: None,
            format: OutputFormat::Table,
        };

        let _cmd = Commands::Search {
            query: "test".to_string(),
            toolchain: None,
            limit: 10,
        };
    }
}
