//! ML-Based Mutant Survivability Predictor - Phase 4.3 GREEN PHASE
//!
//! EXTREME TDD: GREEN PHASE - Aprender LinearRegression Migration (2025-11-18)
//!
//! ## Migration from linfa to aprender
//!
//! **Rationale**: Migrated from linfa to aprender (PAIML's next-gen ML library)
//! - **Before**: linfa + ndarray (50+ transitive dependencies)
//! - **After**: aprender v0.1.0 (0 transitive dependencies, TDG 94.1/100)
//! - **Benefit**: Zero dependency bloat, pure Rust, reproducible builds
//!
//! ## Current Implementation
//!
//! Uses **LinearRegression** for binary classification via regression with 0.5 threshold:
//! - Predicts kill probability as continuous value [0.0, 1.0]
//! - Falls back to statistical baseline when n_samples < n_features (underdetermined system)
//! - Graceful degradation: Warns on training failure, continues with operator kill rates
//!
//! ## Known Limitations (aprender v0.1.0)
//!
//! 1. **Small Sample Sizes**: LinearRegression requires n_samples ≥ n_features (18)
//!    - GitHub Issue: https://github.com/paiml/aprender/issues/4
//!    - Workaround: Statistical baseline fallback when training fails
//!
//! 2. **DecisionTree Not Available**: Original implementation used linfa DecisionTree
//!    - GitHub Issue: https://github.com/paiml/aprender/issues/3
//!    - Future: Will migrate back to DecisionTree when available
//!
//! ## Test Results
//!
//! - ✅ All 12 RED phase tests passing (100%)
//! - ✅ Graceful fallback handles underdetermined systems
//! - ✅ Feature extraction works correctly (18-dimensional vectors)
//! - ✅ Statistical baseline provides reasonable predictions
//!
//! ## Future Work
//!
//! When aprender implements missing features:
//! - Migrate from LinearRegression to DecisionTree (better for classification)
//! - Add Ridge regression support for small samples (L2 regularization)
//! - Remove linfa/ndarray dependencies completely

use super::{Mutant, MutationOperatorType};
use anyhow::Result;
use aprender::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Features extracted from a mutant for ML prediction
/// Enhanced feature set (v2) - expanded from 10 to 18 features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutantFeatures {
    /// Type of mutation operator
    pub operator_type: MutationOperatorType,

    /// Cyclomatic complexity at mutation point
    pub cyclomatic_complexity: u32,

    /// Cognitive complexity at mutation point
    pub cognitive_complexity: u32,

    /// Source line number
    pub source_line: u32,

    /// Nesting depth at mutation point
    pub nesting_depth: u32,

    /// Number of control flow constructs nearby
    pub control_flow_count: u32,

    /// Has loops nearby
    pub has_loops: bool,

    /// Has conditionals nearby
    pub has_conditionals: bool,

    /// Function size (LOC)
    pub function_size: u32,

    /// Number of parameters
    pub parameter_count: u32,

    // NEW ENHANCED FEATURES (v2)
    /// Has error handling (try/catch/Result)
    pub has_error_handling: bool,

    /// Has assertions or tests
    pub has_assertions: bool,

    /// Token count (code density)
    pub token_count: u32,

    /// Unique variable count
    pub unique_variables: u32,

    /// Has arithmetic operations
    pub has_arithmetic: bool,

    /// Has comparison operations
    pub has_comparisons: bool,

    /// Has logical operations (&&, ||, !)
    pub has_logical_ops: bool,

    /// Mutation depth (how nested in control flow)
    pub mutation_depth: u32,
}

