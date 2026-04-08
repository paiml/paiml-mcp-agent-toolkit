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

struct AcceptedSuggestion {
    pattern_id: String,
    violation_type: ViolationType,
    timestamp: std::time::SystemTime,
    outcome: SuggestionOutcome,
}

/// Rejected suggestion record
#[derive(Debug, Clone)]

struct RejectedSuggestion {
    pattern_id: String,
    violation_type: ViolationType,
    timestamp: std::time::SystemTime,
    reason: String,
}

/// Outcome of applying a suggestion

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
