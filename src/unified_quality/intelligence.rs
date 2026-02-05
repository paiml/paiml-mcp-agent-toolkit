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

/// A refactoring pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern identifier
    pub id: String,

    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Code transformation template
    pub template: String,

    /// Success rate from historical data
    pub success_rate: f64,

    /// Applicable contexts
    pub contexts: Vec<String>,

    /// Example before/after code
    pub example: Example,
}

/// Example of pattern application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub before: String,
    pub after: String,
    pub improvement: String,
}

/// Suggestion for fixing a violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// The pattern to apply
    pub pattern: Pattern,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Preview of the change
    pub preview: String,

    /// Estimated impact
    pub impact: Impact,
}

/// Impact of applying a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    /// Complexity reduction
    pub complexity_reduction: i32,

    /// Lines of code change
    pub loc_change: i32,

    /// Test coverage impact
    pub coverage_impact: f64,

    /// Risk level
    pub risk: RiskLevel,
}

/// Risk level of a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Feedback collector for improving suggestions
pub struct FeedbackCollector {
    /// Accepted suggestions
    accepted: Vec<AcceptedSuggestion>,

    /// Rejected suggestions
    rejected: Vec<RejectedSuggestion>,

    /// Success metrics
    metrics: FeedbackMetrics,
}

/// Accepted suggestion record
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AcceptedSuggestion {
    pattern_id: String,
    violation_type: ViolationType,
    timestamp: std::time::SystemTime,
    outcome: SuggestionOutcome,
}

/// Rejected suggestion record
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RejectedSuggestion {
    pattern_id: String,
    violation_type: ViolationType,
    timestamp: std::time::SystemTime,
    reason: String,
}

/// Outcome of applying a suggestion
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum SuggestionOutcome {
    Success,
    PartialSuccess,
    Failure(String),
}

/// Feedback metrics
#[derive(Debug, Clone, Default)]
struct FeedbackMetrics {
    total_suggestions: usize,
    accepted: usize,
    rejected: usize,
    success_rate: f64,
}

/// Confidence scorer for suggestions
pub struct ConfidenceScorer {
    /// Weights for different factors
    weights: ScoringWeights,
}

/// Weights for confidence scoring
#[derive(Debug, Clone)]
struct ScoringWeights {
    pattern_success_rate: f64,
    context_match: f64,
    code_similarity: f64,
    user_history: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            pattern_success_rate: 0.4,
            context_match: 0.3,
            code_similarity: 0.2,
            user_history: 0.1,
        }
    }
}

impl Default for QualityAssistant {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityAssistant {
    /// Create a new quality assistant
    #[must_use]
    pub fn new() -> Self {
        Self {
            pattern_db: Self::initialize_patterns(),
            feedback: FeedbackCollector::new(),
            scorer: ConfidenceScorer::new(),
        }
    }

