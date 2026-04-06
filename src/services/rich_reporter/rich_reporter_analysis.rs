// RichReporter analysis methods: analyze(), generate_recommendations()
// Runs data science pipeline (K-Means, PageRank, Louvain, Isolation Forest, trends)
// and generates prioritized recommendations from results.

impl RichReporter {
    /// Run data science analysis on findings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
                .expect("internal error")
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
                    trend.name,
                    trend.change_percent.abs()
                ));
            }
        }
    }
}
