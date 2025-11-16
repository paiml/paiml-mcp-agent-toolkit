// Intent Classifier: Distinguish hallucination fixes from planned iterations
//
// Specification: Section 2.1 - Multi-Signal Temporal Analysis
// Implements 5-signal classification with <5% false positive rate target

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitIntent {
    HallucinationFix, // Fixing a false claim from previous commit
    PlannedIteration, // Expected follow-up work
    Uncertain,        // Cannot determine with confidence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub message: String,
    pub timestamp_seconds: i64,
    pub modified_files: Vec<String>,
    pub issue_number: Option<u32>,
    pub issue_created_timestamp: Option<i64>,
    pub branch: String,
    pub test_changes: TestChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestChanges {
    pub added_tests: usize,
    pub fixed_tests: usize,
    pub modified_test_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub intent: CommitIntent,
    pub confidence: f64, // 0.0 to 1.0
    pub signals: Vec<SignalResult>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    pub signal_name: String,
    pub vote: CommitIntent,
    pub confidence: f64,
    pub evidence: String,
}

pub struct IntentClassifier {
    hallucination_keywords: Vec<String>,
    iteration_keywords: Vec<String>,
    grace_period_hours: i64,
    code_overlap_threshold: f64,
}

impl IntentClassifier {
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

    fn analyze_commit_message(&self, message: &str) -> SignalResult {
        let message_lower = message.to_lowercase();

        let hallucination_count = self
            .hallucination_keywords
            .iter()
            .filter(|kw| message_lower.contains(kw.as_str()))
            .count();

        let iteration_count = self
            .iteration_keywords
            .iter()
            .filter(|kw| message_lower.contains(kw.as_str()))
            .count();

        if hallucination_count > iteration_count {
            SignalResult {
                signal_name: "commit_message".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.7,
                evidence: format!("{} hallucination keywords detected", hallucination_count),
            }
        } else if iteration_count > hallucination_count {
            SignalResult {
                signal_name: "commit_message".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.7,
                evidence: format!("{} iteration keywords detected", iteration_count),
            }
        } else {
            SignalResult {
                signal_name: "commit_message".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.3,
                evidence: "No clear keyword pattern".to_string(),
            }
        }
    }

    fn analyze_issue_linkage(
        &self,
        original_commit: &CommitInfo,
        followup_commit: &CommitInfo,
    ) -> SignalResult {
        if let (Some(issue_num), Some(issue_created)) = (
            followup_commit.issue_number,
            followup_commit.issue_created_timestamp,
        ) {
            if issue_created > original_commit.timestamp_seconds {
                return SignalResult {
                    signal_name: "issue_linkage".to_string(),
                    vote: CommitIntent::HallucinationFix,
                    confidence: 0.9,
                    evidence: format!("Issue #{} created after original commit", issue_num),
                };
            } else {
                return SignalResult {
                    signal_name: "issue_linkage".to_string(),
                    vote: CommitIntent::PlannedIteration,
                    confidence: 0.8,
                    evidence: format!("Issue #{} existed before original commit", issue_num),
                };
            }
        }

        SignalResult {
            signal_name: "issue_linkage".to_string(),
            vote: CommitIntent::Uncertain,
            confidence: 0.2,
            evidence: "No issue reference".to_string(),
        }
    }

    fn analyze_code_churn(
        &self,
        original_commit: &CommitInfo,
        followup_commit: &CommitInfo,
    ) -> SignalResult {
        let original_files: HashSet<_> = original_commit.modified_files.iter().collect();
        let followup_files: HashSet<_> = followup_commit.modified_files.iter().collect();

        let overlap_count = original_files.intersection(&followup_files).count();
        let total_followup = followup_files.len();

        if total_followup == 0 {
            return SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.1,
                evidence: "No modified files".to_string(),
            };
        }

        let overlap_ratio = overlap_count as f64 / total_followup as f64;

        if overlap_ratio > self.code_overlap_threshold {
            SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: format!(
                    "{:.0}% file overlap suggests fixing same code",
                    overlap_ratio * 100.0
                ),
            }
        } else if overlap_ratio < 0.2 {
            SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.7,
                evidence: format!("{:.0}% overlap suggests new work", overlap_ratio * 100.0),
            }
        } else {
            SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.4,
                evidence: format!("{:.0}% overlap - ambiguous", overlap_ratio * 100.0),
            }
        }
    }

    fn analyze_test_changes(&self, test_changes: &TestChanges) -> SignalResult {
        if test_changes.added_tests > test_changes.fixed_tests {
            SignalResult {
                signal_name: "test_changes".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.7,
                evidence: format!(
                    "{} added vs {} fixed - expanding coverage",
                    test_changes.added_tests, test_changes.fixed_tests
                ),
            }
        } else if test_changes.fixed_tests > test_changes.added_tests {
            SignalResult {
                signal_name: "test_changes".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: format!(
                    "{} fixed vs {} added - fixing broken tests",
                    test_changes.fixed_tests, test_changes.added_tests
                ),
            }
        } else {
            SignalResult {
                signal_name: "test_changes".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.3,
                evidence: "Equal test additions and fixes".to_string(),
            }
        }
    }

    fn analyze_temporal_context(
        &self,
        original_commit: &CommitInfo,
        followup_commit: &CommitInfo,
    ) -> SignalResult {
        let time_diff_hours =
            (followup_commit.timestamp_seconds - original_commit.timestamp_seconds) / 3600;

        if time_diff_hours < self.grace_period_hours {
            SignalResult {
                signal_name: "temporal_context".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.8, // Strong signal for grace period
                evidence: format!(
                    "Within {}-hour grace period ({}h elapsed)",
                    self.grace_period_hours, time_diff_hours
                ),
            }
        } else if original_commit.branch == followup_commit.branch {
            SignalResult {
                signal_name: "temporal_context".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.6, // Moderate signal for same branch
                evidence: format!(
                    "Same branch '{}' suggests related work",
                    original_commit.branch
                ),
            }
        } else {
            SignalResult {
                signal_name: "temporal_context".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.4,
                evidence: format!("{}h after grace period, different branch", time_diff_hours),
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_classifier_compiles() {
        let classifier = IntentClassifier::new();
        assert!(classifier.grace_period_hours == 48);
    }
}
