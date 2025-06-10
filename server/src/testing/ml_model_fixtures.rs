//! ML Model Test Fixtures
//!
//! This module provides test fixtures and utilities for testing machine learning
//! models used in defect prediction and code analysis.

use crate::models::error::PmatError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Test fixture for ML model validation
pub struct MlModelFixture {
    /// Model name for identification
    pub name: String,
    /// Test input features
    pub input_features: Vec<FeatureVector>,
    /// Expected predictions
    pub expected_predictions: Vec<Prediction>,
    /// Model performance metrics
    pub performance_metrics: ModelMetrics,
    /// Test configuration
    pub config: TestConfig,
}

/// Feature vector for ML model input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Complexity metrics
    pub complexity: ComplexityFeatures,
    /// Code churn metrics
    pub churn: ChurnFeatures,
    /// Duplication metrics
    pub duplication: DuplicationFeatures,
    /// Coupling metrics
    pub coupling: CouplingFeatures,
    /// Name quality metrics
    pub name_quality: NameQualityFeatures,
    /// Test coverage metrics
    pub test_coverage: CoverageFeatures,
}

/// Complexity-related features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityFeatures {
    pub cyclomatic_complexity: f32,
    pub cognitive_complexity: f32,
    pub lines_of_code: u32,
    pub function_count: u32,
    pub max_nesting_depth: u32,
}

/// Code churn features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnFeatures {
    pub commits_last_30_days: u32,
    pub authors_count: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub files_changed: u32,
}

/// Code duplication features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationFeatures {
    pub duplicate_line_ratio: f32,
    pub clone_groups: u32,
    pub largest_clone_size: u32,
    pub semantic_similarity_score: f32,
}

/// Coupling-related features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingFeatures {
    pub fan_in: u32,
    pub fan_out: u32,
    pub coupling_ratio: f32,
    pub dependency_depth: u32,
    pub circular_dependencies: u32,
}

/// Name quality features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameQualityFeatures {
    pub avg_identifier_length: f32,
    pub abbreviation_ratio: f32,
    pub consistency_score: f32,
    pub readability_score: f32,
}

/// Test coverage features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageFeatures {
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub function_coverage: f32,
    pub test_to_code_ratio: f32,
}

/// ML model prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// File path being predicted
    pub file_path: PathBuf,
    /// Defect probability (0.0 - 1.0)
    pub defect_probability: f32,
    /// Confidence in prediction (0.0 - 1.0)
    pub confidence: f32,
    /// Feature importance scores
    pub feature_importance: HashMap<String, f32>,
    /// Risk category
    pub risk_category: RiskCategory,
}

/// Risk categories for defect prediction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskCategory {
    Low,
    Medium,
    High,
    Critical,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Accuracy (0.0 - 1.0)
    pub accuracy: f32,
    /// Precision (0.0 - 1.0)
    pub precision: f32,
    /// Recall (0.0 - 1.0)
    pub recall: f32,
    /// F1 score (0.0 - 1.0)
    pub f1_score: f32,
    /// Area under ROC curve (0.0 - 1.0)
    pub auc_roc: f32,
    /// Mean absolute error
    pub mae: f32,
    /// Root mean squared error
    pub rmse: f32,
}

/// Test configuration for ML models
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Tolerance for prediction accuracy
    pub prediction_tolerance: f32,
    /// Minimum required confidence
    pub min_confidence: f32,
    /// Maximum allowed inference time (ms)
    pub max_inference_time_ms: u64,
    /// Expected model version
    pub expected_model_version: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            prediction_tolerance: 0.05, // 5% tolerance
            min_confidence: 0.7,        // 70% minimum confidence
            max_inference_time_ms: 100, // 100ms max inference
            expected_model_version: None,
        }
    }
}

