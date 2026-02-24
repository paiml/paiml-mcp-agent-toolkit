/// Enhanced reporting service
pub struct EnhancedReportingService {
    #[allow(dead_code)]
    renderer: crate::services::renderer::TemplateRenderer,
}

/// Report configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub project_path: PathBuf,
    pub output_format: ReportFormat,
    pub include_visualizations: bool,
    pub include_executive_summary: bool,
    pub include_recommendations: bool,
    pub confidence_threshold: u8,
    pub output_path: Option<PathBuf>,
}

/// Supported report formats
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Html,
    Markdown,
    Json,
    Pdf,
    Dashboard,
}

/// Unified analysis report
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedAnalysisReport {
    pub metadata: ReportMetadata,
    pub executive_summary: ExecutiveSummary,
    pub sections: Vec<ReportSection>,
    pub recommendations: Vec<Recommendation>,
    pub visualizations: Vec<Visualization>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub project_name: String,
    pub project_path: String,
    pub report_date: String,
    pub tool_version: String,
    pub analysis_duration: f64,
    pub analyzed_files: usize,
    pub total_lines: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub overall_health_score: f64,
    pub critical_issues: usize,
    pub high_priority_issues: usize,
    pub key_findings: Vec<String>,
    pub risk_assessment: RiskLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub section_type: SectionType,
    pub content: serde_json::Value,
    pub metrics: HashMap<String, MetricValue>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SectionType {
    Complexity,
    DeadCode,
    Duplication,
    TechnicalDebt,
    Security,
    Performance,
    BigOAnalysis,
    Dependencies,
    TestCoverage,
    CodeSmells,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub unit: String,
    pub trend: Trend,
    pub threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub description: String,
    pub location: Option<Location>,
    pub impact: String,
    pub effort: EffortLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,
    Easy,
    Medium,
    Hard,
    VeryHard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub expected_impact: String,
    pub effort: EffortLevel,
    pub related_findings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Visualization {
    pub title: String,
    pub viz_type: VisualizationType,
    pub data: serde_json::Value,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VisualizationType {
    LineChart,
    BarChart,
    PieChart,
    HeatMap,
    TreeMap,
    NetworkGraph,
    Table,
}

/// Analysis results container
#[derive(Debug)]
pub struct AnalysisResults {
    pub total_duration: std::time::Duration,
    pub analyzed_files: usize,
    pub total_lines: usize,
    pub complexity_analysis: Option<ComplexityAnalysis>,
    pub dead_code_analysis: Option<DeadCodeAnalysis>,
    pub duplication_analysis: Option<DuplicationAnalysis>,
    pub tdg_analysis: Option<TdgAnalysis>,
    pub big_o_analysis: Option<BigOAnalysis>,
}

// Analysis result types (simplified versions)
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub total_cyclomatic: u32,
    pub total_cognitive: u32,
    pub functions: usize,
    pub max_cyclomatic: u32,
    pub high_complexity_functions: usize,
    pub distribution: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeadCodeAnalysis {
    pub dead_lines: usize,
    pub dead_functions: usize,
    pub dead_code_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicationAnalysis {
    pub duplicated_lines: usize,
    pub duplicate_blocks: usize,
    pub duplication_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TdgAnalysis {
    pub average_tdg: f64,
    pub max_tdg: f64,
    pub high_tdg_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BigOAnalysis {
    pub analyzed_functions: usize,
    pub high_complexity_count: usize,
    pub complexity_distribution: HashMap<String, usize>,
}

impl Default for EnhancedReportingService {
    fn default() -> Self {
        Self::new().expect("Failed to create reporting service")
    }
}
