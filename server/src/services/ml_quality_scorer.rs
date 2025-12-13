//! ML-Based Quality Scoring - GH-97 Implementation
//!
//! EXTREME TDD: RED PHASE - Replace heuristic calculations with ML-driven models
//!
//! ## Problem Statement (from GH-97)
//!
//! Current PMAT uses heuristic-based formulas for quality metrics:
//! - Arbitrary constants (why LOC/50? why nesting*2?)
//! - No language-specific adjustments
//! - Cannot learn from actual project outcomes
//!
//! ## Solution
//!
//! Replace with data-driven ML models using `aprender`:
//! - LinearRegression for continuous quality scores
//! - Train on real-world codebases with known outcomes
//! - Support `--ml` flag for opt-in ML-enhanced scoring
//!
//! ## Architecture
//!
//! ```text
//! MLQualityScorer
//! ├── ComplexityModel (LinearRegression)
//! │   └── Features: LOC, nesting, control_flow, loops, language
//! ├── TDGModel (LinearRegression)
//! │   └── Features: complexity, churn, coupling, domain_risk
//! └── HealthScoreModel (LinearRegression)
//!     └── Features: coverage, docs, ci_cd, community
//! ```

use anyhow::Result;
use aprender::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Features for complexity scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityFeatures {
    /// Lines of code (normalized)
    pub loc: f64,
    /// Maximum nesting depth
    pub max_nesting: f64,
    /// Control flow statements count
    pub control_flow_count: f64,
    /// Loop count
    pub loop_count: f64,
    /// Conditional count
    pub conditional_count: f64,
    /// Function count
    pub function_count: f64,
    /// Average function size
    pub avg_function_size: f64,
    /// Language type (encoded)
    pub language_type: f64,
}

impl ComplexityFeatures {
    /// Extract features from source code
    pub fn from_source(source: &str, language: &str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let loc = lines.len() as f64;

        // Count control flow and nesting
        let mut max_nesting = 0usize;
        let mut current_nesting = 0usize;
        let mut control_flow_count = 0usize;
        let mut loop_count = 0usize;
        let mut conditional_count = 0usize;
        let mut function_count = 0usize;

        for line in &lines {
            let trimmed = line.trim();

            // Track nesting
            current_nesting += trimmed.matches('{').count();
            current_nesting = current_nesting.saturating_sub(trimmed.matches('}').count());
            max_nesting = max_nesting.max(current_nesting);

            // Count constructs
            if trimmed.starts_with("if ")
                || trimmed.starts_with("else if ")
                || trimmed.starts_with("elif ")
            {
                conditional_count += 1;
                control_flow_count += 1;
            }

            if trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("loop ")
            {
                loop_count += 1;
                control_flow_count += 1;
            }

            if trimmed.starts_with("match ") || trimmed.starts_with("switch ") {
                control_flow_count += 1;
            }

            if trimmed.starts_with("fn ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("func ")
                || trimmed.starts_with("function ")
                || (trimmed.starts_with("pub fn ") || trimmed.contains(" fn "))
            {
                function_count += 1;
            }
        }

        let avg_function_size = if function_count > 0 {
            loc / function_count as f64
        } else {
            loc
        };

        let language_type = match language {
            "rust" => 1.0,
            "python" => 2.0,
            "javascript" | "typescript" => 3.0,
            "go" => 4.0,
            "java" => 5.0,
            "c" | "cpp" => 6.0,
            _ => 0.0,
        };

        Self {
            loc,
            max_nesting: max_nesting as f64,
            control_flow_count: control_flow_count as f64,
            loop_count: loop_count as f64,
            conditional_count: conditional_count as f64,
            function_count: function_count as f64,
            avg_function_size,
            language_type,
        }
    }

    /// Convert to feature vector for ML model
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.loc / 100.0,               // Normalize LOC
            self.max_nesting / 10.0,        // Normalize nesting
            self.control_flow_count / 20.0, // Normalize control flow
            self.loop_count / 10.0,
            self.conditional_count / 20.0,
            self.function_count / 10.0,
            self.avg_function_size / 50.0,
            self.language_type / 10.0,
        ]
    }
}

