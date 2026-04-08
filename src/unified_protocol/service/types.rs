#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// Data structures for API requests and responses
#[derive(Debug, Deserialize)]
/// Query parameters for list templates.
pub struct ListTemplatesQuery {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
/// Template list.
pub struct TemplateList {
    pub templates: Vec<TemplateInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
/// Information about template.
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Template parameter.
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Deserialize)]
/// Parameters for generate.
pub struct GenerateParams {
    pub template_uri: String,
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
/// Template for generated generation.
pub struct GeneratedTemplate {
    pub template_id: String,
    pub content: String,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Serialize)]
/// Metadata for template.
pub struct TemplateMetadata {
    pub name: String,
    pub version: String,
    pub generated_at: String,
}

#[derive(Debug, Deserialize)]
/// Parameters for complexity.
pub struct ComplexityParams {
    pub project_path: String,
    pub toolchain: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,
    #[serde(default)]
    pub max_cognitive: Option<u32>,
    #[serde(default)]
    pub top_files: Option<usize>,
}

#[derive(Debug, Deserialize)]
/// Parameters for complexity query.
pub struct ComplexityQueryParams {
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub toolchain: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub max_cyclomatic: Option<u32>,
    #[serde(default)]
    pub max_cognitive: Option<u32>,
    #[serde(default)]
    pub top_files: Option<usize>,
}

#[derive(Debug, Serialize)]
/// Analysis results for complexity.
pub struct ComplexityAnalysis {
    pub summary: ComplexitySummary,
    pub files: Vec<FileComplexity>,
}

#[derive(Debug, Serialize)]
/// Summary of complexity analysis.
pub struct ComplexitySummary {
    pub total_functions: usize,
    pub average_complexity: f64,
    pub max_complexity: u32,
    pub files_analyzed: usize,
}

#[derive(Debug, Serialize)]
/// File complexity.
pub struct FileComplexity {
    pub path: String,
    pub functions: Vec<FunctionComplexity>,
}

#[derive(Debug, Serialize)]
/// Function complexity.
pub struct FunctionComplexity {
    pub name: String,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub line_count: u32,
}

