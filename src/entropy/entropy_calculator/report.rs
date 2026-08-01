#![cfg_attr(coverage_nightly, coverage(off))]
//! Report formatting and analysis methods for entropy reports.

use super::types::EntropyReport;

impl EntropyReport {
    /// Calculate total estimated lines of code reduction from all actionable violations
    ///
    /// Sums up the estimated LOC reduction from all detected patterns that can be
    /// refactored into reusable functions or modules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::entropy::entropy_calculator::{EntropyReport, EntropyMetrics};
    /// use pmat::entropy::violation_detector::PatternSummary;
    /// use pmat::entropy::PatternType;
    /// use std::collections::BTreeMap;
    ///
    /// // Create a mock report with empty violations for demonstration
    /// let pattern_summary = PatternSummary {
    ///     pattern_type: PatternType::ErrorHandling,
    ///     repetitions: 0,
    ///     variation_score: 0.0,
    ///     example_code: String::new(),
    /// };
    ///
    /// let report = EntropyReport {
    ///     total_files_analyzed: 5,
    ///     actionable_violations: vec![],  // Empty for simplicity
    ///     pattern_summary,
    ///     entropy_metrics: EntropyMetrics {
    ///         file_level_entropy: Some(0.7),
    ///         module_level_entropy: Some(0.6),
    ///         project_level_entropy: Some(0.55),
    ///         pattern_diversity: Some(0.6),
    ///         total_patterns: 8,
    ///         total_instances: 24,
    ///         total_loc: 500,
    ///         patterns_by_type: BTreeMap::new(),
    ///     },
    ///     measurement_note: None,
    /// };
    ///
    /// // With no actionable violations, LOC reduction should be 0
    /// assert_eq!(report.total_loc_reduction(), 0);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_loc_reduction(&self) -> usize {
        // Violations whose saving could not be estimated contribute nothing
        // rather than a plausible default (`filter_map`, not `unwrap_or(0)` on a
        // number we would then have printed as measured).
        self.actionable_violations
            .iter()
            .filter_map(|v| v.estimated_loc_reduction)
            .sum()
    }

    /// Calculate percentage of codebase that could be reduced through refactoring
    ///
    /// Returns the potential LOC reduction as a percentage of total analyzed code.
    /// Higher percentages indicate more duplication and better refactoring opportunities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::entropy::entropy_calculator::{EntropyReport, EntropyMetrics};
    /// use pmat::entropy::violation_detector::PatternSummary;
    /// use pmat::entropy::PatternType;
    /// use std::collections::BTreeMap;
    ///
    /// let pattern_summary = PatternSummary {
    ///     pattern_type: PatternType::ErrorHandling,
    ///     repetitions: 0,
    ///     variation_score: 0.0,
    ///     example_code: String::new(),
    /// };
    ///
    /// let report = EntropyReport {
    ///     total_files_analyzed: 10,
    ///     actionable_violations: vec![], // Empty violations
    ///     pattern_summary,
    ///     entropy_metrics: EntropyMetrics {
    ///         file_level_entropy: Some(0.8),
    ///         module_level_entropy: Some(0.75),
    ///         project_level_entropy: Some(0.7),
    ///         pattern_diversity: Some(0.75),
    ///         total_patterns: 0,
    ///         total_instances: 0,
    ///         total_loc: 1000, // Total lines analyzed
    ///         patterns_by_type: BTreeMap::new(),
    ///     },
    ///     measurement_note: None,
    /// };
    ///
    /// // With no violations, reduction percentage should be 0
    /// assert_eq!(report.reduction_percentage(), 0.0);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn reduction_percentage(&self) -> f64 {
        if self.entropy_metrics.total_loc > 0 {
            (self.total_loc_reduction() as f64 / self.entropy_metrics.total_loc as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Format as human-readable report
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Entropy Analysis Results\n");
        report.push_str("========================\n\n");

        report.push_str(&format!("Files Analyzed: {}\n", self.total_files_analyzed));
        report.push_str(&format!(
            "Source Lines Analyzed: {}\n",
            self.entropy_metrics.total_loc
        ));
        report.push_str(&format!(
            "Pattern Diversity: {}\n",
            Self::render_measurement(self.entropy_metrics.pattern_diversity)
        ));
        report.push_str(&format!(
            "Actionable Violations: {}\n",
            self.actionable_violations.len()
        ));
        if let Some(note) = &self.measurement_note {
            report.push_str(&format!("Note: {note}\n"));
        }
        report.push('\n');

        // Group violations by severity
        let mut high = Vec::new();
        let mut medium = Vec::new();
        let mut low = Vec::new();

        for violation in &self.actionable_violations {
            match violation.severity {
                crate::entropy::violation_detector::Severity::High => high.push(violation),
                crate::entropy::violation_detector::Severity::Medium => medium.push(violation),
                crate::entropy::violation_detector::Severity::Low => low.push(violation),
            }
        }

        // Every band is rendered. LOW was omitted, so with `--min-severity low`
        // the "Actionable Violations: N" header counted rows the report never
        // listed.
        Self::push_severity_section(&mut report, "HIGH", &high);
        Self::push_severity_section(&mut report, "MEDIUM", &medium);
        Self::push_severity_section(&mut report, "LOW", &low);

        report.push_str(&format!(
            "Total Potential Reduction: {} lines ({:.1}% of analyzed code)\n",
            self.total_loc_reduction(),
            self.reduction_percentage()
        ));

        report
    }

    /// Render one severity band. `saves` reads "not estimated" where the saving
    /// was never derived from a measured pattern size, rather than "0 lines".
    fn push_severity_section(
        report: &mut String,
        label: &str,
        violations: &[&crate::entropy::violation_detector::ActionableViolation],
    ) {
        if violations.is_empty() {
            return;
        }
        report.push_str(&format!("{label} SEVERITY ({}):\n", violations.len()));
        for (i, v) in violations.iter().enumerate() {
            report.push_str(&format!(
                "{}. {}\n   Fix: {} - saves {}\n\n",
                i + 1,
                v.message,
                v.fix_suggestion,
                v.render_loc_reduction()
            ));
        }
    }

    /// Render an optional measurement without ever printing a placeholder number.
    #[must_use]
    pub fn render_measurement(value: Option<f64>) -> String {
        value.map_or_else(|| "not measured".to_string(), |v| format!("{v:.3}"))
    }
}
