// Display and formatting functions for configuration commands.
// Includes: show_configuration_overview, show_configuration, show_configuration_section.

/// Show configuration overview
async fn show_configuration_overview(config_service: &ConfigurationService) -> Result<()> {
    info!("Generating configuration overview");

    let config = config_service.get_config()?;

    println!("PMAT Configuration Overview");
    println!("{}", "=".repeat(35));
    println!();

    println!("Configuration Source:");
    let config_path = std::env::current_dir()?.join("pmat.toml");
    if config_path.exists() {
        println!("  File: {}", config_path.display());
    } else {
        println!("  File: {} (default)", config_path.display());
    }
    println!();

    println!("System Settings:");
    println!("  Project: {}", config.system.project_name);
    println!("  Toolchain: {}", config.system.default_toolchain);
    println!(
        "  Parallel: {} threads",
        config.system.max_concurrent_operations
    );
    println!();

    println!("Quality Gates:");
    println!("  Max Complexity: {}", config.quality.max_complexity);
    println!("  Min Coverage: {}%", config.quality.min_coverage);
    println!("  Allow SATD: {}", config.quality.allow_satd);
    println!("  Require Docs: {}", config.quality.require_docs);
    println!();

    println!("Analysis Settings:");
    println!("  Include: {:?}", config.analysis.include_patterns);
    println!("  Exclude: {:?}", config.analysis.exclude_patterns);
    println!("  Parallel: {}", config.analysis.parallel);
    println!("  Timeout: {}s", config.analysis.timeout_seconds);
    println!();

    println!("Performance Targets:");
    println!(
        "  Startup: {}ms",
        config.performance.target_startup_latency_ms
    );
    println!(
        "  Throughput: {} LOC/s",
        config.performance.target_throughput_loc_per_sec
    );
    println!("  Memory: {}MB", config.performance.target_memory_mb);
    println!();

    println!("MCP Server:");
    println!("  Name: {}", config.mcp.server_name);
    println!("  Version: {}", config.mcp.server_version);
    println!("  Tools: {} enabled", config.mcp.enabled_tools.len());
    println!();

    println!("Roadmap:");
    println!("  Path: {}", config.roadmap.roadmap_path.display());
    println!("  Auto Todos: {}", config.roadmap.auto_generate_todos);
    println!("  Quality Gates: {}", config.roadmap.enforce_quality_gates);
    println!();

    println!("Telemetry:");
    println!("  Enabled: {}", config.telemetry.enabled);
    println!(
        "  Interval: {}s",
        config.telemetry.collection_interval_seconds
    );
    println!();

    println!("Commands:");
    println!("  pmat config --show --section quality    # Show quality settings");
    println!("  pmat config --set quality.max_complexity=25  # Update setting");
    println!("  pmat config --edit                      # Interactive edit");
    println!("  pmat config --validate                  # Validate config");

    Ok(())
}

/// Show detailed configuration
async fn show_configuration(
    config_service: &ConfigurationService,
    section: Option<String>,
) -> Result<()> {
    let config = config_service.get_config()?;

    if let Some(section_name) = section {
        show_configuration_section(&config, &section_name)?;
    } else {
        println!("Complete PMAT Configuration");
        println!("{}", "=".repeat(50));
        println!();

        println!("Raw Configuration (TOML):");
        let toml_content = toml::to_string_pretty(&config)?;
        println!("{toml_content}");

        println!("JSON Format:");
        println!("{}", serde_json::to_string_pretty(&config)?);
    }

    Ok(())
}

/// Show specific configuration section
fn show_configuration_section(config: &PmatConfig, section: &str) -> Result<()> {
    println!("Configuration Section: {section}");
    println!("{}", "=".repeat(30 + section.len()));
    println!();

    match section.to_lowercase().as_str() {
        "system" => {
            println!("System Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.system)?);
        }
        "quality" => {
            println!("Quality Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.quality)?);
        }
        "analysis" => {
            println!("Analysis Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.analysis)?);
        }
        "performance" => {
            println!("Performance Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.performance)?);
        }
        "mcp" => {
            println!("MCP Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.mcp)?);
        }
        "roadmap" => {
            println!("Roadmap Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.roadmap)?);
        }
        "telemetry" => {
            println!("Telemetry Configuration (TOML):");
            println!("{}", toml::to_string_pretty(&config.telemetry)?);
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown section '{section}'. Available: system, quality, analysis, performance, mcp, roadmap, telemetry"
            ));
        }
    }

    Ok(())
}
