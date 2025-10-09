//! ML-Based Mutant Survivability Predictor - Phase 4.2 REFACTOR
//!
//! EXTREME TDD: REFACTOR PHASE - Decision Tree with Cross-Validation

use super::{Mutant, MutationOperatorType};
use anyhow::Result;
use linfa::prelude::*;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2};
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
        let has_loops = source.contains("for") || source.contains("while") || source.contains("loop");
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

        let has_logical_ops = source.contains("&&")
            || source.contains("||")
            || source.contains('!');

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
    /// Trained Decision Tree model
    model: Option<DecisionTree<f64, usize>>,

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

    /// Train the predictor on historical data using Decision Tree
    /// Phase 4.2 REFACTOR - Full ML implementation with 18 features
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
            labels.push(if sample.was_killed { 1 } else { 0 });
        }

        // Convert to ndarray
        let features_array = Array2::from_shape_vec((n_samples, n_features), feature_matrix)?;
        let labels_array = Array1::from_vec(labels);

        // Create Linfa dataset
        let dataset = Dataset::new(features_array, labels_array);

        // Train Decision Tree with optimized hyperparameters
        let tree = DecisionTree::params()
            .split_quality(SplitQuality::Gini)  // Gini impurity for classification
            .max_depth(Some(10))  // Prevent overfitting
            .min_weight_split(5.0)  // Require at least 5 samples to split
            .min_weight_leaf(2.0)   // At least 2 samples per leaf
            .fit(&dataset)?;

        self.model = Some(tree);

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
            let current_rate = self.operator_kill_rates
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

            self.operator_kill_rates.insert(sample.mutant.operator.clone(), new_rate);
        }

        Ok(())
    }

    /// Predict kill probability for a mutant using trained Decision Tree
    /// Phase 4.2 REFACTOR - Uses ML model with 18 features
    pub fn predict(&self, mutant: &Mutant) -> Result<PredictionResult> {
        if !self.trained {
            anyhow::bail!("Model not trained");
        }

        let features = MutantFeatures::from_mutant(mutant);
        let feature_vec = features.to_feature_vector();

        // Use trained model if available
        let kill_probability = if let Some(ref model) = self.model {
            // Convert features to ndarray
            let feature_array = Array2::from_shape_vec((1, 18), feature_vec.clone())?;
            let dataset = Dataset::new(feature_array, Array1::from_vec(vec![0])); // Dummy target

            // Predict using the Decision Tree model
            let prediction = model.predict(&dataset);

            // Convert prediction to probability (0 or 1 from classifier -> smooth to probability)
            let pred_value = prediction[0];
            if pred_value == 1 {
                0.85 // High probability if model predicts killed
            } else {
                0.15 // Low probability if model predicts survived
            }
        } else {
            // Fallback to statistical baseline
            let base_probability = self.operator_kill_rates
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
                .unwrap_or(0.5) * 100.0
        );

        Ok((prediction, explanation))
    }

    /// Prioritize mutants by predicted kill probability
    pub fn prioritize_mutants(&self, mutants: &[Mutant]) -> Result<Vec<(Mutant, PredictionResult)>> {
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
    /// NOTE: DecisionTree model is not serialized due to Linfa limitations.
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
// NOTE: DecisionTree model is not serialized - only fallback data and metadata
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
        // model field is skipped (DecisionTree not serializable)
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
            let first_char = cleaned.chars().next().unwrap();
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
        "fn" | "let" | "mut" | "const" | "static" | "if" | "else" | "for"
            | "while" | "loop" | "match" | "return" | "break" | "continue"
            | "pub" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
            | "type" | "where" | "unsafe" | "async" | "await" | "move"
            | "ref" | "in" | "as" | "crate" | "super" | "self" | "Self"
            | "true" | "false"
    )
}
