// Impl blocks for findings, recommendations, analysis, and metadata types.
// Included by models.rs - shares parent module scope (no `use` imports here).

// ============================================================================
// PopperFinding - Implementation
// ============================================================================

impl PopperFinding {
    /// Create a positive finding
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn positive(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Positive,
            message: message.to_string(),
            location: None,
            impact: 0.0,
        }
    }

    /// Create an informational finding
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn info(message: &str) -> Self {
        Self {
            severity: FindingSeverity::Info,
            message: message.to_string(),
            location: None,
            impact: 0.0,
        }
    }

    /// Create a warning finding
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn warning(message: &str, impact: f64) -> Self {
        Self {
            severity: FindingSeverity::Warning,
            message: message.to_string(),
            location: None,
            impact,
        }
    }

    /// Create a critical finding
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn critical(message: &str, impact: f64) -> Self {
        Self {
            severity: FindingSeverity::Critical,
            message: message.to_string(),
            location: None,
            impact,
        }
    }
}

// ============================================================================
// PopperRecommendation - Implementation
// ============================================================================

impl PopperRecommendation {
    /// Create a new recommendation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        category: &str,
        description: &str,
        priority: RecommendationPriority,
        potential_percent: f64,
    ) -> Self {
        debug_assert!(!category.is_empty(), "category must not be empty");
        Self {
            category: category.to_string(),
            description: description.to_string(),
            priority,
            potential_percent,
            command: None,
        }
    }

    /// Add command to recommendation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_command(mut self, cmd: &str) -> Self {
        debug_assert!(!cmd.is_empty(), "cmd must not be empty");
        self.command = Some(cmd.to_string());
        self
    }
}

// ============================================================================
// AnalysisStatus - Implementation
// ============================================================================

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisStatus::Pass => write!(f, "PASS"),
            AnalysisStatus::Partial => write!(f, "PARTIAL"),
            AnalysisStatus::Fail => write!(f, "FAIL"),
        }
    }
}

// ============================================================================
// PopperMetadata - Implementation
// ============================================================================

impl PopperMetadata {
    /// Create new metadata
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(project_name: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_name,
            version: "1.1.0".to_string(),
            project_path: None,
        }
    }

    /// Set project path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn with_path(mut self, path: PathBuf) -> Self {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.project_path = Some(path);
        self
    }
}
