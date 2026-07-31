// Deep context quality types - scorecards, hotspots, recommendations, provenance
// Included from mod.rs - shares parent module scope (no `use` imports)

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Quality scorecard.
///
/// `None` means pmat did NOT measure the field — not that it measured zero.
/// Four of these were hardcoded literals (`maintainability_index = 70.0`,
/// `modularity_score = 85.0`, `test_coverage = 65.0`, `technical_debt_hours =
/// 40.0`, each tagged `// Placeholder for now`), so an empty directory and the
/// whole pmat repo reported the same "Overall Health: 85.0%". Because the
/// neighbouring `complexity_score` IS real, the constants read as measurements.
///
/// Anything that cannot be measured in the caller's scope must stay `None` and
/// render as "not measured". See `contracts/pmat-no-fabrication-v1.yaml`,
/// equation `measured_or_absent`.
pub struct QualityScorecard {
    pub overall_health: Option<f64>,
    pub complexity_score: Option<f64>,
    pub maintainability_index: Option<f64>,
    pub modularity_score: Option<f64>,
    pub test_coverage: Option<f64>,
    pub technical_debt_hours: Option<f64>,
}

impl QualityScorecard {
    /// Render a score, or say plainly that it was not measured.
    ///
    /// Every display path must go through this: a `None` silently formatted as
    /// `0` would be a fresh fabrication, and a worse one, because zero looks
    /// like a finding.
    #[must_use]
    pub fn render(value: Option<f64>, unit: &str) -> String {
        value.map_or_else(
            || "not measured".to_string(),
            |v| format!("{v:.1}{unit}"),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Template provenance.
pub struct TemplateProvenance {
    pub scaffold_timestamp: DateTime<Utc>,
    pub templates_used: Vec<String>,
    pub parameters: FxHashMap<String, serde_json::Value>,
    pub drift_analysis: DriftAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Analysis results for drift.
pub struct DriftAnalysis {
    pub added_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub drift_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Summary of defect analysis.
pub struct DefectSummary {
    pub total_defects: usize,
    pub by_severity: FxHashMap<String, usize>,
    pub by_type: FxHashMap<String, usize>,
    pub defect_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Hotspot identified in defect analysis.
pub struct DefectHotspot {
    pub location: FileLocation,
    pub composite_score: f32,
    pub contributing_factors: Vec<DefectFactor>,
    pub refactoring_effort: RefactoringEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// File location.
pub struct FileLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defect factor.
pub enum DefectFactor {
    DeadCode {
        confidence: ConfidenceLevel,
        reason: String,
    },
    TechnicalDebt {
        category: TechnicalDebtCategory,
        severity: TechnicalDebtSeverity,
        age_days: u32,
    },
    Complexity {
        _cyclomatic: u32,
        _cognitive: u32,
        violations: Vec<String>,
    },
    ChurnRisk {
        commits: u32,
        authors: u32,
        defect_correlation: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Refactoring estimate.
pub struct RefactoringEstimate {
    pub estimated_hours: f32,
    pub priority: Priority,
    pub impact: Impact,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Priority level for priority.
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Impact.
pub enum Impact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Prioritized recommendation.
pub struct PrioritizedRecommendation {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: Duration,
    pub impact: Impact,
    pub prerequisites: Vec<String>,
}
