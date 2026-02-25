/// Base parameters shared by ALL analysis commands
/// This ensures consistency across all interfaces
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BaseAnalysisContract {
    /// Path to analyze - ALWAYS named 'path', never '`project_path`' or 'file'
    pub path: PathBuf,

    /// Output format - ALWAYS available, ALWAYS same enum
    pub format: OutputFormat,

    /// Output file path - ALWAYS optional
    pub output: Option<PathBuf>,

    /// Number of top files to show - ALWAYS same name, ALWAYS optional
    pub top_files: Option<usize>,

    /// Include test files - ALWAYS same behavior
    pub include_tests: bool,

    /// Analysis timeout in seconds - ALWAYS available
    pub timeout: u64,
}

impl Default for BaseAnalysisContract {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            format: OutputFormat::default(),
            output: None,
            top_files: Some(10),
            include_tests: false,
            timeout: 60,
        }
    }
}

/// Contract for analyze complexity command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeComplexityContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Maximum cyclomatic complexity threshold
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,

    /// Maximum cognitive complexity threshold
    #[serde(default)]
    pub max_cognitive: Option<u32>,

    /// Maximum Halstead difficulty threshold
    #[serde(default)]
    pub max_halstead: Option<f64>,
}

/// Contract for analyze SATD command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeSatdContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Filter by severity level
    #[serde(default)]
    pub severity: Option<SatdSeverity>,

    /// Show only critical debt items
    #[serde(default)]
    pub critical_only: bool,

    /// Use strict mode (only TODO/FIXME/HACK/BUG)
    #[serde(default)]
    pub strict: bool,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,
}

/// Contract for analyze dead code command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeDeadCodeContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Include unreachable code blocks
    #[serde(default)]
    pub include_unreachable: bool,

    /// Minimum dead lines to report a file
    #[serde(default = "default_min_dead_lines")]
    pub min_dead_lines: usize,

    /// Maximum allowed dead code percentage
    #[serde(default = "default_max_percentage")]
    pub max_percentage: f64,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,
}

fn default_min_dead_lines() -> usize {
    10
}

fn default_max_percentage() -> f64 {
    15.0
}

/// Contract for analyze TDG command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeTdgContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// TDG threshold for filtering results
    #[serde(default = "default_tdg_threshold")]
    pub threshold: f64,

    /// Include TDG component breakdown
    #[serde(default)]
    pub include_components: bool,

    /// Show only critical files (TDG > 2.5)
    #[serde(default)]
    pub critical_only: bool,
}

fn default_tdg_threshold() -> f64 {
    1.5
}

/// Contract for analyze lint hotspot command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeLintHotspotContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Analyze a specific file instead of finding hotspot
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Maximum allowed defect density
    #[serde(default = "default_max_density")]
    pub max_density: f64,

    /// Minimum confidence for automated fixes
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,

    /// Enforce quality standards
    #[serde(default)]
    pub enforce: bool,

    /// Dry run - show what would be fixed
    #[serde(default)]
    pub dry_run: bool,
}

fn default_max_density() -> f64 {
    5.0
}

fn default_min_confidence() -> f64 {
    0.8
}

/// Contract for analyze entropy command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyzeEntropyContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Minimum severity level to report
    #[serde(default)]
    pub min_severity: Option<String>,

    /// Maximum number of violations to show (0 = all)
    #[serde(default)]
    pub top_violations: Option<usize>,

    /// Specific file to analyze instead of project
    #[serde(default)]
    pub file: Option<PathBuf>,
}

/// Contract for quality gate command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QualityGateContract {
    /// Base parameters (inherited)
    #[serde(flatten, default)]
    pub base: BaseAnalysisContract,

    /// Quality profile to use
    #[serde(default)]
    pub profile: QualityProfile,

    /// Specific file to check (optional)
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Exit with error if violations found
    #[serde(default)]
    pub fail_on_violation: bool,

    /// Show verbose output
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum QualityProfile {
    #[default]
    Standard,
    Strict,
    Extreme,
    Toyota,
}

/// Contract for refactor auto command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorAutoContract {
    /// File to refactor - MUST be a file, not directory
    pub file: PathBuf,

    /// Output format
    #[serde(default)]
    pub format: OutputFormat,

    /// Output file path
    #[serde(default)]
    pub output: Option<PathBuf>,

    /// Target complexity
    #[serde(default = "default_target_complexity")]
    pub target_complexity: u32,

    /// Dry run mode
    #[serde(default)]
    pub dry_run: bool,

    /// Analysis timeout
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_target_complexity() -> u32 {
    10
}

fn default_timeout() -> u64 {
    60
}