impl MlModelFixture {
    /// Create a new ML model fixture
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input_features: Vec::new(),
            expected_predictions: Vec::new(),
            performance_metrics: ModelMetrics::default(),
            config: TestConfig::default(),
        }
    }

    /// Add a test case with input features and expected prediction
    pub fn with_test_case(
        mut self,
        features: FeatureVector,
        prediction: Prediction,
    ) -> Self {
        self.input_features.push(features);
        self.expected_predictions.push(prediction);
        self
    }

    /// Set model performance metrics
    pub fn with_performance_metrics(mut self, metrics: ModelMetrics) -> Self {
        self.performance_metrics = metrics;
        self
    }

    /// Set test configuration
    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    /// Validate model predictions against expected results
    pub fn validate_predictions(
        &self,
        actual_predictions: &[Prediction],
    ) -> Result<ValidationResult, PmatError> {
        if actual_predictions.len() != self.expected_predictions.len() {
            return Err(PmatError::ValidationError {
                field: "prediction_count".to_string(),
                reason: format!(
                    "Expected {} predictions, got {}",
                    self.expected_predictions.len(),
                    actual_predictions.len()
                ),
            });
        }

        let mut errors = Vec::new();
        let mut total_error = 0.0;
        let mut confidence_sum = 0.0;

        for (i, (actual, expected)) in actual_predictions.iter()
            .zip(self.expected_predictions.iter()).enumerate() {
            
            // Check prediction accuracy within tolerance
            let prediction_error = (actual.defect_probability - expected.defect_probability).abs();
            if prediction_error > self.config.prediction_tolerance {
                errors.push(format!(
                    "Prediction {}: error {:.3} exceeds tolerance {:.3}",
                    i, prediction_error, self.config.prediction_tolerance
                ));
            }
            total_error += prediction_error;

            // Check confidence threshold
            if actual.confidence < self.config.min_confidence {
                errors.push(format!(
                    "Prediction {}: confidence {:.3} below minimum {:.3}",
                    i, actual.confidence, self.config.min_confidence
                ));
            }
            confidence_sum += actual.confidence;

            // Check risk category alignment
            if actual.risk_category != expected.risk_category {
                errors.push(format!(
                    "Prediction {}: risk category {:?} != expected {:?}",
                    i, actual.risk_category, expected.risk_category
                ));
            }
        }

        let mean_error = total_error / actual_predictions.len() as f32;
        let mean_confidence = confidence_sum / actual_predictions.len() as f32;

        Ok(ValidationResult {
            passed: errors.is_empty(),
            errors,
            mean_prediction_error: mean_error,
            mean_confidence,
            model_metrics: self.performance_metrics.clone(),
        })
    }

    /// Create a realistic defect prediction test fixture
    pub fn defect_prediction_fixture() -> Self {
        let high_risk_features = FeatureVector {
            complexity: ComplexityFeatures {
                cyclomatic_complexity: 25.0,
                cognitive_complexity: 45.0,
                lines_of_code: 500,
                function_count: 15,
                max_nesting_depth: 8,
            },
            churn: ChurnFeatures {
                commits_last_30_days: 12,
                authors_count: 4,
                lines_added: 200,
                lines_removed: 150,
                files_changed: 8,
            },
            duplication: DuplicationFeatures {
                duplicate_line_ratio: 0.25,
                clone_groups: 5,
                largest_clone_size: 50,
                semantic_similarity_score: 0.8,
            },
            coupling: CouplingFeatures {
                fan_in: 8,
                fan_out: 12,
                coupling_ratio: 0.6,
                dependency_depth: 6,
                circular_dependencies: 2,
            },
            name_quality: NameQualityFeatures {
                avg_identifier_length: 4.5,
                abbreviation_ratio: 0.3,
                consistency_score: 0.6,
                readability_score: 0.5,
            },
            test_coverage: CoverageFeatures {
                line_coverage: 0.45,
                branch_coverage: 0.35,
                function_coverage: 0.5,
                test_to_code_ratio: 0.3,
            },
        };

        let high_risk_prediction = Prediction {
            file_path: PathBuf::from("src/high_risk_module.rs"),
            defect_probability: 0.85,
            confidence: 0.92,
            feature_importance: [
                ("complexity".to_string(), 0.35),
                ("churn".to_string(), 0.25),
                ("test_coverage".to_string(), 0.20),
                ("duplication".to_string(), 0.15),
                ("coupling".to_string(), 0.05),
            ].iter().cloned().collect(),
            risk_category: RiskCategory::High,
        };

        let low_risk_features = FeatureVector {
            complexity: ComplexityFeatures {
                cyclomatic_complexity: 5.0,
                cognitive_complexity: 8.0,
                lines_of_code: 150,
                function_count: 6,
                max_nesting_depth: 3,
            },
            churn: ChurnFeatures {
                commits_last_30_days: 2,
                authors_count: 1,
                lines_added: 30,
                lines_removed: 10,
                files_changed: 1,
            },
            duplication: DuplicationFeatures {
                duplicate_line_ratio: 0.05,
                clone_groups: 0,
                largest_clone_size: 0,
                semantic_similarity_score: 0.2,
            },
            coupling: CouplingFeatures {
                fan_in: 2,
                fan_out: 3,
                coupling_ratio: 0.2,
                dependency_depth: 2,
                circular_dependencies: 0,
            },
            name_quality: NameQualityFeatures {
                avg_identifier_length: 8.5,
                abbreviation_ratio: 0.1,
                consistency_score: 0.9,
                readability_score: 0.95,
            },
            test_coverage: CoverageFeatures {
                line_coverage: 0.95,
                branch_coverage: 0.90,
                function_coverage: 1.0,
                test_to_code_ratio: 1.2,
            },
        };

        let low_risk_prediction = Prediction {
            file_path: PathBuf::from("src/low_risk_module.rs"),
            defect_probability: 0.15,
            confidence: 0.88,
            feature_importance: [
                ("test_coverage".to_string(), 0.40),
                ("complexity".to_string(), 0.25),
                ("name_quality".to_string(), 0.20),
                ("churn".to_string(), 0.10),
                ("duplication".to_string(), 0.05),
            ].iter().cloned().collect(),
            risk_category: RiskCategory::Low,
        };

        Self::new("defect_prediction")
            .with_test_case(high_risk_features, high_risk_prediction)
            .with_test_case(low_risk_features, low_risk_prediction)
            .with_performance_metrics(ModelMetrics {
                accuracy: 0.87,
                precision: 0.84,
                recall: 0.89,
                f1_score: 0.86,
                auc_roc: 0.91,
                mae: 0.08,
                rmse: 0.12,
            })
    }

    /// Create a complexity prediction fixture
    pub fn complexity_prediction_fixture() -> Self {
        let complex_features = FeatureVector {
            complexity: ComplexityFeatures {
                cyclomatic_complexity: 40.0,
                cognitive_complexity: 65.0,
                lines_of_code: 800,
                function_count: 25,
                max_nesting_depth: 10,
            },
            churn: ChurnFeatures {
                commits_last_30_days: 8,
                authors_count: 3,
                lines_added: 300,
                lines_removed: 100,
                files_changed: 5,
            },
            duplication: DuplicationFeatures {
                duplicate_line_ratio: 0.15,
                clone_groups: 3,
                largest_clone_size: 30,
                semantic_similarity_score: 0.7,
            },
            coupling: CouplingFeatures {
                fan_in: 12,
                fan_out: 18,
                coupling_ratio: 0.8,
                dependency_depth: 8,
                circular_dependencies: 1,
            },
            name_quality: NameQualityFeatures {
                avg_identifier_length: 5.2,
                abbreviation_ratio: 0.25,
                consistency_score: 0.7,
                readability_score: 0.6,
            },
            test_coverage: CoverageFeatures {
                line_coverage: 0.6,
                branch_coverage: 0.55,
                function_coverage: 0.7,
                test_to_code_ratio: 0.4,
            },
        };

        let complex_prediction = Prediction {
            file_path: PathBuf::from("src/complex_module.rs"),
            defect_probability: 0.75,
            confidence: 0.90,
            feature_importance: [
                ("complexity".to_string(), 0.50),
                ("coupling".to_string(), 0.25),
                ("churn".to_string(), 0.15),
                ("test_coverage".to_string(), 0.10),
            ].iter().cloned().collect(),
            risk_category: RiskCategory::High,
        };

        Self::new("complexity_prediction")
            .with_test_case(complex_features, complex_prediction)
            .with_performance_metrics(ModelMetrics {
                accuracy: 0.82,
                precision: 0.79,
                recall: 0.85,
                f1_score: 0.82,
                auc_roc: 0.88,
                mae: 0.12,
                rmse: 0.18,
            })
    }
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.8,
            precision: 0.8,
            recall: 0.8,
            f1_score: 0.8,
            auc_roc: 0.8,
            mae: 0.1,
            rmse: 0.15,
        }
    }
}

