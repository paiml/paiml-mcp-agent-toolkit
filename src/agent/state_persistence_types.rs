/// Persistent state for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Version of the state format
    pub version: String,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,

    /// Currently monitored projects
    pub monitored_projects: HashMap<String, ProjectState>,

    /// Historical quality metrics
    pub quality_history: Vec<QualitySnapshot>,

    /// Agent configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,

    /// Session statistics
    pub statistics: AgentStatistics,
}

/// State of a monitored project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    /// Project identifier
    pub id: String,

    /// Project path
    pub path: PathBuf,

    /// Monitoring start time
    pub started_at: DateTime<Utc>,

    /// Last analysis time
    pub last_analyzed: Option<DateTime<Utc>>,

    /// Current quality metrics
    pub current_metrics: QualityMetrics,

    /// Watch patterns
    pub watch_patterns: Vec<String>,

    /// Custom thresholds
    pub thresholds: QualityThresholds,
}

/// Quality metrics for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// Average complexity
    pub avg_complexity: f64,

    /// Maximum complexity
    pub max_complexity: u32,

    /// SATD count
    pub satd_count: usize,

    /// Dead code percentage
    pub dead_code_percentage: f64,

    /// Quality score (0-100)
    pub quality_score: f64,

    /// Total files analyzed
    pub files_analyzed: usize,

    /// Total violations
    pub total_violations: usize,
}

/// Quality thresholds for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub max_complexity: u32,
    pub satd_tolerance: usize,
    pub dead_code_max_percentage: f64,
    pub min_quality_score: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            max_complexity: 20, // Toyota Way standard
            satd_tolerance: 0,  // Zero tolerance
            dead_code_max_percentage: 10.0,
            min_quality_score: 80.0,
        }
    }
}

/// Historical quality snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,

    /// Project ID
    pub project_id: String,

    /// Metrics at this point in time
    pub metrics: QualityMetrics,

    /// Any violations detected
    pub violations: Vec<String>,
}

/// Agent statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStatistics {
    /// Total monitoring sessions
    pub sessions_count: u64,

    /// Total analyses performed
    pub analyses_performed: u64,

    /// Total violations detected
    pub violations_detected: u64,

    /// Total refactorings suggested
    pub refactorings_suggested: u64,

    /// Agent uptime seconds
    pub total_uptime_seconds: u64,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            monitored_projects: HashMap::new(),
            quality_history: Vec::new(),
            config_overrides: HashMap::new(),
            statistics: AgentStatistics::default(),
        }
    }
}

impl AgentState {
    /// Convert to JSON string
    fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize state")
    }
}
