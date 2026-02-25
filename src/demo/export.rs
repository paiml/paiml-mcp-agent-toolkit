use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::demo::Hotspot;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::dead_code::DeadCodeRankingResult;
// use crate::models::tdg::TDGAnalysis; // Disabled due to compilation errors
use crate::services::complexity::ComplexityReport;
use crate::services::context::FileContext;
use crate::services::deep_context::{
    ContextMetadata, CrossLangReference, DefectSummary, QualityScorecard,
};
use crate::services::satd_detector::SATDAnalysisResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReport {
    pub repository: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: ContextMetadata,

    // Core analysis results - Full structures, not strings
    pub ast_contexts: Vec<FileContext>,
    pub dependency_graph: DependencyGraph,
    pub complexity_analysis: ComplexityAnalysis,
    pub churn_analysis: Option<CodeChurnAnalysis>,

    // Advanced metrics
    pub satd_analysis: Option<SATDAnalysisResult>,
    pub dead_code_results: Option<DeadCodeRankingResult>,
    pub cross_references: Vec<CrossLangReference>,
    pub quality_scorecard: Option<QualityScorecard>,
    pub defect_summary: Option<DefectSummary>,

    // New TDG integration - Disabled due to compilation errors
    // pub tdg_analysis: Option<TDGAnalysis>,

    // Visualizations - these remain as strings
    pub mermaid_graphs: HashMap<String, String>,

    // Summary for backward compatibility
    pub summary: ProjectSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub hotspots: Vec<Hotspot>,
    pub total_files: usize,
    pub average_complexity: f64,
    pub technical_debt_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnAnalysis {
    pub high_churn_files: Vec<ChurnFile>,
    pub analysis_period_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnFile {
    pub path: String,
    pub churn_score: f32,
    pub commit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub analyzed_files: usize,
    pub analysis_time_ms: u64,
}

pub trait Exporter: Send + Sync {
    fn export(&self, report: &ExportReport) -> Result<String>;
    fn file_extension(&self) -> &'static str;
}

pub struct MarkdownExporter;

pub struct JsonExporter {
    pub pretty: bool,
}

pub struct SarifExporter;

pub struct ExportService {
    exporters: std::collections::HashMap<String, Box<dyn Exporter>>,
}

// --- Format implementations (Markdown, JSON, SARIF) ---
include!("export_formats.rs");

// --- ExportService impl and report creation helpers ---
include!("export_service.rs");

// --- Tests ---
include!("export_tests.rs");
