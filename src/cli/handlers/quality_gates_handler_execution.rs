// Extracted from quality_gates_handler.rs — gate execution and config management functions

/// Handle quality-gates command with subcommands
///
/// # Complexity
/// - Time: O(1) for subcommands, O(n) for gate execution
/// - Cyclomatic: 4
pub async fn handle_quality_gates_command(
    command: Option<QualityGatesCommand>,
    config_path: PathBuf,
    report: bool,
    json: bool,
    project_dir: PathBuf,
) -> Result<()> {
    match command {
        Some(QualityGatesCommand::Init { force }) => {
            handle_init_config(&config_path, force)?;
        }
        Some(QualityGatesCommand::Validate) => {
            handle_validate_config(&config_path)?;
        }
        Some(QualityGatesCommand::Show { format }) => {
            handle_show_config(&config_path, format)?;
        }
        None => {
            // Original behavior: run gates
            run_quality_gates(config_path, report, json, project_dir).await?;
        }
    }
    Ok(())
}

/// Run quality gates (original functionality from TICKET-PMAT-5023)
///
/// # Complexity
/// - Time: O(n) where n is codebase size
/// - Cyclomatic: 5
async fn run_quality_gates(
    config_path: PathBuf,
    report: bool,
    json: bool,
    project_dir: PathBuf,
) -> Result<()> {
    debug_assert!(config_path.exists(), "config_path must exist: {}", config_path.display());
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    // Load configuration
    let config = if config_path.exists() {
        load_config_from_file(&config_path)?
    } else {
        GateConfig::default()
    };

    // Run quality gates
    let gate_report = execute_all_gates(&config, &project_dir)?;

    // Output results
    if json {
        output_json(&gate_report)?;
    } else if report {
        output_markdown(&gate_report)?;
    } else {
        output_summary(&gate_report)?;
    }

    // Exit with appropriate code
    if gate_report.passed {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Initialize configuration file (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_init_config(path: &Path, force: bool) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Configuration file already exists. Use --force to overwrite."
        ));
    }

    let config = generate_default_config();
    std::fs::write(path, config)?;

    println!("✅ Created {}", path.display());
    Ok(())
}

/// Validate configuration file (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_validate_config(path: &Path) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let config = load_config_from_file(&path.to_path_buf())?;

    match validate_config(&config) {
        Ok(()) => {
            println!("✅ Configuration is valid");
            Ok(())
        }
        Err(errors) => {
            println!("❌ Configuration has {} error(s):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}

/// Show configuration (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_show_config(path: &Path, format: ConfigFormat) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let config = load_config_from_file(&path.to_path_buf())?;

    match format {
        ConfigFormat::Toml => {
            let toml = generate_config_toml(&config);
            println!("{}", toml);
        }
        ConfigFormat::Json => {
            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
        ConfigFormat::Env => {
            // Not applicable for quality gates config
            return Err(anyhow::anyhow!(
                "Environment variable format not supported for quality gates configuration"
            ));
        }
    }

    Ok(())
}

/// Load gate configuration from TOML file
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn load_config_from_file(path: &PathBuf) -> Result<GateConfig> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let content = std::fs::read_to_string(path)?;
    let config: GateConfigToml = toml::from_str(&content)?;
    Ok(config.into())
}
