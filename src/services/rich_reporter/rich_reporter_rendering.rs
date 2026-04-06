// RichReporter rendering methods: render_text, render_json, render_markdown, render
// Also includes report() and report_mut() accessors.
// Produces ASCII art, JSON, and Markdown output formats.

impl RichReporter {
    /// Render report as text (ASCII art)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

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
            writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

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
            writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

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
                    .expect("internal error")
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
            writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

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
            writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

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
            writeln!(output, "{}", "\u{2501}".repeat(width)).ok();

            for (i, rec) in self.report.recommendations.iter().enumerate() {
                writeln!(output, "{}. {}", i + 1, rec).ok();
            }

            writeln!(output).ok();
        }

        // Footer
        writeln!(output, "{}", "\u{2500}".repeat(width)).ok();
        writeln!(output, "Generated by PMAT | {}", self.report.timestamp).ok();

        output
    }

    /// Render report as JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Render report as Markdown
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render(&self) -> String {
        match self.config.format {
            OutputFormat::Text | OutputFormat::Plain => self.render_text(),
            OutputFormat::Json => self.render_json(),
            OutputFormat::Markdown => self.render_markdown(),
            _ => self.render_text(),
        }
    }

    /// Get the report data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn report(&self) -> &RichReport {
        &self.report
    }

    /// Get mutable report data
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn report_mut(&mut self) -> &mut RichReport {
        &mut self.report
    }
}
