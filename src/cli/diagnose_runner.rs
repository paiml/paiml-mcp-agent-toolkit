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

    fn compute_summary(&self, features: &BTreeMap<String, FeatureResult>) -> DiagnosticSummary {
        debug_assert!(true, "contract: compute_summary");
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
            success_rate: if total > 0 {
                (passed as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    fn extract_error_context(
        &self,
        features: &BTreeMap<String, FeatureResult>,
    ) -> Option<CompactErrorContext> {
        debug_assert!(true, "contract: extract_error_context");
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
        debug_assert!(!error.is_empty(), "error must not be empty");
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
        debug_assert!(true, "contract: generate_fixes");
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
    debug_assert!(true, "contract: print_pretty_report");
    use crate::cli::colors as c;
    println!("{}", c::header("PMAT Self-Diagnostic Report"));
    println!(
        "  {}: {}    {}: {}ms",
        c::label("Version"),
        c::number(&report.version),
        c::label("Duration"),
        c::number(&report.duration_ms.to_string()),
    );
    println!();

    for (feature, result) in &report.features {
        let line = match result.status {
            FeatureStatus::Ok => c::pass(&format!(
                "{} {}({}μs){}",
                c::path(feature),
                c::DIM,
                result.duration_us,
                c::RESET
            )),
            FeatureStatus::Degraded(_) => c::warn(&format!(
                "{} {}({}μs){}",
                c::path(feature),
                c::DIM,
                result.duration_us,
                c::RESET
            )),
            FeatureStatus::Failed => c::fail(&format!(
                "{} {}({}μs){}",
                c::path(feature),
                c::DIM,
                result.duration_us,
                c::RESET
            )),
            FeatureStatus::Skipped(_) => c::skip(&format!(
                "{} {}({}μs){}",
                feature, c::DIM, result.duration_us, c::RESET
            )),
        };
        println!("{line}");

        if let Some(error) = &result.error {
            println!("  {}└─ {error}{}", c::RED, c::RESET);
        }
    }

    println!();
    println!("{}", c::subheader("Summary:"));
    println!("  {}: {}", c::label("Total"), c::number(&report.summary.total.to_string()));
    println!(
        "  {}: {}{}{}",
        c::label("Passed"),
        c::GREEN,
        report.summary.passed,
        c::RESET
    );
    println!(
        "  {}: {}{}{}",
        c::label("Failed"),
        if report.summary.failed > 0 { c::RED } else { c::GREEN },
        report.summary.failed,
        c::RESET
    );
    println!(
        "  {}: {}",
        c::label("Success Rate"),
        c::pct(report.summary.success_rate, 100.0, 80.0)
    );

    if let Some(ctx) = &report.error_context {
        println!();
        println!("{}", c::subheader("Suggested Fixes:"));
        for fix in &ctx.suggested_fixes {
            println!(
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
}
