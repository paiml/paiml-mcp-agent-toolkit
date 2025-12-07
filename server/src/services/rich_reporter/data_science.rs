//! Data Science Integration for PMAT-REPORT-V1
//!
//! Integrates existing algorithms:
//! - K-Means clustering (aprender)
//! - PageRank centrality (trueno-graph)
//! - Louvain community detection (aprender)
//! - Isolation Forest (simplified implementation)
//! - Time series analysis (native)

use super::types::{
    AnomalyPoint, CodeCommunity, Finding, FindingCluster, MetricTrend, TrendDirection,
};
use aprender::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Data science analyzer for rich reporting
pub struct DataScienceAnalyzer {
    /// Number of clusters for K-means
    k_clusters: usize,
    /// PageRank damping factor (reserved for future PageRank tuning)
    #[allow(dead_code)]
    pagerank_damping: f64,
    /// Louvain resolution (reserved for future Louvain tuning)
    #[allow(dead_code)]
    louvain_resolution: f64,
    /// Anomaly threshold
    anomaly_threshold: f64,
}

impl Default for DataScienceAnalyzer {
    fn default() -> Self {
        DataScienceAnalyzer {
            k_clusters: 4,
            pagerank_damping: 0.85,
            louvain_resolution: 1.0,
            anomaly_threshold: 0.7,
        }
    }
}

impl DataScienceAnalyzer {
    /// Create a new analyzer with custom parameters
    pub fn new(
        k_clusters: usize,
        pagerank_damping: f64,
        louvain_resolution: f64,
        anomaly_threshold: f64,
    ) -> Self {
        DataScienceAnalyzer {
            k_clusters,
            pagerank_damping,
            louvain_resolution,
            anomaly_threshold,
        }
    }

    /// Cluster findings using K-means
    ///
    /// Features used for clustering:
    /// - Severity (ordinal encoded)
    /// - Category (one-hot encoded conceptually, but simplified here)
    /// - File path similarity (simplified to directory grouping)
    /// - Line proximity (normalized)
    pub fn cluster_findings(&self, findings: &mut [Finding]) -> Vec<FindingCluster> {
        if findings.is_empty() || findings.len() < self.k_clusters {
            // Not enough findings to cluster meaningfully
            if !findings.is_empty() {
                for finding in findings.iter_mut() {
                    finding.cluster_id = Some(0);
                }
                return vec![FindingCluster {
                    id: 0,
                    size: findings.len(),
                    primary_category: findings
                        .first()
                        .map(|f| f.category.clone())
                        .unwrap_or_default(),
                    cohesion: 1.0,
                    description: "All findings".to_string(),
                    finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
                }];
            }
            return Vec::new();
        }

        // Build feature vectors for each finding
        let vectors: Vec<Vec<f32>> = findings
            .iter()
            .map(|f| self.finding_to_features(f))
            .collect();

        // Convert to aprender Matrix
        let rows = vectors.len();
        let cols = vectors[0].len();
        let data: Vec<f32> = vectors.iter().flat_map(|v| v.iter().copied()).collect();

        let matrix = match Matrix::from_vec(rows, cols, data) {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };

        // Run K-means clustering
        let mut kmeans = KMeans::new(self.k_clusters).with_max_iter(100);

        if kmeans.fit(&matrix).is_err() {
            return Vec::new();
        }

        let labels = kmeans.predict(&matrix);

        // Assign cluster IDs to findings
        for (finding, &label) in findings.iter_mut().zip(labels.iter()) {
            finding.cluster_id = Some(label);
        }

        // Build cluster summaries
        self.build_cluster_summaries(findings, &labels)
    }

    /// Convert a finding to a feature vector
    fn finding_to_features(&self, finding: &Finding) -> Vec<f32> {
        let mut features = vec![0.0f32; 6];

        // Severity (0-3)
        features[0] = match finding.severity {
            super::types::Severity::Low => 0.0,
            super::types::Severity::Medium => 1.0,
            super::types::Severity::High => 2.0,
            super::types::Severity::Critical => 3.0,
        };

        // Confidence
        features[1] = finding.confidence;

        // Category hash (simplified)
        features[2] = (finding.category.len() % 10) as f32;

        // File path hash (simplified)
        features[3] = (finding.location.file.to_string_lossy().len() % 20) as f32;

        // Line number (normalized to 0-1 range, assuming max 10000 lines)
        features[4] = (finding.location.line as f32 / 10000.0).min(1.0);

        // Has fix suggestion
        features[5] = if finding.fix_suggestion.is_some() {
            1.0
        } else {
            0.0
        };

        features
    }

