/// Unified defect report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectReport {
    pub id: String,
    pub category: DefectCategory,
    pub severity: Severity,
    pub confidence: f32,
    pub location: CodeLocation,
    pub signals: Vec<SignalEvidence>,
    pub suggested_fixes: Vec<SuggestedFix>,
    pub decision: OracleDecision,
}

impl DefectReport {
    /// Create a new defect report
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(category: DefectCategory, severity: Severity, location: CodeLocation) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            severity,
            confidence: 0.0,
            location,
            signals: Vec::new(),
            suggested_fixes: Vec::new(),
            decision: OracleDecision::Skip,
        }
    }

    /// Add signal evidence
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_signal(&mut self, signal: SignalEvidence) {
        self.signals.push(signal);
        self.recalculate_confidence();
    }

    /// Recalculate confidence based on signals
    ///
    /// Uses multiplicative combination: category_confidence * max_signal_weight
    /// This ensures low-weight signals properly reduce overall confidence.
    fn recalculate_confidence(&mut self) {
        if self.signals.is_empty() {
            self.confidence = 0.0;
            return;
        }

        // Get max signal weight as confidence modifier
        let max_weight = self.signals.iter().map(|s| s.weight).fold(0.0f32, f32::max);

        // Confidence = category base confidence * signal strength
        self.confidence = self.category.rustc_confidence() * max_weight;
    }

    /// Update oracle decision based on thresholds
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn update_decision(&mut self, auto_apply_threshold: f32, review_threshold: f32) {
        self.decision = if self.confidence >= auto_apply_threshold {
            OracleDecision::AutoApply
        } else if self.confidence >= review_threshold {
            OracleDecision::HumanReview
        } else {
            OracleDecision::Skip
        };
    }
}