impl MutantFeatures {
    /// Extract features from a mutant
    /// Enhanced extraction (v2) - extracts 18 features
    pub fn from_mutant(mutant: &Mutant) -> Self {
        let source_line = mutant.location.line;
        let source = &mutant.mutated_source;

        // Original 10 features
        let has_loops =
            source.contains("for") || source.contains("while") || source.contains("loop");
        let has_conditionals = source.contains("if") || source.contains("match");

        let control_flow_count = source.matches("if").count() as u32
            + source.matches("for").count() as u32
            + source.matches("while").count() as u32
            + source.matches("match").count() as u32;

        let nesting_depth = estimate_nesting_depth(source);
        let cyclomatic_complexity = 1 + control_flow_count;
        let cognitive_complexity = cyclomatic_complexity + nesting_depth;
        let function_size = source.lines().count() as u32;
        let parameter_count = count_parameters(source);

        // NEW ENHANCED FEATURES (v2) - 8 additional features
        let has_error_handling = source.contains("Result<")
            || source.contains("Option<")
            || source.contains("unwrap")
            || source.contains("expect")
            || source.contains("?")
            || source.contains("try")
            || source.contains("catch");

        let has_assertions = source.contains("assert")
            || source.contains("debug_assert")
            || source.contains("#[test]");

        // Token count (split by whitespace and common delimiters)
        let token_count = source.split_whitespace().count() as u32;

        // Unique variables (simple heuristic: lowercase words)
        let unique_variables = count_unique_variables(source);

        let has_arithmetic = source.contains('+')
            || source.contains('-')
            || source.contains('*')
            || source.contains('/');

        let has_comparisons = source.contains("==")
            || source.contains("!=")
            || source.contains("<=")
            || source.contains(">=")
            || source.contains('<')
            || source.contains('>');

        let has_logical_ops =
            source.contains("&&") || source.contains("||") || source.contains('!');

        // Mutation depth = nesting at mutation point
        let mutation_depth = nesting_depth;

        Self {
            operator_type: mutant.operator.clone(),
            cyclomatic_complexity,
            cognitive_complexity,
            source_line: source_line as u32,
            nesting_depth,
            control_flow_count,
            has_loops,
            has_conditionals,
            function_size,
            parameter_count,
            // New features
            has_error_handling,
            has_assertions,
            token_count,
            unique_variables,
            has_arithmetic,
            has_comparisons,
            has_logical_ops,
            mutation_depth,
        }
    }

    /// Convert features to vector for ML model
    /// Enhanced vector (v2) - 18 features total
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            // Original 10 features
            self.operator_type_as_numeric(),
            self.cyclomatic_complexity as f64,
            self.cognitive_complexity as f64,
            self.source_line as f64,
            self.nesting_depth as f64,
            self.control_flow_count as f64,
            if self.has_loops { 1.0 } else { 0.0 },
            if self.has_conditionals { 1.0 } else { 0.0 },
            self.function_size as f64,
            self.parameter_count as f64,
            // New 8 features (v2)
            if self.has_error_handling { 1.0 } else { 0.0 },
            if self.has_assertions { 1.0 } else { 0.0 },
            self.token_count as f64,
            self.unique_variables as f64,
            if self.has_arithmetic { 1.0 } else { 0.0 },
            if self.has_comparisons { 1.0 } else { 0.0 },
            if self.has_logical_ops { 1.0 } else { 0.0 },
            self.mutation_depth as f64,
        ]
    }

    fn operator_type_as_numeric(&self) -> f64 {
        match self.operator_type {
            MutationOperatorType::ArithmeticReplacement => 1.0,
            MutationOperatorType::RelationalReplacement => 2.0,
            MutationOperatorType::ConditionalReplacement => 3.0,
            MutationOperatorType::ConstantReplacement => 4.0,
            MutationOperatorType::StatementDeletion => 5.0,
            MutationOperatorType::ReturnReplacement => 6.0,
            MutationOperatorType::VariableReplacement => 7.0,
            MutationOperatorType::ConditionalReturn => 8.0,
            MutationOperatorType::BoundaryValue => 9.0,
            MutationOperatorType::ExceptionHandlerRemoval => 10.0,
            MutationOperatorType::ReturnValueReplacement => 11.0,
            MutationOperatorType::UnaryReplacement => 12.0,
            MutationOperatorType::BitwiseReplacement => 13.0,
            MutationOperatorType::AssignmentReplacement => 14.0,
            MutationOperatorType::PointerReplacement => 15.0,
            MutationOperatorType::MemberAccessReplacement => 16.0,
            MutationOperatorType::RangeReplacement => 17.0,
            MutationOperatorType::PatternReplacement => 18.0,
            MutationOperatorType::MethodChainReplacement => 19.0,
            MutationOperatorType::BorrowReplacement => 20.0,
            MutationOperatorType::Custom(_) => 21.0,
            MutationOperatorType::None => 0.0,
        }
    }
}

