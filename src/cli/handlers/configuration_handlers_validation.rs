// Validation logic for configuration sections.
// Includes: validate_configuration, validate_all_sections, per-section validators,
// report_validation_results, and print_configuration_statistics.

/// Validate configuration
async fn validate_configuration(config_service: &ConfigurationService) -> Result<()> {
    info!("Validating configuration");

    println!("PMAT Configuration Validation");
    println!("{}", "=".repeat(40));
    println!();

    let config = config_service.get_config()?;
    let mut issues = Vec::new();

    validate_all_sections(&config, &mut issues);
    report_validation_results(&config, &issues)?;

    Ok(())
}

fn validate_all_sections(config: &PmatConfig, issues: &mut Vec<&'static str>) {
    validate_system_config(&config.system, issues);
    validate_quality_config(&config.quality, issues);
    validate_analysis_config(&config.analysis, issues);
    validate_performance_config(&config.performance, issues);
    validate_mcp_config(&config.mcp, issues);
}

fn validate_system_config(
    system_config: &crate::services::configuration_service::SystemConfig,
    issues: &mut Vec<&'static str>,
) {
    if system_config.project_name.is_empty() {
        issues.push("System: project_name cannot be empty");
    }

    if system_config.max_concurrent_operations == 0 {
        issues.push("System: max_concurrent_operations must be > 0");
    }
}

fn validate_quality_config(
    quality_config: &crate::services::configuration_service::QualityConfig,
    issues: &mut Vec<&'static str>,
) {
    if quality_config.max_complexity == 0 {
        issues.push("Quality: max_complexity must be > 0");
    }

    if quality_config.min_coverage > 100.0 || quality_config.min_coverage < 0.0 {
        issues.push("Quality: min_coverage must be between 0 and 100");
    }
}

fn validate_analysis_config(
    analysis_config: &crate::services::configuration_service::AnalysisConfig,
    issues: &mut Vec<&'static str>,
) {
    if analysis_config.max_file_size == 0 {
        issues.push("Analysis: max_file_size must be > 0");
    }

    if analysis_config.timeout_seconds == 0 {
        issues.push("Analysis: timeout_seconds must be > 0");
    }
}

fn validate_performance_config(
    performance_config: &crate::services::configuration_service::PerformanceConfig,
    issues: &mut Vec<&'static str>,
) {
    if performance_config.test_iterations == 0 {
        issues.push("Performance: test_iterations must be > 0");
    }
}

fn validate_mcp_config(
    mcp_config: &crate::services::configuration_service::McpConfig,
    issues: &mut Vec<&'static str>,
) {
    if mcp_config.server_name.is_empty() {
        issues.push("MCP: server_name cannot be empty");
    }

    if mcp_config.request_timeout_seconds == 0 {
        issues.push("MCP: request_timeout_seconds must be > 0");
    }
}

fn report_validation_results(config: &PmatConfig, issues: &[&str]) -> Result<()> {
    if issues.is_empty() {
        report_validation_success();
    } else {
        report_validation_failure(issues)?;
    }

    print_configuration_statistics(config);
    Ok(())
}

fn report_validation_success() {
    println!("Configuration is valid");
    println!("   All settings are within acceptable ranges");
    println!("   No issues detected");
}

fn report_validation_failure(issues: &[&str]) -> Result<()> {
    println!("Configuration validation failed");
    println!("   Found {} issues:", issues.len());
    for issue in issues {
        println!("   - {issue}");
    }
    Err(anyhow::anyhow!("Configuration validation failed"))
}

fn print_configuration_statistics(config: &PmatConfig) {
    println!();
    println!("Configuration Statistics:");
    println!("   Sections: 7");
    println!("   Total Settings: ~50");
    println!("   Custom Settings: {}", config.custom.len());
}
