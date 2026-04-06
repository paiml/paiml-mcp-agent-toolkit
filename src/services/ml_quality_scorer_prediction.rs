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
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        // Serialization deferred until aprender supports it
        Ok(())
    }

    /// Load model from file
    pub fn load(_path: &Path) -> Result<Self> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        // Deserialization deferred until aprender supports it
        Ok(Self::new())
    }
}

impl Default for MLQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}