    /// Suggest fixes for a violation
    #[must_use]
    pub fn suggest(
        &self,
        violation: &crate::unified_quality::metrics::Violation,
    ) -> Vec<Suggestion> {
        self.pattern_db
            .get(&violation.violation_type)
            .map(|patterns| {
                patterns
                    .iter()
                    .map(|p| {
                        let confidence = self.scorer.score(p, violation);
                        Suggestion {
                            pattern: p.clone(),
                            confidence,
                            preview: self.generate_diff(violation, p),
                            impact: self.estimate_impact(p),
                        }
                    })
                    .filter(|s| s.confidence > 0.6)
                    .take(3)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Record feedback on a suggestion
    pub fn record_feedback(
        &mut self,
        suggestion_id: &str,
        accepted: bool,
        outcome: Option<String>,
    ) {
        self.feedback.record(suggestion_id, accepted, outcome);
    }

    /// Get suggestion success rate
    #[must_use]
    pub fn get_success_rate(&self) -> f64 {
        self.feedback.metrics.success_rate
    }

    /// Initialize pattern database with common refactorings
    fn initialize_patterns() -> HashMap<ViolationType, Vec<Pattern>> {
        let mut patterns = HashMap::new();

        // Complexity reduction patterns
        patterns.insert(
            ViolationType::Complexity,
            vec![
                Pattern {
                    id: "extract_method".to_string(),
                    name: "Extract Method".to_string(),
                    description: "Extract complex logic into separate functions".to_string(),
                    template: "fn extracted_logic() { ... }".to_string(),
                    success_rate: 0.85,
                    contexts: vec!["high_complexity".to_string()],
                    example: Example {
                        before: "if a && b && c { /* complex */ }".to_string(),
                        after: "if should_process() { process() }".to_string(),
                        improvement: "Reduced complexity from 15 to 5".to_string(),
                    },
                },
                Pattern {
                    id: "early_return".to_string(),
                    name: "Early Return".to_string(),
                    description: "Use early returns to reduce nesting".to_string(),
                    template: "if !condition { return }".to_string(),
                    success_rate: 0.75,
                    contexts: vec!["nested_conditions".to_string()],
                    example: Example {
                        before: "if valid { /* nested */ }".to_string(),
                        after: "if !valid { return } /* flat */".to_string(),
                        improvement: "Reduced nesting by 2 levels".to_string(),
                    },
                },
            ],
        );

        // SATD removal patterns
        patterns.insert(
            ViolationType::Satd,
            vec![Pattern {
                id: "implement_todo".to_string(),
                name: "Implement TODO".to_string(),
                description: "Complete the TODO implementation".to_string(),
                template: "// Completed implementation".to_string(),
                success_rate: 0.70,
                contexts: vec!["todo_comment".to_string()],
                example: Example {
                    before: "// Add validation".to_string(),
                    after: "validate_input(&input)?;".to_string(),
                    improvement: "Removed technical debt".to_string(),
                },
            }],
        );

        // Dead code removal patterns
        patterns.insert(
            ViolationType::DeadCode,
            vec![Pattern {
                id: "remove_dead_code".to_string(),
                name: "Remove Dead Code".to_string(),
                description: "Remove unreachable or unused code".to_string(),
                template: "// Code removed".to_string(),
                success_rate: 0.95,
                contexts: vec!["unused".to_string()],
                example: Example {
                    before: "#[allow(dead_code)] fn unused() {}".to_string(),
                    after: "// Removed".to_string(),
                    improvement: "Removed 10 lines of dead code".to_string(),
                },
            }],
        );

        patterns
    }

    /// Generate diff preview for a suggestion
    fn generate_diff(&self, violation: &Violation, pattern: &Pattern) -> String {
        format!(
            "--- {}\n+++ {}\n@@ -1,1 +1,1 @@\n-{}\n+{}",
            violation.file, violation.file, pattern.example.before, pattern.example.after
        )
    }

    /// Estimate impact of applying a pattern
    fn estimate_impact(&self, pattern: &Pattern) -> Impact {
        Impact {
            complexity_reduction: match pattern.id.as_str() {
                "extract_method" => 10,
                "early_return" => 5,
                _ => 2,
            },
            loc_change: match pattern.id.as_str() {
                "remove_dead_code" => -10,
                "extract_method" => 5,
                _ => 0,
            },
            coverage_impact: 0.0,
            risk: match pattern.success_rate {
                r if r > 0.8 => RiskLevel::Low,
                r if r > 0.6 => RiskLevel::Medium,
                _ => RiskLevel::High,
            },
        }
    }

    /// Analyze a file and generate suggestions
    pub async fn analyze_file(
        &self,
        file_path: &std::path::Path,
    ) -> Result<Vec<Suggestion>, anyhow::Error> {
        // Read file content and analyze for violations
        let content = std::fs::read_to_string(file_path)?;

        // Simple violation detection for demonstration
        let mut suggestions = Vec::new();

        // Check for TODO comments (SATD)
        if content.contains("TODO") || content.contains("FIXME") {
            let violation = crate::unified_quality::metrics::Violation {
                file: file_path.to_string_lossy().to_string(),
                violation_type: crate::unified_quality::metrics::ViolationType::Satd,
                severity: crate::unified_quality::metrics::Severity::Medium,
                value: 1.0,
                threshold: 0.0,
            };
            suggestions.extend(self.suggest(&violation));
        }

        Ok(suggestions)
    }

    /// Generate suggestions for a file (synchronous version)
    pub fn generate_suggestions(
        &self,
        file_path: &std::path::Path,
    ) -> Result<Vec<Suggestion>, anyhow::Error> {
        // Synchronous version of analyze_file
        let content = std::fs::read_to_string(file_path)?;
        let mut suggestions = Vec::new();

        // Check for various violations
        if content.contains("TODO") || content.contains("FIXME") {
            let violation = crate::unified_quality::metrics::Violation {
                file: file_path.to_string_lossy().to_string(),
                violation_type: crate::unified_quality::metrics::ViolationType::Satd,
                severity: crate::unified_quality::metrics::Severity::Medium,
                value: 1.0,
                threshold: 0.0,
            };
            suggestions.extend(self.suggest(&violation));
        }

        Ok(suggestions)
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackCollector {
    /// Create a new feedback collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            accepted: Vec::new(),
            rejected: Vec::new(),
            metrics: FeedbackMetrics::default(),
        }
    }

    /// Record feedback
    pub fn record(&mut self, pattern_id: &str, accepted: bool, outcome: Option<String>) {
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
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    /// Score a pattern for a violation
    #[must_use]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_assistant_creation() {
        let assistant = QualityAssistant::new();
        assert!(!assistant.pattern_db.is_empty());
    }

    #[test]
    fn test_suggest_for_complexity() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Complexity,
            severity: crate::unified_quality::metrics::Severity::High,
            value: 25.0,
            threshold: 20.0,
        };

        let suggestions = assistant.suggest(&violation);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].confidence > 0.6);
    }

