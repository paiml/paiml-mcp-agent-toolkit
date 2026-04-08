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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    pub(crate) complexity_model: Option<LinearRegression>,
    /// Trained TDG model
    pub(crate) tdg_model: Option<LinearRegression>,
    /// Fallback heuristic weights (when ML unavailable)
    /// Used internally by heuristic_complexity() and heuristic_tdg()
    
    pub(crate) heuristic_weights: HashMap<String, f64>,
    /// Is the model trained?
    pub(crate) trained: bool,
    /// Training sample count
    pub(crate) training_samples: usize,
    /// Feature importance scores
    pub(crate) feature_importance: HashMap<String, f64>,
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
