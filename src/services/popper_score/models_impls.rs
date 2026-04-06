// Impl blocks for core scoring types (PopperScore, PopperGrade, PopperCategoryScores,
// PopperCategoryScore, PopperSubScore).
// Included by models.rs - shares parent module scope (no `use` imports here).

// ============================================================================
// PopperScore - Implementation
// ============================================================================

impl PopperScore {
    /// Create a new empty score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            raw_score: 0.0,
            max_available: 100.0,
            normalized_score: 0.0,
            grade: PopperGrade::F,
            gateway_passed: false,
            categories: PopperCategoryScores::default(),
            recommendations: Vec::new(),
            metadata: PopperMetadata::new("unknown".to_string()),
            analysis: PopperAnalysis::default(),
        }
    }

    /// Calculate normalized score with gateway logic
    ///
    /// ## Algorithm (v1.1)
    ///
    /// Phase 1: Gateway Check
    /// ```text
    /// IF Category_A < 15 THEN:
    ///     Total_Score = 0
    ///     Status = "INSUFFICIENT FALSIFIABILITY"
    /// ```
    ///
    /// Phase 2: Normalized Calculation
    /// ```text
    /// Normalized_Score = (Points_Earned / Points_Available) x 100
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn calculate(&mut self) {
        // Check gateway
        let falsifiability_score = self.categories.falsifiability.earned;
        self.gateway_passed = falsifiability_score >= 15.0;

        if !self.gateway_passed {
            self.raw_score = 0.0;
            self.normalized_score = 0.0;
            self.grade = PopperGrade::InsufficientFalsifiability;
            return;
        }

        // Calculate raw score
        self.raw_score = self.categories.total_earned();
        self.max_available = self.categories.total_available();

        // Normalize to 100%
        if self.max_available > 0.0 {
            self.normalized_score = (self.raw_score / self.max_available) * 100.0;
        } else {
            self.normalized_score = 0.0;
        }

        // Assign grade
        self.grade = PopperGrade::from_normalized_score(self.normalized_score);
    }
}

impl Default for PopperScore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PopperGrade - Implementation
// ============================================================================

const GRADE_THRESHOLDS: &[(f64, PopperGrade)] = &[
    (95.0, PopperGrade::APlus),
    (90.0, PopperGrade::A),
    (85.0, PopperGrade::AMinus),
    (80.0, PopperGrade::BPlus),
    (70.0, PopperGrade::B),
    (60.0, PopperGrade::C),
    (50.0, PopperGrade::D),
];

impl PopperGrade {
    /// Calculate grade from normalized score (0-100)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_normalized_score(score: f64) -> Self {
        debug_assert!(score >= 0.0, "score must be non-negative");
        GRADE_THRESHOLDS
            .iter()
            .find(|(threshold, _)| score >= *threshold)
            .map(|(_, grade)| *grade)
            .unwrap_or(PopperGrade::F)
    }

    /// Check if grade meets Popperian scientific standards
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn meets_standards(&self) -> bool {
        matches!(
            self,
            PopperGrade::APlus | PopperGrade::A | PopperGrade::AMinus
        )
    }

    /// Get interpretation text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn interpretation(&self) -> &'static str {
        match self {
            PopperGrade::APlus => "Exemplary Popperian Science",
            PopperGrade::A => "Strong Scientific Standards",
            PopperGrade::AMinus => "Meets Reproducibility Requirements",
            PopperGrade::BPlus => "Good Practices, Minor Gaps",
            PopperGrade::B => "Acceptable, Improvement Needed",
            PopperGrade::C => "Significant Reproducibility Gaps",
            PopperGrade::D => "Major Falsifiability Issues",
            PopperGrade::F => "Insufficient Rigor for Independent Verification",
            PopperGrade::InsufficientFalsifiability => "GATEWAY FAILED - Not Evaluable as Science",
        }
    }
}

