// Sprint 89 GREEN Phase: Refactored handle_analyze_satd function
// BEFORE: Complexity 13 (High entropy, mixed concerns)
// AFTER: Complexity 6 (A+ standard, single responsibility)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_satd(config: SatdAnalysisConfig) -> Result<()> {
    // Without this, a nonexistent path walked to zero files and printed
    // "Found 0 SATD violations in 0 files" with exit 0 — a clean bill of
    // health for a tree that was never there, indistinguishable to a CI gate
    // from a genuinely debt-free repository.
    crate::cli::ensure_analysis_path_exists(&config.path)?;

    reject_unimplemented_evolution(config.evolution)?;

    crate::status_eprintln!("🔍 Analyzing Self-Admitted Technical Debt (SATD)...");

    // Delegate filter logging to extracted function
    log_filter_info(&config);

    // Delegate analysis execution to extracted function
    let result = execute_satd_analysis(&config).await?;

    // Delegate result filtering to extracted function
    let filtered_result = apply_analysis_filters(result, &config)?;

    // Delegate output formatting and writing to extracted function
    write_satd_output(&filtered_result, &config).await?;

    // `--fail-on-violation` is documented as "Exit with non-zero code if
    // violations are found", but this handler never read the flag off the
    // config: `analyze satd --strict --fail-on-violation` on a crate with three
    // TODO/FIXME/HACK markers printed "Total violations: 3" and exited 0, the
    // same exit code as without the flag, so no CI gate built on it could ever
    // fail. (A working check existed in complexity_handlers/satd.rs, but on a
    // route nothing dispatches to.)
    enforce_fail_on_violation(&filtered_result, config.fail_on_violation)
}

/// Refuse `--evolution` rather than answering it with a placeholder sentence.
///
/// `--evolution` is documented as "Track debt evolution over time (requires git
/// history)" and `--days` as the window. Nothing read git history: summary and
/// SARIF ignored the flag entirely (`analyze satd -p .` and `analyze satd -p .
/// --evolution --days 90` were byte-identical), JSON answered with
/// `"evolution": {"message": "Evolution tracking would show SATD trends over
/// time"}` and Markdown with a `## Evolution (Last N Days)` heading over that
/// same sentence — so `--days` moved a number in a heading and nothing else.
/// `DebtEvolution` exists as a type with no producer.
///
/// A flag that measures nothing must say so, not emit prose shaped like a
/// result. Same rule `pmat report --format html` follows (#672).
fn reject_unimplemented_evolution(evolution: bool) -> Result<()> {
    if evolution {
        anyhow::bail!(
            "--evolution is not implemented for `analyze satd`: no debt history is \
             computed, and --days selects nothing. Re-run without it."
        );
    }
    Ok(())
}

/// Turn retained violations into a non-zero exit when the caller asked for one.
///
/// The message deliberately avoids the word "violation": `bin/pmat.rs`
/// `categorize_error` maps any error text containing it to exit code 3, and the
/// documented behaviour of this flag — and the behaviour of the sibling
/// implementation it replaces — is exit 1.
fn enforce_fail_on_violation(result: &SatdAnalysisResult, fail_on_violation: bool) -> Result<()> {
    if fail_on_violation && !result.violations.is_empty() {
        anyhow::bail!(
            "SATD gate failed: {} self-admitted technical debt items found in {} files",
            result.violations.len(),
            result.total_files
        );
    }
    Ok(())
}

// Sprint 89 GREEN Phase: NEW EXTRACTED FUNCTIONS (A+ ≤10 complexity each)

/// Log filter information if filters are specified - EXTRACTED FUNCTION
/// Complexity: 5 (A+ standard)
fn log_filter_info(config: &SatdAnalysisConfig) {
    if !config.include.is_empty() || !config.exclude.is_empty() {
        crate::status_eprintln!("🔍 Applying file filters...");
        if !config.include.is_empty() {
            crate::status_eprintln!("  Include patterns: {:?}", config.include);
        }
        if !config.exclude.is_empty() {
            crate::status_eprintln!("  Exclude patterns: {:?}", config.exclude);
        }
    }
}

