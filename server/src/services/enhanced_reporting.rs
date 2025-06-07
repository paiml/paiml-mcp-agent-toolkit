//! Enhanced Reporting System - Phase 6 Day 14-15
//!
//! Provides a unified reporting framework that consolidates multiple analysis
//! outputs into comprehensive, multi-format reports with visualizations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

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

impl EnhancedReportingService {
    /// Create new enhanced reporting service
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: crate::services::renderer::TemplateRenderer::new()?,
        })
    }

    /// Generate unified analysis report
    pub async fn generate_report(
        &self,
        config: ReportConfig,
        analysis_results: AnalysisResults,
    ) -> Result<UnifiedAnalysisReport> {
        info!("📊 Generating enhanced analysis report");
        info!("📂 Project: {}", config.project_path.display());
        info!("📄 Format: {:?}", config.output_format);

        let start_time = std::time::Instant::now();

        // Build metadata
        let metadata = self.build_metadata(&config, &analysis_results)?;

        // Generate executive summary
        let executive_summary = self.generate_executive_summary(&analysis_results)?;

        // Build report sections
        let sections = self.build_sections(&analysis_results, &config)?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&analysis_results, &sections)?;

        // Create visualizations if requested
        let visualizations = if config.include_visualizations {
            self.create_visualizations(&analysis_results, &sections)?
        } else {
            Vec::new()
        };

        let report = UnifiedAnalysisReport {
            metadata,
            executive_summary,
            sections,
            recommendations,
            visualizations,
        };

        let duration = start_time.elapsed();
        info!("✅ Report generated in {:?}", duration);

        Ok(report)
    }

    /// Build report metadata
    fn build_metadata(
        &self,
        config: &ReportConfig,
        results: &AnalysisResults,
    ) -> Result<ReportMetadata> {
        Ok(ReportMetadata {
            project_name: config
                .project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            project_path: config.project_path.display().to_string(),
            report_date: chrono::Utc::now().to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            analysis_duration: results.total_duration.as_secs_f64(),
            analyzed_files: results.analyzed_files,
            total_lines: results.total_lines,
        })
    }

    /// Generate executive summary
    fn generate_executive_summary(&self, results: &AnalysisResults) -> Result<ExecutiveSummary> {
        let overall_health_score = self.calculate_health_score(results);
        let critical_issues = self.count_issues_by_severity(results, Severity::Critical);
        let high_priority_issues = self.count_issues_by_severity(results, Severity::High);

        let key_findings = self.extract_key_findings(results);
        let risk_assessment = self.assess_overall_risk(results);

        Ok(ExecutiveSummary {
            overall_health_score,
            critical_issues,
            high_priority_issues,
            key_findings,
            risk_assessment,
        })
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(&self, results: &AnalysisResults) -> f64 {
        let mut score = 100.0;

        // Deduct points for various issues
        if let Some(complexity) = &results.complexity_analysis {
            let avg_complexity = complexity.total_cyclomatic as f64 / complexity.functions as f64;
            if avg_complexity > 10.0 {
                score -= (avg_complexity - 10.0).min(20.0);
            }
        }

        if let Some(dead_code) = &results.dead_code_analysis {
            let dead_code_ratio = dead_code.dead_lines as f64 / results.total_lines as f64;
            score -= (dead_code_ratio * 100.0).min(15.0);
        }

        if let Some(duplication) = &results.duplication_analysis {
            let duplication_ratio =
                duplication.duplicated_lines as f64 / results.total_lines as f64;
            score -= (duplication_ratio * 100.0).min(15.0);
        }

        if let Some(tdg) = &results.tdg_analysis {
            if tdg.average_tdg > 3.0 {
                score -= ((tdg.average_tdg - 3.0) * 5.0).min(20.0);
            }
        }

        score.max(0.0)
    }

    /// Count issues by severity
    fn count_issues_by_severity(&self, _results: &AnalysisResults, _severity: Severity) -> usize {
        // Count from various analyses
        // This is a simplified version - in real implementation, each analysis
        // would contribute its issues

        0
    }

    /// Extract key findings
    fn extract_key_findings(&self, results: &AnalysisResults) -> Vec<String> {
        let mut findings = Vec::new();

        if let Some(complexity) = &results.complexity_analysis {
            if complexity.max_cyclomatic > 20 {
                findings.push(format!(
                    "Found {} functions with high complexity (CC > 20)",
                    complexity.high_complexity_functions
                ));
            }
        }

        if let Some(dead_code) = &results.dead_code_analysis {
            if dead_code.dead_functions > 10 {
                findings.push(format!(
                    "Detected {} unused functions that could be removed",
                    dead_code.dead_functions
                ));
            }
        }

        if let Some(duplication) = &results.duplication_analysis {
            if duplication.duplicate_blocks > 20 {
                findings.push(format!(
                    "Found {} duplicate code blocks affecting maintainability",
                    duplication.duplicate_blocks
                ));
            }
        }

        findings
    }

    /// Assess overall risk level
    fn assess_overall_risk(&self, results: &AnalysisResults) -> RiskLevel {
        let health_score = self.calculate_health_score(results);

        match health_score {
            s if s >= 80.0 => RiskLevel::Low,
            s if s >= 60.0 => RiskLevel::Medium,
            s if s >= 40.0 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Build report sections
    fn build_sections(
        &self,
        results: &AnalysisResults,
        _config: &ReportConfig,
    ) -> Result<Vec<ReportSection>> {
        let mut sections = Vec::new();

        // Complexity section
        if let Some(complexity) = &results.complexity_analysis {
            sections.push(self.build_complexity_section(complexity)?);
        }

        // Dead code section
        if let Some(dead_code) = &results.dead_code_analysis {
            sections.push(self.build_dead_code_section(dead_code)?);
        }

        // Duplication section
        if let Some(duplication) = &results.duplication_analysis {
            sections.push(self.build_duplication_section(duplication)?);
        }

        // Technical debt section
        if let Some(tdg) = &results.tdg_analysis {
            sections.push(self.build_tdg_section(tdg)?);
        }

        // Big-O analysis section
        if let Some(big_o) = &results.big_o_analysis {
            sections.push(self.build_big_o_section(big_o)?);
        }

        Ok(sections)
    }

    /// Build complexity analysis section
    fn build_complexity_section(&self, complexity: &ComplexityAnalysis) -> Result<ReportSection> {
        let mut metrics = HashMap::new();

        metrics.insert(
            "total_cyclomatic".to_string(),
            MetricValue {
                value: complexity.total_cyclomatic as f64,
                unit: "CC".to_string(),
                trend: Trend::Unknown,
                threshold: Some(100.0),
            },
        );

        metrics.insert(
            "average_cyclomatic".to_string(),
            MetricValue {
                value: complexity.total_cyclomatic as f64 / complexity.functions as f64,
                unit: "CC/function".to_string(),
                trend: Trend::Unknown,
                threshold: Some(10.0),
            },
        );

        let findings = vec![Finding {
            severity: if complexity.max_cyclomatic > 20 {
                Severity::High
            } else {
                Severity::Medium
            },
            category: "Complexity".to_string(),
            description: format!(
                "Maximum cyclomatic complexity of {} detected",
                complexity.max_cyclomatic
            ),
            location: None,
            impact: "High complexity increases maintenance cost and bug risk".to_string(),
            effort: EffortLevel::Medium,
        }];

        Ok(ReportSection {
            title: "Code Complexity Analysis".to_string(),
            section_type: SectionType::Complexity,
            content: serde_json::to_value(complexity)?,
            metrics,
            findings,
        })
    }

    /// Build dead code section
    fn build_dead_code_section(&self, dead_code: &DeadCodeAnalysis) -> Result<ReportSection> {
        let mut metrics = HashMap::new();

        metrics.insert(
            "dead_code_ratio".to_string(),
            MetricValue {
                value: dead_code.dead_code_percentage,
                unit: "%".to_string(),
                trend: Trend::Unknown,
                threshold: Some(5.0),
            },
        );

        Ok(ReportSection {
            title: "Dead Code Analysis".to_string(),
            section_type: SectionType::DeadCode,
            content: serde_json::to_value(dead_code)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build duplication section
    fn build_duplication_section(
        &self,
        duplication: &DuplicationAnalysis,
    ) -> Result<ReportSection> {
        let mut metrics = HashMap::new();

        metrics.insert(
            "duplication_ratio".to_string(),
            MetricValue {
                value: duplication.duplication_percentage,
                unit: "%".to_string(),
                trend: Trend::Unknown,
                threshold: Some(3.0),
            },
        );

        Ok(ReportSection {
            title: "Code Duplication Analysis".to_string(),
            section_type: SectionType::Duplication,
            content: serde_json::to_value(duplication)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build TDG section
    fn build_tdg_section(&self, tdg: &TdgAnalysis) -> Result<ReportSection> {
        let mut metrics = HashMap::new();

        metrics.insert(
            "average_tdg".to_string(),
            MetricValue {
                value: tdg.average_tdg,
                unit: "gradient".to_string(),
                trend: Trend::Unknown,
                threshold: Some(3.0),
            },
        );

        Ok(ReportSection {
            title: "Technical Debt Gradient".to_string(),
            section_type: SectionType::TechnicalDebt,
            content: serde_json::to_value(tdg)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build Big-O analysis section
    fn build_big_o_section(&self, big_o: &BigOAnalysis) -> Result<ReportSection> {
        let mut metrics = HashMap::new();

        metrics.insert(
            "high_complexity_functions".to_string(),
            MetricValue {
                value: big_o.high_complexity_count as f64,
                unit: "functions".to_string(),
                trend: Trend::Unknown,
                threshold: Some(0.0),
            },
        );

        Ok(ReportSection {
            title: "Algorithmic Complexity Analysis".to_string(),
            section_type: SectionType::BigOAnalysis,
            content: serde_json::to_value(big_o)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Generate recommendations
    fn generate_recommendations(
        &self,
        results: &AnalysisResults,
        _sections: &[ReportSection],
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if let Some(complexity) = &results.complexity_analysis {
            if complexity.max_cyclomatic > 20 {
                recommendations.push(Recommendation {
                    priority: Priority::High,
                    category: "Complexity".to_string(),
                    title: "Refactor high-complexity functions".to_string(),
                    description: "Functions with cyclomatic complexity > 20 should be decomposed into smaller, more manageable units".to_string(),
                    expected_impact: "Improved maintainability and reduced bug risk".to_string(),
                    effort: EffortLevel::Medium,
                    related_findings: vec!["high_complexity".to_string()],
                });
            }
        }

        // Dead code recommendations
        if let Some(dead_code) = &results.dead_code_analysis {
            if dead_code.dead_functions > 10 {
                recommendations.push(Recommendation {
                    priority: Priority::Medium,
                    category: "Dead Code".to_string(),
                    title: "Remove unused code".to_string(),
                    description: format!(
                        "Remove {} unused functions to improve codebase clarity",
                        dead_code.dead_functions
                    ),
                    expected_impact: "Reduced maintenance burden and improved build times"
                        .to_string(),
                    effort: EffortLevel::Easy,
                    related_findings: vec!["dead_code".to_string()],
                });
            }
        }

        Ok(recommendations)
    }

    /// Create visualizations
    fn create_visualizations(
        &self,
        results: &AnalysisResults,
        sections: &[ReportSection],
    ) -> Result<Vec<Visualization>> {
        let mut visualizations = Vec::new();

        // Complexity distribution chart
        if let Some(complexity) = &results.complexity_analysis {
            visualizations.push(self.create_complexity_distribution_chart(complexity)?);
        }

        // Health score gauge
        let health_score = self.calculate_health_score(results);
        visualizations.push(self.create_health_score_gauge(health_score)?);

        // Issue distribution pie chart
        visualizations.push(self.create_issue_distribution_chart(sections)?);

        Ok(visualizations)
    }

    /// Create complexity distribution chart
    fn create_complexity_distribution_chart(
        &self,
        complexity: &ComplexityAnalysis,
    ) -> Result<Visualization> {
        let data = serde_json::json!({
            "labels": ["0-5", "6-10", "11-15", "16-20", "20+"],
            "datasets": [{
                "label": "Function Count",
                "data": complexity.distribution,
                "backgroundColor": ["#4CAF50", "#8BC34A", "#FFC107", "#FF9800", "#F44336"]
            }]
        });

        Ok(Visualization {
            title: "Complexity Distribution".to_string(),
            viz_type: VisualizationType::BarChart,
            data,
            config: HashMap::new(),
        })
    }

    /// Create health score gauge
    fn create_health_score_gauge(&self, score: f64) -> Result<Visualization> {
        let data = serde_json::json!({
            "value": score,
            "min": 0,
            "max": 100,
            "thresholds": {
                "critical": 40,
                "high": 60,
                "medium": 80
            }
        });

        Ok(Visualization {
            title: "Overall Health Score".to_string(),
            viz_type: VisualizationType::LineChart, // Would be gauge in real implementation
            data,
            config: HashMap::new(),
        })
    }

    /// Create issue distribution chart
    fn create_issue_distribution_chart(&self, sections: &[ReportSection]) -> Result<Visualization> {
        let mut issue_counts = HashMap::new();

        for section in sections {
            let count = section.findings.len();
            if count > 0 {
                issue_counts.insert(format!("{:?}", section.section_type), count);
            }
        }

        let data = serde_json::json!({
            "labels": issue_counts.keys().collect::<Vec<_>>(),
            "datasets": [{
                "data": issue_counts.values().collect::<Vec<_>>()
            }]
        });

        Ok(Visualization {
            title: "Issue Distribution by Category".to_string(),
            viz_type: VisualizationType::PieChart,
            data,
            config: HashMap::new(),
        })
    }

    /// Format report based on output format
    pub async fn format_report(
        &self,
        report: &UnifiedAnalysisReport,
        format: ReportFormat,
    ) -> Result<String> {
        match format {
            ReportFormat::Json => self.format_as_json(report),
            ReportFormat::Markdown => self.format_as_markdown(report),
            ReportFormat::Html => self.format_as_html(report).await,
            ReportFormat::Pdf => self.format_as_pdf(report).await,
            ReportFormat::Dashboard => self.format_as_dashboard(report).await,
        }
    }

    /// Format report as JSON
    fn format_as_json(&self, report: &UnifiedAnalysisReport) -> Result<String> {
        Ok(serde_json::to_string_pretty(report)?)
    }

    /// Format report as Markdown
    fn format_as_markdown(&self, report: &UnifiedAnalysisReport) -> Result<String> {
        let mut md = String::new();

        // Title
        md.push_str(&format!(
            "# {} Analysis Report\n\n",
            report.metadata.project_name
        ));

        // Metadata
        md.push_str("## Metadata\n\n");
        md.push_str(&format!("- **Date**: {}\n", report.metadata.report_date));
        md.push_str(&format!(
            "- **Tool Version**: {}\n",
            report.metadata.tool_version
        ));
        md.push_str(&format!(
            "- **Files Analyzed**: {}\n",
            report.metadata.analyzed_files
        ));
        md.push_str(&format!(
            "- **Total Lines**: {}\n\n",
            report.metadata.total_lines
        ));

        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&format!(
            "**Overall Health Score**: {:.1}/100\n\n",
            report.executive_summary.overall_health_score
        ));
        md.push_str(&format!(
            "**Risk Level**: {:?}\n\n",
            report.executive_summary.risk_assessment
        ));

        if !report.executive_summary.key_findings.is_empty() {
            md.push_str("### Key Findings\n\n");
            for finding in &report.executive_summary.key_findings {
                md.push_str(&format!("- {finding}\n"));
            }
            md.push('\n');
        }

        // Sections
        for section in &report.sections {
            md.push_str(&format!("## {}\n\n", section.title));

            // Metrics table
            if !section.metrics.is_empty() {
                md.push_str("| Metric | Value | Threshold | Trend |\n");
                md.push_str("|--------|-------|-----------|-------|\n");

                for (name, metric) in &section.metrics {
                    let threshold = metric
                        .threshold
                        .map(|t| format!("{t:.1}"))
                        .unwrap_or_else(|| "N/A".to_string());

                    md.push_str(&format!(
                        "| {} | {:.1} {} | {} | {:?} |\n",
                        name, metric.value, metric.unit, threshold, metric.trend
                    ));
                }
                md.push('\n');
            }

            // Findings
            if !section.findings.is_empty() {
                md.push_str("### Findings\n\n");
                for finding in &section.findings {
                    md.push_str(&format!(
                        "- **{:?}**: {}\n",
                        finding.severity, finding.description
                    ));
                }
                md.push('\n');
            }
        }

        // Recommendations
        if !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");

            for rec in &report.recommendations {
                md.push_str(&format!(
                    "### {} - {}\n\n",
                    match rec.priority {
                        Priority::Critical => "🔴 CRITICAL",
                        Priority::High => "🟠 HIGH",
                        Priority::Medium => "🟡 MEDIUM",
                        Priority::Low => "🟢 LOW",
                    },
                    rec.title
                ));
                md.push_str(&format!("{}\n\n", rec.description));
                md.push_str(&format!("**Expected Impact**: {}\n", rec.expected_impact));
                md.push_str(&format!("**Effort**: {:?}\n\n", rec.effort));
            }
        }

        Ok(md)
    }

    /// Format report as HTML
    async fn format_as_html(&self, report: &UnifiedAnalysisReport) -> Result<String> {
        // In a real implementation, this would use templates
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Analysis Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("h1 { color: #333; }\n");
        html.push_str(".metric { background: #f0f0f0; padding: 10px; margin: 10px 0; }\n");
        html.push_str(".health-score { font-size: 48px; font-weight: bold; }\n");
        html.push_str(".critical { color: #d32f2f; }\n");
        html.push_str(".high { color: #f57c00; }\n");
        html.push_str(".medium { color: #fbc02d; }\n");
        html.push_str(".low { color: #388e3c; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&format!(
            "<h1>{} Analysis Report</h1>\n",
            report.metadata.project_name
        ));

        // Executive summary
        html.push_str("<div class='executive-summary'>\n");
        html.push_str("<h2>Executive Summary</h2>\n");
        html.push_str(&format!(
            "<div class='health-score'>{:.1}/100</div>\n",
            report.executive_summary.overall_health_score
        ));
        html.push_str("</div>\n");

        html.push_str("</body>\n</html>");

        Ok(html)
    }

    /// Format report as PDF
    async fn format_as_pdf(&self, _report: &UnifiedAnalysisReport) -> Result<String> {
        // In a real implementation, this would generate actual PDF
        Ok("PDF generation not implemented yet".to_string())
    }

    /// Format report as interactive dashboard
    async fn format_as_dashboard(&self, _report: &UnifiedAnalysisReport) -> Result<String> {
        // In a real implementation, this would generate an interactive dashboard
        Ok("Dashboard generation not implemented yet".to_string())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        let service = EnhancedReportingService::default();

        let results = AnalysisResults {
            total_duration: std::time::Duration::from_secs(10),
            analyzed_files: 100,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 500,
                total_cognitive: 800,
                functions: 50,
                max_cyclomatic: 15,
                high_complexity_functions: 5,
                distribution: vec![10, 20, 15, 3, 2],
            }),
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 100,
                dead_functions: 5,
                dead_code_percentage: 1.0,
            }),
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };

        let score = service.calculate_health_score(&results);
        assert!(score > 80.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_risk_assessment() {
        let service = EnhancedReportingService::default();

        let results = AnalysisResults {
            total_duration: std::time::Duration::from_secs(10),
            analyzed_files: 100,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 2000,
                total_cognitive: 3000,
                functions: 50,
                max_cyclomatic: 50,
                high_complexity_functions: 20,
                distribution: vec![5, 10, 10, 10, 15],
            }),
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 2000,
                dead_functions: 50,
                dead_code_percentage: 20.0,
            }),
            duplication_analysis: Some(DuplicationAnalysis {
                duplicated_lines: 1500,
                duplicate_blocks: 30,
                duplication_percentage: 15.0,
            }),
            tdg_analysis: Some(TdgAnalysis {
                average_tdg: 5.0,
                max_tdg: 8.0,
                high_tdg_files: 20,
            }),
            big_o_analysis: None,
        };

        let risk = service.assess_overall_risk(&results);
        assert!(matches!(risk, RiskLevel::High | RiskLevel::Critical));
    }
}
