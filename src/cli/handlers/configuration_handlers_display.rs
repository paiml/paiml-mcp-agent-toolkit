// Display and formatting functions for configuration commands.
// Includes: show_configuration_overview, show_configuration, show_configuration_section.

/// Show configuration overview
async fn show_configuration_overview(config_service: &ConfigurationService) -> Result<()> {
    use crate::cli::colors as c;

    info!("Generating configuration overview");

    let config = config_service.get_config()?;

    println!("{}", c::header("PMAT Configuration Overview"));
    println!("{}", c::rule());
    println!();

    println!("{}", c::subheader("Configuration Source:"));
    let config_path = std::env::current_dir()?.join("pmat.toml");
    if config_path.exists() {
        println!("  {}: {}", c::label("File"), c::path(&config_path.display().to_string()));
    } else {
        println!("  {}: {} {}", c::label("File"), c::path(&config_path.display().to_string()), c::dim("(default)"));
    }
    println!();

    println!("{}", c::subheader("System Settings:"));
    println!("  {}: {}", c::label("Project"), c::number(&config.system.project_name));
    println!("  {}: {}", c::label("Toolchain"), c::number(&config.system.default_toolchain));
    println!(
        "  {}: {} threads",
        c::label("Parallel"),
        c::number(&config.system.max_concurrent_operations.to_string())
    );
    println!();

    println!("{}", c::subheader("Quality Gates:"));
    println!("  {}: {}", c::label("Max Complexity"), c::number(&config.quality.max_complexity.to_string()));
    println!("  {}: {}", c::label("Min Coverage"), c::pct(config.quality.min_coverage, 80.0, 60.0));
    println!(
        "  {}: {}",
        c::label("Allow SATD"),
        if config.quality.allow_satd { format!("{}{}{}", c::GREEN, config.quality.allow_satd, c::RESET) } else { format!("{}{}{}", c::RED, config.quality.allow_satd, c::RESET) }
    );
    println!(
        "  {}: {}",
        c::label("Require Docs"),
        if config.quality.require_docs { format!("{}{}{}", c::GREEN, config.quality.require_docs, c::RESET) } else { format!("{}{}{}", c::RED, config.quality.require_docs, c::RESET) }
    );
    println!();

    println!("{}", c::subheader("Analysis Settings:"));
    println!("  {}: {:?}", c::label("Include"), config.analysis.include_patterns);
    println!("  {}: {:?}", c::label("Exclude"), config.analysis.exclude_patterns);
    println!(
        "  {}: {}",
        c::label("Parallel"),
        if config.analysis.parallel { format!("{}{}{}", c::GREEN, config.analysis.parallel, c::RESET) } else { format!("{}{}{}", c::RED, config.analysis.parallel, c::RESET) }
    );
    println!("  {}: {}s", c::label("Timeout"), c::number(&config.analysis.timeout_seconds.to_string()));
    println!();

    println!("{}", c::subheader("Performance Targets:"));
    println!(
        "  {}: {}ms",
        c::label("Startup"),
        c::number(&config.performance.target_startup_latency_ms.to_string())
    );
    println!(
        "  {}: {} LOC/s",
        c::label("Throughput"),
        c::number(&config.performance.target_throughput_loc_per_sec.to_string())
    );
    println!("  {}: {}MB", c::label("Memory"), c::number(&config.performance.target_memory_mb.to_string()));
    println!();

    println!("{}", c::subheader("MCP Server:"));
    println!("  {}: {}", c::label("Name"), c::number(&config.mcp.server_name));
    println!("  {}: {}", c::label("Version"), c::number(&config.mcp.server_version));
    println!("  {}: {} enabled", c::label("Tools"), c::number(&config.mcp.enabled_tools.len().to_string()));
    println!();

    println!("{}", c::subheader("Roadmap:"));
    println!("  {}: {}", c::label("Path"), c::path(&config.roadmap.roadmap_path.display().to_string()));
    println!(
        "  {}: {}",
        c::label("Auto Todos"),
        if config.roadmap.auto_generate_todos { format!("{}{}{}", c::GREEN, config.roadmap.auto_generate_todos, c::RESET) } else { format!("{}{}{}", c::RED, config.roadmap.auto_generate_todos, c::RESET) }
    );
    println!(
        "  {}: {}",
        c::label("Quality Gates"),
        if config.roadmap.enforce_quality_gates { format!("{}{}{}", c::GREEN, config.roadmap.enforce_quality_gates, c::RESET) } else { format!("{}{}{}", c::RED, config.roadmap.enforce_quality_gates, c::RESET) }
    );
    println!();

    println!("{}", c::subheader("Telemetry:"));
    println!(
        "  {}: {}",
        c::label("Enabled"),
        if config.telemetry.enabled { format!("{}{}{}", c::GREEN, config.telemetry.enabled, c::RESET) } else { format!("{}{}{}", c::RED, config.telemetry.enabled, c::RESET) }
    );
    println!(
        "  {}: {}s",
        c::label("Interval"),
        c::number(&config.telemetry.collection_interval_seconds.to_string())
    );
    println!();

    println!("{}", c::subheader("Commands:"));
    println!("  {} {}", c::dim("#"), c::dim("Show quality settings"));
    println!("  pmat config --show --section quality");
    println!("  {} {}", c::dim("#"), c::dim("Update setting"));
    println!("  pmat config --set quality.max_complexity=25");
    println!("  {} {}", c::dim("#"), c::dim("Interactive edit"));
    println!("  pmat config --edit");
    println!("  {} {}", c::dim("#"), c::dim("Validate config"));
    println!("  pmat config --validate");

    Ok(())
}

/// Show detailed configuration
async fn show_configuration(
    config_service: &ConfigurationService,
    section: Option<String>,
) -> Result<()> {
    use crate::cli::colors as c;

    let config = config_service.get_config()?;

    if let Some(section_name) = section {
        show_configuration_section(&config, &section_name)?;
    } else {
        println!("{}", c::header("Complete PMAT Configuration"));
        println!("{}", c::rule());
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
    debug_assert!(!section.is_empty(), "section must not be empty");
    use crate::cli::colors as c;

    println!("{}", c::header(&format!("Configuration Section: {section}")));
    println!("{}", c::rule());
    println!();

    match section.to_lowercase().as_str() {
        "system" => {
            println!("{}:", c::subheader("System Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.system)?);
        }
        "quality" => {
            println!("{}:", c::subheader("Quality Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.quality)?);
        }
        "analysis" => {
            println!("{}:", c::subheader("Analysis Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.analysis)?);
        }
        "performance" => {
            println!("{}:", c::subheader("Performance Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.performance)?);
        }
        "mcp" => {
            println!("{}:", c::subheader("MCP Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.mcp)?);
        }
        "roadmap" => {
            println!("{}:", c::subheader("Roadmap Configuration (TOML)"));
            println!("{}", toml::to_string_pretty(&config.roadmap)?);
        }
        "telemetry" => {
            println!("{}:", c::subheader("Telemetry Configuration (TOML)"));
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
