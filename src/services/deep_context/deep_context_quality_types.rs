// Deep context quality types - scorecards, hotspots, recommendations, provenance
// Included from mod.rs - shares parent module scope (no `use` imports)

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScorecard {
    pub overall_health: f64,
    pub complexity_score: f64,
    pub maintainability_index: f64,
    pub modularity_score: f64,
    pub test_coverage: Option<f64>,
    pub technical_debt_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateProvenance {
    pub scaffold_timestamp: DateTime<Utc>,
    pub templates_used: Vec<String>,
    pub parameters: FxHashMap<String, serde_json::Value>,
    pub drift_analysis: DriftAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    pub added_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub drift_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefectSummary {
    pub total_defects: usize,
    pub by_severity: FxHashMap<String, usize>,
    pub by_type: FxHashMap<String, usize>,
    pub defect_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectHotspot {
    pub location: FileLocation,
    pub composite_score: f32,
    pub contributing_factors: Vec<DefectFactor>,
    pub refactoring_effort: RefactoringEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct RefactoringEstimate {
    pub estimated_hours: f32,
    pub priority: Priority,
    pub impact: Impact,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedRecommendation {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: Duration,
    pub impact: Impact,
    pub prerequisites: Vec<String>,
}