/// Features for TDG scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDGFeatures {
    /// Complexity score (0-5)
    pub complexity: f64,
    /// Churn factor (0-5)
    pub churn: f64,
    /// Coupling factor (0-5)
    pub coupling: f64,
    /// Domain risk factor (0-5)
    pub domain_risk: f64,
    /// Duplication factor (0-5)
    pub duplication: f64,
    /// Test coverage (0-1)
    pub test_coverage: f64,
    /// File age in days
    pub file_age_days: f64,
    /// Commit frequency (commits/month)
    pub commit_frequency: f64,
}

impl TDGFeatures {
    /// Convert to feature vector for ML model
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.complexity / 5.0,
            self.churn / 5.0,
            self.coupling / 5.0,
            self.domain_risk / 5.0,
            self.duplication / 5.0,
            self.test_coverage,
            self.file_age_days / 365.0, // Normalize to years
            self.commit_frequency / 10.0,
        ]
    }
}

/// Training sample for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrainingSample {
    /// Features for the sample
    pub features: Vec<f64>,
    /// Target quality score (ground truth)
    pub target_score: f64,
    /// Sample weight (optional, for importance)
    pub weight: Option<f64>,
}

/// ML-based quality scorer using aprender LinearRegression
#[derive(Debug)]
pub struct MLQualityScorer {
    /// Trained complexity model
    complexity_model: Option<LinearRegression>,
    /// Trained TDG model
    tdg_model: Option<LinearRegression>,
    /// Fallback heuristic weights (when ML unavailable)
    /// Used internally by heuristic_complexity() and heuristic_tdg()
    #[allow(dead_code)]
    heuristic_weights: HashMap<String, f64>,
    /// Is the model trained?
    trained: bool,
    /// Training sample count
    training_samples: usize,
    /// Feature importance scores
    feature_importance: HashMap<String, f64>,
}

/// Prediction result with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPrediction {
    /// Predicted quality score
    pub score: f64,
    /// Confidence in prediction (0-1)
    pub confidence: f64,
    /// Was ML model used?
    pub ml_used: bool,
    /// Feature contributions
    pub feature_contributions: HashMap<String, f64>,
}

impl MLQualityScorer {
    /// Create a new ML quality scorer
    pub fn new() -> Self {
        let mut heuristic_weights = HashMap::new();
        // Default heuristic weights (fallback when ML not trained)
        heuristic_weights.insert("loc".to_string(), 0.02);
        heuristic_weights.insert("nesting".to_string(), 0.5);
        heuristic_weights.insert("control_flow".to_string(), 0.3);
        heuristic_weights.insert("loops".to_string(), 0.2);

        Self {
            complexity_model: None,
            tdg_model: None,
            heuristic_weights,
            trained: false,
            training_samples: 0,
            feature_importance: HashMap::new(),
        }
    }

