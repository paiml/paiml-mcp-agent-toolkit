#![cfg_attr(coverage_nightly, coverage(off))]
//! ML-based SurvivabilityPredictor implementation.

use super::types::{MutantFeatures, PredictionResult, TrainingData};
use crate::services::mutation::{Mutant, MutationOperatorType};
use anyhow::Result;
use aprender::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// ML-based survivability predictor
#[derive(Debug)]
pub struct SurvivabilityPredictor {
    /// Trained LinearRegression model (using aprender)
    /// Performs binary classification with 0.5 threshold
    pub(super) model: Option<LinearRegression>,

    /// Historical kill rates by operator type (fallback/baseline)
    pub(super) operator_kill_rates: HashMap<MutationOperatorType, f64>,

    /// Feature importance scores from trained model
    pub(super) feature_importance: HashMap<String, f64>,

    /// Feature names for interpretation
    pub(super) feature_names: Vec<String>,

    /// Is the model trained?
    pub(super) trained: bool,

    /// Training data count
    pub(super) training_samples: usize,
}

impl SurvivabilityPredictor {
    /// Create new predictor
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let feature_names = vec![
            "operator_type".to_string(),
            "cyclomatic_complexity".to_string(),
            "cognitive_complexity".to_string(),
            "source_line".to_string(),
            "nesting_depth".to_string(),
            "control_flow_count".to_string(),
            "has_loops".to_string(),
            "has_conditionals".to_string(),
            "function_size".to_string(),
            "parameter_count".to_string(),
            "has_error_handling".to_string(),
            "has_assertions".to_string(),
            "token_count".to_string(),
            "unique_variables".to_string(),
            "has_arithmetic".to_string(),
            "has_comparisons".to_string(),
            "has_logical_ops".to_string(),
            "mutation_depth".to_string(),
        ];

