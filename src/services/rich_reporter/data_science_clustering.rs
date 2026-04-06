impl DataScienceAnalyzer {
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
    #[allow(clippy::cast_possible_truncation)]
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
        debug_assert!(!findings.is_empty(), "findings must not be empty");
        debug_assert!(!labels.is_empty(), "labels must not be empty");
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
}
