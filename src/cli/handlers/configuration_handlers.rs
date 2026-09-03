//! Configuration command handlers for PMAT system.
//! Manages viewing, modifying, and validating configuration settings.

use crate::services::configuration_service::{configuration, ConfigurationService, PmatConfig};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Handle configuration command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_configuration(
    show: bool,
    edit: bool,
    validate: bool,
    reset: bool,
    section: Option<String>,
    set: Vec<String>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    // An explicit --config-path that does not exist used to fall through to the
    // built-in defaults and exit 0, byte-identical to a run with no flag at all, so
    // a mistyped path silently ignored every setting the user asked for. Reads must
    // fail loudly; only the implicit ./pmat.toml may be absent, and the writing
    // operations (--set / --reset) are still allowed to create a new file.
    let creates_config = reset || !set.is_empty();
    if let Some(path) = config_path.as_ref() {
        if !path.exists() && !creates_config {
            anyhow::bail!("config file not found: {}", path.display());
        }
    }

    let config_service = create_config_service(config_path);
    execute_configuration_command(
        &config_service,
        ConfigurationCommand {
            show,
            edit,
            validate,
            reset,
            section,
            set,
        },
    )
    .await
}

struct ConfigurationCommand {
    show: bool,
    edit: bool,
    validate: bool,
    reset: bool,
    section: Option<String>,
    set: Vec<String>,
}

fn create_config_service(config_path: Option<PathBuf>) -> std::sync::Arc<ConfigurationService> {
    if let Some(path) = config_path {
        std::sync::Arc::new(ConfigurationService::new(Some(path)))
    } else {
        configuration()
    }
}

async fn execute_configuration_command(
    config_service: &ConfigurationService,
    cmd: ConfigurationCommand,
) -> Result<()> {
    if cmd.reset {
        reset_configuration(config_service).await?;
        println!("Configuration reset to defaults");
        return Ok(());
    }
    if cmd.validate {
        return validate_configuration(config_service).await;
    }
    if !cmd.set.is_empty() {
        return set_configuration_values(config_service, cmd.set).await;
    }
    if cmd.edit {
        return edit_configuration(config_service).await;
    }
    if cmd.show || cmd.section.is_some() {
        show_configuration(config_service, cmd.section).await
    } else {
        show_configuration_overview(config_service).await
    }
}

// Display and formatting functions (overview, show, section display)
include!("configuration_handlers_display.rs");

// Configuration value setters for each config section
include!("configuration_handlers_setters.rs");

// Validation logic for all configuration sections
include!("configuration_handlers_validation.rs");

