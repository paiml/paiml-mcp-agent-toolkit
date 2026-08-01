#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Actionable Entropy Analysis Module
//!
//! AST-based pattern entropy detection for identifying real code quality issues.
//! Focuses on actionable violations with clear fixes and LOC reduction estimates.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod entropy_calculator;
pub mod pattern_extractor;
pub mod violation_detector;

pub use entropy_calculator::{EntropyCalculator, EntropyMetrics, EntropyReport};
pub use pattern_extractor::{
    AstPattern, Location, PatternCollection, PatternExtractor, PatternType,
};
pub use violation_detector::{ActionableViolation, PatternSummary, Severity, ViolationDetector};

/// Configuration for entropy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Maximum allowed pattern repetitions before violation
    pub max_pattern_repetition: usize,
    /// Minimum required pattern diversity (0.0-1.0)
    pub min_pattern_diversity: f64,
    /// Maximum allowed cross-file similarity (0.0-1.0)
    pub max_cross_file_similarity: f64,
    /// Maximum allowed pattern inconsistency score (0.0-1.0)
    pub max_inconsistency_score: f64,
    /// Minimum severity level to report
    pub min_severity: Severity,
    /// Pattern types to analyze
    pub pattern_types: Vec<PatternType>,
    /// Paths to exclude from analysis
    pub exclude_paths: Vec<String>,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            max_pattern_repetition: 5,
            min_pattern_diversity: 0.3,
            max_cross_file_similarity: 0.7,
            max_inconsistency_score: 0.8,
            min_severity: Severity::Medium,
            pattern_types: vec![
                PatternType::ErrorHandling,
                PatternType::DataValidation,
                PatternType::ResourceManagement,
                PatternType::ControlFlow,
                PatternType::DataTransformation,
                PatternType::ApiCall,
            ],
            // Shared with every entry point (see `analysis_excludes`) so that two
            // commands asked about the same tree cannot disagree about which
            // files they looked at.
            exclude_paths: Self::analysis_excludes(false),
        }
    }
}

/// Paths no entropy analysis ever reads: build output and vendored code.
const NEVER_ANALYZED: [&str; 3] = ["**/target/**", "**/node_modules/**", "**/.git/**"];

/// Paths excluded unless the caller asked for tests to be included.
const TEST_AND_EXAMPLE_PATHS: [&str; 8] = [
    "tests/**",
    "**/tests/**",
    "examples/**",
    "**/examples/**",
    "benches/**",
    "**/benches/**",
    "**/*test*.rs",
    "**/*.test.rs",
];

impl EntropyConfig {
    /// The one exclusion list every entropy entry point applies.
    ///
    /// `analyze entropy` and `quality-gate --checks entropy` used to build their
    /// own lists: analyze excluded `**/*test*.rs`, the gate only the narrower
    /// `**/*_tests.rs` family. On the same `-p` that was 939 files versus 1328,
    /// so the two commands reported different values for the same metric —
    /// diversity 62.6% / 14 violations / "DataTransformation pattern repeated 17
    /// times" against 63.9% / 16 / "repeated 25 times". Same input, same metric,
    /// two answers.
    #[must_use]
    pub fn analysis_excludes(include_tests: bool) -> Vec<String> {
        let mut excludes: Vec<String> = NEVER_ANALYZED.iter().map(|s| (*s).to_string()).collect();
        if !include_tests {
            excludes.extend(TEST_AND_EXAMPLE_PATHS.iter().map(|s| (*s).to_string()));
        }
        excludes
    }

    /// Load additional exclude patterns from `.pmatignore` and `.gitignore`.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_project_ignores(mut self, project_path: &std::path::Path) -> Self {
        for ignore_file in &[".pmatignore", ".paimlignore"] {
            let path = project_path.join(ignore_file);
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        self.exclude_paths.push(trimmed.to_string());
                    }
                }
            }
        }
        self
    }
}

