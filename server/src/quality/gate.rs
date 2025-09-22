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
pub struct QualityReport {
    pub passed: bool,
    pub metrics: QualityMetrics,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub nesting_depth: u32,
    pub satd_count: usize,
    pub entropy: f64,
    pub efficiency: String,
}

impl QualityReport {
    pub fn passed() -> Self {
        Self {
            passed: true,
            metrics: QualityMetrics::default(),
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

pub struct QualityGateRunner {
    _analyzers: Vec<Box<dyn QualityAnalyzer>>,
    thresholds: QualityThresholds,
}

impl QualityGateRunner {
    pub fn new(thresholds: QualityThresholds) -> Self {
        Self {
            _analyzers: vec![
                // TODO: Fix analyzer trait implementations
                // Box::new(ComplexityAnalyzer::new()),
                // Box::new(SatdDetector::new()),
                // Box::new(EfficiencyAnalyzer::new()),
                // Box::new(EntropyCalculator::new()),
            ],
            thresholds,
        }
    }

    pub fn strict() -> Self {
        Self::new(QualityThresholds::default())
    }

    pub fn validate_module(&self, module_path: &Path) -> Result<QualityReport, QualityViolation> {
        let source = fs::read_to_string(module_path)
            .map_err(|e| QualityViolation::ParseError(e.to_string()))?;

        // Parse AST
        let ast =
            syn::parse_file(&source).map_err(|e| QualityViolation::ParseError(e.to_string()))?;

        // Run complexity analysis
        let complexity = self.analyze_complexity(&ast)?;
        if complexity > self.thresholds.max_cyclomatic {
            return Err(QualityViolation::ExcessiveComplexity {
                found: complexity,
                max: self.thresholds.max_cyclomatic,
                location: module_path.to_path_buf(),
            });
        }

        // Run SATD detection
        let satd_results = self.detect_satd(&source)?;
        if satd_results.count > self.thresholds.satd_tolerance {
            return Err(QualityViolation::SatdDetected {
                count: satd_results.count,
                patterns: satd_results.patterns,
                location: module_path.to_path_buf(),
            });
        }

        // Run efficiency analysis
        let efficiency = self.analyze_efficiency(&ast)?;
        if !self.is_efficiency_acceptable(&efficiency) {
            return Err(QualityViolation::InefficientAlgorithm {
                function: "unknown".to_string(),
                complexity: efficiency,
                required: self.thresholds.max_big_o.clone(),
            });
        }

        // Calculate entropy
        let entropy = self.calculate_entropy(&source);
        if entropy < self.thresholds.min_entropy {
            return Err(QualityViolation::InsufficientDiversity {
                entropy,
                required: self.thresholds.min_entropy,
            });
        }

        Ok(QualityReport::passed())
    }

    fn analyze_complexity(&self, ast: &syn::File) -> Result<u32, QualityViolation> {
        let analyzer = ComplexityAnalyzer::new();
        Ok(analyzer.calculate_cyclomatic(ast))
    }

    fn detect_satd(&self, source: &str) -> Result<SatdResult, QualityViolation> {
        let detector = SatdDetector::new();
        Ok(detector.detect(source))
    }

    fn analyze_efficiency(&self, ast: &syn::File) -> Result<String, QualityViolation> {
        let analyzer = EfficiencyAnalyzer::new();
        Ok(analyzer.analyze(ast))
    }

    fn calculate_entropy(&self, source: &str) -> f64 {
        let calculator = EntropyCalculator::new();
        calculator.calculate(source)
    }

    fn is_efficiency_acceptable(&self, efficiency: &str) -> bool {
        // Simple comparison logic for now
        let order = self.parse_big_o(&self.thresholds.max_big_o);
        let actual = self.parse_big_o(efficiency);
        actual <= order
    }

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
pub struct SatdResult {
    pub count: usize,
    pub patterns: Vec<String>,
}

pub trait QualityAnalyzer: Send + Sync {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics;
}
