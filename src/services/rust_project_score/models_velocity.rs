// ============================================================================
// ScoreVelocity - Kaizen Continuous Improvement Tracking (NEW in v1.1)
// ============================================================================

/// Kaizen: Continuous improvement tracking
///
/// Tracks score changes over time to encourage incremental progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreVelocity {
    /// Current score
    pub current: f64,

    /// Previous score (from baseline)
    pub previous: f64,

    /// Change in points
    pub delta: f64,

    /// Change as percentage
    pub delta_percent: f64,

    /// Days since baseline
    pub days_elapsed: u64,

    /// Points per day improvement rate
    pub points_per_day: f64,

    /// Most improved category
    pub most_improved: Option<String>,

    /// Projected days to next grade
    pub days_to_next_grade: Option<u64>,
}

impl ScoreVelocity {
    /// Calculate velocity from previous and current scores
    ///
    /// # Arguments
    /// * `previous` - Previous baseline score
    /// * `current` - Current score
    /// * `days` - Days elapsed since baseline
    pub fn calculate(previous: f64, current: f64, days: u64) -> Self {
        let delta = current - previous;
        let delta_percent = if previous == 0.0 {
            0.0
        } else {
            (delta / previous) * 100.0
        };

        let points_per_day = if days == 0 { 0.0 } else { delta / days as f64 };

        Self {
            current,
            previous,
            delta,
            delta_percent,
            days_elapsed: days,
            points_per_day,
            most_improved: None,
            days_to_next_grade: None,
        }
    }
}

// ============================================================================
// Recommendation - Actionable Improvement Suggestions
// ============================================================================

/// Priority level for recommendations
///
/// Ordered from highest to lowest priority for sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Category this recommendation applies to
    pub category: String,

    /// Description of the recommendation
    pub description: String,

    /// Priority level
    pub priority: RecommendationPriority,

    /// Potential points to gain
    pub potential_points: f64,
}

impl Recommendation {
    /// Create a new recommendation
    pub fn new(
        category: String,
        description: String,
        priority: RecommendationPriority,
        potential_points: f64,
    ) -> Self {
        Self {
            category,
            description,
            priority,
            potential_points,
        }
    }
}

// ============================================================================
// ScoreMetadata - Project Information
// ============================================================================

/// Metadata about the scoring analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    /// Timestamp of analysis
    pub timestamp: String,

    /// Project name
    pub project_name: String,

    /// Specification version
    pub version: String,
}

impl ScoreMetadata {
    /// Create new metadata
    pub fn new(project_name: String, version: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_name,
            version,
        }
    }
}

