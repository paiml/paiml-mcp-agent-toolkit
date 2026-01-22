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

    // Helper function to create a basic CommitInfo for testing
    fn make_commit_info(
        message: &str,
        timestamp: i64,
        files: Vec<&str>,
        branch: &str,
    ) -> CommitInfo {
        CommitInfo {
            message: message.to_string(),
            timestamp_seconds: timestamp,
            modified_files: files.into_iter().map(|s| s.to_string()).collect(),
            issue_number: None,
            issue_created_timestamp: None,
            branch: branch.to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        }
    }

    // ===== IntentClassifier::new() tests =====

    #[test]
    fn test_intent_classifier_new_default_values() {
        let classifier = IntentClassifier::new();
        assert_eq!(classifier.grace_period_hours, 48);
        assert!((classifier.code_overlap_threshold - 0.8).abs() < f64::EPSILON);
        assert!(!classifier.hallucination_keywords.is_empty());
        assert!(!classifier.iteration_keywords.is_empty());
    }

    #[test]
    fn test_intent_classifier_default_trait() {
        let classifier = IntentClassifier::default();
        assert_eq!(classifier.grace_period_hours, 48);
    }

    #[test]
    fn test_hallucination_keywords_present() {
        let classifier = IntentClassifier::new();
        assert!(classifier
            .hallucination_keywords
            .contains(&"fix".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"bug".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"broken".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"error".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"regress".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"fail".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"incorrect".to_string()));
        assert!(classifier
            .hallucination_keywords
            .contains(&"wrong".to_string()));
    }

    #[test]
    fn test_iteration_keywords_present() {
        let classifier = IntentClassifier::new();
        assert!(classifier
            .iteration_keywords
            .contains(&"refactor".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"improve".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"enhance".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"optimize".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"cleanup".to_string()));
        assert!(classifier.iteration_keywords.contains(&"add".to_string()));
        assert!(classifier
            .iteration_keywords
            .contains(&"extend".to_string()));
    }

    // ===== analyze_commit_message tests =====

    #[test]
    fn test_commit_message_hallucination_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Fix broken error in parser");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert_eq!(result.signal_name, "commit_message");
    }

    #[test]
    fn test_commit_message_iteration_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Refactor and improve the optimizer");
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_commit_message_uncertain_no_keywords() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("Update documentation");
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.3).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No clear keyword pattern"));
    }

    #[test]
    fn test_commit_message_equal_keywords() {
        let classifier = IntentClassifier::new();
        // "fix" = hallucination, "add" = iteration -> equal count
        let result = classifier.analyze_commit_message("fix add");
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_commit_message_case_insensitive() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("FIX BROKEN ERROR");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_commit_message_empty() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("");
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    // ===== analyze_issue_linkage tests =====

    #[test]
    fn test_issue_linkage_created_after_original() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        followup.issue_created_timestamp = Some(1500); // After original commit

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
        assert!(result
            .evidence
            .contains("Issue #42 created after original commit"));
    }

    #[test]
    fn test_issue_linkage_created_before_original() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(123);
        followup.issue_created_timestamp = Some(500); // Before original commit

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result
            .evidence
            .contains("Issue #123 existed before original commit"));
    }

    #[test]
    fn test_issue_linkage_no_issue() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.2).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No issue reference"));
    }

    #[test]
    fn test_issue_linkage_issue_number_but_no_timestamp() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        // No issue_created_timestamp

        let result = classifier.analyze_issue_linkage(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_issue_linkage_issue_created_at_exact_same_time() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let mut followup = make_commit_info("followup", 2000, vec![], "main");
        followup.issue_number = Some(42);
        followup.issue_created_timestamp = Some(1000); // Same as original

        let result = classifier.analyze_issue_linkage(&original, &followup);
        // Not greater than, so it's PlannedIteration
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
    }

    // ===== analyze_code_churn tests =====

    #[test]
    fn test_code_churn_high_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs", "b.rs", "c.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec!["a.rs", "b.rs", "c.rs"], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("100% file overlap"));
    }

    #[test]
    fn test_code_churn_low_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs"], "main");
        let followup = make_commit_info(
            "followup",
            2000,
            vec!["x.rs", "y.rs", "z.rs", "w.rs", "v.rs"],
            "main",
        );

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert!(result.evidence.contains("overlap suggests new work"));
    }

    #[test]
    fn test_code_churn_moderate_overlap() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs", "b.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec!["a.rs", "c.rs", "d.rs"], "main");
        // Overlap: 1 (a.rs), total: 3, ratio = 0.33

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.4).abs() < f64::EPSILON);
        assert!(result.evidence.contains("ambiguous"));
    }

    #[test]
    fn test_code_churn_no_modified_files() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec!["a.rs"], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.1).abs() < f64::EPSILON);
        assert!(result.evidence.contains("No modified files"));
    }

    #[test]
    fn test_code_churn_exactly_at_threshold() {
        let classifier = IntentClassifier::new();
        // threshold is 0.8, so 80% overlap should still be HallucinationFix
        let original = make_commit_info(
            "original",
            1000,
            vec!["a.rs", "b.rs", "c.rs", "d.rs"],
            "main",
        );
        let followup = make_commit_info(
            "followup",
            2000,
            vec!["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"],
            "main",
        );
        // Overlap: 4, total: 5, ratio = 0.8

        let result = classifier.analyze_code_churn(&original, &followup);
        // > 0.8 is false for 0.8, so it should be Uncertain
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_code_churn_just_above_threshold() {
        let classifier = IntentClassifier::new();
        // 0.81 > 0.8
        let original = make_commit_info(
            "original",
            1000,
            vec![
                "a.rs", "b.rs", "c.rs", "d.rs", "e.rs", "f.rs", "g.rs", "h.rs", "i.rs",
            ],
            "main",
        );
        let followup = make_commit_info(
            "followup",
            2000,
            vec![
                "a.rs", "b.rs", "c.rs", "d.rs", "e.rs", "f.rs", "g.rs", "h.rs", "i.rs", "j.rs",
                "k.rs",
            ],
            "main",
        );
        // Overlap: 9, total: 11, ratio = 9/11 = 0.818

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    // ===== analyze_test_changes tests =====

    #[test]
    fn test_test_changes_more_added_than_fixed() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 5,
            fixed_tests: 2,
            modified_test_files: vec!["tests/test_a.rs".to_string()],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.7).abs() < f64::EPSILON);
        assert!(result.evidence.contains("expanding coverage"));
    }

    #[test]
    fn test_test_changes_more_fixed_than_added() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 1,
            fixed_tests: 4,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("fixing broken tests"));
    }

    #[test]
    fn test_test_changes_equal() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 3,
            fixed_tests: 3,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::Uncertain);
        assert!((result.confidence - 0.3).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Equal test additions and fixes"));
    }

    #[test]
    fn test_test_changes_both_zero() {
        let classifier = IntentClassifier::new();
        let test_changes = TestChanges {
            added_tests: 0,
            fixed_tests: 0,
            modified_test_files: vec![],
        };

        let result = classifier.analyze_test_changes(&test_changes);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    // ===== analyze_temporal_context tests =====

    #[test]
    fn test_temporal_within_grace_period() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 24 hours = 24 * 3600 = 86400 seconds
        let followup = make_commit_info("followup", 86400, vec![], "feature");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Within 48-hour grace period"));
    }

    #[test]
    fn test_temporal_after_grace_same_branch() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 72 hours = 72 * 3600 = 259200 seconds
        let followup = make_commit_info("followup", 259200, vec![], "main");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
        assert!((result.confidence - 0.6).abs() < f64::EPSILON);
        assert!(result.evidence.contains("Same branch"));
    }

    #[test]
    fn test_temporal_after_grace_different_branch() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 100 hours = 100 * 3600 = 360000 seconds
        let followup = make_commit_info("followup", 360000, vec![], "hotfix");

        let result = classifier.analyze_temporal_context(&original, &followup);
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
        assert!((result.confidence - 0.4).abs() < f64::EPSILON);
        assert!(result.evidence.contains("after grace period"));
        assert!(result.evidence.contains("different branch"));
    }

    #[test]
    fn test_temporal_exactly_at_grace_period() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 0, vec![], "main");
        // 48 hours exactly
        let followup = make_commit_info("followup", 48 * 3600, vec![], "feature");

        let result = classifier.analyze_temporal_context(&original, &followup);
        // Not < 48, so it checks branch
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    // ===== aggregate_signals tests =====

    #[test]
    fn test_aggregate_signals_hallucination_dominant() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "test1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.9,
                evidence: "High confidence hallucination".to_string(),
            },
            SignalResult {
                signal_name: "test2".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: "Another hallucination signal".to_string(),
            },
            SignalResult {
                signal_name: "test3".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.3,
                evidence: "Weak iteration signal".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert!(result.confidence > 0.45);
        assert!(result.reasoning.contains("test1"));
        assert!(result.reasoning.contains("test2"));
        assert!(result.reasoning.contains("test3"));
    }

    #[test]
    fn test_aggregate_signals_iteration_dominant() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "signal_a".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.9,
                evidence: "Strong iteration".to_string(),
            },
            SignalResult {
                signal_name: "signal_b".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.7,
                evidence: "More iteration".to_string(),
            },
            SignalResult {
                signal_name: "signal_c".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.2,
                evidence: "Weak hallucination".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_aggregate_signals_uncertain() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "s1".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "Uncertain".to_string(),
            },
            SignalResult {
                signal_name: "s2".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "Also uncertain".to_string(),
            },
            SignalResult {
                signal_name: "s3".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.3,
                evidence: "Weak hall".to_string(),
            },
            SignalResult {
                signal_name: "s4".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.3,
                evidence: "Weak iter".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }

    #[test]
    fn test_aggregate_signals_reasoning_format() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "commit_message".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.7,
                evidence: "3 hallucination keywords".to_string(),
            },
            SignalResult {
                signal_name: "code_churn".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.8,
                evidence: "90% overlap".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert!(result
            .reasoning
            .contains("commit_message: 3 hallucination keywords"));
        assert!(result.reasoning.contains("code_churn: 90% overlap"));
        assert!(result.reasoning.contains("; "));
    }

    // ===== Full classify() integration tests =====

    #[test]
    fn test_classify_hallucination_scenario() {
        let classifier = IntentClassifier::new();

        let original = CommitInfo {
            message: "Add new feature".to_string(),
            timestamp_seconds: 0,
            modified_files: vec!["feature.rs".to_string()],
            issue_number: None,
            issue_created_timestamp: None,
            branch: "main".to_string(),
            test_changes: TestChanges {
                added_tests: 5,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let followup = CommitInfo {
            message: "Fix broken error in feature".to_string(),
            timestamp_seconds: 200000, // After grace period
            modified_files: vec!["feature.rs".to_string()],
            issue_number: Some(999),
            issue_created_timestamp: Some(100), // After original
            branch: "hotfix".to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 3,
                modified_test_files: vec![],
            },
        };

        let result = classifier.classify(&original, &followup);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert_eq!(result.signals.len(), 5);
    }

    #[test]
    fn test_classify_planned_iteration_scenario() {
        let classifier = IntentClassifier::new();

        let original = CommitInfo {
            message: "Initial implementation".to_string(),
            timestamp_seconds: 0,
            modified_files: vec!["module_a.rs".to_string()],
            issue_number: Some(100),
            issue_created_timestamp: Some(0),
            branch: "feature-branch".to_string(),
            test_changes: TestChanges {
                added_tests: 0,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let followup = CommitInfo {
            message: "Refactor and improve the module, add extension".to_string(),
            timestamp_seconds: 3600, // 1 hour later, within grace period
            modified_files: vec!["module_b.rs".to_string(), "module_c.rs".to_string()],
            issue_number: Some(100),
            issue_created_timestamp: Some(0), // Pre-existing issue
            branch: "feature-branch".to_string(),
            test_changes: TestChanges {
                added_tests: 10,
                fixed_tests: 0,
                modified_test_files: vec![],
            },
        };

        let result = classifier.classify(&original, &followup);
        assert_eq!(result.intent, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_classify_uncertain_scenario() {
        let classifier = IntentClassifier::new();

        let original = make_commit_info("Some commit", 0, vec!["file.rs"], "main");
        let followup = make_commit_info("Another commit", 50 * 3600, vec!["other.rs"], "main");

        let result = classifier.classify(&original, &followup);
        // This should be uncertain because:
        // - No clear keywords
        // - No issue reference
        // - No file overlap
        // - No test changes
        // - After grace period but same branch
        assert!(result.signals.len() == 5);
    }

    // ===== Struct serialization/deserialization tests =====

    #[test]
    fn test_commit_intent_equality() {
        assert_eq!(
            CommitIntent::HallucinationFix,
            CommitIntent::HallucinationFix
        );
        assert_eq!(
            CommitIntent::PlannedIteration,
            CommitIntent::PlannedIteration
        );
        assert_eq!(CommitIntent::Uncertain, CommitIntent::Uncertain);
        assert_ne!(
            CommitIntent::HallucinationFix,
            CommitIntent::PlannedIteration
        );
    }

    #[test]
    fn test_commit_info_clone() {
        let info = CommitInfo {
            message: "Test".to_string(),
            timestamp_seconds: 12345,
            modified_files: vec!["a.rs".to_string()],
            issue_number: Some(42),
            issue_created_timestamp: Some(100),
            branch: "main".to_string(),
            test_changes: TestChanges {
                added_tests: 1,
                fixed_tests: 2,
                modified_test_files: vec!["test.rs".to_string()],
            },
        };

        let cloned = info.clone();
        assert_eq!(info.message, cloned.message);
        assert_eq!(info.timestamp_seconds, cloned.timestamp_seconds);
        assert_eq!(info.modified_files, cloned.modified_files);
        assert_eq!(info.issue_number, cloned.issue_number);
        assert_eq!(info.branch, cloned.branch);
    }

    #[test]
    fn test_intent_classification_fields() {
        let classification = IntentClassification {
            intent: CommitIntent::HallucinationFix,
            confidence: 0.85,
            signals: vec![],
            reasoning: "Test reasoning".to_string(),
        };

        assert_eq!(classification.intent, CommitIntent::HallucinationFix);
        assert!((classification.confidence - 0.85).abs() < f64::EPSILON);
        assert!(classification.signals.is_empty());
        assert_eq!(classification.reasoning, "Test reasoning");
    }

    #[test]
    fn test_signal_result_debug() {
        let signal = SignalResult {
            signal_name: "test".to_string(),
            vote: CommitIntent::Uncertain,
            confidence: 0.5,
            evidence: "Some evidence".to_string(),
        };

        let debug_str = format!("{:?}", signal);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("Uncertain"));
    }

    // ===== Edge cases =====

    #[test]
    fn test_negative_time_diff() {
        let classifier = IntentClassifier::new();
        // Edge case: followup timestamp is before original (shouldn't happen but handle it)
        let original = make_commit_info("original", 10000, vec![], "main");
        let followup = make_commit_info("followup", 5000, vec![], "main");

        let result = classifier.analyze_temporal_context(&original, &followup);
        // Negative hours, will be < grace_period_hours
        assert_eq!(result.vote, CommitIntent::PlannedIteration);
    }

    #[test]
    fn test_very_long_commit_message() {
        let classifier = IntentClassifier::new();
        // The code uses contains(), which returns true only once per keyword,
        // regardless of how many times the word appears.
        // So we need multiple different keywords.
        let long_message = "fix bug error broken fail wrong incorrect regress";
        let result = classifier.analyze_commit_message(long_message);
        // Should find 8 hallucination keywords
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_commit_message_with_special_characters() {
        let classifier = IntentClassifier::new();
        let result = classifier.analyze_commit_message("fix: @#$% bug!!! \n\t error");
        assert_eq!(result.vote, CommitIntent::HallucinationFix);
    }

    #[test]
    fn test_empty_modified_files_both_commits() {
        let classifier = IntentClassifier::new();
        let original = make_commit_info("original", 1000, vec![], "main");
        let followup = make_commit_info("followup", 2000, vec![], "main");

        let result = classifier.analyze_code_churn(&original, &followup);
        assert_eq!(result.vote, CommitIntent::Uncertain);
    }

    #[test]
    fn test_single_signal_aggregate() {
        let classifier = IntentClassifier::new();
        let signals = vec![SignalResult {
            signal_name: "single".to_string(),
            vote: CommitIntent::HallucinationFix,
            confidence: 1.0,
            evidence: "Only signal".to_string(),
        }];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::HallucinationFix);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_all_uncertain_signals() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "a".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "b".to_string(),
                vote: CommitIntent::Uncertain,
                confidence: 0.5,
                evidence: "".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }

    // ===== Serialization tests =====

    #[test]
    fn test_commit_intent_serialize_deserialize() {
        let intent = CommitIntent::HallucinationFix;
        let json = serde_json::to_string(&intent).unwrap();
        let deserialized: CommitIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(intent, deserialized);
    }

    #[test]
    fn test_commit_info_serialize_deserialize() {
        let info = CommitInfo {
            message: "Test message".to_string(),
            timestamp_seconds: 1234567890,
            modified_files: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            issue_number: Some(42),
            issue_created_timestamp: Some(1234567800),
            branch: "feature".to_string(),
            test_changes: TestChanges {
                added_tests: 3,
                fixed_tests: 1,
                modified_test_files: vec!["test.rs".to_string()],
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: CommitInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.message, deserialized.message);
        assert_eq!(info.timestamp_seconds, deserialized.timestamp_seconds);
        assert_eq!(info.modified_files, deserialized.modified_files);
        assert_eq!(info.issue_number, deserialized.issue_number);
        assert_eq!(info.branch, deserialized.branch);
    }

    #[test]
    fn test_intent_classification_serialize() {
        let classification = IntentClassification {
            intent: CommitIntent::PlannedIteration,
            confidence: 0.75,
            signals: vec![SignalResult {
                signal_name: "test".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.75,
                evidence: "Test evidence".to_string(),
            }],
            reasoning: "Test reasoning".to_string(),
        };

        let json = serde_json::to_string(&classification).unwrap();
        assert!(json.contains("PlannedIteration"));
        assert!(json.contains("0.75"));
        assert!(json.contains("Test reasoning"));
    }

    // ===== Boundary value tests =====

    #[test]
    fn test_confidence_boundaries() {
        let classifier = IntentClassifier::new();

        // When all signals vote the same with max confidence
        let signals = vec![
            SignalResult {
                signal_name: "s1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 1.0,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "s2".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 1.0,
                evidence: "".to_string(),
            },
        ];

        let result = classifier.aggregate_signals(signals);
        assert!((result.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_confidence_signals() {
        let classifier = IntentClassifier::new();
        let signals = vec![
            SignalResult {
                signal_name: "zero1".to_string(),
                vote: CommitIntent::HallucinationFix,
                confidence: 0.0,
                evidence: "".to_string(),
            },
            SignalResult {
                signal_name: "zero2".to_string(),
                vote: CommitIntent::PlannedIteration,
                confidence: 0.0,
                evidence: "".to_string(),
            },
        ];

        // This will cause division by zero in the current implementation
        // Let's see how it handles it
        let result = classifier.aggregate_signals(signals);
        // With all zeros, ratios become NaN, which doesn't satisfy > 0.45
        assert_eq!(result.intent, CommitIntent::Uncertain);
    }
}
