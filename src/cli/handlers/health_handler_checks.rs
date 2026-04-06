/// Run build health check
async fn run_build_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use crate::cli::progress::ProgressIndicator;

    // Check if Cargo.toml exists
    let cargo_toml = project_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        return Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".to_string(),
            details: None,
        });
    }

    // Show progress for build check
    let progress = ProgressIndicator::new("Running build check...");

    // Try to build
    let start = std::time::Instant::now();
    let output = tokio::process::Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(project_dir)
        .output()
        .await?;
    let duration = start.elapsed();

    if output.status.success() {
        progress.finish_with_message(&format!(
            "Build check passed ({:.1}s)",
            duration.as_secs_f64()
        ));
        Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Pass,
            message: "Project builds successfully".to_string(),
            details: None,
        })
    } else {
        progress.finish_with_error("Build check failed");
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(HealthCheck {
            name: "Build".to_string(),
            status: CheckStatus::Fail,
            message: "Build failed".to_string(),
            details: Some(stderr.lines().take(5).collect::<Vec<_>>().join("\n")),
        })
    }
}

/// Run test health check
async fn run_test_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use crate::cli::progress::ProgressIndicator;

    let progress = ProgressIndicator::new("Running tests...");
    let start = std::time::Instant::now();

    let output = tokio::process::Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .arg("--no-fail-fast")
        .current_dir(project_dir)
        .output()
        .await?;

    let duration = start.elapsed();

    if output.status.success() {
        progress.finish_with_message(&format!("Tests passed ({:.1}s)", duration.as_secs_f64()));
        Ok(HealthCheck {
            name: "Tests".to_string(),
            status: CheckStatus::Pass,
            message: "All tests passing".to_string(),
            details: None,
        })
    } else {
        progress.finish_with_error("Tests failed");
        Ok(HealthCheck {
            name: "Tests".to_string(),
            status: CheckStatus::Fail,
            message: "Some tests failing".to_string(),
            details: None,
        })
    }
}

/// Run coverage health check
async fn run_coverage_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use crate::cli::progress::ProgressIndicator;

    let progress = ProgressIndicator::new("Running coverage check...");
    let start = std::time::Instant::now();

    // Use cargo llvm-cov to get coverage
    let output = tokio::process::Command::new("cargo")
        .arg("llvm-cov")
        .arg("--quiet")
        .arg("--summary-only")
        .current_dir(project_dir)
        .output()
        .await;

    let duration = start.elapsed();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            // Parse coverage percentage from output
            let coverage = parse_coverage_percentage(&stdout);

            let status = if coverage >= 80.0 {
                CheckStatus::Pass
            } else if coverage >= 60.0 {
                CheckStatus::Warn
            } else {
                CheckStatus::Fail
            };

            progress.finish_with_message(&format!(
                "Coverage: {:.1}% ({:.1}s)",
                coverage,
                duration.as_secs_f64()
            ));

            Ok(HealthCheck {
                name: "Coverage".to_string(),
                status,
                message: format!("Coverage: {:.1}%", coverage),
                details: Some(format!("Target: ≥80%, Current: {:.1}%", coverage)),
            })
        }
        _ => {
            progress.finish_with_message("cargo-llvm-cov not available");
            Ok(HealthCheck {
                name: "Coverage".to_string(),
                status: CheckStatus::Skip,
                message: "cargo-llvm-cov not available".to_string(),
                details: Some("Install with: cargo install cargo-llvm-cov".to_string()),
            })
        }
    }
}

/// Run complexity health check
async fn run_complexity_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use crate::cli::progress::ProgressIndicator;

    let progress = ProgressIndicator::new("Running complexity check...");
    let start = std::time::Instant::now();

    // Detect project toolchain
    let toolchain = super::super::analysis_utilities::detect_toolchain(project_dir);

    // Analyze project files with default thresholds
    let include: Vec<String> = Vec::new();
    let metrics = super::super::analysis_utilities::analyze_project_files(
        project_dir,
        toolchain.as_deref(),
        &include,
        20, // cyclomatic threshold
        15, // cognitive threshold
    )
    .await;

    let duration = start.elapsed();

    match metrics {
        Ok(file_metrics) => {
            let (total_functions, violations, max_complexity) =
                count_complexity_violations(&file_metrics);

            let status = if violations == 0 {
                CheckStatus::Pass
            } else if violations <= 5 {
                CheckStatus::Warn
            } else {
                CheckStatus::Fail
            };

            progress.finish_with_message(&format!(
                "Complexity: {} functions, {} violations ({:.1}s)",
                total_functions,
                violations,
                duration.as_secs_f64()
            ));

            Ok(HealthCheck {
                name: "Complexity".to_string(),
                status,
                message: format!(
                    "{} functions analyzed, {} exceed threshold (max: {})",
                    total_functions, violations, max_complexity
                ),
                details: if violations > 0 {
                    Some("Run 'pmat analyze complexity' for detailed report".to_string())
                } else {
                    None
                },
            })
        }
        Err(_) => {
            progress.finish_with_message("Complexity check skipped");
            Ok(HealthCheck {
                name: "Complexity".to_string(),
                status: CheckStatus::Skip,
                message: "Could not analyze project complexity".to_string(),
                details: Some(
                    "Use 'pmat analyze complexity' for detailed analysis".to_string(),
                ),
            })
        }
    }
}

