use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use syn;
use thiserror::Error;

use super::complexity::ComplexityAnalyzer;
use super::efficiency::EfficiencyAnalyzer;
use super::entropy::EntropyCalculator;
use super::satd::SatdDetector;

#[derive(Debug, Error)]
/// Quality violation.
pub enum QualityViolation {
    #[error("Excessive complexity: found {found}, max allowed {max} at {location:?}")]
    ExcessiveComplexity {
        found: u32,
        max: u32,
        location: std::path::PathBuf,
    },
    #[error("SATD detected: {count} occurrences of {patterns:?} at {location:?}")]
    SatdDetected {
        count: usize,
        patterns: Vec<String>,
        location: std::path::PathBuf,
    },
    #[error("Inefficient algorithm: function {function} has complexity {complexity}, required {required}")]
    InefficientAlgorithm {
        function: String,
        complexity: String,
        required: String,
    },
    #[error("Insufficient diversity: entropy {entropy}, required {required}")]
    InsufficientDiversity { entropy: f64, required: f64 },
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Threshold values for qualitys.
pub struct QualityThresholds {
    pub max_cyclomatic: u32,
    pub max_cognitive: u32,
    pub max_nesting: u32,
    pub max_params: usize,
    pub max_lines: usize,
    pub satd_tolerance: usize,
    pub max_big_o: String,
    pub min_entropy: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_cyclomatic: 10,
            max_cognitive: 7,
            max_nesting: 3,
            max_params: 4,
            max_lines: 50,
            satd_tolerance: 0,
            max_big_o: "O(n log n)".to_string(),
            min_entropy: 3.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Report containing quality data.
pub struct QualityReport {
    pub passed: bool,
    pub metrics: QualityMetrics,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Quality metrics.
pub struct QualityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub nesting_depth: u32,
    pub satd_count: usize,
    pub entropy: f64,
    pub efficiency: String,
}

impl QualityReport {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// A passing report whose `metrics` are the [`QualityMetrics::default`]
    /// CONSTANTS — cyclomatic 1, cognitive 0, entropy 0.0, `O(1)` — not
    /// measurements of anything.
    ///
    /// Use [`QualityReport::measured`] when the numbers are known;
    /// `validate_module` used to return this, so every module that passed
    /// reported those same six values regardless of its contents.
    pub fn passed() -> Self {
        Self {
            passed: true,
            metrics: QualityMetrics::default(),
            violations: Vec::new(),
        }
    }

