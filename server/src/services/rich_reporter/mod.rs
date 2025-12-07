//! PMAT-REPORT-V1: Universal Rich Reporting with Data Science and ASCII Visualization
//!
//! This module provides a unified reporting framework for ALL PMAT commands,
//! integrating advanced data science methods with Toyota Way principles.
//!
//! # Toyota Way Foundations
//!
//! - **Mieruka (Visual Management)**: ASCII progress bars, box drawing, color-coded severity
//! - **Genchi Genbutsu (Go and See)**: Evidence-based reports with confidence scores
//! - **Jidoka (Built-in Quality)**: Prioritized recommendations with auto-fix markers
//! - **Muda Elimination**: Inverted pyramid structure, progressive disclosure
//!
//! # Data Science Methods
//!
//! - **K-Means Clustering**: Group similar defects for batch remediation
//! - **PageRank Centrality**: Identify high-impact defects via dependency graph
//! - **Louvain Community Detection**: Discover architectural boundaries
//! - **Isolation Forest**: Detect anomalous code patterns
//! - **Time Series Analysis**: Track quality trends with change point detection
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::services::rich_reporter::{RichReporter, ReportConfig};
//!
//! let config = ReportConfig::default();
//! let reporter = RichReporter::new(config);
//!
//! // Add findings
//! reporter.add_finding(finding);
//!
//! // Analyze with data science
//! reporter.analyze();
//!
//! // Render report
//! let output = reporter.render_text();
//! println!("{}", output);
//! ```

pub mod ascii_viz;
pub mod data_science;
pub mod types;

pub use ascii_viz::*;
pub use data_science::DataScienceAnalyzer;
pub use types::*;

use std::fmt::Write;

/// Universal rich reporter for PMAT commands
pub struct RichReporter {
    /// Report configuration
    config: ReportConfig,
    /// The report being built
    report: RichReport,
    /// Data science analyzer
    analyzer: DataScienceAnalyzer,
    /// File dependencies for PageRank/Louvain
    dependencies: Vec<(String, String)>,
    /// Metric history for trends
    metric_history: Vec<(String, Vec<(i64, f64)>)>,
}

impl RichReporter {
    /// Create a new rich reporter
    pub fn new(config: ReportConfig) -> Self {
        let analyzer = DataScienceAnalyzer::new(
            config.k_clusters,
            config.pagerank_damping,
            config.louvain_resolution,
            config.anomaly_threshold,
        );

        RichReporter {
            config,
            report: RichReport::new("PMAT Report", ""),
            analyzer,
            dependencies: Vec::new(),
            metric_history: Vec::new(),
        }
    }