/// Main entropy analyzer
pub struct EntropyAnalyzer {
    config: EntropyConfig,
    pattern_extractor: PatternExtractor,
    violation_detector: ViolationDetector,
    entropy_calculator: EntropyCalculator,
}

impl Default for EntropyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl EntropyAnalyzer {
    /// Create new analyzer with default config
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(EntropyConfig::default())
    }

    /// Create analyzer with custom config
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_config(config: EntropyConfig) -> Self {
        Self {
            config: config.clone(),
            pattern_extractor: PatternExtractor::new(config.clone()),
            violation_detector: ViolationDetector::new(config.clone()),
            entropy_calculator: EntropyCalculator::new(config),
        }
    }

    /// Analyze entropy for a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self, project_path: &Path) -> Result<EntropyReport> {
        // Step 1: Extract AST patterns from project context
        let patterns = self
            .pattern_extractor
            .extract_patterns(project_path)
            .await?;

        // Step 2: Calculate entropy metrics
        let entropy_metrics = self.entropy_calculator.calculate(&patterns)?;

        // Step 3: Detect actionable violations
        let violations = self
            .violation_detector
            .detect_violations(&patterns, &entropy_metrics)?;

        // Step 4: Generate report
        let measurement_note = Self::measurement_note(&entropy_metrics, patterns.file_count());

        Ok(EntropyReport {
            total_files_analyzed: patterns.file_count(),
            actionable_violations: violations,
            pattern_summary: patterns.summary(),
            entropy_metrics,
            measurement_note,
        })
    }

    /// Explain an absent entropy measurement instead of leaving the reader with
    /// a block of zeros (#650): on a small crate no pattern repeats often enough
    /// to form a distribution, and "0.0 diversity" reads as the worst possible
    /// finding rather than "nothing to measure".
    ///
    /// The thresholds quoted here are read out of `RUST_PATTERN_THRESHOLDS`, the
    /// same table the extractors enforce. The previous wording was a hand-written
    /// "at least 3 structurally identical occurrences", true for only two of the
    /// six constructs: a fixture with five identical validation lines measured
    /// nothing while the note promised three would suffice.
    fn measurement_note(metrics: &EntropyMetrics, files: usize) -> Option<String> {
        if metrics.pattern_diversity.is_some() {
            return None;
        }
        Some(format!(
            "entropy not measured: no repeated pattern was detected in {files} file(s) \
             / {loc} source line(s). A construct is only counted once it recurs, \
             structurally identical, within a single file at least this many times: \
             {thresholds}. Small inputs therefore legitimately yield no distribution \
             to take the entropy of.",
            loc = metrics.total_loc,
            thresholds = Self::threshold_sentence()
        ))
    }

    /// "if/else chains 3, Result handling 3, API calls 4, …" — generated from the
    /// enforced table so the note cannot drift from the code.
    fn threshold_sentence() -> String {
        use crate::entropy::pattern_extractor::RUST_PATTERN_THRESHOLDS;

        let mut entries: Vec<(usize, &'static str)> = RUST_PATTERN_THRESHOLDS
            .iter()
            .map(|t| (t.effective_minimum(), t.name))
            .collect();
        // Sorted (count, then name) so the sentence is byte-identical run to run
        // and reads smallest-threshold-first.
        entries.sort_unstable();
        entries
            .iter()
            .map(|(n, name)| format!("{name} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// Regression tests for determinism and the measured-or-absent contract.
#[cfg(test)]
#[path = "determinism_tests.rs"]
mod determinism_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EntropyConfig::default();
        assert_eq!(config.max_pattern_repetition, 5);
        assert_eq!(config.min_pattern_diversity, 0.3);
        assert_eq!(config.max_cross_file_similarity, 0.7);
    }

    #[tokio::test]
    async fn test_analyzer_creation() {
        let analyzer = EntropyAnalyzer::new();
        assert!(!analyzer.config.pattern_types.is_empty());
    }
}