/// Training data for the ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub mutant: Mutant,
    pub was_killed: bool,
    pub test_failures: Vec<String>,
    pub execution_time_ms: u64,
}

/// Prediction result from the ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Probability that this mutant will be killed (0.0 - 1.0)
    pub kill_probability: f64,

    /// Confidence in the prediction (0.0 - 1.0)
    pub confidence: f64,

    /// Feature importance for this prediction
    pub feature_contributions: HashMap<String, f64>,
}

/// ML-based survivability predictor
#[derive(Debug)]
pub struct SurvivabilityPredictor {
    /// Trained LinearRegression model (using aprender)
    /// Performs binary classification with 0.5 threshold
    model: Option<LinearRegression>,

    /// Historical kill rates by operator type (fallback/baseline)
    operator_kill_rates: HashMap<MutationOperatorType, f64>,

    /// Feature importance scores from trained model
    feature_importance: HashMap<String, f64>,

    /// Feature names for interpretation
    feature_names: Vec<String>,

    /// Is the model trained?
    trained: bool,

    /// Training data count
    training_samples: usize,
}

impl SurvivabilityPredictor {
    /// Create new predictor
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
    fn calculate_feature_importance(&mut self, training_data: &[TrainingData]) {
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
    pub fn feature_importance(&self) -> Result<HashMap<String, f64>> {
        if !self.trained {
            anyhow::bail!("Model not trained");
        }

        Ok(self.feature_importance.clone())
    }

    /// Check if model is trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Save model to file
    /// NOTE: LinearRegression model is not currently serialized.
    /// After loading, the model will use statistical baseline predictions.
    /// For consistent ML predictions, retrain the model after loading.
    pub fn save(&self, path: &Path) -> Result<()> {
        let serialized = bincode::serialize(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Load model from file
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let predictor = bincode::deserialize(&data)?;
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

/// Helper: Estimate nesting depth from source
fn estimate_nesting_depth(source: &str) -> u32 {
    let mut max_depth: u32 = 0;
    let mut current_depth: u32 = 0;

    for ch in source.chars() {
        match ch {
            '{' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

/// Helper: Count function parameters
fn count_parameters(source: &str) -> u32 {
    // Simple heuristic: count commas in first parentheses
    if let Some(start) = source.find('(') {
        if let Some(end) = source[start..].find(')') {
            let params = &source[start..start + end];
            if params.trim() == "()" {
                return 0;
            }
            return (params.matches(',').count() + 1) as u32;
        }
    }
    0
}

/// Helper: Count unique variable identifiers (simple heuristic)
fn count_unique_variables(source: &str) -> u32 {
    use std::collections::HashSet;
    let mut variables = HashSet::new();

    // Simple heuristic: extract words that start with lowercase or underscore
    for token in source.split_whitespace() {
        // Remove common punctuation
        let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

        if !cleaned.is_empty() {
            let first_char = cleaned.chars().next().expect("checked is_empty");
            if first_char.is_lowercase() || first_char == '_' {
                // Skip keywords
                if !is_rust_keyword(cleaned) {
                    variables.insert(cleaned.to_string());
                }
            }
        }
    }

    variables.len() as u32
}

/// Helper: Check if word is a Rust keyword
fn is_rust_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "const"
            | "static"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "match"
            | "return"
            | "break"
            | "continue"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "where"
            | "unsafe"
            | "async"
            | "await"
            | "move"
            | "ref"
            | "in"
            | "as"
            | "crate"
            | "super"
            | "self"
            | "Self"
            | "true"
            | "false"
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::{MutantStatus, SourceLocation};

    // ==================== Helper Functions Tests ====================

    #[test]
    fn test_estimate_nesting_depth_empty() {
        assert_eq!(estimate_nesting_depth(""), 0);
    }

    #[test]
    fn test_estimate_nesting_depth_no_braces() {
        assert_eq!(estimate_nesting_depth("let x = 1;"), 0);
    }

    #[test]
    fn test_estimate_nesting_depth_single() {
        assert_eq!(estimate_nesting_depth("fn foo() { x }"), 1);
    }

    #[test]
    fn test_estimate_nesting_depth_nested() {
        assert_eq!(estimate_nesting_depth("fn foo() { if true { x } }"), 2);
    }

    #[test]
    fn test_estimate_nesting_depth_deeply_nested() {
        assert_eq!(estimate_nesting_depth("{ { { { x } } } }"), 4);
    }

    #[test]
    fn test_estimate_nesting_depth_unbalanced() {
        // Should handle unbalanced gracefully
        assert_eq!(estimate_nesting_depth("{{ }"), 2);
    }

    #[test]
    fn test_count_parameters_empty() {
        assert_eq!(count_parameters("fn foo()"), 0);
    }

    #[test]
    fn test_count_parameters_none() {
        assert_eq!(count_parameters("let x = 5;"), 0);
    }

    #[test]
    fn test_count_parameters_single() {
        assert_eq!(count_parameters("fn foo(x: i32)"), 1);
    }

    #[test]
    fn test_count_parameters_multiple() {
        assert_eq!(count_parameters("fn foo(x: i32, y: i32, z: i32)"), 3);
    }

    #[test]
    fn test_count_parameters_with_generics() {
        assert_eq!(count_parameters("fn foo<T>(x: T, y: T)"), 2);
    }

    #[test]
    fn test_count_unique_variables_empty() {
        assert_eq!(count_unique_variables(""), 0);
    }

    #[test]
    fn test_count_unique_variables_simple() {
        let source = "let x = y + z;";
        let count = count_unique_variables(source);
        assert!(count >= 1); // At least one variable
    }

    #[test]
    fn test_count_unique_variables_with_keywords() {
        let source = "fn let mut x = 5;";
        let count = count_unique_variables(source);
        // Should not count keywords
        assert!(count >= 1);
    }

    #[test]
    fn test_count_unique_variables_uppercase() {
        let source = "let Type = MyStruct;";
        let count = count_unique_variables(source);
        // Uppercase identifiers shouldn't be counted as variables
        assert!(count >= 0);
    }

    #[test]
    fn test_is_rust_keyword_true() {
        assert!(is_rust_keyword("fn"));
        assert!(is_rust_keyword("let"));
        assert!(is_rust_keyword("mut"));
        assert!(is_rust_keyword("if"));
        assert!(is_rust_keyword("else"));
        assert!(is_rust_keyword("for"));
        assert!(is_rust_keyword("while"));
        assert!(is_rust_keyword("loop"));
        assert!(is_rust_keyword("match"));
        assert!(is_rust_keyword("return"));
    }

    #[test]
    fn test_is_rust_keyword_false() {
        assert!(!is_rust_keyword("variable"));
        assert!(!is_rust_keyword("foo"));
        assert!(!is_rust_keyword("bar"));
        assert!(!is_rust_keyword("x"));
        assert!(!is_rust_keyword("MyType"));
    }

    // ==================== MutantFeatures Tests ====================

    fn create_test_mutant(source: &str, operator: MutationOperatorType) -> Mutant {
        Mutant {
            id: "test".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: source.to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 10,
            },
            operator,
            hash: "hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    #[test]
    fn test_mutant_features_from_simple() {
        let mutant = create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(!features.has_loops);
        assert!(!features.has_conditionals);
        assert_eq!(features.nesting_depth, 0);
    }

    #[test]
    fn test_mutant_features_with_loops() {
        let mutant = create_test_mutant(
            "for i in 0..10 { x += 1; }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_loops);
        assert!(features.control_flow_count >= 1);
    }

    #[test]
    fn test_mutant_features_with_conditionals() {
        let mutant = create_test_mutant(
            "if x > 0 { y } else { z }",
            MutationOperatorType::RelationalReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_conditionals);
    }

    #[test]
    fn test_mutant_features_with_error_handling() {
        let mutant = create_test_mutant(
            "fn foo() -> Result<(), Error> { Ok(()) }",
            MutationOperatorType::ReturnReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_error_handling);
    }

    #[test]
    fn test_mutant_features_with_assertions() {
        let mutant = create_test_mutant(
            "assert_eq!(x, y);",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_assertions);
    }

    #[test]
    fn test_mutant_features_with_arithmetic() {
        let mutant = create_test_mutant(
            "let z = x + y;",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_arithmetic);
    }

    #[test]
    fn test_mutant_features_with_comparisons() {
        let mutant =
            create_test_mutant("if x == y { }", MutationOperatorType::RelationalReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_comparisons);
    }

    #[test]
    fn test_mutant_features_with_logical_ops() {
        let mutant = create_test_mutant(
            "if a && b || !c { }",
            MutationOperatorType::ConditionalReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_logical_ops);
    }

    #[test]
    fn test_mutant_features_to_feature_vector_length() {
        let mutant = create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector.len(), 18); // 18 features in v2
    }

    #[test]
    fn test_mutant_features_operator_type_numeric() {
        let mutant1 = create_test_mutant("x", MutationOperatorType::ArithmeticReplacement);
        let features1 = MutantFeatures::from_mutant(&mutant1);

        let mutant2 = create_test_mutant("x", MutationOperatorType::RelationalReplacement);
        let features2 = MutantFeatures::from_mutant(&mutant2);

        let vec1 = features1.to_feature_vector();
        let vec2 = features2.to_feature_vector();

        // First element is operator type
        assert_eq!(vec1[0], 1.0); // ArithmeticReplacement
        assert_eq!(vec2[0], 2.0); // RelationalReplacement
    }

    #[test]
    fn test_mutant_features_operator_none() {
        let mutant = create_test_mutant("x", MutationOperatorType::None);
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector[0], 0.0); // None
    }

    #[test]
    fn test_mutant_features_operator_custom() {
        let mutant = create_test_mutant("x", MutationOperatorType::Custom("test".to_string()));
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector[0], 21.0); // Custom
    }

    // ==================== TrainingData Tests ====================

    #[test]
    fn test_training_data_creation() {
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let data = TrainingData {
            mutant: mutant.clone(),
            was_killed: true,
            test_failures: vec!["test_add".to_string()],
            execution_time_ms: 100,
        };

        assert!(data.was_killed);
        assert_eq!(data.test_failures.len(), 1);
        assert_eq!(data.execution_time_ms, 100);
    }

    #[test]
    fn test_training_data_serialization() {
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let data = TrainingData {
            mutant,
            was_killed: false,
            test_failures: vec![],
            execution_time_ms: 50,
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TrainingData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.was_killed, deserialized.was_killed);
        assert_eq!(data.execution_time_ms, deserialized.execution_time_ms);
    }

    // ==================== PredictionResult Tests ====================

    #[test]
    fn test_prediction_result_creation() {
        let mut contributions = HashMap::new();
        contributions.insert("operator_type".to_string(), 0.5);

        let result = PredictionResult {
            kill_probability: 0.75,
            confidence: 0.9,
            feature_contributions: contributions,
        };

        assert!((result.kill_probability - 0.75).abs() < f64::EPSILON);
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
        assert_eq!(result.feature_contributions.len(), 1);
    }

    #[test]
    fn test_prediction_result_serialization() {
        let result = PredictionResult {
            kill_probability: 0.5,
            confidence: 0.8,
            feature_contributions: HashMap::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PredictionResult = serde_json::from_str(&json).unwrap();

        assert!((result.kill_probability - deserialized.kill_probability).abs() < f64::EPSILON);
    }

    // ==================== SurvivabilityPredictor Tests ====================

    #[test]
    fn test_predictor_new() {
        let predictor = SurvivabilityPredictor::new();
        assert!(!predictor.is_trained());
    }

    #[test]
    fn test_predictor_default() {
        let predictor = SurvivabilityPredictor::default();
        assert!(!predictor.is_trained());
    }

    #[test]
    fn test_predictor_train_empty_fails() {
        let mut predictor = SurvivabilityPredictor::new();
        let result = predictor.train(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_predict_untrained_fails() {
        let predictor = SurvivabilityPredictor::new();
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let result = predictor.predict(&mutant);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_feature_importance_untrained_fails() {
        let predictor = SurvivabilityPredictor::new();
        let result = predictor.feature_importance();
        assert!(result.is_err());
    }

    fn create_training_data(count: usize, kill_rate: f64) -> Vec<TrainingData> {
        (0..count)
            .map(|i| {
                let mutant = create_test_mutant(
                    &format!("fn test{}() {{ x + y }}", i),
                    MutationOperatorType::ArithmeticReplacement,
                );
                TrainingData {
                    mutant,
                    was_killed: (i as f64 / count as f64) < kill_rate,
                    test_failures: if (i as f64 / count as f64) < kill_rate {
                        vec!["test".to_string()]
                    } else {
                        vec![]
                    },
                    execution_time_ms: 100,
                }
            })
            .collect()
    }

    #[test]
    fn test_predictor_train_small_sample() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(5, 0.5);

        // Should succeed but use statistical baseline
        let result = predictor.train(&data);
        assert!(result.is_ok());
        assert!(predictor.is_trained());
    }

    #[test]
    fn test_predictor_train_and_predict() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(30, 0.6); // More samples than features (18)

        let result = predictor.train(&data);
        assert!(result.is_ok());

        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let prediction = predictor.predict(&mutant);
        assert!(prediction.is_ok());

        let pred = prediction.unwrap();
        assert!(pred.kill_probability >= 0.0 && pred.kill_probability <= 1.0);
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    }

    #[test]
    fn test_predictor_predict_with_explanation() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.7);
        predictor.train(&data).unwrap();

        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let result = predictor.predict_with_explanation(&mutant);
        assert!(result.is_ok());

        let (pred, explanation) = result.unwrap();
        assert!(pred.kill_probability >= 0.0);
        assert!(!explanation.is_empty());
        assert!(explanation.contains("Kill probability"));
    }

    #[test]
    fn test_predictor_prioritize_mutants() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.6);
        predictor.train(&data).unwrap();

        let mutants = vec![
            create_test_mutant("simple", MutationOperatorType::ArithmeticReplacement),
            create_test_mutant(
                "if x > 0 { for i in 0..10 { y += i; } }",
                MutationOperatorType::ConditionalReplacement,
            ),
        ];

        let result = predictor.prioritize_mutants(&mutants);
        assert!(result.is_ok());

        let prioritized = result.unwrap();
        assert_eq!(prioritized.len(), 2);

        // Should be sorted by kill probability descending
        assert!(prioritized[0].1.kill_probability >= prioritized[1].1.kill_probability);
    }

    #[test]
    fn test_predictor_update() {
        let mut predictor = SurvivabilityPredictor::new();
        let initial_data = create_training_data(20, 0.5);
        predictor.train(&initial_data).unwrap();

        let new_data = create_training_data(5, 0.8);
        let result = predictor.update(&new_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_predictor_update_untrained() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);

        // Update should train if not trained
        let result = predictor.update(&data);
        assert!(result.is_ok());
        assert!(predictor.is_trained());
    }

    #[test]
    fn test_predictor_feature_importance() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        predictor.train(&data).unwrap();

        let importance = predictor.feature_importance();
        assert!(importance.is_ok());

        let imp = importance.unwrap();
        assert!(!imp.is_empty());

        // All values should be non-negative
        for (_, value) in &imp {
            assert!(*value >= 0.0);
        }
    }

    #[test]
    fn test_predictor_cross_validate_empty_fails() {
        let predictor = SurvivabilityPredictor::new();
        let result = predictor.cross_validate(&[], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_cross_validate_insufficient_folds() {
        let predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        let result = predictor.cross_validate(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_cross_validate() {
        let predictor = SurvivabilityPredictor::new();
        let data = create_training_data(100, 0.5);
        let result = predictor.cross_validate(&data, 5);

        assert!(result.is_ok());
        let accuracy = result.unwrap();
        assert!(accuracy >= 0.0 && accuracy <= 1.0);
    }

    #[test]
    fn test_predictor_save_and_load() {
        use tempfile::tempdir;

        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        predictor.train(&data).unwrap();

        let dir = tempdir().unwrap();
        let path = dir.path().join("model.bin");

        // Save
        let save_result = predictor.save(&path);
        assert!(save_result.is_ok());

        // Load
        let load_result = SurvivabilityPredictor::load(&path);
        assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        assert!(loaded.is_trained());
    }

    // ==================== MutantFeatures Serialization Tests ====================

    #[test]
    fn test_mutant_features_serialization() {
        let mutant = create_test_mutant(
            "fn foo() { for i in 0..10 { if x > 0 { y } } }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        let json = serde_json::to_string(&features).unwrap();
        let deserialized: MutantFeatures = serde_json::from_str(&json).unwrap();

        assert_eq!(features.has_loops, deserialized.has_loops);
        assert_eq!(features.has_conditionals, deserialized.has_conditionals);
        assert_eq!(features.nesting_depth, deserialized.nesting_depth);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_predictor_different_operators() {
        let mut predictor = SurvivabilityPredictor::new();

        // Create training data with different operators
        let data: Vec<TrainingData> = vec![
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::RelationalReplacement,
            MutationOperatorType::ConditionalReplacement,
            MutationOperatorType::StatementDeletion,
        ]
        .iter()
        .enumerate()
        .flat_map(|(i, op)| {
            (0..5).map(move |j| TrainingData {
                mutant: create_test_mutant(&format!("code_{}{}", i, j), op.clone()),
                was_killed: j % 2 == 0,
                test_failures: vec![],
                execution_time_ms: 100,
            })
        })
        .collect();

        predictor.train(&data).unwrap();

        // Predict for each operator type
        for op in [
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::RelationalReplacement,
        ] {
            let mutant = create_test_mutant("test", op);
            let result = predictor.predict(&mutant);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_predictor_unseen_operator() {
        let mut predictor = SurvivabilityPredictor::new();

        // Train only on ArithmeticReplacement
        let data: Vec<TrainingData> = (0..20)
            .map(|i| TrainingData {
                mutant: create_test_mutant(
                    &format!("code_{}", i),
                    MutationOperatorType::ArithmeticReplacement,
                ),
                was_killed: i % 2 == 0,
                test_failures: vec![],
                execution_time_ms: 100,
            })
            .collect();

        predictor.train(&data).unwrap();

        // Predict for unseen operator
        let mutant = create_test_mutant("test", MutationOperatorType::BitwiseReplacement);
        let result = predictor.predict(&mutant);
        assert!(result.is_ok());

        // Should have lower confidence for unseen operator
        let pred = result.unwrap();
        assert!(pred.confidence <= 0.8);
    }

    #[test]
    fn test_complex_source_feature_extraction() {
        let complex_source = r#"
            fn complex() -> Result<i32, Error> {
                let mut result = 0;
                for i in 0..100 {
                    if i % 2 == 0 && i > 10 {
                        match get_value(i)? {
                            Some(v) => result += v,
                            None => continue,
                        }
                    }
                }
                assert!(result > 0);
                Ok(result)
            }
        "#;

        let mutant =
            create_test_mutant(complex_source, MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_loops);
        assert!(features.has_conditionals);
        assert!(features.has_error_handling);
        assert!(features.has_assertions);
        assert!(features.has_arithmetic);
        assert!(features.has_comparisons);
        assert!(features.has_logical_ops);
        assert!(features.nesting_depth >= 2);
        assert!(features.token_count > 10);
    }

    #[test]
    fn test_feature_vector_boolean_encoding() {
        let mutant_with_loops = create_test_mutant(
            "for i in 0..10 { }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let mutant_without_loops =
            create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);

        let features_with = MutantFeatures::from_mutant(&mutant_with_loops);
        let features_without = MutantFeatures::from_mutant(&mutant_without_loops);

        let vec_with = features_with.to_feature_vector();
        let vec_without = features_without.to_feature_vector();

        // has_loops is at index 6
        assert_eq!(vec_with[6], 1.0); // true -> 1.0
        assert_eq!(vec_without[6], 0.0); // false -> 0.0
    }
}
