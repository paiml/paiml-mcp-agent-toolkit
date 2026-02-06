#![cfg_attr(coverage_nightly, coverage(off))]
// Tarantula-style Spectrum-Based Fault Localization (SBFL)
// Issue #103: Fault localization integration
// Toyota Way: Start with simplest formula, evolve based on evidence
// Phase 1: Classic Tarantula + Ochiai + DStar formulas
// Muda: Avoid waste by using lightweight SBFL before expensive MBFL
// Muri: Prevent overburden by presenting only top-N suspicious statements

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Represents a code location for fault localization
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatementId {
    pub file: PathBuf,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl StatementId {
    pub fn new(file: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

impl std::fmt::Display for StatementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Coverage information for a single statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementCoverage {
    pub id: StatementId,
    pub executed_by_passed: usize,
    pub executed_by_failed: usize,
}

impl StatementCoverage {
    pub fn new(id: StatementId, passed: usize, failed: usize) -> Self {
        Self {
            id,
            executed_by_passed: passed,
            executed_by_failed: failed,
        }
    }
}

/// Available fault localization formulas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SbflFormula {
    /// Original Tarantula formula (Jones & Harrold, 2005)
    #[default]
    Tarantula,
    /// Ochiai formula - often outperforms Tarantula (Abreu et al., 2009)
    Ochiai,
    /// DStar with configurable exponent (Wong et al., 2014)
    DStar { exponent: u32 },
}

impl std::fmt::Display for SbflFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SbflFormula::Tarantula => write!(f, "Tarantula"),
            SbflFormula::Ochiai => write!(f, "Ochiai"),
            SbflFormula::DStar { exponent } => write!(f, "DStar{}", exponent),
        }
    }
}

impl std::str::FromStr for SbflFormula {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tarantula" => Ok(SbflFormula::Tarantula),
            "ochiai" => Ok(SbflFormula::Ochiai),
            "dstar2" => Ok(SbflFormula::DStar { exponent: 2 }),
            "dstar3" => Ok(SbflFormula::DStar { exponent: 3 }),
            other => Err(anyhow!("Unknown SBFL formula: {}", other)),
        }
    }
}

/// Individual suspiciousness ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousnessRanking {
    pub rank: usize,
    pub statement: StatementId,
    pub suspiciousness: f32,
    pub scores: HashMap<String, f32>,
    pub explanation: String,
    pub failed_coverage: usize,
    pub passed_coverage: usize,
}

/// Result of fault localization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultLocalizationResult {
    pub rankings: Vec<SuspiciousnessRanking>,
    pub formula_used: SbflFormula,
    pub confidence: f32,
    pub total_passed_tests: usize,
    pub total_failed_tests: usize,
}

/// Classic Tarantula suspiciousness formula
///
/// Formula: (failed/totalFailed) / ((passed/totalPassed) + (failed/totalFailed))
///
/// Reference: Jones, J.A., Harrold, M.J. (2005). ASE '05
pub fn tarantula(failed: usize, passed: usize, total_failed: usize, total_passed: usize) -> f32 {
    let failed_ratio = if total_failed > 0 {
        failed as f32 / total_failed as f32
    } else {
        0.0
    };

    let passed_ratio = if total_passed > 0 {
        passed as f32 / total_passed as f32
    } else {
        0.0
    };

    let denominator = passed_ratio + failed_ratio;
    if denominator == 0.0 {
        0.0
    } else {
        failed_ratio / denominator
    }
}

/// Ochiai suspiciousness formula (from molecular biology)
///
/// Formula: failed / sqrt(totalFailed * (failed + passed))
///
/// Reference: Abreu et al. (2009). JSS 82(11)
pub fn ochiai(failed: usize, passed: usize, total_failed: usize) -> f32 {
    let denominator = ((total_failed * (failed + passed)) as f32).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        failed as f32 / denominator
    }
}

/// DStar suspiciousness formula with configurable exponent
///
/// Formula: failed^* / (passed + (totalFailed - failed))
///
/// Reference: Wong et al. (2014). IEEE TSE 40(1)
pub fn dstar(failed: usize, passed: usize, total_failed: usize, star: u32) -> f32 {
    let numerator = (failed as f32).powi(star as i32);
    let not_failed = total_failed.saturating_sub(failed);
    let denominator = passed as f32 + not_failed as f32;

    if denominator == 0.0 {
        if numerator > 0.0 {
            f32::MAX // Avoid infinity, use max finite value
        } else {
            0.0
        }
    } else {
        numerator / denominator
    }
}

