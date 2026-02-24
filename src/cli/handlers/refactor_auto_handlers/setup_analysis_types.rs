// Type definitions for refactor-auto setup and analysis
//
// Contains: configuration types, state, quality profiles, JSON response structs,
// analysis result types, refactoring request types, and GitHub integration types.
// Split from setup_analysis.rs for file health compliance (CB-040).

/// Configuration for refactor auto command
#[derive(Debug, Clone)]
pub struct RefactorAutoConfig {
    pub project_path: PathBuf,
    pub single_file_mode: bool,
    pub file: Option<PathBuf>,
    pub format: RefactorAutoOutputFormat,
    pub max_iterations: u32,
    pub cache_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub ci_mode: bool,
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
    pub ignore_file: Option<PathBuf>,
    pub test_file: Option<PathBuf>,
    pub test_name: Option<String>,
    pub github_issue_url: Option<String>,
    pub bug_report_path: Option<PathBuf>,
}

/// Quality profile configuration for refactor auto
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for quality gate enforcement
struct QualityProfile {
    pub coverage_min: f64,
    pub complexity_max: u16,
    pub complexity_target: u16,
    pub satd_allowed: usize,
}

impl Default for QualityProfile {
    fn default() -> Self {
        // EXTREME quality profile - the highest standards
        Self {
            coverage_min: 80.0,    // Minimum 80% test coverage
            complexity_max: 20,    // Toyota Way standard: maximum cyclomatic complexity of 20
            complexity_target: 10, // Target complexity of 10 for good readability
            satd_allowed: 0,       // Zero self-admitted technical debt
        }
    }
}

// JSON response structs for lint-hotspot and compilation error analysis
#[derive(serde::Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization
struct LintHotspotJsonResponse {
    hotspot: LintHotspotJson,
    all_violations: Vec<ViolationDetailJson>,
    total_project_violations: usize,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization
struct LintHotspotJson {
    file: PathBuf,
    defect_density: f64,
    total_violations: usize,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Used for JSON deserialization
struct ViolationDetailJson {
    file: PathBuf,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
    lint_name: String,
    message: String,
    severity: String,
    suggestion: Option<String>,
    machine_applicable: bool,
}

/// Automated refactor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorState {
    pub iteration: u32,
    pub context_generated: bool,
    pub context_path: PathBuf,
    pub current_file: Option<PathBuf>,
    pub files_completed: Vec<PathBuf>,
    pub quality_metrics: QualityMetrics,
    pub progress: RefactorProgress,
    pub start_time: std::time::SystemTime,
}

// QualityMetrics, RefactorProgress, RefactorPhase moved to refactor_auto_types.rs

/// Refactoring configuration for the extracted functions
#[derive(Debug, Clone)]
struct RefactorConfig {
    project_path: PathBuf,
    mode: RefactorMode,
    quality_profile: QualityProfile,
    patterns: PatternConfig,
    output: OutputConfig,
}

/// Refactoring modes to handle different scenarios
#[derive(Debug, Clone)]
enum RefactorMode {
    ProjectWide,
    SingleFile(PathBuf),
    BugReport(PathBuf),
    GitHubIssue(String),
}

/// Pattern configuration for file discovery and filtering
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for pattern-based file discovery
struct PatternConfig {
    root_path: PathBuf,
    ignore_file: Option<String>,
    patterns: Vec<String>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    ignore_file_path: Option<PathBuf>,
    file_extensions: Vec<String>,
}

/// Output configuration for different formats
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for output configuration
struct OutputConfig {
    format: RefactorAutoOutputFormat,
    dry_run: bool,
    max_iterations: u32,
    verbose: bool,
}

/// Context for refactoring operations
#[derive(Debug)]
struct RefactorContext {
    config: RefactorConfig,
    ignore_patterns: Vec<String>,
    source_files: Vec<PathBuf>,
    start_time: std::time::Instant,
}

/// GitHub issue reference structure
#[derive(Debug, Clone)]
struct GitHubIssueRef {
    owner: String,
    repo: String,
    issue_number: u64,
}

/// GitHub issue content structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GitHubIssueContent {
    title: String,
    body: String,
    number: u64,
}

/// Project quality analysis results
#[derive(Debug)]
#[allow(dead_code)] // Reserved for quality analysis
struct ProjectQualityAnalysis {
    lint_violations: Vec<ViolationDetailJson>,
    complexity_analysis: ComplexityAnalysis,
    satd_analysis: SatdAnalysis,
    coverage_analysis: CoverageAnalysis,
    total_files_analyzed: usize,
    analysis_timestamp: std::time::SystemTime,
}

/// Complexity analysis results
#[derive(Debug)]
#[allow(dead_code)] // Reserved for complexity analysis
struct ComplexityAnalysis {
    high_complexity_violations: Vec<ComplexityViolation>,
    high_complexity_count: usize,
    total_functions: usize,
    average_complexity: f64,
}

/// SATD analysis results
#[derive(Debug)]
#[allow(dead_code)] // Reserved for SATD analysis
struct SatdAnalysis {
    satd_comments: Vec<SatdComment>,
    total_satd_count: usize,
    files_with_satd: usize,
}

/// Coverage analysis results
#[derive(Debug)]
#[allow(dead_code)] // Reserved for coverage analysis
struct CoverageAnalysis {
    overall_coverage_percent: f64,
    files_with_low_coverage: Vec<PathBuf>,
    uncovered_lines: Vec<UncoveredLine>,
}

/// Individual complexity violation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for complexity violations
struct ComplexityViolation {
    file: PathBuf,
    function_name: String,
    complexity: u32,
    line_number: u32,
    suggestion: String,
}

/// Individual SATD comment
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for SATD analysis output
struct SatdComment {
    file: PathBuf,
    line_number: u32,
    comment_text: String,
    satd_type: String, // Type of SATD marker (e.g., requirement, defect, design)
}

/// Uncovered code line
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for coverage analysis output
struct UncoveredLine {
    file: PathBuf,
    line_number: u32,
    content: String,
}

/// Individual refactoring request
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for refactoring request generation
struct RefactoringRequest {
    request_type: RefactoringType,
    target_file: PathBuf,
    priority: RefactoringPriority,
    description: String,
    ai_instructions: String,
    estimated_effort: RefactoringEffort,
}

/// Types of refactoring requests
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by RefactoringRequest
enum RefactoringType {
    ComplexityReduction,
    LintFix,
    SatdCleanup,
    CoverageImprovement,
    SecurityFix,
}

/// Refactoring priority levels
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by RefactoringRequest
enum RefactoringPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Refactoring effort estimation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by RefactoringRequest
enum RefactoringEffort {
    Trivial,   // < 30 minutes
    Minor,     // 30 minutes - 2 hours
    Moderate,  // 2 - 8 hours
    Major,     // 8 - 24 hours
    Extensive, // > 24 hours
}
