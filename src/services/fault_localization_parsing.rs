// LCOV coverage parsing and high-level fault localization integration.
// Contains: LcovParser impl (parse_file, parse, combine_coverage)
// and FaultLocalizer impl (run_localization, generate_report, format_terminal_report, enrich_with_tdg).

/// Which of the two LCOV input shapes a spectrum is.
///
/// `pmat localize --passed-coverage` takes a single path, and users feed it
/// either shape. The number in a `DA:` line means something different in each,
/// so the shape has to be decided before the number is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectrumShape {
    /// Per-test LCOV reports concatenated: a statement appears once per test
    /// that executed it, so records are the test count.
    PerTest,
    /// One aggregate record per file, as `cargo llvm-cov --lcov` emits: the DA
    /// hit count is the only per-statement signal in the file.
    Aggregate,
}

impl LcovParser {
    /// Parse LCOV format coverage file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<(StatementId, usize)>> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("Failed to read LCOV file: {}", e))?;
        Self::parse(&content)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Parse the input.
    pub fn parse(content: &str) -> Result<Vec<(StatementId, usize)>> {
        let mut results = Vec::new();
        let mut current_file: Option<PathBuf> = None;

        for line in content.lines() {
            let line = line.trim();

            if let Some(path) = line.strip_prefix("SF:") {
                current_file = Some(PathBuf::from(path));
            } else if let Some(da) = line.strip_prefix("DA:") {
                if let Some(ref file) = current_file {
                    let parts: Vec<&str> = da.split(',').collect();
                    if parts.len() >= 2 {
                        if let (Ok(line_num), Ok(count)) =
                            (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                        {
                            results.push((StatementId::new(file.clone(), line_num), count));
                        }
                    }
                }
            } else if line == "end_of_record" {
                current_file = None;
            }
        }

        Ok(results)
    }

    /// How an LCOV spectrum encodes "how many tests executed this statement".
    ///
    /// SBFL needs a TEST count per statement; LCOV records an EXECUTION count.
    /// The two coincide only in one of the two shapes users actually feed in,
    /// so the shape has to be decided before the counts mean anything.
    fn spectrum_shape(spectrum: &[(StatementId, usize)]) -> SpectrumShape {
        let mut seen: HashSet<&StatementId> = HashSet::new();
        for (stmt, _) in spectrum {
            if !seen.insert(stmt) {
                // The same statement appeared in two records, so the file is a
                // concatenation of per-test reports and the records themselves
                // are the test count.
                return SpectrumShape::PerTest;
            }
        }
        SpectrumShape::Aggregate
    }

    /// Tally one spectrum into "number of tests that executed this statement".
    ///
    /// `total_tests` (the `--passed-count` / `--failed-count` the user
    /// declared) is the ceiling: no statement can be executed by more tests
    /// than were run, and an aggregate hit count routinely exceeds it (a loop
    /// body hit 1000 times by 3 tests).
    fn tally(
        spectrum: &[(StatementId, usize)],
        total_tests: usize,
        shape: SpectrumShape,
    ) -> HashMap<StatementId, usize> {
        let mut tally: HashMap<StatementId, usize> = HashMap::new();
        for (stmt, count) in spectrum {
            if *count == 0 {
                continue;
            }
            let contribution = match shape {
                // One record per test: the record IS the evidence that one test
                // executed the statement, whatever the in-test loop count was.
                SpectrumShape::PerTest => 1,
                // One aggregate record per file: the hit count is the only
                // signal there is, so it stands in for the test count.
                SpectrumShape::Aggregate => *count,
            };
            *tally.entry(stmt.clone()).or_insert(0) += contribution;
        }
        if total_tests > 0 {
            for v in tally.values_mut() {
                *v = (*v).min(total_tests);
            }
        }
        tally
    }

    /// Combine coverage from multiple test runs (passed and failed).
    ///
    /// Accepts both LCOV shapes users produce:
    ///
    /// * **Aggregate** — one record per file, as `cargo llvm-cov --lcov` emits.
    ///   The DA hit count is used as the per-statement test count, capped at
    ///   `total_passed` / `total_failed`. This used to be read as a boolean:
    ///   every statement in a single-record file tallied to exactly 0 or 1, so
    ///   an entire spectrum collapsed to one score, every statement tied, and
    ///   the top-N was whatever order the `HashMap` happened to yield — six
    ///   identical invocations returned six different answers (#949).
    /// * **Per-test** — per-test reports concatenated, one record per test.
    ///   Each record counts as one test, so an in-test loop does not inflate
    ///   the spectrum.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn combine_coverage(
        passed_coverage: &[(StatementId, usize)],
        failed_coverage: &[(StatementId, usize)],
        total_passed: usize,
        total_failed: usize,
    ) -> Vec<StatementCoverage> {
        let passed = Self::tally(
            passed_coverage,
            total_passed,
            Self::spectrum_shape(passed_coverage),
        );
        let failed = Self::tally(
            failed_coverage,
            total_failed,
            Self::spectrum_shape(failed_coverage),
        );

        let mut ids: Vec<StatementId> = passed.keys().chain(failed.keys()).cloned().collect();
        ids.sort();
        ids.dedup();

        ids.into_iter()
            .map(|id| {
                let p = passed.get(&id).copied().unwrap_or(0);
                let f = failed.get(&id).copied().unwrap_or(0);
                StatementCoverage::new(id, p, f)
            })
            .collect()
    }
}