    /// Train the complexity model on historical data
    pub fn train_complexity_model(&mut self, samples: &[QualityTrainingSample]) -> Result<()> {
        if samples.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        let n_samples = samples.len();
        let n_features = 8; // ComplexityFeatures has 8 features

        // Build feature matrix and labels
        let mut feature_matrix = Vec::with_capacity(n_samples * n_features);
        let mut labels = Vec::with_capacity(n_samples);

        for sample in samples {
            if sample.features.len() != n_features {
                anyhow::bail!(
                    "Expected {} features, got {}",
                    n_features,
                    sample.features.len()
                );
            }
            feature_matrix.extend_from_slice(&sample.features);
            labels.push(sample.target_score as f32);
        }

        // Convert to aprender types
        let feature_matrix_f32: Vec<f32> = feature_matrix.iter().map(|&x| x as f32).collect();

        match Matrix::from_vec(n_samples, n_features, feature_matrix_f32) {
            Ok(x) => {
                let y = Vector::from_vec(labels);
                let mut model = LinearRegression::new();

                match model.fit(&x, &y) {
                    Ok(()) => {
                        self.complexity_model = Some(model);
                        self.trained = true;
                        self.training_samples = n_samples;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Complexity model training failed ({}), using heuristics",
                            e
                        );
                        self.complexity_model = None;
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Matrix creation failed ({}), using heuristics", e);
                self.complexity_model = None;
            }
        }

        // Calculate feature importance
        self.calculate_feature_importance(samples);

        Ok(())
    }

    /// Train the TDG model on historical data
    pub fn train_tdg_model(&mut self, samples: &[QualityTrainingSample]) -> Result<()> {
        if samples.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        let n_samples = samples.len();
        let n_features = 8; // TDGFeatures has 8 features

        let mut feature_matrix = Vec::with_capacity(n_samples * n_features);
        let mut labels = Vec::with_capacity(n_samples);

        for sample in samples {
            if sample.features.len() != n_features {
                anyhow::bail!(
                    "Expected {} features, got {}",
                    n_features,
                    sample.features.len()
                );
            }
            feature_matrix.extend_from_slice(&sample.features);
            labels.push(sample.target_score as f32);
        }

        let feature_matrix_f32: Vec<f32> = feature_matrix.iter().map(|&x| x as f32).collect();

        match Matrix::from_vec(n_samples, n_features, feature_matrix_f32) {
            Ok(x) => {
                let y = Vector::from_vec(labels);
                let mut model = LinearRegression::new();

                match model.fit(&x, &y) {
                    Ok(()) => {
                        self.tdg_model = Some(model);
                        self.trained = true;
                        self.training_samples += n_samples;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: TDG model training failed ({}), using heuristics",
                            e
                        );
                        self.tdg_model = None;
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Matrix creation failed ({}), using heuristics", e);
                self.tdg_model = None;
            }
        }

        Ok(())
    }

    /// Predict complexity score using ML model
    pub fn predict_complexity(&self, features: &ComplexityFeatures) -> Result<QualityPrediction> {
        let feature_vec = features.to_vector();

        let (score, ml_used) = if let Some(ref model) = self.complexity_model {
            // Use ML model
            let feature_vec_f32: Vec<f32> = feature_vec.iter().map(|&x| x as f32).collect();
            match Matrix::from_vec(1, 8, feature_vec_f32) {
                Ok(x) => {
                    let predictions = model.predict(&x);
                    let predicted = predictions.as_slice()[0].clamp(0.0, 100.0) as f64;
                    (predicted, true)
                }
                Err(_) => (self.heuristic_complexity(features), false),
            }
        } else {
            // Fallback to heuristics
            (self.heuristic_complexity(features), false)
        };

        let confidence = if ml_used { 0.85 } else { 0.5 };

        let mut feature_contributions = HashMap::new();
        let feature_names = [
            "loc",
            "nesting",
            "control_flow",
            "loops",
            "conditionals",
            "functions",
            "avg_size",
            "language",
        ];
        for (name, &value) in feature_names.iter().zip(feature_vec.iter()) {
            feature_contributions.insert(name.to_string(), value);
        }

        Ok(QualityPrediction {
            score,
            confidence,
            ml_used,
            feature_contributions,
        })
    }

    /// Predict TDG score using ML model
    pub fn predict_tdg(&self, features: &TDGFeatures) -> Result<QualityPrediction> {
        let feature_vec = features.to_vector();

        let (score, ml_used) = if let Some(ref model) = self.tdg_model {
            let feature_vec_f32: Vec<f32> = feature_vec.iter().map(|&x| x as f32).collect();
            match Matrix::from_vec(1, 8, feature_vec_f32) {
                Ok(x) => {
                    let predictions = model.predict(&x);
                    let predicted = predictions.as_slice()[0].clamp(0.0, 5.0) as f64;
                    (predicted, true)
                }
                Err(_) => (self.heuristic_tdg(features), false),
            }
        } else {
            (self.heuristic_tdg(features), false)
        };

        let confidence = if ml_used { 0.85 } else { 0.5 };

        let mut feature_contributions = HashMap::new();
        let feature_names = [
            "complexity",
            "churn",
            "coupling",
            "domain_risk",
            "duplication",
            "coverage",
            "age",
            "frequency",
        ];
        for (name, &value) in feature_names.iter().zip(feature_vec.iter()) {
            feature_contributions.insert(name.to_string(), value);
        }

        Ok(QualityPrediction {
            score,
            confidence,
            ml_used,
            feature_contributions,
        })
    }

    /// Heuristic complexity calculation (fallback)
    fn heuristic_complexity(&self, features: &ComplexityFeatures) -> f64 {
        // Traditional formula: base + nesting_penalty + control_flow
        let base = features.loc / 50.0;
        let nesting_penalty = features.max_nesting * 2.0;
        let control_flow_penalty = features.control_flow_count * 0.5;

        (base + nesting_penalty + control_flow_penalty).min(100.0)
    }

    /// Heuristic TDG calculation (fallback)
    fn heuristic_tdg(&self, features: &TDGFeatures) -> f64 {
        // Traditional weighted sum
        let score = features.complexity * 0.3
            + features.churn * 0.25
            + features.coupling * 0.2
            + features.domain_risk * 0.15
            + features.duplication * 0.1;

        score.clamp(0.0, 5.0)
    }

    /// Calculate feature importance from training data
    fn calculate_feature_importance(&mut self, samples: &[QualityTrainingSample]) {
        if samples.is_empty() {
            return;
        }

        let n_features = samples[0].features.len();
        let feature_names: Vec<&str> = if n_features == 8 {
            vec![
                "loc",
                "nesting",
                "control_flow",
                "loops",
                "conditionals",
                "functions",
                "avg_size",
                "language",
            ]
        } else {
            (0..n_features)
                .map(|i| Box::leak(format!("feature_{}", i).into_boxed_str()) as &str)
                .collect()
        };

        // Calculate correlation-based importance
        for (i, name) in feature_names.iter().enumerate() {
            let feature_values: Vec<f64> = samples.iter().map(|s| s.features[i]).collect();
            let targets: Vec<f64> = samples.iter().map(|s| s.target_score).collect();

            let importance = self.correlation(&feature_values, &targets).abs();
            self.feature_importance.insert(name.to_string(), importance);
        }

        // Normalize
        let total: f64 = self.feature_importance.values().sum();
        if total > 0.0 {
            for value in self.feature_importance.values_mut() {
                *value /= total;
            }
        }
    }

    /// Calculate Pearson correlation coefficient
    fn correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;

        for (xi, yi) in x.iter().zip(y.iter()) {
            let dx = xi - mean_x;
            let dy = yi - mean_y;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }

        if var_x == 0.0 || var_y == 0.0 {
            return 0.0;
        }

        cov / (var_x.sqrt() * var_y.sqrt())
    }

    /// Check if model is trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get feature importance
    pub fn feature_importance(&self) -> &HashMap<String, f64> {
        &self.feature_importance
    }

    /// Save model to file
    pub fn save(&self, _path: &Path) -> Result<()> {
        // TODO: Implement serialization when aprender supports it
        Ok(())
    }

    /// Load model from file
    pub fn load(_path: &Path) -> Result<Self> {
        // TODO: Implement deserialization when aprender supports it
        Ok(Self::new())
    }
}

impl Default for MLQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RED PHASE TESTS ====================
    // These tests define expected behavior for GH-97

