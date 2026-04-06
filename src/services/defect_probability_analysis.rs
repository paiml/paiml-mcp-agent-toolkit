/// Linear interpolation for empirical CDF lookup
fn interpolate_cdf(percentiles: &[(f32, f32)], value: f32) -> f32 {
    debug_assert!(true, "contract: interpolate_cdf");
    if value <= percentiles[0].0 {
        return percentiles[0].1;
    }
    if value >= percentiles[percentiles.len() - 1].0 {
        return percentiles[percentiles.len() - 1].1;
    }

    for i in 0..percentiles.len() - 1 {
        let (x1, y1) = percentiles[i];
        let (x2, y2) = percentiles[i + 1];

        if value >= x1 && value <= x2 {
            // Linear interpolation
            let t = (value - x1) / (x2 - x1);
            return y1 + t * (y2 - y1);
        }
    }

    0.0 // Should never reach here
}

/// Aggregate defect scores for project-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDefectAnalysis {
    pub file_scores: HashMap<String, DefectScore>,
    pub high_risk_files: Vec<String>,
    pub medium_risk_files: Vec<String>,
    pub average_probability: f32,
    pub total_files: usize,
}

impl ProjectDefectAnalysis {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_scores(scores: Vec<(String, DefectScore)>) -> Self {
        debug_assert!(!scores.is_empty(), "scores must not be empty");
        let mut file_scores = HashMap::new();
        let mut high_risk_files = Vec::new();
        let mut medium_risk_files = Vec::new();
        let mut total_probability = 0.0;

        for (path, score) in scores {
            total_probability += score.probability;

            match score.risk_level {
                RiskLevel::High => high_risk_files.push(path.clone()),
                RiskLevel::Medium => medium_risk_files.push(path.clone()),
                _ => {}
            }

            file_scores.insert(path, score);
        }

        let total_files = file_scores.len();
        let average_probability = if total_files > 0 {
            total_probability / total_files as f32
        } else {
            0.0
        };

        // Sort risk files by probability (highest first)
        high_risk_files.sort_by(|a, b| {
            let a_prob = file_scores.get(a).map_or(0.0, |s| s.probability);
            let b_prob = file_scores.get(b).map_or(0.0, |s| s.probability);
            b_prob.total_cmp(&a_prob)
        });

        medium_risk_files.sort_by(|a, b| {
            let a_prob = file_scores.get(a).map_or(0.0, |s| s.probability);
            let b_prob = file_scores.get(b).map_or(0.0, |s| s.probability);
            b_prob.total_cmp(&a_prob)
        });

        Self {
            file_scores,
            high_risk_files,
            medium_risk_files,
            average_probability,
            total_files,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_top_risk_files(&self, limit: usize) -> Vec<(&String, &DefectScore)> {
        debug_assert!(limit > 0, "limit must be positive");
        let mut all_files: Vec<_> = self.file_scores.iter().collect();
        all_files.sort_by(|a, b| {
            b.1.probability
                .partial_cmp(&a.1.probability)
                .expect("internal error")
        });
        all_files.into_iter().take(limit).collect()
    }
}