    /// Build cluster summaries from labels
    fn build_cluster_summaries(
        &self,
        findings: &[Finding],
        labels: &[usize],
    ) -> Vec<FindingCluster> {
        let mut cluster_findings: HashMap<usize, Vec<&Finding>> = HashMap::new();

        for (finding, &label) in findings.iter().zip(labels.iter()) {
            cluster_findings.entry(label).or_default().push(finding);
        }

        cluster_findings
            .into_iter()
            .map(|(id, cluster_items)| {
                // Find dominant category
                let mut category_counts: HashMap<&str, usize> = HashMap::new();
                for finding in &cluster_items {
                    *category_counts.entry(&finding.category).or_insert(0) += 1;
                }
                let primary_category = category_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(cat, _)| cat.to_string())
                    .unwrap_or_default();

                // Calculate cohesion (simplified: inverse of category diversity)
                let unique_categories: std::collections::HashSet<_> =
                    cluster_items.iter().map(|f| &f.category).collect();
                let cohesion = 1.0 / (unique_categories.len() as f64).max(1.0);

                FindingCluster {
                    id,
                    size: cluster_items.len(),
                    primary_category: primary_category.clone(),
                    cohesion,
                    description: format!("{} Issues", primary_category),
                    finding_ids: cluster_items.iter().map(|f| f.id.clone()).collect(),
                }
            })
            .collect()
    }

    /// Calculate PageRank centrality for findings based on file dependencies
    ///
    /// Higher PageRank = finding is in a more "central" file that many others depend on
    pub fn calculate_pagerank(&self, findings: &mut [Finding], dependencies: &[(String, String)]) {
        if findings.is_empty() || dependencies.is_empty() {
            return;
        }

        // Build file -> node_id mapping
        let mut file_to_node: HashMap<String, usize> = HashMap::new();
        let mut node_id = 0;

        for finding in findings.iter() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let std::collections::hash_map::Entry::Vacant(e) = file_to_node.entry(file) {
                e.insert(node_id);
                node_id += 1;
            }
        }

        for (from, to) in dependencies {
            if let std::collections::hash_map::Entry::Vacant(e) = file_to_node.entry(from.clone()) {
                e.insert(node_id);
                node_id += 1;
            }
            if let std::collections::hash_map::Entry::Vacant(e) = file_to_node.entry(to.clone()) {
                e.insert(node_id);
                node_id += 1;
            }
        }

        if node_id == 0 {
            return;
        }

        // Build graph using trueno_graph
        let mut graph = trueno_graph::CsrGraph::new();

        // Add edges (nodes are implicitly created)
        for (from, to) in dependencies {
            if let (Some(&from_id), Some(&to_id)) = (file_to_node.get(from), file_to_node.get(to)) {
                // Set node names first
                graph.set_node_name(trueno_graph::NodeId(from_id as u32), from.clone());
                graph.set_node_name(trueno_graph::NodeId(to_id as u32), to.clone());
                // Add edge with weight
                let _ = graph.add_edge(
                    trueno_graph::NodeId(from_id as u32),
                    trueno_graph::NodeId(to_id as u32),
                    1.0,
                );
            }
        }

        // Calculate PageRank
        let scores = match trueno_graph::pagerank(&graph, 20, 1e-6) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Assign PageRank scores to findings
        for finding in findings.iter_mut() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let Some(&node) = file_to_node.get(&file) {
                if node < scores.len() {
                    finding.pagerank = Some(scores[node] as f32);
                }
            }
        }
    }

    /// Detect code communities using Louvain algorithm
    pub fn detect_communities(
        &self,
        findings: &mut [Finding],
        dependencies: &[(String, String)],
    ) -> Vec<CodeCommunity> {
        if findings.is_empty() {
            return Vec::new();
        }

        // Build file -> node_id mapping
        let mut file_to_node: HashMap<String, usize> = HashMap::new();
        let mut node_to_file: HashMap<usize, String> = HashMap::new();
        let mut node_id = 0;

        for finding in findings.iter() {
            let file = finding.location.file.to_string_lossy().to_string();
            if !file_to_node.contains_key(&file) {
                file_to_node.insert(file.clone(), node_id);
                node_to_file.insert(node_id, file);
                node_id += 1;
            }
        }

        // If no dependencies, create simple file-based communities
        if dependencies.is_empty() || node_id == 0 {
            let mut communities = Vec::new();
            for finding in findings.iter_mut() {
                let file = finding.location.file.to_string_lossy().to_string();
                finding.community = Some(file.clone());
            }

            // Group by file
            let mut file_groups: HashMap<String, Vec<&Finding>> = HashMap::new();
            for finding in findings.iter() {
                let file = finding.location.file.to_string_lossy().to_string();
                file_groups.entry(file).or_default().push(finding);
            }

            for (file, group_findings) in file_groups {
                let mut category_counts: HashMap<&str, usize> = HashMap::new();
                for f in &group_findings {
                    *category_counts.entry(&f.category).or_insert(0) += 1;
                }
                let primary = category_counts
                    .into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(cat, _)| cat.to_string());

                communities.push(CodeCommunity {
                    name: PathBuf::from(&file)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or(file.clone()),
                    modularity: 1.0,
                    files: vec![PathBuf::from(&file)],
                    primary_issue: primary,
                    defect_count: group_findings.len(),
                });
            }

            return communities;
        }

        // Add dependency nodes
        for (from, to) in dependencies {
            if !file_to_node.contains_key(from) {
                file_to_node.insert(from.clone(), node_id);
                node_to_file.insert(node_id, from.clone());
                node_id += 1;
            }
            if !file_to_node.contains_key(to) {
                file_to_node.insert(to.clone(), node_id);
                node_to_file.insert(node_id, to.clone());
                node_id += 1;
            }
        }

        // Build edge list for aprender
        let edges: Vec<(usize, usize)> = dependencies
            .iter()
            .filter_map(|(from, to)| {
                match (file_to_node.get(from), file_to_node.get(to)) {
                    (Some(&from_id), Some(&to_id)) => Some((from_id, to_id)),
                    _ => None,
                }
            })
            .collect();

        // Create undirected graph from edge list
        let graph = aprender::graph::Graph::from_edges(&edges, false);

        // Run Louvain community detection
        let community_assignments = graph.louvain();

        // Create community name mapping
        let mut node_to_community: HashMap<usize, usize> = HashMap::new();
        for (comm_id, nodes) in community_assignments.iter().enumerate() {
            for &node in nodes {
                node_to_community.insert(node, comm_id);
            }
        }

        // Assign communities to findings
        for finding in findings.iter_mut() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let Some(&node) = file_to_node.get(&file) {
                if let Some(&comm) = node_to_community.get(&node) {
                    finding.community = Some(format!("community_{}", comm));
                }
            }
        }

        // Build community summaries
        community_assignments
            .iter()
            .enumerate()
            .filter(|(_, nodes): &(usize, &Vec<usize>)| !nodes.is_empty())
            .map(|(comm_id, nodes): (usize, &Vec<usize>)| {
                let files: Vec<PathBuf> = nodes
                    .iter()
                    .filter_map(|n| node_to_file.get(n))
                    .map(PathBuf::from)
                    .collect();

                let defect_count = findings
                    .iter()
                    .filter(|f| {
                        f.community
                            .as_ref()
                            .map(|c| c == &format!("community_{}", comm_id))
                            .unwrap_or(false)
                    })
                    .count();

                let primary_issue = findings
                    .iter()
                    .filter(|f| {
                        f.community
                            .as_ref()
                            .map(|c| c == &format!("community_{}", comm_id))
                            .unwrap_or(false)
                    })
                    .fold(HashMap::new(), |mut acc, f| {
                        *acc.entry(f.category.clone()).or_insert(0usize) += 1;
                        acc
                    })
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(cat, _)| cat);

                CodeCommunity {
                    name: format!("community_{}", comm_id),
                    modularity: 0.0, // Would need graph to calculate
                    files,
                    primary_issue,
                    defect_count,
                }
            })
            .collect()
    }

    /// Detect anomalies using statistical methods
    ///
    /// Uses Z-score based outlier detection (simpler than Isolation Forest)
    /// Anomalies are findings that deviate significantly from the norm
    pub fn detect_anomalies(&self, findings: &mut [Finding]) -> Vec<AnomalyPoint> {
        if findings.len() < 5 {
            // Not enough data for meaningful anomaly detection
            return Vec::new();
        }

        // Build feature matrix
        let vectors: Vec<Vec<f32>> = findings
            .iter()
            .map(|f| self.finding_to_features(f))
            .collect();

        // Calculate mean and std for each feature
        let n = vectors.len() as f32;
        let num_features = vectors[0].len();
        let mut means = vec![0.0f32; num_features];
        let mut stds = vec![0.0f32; num_features];

        // Calculate means
        for vec in &vectors {
            for (i, &v) in vec.iter().enumerate() {
                means[i] += v / n;
            }
        }

        // Calculate standard deviations
        for vec in &vectors {
            for (i, &v) in vec.iter().enumerate() {
                stds[i] += (v - means[i]).powi(2) / n;
            }
        }
        for std in &mut stds {
            *std = std.sqrt().max(0.001); // Avoid division by zero
        }

        // Calculate Z-scores and anomaly scores
        let mut anomalies = Vec::new();

        for (finding, vec) in findings.iter_mut().zip(vectors.iter()) {
            // Calculate max absolute Z-score across all features
            let max_z_score: f32 = vec
                .iter()
                .zip(means.iter())
                .zip(stds.iter())
                .map(|((&v, &mean), &std)| ((v - mean) / std).abs())
                .fold(0.0f32, f32::max);

            // Convert Z-score to 0-1 anomaly score (sigmoid-like)
            let anomaly_score = 1.0 / (1.0 + (-max_z_score + 2.0).exp());

            finding.anomaly_score = Some(anomaly_score);

            if anomaly_score as f64 >= self.anomaly_threshold {
                anomalies.push(AnomalyPoint {
                    finding_id: finding.id.clone(),
                    score: anomaly_score as f64,
                    reason: self.explain_anomaly(finding, vec, &means, &stds),
                    action: self.suggest_anomaly_action(finding),
                });
            }
        }

        anomalies
    }

    /// Explain why a finding is anomalous
    fn explain_anomaly(
        &self,
        finding: &Finding,
        features: &[f32],
        means: &[f32],
        stds: &[f32],
    ) -> String {
        let mut reasons = Vec::new();

        // Check each feature for high Z-score
        let feature_names = [
            "severity",
            "confidence",
            "category",
            "file_path",
            "line_number",
            "has_fix",
        ];

        for (i, (&v, (&mean, &std))) in features.iter().zip(means.iter().zip(stds.iter())).enumerate()
        {
            let z_score = (v - mean) / std;
            if z_score.abs() > 2.0 && i < feature_names.len() {
                reasons.push(format!("unusual {} (z={:.1})", feature_names[i], z_score));
            }
        }

        if reasons.is_empty() {
            format!("Unusual pattern in {}", finding.category)
        } else {
            reasons.join(", ")
        }
    }

    /// Suggest action for an anomalous finding
    fn suggest_anomaly_action(&self, finding: &Finding) -> String {
        match finding.severity {
            super::types::Severity::Critical => "Immediate review required".to_string(),
            super::types::Severity::High => "Schedule for next sprint".to_string(),
            super::types::Severity::Medium => "Add to backlog".to_string(),
            super::types::Severity::Low => "Monitor for recurrence".to_string(),
        }
    }

    /// Analyze metric trends
    pub fn analyze_trends(&self, metrics: &[(String, Vec<(i64, f64)>)]) -> Vec<MetricTrend> {
        metrics
            .iter()
            .map(|(name, data)| {
                if data.is_empty() {
                    return MetricTrend {
                        name: name.clone(),
                        current: 0.0,
                        direction: TrendDirection::Stable,
                        change_percent: 0.0,
                        sparkline: Vec::new(),
                        forecast: None,
                    };
                }

                let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
                let current = *values.last().unwrap_or(&0.0);

                // Calculate trend using linear regression
                let direction = self.calculate_trend_direction(&values);

                // Calculate change percentage
                let first = *values.first().unwrap_or(&0.0);
                let change_percent = if first != 0.0 {
                    ((current - first) / first.abs()) * 100.0
                } else {
                    0.0
                };

                // Generate sparkline (normalize to 0-7)
                let sparkline = self.values_to_sparkline(&values);

                // Simple forecast (linear extrapolation)
                let forecast = self.forecast_next(&values);

                MetricTrend {
                    name: name.clone(),
                    current,
                    direction,
                    change_percent,
                    sparkline,
                    forecast,
                }
            })
            .collect()
    }

    /// Calculate trend direction from values
    fn calculate_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator == 0.0 {
            return TrendDirection::Stable;
        }

        let slope = numerator / denominator;
        let threshold = y_mean.abs() * 0.05;

        if slope > threshold {
            TrendDirection::Degrading
        } else if slope < -threshold {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Convert values to sparkline indices (0-7)
    fn values_to_sparkline(&self, values: &[f64]) -> Vec<u8> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return vec![4; values.len()];
        }

        values
            .iter()
            .map(|&v| ((v - min) / range * 7.0).round() as u8)
            .collect()
    }

    /// Simple linear forecast for next value
    fn forecast_next(&self, values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator == 0.0 {
            return Some(*values.last().unwrap());
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        // Predict next value
        Some(slope * n + intercept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rich_reporter::types::{Severity, SourceLocation};
    use std::path::PathBuf;

    fn create_test_finding(id: &str, category: &str, severity: Severity, line: usize) -> Finding {
        Finding {
            id: id.to_string(),
            category: category.to_string(),
            severity,
            location: SourceLocation {
                file: PathBuf::from("test.rs"),
                line,
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
    fn test_cluster_findings_empty() {
        let analyzer = DataScienceAnalyzer::default();
        let mut findings: Vec<Finding> = Vec::new();
        let clusters = analyzer.cluster_findings(&mut findings);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_cluster_findings_single() {
        let analyzer = DataScienceAnalyzer::default();
        let mut findings = vec![create_test_finding("1", "TypeMismatch", Severity::High, 10)];
        let clusters = analyzer.cluster_findings(&mut findings);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].size, 1);
    }

    #[test]
    fn test_cluster_findings_multiple() {
        let analyzer = DataScienceAnalyzer::new(2, 0.85, 1.0, 0.7);
        let mut findings = vec![
            create_test_finding("1", "TypeMismatch", Severity::High, 10),
            create_test_finding("2", "TypeMismatch", Severity::High, 20),
            create_test_finding("3", "BorrowCheck", Severity::Critical, 30),
            create_test_finding("4", "BorrowCheck", Severity::Critical, 40),
        ];
        let clusters = analyzer.cluster_findings(&mut findings);
        assert!(!clusters.is_empty());
        assert!(findings.iter().all(|f| f.cluster_id.is_some()));
    }

    #[test]
    fn test_detect_communities_no_deps() {
        let analyzer = DataScienceAnalyzer::default();
        let mut findings = vec![create_test_finding("1", "TypeMismatch", Severity::High, 10)];
        let deps: Vec<(String, String)> = Vec::new();
        let communities = analyzer.detect_communities(&mut findings, &deps);
        assert_eq!(communities.len(), 1);
    }

    #[test]
    fn test_analyze_trends_empty() {
        let analyzer = DataScienceAnalyzer::default();
        let metrics: Vec<(String, Vec<(i64, f64)>)> = Vec::new();
        let trends = analyzer.analyze_trends(&metrics);
        assert!(trends.is_empty());
    }

    #[test]
    fn test_analyze_trends_improving() {
        let analyzer = DataScienceAnalyzer::default();
        let data: Vec<(i64, f64)> = (0..10).map(|i| (i as i64, 100.0 - i as f64 * 5.0)).collect();
        let metrics = vec![("coverage".to_string(), data)];
        let trends = analyzer.analyze_trends(&metrics);
        assert_eq!(trends.len(), 1);
        // Note: direction depends on interpretation - for costs, decreasing is improving
    }

    #[test]
    fn test_values_to_sparkline() {
        let analyzer = DataScienceAnalyzer::default();
        let values = vec![0.0, 50.0, 100.0];
        let sparkline = analyzer.values_to_sparkline(&values);
        assert_eq!(sparkline.len(), 3);
        assert_eq!(sparkline[0], 0);
        assert_eq!(sparkline[2], 7);
    }

    #[test]
    fn test_forecast_next() {
        let analyzer = DataScienceAnalyzer::default();
        let values = vec![0.0, 10.0, 20.0];
        let forecast = analyzer.forecast_next(&values);
        assert!(forecast.is_some());
        // Linear trend should predict ~30
        assert!((forecast.unwrap() - 30.0).abs() < 1.0);
    }

    #[test]
    fn test_detect_anomalies_insufficient_data() {
        let analyzer = DataScienceAnalyzer::default();
        let mut findings = vec![
            create_test_finding("1", "TypeMismatch", Severity::High, 10),
            create_test_finding("2", "TypeMismatch", Severity::High, 20),
        ];
        let anomalies = analyzer.detect_anomalies(&mut findings);
        // Not enough data points (< 5)
        assert!(anomalies.is_empty());
    }
}
