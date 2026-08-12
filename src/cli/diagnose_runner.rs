// SelfDiagnostic runner, error context extraction, and output formatting
// Included from diagnose.rs - do NOT add `use` imports or `#!` attributes here

impl Default for SelfDiagnostic {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfDiagnostic {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            tests: vec![
                // Core parsing
                Box::new(RustAstTest),
                Box::new(TypeScriptAstTest),
                Box::new(PythonAstTest),
                // Analysis engines
                Box::new(ComplexityAnalysisTest),
                Box::new(DeepContextTest),
                // Infrastructure
                Box::new(CacheSubsystemTest),
                Box::new(GitIntegrationTest),
                // Output formats
                Box::new(MermaidGeneratorTest),
            ],
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run_diagnostic(&self, args: &DiagnoseArgs) -> DiagnosticReport {
        let start = Instant::now();
        let mut features = BTreeMap::new();

        for test in &self.tests {
            let test_name = test.name();

            // Check if should skip
            if !args.only.is_empty() && !args.only.contains(&test_name.to_string()) {
                continue;
            }
            if args.skip.contains(&test_name.to_string()) {
                features.insert(
                    test_name.to_string(),
                    FeatureResult {
                        status: FeatureStatus::Skipped("User requested skip".to_string()),
                        duration_us: 0,
                        error: None,
                        metrics: None,
                    },
                );
                continue;
            }

            let test_start = Instant::now();
            let result =
                match timeout(Duration::from_secs(args.timeout.min(10)), test.execute()).await {
                    Ok(Ok(metrics)) => FeatureResult {
                        status: FeatureStatus::Ok,
                        duration_us: test_start.elapsed().as_micros() as u64,
                        error: None,
                        metrics: Some(metrics),
                    },
                    Ok(Err(e)) => FeatureResult {
                        status: FeatureStatus::Failed,
                        duration_us: test_start.elapsed().as_micros() as u64,
                        error: Some(format!("{e:?}")),
                        metrics: None,
                    },
                    Err(_) => FeatureResult {
                        status: FeatureStatus::Failed,
                        duration_us: 10_000_000, // timeout
                        error: Some("Test timeout after 10s".into()),
                        metrics: None,
                    },
                };

            features.insert(test_name.to_string(), result);
        }

        let summary = self.compute_summary(&features);
        let error_context = self.extract_error_context(&features);

        DiagnosticReport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_info: BuildInfo::current(),
            timestamp: Utc::now(),
            duration_ms: start.elapsed().as_millis() as u64,
            features,
            summary,
            error_context,
        }
    }

    /// Reject `--only`/`--skip` values that name no check.
    ///
    /// A misspelled `--only` matched nothing, so the run silently executed ZERO
    /// checks and still exited 0, reporting "Total: 0 / Success Rate: 0.0%" —
    /// a diagnostic that verified nothing while looking like it had run. A
    /// filter that selects nothing is a typo, not a result.
    fn validate_filters(&self, args: &DiagnoseArgs) -> Result<()> {
        let known: Vec<&str> = self.tests.iter().map(|t| t.name()).collect();
        for (flag, requested) in [("--only", &args.only), ("--skip", &args.skip)] {
            for name in requested {
                if !known.contains(&name.as_str()) {
                    anyhow::bail!(
                        "unknown {flag} feature test '{name}'. Available: {}",
                        known.join(", ")
                    );
                }
            }
        }
        Ok(())
    }

