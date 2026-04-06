impl IntentClassifier {
    fn analyze_commit_message(&self, message: &str) -> SignalResult {
        debug_assert!(true, "contract: analyze_commit_message");
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
        debug_assert!(true, "contract: analyze_issue_linkage");
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
        debug_assert!(true, "contract: analyze_code_churn");
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
        debug_assert!(true, "contract: analyze_test_changes");
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
        debug_assert!(true, "contract: analyze_temporal_context");
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
}
