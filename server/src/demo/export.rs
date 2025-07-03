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

impl Exporter for MarkdownExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        let mut output = String::with_capacity(1024);

        // Header
        output.push_str(&format!(
            "# Analysis: {}\n\nGenerated: {}\n\n",
            report.repository,
            report.timestamp.format("%Y-%m-%d %H:%M UTC")
        ));

        // Summary section
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Analyzed Files**: {}\n",
            report.summary.analyzed_files
        ));
        output.push_str(&format!(
            "- **Average Complexity**: {:.2}\n",
            report.complexity_analysis.average_complexity
        ));
        output.push_str(&format!(
            "- **Technical Debt**: {} hours\n",
            report.complexity_analysis.technical_debt_hours
        ));
        output.push_str(&format!(
            "- **Analysis Time**: {}ms\n\n",
            report.summary.analysis_time_ms
        ));

        // Dependency Graph
        output.push_str("## Dependency Graph\n\n```mermaid\n");
        if let Some(mermaid) = report.mermaid_graphs.get("main") {
            output.push_str(mermaid);
        } else {
            output.push_str("graph TD\n    NoData[No dependency graph available]");
        }
        output.push_str("\n```\n\n");

        // Complexity Hotspots
        output.push_str("## Complexity Hotspots\n\n");
        output.push_str("| File | Complexity | Cognitive Load |\n");
        output.push_str("|------|------------|----------------|\n");

        for hotspot in &report.complexity_analysis.hotspots {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                hotspot.file,
                hotspot.complexity,
                hotspot.churn_score // Using churn_score as cognitive load
            ));
        }
        output.push('\n');

        // Churn Analysis (if available)
        if let Some(churn) = &report.churn_analysis {
            output.push_str(&format!(
                "## Code Churn Analysis (Last {} Days)\n\n",
                churn.period_days
            ));
            output.push_str("| File | Churn Score | Commits |\n");
            output.push_str("|------|-------------|----------|\n");

            for file in churn.files.iter().take(10) {
                output.push_str(&format!(
                    "| {} | {:.2} | {} |\n",
                    file.relative_path, file.churn_score, file.commit_count
                ));
            }
            output.push('\n');
        }

        // Raw Data Section
        output.push_str("<details>\n<summary>Raw Data</summary>\n\n```json\n");
        output.push_str(&serde_json::to_string_pretty(report)?);
        output.push_str("\n```\n</details>\n");

        Ok(output)
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }
}

pub struct JsonExporter {
    pub pretty: bool,
}

impl JsonExporter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl Exporter for JsonExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(report)?)
        } else {
            Ok(serde_json::to_string(report)?)
        }
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

pub struct SarifExporter;

impl Exporter for SarifExporter {
    fn export(&self, report: &ExportReport) -> Result<String> {
        let sarif = serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "pmat",
                        "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": [{
                            "id": "COMPLEXITY001",
                            "name": "HighComplexity",
                            "shortDescription": {
                                "text": "Function has high cyclomatic complexity"
                            },
                            "fullDescription": {
                                "text": "Functions with high cyclomatic complexity are harder to understand, test, and maintain."
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        }]
                    }
                },
                "results": report.complexity_analysis.hotspots.iter().filter_map(|hotspot| {
                    if hotspot.complexity > 10 {
                        Some(serde_json::json!({
                            "ruleId": "COMPLEXITY001",
                            "ruleIndex": 0,
                            "level": if hotspot.complexity > 20 { "error" } else { "warning" },
                            "message": {
                                "text": format!("Function '{}' has cyclomatic complexity of {}",
                                    hotspot.file, hotspot.complexity)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": hotspot.file.split("::").next().unwrap_or(&hotspot.file)
                                    }
                                }
                            }],
                            "properties": {
                                "complexity": hotspot.complexity,
                                "cognitiveComplexity": hotspot.churn_score
                            }
                        }))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>(),
                "invocations": [{
                    "executionSuccessful": true,
                    "endTimeUtc": report.timestamp.to_rfc3339()
                }]
            }]
        });

        Ok(serde_json::to_string_pretty(&sarif)?)
    }

    fn file_extension(&self) -> &'static str {
        "sarif"
    }
}