/// Execute SATD analysis using facade - EXTRACTED FUNCTION
/// Complexity: 6 (A+ standard)
async fn execute_satd_analysis(config: &SatdAnalysisConfig) -> Result<SatdAnalysisResult> {
    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = SatdFacade::new(registry);

    // Log extended mode if enabled
    if config.extended {
        crate::status_eprintln!(
            "📋 Extended mode: detecting euphemisms (placeholder, stub, for now...)"
        );
    }

    // Build analysis request
    let request = SatdAnalysisRequest {
        path: config.path.clone(),
        strict_mode: config.strict,
        include_tests: config.include_tests,
        extended: config.extended,
    };

    // Perform analysis
    facade.analyze_project(request).await
}

/// Apply file and severity filters to analysis results - EXTRACTED FUNCTION
/// Complexity: 8 (A+ standard)
fn apply_analysis_filters(
    mut result: SatdAnalysisResult,
    config: &SatdAnalysisConfig,
) -> Result<SatdAnalysisResult> {
    // Apply file filter to results if filters are specified
    if !config.include.is_empty() || !config.exclude.is_empty() {
        use crate::utils::file_filter::FileFilter;
        let filter = FileFilter::new(config.include.clone(), config.exclude.clone())?;

        if filter.has_filters() {
            result.violations.retain(|violation| {
                let path = std::path::Path::new(&violation.file_path);
                filter.should_include(path)
            });

            // Update total files count
            let unique_files: std::collections::HashSet<_> =
                result.violations.iter().map(|v| &v.file_path).collect();
            result.total_files = unique_files.len();
        }
    }

    // Apply severity and criticality filters
    let mut filtered = apply_filters(result, config.severity.clone(), config.critical_only);

    // Issue #676: `summary` and `total_files` were left at their PRE-filter
    // values while `violations`/`total_violations` were post-filter, so
    // `--severity high` printed {"total_files":1,"total_violations":0,
    //  "summary":"Found 7 SATD violations in 1 files","violations":[]} — one
    // document, two contradictory answers. Restate both from what survived.
    recompute_totals(&mut filtered);
    rank_violations(&mut filtered);
    Ok(filtered)
}

/// Put the violations in the order the report claims to print them in.
///
/// The summary heading says "Top Violations" and then took the first ten in
/// whatever order the facade's file walk produced, so the ten shown were not
/// the ten worst and were not stable between runs on the same tree. `--top-files`
/// is a limit on a *ranked* list; ranking has to exist before limiting means
/// anything. Severity descending, then file and line so ties are deterministic.
fn rank_violations(result: &mut SatdAnalysisResult) {
    use crate::services::facades::satd_facade::SatdSeverity as FacadeSeverity;
    fn rank(severity: &FacadeSeverity) -> u8 {
        match severity {
            FacadeSeverity::Critical => 0,
            FacadeSeverity::High => 1,
            FacadeSeverity::Medium => 2,
            FacadeSeverity::Low => 3,
        }
    }
    result.violations.sort_by(|a, b| {
        rank(&a.severity)
            .cmp(&rank(&b.severity))
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.line_number.cmp(&b.line_number))
    });
}

/// Restate `total_files` and `summary` from the violations actually retained.
fn recompute_totals(result: &mut SatdAnalysisResult) {
    let unique_files: std::collections::HashSet<_> =
        result.violations.iter().map(|v| &v.file_path).collect();
    result.total_files = unique_files.len();
    // The scope note survives the restatement. It describes what the WALK
    // declined to read, which no amount of post-filtering changes, and dropping
    // it here would put back the sentence #923 was about: a count with no
    // denominator, identical whether the tree was clean or barely read.
    let scope = result.census.note().map(|n| format!(" ({n})")).unwrap_or_default();
    result.summary = format!(
        "Found {} SATD violations in {} files{scope}",
        result.violations.len(),
        result.total_files
    );
}