        Self {
            model: None,
            operator_kill_rates: HashMap::new(),
            feature_importance: HashMap::new(),
            feature_names,
            trained: false,
            training_samples: 0,
        }
    }

    /// Train the predictor on historical data using LinearRegression
    /// Phase 4.3 GREEN - Aprender migration (0 dependencies vs linfa's 50+)
    #[allow(clippy::cast_possible_truncation)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn train(&mut self, training_data: &[TrainingData]) -> Result<()> {
        if training_data.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        // Extract features and labels
        let n_samples = training_data.len();
        let n_features = 18;

        let mut feature_matrix = Vec::with_capacity(n_samples * n_features);
        let mut labels = Vec::with_capacity(n_samples);

        for sample in training_data {
            let features = MutantFeatures::from_mutant(&sample.mutant);
            feature_matrix.extend_from_slice(&features.to_feature_vector());
            // Use 0.0 and 1.0 for regression-based classification
            labels.push(if sample.was_killed { 1.0 } else { 0.0 });
        }

        // Convert to aprender Matrix and Vector (aprender uses f32)
        let feature_matrix_f32: Vec<f32> = feature_matrix.iter().map(|&x| x as f32).collect();
        let labels_f32: Vec<f32> = labels.iter().map(|&x| x as f32).collect();

        // Try to train LinearRegression model
        // NOTE: This may fail for small sample sizes (n_samples < n_features = 18)
        // due to underdetermined system (matrix not positive definite)
        // In that case, we fall back to statistical baseline only
        match Matrix::from_vec(n_samples, n_features, feature_matrix_f32) {
            Ok(x) => {
                let y = Vector::from_vec(labels_f32);
                let mut model = LinearRegression::new();

                match model.fit(&x, &y) {
                    Ok(()) => {
                        self.model = Some(model);
                    }
                    Err(e) => {
                        // Model training failed (likely underdetermined system)
                        // Fall back to statistical baseline only
                        eprintln!("Warning: LinearRegression training failed ({}), using statistical baseline only", e);
                        self.model = None;
                    }
                }
            }
            Err(e) => {
                // Matrix creation failed
                eprintln!(
                    "Warning: Matrix creation failed ({}), using statistical baseline only",
                    e
                );
                self.model = None;
            }
        }

        // Calculate statistical baseline (fallback)
        let mut operator_counts: HashMap<MutationOperatorType, (usize, usize)> = HashMap::new();
        for sample in training_data {
            let entry = operator_counts
                .entry(sample.mutant.operator.clone())
                .or_insert((0, 0));
            entry.0 += 1;
            if sample.was_killed {
                entry.1 += 1;
            }
        }

        for (operator, (total, killed)) in operator_counts {
            let kill_rate = killed as f64 / total as f64;
            self.operator_kill_rates.insert(operator, kill_rate);
        }

        // Calculate feature importance from training data variance
        self.calculate_feature_importance(training_data);

        self.trained = true;
        self.training_samples = training_data.len();

        Ok(())
    }

    /// Perform k-fold cross-validation to measure model accuracy
    /// Returns average accuracy across folds
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cross_validate(&self, training_data: &[TrainingData], k_folds: usize) -> Result<f64> {
        if training_data.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }
        if k_folds < 2 {
            anyhow::bail!("k_folds must be at least 2");
        }

        let n_samples = training_data.len();
        let fold_size = n_samples / k_folds;

        if fold_size < 2 {
            anyhow::bail!("Not enough samples for {}-fold cross-validation", k_folds);
        }

        let mut accuracies = Vec::new();

        for fold in 0..k_folds {
            // Split data into train and test
            let test_start = fold * fold_size;
            let test_end = if fold == k_folds - 1 {
                n_samples
            } else {
                (fold + 1) * fold_size
            };

            let mut train_data = Vec::new();
            let mut test_data = Vec::new();

            for (i, sample) in training_data.iter().enumerate() {
                if i >= test_start && i < test_end {
                    test_data.push(sample.clone());
                } else {
                    train_data.push(sample.clone());
                }
            }

            // Train model on fold
            let mut fold_predictor = SurvivabilityPredictor::new();
            fold_predictor.train(&train_data)?;

            // Evaluate on test set
            let mut correct = 0;
            for sample in &test_data {
                if let Ok(prediction) = fold_predictor.predict(&sample.mutant) {
                    let predicted_killed = prediction.kill_probability > 0.5;
                    if predicted_killed == sample.was_killed {
                        correct += 1;
                    }
                }
            }

            let accuracy = correct as f64 / test_data.len() as f64;
            accuracies.push(accuracy);
        }

        // Return average accuracy
        let avg_accuracy = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
        Ok(avg_accuracy)
    }

    /// Calculate feature importance based on variance and correlation
    pub(super) fn calculate_feature_importance(&mut self, training_data: &[TrainingData]) {
        // Simple importance: measure feature variance for killed vs survived mutants
        let mut killed_features: Vec<Vec<f64>> = Vec::new();
        let mut survived_features: Vec<Vec<f64>> = Vec::new();

        for sample in training_data {
            let features = MutantFeatures::from_mutant(&sample.mutant);
            let feature_vec = features.to_feature_vector();

            if sample.was_killed {
                killed_features.push(feature_vec);
            } else {
                survived_features.push(feature_vec);
            }
        }

        // Calculate mean difference for each feature
        for (i, name) in self.feature_names.iter().enumerate() {
            let killed_mean = if !killed_features.is_empty() {
                killed_features.iter().map(|f| f[i]).sum::<f64>() / killed_features.len() as f64
            } else {
                0.0
            };

            let survived_mean = if !survived_features.is_empty() {
                survived_features.iter().map(|f| f[i]).sum::<f64>() / survived_features.len() as f64
            } else {
                0.0
            };

            let importance = (killed_mean - survived_mean).abs();
            self.feature_importance.insert(name.clone(), importance);
        }

        // Normalize importance scores
        let total_importance: f64 = self.feature_importance.values().sum();
        if total_importance > 0.0 {
            for value in self.feature_importance.values_mut() {
                *value /= total_importance;
            }
        }
    }

    /// Update model with new data (incremental learning)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update(&mut self, new_data: &[TrainingData]) -> Result<()> {
        if !self.trained {
            return self.train(new_data);
        }

        // Incremental update: re-train with combined data
        // Phase 1: Simple approach
        self.training_samples += new_data.len();

        // Update kill rates
        for sample in new_data {
            let current_rate = self
                .operator_kill_rates
                .get(&sample.mutant.operator)
                .copied()
                .unwrap_or(0.5);

            // Simple exponential moving average
            let alpha = 0.3; // Learning rate
            let new_rate = if sample.was_killed {
                current_rate * (1.0 - alpha) + alpha
            } else {
                current_rate * (1.0 - alpha)
            };

            self.operator_kill_rates
                .insert(sample.mutant.operator.clone(), new_rate);
        }

        Ok(())
    }

    /// Predict kill probability for a mutant using trained LinearRegression
    /// Phase 4.3 GREEN - Uses aprender LinearRegression with 18 features
    #[allow(clippy::cast_possible_truncation)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn predict(&self, mutant: &Mutant) -> Result<PredictionResult> {
        if !self.trained {
            anyhow::bail!("Model not trained");
        }

        let features = MutantFeatures::from_mutant(mutant);
        let feature_vec = features.to_feature_vector();

        // Use trained model if available
        let kill_probability = if let Some(ref model) = self.model {
            // Convert features to aprender Matrix (1 row, 18 cols) - use f32
            let feature_vec_f32: Vec<f32> = feature_vec.iter().map(|&x| x as f32).collect();
            let x = Matrix::from_vec(1, 18, feature_vec_f32)
                .map_err(|e| anyhow::anyhow!("Failed to create prediction matrix: {}", e))?;

            // Predict using LinearRegression
            let predictions = model.predict(&x);

            // Extract predicted value (continuous 0.0-1.0 from regression)
            // Clamp to [0.0, 1.0] range for probability
            // Access first element via as_slice() and convert to f64
            predictions.as_slice()[0].clamp(0.0, 1.0) as f64
        } else {
            // Fallback to statistical baseline
            let base_probability = self
                .operator_kill_rates
                .get(&mutant.operator)
                .copied()
                .unwrap_or(0.5);

            let complexity_factor = 1.0 + (features.cyclomatic_complexity as f64 / 100.0);
            (base_probability * complexity_factor).min(1.0)
        };

        // Confidence based on model and whether operator was seen
        let has_seen_operator = self.operator_kill_rates.contains_key(&mutant.operator);
        let confidence = if self.model.is_some() {
            if has_seen_operator {
                0.9 // High confidence with trained model for seen operators
            } else {
                0.7 // Medium confidence for unseen operators even with model
            }
        } else if has_seen_operator {
            0.8 // Good confidence with statistical baseline for seen operators
        } else {
            0.5 // Low confidence for unseen operators with baseline
        };

        // Feature contributions weighted by importance
        let mut feature_contributions = HashMap::new();
        for (name, &value) in self.feature_names.iter().zip(feature_vec.iter()) {
            let importance = self.feature_importance.get(name).copied().unwrap_or(0.0);
            feature_contributions.insert(name.clone(), value * importance);
        }

        Ok(PredictionResult {
            kill_probability,
            confidence,
            feature_contributions,
        })
    }

    /// Predict with human-readable explanation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn predict_with_explanation(&self, mutant: &Mutant) -> Result<(PredictionResult, String)> {
        let prediction = self.predict(mutant)?;

        let explanation = format!(
            "Kill probability: {:.1}% (confidence: {:.1}%). \
             Based on operator type {:?} with historical kill rate of {:.1}%.",
            prediction.kill_probability * 100.0,
            prediction.confidence * 100.0,
            mutant.operator,
            self.operator_kill_rates
                .get(&mutant.operator)
                .copied()
                .unwrap_or(0.5)
                * 100.0
        );

        Ok((prediction, explanation))
    }

    /// Prioritize mutants by predicted kill probability
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn prioritize_mutants(
        &self,
        mutants: &[Mutant],
    ) -> Result<Vec<(Mutant, PredictionResult)>> {
        let mut results = Vec::new();

        for mutant in mutants {
            let prediction = self.predict(mutant)?;
            results.push((mutant.clone(), prediction));
        }

        // Sort by kill probability (descending)
        results.sort_by(|a, b| {
            b.1.kill_probability
                .partial_cmp(&a.1.kill_probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Get feature importance scores
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn feature_importance(&self) -> Result<HashMap<String, f64>> {
        if !self.trained {
            anyhow::bail!("Model not trained");
        }

        Ok(self.feature_importance.clone())
    }

    /// Check if model is trained
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Save model to file
    /// NOTE: LinearRegression model is not currently serialized.
    /// After loading, the model will use statistical baseline predictions.
    /// For consistent ML predictions, retrain the model after loading.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save(&self, path: &Path) -> Result<()> {
        let serialized = rmp_serde::to_vec(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Load model from file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let predictor = rmp_serde::from_slice(&data)?;
        Ok(predictor)
    }
}

impl Default for SurvivabilityPredictor {
    fn default() -> Self {
        Self::new()
    }
}

// Make it serializable for save/load
// NOTE: LinearRegression model is not currently serialized - only fallback data and metadata
impl Serialize for SurvivabilityPredictor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SurvivabilityPredictor", 5)?;
        state.serialize_field("operator_kill_rates", &self.operator_kill_rates)?;
        state.serialize_field("feature_importance", &self.feature_importance)?;
        state.serialize_field("feature_names", &self.feature_names)?;
        state.serialize_field("trained", &self.trained)?;
        state.serialize_field("training_samples", &self.training_samples)?;
        // model field is skipped (LinearRegression serialization not yet implemented in aprender)
        state.end()
    }
}

impl<'de> Deserialize<'de> for SurvivabilityPredictor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PredictorData {
            operator_kill_rates: HashMap<MutationOperatorType, f64>,
            feature_importance: HashMap<String, f64>,
            feature_names: Vec<String>,
            trained: bool,
            training_samples: usize,
        }

        let data = PredictorData::deserialize(deserializer)?;
        Ok(Self {
            model: None, // Model must be retrained after loading
            operator_kill_rates: data.operator_kill_rates,
            feature_importance: data.feature_importance,
            feature_names: data.feature_names,
            trained: data.trained,
            training_samples: data.training_samples,
        })
    }
}
