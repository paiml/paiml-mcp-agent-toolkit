/// Errors that can occur during PDMT GitHub integration
#[derive(Error, Debug)]
pub enum PdmtGitHubError {
    #[error("Invalid issue configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Template generation failed: {message}")]
    TemplateGeneration { message: String },

    #[error("PDMT validation failed: {validation_errors:?}")]
    ValidationFailed { validation_errors: Vec<String> },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// GitHub issue types supported by PDMT
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Feature,
    Bug,
    Enhancement,
    Refactor,
    Documentation,
    Testing,
}

impl IssueType {
    /// Get the default labels for this issue type
    pub fn default_labels(&self) -> Vec<String> {
        match self {
            IssueType::Feature => vec!["enhancement".to_string(), "feature".to_string()],
            IssueType::Bug => vec!["bug".to_string()],
            IssueType::Enhancement => vec!["enhancement".to_string()],
            IssueType::Refactor => vec!["refactor".to_string(), "technical-debt".to_string()],
            IssueType::Documentation => vec!["documentation".to_string()],
            IssueType::Testing => vec!["testing".to_string()],
        }
    }

    /// Get the issue type prefix for titles
    pub fn title_prefix(&self) -> &'static str {
        match self {
            IssueType::Feature => "feat:",
            IssueType::Bug => "fix:",
            IssueType::Enhancement => "enhance:",
            IssueType::Refactor => "refactor:",
            IssueType::Documentation => "docs:",
            IssueType::Testing => "test:",
        }
    }
}

/// Priority levels for issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Get the priority label
    pub fn label(&self) -> String {
        match self {
            Priority::Low => "priority:low".to_string(),
            Priority::Medium => "priority:medium".to_string(),
            Priority::High => "priority:high".to_string(),
            Priority::Critical => "priority:critical".to_string(),
        }
    }
}

/// PDMT configuration for issue generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtConfig {
    pub seed: u64,
    pub quality_level: QualityLevel,
    pub granularity: Granularity,
    pub enforce_standards: bool,
}

impl Default for PdmtConfig {
    fn default() -> Self {
        Self {
            seed: 42, // Deterministic seed
            quality_level: QualityLevel::Strict,
            granularity: Granularity::High,
            enforce_standards: true,
        }
    }
}

/// Quality enforcement levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QualityLevel {
    Strict,
    Advisory,
    AutoFix,
}

/// Task breakdown granularity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Granularity {
    Low,
    Medium,
    High,
}

/// Request structure for PDMT issue generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtIssueRequest {
    pub title: String,
    pub description: String,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub complexity_estimate: Option<u8>,
    pub assignees: Vec<String>,
    pub custom_labels: Vec<String>,
    pub config: Option<PdmtConfig>,
}

/// Generated issue template with PDMT metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtIssueTemplate {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub metadata: PdmtMetadata,
}

/// PDMT metadata embedded in issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdmtMetadata {
    pub seed: u64,
    pub quality_level: QualityLevel,
    pub granularity: Granularity,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub complexity_estimate: Option<u8>,
    pub generated_at: String,
    pub validation_commands: Vec<String>,
    pub success_criteria: Vec<String>,
    pub quality_requirements: QualityRequirements,
}

/// Quality requirements for the issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub test_coverage: u8,       // Minimum percentage
    pub max_complexity: u8,      // Maximum cyclomatic complexity
    pub satd_tolerance: u8,      // Zero for strict
    pub documentation_required: bool,
    pub property_tests_required: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            test_coverage: 85,
            max_complexity: 20,
            satd_tolerance: 0, // Zero tolerance
            documentation_required: true,
            property_tests_required: true,
        }
    }
}