    /// Set report title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.report.title = title.into();
        self
    }

    /// Set project name
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.report.project = project.into();
        self
    }

    /// Add a finding to the report
    pub fn add_finding(&mut self, finding: Finding) {
        self.report.findings.push(finding);
    }

    /// Add multiple findings
    pub fn add_findings(&mut self, findings: impl IntoIterator<Item = Finding>) {
        self.report.findings.extend(findings);
    }

    /// Add file dependencies for PageRank/Louvain analysis
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.dependencies.push((from.into(), to.into()));
    }

    /// Add metric history for trend analysis
    pub fn add_metric_history(&mut self, name: impl Into<String>, data: Vec<(i64, f64)>) {
        self.metric_history.push((name.into(), data));
    }

    /// Set quality score
    pub fn set_quality_score(&mut self, score: f64) {
        self.report.quality_score = score;
    }

    /// Add a summary metric
    pub fn add_summary(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.report.summary.insert(key.into(), value.into());
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: impl Into<String>) {
        self.report.recommendations.push(recommendation.into());
    }

    /// Run data science analysis on findings
    pub fn analyze(&mut self) {
        // 1. Cluster findings (K-means)
        self.report.clusters = self.analyzer.cluster_findings(&mut self.report.findings);

        // 2. Calculate PageRank centrality
        self.analyzer
            .calculate_pagerank(&mut self.report.findings, &self.dependencies);

        // 3. Detect communities (Louvain)
        self.report.communities = self
            .analyzer
            .detect_communities(&mut self.report.findings, &self.dependencies);

        // 4. Detect anomalies (Isolation Forest)
        self.report.anomalies = self.analyzer.detect_anomalies(&mut self.report.findings);

        // 5. Analyze trends
        self.report.trends = self.analyzer.analyze_trends(&self.metric_history);

        // 6. Calculate Andon status
        self.report.calculate_andon_status();

        // 7. Generate recommendations based on analysis
        self.generate_recommendations();
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(&mut self) {
        // Clear existing recommendations
        self.report.recommendations.clear();

        // Recommend based on clusters
        for cluster in &self.report.clusters {
            if cluster.size >= 3 {
                self.report.recommendations.push(format!(
                    "Batch fix {} {} issues (cluster cohesion: {:.0}%)",
                    cluster.size,
                    cluster.primary_category,
                    cluster.cohesion * 100.0
                ));
            }
        }

        // Recommend based on high-PageRank findings
        let mut high_pagerank: Vec<_> = self
            .report
            .findings
            .iter()
            .filter(|f| f.pagerank.unwrap_or(0.0) > 0.1)
            .collect();
        high_pagerank.sort_by(|a, b| {
            b.pagerank
                .unwrap_or(0.0)
                .partial_cmp(&a.pagerank.unwrap_or(0.0))
                .unwrap()
        });

        for finding in high_pagerank.iter().take(3) {
            self.report.recommendations.push(format!(
                "Priority: Fix {} in {} (high centrality: {:.2})",
                finding.category,
                finding.location,
                finding.pagerank.unwrap_or(0.0)
            ));
        }

        // Recommend based on anomalies
        for anomaly in &self.report.anomalies {
            self.report.recommendations.push(format!(
                "Investigate anomaly: {} (score: {:.2}) - {}",
                anomaly.finding_id, anomaly.score, anomaly.action
            ));
        }

        // Recommend based on degrading trends
        for trend in &self.report.trends {
            if trend.direction == TrendDirection::Degrading && trend.change_percent.abs() > 10.0 {
                self.report.recommendations.push(format!(
                    "Address {} regression: {:.1}% degradation over window",
                    trend.name, trend.change_percent.abs()
                ));
            }
        }
    }

    /// Render report as text (ASCII art)
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        let box_drawer = BoxDrawer::default();
        let width = self.config.width;

        // Header
        writeln!(
            output,
            "{}",
            box_drawer.draw_box(
                &[
                    &self.report.title,
                    &format!(
                        "{} | {} | {}",
                        self.report.timestamp,
                        self.report.project,
                        self.report.andon_status.display()
                    ),
                ],
                width - 4
            )
        )
        .ok();

        writeln!(output).ok();

        // Summary section
        writeln!(output, "Summary").ok();
        writeln!(output, "{}", "━".repeat(width)).ok();

        let progress = ProgressBar::new(20);
        writeln!(
            output,
            "Quality Score: {} {:.1}%",
            progress.render(self.report.quality_score / 100.0),
            self.report.quality_score
        )
        .ok();

        let counts = self.report.findings_by_severity();
        writeln!(
            output,
            "Findings: {} total ({} critical, {} high, {} medium, {} low)",
            self.report.findings.len(),
            counts.get(&Severity::Critical).unwrap_or(&0),
            counts.get(&Severity::High).unwrap_or(&0),
            counts.get(&Severity::Medium).unwrap_or(&0),
            counts.get(&Severity::Low).unwrap_or(&0),
        )
        .ok();

        // Trends sparklines
        if !self.report.trends.is_empty() {
            writeln!(output).ok();
            let sparkline = Sparkline::default();
            for trend in &self.report.trends {
                writeln!(
                    output,
                    "{}: {} {} ({:+.1}%)",
                    trend.name,
                    sparkline.render(&trend.sparkline),
                    trend.direction.arrow(),
                    trend.change_percent
                )
                .ok();
            }
        }

        writeln!(output).ok();

        // Findings by cluster
        if !self.report.clusters.is_empty() {
            writeln!(output, "Defect Clusters (K-Means)").ok();
            writeln!(output, "{}", "━".repeat(width)).ok();

            for cluster in &self.report.clusters {
                writeln!(
                    output,
                    "{}",
                    TreeRenderer::branch(&format!(
                        "Cluster {}: {} ({} items, cohesion: {:.0}%)",
                        cluster.id,
                        cluster.primary_category,
                        cluster.size,
                        cluster.cohesion * 100.0
                    ))
                )
                .ok();

                for (i, finding_id) in cluster.finding_ids.iter().take(3).enumerate() {
                    if let Some(finding) = self.report.findings.iter().find(|f| &f.id == finding_id)
                    {
                        let prefix = if i == cluster.finding_ids.len().min(3) - 1 {
                            TreeRenderer::last_branch
                        } else {
                            TreeRenderer::branch
                        };
                        writeln!(
                            output,
                            "    {}",
                            prefix(&format!(
                                "{} {} - {}",
                                finding.severity.indicator(),
                                finding.location,
                                finding.message.chars().take(40).collect::<String>()
                            ))
                        )
                        .ok();
                    }
                }

                if cluster.finding_ids.len() > 3 {
                    writeln!(
                        output,
                        "    {}",
                        TreeRenderer::last_branch(&format!(
                            "... and {} more",
                            cluster.finding_ids.len() - 3
                        ))
                    )
                    .ok();
                }
            }

            writeln!(output).ok();
        }

        // PageRank centrality
        let high_pagerank: Vec<_> = self
            .report
            .findings
            .iter()
            .filter(|f| f.pagerank.is_some())
            .collect();

        if !high_pagerank.is_empty() {
            writeln!(output, "Defect Centrality (PageRank)").ok();
            writeln!(output, "{}", "━".repeat(width)).ok();

            let table = TableRenderer::new(vec![4, 8, 30, 20])
                .with_alignments(vec![true, true, false, false]);

            writeln!(
                output,
                "{}",
                table.render_header(&["Rank", "Score", "Location", "Category"])
            )
            .ok();

            let mut sorted = high_pagerank.clone();
            sorted.sort_by(|a, b| {
                b.pagerank
                    .unwrap_or(0.0)
                    .partial_cmp(&a.pagerank.unwrap_or(0.0))
                    .unwrap()
            });

            for (i, finding) in sorted.iter().take(5).enumerate() {
                writeln!(
                    output,
                    "{}",
                    table.render_row(&[
                        &format!("{}", i + 1),
                        &format!("{:.3}", finding.pagerank.unwrap_or(0.0)),
                        &finding.location.to_string(),
                        &finding.category,
                    ])
                )
                .ok();
            }

            writeln!(output, "{}", table.render_footer()).ok();
            writeln!(output).ok();
        }

        // Communities
        if !self.report.communities.is_empty() {
            writeln!(output, "Code Communities (Louvain)").ok();
            writeln!(output, "{}", "━".repeat(width)).ok();

            for community in &self.report.communities {
                writeln!(
                    output,
                    "{}",
                    box_drawer.draw_box(
                        &[
                            &format!(
                                "{} ({} files, {} defects)",
                                community.name,
                                community.files.len(),
                                community.defect_count
                            ),
                            &format!(
                                "Primary issue: {}",
                                community.primary_issue.as_deref().unwrap_or("None")
                            ),
                        ],
                        width - 10
                    )
                )
                .ok();
            }

            writeln!(output).ok();
        }

        // Anomalies
        if !self.report.anomalies.is_empty() {
            writeln!(output, "Anomalies Detected (Isolation Forest)").ok();
            writeln!(output, "{}", "━".repeat(width)).ok();

            let anomaly_bar = ProgressBar::new(30);
            for anomaly in &self.report.anomalies {
                writeln!(
                    output,
                    "{} {} (score: {:.2})",
                    StatusIndicator::warning(),
                    anomaly.finding_id,
                    anomaly.score
                )
                .ok();
                writeln!(output, "    {}", anomaly_bar.render(anomaly.score)).ok();
                writeln!(output, "    Reason: {}", anomaly.reason).ok();
                writeln!(output, "    Action: {}", anomaly.action).ok();
            }

            writeln!(output).ok();
        }

        // Recommendations
        if !self.report.recommendations.is_empty() {
            writeln!(output, "Recommendations").ok();
            writeln!(output, "{}", "━".repeat(width)).ok();

            for (i, rec) in self.report.recommendations.iter().enumerate() {
                writeln!(output, "{}. {}", i + 1, rec).ok();
            }

            writeln!(output).ok();
        }

        // Footer
        writeln!(output, "{}", "─".repeat(width)).ok();
        writeln!(output, "Generated by PMAT | {}", self.report.timestamp).ok();

        output
    }

    /// Render report as JSON
    pub fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Render report as Markdown
    pub fn render_markdown(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "# {}", self.report.title).ok();
        writeln!(output).ok();
        writeln!(
            output,
            "**Project**: {} | **Date**: {} | **Status**: {}",
            self.report.project,
            self.report.timestamp,
            self.report.andon_status.display()
        )
        .ok();
        writeln!(output).ok();

        // Summary
        writeln!(output, "## Summary").ok();
        writeln!(output).ok();
        writeln!(
            output,
            "- **Quality Score**: {:.1}%",
            self.report.quality_score
        )
        .ok();
        writeln!(
            output,
            "- **Total Findings**: {}",
            self.report.findings.len()
        )
        .ok();

        let counts = self.report.findings_by_severity();
        writeln!(
            output,
            "- **By Severity**: {} Critical, {} High, {} Medium, {} Low",
            counts.get(&Severity::Critical).unwrap_or(&0),
            counts.get(&Severity::High).unwrap_or(&0),
            counts.get(&Severity::Medium).unwrap_or(&0),
            counts.get(&Severity::Low).unwrap_or(&0),
        )
        .ok();

        writeln!(output).ok();

        // Clusters
        if !self.report.clusters.is_empty() {
            writeln!(output, "## Defect Clusters").ok();
            writeln!(output).ok();

            for cluster in &self.report.clusters {
                writeln!(
                    output,
                    "### Cluster {}: {} ({} items)",
                    cluster.id, cluster.primary_category, cluster.size
                )
                .ok();
                writeln!(output, "- Cohesion: {:.0}%", cluster.cohesion * 100.0).ok();
                writeln!(output).ok();
            }
        }

        // Recommendations
        if !self.report.recommendations.is_empty() {
            writeln!(output, "## Recommendations").ok();
            writeln!(output).ok();

            for rec in &self.report.recommendations {
                writeln!(output, "- {}", rec).ok();
            }

            writeln!(output).ok();
        }

        output
    }

    /// Render based on configured format
    pub fn render(&self) -> String {
        match self.config.format {
            OutputFormat::Text => self.render_text(),
            OutputFormat::Json => self.render_json(),
            OutputFormat::Markdown => self.render_markdown(),
        }
    }

    /// Get the report data
    pub fn report(&self) -> &RichReport {
        &self.report
    }

    /// Get mutable report data
    pub fn report_mut(&mut self) -> &mut RichReport {
        &mut self.report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_finding(id: &str, category: &str, severity: Severity) -> Finding {
        Finding {
            id: id.to_string(),
            category: category.to_string(),
            severity,
            location: SourceLocation {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
                scope: None,
            },
            message: "Test finding".to_string(),
            confidence: 0.9,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: None,
        }
    }

    #[test]
    fn test_rich_reporter_new() {
        let config = ReportConfig::default();
        let reporter = RichReporter::new(config);
        assert_eq!(reporter.report.findings.len(), 0);
    }

    #[test]
    fn test_rich_reporter_add_finding() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);
        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        assert_eq!(reporter.report.findings.len(), 1);
    }

    #[test]
    fn test_rich_reporter_analyze() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);

        for i in 0..5 {
            reporter.add_finding(create_test_finding(
                &format!("{}", i),
                "TypeMismatch",
                Severity::High,
            ));
        }

        reporter.analyze();

        // Should have assigned cluster IDs
        assert!(reporter.report.findings.iter().all(|f| f.cluster_id.is_some()));
    }

    #[test]
    fn test_rich_reporter_render_text() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config)
            .with_title("Test Report")
            .with_project("test-project");

        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.set_quality_score(85.0);
        reporter.analyze();

        let output = reporter.render_text();
        assert!(output.contains("Test Report"));
        assert!(output.contains("Quality Score"));
        assert!(output.contains("85.0%"));
    }

    #[test]
    fn test_rich_reporter_render_json() {
        let config = ReportConfig {
            format: OutputFormat::Json,
            ..Default::default()
        };
        let mut reporter = RichReporter::new(config);
        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.analyze();

        let output = reporter.render_json();
        assert!(output.contains("\"findings\""));
        assert!(output.contains("TypeMismatch"));
    }

    #[test]
    fn test_rich_reporter_render_markdown() {
        let config = ReportConfig {
            format: OutputFormat::Markdown,
            ..Default::default()
        };
        let mut reporter = RichReporter::new(config)
            .with_title("Test Report")
            .with_project("test-project");

        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.analyze();

        let output = reporter.render_markdown();
        assert!(output.contains("# Test Report"));
        assert!(output.contains("## Summary"));
    }

    #[test]
    fn test_andon_status_calculation() {
        let mut report = RichReport::new("Test", "test");

        // No findings = Green
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Green);

        // High finding = Yellow
        report.findings.push(create_test_finding("1", "Test", Severity::High));
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Yellow);

        // Critical finding = Red
        report.findings.push(create_test_finding("2", "Test", Severity::Critical));
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Red);
    }

    #[test]
    fn test_trend_analysis_integration() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);

        // Add metric history
        let data: Vec<(i64, f64)> = (0..10).map(|i| (i as i64, 50.0 + i as f64 * 2.0)).collect();
        reporter.add_metric_history("coverage", data);

        reporter.analyze();

        assert_eq!(reporter.report.trends.len(), 1);
        assert_eq!(reporter.report.trends[0].name, "coverage");
    }
}
