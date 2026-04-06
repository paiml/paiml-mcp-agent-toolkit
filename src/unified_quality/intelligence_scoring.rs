impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackCollector {
    /// Create a new feedback collector
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            accepted: Vec::new(),
            rejected: Vec::new(),
            metrics: FeedbackMetrics::default(),
        }
    }

    /// Record feedback
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn record(&mut self, pattern_id: &str, accepted: bool, outcome: Option<String>) {
        debug_assert!(!pattern_id.is_empty(), "pattern_id must not be empty");
        use std::time::SystemTime;

        self.metrics.total_suggestions += 1;

        if accepted {
            self.metrics.accepted += 1;
            self.accepted.push(AcceptedSuggestion {
                pattern_id: pattern_id.to_string(),
                violation_type: ViolationType::Complexity,
                timestamp: SystemTime::now(),
                outcome: outcome.map_or(SuggestionOutcome::Success, |msg| {
                    if msg.contains("partial") {
                        SuggestionOutcome::PartialSuccess
                    } else {
                        SuggestionOutcome::Failure(msg)
                    }
                }),
            });
        } else {
            self.metrics.rejected += 1;
            self.rejected.push(RejectedSuggestion {
                pattern_id: pattern_id.to_string(),
                violation_type: ViolationType::Complexity,
                timestamp: SystemTime::now(),
                reason: outcome.unwrap_or_else(|| "No reason provided".to_string()),
            });
        }

        self.metrics.success_rate =
            self.metrics.accepted as f64 / self.metrics.total_suggestions as f64;
    }
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer {
    /// Create a new confidence scorer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    /// Score a pattern for a violation
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn score(&self, pattern: &Pattern, _violation: &Violation) -> f64 {
        let mut score = 0.0;

        // Pattern success rate component
        score += pattern.success_rate * self.weights.pattern_success_rate;

        // Context match component
        let context_match = if pattern.contexts.contains(&"high_complexity".to_string()) {
            1.0
        } else {
            0.5
        };
        score += context_match * self.weights.context_match;

        // Code similarity component (simplified)
        let similarity = 0.7; // Would use actual similarity metric
        score += similarity * self.weights.code_similarity;

        // User history component
        let user_preference = 0.8; // Would use actual user history
        score += user_preference * self.weights.user_history;

        score.min(1.0)
    }
}