    /// A passing report carrying the metrics that were actually measured.
    #[must_use]
    pub fn measured(metrics: QualityMetrics) -> Self {
        Self {
            passed: true,
            metrics,
            violations: Vec::new(),
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 0,
            nesting_depth: 0,
            satd_count: 0,
            entropy: 0.0,
            efficiency: "O(1)".to_string(),
        }
    }
}

/// Quality gate runner.
pub struct QualityGateRunner {
    /// Analyzers whose input is the parsed AST.
    ///
    /// `SatdDetector` and `EntropyCalculator` are deliberately absent: both
    /// read the source TEXT, which [`QualityAnalyzer::analyze`] — whose only
    /// argument is a `&syn::File` — cannot hand them. That signature mismatch
    /// is why this registry was left empty behind a "fix analyzer trait
    /// implementations" debt marker for four releases (#973); the two text-based
    /// detectors are called directly in [`QualityGateRunner::measure`].
    analyzers: Vec<Box<dyn QualityAnalyzer>>,
    thresholds: QualityThresholds,
}

impl QualityGateRunner {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(thresholds: QualityThresholds) -> Self {
        Self {
            analyzers: vec![
                Box::new(ComplexityAnalyzer::new()),
                Box::new(EfficiencyAnalyzer::new()),
            ],
            thresholds,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Strict.
    pub fn strict() -> Self {
        Self::new(QualityThresholds::default())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Validate module.
    pub fn validate_module(&self, module_path: &Path) -> Result<QualityReport, QualityViolation> {
        let source = fs::read_to_string(module_path)
            .map_err(|e| QualityViolation::ParseError(e.to_string()))?;

        // Parse AST
        let ast =
            syn::parse_file(&source).map_err(|e| QualityViolation::ParseError(e.to_string()))?;

        // Measure once, then judge. Every number below is a measurement of
        // THIS module and is carried into the returned report.
        let satd_results = self.detect_satd(&source)?;
        let metrics = self.measure(&ast, &source, satd_results.count);

        // Run complexity analysis
        if metrics.cyclomatic_complexity > self.thresholds.max_cyclomatic {
            return Err(QualityViolation::ExcessiveComplexity {
                found: metrics.cyclomatic_complexity,
                max: self.thresholds.max_cyclomatic,
                location: module_path.to_path_buf(),
            });
        }

        // Run SATD detection
        if satd_results.count > self.thresholds.satd_tolerance {
            return Err(QualityViolation::SatdDetected {
                count: satd_results.count,
                patterns: satd_results.patterns,
                location: module_path.to_path_buf(),
            });
        }

        // Run efficiency analysis
        if !self.is_efficiency_acceptable(&metrics.efficiency) {
            return Err(QualityViolation::InefficientAlgorithm {
                function: "unknown".to_string(),
                complexity: metrics.efficiency,
                required: self.thresholds.max_big_o.clone(),
            });
        }

        // Calculate entropy
        if metrics.entropy < self.thresholds.min_entropy {
            return Err(QualityViolation::InsufficientDiversity {
                entropy: metrics.entropy,
                required: self.thresholds.min_entropy,
            });
        }

        Ok(QualityReport::measured(metrics))
    }

    /// Fold every registered analyzer's view of `ast`, plus the source-text
    /// entropy and the already-computed SATD count, into one measured
    /// [`QualityMetrics`].
    ///
    /// Analyzers report only the dimensions they know about and leave the rest
    /// at zero, so numeric dimensions combine by `max` and the efficiency class
    /// by "worst wins" (via [`QualityGateRunner::parse_big_o`]).
    fn measure(&self, ast: &syn::File, source: &str, satd_count: usize) -> QualityMetrics {
        let mut metrics = QualityMetrics {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            nesting_depth: 0,
            satd_count,
            entropy: self.calculate_entropy(source),
            efficiency: "O(1)".to_string(),
        };

        for analyzer in &self.analyzers {
            let m = analyzer.analyze(ast);
            metrics.cyclomatic_complexity =
                metrics.cyclomatic_complexity.max(m.cyclomatic_complexity);
            metrics.cognitive_complexity = metrics.cognitive_complexity.max(m.cognitive_complexity);
            metrics.nesting_depth = metrics.nesting_depth.max(m.nesting_depth);
            if self.parse_big_o(&m.efficiency) > self.parse_big_o(&metrics.efficiency) {
                metrics.efficiency = m.efficiency;
            }
        }

        metrics
    }

    // `analyze_complexity` and `analyze_efficiency` used to live here, each
    // building its own `ComplexityAnalyzer` / `EfficiencyAnalyzer` — a second
    // implementation of what the (then-empty) analyzer registry existed to do.
    // Both are gone; `measure` drives the registry instead.

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn detect_satd(&self, source: &str) -> Result<SatdResult, QualityViolation> {
        let detector = SatdDetector::new();
        Ok(detector.detect(source))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn calculate_entropy(&self, source: &str) -> f64 {
        let calculator = EntropyCalculator::new();
        calculator.calculate(source)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn is_efficiency_acceptable(&self, efficiency: &str) -> bool {
        // Simple comparison logic for now
        let order = self.parse_big_o(&self.thresholds.max_big_o);
        let actual = self.parse_big_o(efficiency);
        actual <= order
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn parse_big_o(&self, notation: &str) -> u32 {
        // Simplified parsing - assign numeric values to complexity classes
        match notation {
            "O(1)" => 1,
            "O(log n)" => 2,
            "O(n)" => 3,
            "O(n log n)" => 4,
            "O(n^2)" => 5,
            "O(n^3)" => 6,
            _ => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Result of satd operation.
pub struct SatdResult {
    pub count: usize,
    pub patterns: Vec<String>,
}

// `QualityAnalyzer` was declared TWICE — once here and once in
// `super::analyzers` — and the two declarations differed (`analyzers` adds
// `name()`). Every existing `impl` targeted the OTHER one, so this copy had
// zero implementors and the registry it typed could not be filled: that is the
// whole content of the "fix analyzer trait implementations" debt marker (#973).
// The duplicate is deleted; the path `quality::gate::QualityAnalyzer` keeps
// resolving, now to the single declaration that has implementors.
pub use super::analyzers::QualityAnalyzer;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    include!("gate_tests_types.rs");
    include!("gate_tests_runner.rs");
    include!("gate_tests_validation.rs");
}