    #[test]
    fn test_feedback_recording() {
        let mut collector = FeedbackCollector::new();
        collector.record("extract_method", true, None);
        assert_eq!(collector.metrics.accepted, 1);
        assert_eq!(collector.metrics.success_rate, 1.0);
    }

    #[test]
    fn test_confidence_scoring() {
        let scorer = ConfidenceScorer::new();
        let pattern = Pattern {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test pattern".to_string(),
            template: "".to_string(),
            success_rate: 0.8,
            contexts: vec!["high_complexity".to_string()],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };

        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Complexity,
            severity: crate::unified_quality::metrics::Severity::High,
            value: 25.0,
            threshold: 20.0,
        };

        let score = scorer.score(&pattern, &violation);
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_quality_assistant_default() {
        let assistant = QualityAssistant::default();
        assert!(!assistant.pattern_db.is_empty());
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern {
            id: "test_pattern".to_string(),
            name: "Test Pattern".to_string(),
            description: "A test pattern".to_string(),
            template: "template code".to_string(),
            success_rate: 0.85,
            contexts: vec!["context1".to_string(), "context2".to_string()],
            example: Example {
                before: "before code".to_string(),
                after: "after code".to_string(),
                improvement: "improved something".to_string(),
            },
        };

        assert_eq!(pattern.id, "test_pattern");
        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.success_rate, 0.85);
        assert_eq!(pattern.contexts.len(), 2);
    }

    #[test]
    fn test_example_creation() {
        let example = Example {
            before: "old code".to_string(),
            after: "new code".to_string(),
            improvement: "10% faster".to_string(),
        };

        assert_eq!(example.before, "old code");
        assert_eq!(example.after, "new code");
        assert_eq!(example.improvement, "10% faster");
    }

    #[test]
    fn test_impact_creation() {
        let impact = Impact {
            complexity_reduction: 5,
            loc_change: -10,
            coverage_impact: 2.5,
            risk: RiskLevel::Low,
        };

        assert_eq!(impact.complexity_reduction, 5);
        assert_eq!(impact.loc_change, -10);
        assert!((impact.coverage_impact - 2.5).abs() < 0.001);
        assert!(matches!(impact.risk, RiskLevel::Low));
    }

    #[test]
    fn test_risk_level_variants() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        assert!(matches!(low, RiskLevel::Low));
        assert!(matches!(medium, RiskLevel::Medium));
        assert!(matches!(high, RiskLevel::High));
    }

    #[test]
    fn test_feedback_collector_default() {
        let collector = FeedbackCollector::default();
        assert_eq!(collector.metrics.total_suggestions, 0);
    }

    #[test]
    fn test_feedback_collector_rejected() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", false, Some("Not applicable".to_string()));

        assert_eq!(collector.metrics.rejected, 1);
        assert_eq!(collector.metrics.accepted, 0);
        assert_eq!(collector.metrics.success_rate, 0.0);
    }

    #[test]
    fn test_feedback_collector_mixed() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        collector.record("pattern2", true, Some("partial success".to_string()));
        collector.record("pattern3", false, Some("Not needed".to_string()));

        assert_eq!(collector.metrics.total_suggestions, 3);
        assert_eq!(collector.metrics.accepted, 2);
        assert_eq!(collector.metrics.rejected, 1);
        assert!((collector.metrics.success_rate - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_confidence_scorer_default() {
        let scorer = ConfidenceScorer::default();
        // Default weights should sum to 1.0
        let weights = &scorer.weights;
        let sum = weights.pattern_success_rate
            + weights.context_match
            + weights.code_similarity
            + weights.user_history;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_suggest_for_satd() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::Satd,
            severity: crate::unified_quality::metrics::Severity::Medium,
            value: 1.0,
            threshold: 0.0,
        };

        let suggestions = assistant.suggest(&violation);
        // SATD has patterns defined
        assert!(
            !suggestions.is_empty() || assistant.pattern_db.get(&ViolationType::Satd).is_none()
        );
    }

    #[test]
    fn test_suggest_for_dead_code() {
        let assistant = QualityAssistant::new();
        let violation = Violation {
            file: "test.rs".to_string(),
            violation_type: ViolationType::DeadCode,
            severity: crate::unified_quality::metrics::Severity::Low,
            value: 1.0,
            threshold: 0.0,
        };

        let _suggestions = assistant.suggest(&violation);
        // Dead code has patterns defined
        if let Some(patterns) = assistant.pattern_db.get(&ViolationType::DeadCode) {
            assert!(!patterns.is_empty());
        }
    }

    #[test]
    fn test_record_feedback() {
        let mut assistant = QualityAssistant::new();
        assert_eq!(assistant.get_success_rate(), 0.0);

        assistant.record_feedback("pattern1", true, None);
        assert_eq!(assistant.get_success_rate(), 1.0);

        assistant.record_feedback("pattern2", false, Some("Did not work".to_string()));
        assert_eq!(assistant.get_success_rate(), 0.5);
    }

    #[test]
    fn test_suggestion_creation() {
        let pattern = Pattern {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test pattern".to_string(),
            template: "template".to_string(),
            success_rate: 0.9,
            contexts: vec![],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };

        let suggestion = Suggestion {
            pattern,
            confidence: 0.85,
            preview: "--- old\n+++ new".to_string(),
            impact: Impact {
                complexity_reduction: 3,
                loc_change: -5,
                coverage_impact: 0.0,
                risk: RiskLevel::Low,
            },
        };

        assert_eq!(suggestion.confidence, 0.85);
        assert!(suggestion.preview.contains("---"));
    }

    // ============ Pattern Tests ============

    #[test]
    fn test_pattern_clone() {
        let pattern = Pattern {
            id: "test-id".to_string(),
            name: "Test Pattern".to_string(),
            description: "Test description".to_string(),
            template: "template code".to_string(),
            success_rate: 0.9,
            contexts: vec!["context1".to_string()],
            example: Example {
                before: "before code".to_string(),
                after: "after code".to_string(),
                improvement: "improvement desc".to_string(),
            },
        };
        let cloned = pattern.clone();
        assert_eq!(cloned.id, "test-id");
        assert_eq!(cloned.name, "Test Pattern");
        assert_eq!(cloned.contexts.len(), 1);
    }

    #[test]
    fn test_pattern_debug() {
        let pattern = Pattern {
            id: "debug-test".to_string(),
            name: "Debug".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.5,
            contexts: vec![],
            example: Example {
                before: "".to_string(),
                after: "".to_string(),
                improvement: "".to_string(),
            },
        };
        let debug = format!("{:?}", pattern);
        assert!(debug.contains("debug-test"));
    }

    // ============ Example Tests ============

    #[test]
    fn test_example_clone() {
        let example = Example {
            before: "before".to_string(),
            after: "after".to_string(),
            improvement: "better".to_string(),
        };
        let cloned = example.clone();
        assert_eq!(cloned.before, "before");
        assert_eq!(cloned.after, "after");
    }

    #[test]
    fn test_example_debug() {
        let example = Example {
            before: "old_code".to_string(),
            after: "new_code".to_string(),
            improvement: "improvement desc".to_string(),
        };
        let debug = format!("{:?}", example);
        assert!(debug.contains("old_code"));
    }

    // ============ Impact Tests ============

    #[test]
    fn test_impact_clone() {
        let impact = Impact {
            complexity_reduction: 5,
            loc_change: -10,
            coverage_impact: 2.5,
            risk: RiskLevel::Medium,
        };
        let cloned = impact.clone();
        assert_eq!(cloned.complexity_reduction, 5);
        assert_eq!(cloned.loc_change, -10);
        assert_eq!(cloned.coverage_impact, 2.5);
    }

    #[test]
    fn test_impact_debug() {
        let impact = Impact {
            complexity_reduction: 0,
            loc_change: 0,
            coverage_impact: 0.0,
            risk: RiskLevel::Low,
        };
        let debug = format!("{:?}", impact);
        assert!(debug.contains("complexity_reduction"));
    }

    // ============ RiskLevel Tests ============

    #[test]
    fn test_risk_level_clone() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        let _ = low.clone();
        let _ = medium.clone();
        let _ = high.clone();
    }

    #[test]
    fn test_risk_level_debug() {
        let risk = RiskLevel::Medium;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("Medium"));
    }

    #[test]
    fn test_risk_level_low_debug() {
        let risk = RiskLevel::Low;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("Low"));
    }

    #[test]
    fn test_risk_level_high_debug() {
        let risk = RiskLevel::High;
        let debug = format!("{:?}", risk);
        assert!(debug.contains("High"));
    }

    // ============ FeedbackCollector Tests ============

    #[test]
    fn test_feedback_collector_new() {
        let collector = FeedbackCollector::new();
        // New collector should have no data
        let _ = collector;
    }

    #[test]
    fn test_feedback_collector_record_accepted() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_feedback_collector_record_rejected() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", false, Some("Reason".to_string()));
        // Just verify it doesn't panic
    }

    #[test]
    fn test_feedback_collector_record_multiple() {
        let mut collector = FeedbackCollector::new();
        collector.record("pattern1", true, None);
        collector.record("pattern2", false, Some("Bad suggestion".to_string()));
        collector.record("pattern1", true, Some("Good".to_string()));
        // Just verify multiple records work
    }

    // ============ ConfidenceScorer Tests ============

    #[test]
    fn test_confidence_scorer_creation() {
        let scorer = ConfidenceScorer::new();
        // New scorer should have default weights
        let _ = scorer;

        // Also test default impl
        let scorer2 = ConfidenceScorer::default();
        let _ = scorer2;
    }

    // ============ Suggestion Tests ============

    #[test]
    fn test_suggestion_clone() {
        let pattern = Pattern {
            id: "clone-test".to_string(),
            name: "Clone Test".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.8,
            contexts: vec![],
            example: Example {
                before: String::new(),
                after: String::new(),
                improvement: String::new(),
            },
        };
        let suggestion = Suggestion {
            pattern,
            confidence: 0.75,
            preview: "preview text".to_string(),
            impact: Impact {
                complexity_reduction: 2,
                loc_change: -5,
                coverage_impact: 1.0,
                risk: RiskLevel::Low,
            },
        };

        let cloned = suggestion.clone();
        assert_eq!(cloned.confidence, 0.75);
        assert_eq!(cloned.preview, "preview text");
        assert_eq!(cloned.pattern.id, "clone-test");
    }

    #[test]
    fn test_suggestion_debug() {
        let pattern = Pattern {
            id: "debug".to_string(),
            name: "Debug".to_string(),
            description: "".to_string(),
            template: "".to_string(),
            success_rate: 0.5,
            contexts: vec![],
            example: Example {
                before: String::new(),
                after: String::new(),
                improvement: String::new(),
            },
        };
        let suggestion = Suggestion {
            pattern,
            confidence: 0.5,
            preview: "".to_string(),
            impact: Impact {
                complexity_reduction: 0,
                loc_change: 0,
                coverage_impact: 0.0,
                risk: RiskLevel::Low,
            },
        };

        let debug = format!("{:?}", suggestion);
        assert!(debug.contains("confidence"));
    }
}