/// Count functions exceeding cyclomatic complexity threshold
fn count_complexity_violations(
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
) -> (usize, usize, u16) {
    let mut total_functions = 0;
    let mut violations = 0;
    let mut max_complexity: u16 = 0;

    for file in file_metrics {
        for func in &file.functions {
            total_functions += 1;
            if func.metrics.cyclomatic > max_complexity {
                max_complexity = func.metrics.cyclomatic;
            }
            if func.metrics.cyclomatic > 20 {
                violations += 1;
            }
        }
    }

    (total_functions, violations, max_complexity)
}

/// Run SATD health check
async fn run_satd_check(project_dir: &PathBuf) -> Result<HealthCheck> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    use crate::cli::progress::ProgressIndicator;
    use crate::services::detection::satd::Severity;
    use crate::services::detection::UnifiedDetectionProcessor;

    let progress = ProgressIndicator::new("Running SATD check...");
    let start = std::time::Instant::now();

    // Use unified detection processor for SATD analysis
    let processor = UnifiedDetectionProcessor::new();
    let result = processor.detect_satd(project_dir).await;

    let duration = start.elapsed();

    match result {
        Ok(satd_result) => {
            // Count SATD items by severity
            let total_items = satd_result.items.len();
            let high_severity = satd_result
                .items
                .iter()
                .filter(|m| matches!(m.severity, Severity::High | Severity::Critical))
                .count();

            let status = if total_items == 0 {
                CheckStatus::Pass
            } else if high_severity == 0 {
                CheckStatus::Warn
            } else {
                CheckStatus::Fail
            };

            progress.finish_with_message(&format!(
                "SATD: {} items found ({} high severity) ({:.1}s)",
                total_items,
                high_severity,
                duration.as_secs_f64()
            ));

            Ok(HealthCheck {
                name: "SATD".to_string(),
                status,
                message: format!(
                    "{} technical debt items ({} high severity)",
                    total_items, high_severity
                ),
                details: if total_items > 0 {
                    Some(format!(
                        "Files with debt: {}. Run 'pmat analyze satd' for details",
                        satd_result.files_with_debt
                    ))
                } else {
                    None
                },
            })
        }
        Err(_) => {
            progress.finish_with_message("SATD check skipped");
            Ok(HealthCheck {
                name: "SATD".to_string(),
                status: CheckStatus::Skip,
                message: "Could not analyze technical debt".to_string(),
                details: Some("Use 'pmat analyze satd' for detailed analysis".to_string()),
            })
        }
    }
}

/// Run multiple health checks in parallel (TICKET-PMAT-6010)
///
/// # Complexity
/// - Time: O(max(check_times)) instead of O(sum(check_times))
/// - Cyclomatic: 4
async fn run_checks_parallel(
    project_dir: &PathBuf,
    check_types: Vec<CheckType>,
) -> Result<Vec<HealthCheck>> {
    debug_assert!(project_dir.exists(), "project_dir must exist: {}", project_dir.display());
    let mut set = JoinSet::new();

    // Spawn parallel tasks for each check
    for check_type in check_types {
        let dir = project_dir.clone();
        set.spawn(async move {
            match check_type {
                CheckType::Build => run_build_check(&dir).await,
                CheckType::Tests => run_test_check(&dir).await,
                CheckType::Coverage => run_coverage_check(&dir).await,
                CheckType::Complexity => run_complexity_check(&dir).await,
                CheckType::Satd => run_satd_check(&dir).await,
            }
        });
    }

    // Collect results as they complete
    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res??);
    }

    Ok(results)
}
