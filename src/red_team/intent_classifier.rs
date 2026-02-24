// Intent Classifier: Distinguish hallucination fixes from planned iterations
//
// Specification: Section 2.1 - Multi-Signal Temporal Analysis
// Implements 5-signal classification with <5% false positive rate target

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Type definitions: CommitIntent, CommitInfo, TestChanges, IntentClassification,
// SignalResult, IntentClassifier
include!("intent_classifier_types.rs");

// Core implementation: new(), classify(), aggregate_signals(), Default
include!("intent_classifier_core.rs");

// Signal analysis methods: analyze_commit_message, analyze_issue_linkage,
// analyze_code_churn, analyze_test_changes, analyze_temporal_context
include!("intent_classifier_signals.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // Signal analysis tests: constructor, keyword, issue, churn, test_changes, temporal
    include!("intent_classifier_tests_signals.rs");

    // Integration, aggregation, serialization, edge case, and boundary tests
    include!("intent_classifier_tests_integration.rs");
}
