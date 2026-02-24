impl DataScienceAnalyzer {
    /// Detect anomalies using statistical methods
    ///
    /// Uses Z-score based outlier detection (simpler than Isolation Forest)
    /// Anomalies are findings that deviate significantly from the norm
    #[allow(clippy::cast_possible_truncation)]
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

        for (i, (&v, (&mean, &std))) in features
            .iter()
            .zip(means.iter().zip(stds.iter()))
            .enumerate()
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
}