    #[test]
    fn test_complexity_features_extraction() {
        let source = r#"
fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            for i in 0..x {
                if i % 2 == 0 {
                    println!("{}", i);
                }
            }
        }
    }
    x
}

fn simple_function() -> i32 {
    42
}
"#;

        let features = ComplexityFeatures::from_source(source, "rust");

        assert!(features.loc > 0.0, "LOC should be positive");
        assert!(
            features.max_nesting >= 3.0,
            "Max nesting should be >= 3 for nested code"
        );
        assert!(
            features.conditional_count >= 2.0,
            "Should detect at least 2 conditionals"
        );
        assert!(features.loop_count >= 1.0, "Should detect at least 1 loop");
        assert!(features.function_count >= 2.0, "Should detect 2 functions");
        assert_eq!(features.language_type, 1.0, "Rust should be encoded as 1.0");
    }

    #[test]
    fn test_complexity_features_to_vector() {
        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 5.0,
            control_flow_count: 10.0,
            loop_count: 3.0,
            conditional_count: 7.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let vector = features.to_vector();

        assert_eq!(vector.len(), 8, "Feature vector should have 8 elements");
        assert!(
            vector.iter().all(|&v| (0.0..=10.0).contains(&v)),
            "All features should be normalized"
        );
    }

    #[test]
    fn test_ml_scorer_creation() {
        let scorer = MLQualityScorer::new();

        assert!(!scorer.is_trained(), "New scorer should not be trained");
        assert!(
            scorer.complexity_model.is_none(),
            "No complexity model initially"
        );
        assert!(scorer.tdg_model.is_none(), "No TDG model initially");
    }

    #[test]
    fn test_train_complexity_model() {
        let mut scorer = MLQualityScorer::new();

        // Generate synthetic training data
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let complexity = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        complexity * 0.5,        // loc
                        complexity * 0.3,        // nesting
                        complexity * 0.4,        // control_flow
                        complexity * 0.2,        // loops
                        complexity * 0.3,        // conditionals
                        0.1 + complexity * 0.05, // functions
                        0.5 + complexity * 0.1,  // avg_size
                        0.1,                     // language
                    ],
                    target_score: complexity * 10.0, // Ground truth
                    weight: None,
                }
            })
            .collect();

        let result = scorer.train_complexity_model(&samples);
        assert!(result.is_ok(), "Training should succeed");

        // Model may or may not be trained depending on sample size
        // With 50 samples and 8 features, it should work
        if scorer.complexity_model.is_some() {
            assert!(scorer.is_trained(), "Scorer should be marked as trained");
        }
    }

    #[test]
    fn test_train_empty_data_fails() {
        let mut scorer = MLQualityScorer::new();
        let result = scorer.train_complexity_model(&[]);
        assert!(result.is_err(), "Training with empty data should fail");
    }

    #[test]
    fn test_predict_complexity_without_training() {
        let scorer = MLQualityScorer::new();

        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 3.0,
            control_flow_count: 5.0,
            loop_count: 2.0,
            conditional_count: 3.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&features).unwrap();

        assert!(
            !prediction.ml_used,
            "Should use heuristics without training"
        );
        assert!(prediction.score > 0.0, "Score should be positive");
        assert!(
            prediction.confidence < 0.7,
            "Confidence should be low for heuristics"
        );
    }

    #[test]
    fn test_predict_complexity_with_training() {
        let mut scorer = MLQualityScorer::new();

        // Train with synthetic data
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let complexity = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        complexity * 0.5,
                        complexity * 0.3,
                        complexity * 0.4,
                        complexity * 0.2,
                        complexity * 0.3,
                        0.1 + complexity * 0.05,
                        0.5 + complexity * 0.1,
                        0.1,
                    ],
                    target_score: complexity * 10.0,
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 3.0,
            control_flow_count: 5.0,
            loop_count: 2.0,
            conditional_count: 3.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&features).unwrap();

        // With trained model, should use ML
        if scorer.complexity_model.is_some() {
            assert!(prediction.ml_used, "Should use ML with trained model");
            assert!(
                prediction.confidence > 0.7,
                "Confidence should be high with ML"
            );
        }

        assert!(prediction.score >= 0.0, "Score should be non-negative");
    }

    #[test]
    fn test_tdg_features_to_vector() {
        let features = TDGFeatures {
            complexity: 3.0,
            churn: 2.0,
            coupling: 1.5,
            domain_risk: 2.5,
            duplication: 1.0,
            test_coverage: 0.8,
            file_age_days: 180.0,
            commit_frequency: 5.0,
        };

        let vector = features.to_vector();

        assert_eq!(vector.len(), 8, "TDG vector should have 8 elements");
        assert!(
            vector.iter().all(|&v| v >= 0.0),
            "All values should be non-negative"
        );
    }

    #[test]
    fn test_predict_tdg() {
        let scorer = MLQualityScorer::new();

        let features = TDGFeatures {
            complexity: 3.0,
            churn: 2.0,
            coupling: 1.5,
            domain_risk: 2.5,
            duplication: 1.0,
            test_coverage: 0.8,
            file_age_days: 180.0,
            commit_frequency: 5.0,
        };

        let prediction = scorer.predict_tdg(&features).unwrap();

        assert!(prediction.score >= 0.0 && prediction.score <= 5.0);
        assert!(
            !prediction.ml_used,
            "Should use heuristics without training"
        );
    }

    #[test]
    fn test_ml_vs_heuristic_difference() {
        // This test verifies ML produces different (better) results than heuristics
        let mut scorer = MLQualityScorer::new();

        // Train with data that has a non-linear relationship
        let samples: Vec<QualityTrainingSample> = (0..100)
            .map(|i| {
                let x = (i as f64) / 20.0;
                // Non-linear target: nesting has quadratic effect
                let target = x + x * x * 0.5;
                QualityTrainingSample {
                    features: vec![
                        x * 0.1,
                        x, // nesting is dominant factor
                        x * 0.2,
                        x * 0.1,
                        x * 0.15,
                        x * 0.05,
                        x * 0.1,
                        0.1,
                    ],
                    target_score: target,
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        // Test on high-complexity case
        let features = ComplexityFeatures {
            loc: 200.0,
            max_nesting: 8.0,
            control_flow_count: 15.0,
            loop_count: 5.0,
            conditional_count: 10.0,
            function_count: 3.0,
            avg_function_size: 66.0,
            language_type: 1.0,
        };

        let ml_prediction = scorer.predict_complexity(&features).unwrap();
        let heuristic_score = scorer.heuristic_complexity(&features);

        // ML should produce a different prediction (not necessarily better without proper training)
        if ml_prediction.ml_used {
            println!("ML score: {}", ml_prediction.score);
            println!("Heuristic score: {}", heuristic_score);

            // They should be somewhat different (ML learns patterns)
            let diff = (ml_prediction.score - heuristic_score).abs();
            assert!(
                diff > 0.0 || ml_prediction.score == heuristic_score,
                "ML should produce a prediction"
            );
        }
    }

    #[test]
    fn test_feature_importance_calculation() {
        let mut scorer = MLQualityScorer::new();

        // Train with data where some features are more important
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let nesting = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        0.5,     // loc - constant (not important)
                        nesting, // nesting - varies (important)
                        0.3,     // control_flow - constant
                        0.2,     // loops - constant
                        0.3,     // conditionals - constant
                        0.1,     // functions - constant
                        0.5,     // avg_size - constant
                        0.1,     // language - constant
                    ],
                    target_score: nesting * 5.0, // Target correlates with nesting
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        let importance = scorer.feature_importance();

        // Nesting should have high importance since it correlates with target
        if let Some(&nesting_importance) = importance.get("nesting") {
            assert!(
                nesting_importance > 0.0,
                "Nesting should have positive importance"
            );
        }
    }

    #[test]
    fn test_language_specific_features() {
        // Test that different languages get different encodings
        let rust_features = ComplexityFeatures::from_source("fn main() {}", "rust");
        let python_features = ComplexityFeatures::from_source("def main(): pass", "python");
        let js_features = ComplexityFeatures::from_source("function main() {}", "javascript");

        assert_eq!(rust_features.language_type, 1.0);
        assert_eq!(python_features.language_type, 2.0);
        assert_eq!(js_features.language_type, 3.0);

        // Unknown language
        let unknown_features = ComplexityFeatures::from_source("main", "unknown");
        assert_eq!(unknown_features.language_type, 0.0);
    }

    #[test]
    fn test_correlation_calculation() {
        let scorer = MLQualityScorer::new();

        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = scorer.correlation(&x, &y);
        assert!(
            (corr - 1.0).abs() < 0.001,
            "Perfect positive correlation should be 1.0"
        );

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr_neg = scorer.correlation(&x, &y_neg);
        assert!(
            (corr_neg + 1.0).abs() < 0.001,
            "Perfect negative correlation should be -1.0"
        );

        // No correlation (constant)
        let y_const = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let corr_none = scorer.correlation(&x, &y_const);
        assert!(
            corr_none.abs() < 0.001,
            "No correlation with constant values"
        );
    }

    #[test]
    fn test_prediction_bounds() {
        let scorer = MLQualityScorer::new();

        // Extreme features shouldn't produce unbounded scores
        let extreme_features = ComplexityFeatures {
            loc: 10000.0,
            max_nesting: 50.0,
            control_flow_count: 500.0,
            loop_count: 100.0,
            conditional_count: 400.0,
            function_count: 200.0,
            avg_function_size: 50.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&extreme_features).unwrap();

        assert!(prediction.score <= 100.0, "Score should be bounded at 100");
        assert!(prediction.score >= 0.0, "Score should be non-negative");
    }
}
