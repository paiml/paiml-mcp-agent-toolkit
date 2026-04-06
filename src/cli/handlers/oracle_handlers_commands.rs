/// Handle `pmat oracle fix` - Run PDCA fix loop
async fn handle_oracle_fix(
    path: &Path,
    max_iterations: usize,
    auto_apply_threshold: f32,
    review_threshold: f32,
    dry_run: bool,
    format: OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    println!("🔮 PMAT Oracle - PDCA Quality Improvement Loop");
    println!("   Path: {}", path.display());
    println!("   Max iterations: {}", max_iterations);
    println!(
        "   Thresholds: auto={:.2}, review={:.2}",
        auto_apply_threshold, review_threshold
    );
    if dry_run {
        println!("   Mode: DRY RUN (no changes will be applied)");
    }
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    // Create config
    let config = OracleConfig {
        max_iterations,
        auto_apply_threshold,
        review_threshold,
        ..Default::default()
    };
    let targets = ConvergenceTargets::default();

    // Create and run PDCA loop
    let pdca = PdcaLoop::with_config(config, targets.clone());

    if dry_run {
        println!("🔍 Dry run: Collecting signals only...\n");
        // Just run one iteration without applying fixes
        let results = pdca.run_iterations(path, 1).await?;
        if let Some(result) = results.first() {
            format_iteration_result(result, &format, output)?;
        }
    } else {
        println!("🚀 Starting PDCA loop...\n");
        let results = pdca.run(path).await?;

        // Format and output results
        let formatted = format_pdca_results(&results, &targets, format)?;

        if let Some(output_path) = output {
            std::fs::write(output_path, &formatted)?;
            println!("✅ Results written to: {}", output_path.display());
        } else {
            println!("{}", formatted);
        }
    }

    Ok(())
}

/// Handle `pmat oracle status` - Show current quality status
async fn handle_oracle_status(path: &Path, format: OracleOutputFormat) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    println!("📊 PMAT Oracle - Project Quality Status");
    println!("   Path: {}", path.display());
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let targets = ConvergenceTargets::default();

    // Collect current metrics (simplified for now - would integrate with actual PMAT commands)
    let metrics = collect_project_metrics(path).await?;
    let status = targets.check(&metrics);

    let formatted = format_status(&metrics, &targets, &status, format)?;
    println!("{}", formatted);

    Ok(())
}

/// Handle `pmat oracle single` - Run single PDCA iteration
async fn handle_oracle_single(
    path: &Path,
    format: OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    println!("⚡ PMAT Oracle - Single PDCA Iteration");
    println!("   Path: {}", path.display());
    println!();

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let pdca = PdcaLoop::new();
    let result = pdca.run_single(path).await?;

    let formatted = format_single_result(&result, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        println!("✅ Results written to: {}", output_path.display());
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

/// Collect project metrics (simplified implementation)
///
/// Returns default metrics. Full implementation would run:
/// - `pmat tdg` for TDG score
/// - `pmat analyze complexity` for cyclomatic/cognitive complexity
/// - `pmat analyze satd` for SATD markers
/// - `pmat analyze dead-code` for dead code items
/// - `cargo test` for test coverage/failures
///
/// Oracle-driven convergence uses these metrics to guide iterative improvements.
async fn collect_project_metrics(_path: &Path) -> Result<ProjectMetrics> {
    debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
    // Stub implementation - full metrics collection would be expensive
    // and is meant for CI/CD pipelines, not interactive use.
    Ok(ProjectMetrics::default())
}
