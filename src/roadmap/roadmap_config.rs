// Configuration structs for roadmap management with default implementations.
// Covers RoadmapConfig, QualityGateConfig, GitConfig, and TrackingConfig.

/// Configuration for roadmap management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapConfig {
    pub enabled: bool,
    pub path: PathBuf,
    pub auto_generate_todos: bool,
    pub enforce_quality_gates: bool,
    pub require_task_ids: bool,
    pub task_id_pattern: String,
    pub quality_gates: QualityGateConfig,
    pub git: GitConfig,
    pub tracking: TrackingConfig,
}

impl Default for RoadmapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: PathBuf::from("docs/execution/roadmap.md"),
            auto_generate_todos: true,
            enforce_quality_gates: true,
            require_task_ids: true,
            task_id_pattern: "PMAT-[0-9]{4}".to_string(),
            quality_gates: QualityGateConfig::default(),
            git: GitConfig::default(),
            tracking: TrackingConfig::default(),
        }
    }
}

/// Quality gate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    pub complexity_max: u32,
    pub coverage_min: u8,
    pub documentation_required: bool,
    pub satd_tolerance: u32,
    pub lint_compliance: bool,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            complexity_max: 20,
            coverage_min: 80,
            documentation_required: true,
            satd_tolerance: 0,
            lint_compliance: true,
        }
    }
}

/// Git integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub create_branches: bool,
    pub branch_pattern: String,
    pub commit_pattern: String,
    pub require_quality_check: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            create_branches: false, // DISABLED: per CLAUDE.md zero-branching policy
            branch_pattern: "feature/{task_id}".to_string(),
            commit_pattern: "{task_id}: {message}".to_string(),
            require_quality_check: true,
        }
    }
}

/// Tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub velocity_tracking: bool,
    pub burndown_charts: bool,
    pub quality_metrics: bool,
    pub export_format: String,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            velocity_tracking: true,
            burndown_charts: true,
            quality_metrics: true,
            export_format: "markdown".to_string(),
        }
    }
}
