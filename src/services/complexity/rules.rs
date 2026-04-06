#![cfg_attr(coverage_nightly, coverage(off))]
//! Complexity rule implementations for cyclomatic and cognitive complexity.

use super::types::{ComplexityMetrics, ComplexityThresholds, Violation};

/// Trait for complexity rules
pub trait ComplexityRule: Send + Sync {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation>;

    #[inline(always)]
    fn exceeds_threshold(&self, value: u16, threshold: u16) -> bool {
        value > threshold
    }
}

/// Cyclomatic complexity rule implementation
pub struct CyclomaticComplexityRule {
    pub(crate) warn_threshold: u16,
    pub(crate) error_threshold: u16,
}

impl CyclomaticComplexityRule {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cyclomatic_warn,
            error_threshold: thresholds.cyclomatic_error,
        }
    }
}

impl ComplexityRule for CyclomaticComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cyclomatic, self.error_threshold) {
            Some(Violation::Error {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cyclomatic, self.error_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cyclomatic, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cyclomatic-complexity".to_string(),
                message: format!(
                    "Cyclomatic complexity of {} exceeds recommended complexity of {}",
                    metrics.cyclomatic, self.warn_threshold
                ),
                value: metrics.cyclomatic,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}

/// Cognitive complexity rule implementation
pub struct CognitiveComplexityRule {
    pub(crate) warn_threshold: u16,
    pub(crate) error_threshold: u16,
}

impl CognitiveComplexityRule {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(thresholds: &ComplexityThresholds) -> Self {
        Self {
            warn_threshold: thresholds.cognitive_warn,
            error_threshold: thresholds.cognitive_error,
        }
    }
}

impl ComplexityRule for CognitiveComplexityRule {
    fn evaluate(
        &self,
        metrics: &ComplexityMetrics,
        file: &str,
        line: u32,
        function: Option<&str>,
    ) -> Option<Violation> {
        if self.exceeds_threshold(metrics.cognitive, self.error_threshold) {
            Some(Violation::Error {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds maximum allowed complexity of {}",
                    metrics.cognitive, self.error_threshold
                ),
                value: metrics.cognitive,
                threshold: self.error_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else if self.exceeds_threshold(metrics.cognitive, self.warn_threshold) {
            Some(Violation::Warning {
                rule: "cognitive-complexity".to_string(),
                message: format!(
                    "Cognitive complexity of {} exceeds recommended complexity of {}",
                    metrics.cognitive, self.warn_threshold
                ),
                value: metrics.cognitive,
                threshold: self.warn_threshold,
                file: file.to_string(),
                line,
                function: function.map(String::from),
            })
        } else {
            None
        }
    }
}
