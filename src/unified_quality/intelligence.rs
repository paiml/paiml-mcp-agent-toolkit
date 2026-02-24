#![cfg_attr(coverage_nightly, coverage(off))]
//! Intelligence Layer: Pattern-Based Suggestion Engine
//!
//! Phase 2 Implementation (Months 4-6)
//! Suggestion engine using successful patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_quality::metrics::{Violation, ViolationType};

/// Suggestion engine using successful patterns
pub struct QualityAssistant {
    /// Curated patterns with success rates
    pattern_db: HashMap<ViolationType, Vec<Pattern>>,

    /// User feedback for continuous improvement
    feedback: FeedbackCollector,

    /// Confidence scoring based on context
    scorer: ConfidenceScorer,
}

// --- Type definitions: Pattern, Example, Suggestion, Impact, RiskLevel,
//     FeedbackCollector, ConfidenceScorer, and supporting types ---
include!("intelligence_types.rs");

// --- QualityAssistant impl (Default + core methods) ---
include!("intelligence_assistant.rs");

// --- FeedbackCollector and ConfidenceScorer impls ---
include!("intelligence_scoring.rs");

// --- Tests ---
include!("intelligence_tests.rs");
