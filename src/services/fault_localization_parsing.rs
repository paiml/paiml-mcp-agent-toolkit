// LCOV coverage parsing and high-level fault localization integration.
// Contains: LcovParser impl (parse_file, parse, combine_coverage)
// and FaultLocalizer impl (run_localization, generate_report, format_terminal_report, enrich_with_tdg).

impl LcovParser {
    /// Parse LCOV format coverage file
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<(StatementId, usize)>> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("Failed to read LCOV file: {}", e))?;
        Self::parse(&content)
    }

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

    /// Combine coverage from multiple test runs (passed and failed)
    pub fn combine_coverage(
        passed_coverage: &[(StatementId, usize)],
        failed_coverage: &[(StatementId, usize)],
    ) -> Vec<StatementCoverage> {
        let mut coverage_map: HashMap<StatementId, (usize, usize)> = HashMap::new();

        // Count passed test coverage
        for (stmt, count) in passed_coverage {
            if *count > 0 {
                coverage_map.entry(stmt.clone()).or_insert((0, 0)).0 += 1;
            }
        }

        // Count failed test coverage
        for (stmt, count) in failed_coverage {
            if *count > 0 {
                coverage_map.entry(stmt.clone()).or_insert((0, 0)).1 += 1;
            }
        }

        coverage_map
            .into_iter()
            .map(|(id, (passed, failed))| StatementCoverage::new(id, passed, failed))
            .collect()
    }
}

impl FaultLocalizer {
    /// Check if cargo-llvm-cov is available
    pub fn is_coverage_tool_available() -> bool {
        std::process::Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run fault localization on coverage data
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

        // Combine coverage data
        let combined = LcovParser::combine_coverage(passed_coverage, failed_coverage);

        // Run SBFL localization
        let localizer = SbflLocalizer::new()
            .with_formula(formula)
            .with_top_n(top_n)
            .with_explanations(true);

        localizer.localize(&combined, total_passed, total_failed)
    }

    /// Generate report in specified format
    pub fn generate_report(
        result: &FaultLocalizationResult,
        format: ReportFormat,
    ) -> Result<String> {
        match format {
            ReportFormat::Yaml => {
                serde_yaml_ng::to_string(result).map_err(|e| anyhow!("Failed to generate YAML: {}", e))
            }
            ReportFormat::Json => serde_json::to_string_pretty(result)
                .map_err(|e| anyhow!("Failed to generate JSON: {}", e)),
            ReportFormat::Terminal => Ok(Self::format_terminal_report(result)),
        }
    }

    /// Format report for terminal output
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
    #[allow(dead_code)]
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
