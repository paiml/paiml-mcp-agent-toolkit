impl IntentClassifier {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            hallucination_keywords: vec![
                "fix".to_string(),
                "bug".to_string(),
                "broken".to_string(),
                "error".to_string(),
                "regress".to_string(),
                "fail".to_string(),
                "incorrect".to_string(),
                "wrong".to_string(),
            ],
            iteration_keywords: vec![
                "refactor".to_string(),
                "improve".to_string(),
                "enhance".to_string(),
                "optimize".to_string(),
                "cleanup".to_string(),
                "add".to_string(),
                "extend".to_string(),
            ],
            grace_period_hours: 48,
            code_overlap_threshold: 0.8,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Classify.
    pub fn classify(
        &self,
        original_commit: &CommitInfo,
        followup_commit: &CommitInfo,
    ) -> IntentClassification {
        // Multi-signal analysis
        let signals = vec![
            // Signal 1: Commit message language analysis
            self.analyze_commit_message(&followup_commit.message),
            // Signal 2: Issue tracker linkage
            self.analyze_issue_linkage(original_commit, followup_commit),
            // Signal 3: Code churn analysis
            self.analyze_code_churn(original_commit, followup_commit),
            // Signal 4: Test additions vs fixes
            self.analyze_test_changes(&followup_commit.test_changes),
            // Signal 5: Sprint/milestone context (grace period)
            self.analyze_temporal_context(original_commit, followup_commit),
        ];

        // Aggregate signals
        self.aggregate_signals(signals)
    }

    fn aggregate_signals(&self, signals: Vec<SignalResult>) -> IntentClassification {
        let mut hallucination_score = 0.0;
        let mut iteration_score = 0.0;
        let mut uncertain_score = 0.0;

        for signal in &signals {
            match signal.vote {
                CommitIntent::HallucinationFix => hallucination_score += signal.confidence,
                CommitIntent::PlannedIteration => iteration_score += signal.confidence,
                CommitIntent::Uncertain => uncertain_score += signal.confidence,
            }
        }

        let total_score = hallucination_score + iteration_score + uncertain_score;
        let hallucination_ratio = hallucination_score / total_score;
        let iteration_ratio = iteration_score / total_score;

        let (intent, confidence) = if hallucination_ratio > 0.45 {
            (CommitIntent::HallucinationFix, hallucination_ratio)
        } else if iteration_ratio > 0.45 {
            (CommitIntent::PlannedIteration, iteration_ratio)
        } else {
            (
                CommitIntent::Uncertain,
                1.0 - (hallucination_ratio.max(iteration_ratio)),
            )
        };

        let reasoning = signals
            .iter()
            .map(|s| format!("{}: {}", s.signal_name, s.evidence))
            .collect::<Vec<_>>()
            .join("; ");

        IntentClassification {
            intent,
            confidence,
            signals,
            reasoning,
        }
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}