/// Result of model validation
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Mean prediction error
    pub mean_prediction_error: f32,
    /// Mean confidence score
    pub mean_confidence: f32,
    /// Model performance metrics
    pub model_metrics: ModelMetrics,
}

/// Batch model tester for multiple fixtures
pub struct BatchModelTester {
    fixtures: Vec<MlModelFixture>,
}

impl BatchModelTester {
    /// Create a new batch tester
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
        }
    }

    /// Add a fixture to the batch
    pub fn add_fixture(mut self, fixture: MlModelFixture) -> Self {
        self.fixtures.push(fixture);
        self
    }

    /// Add all standard fixtures
    pub fn with_standard_fixtures(self) -> Self {
        self.add_fixture(MlModelFixture::defect_prediction_fixture())
            .add_fixture(MlModelFixture::complexity_prediction_fixture())
    }

    /// Run all fixtures and return results
    pub fn run_all_tests<F>(&self, model_fn: F) -> Vec<(String, ValidationResult)>
    where
        F: Fn(&[FeatureVector]) -> Result<Vec<Prediction>, PmatError>,
    {
        let mut results = Vec::new();

        for fixture in &self.fixtures {
            let predictions = match model_fn(&fixture.input_features) {
                Ok(preds) => preds,
                Err(e) => {
                    results.push((
                        fixture.name.clone(),
                        ValidationResult {
                            passed: false,
                            errors: vec![format!("Model execution failed: {}", e)],
                            mean_prediction_error: f32::INFINITY,
                            mean_confidence: 0.0,
                            model_metrics: ModelMetrics::default(),
                        },
                    ));
                    continue;
                }
            };

            match fixture.validate_predictions(&predictions) {
                Ok(result) => results.push((fixture.name.clone(), result)),
                Err(e) => {
                    results.push((
                        fixture.name.clone(),
                        ValidationResult {
                            passed: false,
                            errors: vec![format!("Validation failed: {}", e)],
                            mean_prediction_error: f32::INFINITY,
                            mean_confidence: 0.0,
                            model_metrics: ModelMetrics::default(),
                        },
                    ));
                }
            }
        }

        results
    }

    /// Check if all tests passed
    pub fn all_passed(&self, results: &[(String, ValidationResult)]) -> bool {
        results.iter().all(|(_, result)| result.passed)
    }

    /// Generate summary report
    pub fn generate_report(&self, results: &[(String, ValidationResult)]) -> String {
        let mut report = String::new();
        report.push_str("ML Model Test Report\n");
        report.push_str("===================\n\n");

        let passed_count = results.iter().filter(|(_, r)| r.passed).count();
        let total_count = results.len();

        report.push_str(&format!("Overall: {}/{} tests passed\n\n", passed_count, total_count));

        for (name, result) in results {
            report.push_str(&format!("Test: {}\n", name));
            report.push_str(&format!("Status: {}\n", if result.passed { "PASSED" } else { "FAILED" }));
            report.push_str(&format!("Mean Error: {:.4}\n", result.mean_prediction_error));
            report.push_str(&format!("Mean Confidence: {:.4}\n", result.mean_confidence));

            if !result.errors.is_empty() {
                report.push_str("Errors:\n");
                for error in &result.errors {
                    report.push_str(&format!("  - {}\n", error));
                }
            }

            report.push_str(&format!("Performance Metrics:\n"));
            report.push_str(&format!("  Accuracy: {:.3}\n", result.model_metrics.accuracy));
            report.push_str(&format!("  Precision: {:.3}\n", result.model_metrics.precision));
            report.push_str(&format!("  Recall: {:.3}\n", result.model_metrics.recall));
            report.push_str(&format!("  F1-Score: {:.3}\n", result.model_metrics.f1_score));
            report.push_str(&format!("  AUC-ROC: {:.3}\n", result.model_metrics.auc_roc));
            report.push_str("\n");
        }

        report
    }
}