impl fmt::Display for PopperGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PopperGrade::APlus => write!(f, "A+"),
            PopperGrade::A => write!(f, "A"),
            PopperGrade::AMinus => write!(f, "A-"),
            PopperGrade::BPlus => write!(f, "B+"),
            PopperGrade::B => write!(f, "B"),
            PopperGrade::C => write!(f, "C"),
            PopperGrade::D => write!(f, "D"),
            PopperGrade::F => write!(f, "F"),
            PopperGrade::InsufficientFalsifiability => write!(f, "GATEWAY FAILED"),
        }
    }
}

// ============================================================================
// PopperCategoryScores - Implementation
// ============================================================================

impl PopperCategoryScores {
    /// Calculate total earned points
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_earned(&self) -> f64 {
        let mut total = self.falsifiability.earned
            + self.reproducibility.earned
            + self.transparency.earned
            + self.statistical_rigor.earned
            + self.historical_integrity.earned;

        // Only add ML if applicable
        if !self.ml_reproducibility.is_not_applicable {
            total += self.ml_reproducibility.earned;
        }

        total
    }

    /// Calculate total available points (may be <100 if ML is N/A)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_available(&self) -> f64 {
        let mut total = self.falsifiability.max
            + self.reproducibility.max
            + self.transparency.max
            + self.statistical_rigor.max
            + self.historical_integrity.max;

        // Only add ML if applicable
        if !self.ml_reproducibility.is_not_applicable {
            total += self.ml_reproducibility.max;
        }

        total
    }
}

impl Default for PopperCategoryScores {
    fn default() -> Self {
        Self {
            falsifiability: PopperCategoryScore::new("Falsifiability & Testability", 0.0, 25.0),
            reproducibility: PopperCategoryScore::new("Reproducibility Infrastructure", 0.0, 25.0),
            transparency: PopperCategoryScore::new("Transparency & Openness", 0.0, 20.0),
            statistical_rigor: PopperCategoryScore::new("Statistical Rigor", 0.0, 15.0),
            historical_integrity: PopperCategoryScore::new("Historical Integrity", 0.0, 10.0),
            ml_reproducibility: PopperCategoryScore::new_na("ML/AI Reproducibility", 5.0),
        }
    }
}

// ============================================================================
// PopperCategoryScore - Implementation
// ============================================================================

impl PopperCategoryScore {
    /// Create a new category score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(name: &str, earned: f64, max: f64) -> Self {
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            name: name.to_string(),
            earned,
            max,
            is_applicable: true,
            is_not_applicable: false,
            sub_scores: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Create a N/A category score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_na(name: &str, max: f64) -> Self {
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            name: name.to_string(),
            earned: 0.0,
            max,
            is_applicable: false,
            is_not_applicable: true,
            sub_scores: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Calculate percentage (0-100)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn percentage(&self) -> f64 {
        if self.is_not_applicable || self.max == 0.0 {
            0.0
        } else {
            (self.earned / self.max) * 100.0
        }
    }

    /// Mark as applicable (for ML projects)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn mark_applicable(&mut self) {
        self.is_applicable = true;
        self.is_not_applicable = false;
    }

    /// Mark as not applicable
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn mark_not_applicable(&mut self) {
        self.is_applicable = false;
        self.is_not_applicable = true;
    }

    /// Add a sub-score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn add_sub_score(&mut self, sub: PopperSubScore) {
        self.earned += sub.earned;
        self.sub_scores.push(sub);
    }

    /// Add a finding
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_finding(&mut self, finding: PopperFinding) {
        self.findings.push(finding);
    }
}

// ============================================================================
// PopperSubScore - Implementation
// ============================================================================

impl PopperSubScore {
    /// Create a new sub-score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(id: &str, name: &str, earned: f64, max: f64, description: &str) -> Self {
        debug_assert!(!id.is_empty(), "id must not be empty");
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            id: id.to_string(),
            name: name.to_string(),
            earned,
            max,
            description: description.to_string(),
        }
    }
}
