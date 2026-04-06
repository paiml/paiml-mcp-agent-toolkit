// Mutating operations on configuration: set values, edit interactively, reset to defaults.
// Includes: set_configuration_values, edit_configuration, reset_configuration.

/// Set configuration values
async fn set_configuration_values(
    config_service: &ConfigurationService,
    set_values: Vec<String>,
) -> Result<()> {
    info!("Updating configuration values");

    for set_value in set_values {
        let parts: Vec<&str> = set_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid format '{set_value}'. Use key=value"
            ));
        }

        let key = parts[0];
        let value = parts[1];

        println!("Setting {key}: {value}");

        config_service
            .update_config(|config| set_config_value(config, key, value))
            .await?;
    }

    println!("Configuration updated successfully");
    Ok(())
}

/// Edit configuration interactively
async fn edit_configuration(config_service: &ConfigurationService) -> Result<()> {
    println!("Interactive Configuration Editor");
    println!("{}", "=".repeat(40));
    println!();

    let config = config_service.get_config()?;
    let toml_content = toml::to_string_pretty(&config)?;

    // Create temporary file with current config
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("pmat_config.toml");
    tokio::fs::write(&temp_file, &toml_content).await?;

    // Detect editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    println!("Opening configuration in {editor} editor...");
    println!("Save and exit to apply changes, or exit without saving to cancel.");
    println!();

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&temp_file)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with error"));
    }

    // Read edited content
    let edited_content = tokio::fs::read_to_string(&temp_file).await?;

    // Parse and validate
    let new_config: PmatConfig = toml::from_str(&edited_content)?;

    // Update configuration
    config_service
        .update_config(|config| {
            *config = new_config;
            Ok(())
        })
        .await?;

    println!("Configuration updated successfully");
    Ok(())
}

/// Reset configuration to defaults
async fn reset_configuration(config_service: &ConfigurationService) -> Result<()> {
    info!("Resetting configuration to defaults");

    config_service
        .update_config(|config| {
            *config =
                crate::services::configuration_service::ConfigurationService::default_config();
            Ok(())
        })
        .await?;

    Ok(())
}
