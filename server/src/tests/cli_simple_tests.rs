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
#[cfg(feature = "installer-gen")]
mod installer_simple_tests {
    use crate::installer::{Error as InstallerError, ShellContext};

    #[test]
    fn test_shell_context_basic() {
        let ctx = ShellContext;

        // Test simple command
        let result = ctx.command("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_shell_context_test_dir() {
        let ctx = ShellContext;

        // Test current directory exists
        assert!(ctx.test_dir("."));

        // Test non-existent directory
        assert!(!ctx.test_dir("/nonexistent/path/12345"));
    }

    #[test]
    fn test_installer_error_display() {
        let err = InstallerError::UnsupportedPlatform("test".to_string());
        assert_eq!(err.to_string(), "Unsupported platform: test");

        let err = InstallerError::DownloadFailed("test".to_string());
        assert_eq!(err.to_string(), "Download failed: test");

        let err = InstallerError::ChecksumMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert_eq!(err.to_string(), "Checksum mismatch: expected abc, got def");

        let err = InstallerError::InstallFailed("test".to_string());
        assert_eq!(err.to_string(), "Installation failed: test");

        let err = InstallerError::CommandFailed("test".to_string());
        assert_eq!(err.to_string(), "Command failed: test");
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