pub struct ExportService {
    exporters: std::collections::HashMap<String, Box<dyn Exporter>>,
}

impl ExportService {
    pub fn new() -> Self {
        let mut exporters: std::collections::HashMap<String, Box<dyn Exporter>> =
            std::collections::HashMap::new();

        exporters.insert("markdown".to_string(), Box::new(MarkdownExporter));
        exporters.insert("json".to_string(), Box::new(JsonExporter::new(true)));
        exporters.insert("sarif".to_string(), Box::new(SarifExporter));

        Self { exporters }
    }

    pub fn export(&self, format: &str, report: &ExportReport) -> Result<String> {
        self.exporters
            .get(format)
            .ok_or_else(|| anyhow::anyhow!("Unsupported export format: {}", format))?
            .export(report)
    }

    pub fn save_to_file(&self, format: &str, report: &ExportReport, path: &Path) -> Result<()> {
        let content = self.export(format, report)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn supported_formats(&self) -> Vec<&str> {
        self.exporters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ExportService {
    fn default() -> Self {
        Self::new()
    }
}

// Helper to create ExportReport from analysis results
pub fn create_export_report(
    repo_name: &str,
    dag: &DependencyGraph,
    complexity: Option<&ComplexityReport>,
    churn: Option<&CodeChurnAnalysis>,
    mermaid_diagram: &str,
    analysis_time_ms: u64,
) -> ExportReport {
    create_full_export_report(
        repo_name,
        dag,
        complexity,
        churn,
        mermaid_diagram,
        analysis_time_ms,
        vec![], // empty ast_contexts for backward compatibility
        None,   // no SATD analysis
        None,   // no dead code summary
        vec![], // no cross references
        None,   // no quality scorecard
        None,   // no defect summary
    )
}

// Full helper to create comprehensive ExportReport
#[allow(clippy::too_many_arguments)]
pub fn create_full_export_report(
    repo_name: &str,
    dag: &DependencyGraph,
    complexity: Option<&ComplexityReport>,
    churn: Option<&CodeChurnAnalysis>,
    mermaid_diagram: &str,
    analysis_time_ms: u64,
    ast_contexts: Vec<FileContext>,
    satd_analysis: Option<SATDAnalysisResult>,
    dead_code_results: Option<DeadCodeRankingResult>,
    cross_references: Vec<CrossLangReference>,
    quality_scorecard: Option<QualityScorecard>,
    defect_summary: Option<DefectSummary>,
    // tdg_analysis: Option<TDGAnalysis>, // Disabled due to compilation errors
) -> ExportReport {
    let complexity_analysis = if let Some(c) = complexity {
        let hotspots = c
            .files
            .iter()
            .flat_map(|file| {
                file.functions.iter().map(move |func| Hotspot {
                    file: format!("{}::{}", file.path, func.name),
                    complexity: func.metrics.cyclomatic as u32,
                    churn_score: func.metrics.cognitive as u32,
                })
            })
            .collect();

        ComplexityAnalysis {
            hotspots,
            total_files: c.files.len(),
            average_complexity: c.summary.median_cyclomatic as f64,
            technical_debt_hours: c.summary.technical_debt_hours as u32,
        }
    } else {
        ComplexityAnalysis {
            hotspots: vec![],
            total_files: 0,
            average_complexity: 0.0,
            technical_debt_hours: 0,
        }
    };

    // Create mermaid graphs map
    let mut mermaid_graphs = HashMap::new();
    mermaid_graphs.insert("main".to_string(), mermaid_diagram.to_string());

    // Create default metadata if not provided
    let metadata = ContextMetadata {
        generated_at: Utc::now(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        project_root: std::path::PathBuf::from(repo_name),
        cache_stats: crate::services::deep_context::CacheStats {
            hit_rate: 0.0,
            memory_efficiency: 0.0,
            time_saved_ms: 0,
        },
        analysis_duration: std::time::Duration::from_millis(analysis_time_ms),
    };

    ExportReport {
        repository: repo_name.to_string(),
        timestamp: Utc::now(),
        metadata,
        ast_contexts,
        dependency_graph: dag.clone(),
        complexity_analysis,
        churn_analysis: churn.cloned(),
        satd_analysis,
        dead_code_results,
        cross_references,
        quality_scorecard,
        defect_summary,
        // tdg_analysis, // Disabled due to compilation errors
        mermaid_graphs,
        summary: ProjectSummary {
            total_nodes: dag.nodes.len(),
            total_edges: dag.edges.len(),
            analyzed_files: complexity.map(|c| c.files.len()).unwrap_or(0),
            analysis_time_ms,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_export() {
        let mut mermaid_graphs = HashMap::new();
        mermaid_graphs.insert("main".to_string(), "graph TD\n    A --> B".to_string());

        let report = ExportReport {
            repository: "test-repo".to_string(),
            timestamp: Utc::now(),
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("test-repo"),
                cache_stats: crate::services::deep_context::CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: std::time::Duration::from_millis(1500),
            },
            ast_contexts: vec![],
            dependency_graph: DependencyGraph::new(),
            complexity_analysis: ComplexityAnalysis {
                hotspots: vec![Hotspot {
                    file: "main.rs::main".to_string(),
                    complexity: 15,
                    churn_score: 20,
                }],
                total_files: 10,
                average_complexity: 5.5,
                technical_debt_hours: 8,
            },
            churn_analysis: None,
            satd_analysis: None,
            dead_code_results: None,
            cross_references: vec![],
            quality_scorecard: None,
            defect_summary: None,
            mermaid_graphs,
            summary: ProjectSummary {
                total_nodes: 5,
                total_edges: 4,
                analyzed_files: 10,
                analysis_time_ms: 1500,
            },
        };

        let exporter = MarkdownExporter;
        let result = exporter.export(&report).unwrap();

        assert!(result.contains("# Analysis: test-repo"));
        assert!(result.contains("```mermaid"));
        assert!(result.contains("graph TD"));
        assert!(result.contains("main.rs::main"));
        assert!(result.contains("| 15 |"));
    }

    #[test]
    fn test_json_export() {
        let mut mermaid_graphs = HashMap::new();
        mermaid_graphs.insert("main".to_string(), "graph TD".to_string());

        let report = ExportReport {
            repository: "test-repo".to_string(),
            timestamp: Utc::now(),
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("test-repo"),
                cache_stats: crate::services::deep_context::CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: std::time::Duration::from_millis(100),
            },
            ast_contexts: vec![],
            dependency_graph: DependencyGraph::new(),
            complexity_analysis: ComplexityAnalysis {
                hotspots: vec![],
                total_files: 0,
                average_complexity: 0.0,
                technical_debt_hours: 0,
            },
            churn_analysis: None,
            satd_analysis: None,
            dead_code_results: None,
            cross_references: vec![],
            quality_scorecard: None,
            defect_summary: None,
            mermaid_graphs,
            summary: ProjectSummary {
                total_nodes: 0,
                total_edges: 0,
                analyzed_files: 0,
                analysis_time_ms: 100,
            },
        };

        let exporter = JsonExporter::new(false);
        let result = exporter.export(&report).unwrap();

        assert!(result.contains("\"repository\":\"test-repo\""));
        assert!(!result.contains('\n')); // Not pretty printed
    }

    #[test]
    fn test_export_service() {
        let service = ExportService::new();
        let formats = service.supported_formats();

        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"sarif"));
    }
}