    fn compute_summary(&self, features: &BTreeMap<String, FeatureResult>) -> DiagnosticSummary {
        let total = features.len();
        let mut passed = 0;
        let mut failed = 0;
        let mut degraded = 0;
        let mut skipped = 0;

        for result in features.values() {
            match &result.status {
                FeatureStatus::Ok => passed += 1,
                FeatureStatus::Failed => failed += 1,
                FeatureStatus::Degraded(_) => degraded += 1,
                FeatureStatus::Skipped(_) => skipped += 1,
            }
        }

        DiagnosticSummary {
            total,
            passed,
            failed,
            degraded,
            skipped,
            all_passed: failed == 0 && degraded == 0,
            // Rate over the checks that RAN. Dividing by `total` counted the
            // skipped ones as failures, so `--skip ast.rust` printed
            // "Success Rate: 87.5%" beside "Failed: 0" / all_passed: true.
            success_rate: {
                let executed = total - skipped;
                if executed > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    Some((passed as f64 / executed as f64) * 100.0)
                } else {
                    None
                }
            },
        }
    }

    fn extract_error_context(
        &self,
        features: &BTreeMap<String, FeatureResult>,
    ) -> Option<CompactErrorContext> {
        let failed: Vec<_> = features
            .iter()
            .filter(|(_, r)| matches!(r.status, FeatureStatus::Failed))
            .map(|(name, _)| name.clone())
            .collect();

        if failed.is_empty() {
            return None;
        }

        let mut error_patterns = BTreeMap::new();
        for (feature, result) in features {
            if let Some(error) = &result.error {
                let pattern = self.classify_error(error);
                error_patterns
                    .entry(pattern)
                    .or_insert_with(Vec::new)
                    .push(feature.clone());
            }
        }

        Some(CompactErrorContext {
            failed_features: failed,
            error_patterns: error_patterns.clone(),
            suggested_fixes: self.generate_fixes(&error_patterns),
            environment: EnvironmentSnapshot::capture(),
        })
    }

    fn classify_error(&self, error: &str) -> String {
        if error.contains("Permission denied") {
            "permission_denied".into()
        } else if error.contains("not found") {
            "file_not_found".into()
        } else if error.contains("timeout") {
            "timeout".into()
        } else if error.contains("git") {
            "git_error".into()
        } else {
            "unknown".into()
        }
    }

    fn generate_fixes(&self, error_patterns: &BTreeMap<String, Vec<String>>) -> Vec<SuggestedFix> {
        let mut fixes = Vec::new();

        for (pattern, features) in error_patterns {
            let fix = match pattern.as_str() {
                "permission_denied" => SuggestedFix {
                    feature: features.join(", "),
                    error_pattern: pattern.clone(),
                    fix_command: Some("chmod +r <file>".into()),
                    documentation_link: None,
                },
                "git_error" => SuggestedFix {
                    feature: features.join(", "),
                    error_pattern: pattern.clone(),
                    fix_command: Some("git init".into()),
                    documentation_link: Some(
                        "https://github.com/paiml/paiml-mcp-agent-toolkit#git-integration".into(),
                    ),
                },
                _ => SuggestedFix {
                    feature: features.join(", "),
                    error_pattern: pattern.clone(),
                    fix_command: None,
                    documentation_link: None,
                },
            };
            fixes.push(fix);
        }

        fixes
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_diagnose(args: DiagnoseArgs) -> Result<()> {
    let diagnostic = SelfDiagnostic::new();
    diagnostic.validate_filters(&args)?;
    let report = diagnostic.run_diagnostic(&args).await;

    match args.format {
        DiagnosticFormat::Pretty => print_pretty_report(&report),
        DiagnosticFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        DiagnosticFormat::Compact => {
            // Ultra-compact for Claude Code consumption
            let compact = json!({
                "v": report.version,
                "ok": report.summary.all_passed,
                "failed": report.error_context.as_ref().map(|c| &c.failed_features),
                "fixes": report.error_context.as_ref().map(|c| &c.suggested_fixes),
            });
            println!("{}", serde_json::to_string(&compact)?);
        }
    }

    Ok(())
}

fn print_pretty_report(report: &DiagnosticReport) {
    print!("{}", format_pretty_report(report));
}

/// Render the pretty report to a string.
///
/// Split out from the printer so the colour contract is assertable: with colour
/// off this must contain no ESC byte at all. It used to be one long chain of
/// `println!`s interpolating the raw `c::DIM`/`c::RESET`/`c::GREEN` consts,
/// which no test could reach and which `--color never` could not switch off.
pub(crate) fn format_pretty_report(report: &DiagnosticReport) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "{}", c::header("PMAT Self-Diagnostic Report"));
    let _ = writeln!(
        out,
        "  {}: {}    {}: {}ms",
        c::label("Version"),
        c::number(&report.version),
        c::label("Duration"),
        c::number(&report.duration_ms.to_string()),
    );
    let _ = writeln!(out);

    for (feature, result) in &report.features {
        // GH #684: the timing suffix used to interpolate the raw `c::DIM` /
        // `c::RESET` consts, which are `const` and so cannot consult
        // `colors_enabled`. `pmat diagnose --color never > out.txt` wrote ten
        // escape-bearing lines, byte-identical to `--color auto`, and NO_COLOR=1
        // was ignored for the same reason. `c::dim` is the helper that honours
        // the rule, and the timing is now built once instead of four times.
        let timing = c::dim(&format!("({}\u{3bc}s)", result.duration_us));
        let line = match result.status {
            FeatureStatus::Ok => c::pass(&format!("{} {timing}", c::path(feature))),
            FeatureStatus::Degraded(_) => c::warn(&format!("{} {timing}", c::path(feature))),
            FeatureStatus::Failed => c::fail(&format!("{} {timing}", c::path(feature))),
            FeatureStatus::Skipped(_) => c::skip(&format!("{feature} {timing}")),
        };
        let _ = writeln!(out, "{line}");

        if let Some(error) = &result.error {
            let _ = writeln!(
                out,
                "  {}",
                c::colored(c::RED, &format!("\u{2514}\u{2500} {error}"))
            );
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c::subheader("Summary:"));
    let _ = writeln!(
        out,
        "  {}: {}",
        c::label("Total"),
        c::number(&report.summary.total.to_string())
    );
    let _ = writeln!(
        out,
        "  {}: {}",
        c::label("Passed"),
        c::colored(c::GREEN, &report.summary.passed.to_string())
    );
    let _ = writeln!(
        out,
        "  {}: {}",
        c::label("Failed"),
        c::colored(
            if report.summary.failed > 0 {
                c::RED
            } else {
                c::GREEN
            },
            &report.summary.failed.to_string()
        )
    );
    let _ = writeln!(
        out,
        "  {}: {}",
        c::label("Success Rate"),
        match report.summary.success_rate {
            Some(rate) => c::pct(rate, 100.0, 80.0),
            // No check executed, so there is no rate; "0.0%" read as a total
            // failure of checks that were never run.
            None => c::dim("not measured (no check executed)"),
        }
    );

    if let Some(ctx) = &report.error_context {
        let _ = writeln!(out);
        let _ = writeln!(out, "{}", c::subheader("Suggested Fixes:"));
        for fix in &ctx.suggested_fixes {
            let _ = writeln!(
                out,
                "  {} {}: {}",
                c::warn(""),
                c::label(&fix.feature),
                c::dim(
                    fix.fix_command
                        .as_ref()
                        .unwrap_or(&"See documentation".into())
                )
            );
        }
    }
    out
}
