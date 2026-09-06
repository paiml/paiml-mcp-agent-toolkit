// Extracted from quality_gates_handler.rs — gate execution and config management functions

/// Handle quality-gates command with subcommands
///
/// # Complexity
/// - Time: O(1) for subcommands, O(n) for gate execution
/// - Cyclomatic: 4
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    let config = load_config_from_file(&path.to_path_buf())?;

    match validate_config(&config) {
        Ok(()) => {
            println!("{}", validation_verdict(&[]));
            Ok(())
        }
        Err(errors) => {
            println!("{}", validation_verdict(&errors));
            std::process::exit(1);
        }
    }
}

/// The verdict `quality-gates validate` prints: one line when the file is
/// valid, a headline plus one line per error otherwise.
fn validation_verdict(errors: &[String]) -> String {
    use crate::cli::colors as c;
    if errors.is_empty() {
        return c::colored(c::GREEN, "✅ Configuration is valid");
    }
    let mut out = c::colored(c::RED, &format!("❌ Configuration has {} error(s):", errors.len()));
    for error in errors {
        out.push_str(&format!("\n  - {error}"));
    }
    out
}

#[cfg(test)]
mod verdict_tests {
    use super::validation_verdict;

    /// PMAT-688: `quality-gates validate --color always` emitted the same
    /// bytes as `--color auto`; the verdict was a plain `println!`.
    #[test]
    fn validation_verdict_carries_colour_when_forced() {
        let _on = crate::cli::colors::ForcedColor::on();
        let ok = validation_verdict(&[]);
        assert!(ok.contains("Configuration is valid") && ok.contains("\x1b["), "{ok:?}");
        let bad = validation_verdict(&["gates.max_complexity must be positive".to_string()]);
        assert!(bad.contains("1 error(s)") && bad.contains("\x1b["), "{bad:?}");
        assert!(bad.contains("  - gates.max_complexity must be positive"), "{bad}");
    }

    #[test]
    fn validation_verdict_is_plain_when_colours_are_off() {
        let _off = crate::cli::colors::ForcedColor::off();
        assert_eq!(validation_verdict(&[]), "✅ Configuration is valid");
        let bad = validation_verdict(&["a".to_string(), "b".to_string()]);
        assert_eq!(bad, "❌ Configuration has 2 error(s):\n  - a\n  - b");
    }
}


/// Show configuration (TICKET-PMAT-5024)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn handle_show_config(path: &Path, format: ConfigFormat) -> Result<()> {
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
    let content = std::fs::read_to_string(path)?;
    let config: GateConfigToml = toml::from_str(&content)?;
    Ok(config.into())
}