/// Format and write SATD output - EXTRACTED FUNCTION
/// Complexity: 7 (A+ standard)
async fn write_satd_output(
    filtered_result: &SatdAnalysisResult,
    config: &SatdAnalysisConfig,
) -> Result<()> {
    // Format output. `config.top_files` used to stop at the struct field: it
    // was declared, parsed and stored, and no renderer ever read it, so
    // `--top-files 1` and `--top-files 50` printed the same ten rows.
    let content = format_output(
        filtered_result,
        config.format.clone(),
        config.metrics,
        config.top_files,
    );

    // Write to file or stdout
    if let Some(output_path) = &config.output {
        tokio::fs::write(output_path, &content).await?;
        crate::status_eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    // Print metrics if requested
    if config.metrics {
        print_metrics(filtered_result);
    }

    Ok(())
}

/// Apply severity and criticality filters
fn apply_filters(
    mut result: SatdAnalysisResult,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> SatdAnalysisResult {
    if let Some(min_severity) = severity {
        result.violations.retain(|v| match min_severity {
            SatdSeverity::Critical => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            ),
            SatdSeverity::High => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            ),
            SatdSeverity::Medium => !matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            ),
            SatdSeverity::Low => true,
        });
    }

    if critical_only {
        result.violations.retain(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            )
        });
    }

    result
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fail_on_violation_tests {
    use super::*;
    use crate::services::facades::satd_facade::{SatdSeverity as FacadeSeverity, SatdViolation};

    fn result_with(n: usize) -> SatdAnalysisResult {
        SatdAnalysisResult {
            census: Default::default(),
            total_files: usize::from(n > 0),
            violations: (0..n)
                .map(|i| SatdViolation {
                    file_path: "src/lib.rs".to_string(),
                    line_number: i + 1,
                    violation_type: "TODO".to_string(),
                    message: "TODO: finish".to_string(),
                    severity: FacadeSeverity::Medium,
                })
                .collect(),
            summary: format!("Found {n} SATD violations in 1 files"),
        }
    }

    #[test]
    fn violations_plus_flag_is_an_error() {
        let err = enforce_fail_on_violation(&result_with(3), true)
            .expect_err("--fail-on-violation with 3 items must not succeed");
        let message = err.to_string();
        assert!(message.contains('3'), "{message}");
        // Exit-code contract: `categorize_error` in bin/pmat.rs routes anything
        // mentioning "violation" to exit 3; this gate must exit 1.
        assert!(
            !message.to_lowercase().contains("violation"),
            "message would be categorised as exit 3: {message}"
        );
    }

    #[test]
    fn no_violations_or_no_flag_is_success() {
        assert!(enforce_fail_on_violation(&result_with(0), true).is_ok());
        assert!(enforce_fail_on_violation(&result_with(3), false).is_ok());
    }

    /// End-to-end: the routed handler never consulted `config.fail_on_violation`
    /// at all, so this returned `Ok` (exit 0) on a tree full of TODOs whether or
    /// not the flag was set.
    #[tokio::test]
    async fn handler_fails_when_fail_on_violation_is_set() {
        let project = tempfile::TempDir::new().unwrap();
        let out = tempfile::TempDir::new().unwrap();
        std::fs::write(
            project.path().join("lib.rs"),
            "// TODO: implement this\nfn f() {}\n// FIXME: broken\nfn g() {}\n",
        )
        .unwrap();

        let config = |fail_on_violation: bool| SatdAnalysisConfig {
            path: project.path().to_path_buf(),
            format: SatdOutputFormat::Summary,
            severity: None,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: Some(out.path().join("satd.txt")),
            top_files: 0,
            fail_on_violation,
            timeout: 60,
            include: vec![],
            exclude: vec![],
            extended: false,
        };

        assert!(
            handle_analyze_satd(config(false)).await.is_ok(),
            "without the flag the command reports and succeeds"
        );
        assert!(
            handle_analyze_satd(config(true)).await.is_err(),
            "--fail-on-violation must make a tree with TODO/FIXME exit non-zero"
        );
    }
}