impl FaultLocalizer {
    /// Check if cargo-llvm-cov is available
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_coverage_tool_available() -> bool {
        std::process::Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run fault localization on coverage data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn run_localization(
        passed_coverage: &[(StatementId, usize)],
        failed_coverage: &[(StatementId, usize)],
        total_passed: usize,
        total_failed: usize,
        formula: SbflFormula,
        top_n: usize,
    ) -> FaultLocalizationResult {
        info!(
            "Running fault localization: {} passed, {} failed tests",
            total_passed, total_failed
        );

        // Combine coverage data. The declared test totals are the ceiling on
        // any statement's spectrum, so they have to reach the tally.
        let combined = LcovParser::combine_coverage(
            passed_coverage,
            failed_coverage,
            total_passed,
            total_failed,
        );

        // Run SBFL localization
        let localizer = SbflLocalizer::new()
            .with_formula(formula)
            .with_top_n(top_n)
            .with_explanations(true);

        localizer.localize(&combined, total_passed, total_failed)
    }

    /// Generate report in specified format
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_report(
        result: &FaultLocalizationResult,
        format: ReportFormat,
    ) -> Result<String> {
        match format {
            ReportFormat::Yaml => serde_yaml_ng::to_string(result)
                .map_err(|e| anyhow!("Failed to generate YAML: {}", e)),
            ReportFormat::Json => serde_json::to_string_pretty(result)
                .map_err(|e| anyhow!("Failed to generate JSON: {}", e)),
            ReportFormat::Terminal => Ok(Self::format_terminal_report(result)),
        }
    }

    /// Format report for terminal output
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_terminal_report(result: &FaultLocalizationResult) -> String {
        let mut output = String::new();

        output.push_str(
            "╔══════════════════════════════════════════════════════════════════════════════╗\n",
        );
        output.push_str(&format!(
            "║           FAULT LOCALIZATION REPORT - {}                              \n",
            result.formula_used
        ));
        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );
        output.push_str(&format!(
            "║ Tests: {} passed, {} failed                                                \n",
            result.total_passed_tests, result.total_failed_tests
        ));
        output.push_str(&format!(
            "║ Confidence: {:.2}                                                          \n",
            result.confidence
        ));
        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );
        output.push_str(
            "║  TOP SUSPICIOUS STATEMENTS                                                   ║\n",
        );
        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );

        for ranking in &result.rankings {
            let bar_len = (ranking.suspiciousness * 20.0).min(20.0) as usize;
            let progress_bar = format!("{}{}", "█".repeat(bar_len), "░".repeat(20 - bar_len));

            // Truncate file path for display
            let file_display = ranking.statement.file.display().to_string();
            let file_short = if file_display.len() > 30 {
                format!(
                    "...{}",
                    file_display
                        .get(file_display.len() - 27..)
                        .unwrap_or(&file_display)
                )
            } else {
                file_display
            };

            output.push_str(&format!(
                "║  #{:<2} {:30}:{:<5}  {} {:.2}  ║\n",
                ranking.rank,
                file_short,
                ranking.statement.line,
                progress_bar,
                ranking.suspiciousness
            ));
        }

        output.push_str(
            "╚══════════════════════════════════════════════════════════════════════════════╝\n",
        );

        // Add detailed explanations
        if !result.rankings.is_empty() {
            output.push_str("\n📋 Detailed Analysis:\n");
            for ranking in &result.rankings {
                output.push_str(&format!(
                    "\n  #{} {} (score: {:.3})\n",
                    ranking.rank, ranking.statement, ranking.suspiciousness
                ));
                output.push_str(&format!("     {}\n", ranking.explanation));
                output.push_str(&format!(
                    "     All scores: tarantula={:.3}, ochiai={:.3}, dstar2={:.3}, dstar3={:.3}\n",
                    ranking.scores.get("tarantula").unwrap_or(&0.0),
                    ranking.scores.get("ochiai").unwrap_or(&0.0),
                    ranking.scores.get("dstar2").unwrap_or(&0.0),
                    ranking.scores.get("dstar3").unwrap_or(&0.0),
                ));
            }
        }

        output
    }

    /// Enrich fault localization results with TDG scores
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn enrich_with_tdg(
        result: &mut FaultLocalizationResult,
        tdg_scores: &HashMap<String, f32>,
    ) {
        for ranking in &mut result.rankings {
            let file_path = ranking.statement.file.to_string_lossy().to_string();
            if let Some(&tdg) = tdg_scores.get(&file_path) {
                ranking.scores.insert("tdg".to_string(), tdg);
            }
        }
    }
}
