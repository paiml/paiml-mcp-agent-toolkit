/// Whether decorative banners/progress lines may be printed to stdout.
///
/// In JSON mode stdout must contain ONLY the JSON payload (jq-parseable),
/// so banners are suppressed.
fn banner_enabled(format: OracleOutputFormat) -> bool {
    format != OracleOutputFormat::Json
}

/// Print the "results written" confirmation; stderr in JSON mode to keep
/// stdout reserved for the payload.
fn notify_results_written(output_path: &Path, format: OracleOutputFormat) {
    if banner_enabled(format) {
        println!("✅ Results written to: {}", output_path.display());
    } else {
        eprintln!("✅ Results written to: {}", output_path.display());
    }
}

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
    if banner_enabled(format) {
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
    }

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
        if banner_enabled(format) {
            println!("🔍 Dry run: Collecting signals only...\n");
        }
        // Just run one iteration without applying fixes
        let results = pdca.run_iterations(path, 1).await?;
        if let Some(result) = results.first() {
            format_iteration_result(result, &format, output)?;
        }
    } else {
        if banner_enabled(format) {
            println!("🚀 Starting PDCA loop...\n");
        }
        let results = pdca.run(path).await?;

        // Format and output results
        let formatted = format_pdca_results(&results, &targets, format)?;

        if let Some(output_path) = output {
            std::fs::write(output_path, &formatted)?;
            notify_results_written(output_path, format);
        } else {
            println!("{}", formatted);
        }
    }

    Ok(())
}

/// Handle `pmat oracle status` - Show current quality status
async fn handle_oracle_status(path: &Path, format: OracleOutputFormat) -> Result<()> {
    if banner_enabled(format) {
        println!("📊 PMAT Oracle - Project Quality Status");
        println!("   Path: {}", path.display());
        println!();
    }

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let targets = ConvergenceTargets::default();

    if banner_enabled(format) {
        println!("   Collecting signals (cargo build / clippy / test)...");
        println!();
    }

    let collected = collect_project_metrics(path).await?;
    let status = convergence_status_with_gaps(&targets, &collected);

    let formatted = format_status(&collected.metrics, &targets, &status, format)?;
    println!("{}", formatted);

    Ok(())
}

/// Metrics plus the names of the targets nothing actually measured.
///
/// `ProjectMetrics` has no "unknown" state — every field is a plain number —
/// so an unmeasured target reads as a perfect 0. Carrying the gaps alongside
/// lets the status report say so instead.
struct CollectedMetrics {
    metrics: ProjectMetrics,
    unmeasured: Vec<&'static str>,
}

/// Convergence cannot be declared over targets that were never measured, so
/// every gap is folded into the "remaining" list and Converged is suppressed.
fn convergence_status_with_gaps(
    targets: &ConvergenceTargets,
    collected: &CollectedMetrics,
) -> crate::services::oracle::ConvergenceStatus {
    use crate::services::oracle::ConvergenceStatus;

    let checked = targets.check(&collected.metrics);
    if collected.unmeasured.is_empty() {
        return checked;
    }

    let mut remaining = match checked {
        ConvergenceStatus::Converged => Vec::new(),
        ConvergenceStatus::NotConverged { remaining } => remaining,
    };
    for gap in &collected.unmeasured {
        remaining.push(format!("{}: not measured", gap));
    }
    ConvergenceStatus::NotConverged { remaining }
}

/// Collect project metrics from the same signal collectors `oracle single`
/// runs.
///
/// This was `async fn collect_project_metrics(_path: &Path) -> ... {
/// Ok(ProjectMetrics::default()) }` — the path was discarded, so an empty
/// directory, a crate that does not compile, and this 4000-file repo all
/// reported "Compiler Errors: 0 (target: ≤0)" in an identical 10 ms. A crate
/// that fails to build must never be able to reach a green compiler-error
/// line. Coverage, mutation score, TDG and the Rust project score still have
/// no collector behind them here; they are reported as gaps rather than as 0.
async fn collect_project_metrics(path: &Path) -> Result<CollectedMetrics> {
    use crate::services::oracle::{AggregatedCollector, SignalSource};

    let mut metrics = ProjectMetrics::default();
    let mut unmeasured = vec![
        "test coverage",
        "mutation score",
        "TDG score",
        "rust project score",
        "SATD markers",
        "dead code",
        "complexity",
    ];

    // Without a manifest cargo cannot run at all; reporting its silence as
    // "0 errors" is exactly the fabrication this replaced.
    if !path.join("Cargo.toml").exists() {
        unmeasured.push("compiler errors");
        unmeasured.push("clippy warnings");
        unmeasured.push("test failures");
        return Ok(CollectedMetrics {
            metrics,
            unmeasured,
        });
    }

    let signals = AggregatedCollector::new().collect_all(path).await?;
    metrics.compiler_errors = signals
        .iter()
        .filter(|s| s.source == SignalSource::Rustc)
        .count();
    metrics.clippy_warnings = signals
        .iter()
        .filter(|s| s.source == SignalSource::Clippy)
        .count();
    metrics.test_failures = signals
        .iter()
        .filter(|s| s.source == SignalSource::CargoTest)
        .count();

    Ok(CollectedMetrics {
        metrics,
        unmeasured,
    })
}

#[cfg(test)]
mod oracle_status_metric_tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_project_metrics_reports_gaps_without_a_manifest() {
        // Regression: this returned ProjectMetrics::default() for every path,
        // so a directory cargo cannot even be pointed at reported a clean
        // "Compiler Errors: 0".
        let dir = tempfile::TempDir::new().unwrap();
        let collected = collect_project_metrics(dir.path()).await.unwrap();

        assert!(
            collected.unmeasured.contains(&"compiler errors"),
            "cargo cannot run without a manifest: {:?}",
            collected.unmeasured
        );
        assert!(collected.unmeasured.contains(&"test coverage"));
    }

    #[test]
    fn test_unmeasured_targets_block_convergence() {
        use crate::services::oracle::ConvergenceStatus;

        // A metrics block that satisfies every default target on paper.
        let targets = ConvergenceTargets::default();
        let metrics = ProjectMetrics {
            test_coverage: 1.0,
            mutation_score: 1.0,
            tdg_score: 100.0,
            rust_project_score: 100,
            ..Default::default()
        };

        let all_measured = CollectedMetrics {
            metrics: metrics.clone(),
            unmeasured: Vec::new(),
        };
        assert!(matches!(
            convergence_status_with_gaps(&targets, &all_measured),
            ConvergenceStatus::Converged
        ));

        let with_gap = CollectedMetrics {
            metrics,
            unmeasured: vec!["TDG score"],
        };
        match convergence_status_with_gaps(&targets, &with_gap) {
            ConvergenceStatus::Converged => {
                panic!("must not declare convergence over a target nothing measured")
            }
            ConvergenceStatus::NotConverged { remaining } => {
                assert!(remaining.iter().any(|r| r.contains("TDG score: not measured")));
            }
        }
    }
}

/// Handle `pmat oracle single` - Run single PDCA iteration
async fn handle_oracle_single(
    path: &Path,
    format: OracleOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    if banner_enabled(format) {
        println!("⚡ PMAT Oracle - Single PDCA Iteration");
        println!("   Path: {}", path.display());
        println!();
    }

    // Validate path
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let pdca = PdcaLoop::new();
    let result = pdca.run_single(path).await?;

    let formatted = format_single_result(&result, format)?;

    if let Some(output_path) = output {
        std::fs::write(output_path, &formatted)?;
        notify_results_written(output_path, format);
    } else {
        println!("{}", formatted);
    }

    Ok(())
}