/// Spectrum-Based Fault Localizer
///
/// Implements the core SBFL algorithms following Toyota Way principles:
/// - Start simple (Tarantula baseline)
/// - Measure and evolve (compare formulas)
/// - Eliminate waste (skip expensive analysis when simple works)
pub struct SbflLocalizer {
    formula: SbflFormula,
    top_n: usize,
    include_explanations: bool,
    min_confidence_threshold: f32,
}

impl Default for SbflLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SbflLocalizer {
    pub fn new() -> Self {
        Self {
            formula: SbflFormula::Tarantula,
            top_n: 10,
            include_explanations: true,
            min_confidence_threshold: 0.0,
        }
    }

    pub fn with_formula(mut self, formula: SbflFormula) -> Self {
        self.formula = formula;
        self
    }

    pub fn with_top_n(mut self, n: usize) -> Self {
        self.top_n = n;
        self
    }

    pub fn with_explanations(mut self, include: bool) -> Self {
        self.include_explanations = include;
        self
    }

    #[allow(dead_code)]
    pub fn with_min_confidence(mut self, threshold: f32) -> Self {
        self.min_confidence_threshold = threshold;
        self
    }

    /// Localize faults using the configured SBFL formula
    pub fn localize(
        &self,
        coverage: &[StatementCoverage],
        total_passed: usize,
        total_failed: usize,
    ) -> FaultLocalizationResult {
        info!(
            "Running {:?} fault localization on {} statements",
            self.formula,
            coverage.len()
        );

        // Calculate suspiciousness for each statement
        let mut scored: Vec<(StatementId, f32, usize, usize)> = coverage
            .iter()
            .map(|cov| {
                let score = self.calculate_score(
                    cov.executed_by_failed,
                    cov.executed_by_passed,
                    total_failed,
                    total_passed,
                );
                (
                    cov.id.clone(),
                    score,
                    cov.executed_by_failed,
                    cov.executed_by_passed,
                )
            })
            .collect();

        // Sort by suspiciousness (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        let rankings: Vec<SuspiciousnessRanking> = scored
            .into_iter()
            .take(self.top_n)
            .enumerate()
            .filter(|(_, (_, score, _, _))| *score >= self.min_confidence_threshold)
            .map(|(rank, (stmt, score, failed, passed))| {
                let explanation = if self.include_explanations {
                    self.generate_explanation(failed, passed, total_failed, total_passed, score)
                } else {
                    String::new()
                };

                // Calculate all formula scores for comparison
                let mut scores = HashMap::new();
                scores.insert(
                    "tarantula".to_string(),
                    tarantula(failed, passed, total_failed, total_passed),
                );
                scores.insert("ochiai".to_string(), ochiai(failed, passed, total_failed));
                scores.insert("dstar2".to_string(), dstar(failed, passed, total_failed, 2));
                scores.insert("dstar3".to_string(), dstar(failed, passed, total_failed, 3));

                SuspiciousnessRanking {
                    rank: rank + 1,
                    statement: stmt,
                    suspiciousness: score,
                    scores,
                    explanation,
                    failed_coverage: failed,
                    passed_coverage: passed,
                }
            })
            .collect();

        // Calculate confidence based on test coverage density
        let confidence = self.calculate_confidence(coverage.len(), total_passed, total_failed);

        debug!(
            "Localized {} suspicious statements with confidence {}",
            rankings.len(),
            confidence
        );

        FaultLocalizationResult {
            rankings,
            formula_used: self.formula,
            confidence,
            total_passed_tests: total_passed,
            total_failed_tests: total_failed,
        }
    }

    fn calculate_score(
        &self,
        failed: usize,
        passed: usize,
        total_failed: usize,
        total_passed: usize,
    ) -> f32 {
        match self.formula {
            SbflFormula::Tarantula => tarantula(failed, passed, total_failed, total_passed),
            SbflFormula::Ochiai => ochiai(failed, passed, total_failed),
            SbflFormula::DStar { exponent } => dstar(failed, passed, total_failed, exponent),
        }
    }

    fn generate_explanation(
        &self,
        failed: usize,
        passed: usize,
        total_failed: usize,
        total_passed: usize,
        score: f32,
    ) -> String {
        let failed_pct = if total_failed > 0 {
            (failed as f32 / total_failed as f32 * 100.0) as u32
        } else {
            0
        };

        let passed_pct = if total_passed > 0 {
            (passed as f32 / total_passed as f32 * 100.0) as u32
        } else {
            0
        };

        format!(
            "Executed by {}% of failing tests ({}/{}) and {}% of passing tests ({}/{}). \
             Suspiciousness score: {:.3}",
            failed_pct, failed, total_failed, passed_pct, passed, total_passed, score
        )
    }

