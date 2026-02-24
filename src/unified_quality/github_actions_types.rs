/// GitHub Actions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Repository owner/name
    pub repository: String,

    /// GitHub token for API access
    pub token: String,

    /// Quality gate thresholds
    pub quality_thresholds: QualityThresholds,

    /// Workflow triggers
    pub triggers: WorkflowTriggers,

    /// Comment settings
    pub comments: CommentConfig,
}

/// Quality thresholds for GitHub Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Maximum allowed complexity increase
    pub max_complexity_increase: u32,

    /// Maximum allowed SATD increase
    pub max_satd_increase: u32,

    /// Minimum coverage requirement
    pub min_coverage: f64,

    /// Block PR if thresholds exceeded
    pub block_on_violation: bool,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_complexity_increase: 50,
            max_satd_increase: 5,
            min_coverage: 0.8,
            block_on_violation: true,
        }
    }
}

/// Workflow triggers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTriggers {
    /// Run on pull request
    pub on_pull_request: bool,

    /// Run on push to main
    pub on_push_main: bool,

    /// Run on schedule
    pub on_schedule: Option<String>,

    /// Specific branches to monitor
    pub branches: Vec<String>,
}

impl Default for WorkflowTriggers {
    fn default() -> Self {
        Self {
            on_pull_request: true,
            on_push_main: true,
            on_schedule: Some("0 6 * * 1".to_string()), // Weekly on Monday 6 AM
            branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

/// Comment configuration for GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentConfig {
    /// Post quality summary as PR comment
    pub post_summary: bool,

    /// Post detailed metrics
    pub post_details: bool,

    /// Update existing comments
    pub update_existing: bool,

    /// Comment template
    pub template: CommentTemplate,
}

impl Default for CommentConfig {
    fn default() -> Self {
        Self {
            post_summary: true,
            post_details: false,
            update_existing: true,
            template: CommentTemplate::default(),
        }
    }
}

/// Comment template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentTemplate {
    /// Header for quality reports
    pub header: String,

    /// Success message template
    pub success_template: String,

    /// Warning message template
    pub warning_template: String,

    /// Failure message template
    pub failure_template: String,
}

impl Default for CommentTemplate {
    fn default() -> Self {
        Self {
            header: "## 📊 Code Quality Report".to_string(),
            success_template: "✅ **Quality checks passed!**\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}".to_string(),
            warning_template: "⚠️ **Quality warnings detected:**\n\n{warnings}\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}".to_string(),
            failure_template: "❌ **Quality checks failed:**\n\n{failures}\n\n- Complexity: {complexity}\n- SATD Count: {satd_count}\n- Coverage: {coverage:.1%}\n\nPlease address these issues before merging.".to_string(),
        }
    }
}

/// GitHub Actions workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Overall status
    pub status: WorkflowStatus,

    /// Quality analysis results
    pub analysis: QualityAnalysis,

    /// Enforcement decision
    pub decision: Decision,

    /// Generated comment (if any)
    pub comment: Option<String>,

    /// Workflow outputs for GitHub Actions
    pub outputs: HashMap<String, String>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Quality checks passed
    Success,

    /// Quality issues found but not blocking
    Warning,

    /// Quality checks failed - blocking merge
    Failure,

    /// Error during execution
    Error(String),
}

/// Quality analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysis {
    /// Files analyzed
    pub files_analyzed: usize,

    /// Total complexity
    pub total_complexity: u32,

    /// Complexity change from base
    pub complexity_change: i32,

    /// SATD count
    pub satd_count: u32,

    /// SATD change from base
    pub satd_change: i32,

    /// Test coverage
    pub coverage: f64,

    /// Coverage change from base
    pub coverage_change: f64,

    /// Quality violations found
    pub violations: Vec<QualityViolation>,
}

/// Quality violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    /// File path
    pub file: String,

    /// Violation type
    pub violation_type: String,

    /// Severity level
    pub severity: ViolationSeverity,

    /// Description
    pub message: String,

    /// Line number (if applicable)
    pub line: Option<u32>,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
