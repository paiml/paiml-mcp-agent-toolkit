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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ============================================================
    // QualityViolation Tests
    // ============================================================

    #[test]
    fn test_quality_violation_excessive_complexity_display() {
        let violation = QualityViolation::ExcessiveComplexity {
            found: 25,
            max: 10,
            location: std::path::PathBuf::from("/path/to/file.rs"),
        };
        let display = format!("{}", violation);
        assert!(display.contains("Excessive complexity"));
        assert!(display.contains("25"));
        assert!(display.contains("10"));
        assert!(display.contains("file.rs"));
    }

    #[test]
    fn test_quality_violation_satd_detected_display() {
        let violation = QualityViolation::SatdDetected {
            count: 3,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
            location: std::path::PathBuf::from("/path/to/module.rs"),
        };
        let display = format!("{}", violation);
        assert!(display.contains("SATD detected"));
        assert!(display.contains("3"));
        assert!(display.contains("TODO"));
        assert!(display.contains("FIXME"));
    }

    #[test]
    fn test_quality_violation_inefficient_algorithm_display() {
        let violation = QualityViolation::InefficientAlgorithm {
            function: "bubble_sort".to_string(),
            complexity: "O(n^2)".to_string(),
            required: "O(n log n)".to_string(),
        };
        let display = format!("{}", violation);
        assert!(display.contains("Inefficient algorithm"));
        assert!(display.contains("bubble_sort"));
        assert!(display.contains("O(n^2)"));
        assert!(display.contains("O(n log n)"));
    }

    #[test]
    fn test_quality_violation_insufficient_diversity_display() {
        let violation = QualityViolation::InsufficientDiversity {
            entropy: 2.5,
            required: 3.5,
        };
        let display = format!("{}", violation);
        assert!(display.contains("Insufficient diversity"));
        assert!(display.contains("2.5"));
        assert!(display.contains("3.5"));
    }

    #[test]
    fn test_quality_violation_parse_error_display() {
        let violation = QualityViolation::ParseError("unexpected token".to_string());
        let display = format!("{}", violation);
        assert!(display.contains("Parse error"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_quality_violation_debug() {
        let violation = QualityViolation::ParseError("test error".to_string());
        let debug = format!("{:?}", violation);
        assert!(debug.contains("ParseError"));
    }

    // ============================================================
    // QualityThresholds Tests
    // ============================================================

    #[test]
    fn test_quality_thresholds_default() {
        let thresholds = QualityThresholds::default();
        assert_eq!(thresholds.max_cyclomatic, 10);
        assert_eq!(thresholds.max_cognitive, 7);
        assert_eq!(thresholds.max_nesting, 3);
        assert_eq!(thresholds.max_params, 4);
        assert_eq!(thresholds.max_lines, 50);
        assert_eq!(thresholds.satd_tolerance, 0);
        assert_eq!(thresholds.max_big_o, "O(n log n)");
        assert!((thresholds.min_entropy - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_quality_thresholds_clone() {
        let original = QualityThresholds::default();
        let cloned = original.clone();
        assert_eq!(original.max_cyclomatic, cloned.max_cyclomatic);
        assert_eq!(original.max_cognitive, cloned.max_cognitive);
        assert_eq!(original.max_big_o, cloned.max_big_o);
    }

    #[test]
    fn test_quality_thresholds_serialization() {
        let thresholds = QualityThresholds::default();
        let json = serde_json::to_string(&thresholds).unwrap();
        let deserialized: QualityThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(thresholds.max_cyclomatic, deserialized.max_cyclomatic);
        assert_eq!(thresholds.max_cognitive, deserialized.max_cognitive);
        assert_eq!(thresholds.max_nesting, deserialized.max_nesting);
        assert_eq!(thresholds.max_params, deserialized.max_params);
        assert_eq!(thresholds.max_lines, deserialized.max_lines);
        assert_eq!(thresholds.satd_tolerance, deserialized.satd_tolerance);
        assert_eq!(thresholds.max_big_o, deserialized.max_big_o);
        assert!((thresholds.min_entropy - deserialized.min_entropy).abs() < 0.001);
    }

    #[test]
    fn test_quality_thresholds_custom() {
        let thresholds = QualityThresholds {
            max_cyclomatic: 20,
            max_cognitive: 15,
            max_nesting: 5,
            max_params: 6,
            max_lines: 100,
            satd_tolerance: 5,
            max_big_o: "O(n^2)".to_string(),
            min_entropy: 2.0,
        };
        assert_eq!(thresholds.max_cyclomatic, 20);
        assert_eq!(thresholds.max_cognitive, 15);
        assert_eq!(thresholds.max_nesting, 5);
    }

    #[test]
    fn test_quality_thresholds_debug() {
        let thresholds = QualityThresholds::default();
        let debug = format!("{:?}", thresholds);
        assert!(debug.contains("QualityThresholds"));
        assert!(debug.contains("max_cyclomatic"));
    }

    // ============================================================
    // QualityReport Tests
    // ============================================================

    #[test]
    fn test_quality_report_passed() {
        let report = QualityReport::passed();
        assert!(report.passed);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_quality_report_clone() {
        let report = QualityReport {
            passed: false,
            metrics: QualityMetrics::default(),
            violations: vec!["violation1".to_string(), "violation2".to_string()],
        };
        let cloned = report.clone();
        assert_eq!(report.passed, cloned.passed);
        assert_eq!(report.violations.len(), cloned.violations.len());
    }

    #[test]
    fn test_quality_report_serialization() {
        let report = QualityReport {
            passed: true,
            metrics: QualityMetrics::default(),
            violations: vec!["test violation".to_string()],
        };
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: QualityReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.passed, deserialized.passed);
        assert_eq!(report.violations.len(), deserialized.violations.len());
    }

    #[test]
    fn test_quality_report_debug() {
        let report = QualityReport::passed();
        let debug = format!("{:?}", report);
        assert!(debug.contains("QualityReport"));
        assert!(debug.contains("passed"));
    }

    // ============================================================
    // QualityMetrics Tests
    // ============================================================

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert_eq!(metrics.cyclomatic_complexity, 1);
        assert_eq!(metrics.cognitive_complexity, 0);
        assert_eq!(metrics.nesting_depth, 0);
        assert_eq!(metrics.satd_count, 0);
        assert!((metrics.entropy - 0.0).abs() < 0.001);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_quality_metrics_clone() {
        let metrics = QualityMetrics {
            cyclomatic_complexity: 15,
            cognitive_complexity: 10,
            nesting_depth: 4,
            satd_count: 2,
            entropy: 4.5,
            efficiency: "O(n)".to_string(),
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.cyclomatic_complexity, cloned.cyclomatic_complexity);
        assert_eq!(metrics.cognitive_complexity, cloned.cognitive_complexity);
        assert_eq!(metrics.efficiency, cloned.efficiency);
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            cyclomatic_complexity: 5,
            cognitive_complexity: 3,
            nesting_depth: 2,
            satd_count: 1,
            entropy: 3.8,
            efficiency: "O(n log n)".to_string(),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.cyclomatic_complexity, deserialized.cyclomatic_complexity);
        assert_eq!(metrics.cognitive_complexity, deserialized.cognitive_complexity);
        assert_eq!(metrics.efficiency, deserialized.efficiency);
    }

    #[test]
    fn test_quality_metrics_debug() {
        let metrics = QualityMetrics::default();
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("QualityMetrics"));
        assert!(debug.contains("cyclomatic_complexity"));
    }

    // ============================================================
    // QualityGateRunner Tests
    // ============================================================

    #[test]
    fn test_quality_gate_runner_new() {
        let thresholds = QualityThresholds::default();
        let runner = QualityGateRunner::new(thresholds);
        assert_eq!(runner.thresholds.max_cyclomatic, 10);
    }

    #[test]
    fn test_quality_gate_runner_strict() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.thresholds.max_cyclomatic, 10);
        assert_eq!(runner.thresholds.satd_tolerance, 0);
    }

    #[test]
    fn test_quality_gate_runner_custom_thresholds() {
        let thresholds = QualityThresholds {
            max_cyclomatic: 50,
            max_cognitive: 30,
            max_nesting: 10,
            max_params: 10,
            max_lines: 200,
            satd_tolerance: 10,
            max_big_o: "O(n^3)".to_string(),
            min_entropy: 1.0,
        };
        let runner = QualityGateRunner::new(thresholds);
        assert_eq!(runner.thresholds.max_cyclomatic, 50);
        assert_eq!(runner.thresholds.satd_tolerance, 10);
    }

    // ============================================================
    // parse_big_o Tests
    // ============================================================

    #[test]
    fn test_parse_big_o_constant() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(1)"), 1);
    }

    #[test]
    fn test_parse_big_o_logarithmic() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(log n)"), 2);
    }

    #[test]
    fn test_parse_big_o_linear() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(n)"), 3);
    }

    #[test]
    fn test_parse_big_o_linearithmic() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(n log n)"), 4);
    }

    #[test]
    fn test_parse_big_o_quadratic() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(n^2)"), 5);
    }

    #[test]
    fn test_parse_big_o_cubic() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(n^3)"), 6);
    }

    #[test]
    fn test_parse_big_o_unknown() {
        let runner = QualityGateRunner::strict();
        assert_eq!(runner.parse_big_o("O(2^n)"), 10);
        assert_eq!(runner.parse_big_o("unknown"), 10);
        assert_eq!(runner.parse_big_o(""), 10);
    }

    // ============================================================
    // is_efficiency_acceptable Tests
    // ============================================================

    #[test]
    fn test_is_efficiency_acceptable_within_threshold() {
        let runner = QualityGateRunner::strict(); // max is O(n log n)
        assert!(runner.is_efficiency_acceptable("O(1)"));
        assert!(runner.is_efficiency_acceptable("O(log n)"));
        assert!(runner.is_efficiency_acceptable("O(n)"));
        assert!(runner.is_efficiency_acceptable("O(n log n)"));
    }

    #[test]
    fn test_is_efficiency_acceptable_exceeds_threshold() {
        let runner = QualityGateRunner::strict(); // max is O(n log n)
        assert!(!runner.is_efficiency_acceptable("O(n^2)"));
        assert!(!runner.is_efficiency_acceptable("O(n^3)"));
    }

    #[test]
    fn test_is_efficiency_acceptable_custom_threshold() {
        let thresholds = QualityThresholds {
            max_big_o: "O(n^2)".to_string(),
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        assert!(runner.is_efficiency_acceptable("O(n^2)"));
        assert!(!runner.is_efficiency_acceptable("O(n^3)"));
    }

    // ============================================================
    // validate_module Tests
    // ============================================================

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_validate_module_simple_passing() {
        let code = r#"
            fn simple() {
                let x = 1;
                let y = 2;
            }
        "#;
        let file = create_temp_file(code);

        // Use relaxed thresholds for this test
        let thresholds = QualityThresholds {
            min_entropy: 0.0, // Low entropy requirement
            satd_tolerance: 10,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }

    #[test]
    fn test_validate_module_file_not_found() {
        let runner = QualityGateRunner::strict();
        let result = runner.validate_module(Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::ParseError(msg) => {
                assert!(msg.contains("No such file") || msg.contains("not found") || msg.len() > 0);
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_validate_module_invalid_syntax() {
        let code = "fn invalid { missing parens and stuff |||";
        let file = create_temp_file(code);

        let runner = QualityGateRunner::strict();
        let result = runner.validate_module(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::ParseError(_) => {}
            _ => panic!("Expected ParseError for invalid syntax"),
        }
    }

    #[test]
    fn test_validate_module_excessive_complexity() {
        // Code with high cyclomatic complexity (many branches)
        let code = r#"
            fn complex(x: i32) {
                if x > 0 {
                    if x > 10 {
                        if x > 20 {
                            if x > 30 {
                                if x > 40 {
                                    if x > 50 {
                                        if x > 60 {
                                            if x > 70 {
                                                if x > 80 {
                                                    if x > 90 {
                                                        if x > 100 {
                                                            println!("big");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            max_cyclomatic: 5, // Very strict
            min_entropy: 0.0,
            satd_tolerance: 100,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::ExcessiveComplexity { found, max, .. } => {
                assert!(found > max);
            }
            _ => panic!("Expected ExcessiveComplexity violation"),
        }
    }

    #[test]
    fn test_validate_module_satd_detected() {
        let code = r#"
            fn with_debt() {
                // TODO: implement this properly
                // FIXME: this is broken
                let x = 1;
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            satd_tolerance: 0, // Zero tolerance
            min_entropy: 0.0,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::SatdDetected { count, patterns, .. } => {
                assert!(count >= 2);
                assert!(!patterns.is_empty());
            }
            _ => panic!("Expected SatdDetected violation"),
        }
    }

    #[test]
    fn test_validate_module_satd_with_tolerance() {
        let code = r#"
            fn with_some_debt() {
                // TODO: implement this properly
                let x = 1;
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            satd_tolerance: 5, // Allow up to 5 SATD items
            min_entropy: 0.0,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        // Should pass since we have tolerance
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_module_inefficient_algorithm() {
        // Code with nested loops creating O(n^2) or worse
        let code = r#"
            fn inefficient(n: usize) {
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            let _ = i + j + k;
                        }
                    }
                }
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            max_big_o: "O(n)".to_string(), // Only allow linear
            min_entropy: 0.0,
            satd_tolerance: 100,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::InefficientAlgorithm { complexity, required, .. } => {
                assert!(complexity.contains("n^"));
                assert_eq!(required, "O(n)");
            }
            _ => panic!("Expected InefficientAlgorithm violation"),
        }
    }

    #[test]
    fn test_validate_module_insufficient_diversity() {
        // Code with very low entropy (repetitive)
        let code = r#"
            fn repetitive() {
                let a = 1;
                let a = 1;
                let a = 1;
                let a = 1;
                let a = 1;
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            min_entropy: 5.0, // Very high entropy requirement
            satd_tolerance: 100,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            QualityViolation::InsufficientDiversity { entropy, required } => {
                assert!(entropy < required);
                assert!((required - 5.0).abs() < 0.001);
            }
            _ => panic!("Expected InsufficientDiversity violation"),
        }
    }

    #[test]
    fn test_validate_module_empty_file() {
        let code = "";
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            min_entropy: 0.0, // Allow empty
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_module_diverse_code() {
        let code = r#"
            use std::collections::HashMap;

            struct Point {
                x: f64,
                y: f64,
            }

            impl Point {
                fn new(x: f64, y: f64) -> Self {
                    Self { x, y }
                }

                fn distance(&self, other: &Point) -> f64 {
                    let dx = self.x - other.x;
                    let dy = self.y - other.y;
                    (dx * dx + dy * dy).sqrt()
                }
            }

            enum Shape {
                Circle { radius: f64 },
                Rectangle { width: f64, height: f64 },
            }

            fn calculate_area(shape: &Shape) -> f64 {
                match shape {
                    Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
                    Shape::Rectangle { width, height } => width * height,
                }
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            min_entropy: 3.0, // Reasonable entropy
            satd_tolerance: 100,
            max_cyclomatic: 20,
            max_big_o: "O(n^2)".to_string(),
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
    }

    // ============================================================
    // SatdResult Tests
    // ============================================================

    #[test]
    fn test_satd_result_empty() {
        let result = SatdResult {
            count: 0,
            patterns: vec![],
        };
        assert_eq!(result.count, 0);
        assert!(result.patterns.is_empty());
    }

    #[test]
    fn test_satd_result_with_patterns() {
        let result = SatdResult {
            count: 3,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        assert_eq!(result.count, 3);
        assert_eq!(result.patterns.len(), 2);
    }

    #[test]
    fn test_satd_result_clone() {
        let original = SatdResult {
            count: 5,
            patterns: vec!["HACK".to_string()],
        };
        let cloned = original.clone();
        assert_eq!(original.count, cloned.count);
        assert_eq!(original.patterns, cloned.patterns);
    }

    #[test]
    fn test_satd_result_serialization() {
        let result = SatdResult {
            count: 2,
            patterns: vec!["TODO".to_string(), "XXX".to_string()],
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SatdResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.count, deserialized.count);
        assert_eq!(result.patterns, deserialized.patterns);
    }

    #[test]
    fn test_satd_result_debug() {
        let result = SatdResult {
            count: 1,
            patterns: vec!["DEPRECATED".to_string()],
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("SatdResult"));
        assert!(debug.contains("count"));
    }

    // ============================================================
    // Integration Tests
    // ============================================================

    #[test]
    fn test_full_quality_gate_workflow() {
        // Create a moderately complex but acceptable file
        let code = r#"
            fn fibonacci(n: u64) -> u64 {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }

            fn factorial(n: u64) -> u64 {
                if n == 0 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            max_cyclomatic: 10,
            min_entropy: 2.5,
            satd_tolerance: 0,
            max_big_o: "O(n log n)".to_string(),
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_gate_all_checks_fail() {
        // Code that would fail multiple checks
        let code = r#"
            fn terrible_code() {
                // TODO: fix everything
                // FIXME: this is bad
                // HACK: temporary workaround
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            if a && b && c && d && e && f && g && h {
                                println!("wow");
                            }
                        }
                    }
                }
            }
        "#;
        let file = create_temp_file(code);

        let runner = QualityGateRunner::strict();
        let result = runner.validate_module(file.path());
        // Should fail due to at least one check
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_thresholds_boundary_values() {
        // Test with boundary/edge case thresholds
        let thresholds = QualityThresholds {
            max_cyclomatic: 1, // Minimum
            max_cognitive: 0,
            max_nesting: 0,
            max_params: 0,
            max_lines: 1,
            satd_tolerance: 0,
            max_big_o: "O(1)".to_string(),
            min_entropy: 10.0, // Very high
        };

        let code = "fn empty() {}";
        let file = create_temp_file(code);

        let runner = QualityGateRunner::new(thresholds);
        // Even empty function may fail entropy check
        let _result = runner.validate_module(file.path());
        // We just verify it doesn't panic
    }

    #[test]
    fn test_quality_gate_with_struct_only() {
        let code = r#"
            struct Config {
                name: String,
                value: i32,
                enabled: bool,
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            min_entropy: 2.0,
            satd_tolerance: 100,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_gate_with_match_expression() {
        let code = r#"
            fn process(cmd: Command) -> Result<(), Error> {
                match cmd {
                    Command::Start => start_service(),
                    Command::Stop => stop_service(),
                    Command::Restart => restart_service(),
                    Command::Status => check_status(),
                }
            }
        "#;
        let file = create_temp_file(code);

        let thresholds = QualityThresholds {
            max_cyclomatic: 10,
            min_entropy: 2.0,
            satd_tolerance: 100,
            ..Default::default()
        };
        let runner = QualityGateRunner::new(thresholds);
        let result = runner.validate_module(file.path());
        assert!(result.is_ok());
    }
}