impl Default for BatchModelTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_model_fixture_creation() {
        let fixture = MlModelFixture::new("test_model");
        assert_eq!(fixture.name, "test_model");
        assert!(fixture.input_features.is_empty());
        assert!(fixture.expected_predictions.is_empty());
    }

    #[test]
    fn test_defect_prediction_fixture() {
        let fixture = MlModelFixture::defect_prediction_fixture();
        assert_eq!(fixture.name, "defect_prediction");
        assert_eq!(fixture.input_features.len(), 2);
        assert_eq!(fixture.expected_predictions.len(), 2);

        // Check high risk prediction
        assert_eq!(fixture.expected_predictions[0].risk_category, RiskCategory::High);
        assert!(fixture.expected_predictions[0].defect_probability > 0.8);

        // Check low risk prediction  
        assert_eq!(fixture.expected_predictions[1].risk_category, RiskCategory::Low);
        assert!(fixture.expected_predictions[1].defect_probability < 0.2);
    }

    #[test]
    fn test_validation_with_accurate_predictions() {
        let fixture = MlModelFixture::defect_prediction_fixture();
        
        // Use exact expected predictions (should pass)
        let validation_result = fixture.validate_predictions(&fixture.expected_predictions).unwrap();
        
        assert!(validation_result.passed);
        assert!(validation_result.errors.is_empty());
        assert_eq!(validation_result.mean_prediction_error, 0.0);
    }

    #[test]
    fn test_validation_with_inaccurate_predictions() {
        let fixture = MlModelFixture::defect_prediction_fixture();
        
        // Create inaccurate predictions
        let mut bad_predictions = fixture.expected_predictions.clone();
        bad_predictions[0].defect_probability = 0.1; // Should be 0.85
        bad_predictions[0].confidence = 0.5; // Below minimum
        
        let validation_result = fixture.validate_predictions(&bad_predictions).unwrap();
        
        assert!(!validation_result.passed);
        assert!(!validation_result.errors.is_empty());
        assert!(validation_result.mean_prediction_error > fixture.config.prediction_tolerance);
    }

    #[test]
    fn test_batch_model_tester() {
        let tester = BatchModelTester::new().with_standard_fixtures();
        
        // Mock model function that returns exact expected predictions
        let mock_model = |features: &[FeatureVector]| -> Result<Vec<Prediction>, PmatError> {
            let mut predictions = Vec::new();
            
            for (i, _) in features.iter().enumerate() {
                let prediction = if i == 0 {
                    // High risk prediction
                    Prediction {
                        file_path: PathBuf::from("src/high_risk_module.rs"),
                        defect_probability: 0.85,
                        confidence: 0.92,
                        feature_importance: HashMap::new(),
                        risk_category: RiskCategory::High,
                    }
                } else {
                    // Low risk prediction
                    Prediction {
                        file_path: PathBuf::from("src/low_risk_module.rs"),
                        defect_probability: 0.15,
                        confidence: 0.88,
                        feature_importance: HashMap::new(),
                        risk_category: RiskCategory::Low,
                    }
                };
                predictions.push(prediction);
            }
            
            Ok(predictions)
        };
        
        let results = tester.run_all_tests(mock_model);
        assert!(!results.is_empty());
        
        let report = tester.generate_report(&results);
        assert!(report.contains("ML Model Test Report"));
    }

    #[test]
    fn test_risk_category_variants() {
        assert_eq!(RiskCategory::Low, RiskCategory::Low);
        assert_ne!(RiskCategory::Low, RiskCategory::High);
    }

    #[test]
    fn test_model_metrics_default() {
        let metrics = ModelMetrics::default();
        assert_eq!(metrics.accuracy, 0.8);
        assert_eq!(metrics.precision, 0.8);
        assert_eq!(metrics.f1_score, 0.8);
    }
}
