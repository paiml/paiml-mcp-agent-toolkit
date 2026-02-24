/// Severity levels for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

/// An actionable violation with fix suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionableViolation {
    pub severity: Severity,
    pub pattern: PatternSummary,
    pub message: String,
    pub fix_suggestion: String,
    pub estimated_loc_reduction: usize,
    pub affected_files: Vec<PathBuf>,
    pub priority_score: f64,
}

/// Summary of a pattern causing violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSummary {
    pub pattern_type: PatternType,
    pub repetitions: usize,
    pub variation_score: f64,
    pub example_code: String,
}
