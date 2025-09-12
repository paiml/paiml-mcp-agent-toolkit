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
    pub fn new() -> Self {
        Self {
            pattern_db: Self::initialize_patterns(),
            feedback: FeedbackCollector::new(),
            scorer: ConfidenceScorer::new(),
        }
    }

    /// Suggest fixes for a violation
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
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    /// Score a pattern for a violation
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
}
