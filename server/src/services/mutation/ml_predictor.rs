//! ML-Based Mutant Survivability Predictor - Phase 4.2
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::{Mutant, MutationOperatorType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Features extracted from a mutant for ML prediction
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
}

impl MutantFeatures {
    /// Extract features from a mutant
    pub fn from_mutant(mutant: &Mutant) -> Self {
        // Phase 1: Simple heuristic feature extraction
        // Phase 2: Full AST analysis

        let source_line = mutant.location.line;
        let source = &mutant.mutated_source;

        // Simple pattern-based analysis
        let has_loops = source.contains("for") || source.contains("while") || source.contains("loop");
        let has_conditionals = source.contains("if") || source.contains("match");

        let control_flow_count = source.matches("if").count() as u32
            + source.matches("for").count() as u32
            + source.matches("while").count() as u32
            + source.matches("match").count() as u32;

        // Estimate nesting by counting braces
        let nesting_depth = estimate_nesting_depth(source);

        // Estimate complexity
        let cyclomatic_complexity = 1 + control_flow_count;
        let cognitive_complexity = cyclomatic_complexity + nesting_depth;

        let function_size = source.lines().count() as u32;
        let parameter_count = count_parameters(source);

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
        }
    }

    /// Convert features to vector for ML model
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
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
    /// Historical kill rates by operator type
    operator_kill_rates: HashMap<MutationOperatorType, f64>,

    /// Feature importance scores
    feature_importance: HashMap<String, f64>,

    /// Is the model trained?
    trained: bool,

    /// Training data count
    training_samples: usize,
}

impl SurvivabilityPredictor {
    /// Create new predictor
    pub fn new() -> Self {
        Self {
            operator_kill_rates: HashMap::new(),
            feature_importance: HashMap::new(),
            trained: false,
            training_samples: 0,
        }
    }

    /// Train the predictor on historical data
    pub fn train(&mut self, training_data: &[TrainingData]) -> Result<()> {
        if training_data.is_empty() {
            anyhow::bail!("Training data cannot be empty");
        }

        // Phase 1: Simple statistical model
        // Calculate kill rates by operator type
        let mut operator_counts: HashMap<MutationOperatorType, (usize, usize)> = HashMap::new();

        for sample in training_data {
            let entry = operator_counts
                .entry(sample.mutant.operator.clone())
                .or_insert((0, 0));
            entry.0 += 1; // Total count
            if sample.was_killed {
                entry.1 += 1; // Killed count
            }
        }

        // Calculate kill rates
        for (operator, (total, killed)) in operator_counts {
            let kill_rate = killed as f64 / total as f64;
            self.operator_kill_rates.insert(operator, kill_rate);
        }

        // Simple feature importance (operator type is most important for now)
        self.feature_importance.insert("operator_type".to_string(), 0.6);
        self.feature_importance.insert("complexity".to_string(), 0.3);
        self.feature_importance.insert("nesting".to_string(), 0.1);

        self.trained = true;
        self.training_samples = training_data.len();

        Ok(())
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

    /// Predict kill probability for a mutant
    pub fn predict(&self, mutant: &Mutant) -> Result<PredictionResult> {
        if !self.trained {
            anyhow::bail!("Model not trained");
        }

        let features = MutantFeatures::from_mutant(mutant);

        // Phase 1: Use operator-based kill rate
        let base_probability = self.operator_kill_rates
            .get(&mutant.operator)
            .copied()
            .unwrap_or(0.5); // Default for unseen operators

        // Adjust for complexity
        let complexity_factor = 1.0 + (features.cyclomatic_complexity as f64 / 100.0);
        let kill_probability = (base_probability * complexity_factor).min(1.0);

        // Confidence based on training data
        let has_seen_operator = self.operator_kill_rates.contains_key(&mutant.operator);
        let confidence = if has_seen_operator {
            0.8 // High confidence for seen operators
        } else {
            0.5 // Medium confidence for unseen
        };

        let mut feature_contributions = HashMap::new();
        feature_contributions.insert("operator_type".to_string(), base_probability);
        feature_contributions.insert("complexity".to_string(), complexity_factor - 1.0);

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
impl Serialize for SurvivabilityPredictor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SurvivabilityPredictor", 4)?;
        state.serialize_field("operator_kill_rates", &self.operator_kill_rates)?;
        state.serialize_field("feature_importance", &self.feature_importance)?;
        state.serialize_field("trained", &self.trained)?;
        state.serialize_field("training_samples", &self.training_samples)?;
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
            trained: bool,
            training_samples: usize,
        }

        let data = PredictorData::deserialize(deserializer)?;
        Ok(Self {
            operator_kill_rates: data.operator_kill_rates,
            feature_importance: data.feature_importance,
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