    fn calculate_confidence(
        &self,
        statement_count: usize,
        total_passed: usize,
        total_failed: usize,
    ) -> f32 {
        let total_tests = total_passed + total_failed;
        if total_tests == 0 || total_failed == 0 {
            return 0.0;
        }

        // Factor 1: Log scale for failing test count (diminishing returns)
        let fail_factor = (total_failed as f32).ln().min(3.0) / 3.0;

        // Factor 2: Failing ratio (sweet spot around 5-20%)
        let fail_ratio = total_failed as f32 / total_tests as f32;
        let ratio_factor = if fail_ratio < 0.01 {
            fail_ratio * 10.0 // Very few failures = low confidence
        } else if fail_ratio > 0.5 {
            1.0 - (fail_ratio - 0.5) // Too many failures = less localizing
        } else {
            1.0
        };

        // Factor 3: Statement coverage (more covered = more context)
        let coverage_factor = (statement_count as f32).ln().min(7.0) / 7.0;

        (fail_factor * ratio_factor * coverage_factor).min(1.0)
    }
}

/// LCOV coverage data parser for cargo-llvm-cov integration
#[derive(Debug, Default)]
pub struct LcovParser;

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

/// Report output format for fault localization results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    #[default]
    Terminal,
    Json,
    Yaml,
}

impl std::str::FromStr for ReportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "terminal" | "text" => Ok(ReportFormat::Terminal),
            "json" => Ok(ReportFormat::Json),
            "yaml" => Ok(ReportFormat::Yaml),
            other => Err(anyhow!("Unknown format: {}", other)),
        }
    }
}

/// Tarantula integration wrapper
///
/// Provides high-level interface for fault localization that integrates
/// with cargo-llvm-cov for coverage and pmat for TDG enrichment.
pub struct FaultLocalizer;

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
                serde_yaml::to_string(result).map_err(|e| anyhow!("Failed to generate YAML: {}", e))
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
                format!("...{}", &file_display[file_display.len() - 27..])
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarantula_perfect_fault() {
        // Statement executed by all failing tests, no passing tests
        let score = tarantula(10, 0, 10, 100);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tarantula_perfect_clean() {
        // Statement executed by all passing tests, no failing tests
        let score = tarantula(0, 100, 10, 100);
        assert!(score.abs() < 0.001);
    }

    #[test]
    fn test_tarantula_mixed() {
        let score = tarantula(5, 50, 10, 100);
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_ochiai_perfect_fault() {
        let score = ochiai(10, 0, 10);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dstar_mixed() {
        let score = dstar(5, 50, 10, 2);
        // 25 / (50 + 5) = 0.4545...
        assert!((score - 0.4545).abs() < 0.01);
    }

    #[test]
    fn test_localizer_basic() {
        let localizer = SbflLocalizer::new();

        let coverage = vec![
            StatementCoverage::new(StatementId::new("file.rs", 10), 0, 10), // All failing
            StatementCoverage::new(StatementId::new("file.rs", 20), 100, 0), // All passing
            StatementCoverage::new(StatementId::new("file.rs", 30), 50, 5), // Mixed
        ];

        let result = localizer.localize(&coverage, 100, 10);

        assert_eq!(result.rankings.len(), 3);
        assert_eq!(result.rankings[0].statement.line, 10); // Most suspicious first
    }

    #[test]
    fn test_lcov_parser() {
        let lcov = r#"
SF:src/main.rs
DA:1,10
DA:2,5
DA:3,0
end_of_record
SF:src/lib.rs
DA:10,1
end_of_record
"#;

        let result = LcovParser::parse(lcov).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0.file.to_str().unwrap(), "src/main.rs");
        assert_eq!(result[0].0.line, 1);
        assert_eq!(result[0].1, 10);
    }

    #[test]
    fn test_formula_from_str() {
        assert!(matches!(
            "tarantula".parse::<SbflFormula>().unwrap(),
            SbflFormula::Tarantula
        ));
        assert!(matches!(
            "ochiai".parse::<SbflFormula>().unwrap(),
            SbflFormula::Ochiai
        ));
        assert!(matches!(
            "dstar2".parse::<SbflFormula>().unwrap(),
            SbflFormula::DStar { exponent: 2 }
        ));
    }
}
