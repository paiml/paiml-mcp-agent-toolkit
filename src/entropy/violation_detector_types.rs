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
    /// The concrete repeated pattern this violation is about, when there is one.
    ///
    /// `None` for project-level findings such as low pattern diversity, which are
    /// not about any particular construct. That case used to carry a placeholder
    /// struct — `repetitions: 0`, `example_code: "Various repetitive patterns"`,
    /// `variation_score` set to literally `1 - diversity` — so one object said
    /// "pattern diversity is LOW (11.9%)" and "variation_score is HIGH (0.88)"
    /// about the same number (#650).
    #[serde(default)]
    pub pattern: Option<PatternSummary>,
    pub message: String,
    pub fix_suggestion: String,
    /// Lines this refactor is estimated to remove, when that can be derived from
    /// measured pattern sizes.
    ///
    /// `None` where it cannot. The low-diversity finding used to report
    /// `total_loc * 0.15`: a fixed 15% of the project regardless of the diversity
    /// it claimed to follow from (358 LOC -> 53, 144 -> 21, 1200 -> 180,
    /// 158020 -> 23703). A constant dressed as an estimate is a fabrication.
    #[serde(default)]
    pub estimated_loc_reduction: Option<usize>,
    pub affected_files: Vec<PathBuf>,
    pub priority_score: f64,
}

impl ActionableViolation {
    /// Render the LOC saving for human output: the measured number, or an
    /// explicit "not estimated" rather than a plausible-looking zero.
    #[must_use]
    pub fn render_loc_reduction(&self) -> String {
        self.estimated_loc_reduction
            .map_or_else(|| "not estimated".to_string(), |n| format!("{n} lines"))
    }
}

/// Summary of a pattern causing violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSummary {
    pub pattern_type: PatternType,
    pub repetitions: usize,
    pub variation_score: f64,
    pub example_code: String,
}