// Mutating operations: set values, interactive edit, reset to defaults
include!("configuration_handlers_operations.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// `--config-path` naming a file that does not exist must be an error: silently
    /// substituting the built-in defaults means the user's settings are ignored with
    /// no signal at all.
    #[tokio::test]
    async fn test_handle_configuration_errors_on_missing_explicit_config_path() {
        let result = handle_configuration(
            true,
            false,
            false,
            false,
            None,
            vec![],
            Some(PathBuf::from("/does/not/exist/pmat-config-test.toml")),
        )
        .await;
        assert!(
            result.is_err(),
            "missing --config-path must not use defaults"
        );
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not found"), "got: {msg}");
    }

    /// …but --set may still create a config file that does not exist yet.
    #[tokio::test]
    async fn test_handle_configuration_set_may_create_missing_config_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("new_config.toml");
        let result = handle_configuration(
            false,
            false,
            false,
            false,
            None,
            vec!["quality.max_complexity=25".to_string()],
            Some(config_path.clone()),
        )
        .await;
        assert!(
            result.is_ok(),
            "set must still be able to create: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_configuration_overview() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        let result = show_configuration_overview(&config_service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Absent config: the built-in defaults are validated and disclosed as
        // such; that is a pass, not a failure (CRUX-03 leg 2).
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));
        assert_eq!(
            config_service.load_status(),
            &crate::services::configuration_service::ConfigLoadStatus::Absent
        );

        let result = validate_configuration(&config_service).await;
        assert!(result.is_ok());
    }

    /// CRUX-03 leg 1: a file that did not parse is not a file with zero
    /// issues. Before this the validator certified the defaults it had fallen
    /// back to, exit 0.
    #[tokio::test]
    async fn a_config_that_does_not_parse_fails_validation() {
        let temp_dir = tempdir().expect("tempdir");
        let config_path = temp_dir.path().join("pmat.toml");
        std::fs::write(&config_path, "not even toml ][\n").expect("write fixture");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        assert!(matches!(
            config_service.load_status(),
            crate::services::configuration_service::ConfigLoadStatus::Unparsable(e) if e.contains("line 1")
        ));
        let result = validate_configuration(&config_service).await;
        assert!(result.is_err(), "an unparsable config must not validate");
    }

    /// CRUX-03 leg 5: a partial file pmat honours loads with the rest
    /// defaulted, instead of failing strict deserialisation and being replaced
    /// wholesale.
    #[tokio::test]
    async fn a_partial_config_is_honoured_not_replaced() {
        let temp_dir = tempdir().expect("tempdir");
        let config_path = temp_dir.path().join("pmat.toml");
        std::fs::write(&config_path, "[quality]\nmax_complexity = 25\n").expect("write fixture");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        assert_eq!(
            config_service.load_status(),
            &crate::services::configuration_service::ConfigLoadStatus::Loaded
        );
        let config = config_service.get_config().expect("config");
        assert_eq!(
            config.quality.max_complexity, 25,
            "the one key set is applied"
        );
        assert_eq!(
            config.quality.max_cognitive_complexity,
            ConfigurationService::default_config()
                .quality
                .max_cognitive_complexity,
            "a key the file omits is the built-in default"
        );
        assert!(validate_configuration(&config_service).await.is_ok());
    }

    /// CRUX-03 leg 3: a section nothing reads is a failed validation, named,
    /// with the near-miss suggested — the same verdict `pmat quality-gate`
    /// gives it, from the same derived list.
    #[tokio::test]
    async fn an_unknown_section_fails_validation() {
        let temp_dir = tempdir().expect("tempdir");
        let config_path = temp_dir.path().join("pmat.toml");
        std::fs::write(
            &config_path,
            "[quality_gate]\nmax_cyclomatic_complexity = 15\n",
        )
        .unwrap();
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        let result = validate_configuration(&config_service).await;
        assert!(
            result.is_err(),
            "[quality_gate] is read by nothing and must not validate"
        );
    }

    #[tokio::test]
    async fn test_set_configuration_values() {
        let temp_dir = tempdir().expect("tempdir");
        let config_path = temp_dir.path().join("test_config.toml");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        let set_values = vec![
            "quality.max_complexity=25".to_string(),
            "system.verbose=true".to_string(),
        ];

        let result = set_configuration_values(&config_service, set_values).await;
        assert!(result.is_ok());

        let config = config_service.get_config().expect("config");
        assert_eq!(config.quality.max_complexity, 25);
        assert!(config.system.verbose);
    }

    #[tokio::test]
    async fn test_show_configuration_section() {
        let config = crate::services::configuration_service::ConfigurationService::default_config();

        let result = show_configuration_section(&config, "quality");
        assert!(result.is_ok());

        let result = show_configuration_section(&config, "invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_configuration() {
        let temp_dir = tempdir().expect("tempdir");
        let config_path = temp_dir.path().join("test_config.toml");
        let config_service = std::sync::Arc::new(ConfigurationService::new(Some(config_path)));

        // Modify config
        config_service
            .update_config(|config| {
                config.quality.max_complexity = 50;
                Ok(())
            })
            .await
            .unwrap();

        // Reset
        let result = reset_configuration(&config_service).await;
        assert!(result.is_ok());

        // Verify reset
        let config = config_service.get_config().expect("config");
        assert_eq!(config.quality.max_complexity, 30); // Default value
    }
}