#[derive(Debug, Deserialize)]
/// Parameters for churn.
pub struct ChurnParams {
    pub project_path: String,
    #[serde(default)]
    pub period_days: u32,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
/// Analysis results for churn.
pub struct ChurnAnalysis {
    pub summary: ChurnSummary,
    pub hotspots: Vec<ChurnHotspot>,
}

#[derive(Debug, Serialize)]
/// Summary of churn analysis.
pub struct ChurnSummary {
    pub total_commits: usize,
    pub files_changed: usize,
    pub period_days: u32,
}

#[derive(Debug, Serialize)]
/// Hotspot identified in churn analysis.
pub struct ChurnHotspot {
    pub file: String,
    pub changes: u32,
    pub authors: Vec<String>,
}

#[derive(Debug, Deserialize)]
/// Parameters for dag.
pub struct DagParams {
    pub project_path: String,
    #[serde(default)]
    pub dag_type: String,
    #[serde(default)]
    pub show_complexity: bool,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
/// Analysis results for dag.
pub struct DagAnalysis {
    pub graph: String,
    pub nodes: usize,
    pub edges: usize,
    pub cycles: Vec<String>,
}

#[derive(Debug, Deserialize)]
/// Parameters for context.
pub struct ContextParams {
    pub toolchain: String,
    pub project_path: String,
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
/// Context for project operations.
pub struct ProjectContext {
    pub project_name: String,
    pub toolchain: String,
    pub structure: ProjectStructure,
    pub metrics: ContextMetrics,
}

#[derive(Debug, Serialize)]
/// Project structure.
pub struct ProjectStructure {
    pub directories: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
/// Context metrics.
pub struct ContextMetrics {
    pub total_files: usize,
    pub total_lines: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Deserialize)]
/// Parameters for dead code.
pub struct DeadCodeParams {
    pub project_path: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub top_files: Option<usize>,
    #[serde(default)]
    pub include_unreachable: bool,
    #[serde(default)]
    pub min_dead_lines: usize,
    #[serde(default)]
    pub include_tests: bool,
}

#[derive(Debug, Serialize)]
/// Analysis results for dead code.
pub struct DeadCodeAnalysis {
    pub summary: DeadCodeSummary,
    pub files: Vec<FileDeadCode>,
}

#[derive(Debug, Serialize)]
/// Summary of dead code analysis.
pub struct DeadCodeSummary {
    pub total_files_analyzed: usize,
    pub files_with_dead_code: usize,
    pub total_dead_lines: usize,
    pub dead_percentage: f64,
}

#[derive(Debug, Serialize)]
/// File dead code.
pub struct FileDeadCode {
    pub path: String,
    pub dead_lines: usize,
    pub dead_percentage: f64,
    pub dead_functions: usize,
    pub dead_classes: usize,
    pub confidence: String,
}

#[derive(Debug, Deserialize)]
/// Parameters for makefile lint.
pub struct MakefileLintParams {
    pub path: String,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub fix: bool,
    #[serde(default)]
    pub gnu_version: String,
}

#[derive(Debug, Serialize)]
/// Analysis results for makefile lint.
pub struct MakefileLintAnalysis {
    pub path: String,
    pub violations: Vec<MakefileLintViolation>,
    pub quality_score: f32,
    pub rules_applied: Vec<String>,
}

#[derive(Debug, Serialize)]
/// Violation record for makefile lint.
pub struct MakefileLintViolation {
    pub rule: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
/// Parameters for provability.
pub struct ProvabilityParams {
    pub project_path: String,
    #[serde(default)]
    pub functions: Option<Vec<String>>,
    #[serde(default)]
    pub analysis_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
/// Analysis results for provability.
pub struct ProvabilityAnalysis {
    pub project_path: String,
    pub analysis_depth: usize,
    pub functions_analyzed: usize,
    pub average_provability_score: f64,
    pub summaries: Vec<ProvabilitySummary>,
}

#[derive(Debug, Serialize)]
/// Summary of provability analysis.
pub struct ProvabilitySummary {
    pub function_id: String,
    pub provability_score: f64,
    pub verified_properties:
        Vec<crate::services::lightweight_provability_analyzer::VerifiedProperty>,
    pub analysis_time_us: u128,
}

#[derive(Debug, Deserialize)]
/// Parameters for satd.
pub struct SatdParams {
    pub project_path: String,
    #[serde(default)]
    pub strict: Option<bool>,
    #[serde(default)]
    pub exclude_tests: Option<bool>,
    #[serde(default)]
    pub critical_only: Option<bool>,
}

#[derive(Debug, Serialize)]
/// Analysis results for satd.
pub struct SatdAnalysis {
    pub project_path: String,
    pub total_debt_items: usize,
    pub debt_density: f64,
    pub critical_items: usize,
    pub categories: std::collections::HashMap<String, usize>,
    pub files: Vec<SatdFile>,
}

#[derive(Debug, Serialize)]
/// Satd file.
pub struct SatdFile {
    pub path: String,
    pub debt_count: usize,
    pub items: Vec<SatdItem>,
}

#[derive(Debug, Serialize)]
/// Satd item.
pub struct SatdItem {
    pub line: usize,
    pub category: String,
    pub severity: String,
    pub text: String,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
/// Parameters for lint hotspot.
pub struct LintHotspotParams {
    pub project_path: String,
    #[serde(default)]
    pub top_files: Option<usize>,
    #[serde(default)]
    pub min_violations: Option<usize>,
    #[serde(default)]
    pub include: Option<String>,
    #[serde(default)]
    pub exclude: Option<String>,
}

#[derive(Debug, Serialize)]
/// Analysis results for lint hotspot.
pub struct LintHotspotAnalysis {
    pub project_path: String,
    pub total_files_analyzed: usize,
    pub total_violations: usize,
    pub average_violations_per_file: f64,
    pub hotspots: Vec<LintHotspot>,
}

#[derive(Debug, Serialize)]
/// Hotspot identified in lint analysis.
pub struct LintHotspot {
    pub file_path: String,
    pub violations: usize,
    pub lines_of_code: usize,
    pub defect_density: f64,
    pub severity_distribution: std::collections::HashMap<String, usize>,
}
