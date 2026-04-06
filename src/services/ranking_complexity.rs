pub struct ComplexityRanker {
    /// Weight for cyclomatic complexity (0.0 - 1.0)
    pub cyclomatic_weight: f64,
    /// Weight for cognitive complexity (0.0 - 1.0)
    pub cognitive_weight: f64,
    /// Weight for function count (0.0 - 1.0)
    pub function_count_weight: f64,
}

impl Default for ComplexityRanker {
    fn default() -> Self {
        Self {
            cyclomatic_weight: 0.4,
            cognitive_weight: 0.4,
            function_count_weight: 0.2,
        }
    }
}

impl ComplexityRanker {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(cyclomatic_weight: f64, cognitive_weight: f64, function_count_weight: f64) -> Self {
        Self {
            cyclomatic_weight,
            cognitive_weight,
            function_count_weight,
        }
    }

    /// Calculate composite complexity score from file metrics
    fn calculate_composite_score(
        &self,
        metrics: &FileComplexityMetrics,
    ) -> CompositeComplexityScore {
        // Extract metrics from functions and classes
        let all_functions: Vec<_> = metrics
            .functions
            .iter()
            .chain(metrics.classes.iter().flat_map(|c| &c.methods))
            .collect();

        let function_count = all_functions.len();

        if function_count == 0 {
            return CompositeComplexityScore::default();
        }

        // Calculate max cyclomatic complexity
        let cyclomatic_max = all_functions
            .iter()
            .map(|f| u32::from(f.metrics.cyclomatic))
            .max()
            .unwrap_or(0);

        // Calculate average cognitive complexity
        let cognitive_total: u32 = all_functions
            .iter()
            .map(|f| u32::from(f.metrics.cognitive))
            .sum();
        let cognitive_avg = f64::from(cognitive_total) / function_count as f64;

        // Mock halstead effort (would need proper calculation)
        let halstead_effort = f64::from(
            all_functions
                .iter()
                .map(|f| u32::from(f.metrics.lines) * 10) // Simple approximation
                .sum::<u32>(),
        );

        // Calculate composite score
        let normalized_cyclomatic = f64::from(cyclomatic_max).min(50.0) / 50.0; // Normalize to 0-1
        let normalized_cognitive = cognitive_avg.min(100.0) / 100.0; // Normalize to 0-1
        let normalized_function_count = (function_count as f64).min(100.0) / 100.0; // Normalize to 0-1

        let total_score = (self.cyclomatic_weight * normalized_cyclomatic * 100.0)
            + (self.cognitive_weight * normalized_cognitive * 100.0)
            + (self.function_count_weight * normalized_function_count * 50.0); // Function count weighted less

        CompositeComplexityScore {
            cyclomatic_max,
            cognitive_avg,
            halstead_effort,
            function_count,
            total_score,
        }
    }
}

/// Language-specific scoring parameters: (size_divisor, weight, func_div, cyc_div, cog_div, halstead_mul)
fn language_score_params(ext: &str) -> Option<(f64, f64, f64, f64, f64, f64)> {
    match ext {
        "rs" => Some((1000.0, 1.0, 10.0, 5.0, 3.0, 10.0)),
        "ts" | "tsx" | "js" | "jsx" => Some((1200.0, 0.9, 12.0, 6.0, 4.0, 8.0)),
        "py" => Some((800.0, 1.1, 8.0, 4.0, 2.5, 12.0)),
        _ => None,
    }
}

fn score_from_file_size(file_path: &Path, params: (f64, f64, f64, f64, f64, f64)) -> CompositeComplexityScore {
    let (size_div, weight, func_div, cyc_div, cog_div, halstead_mul) = params;
    let Ok(metadata) = std::fs::metadata(file_path) else {
        return CompositeComplexityScore::default();
    };
    let size_score = (metadata.len() as f64 / size_div).min(100.0);
    CompositeComplexityScore {
        total_score: size_score * weight,
        function_count: (size_score / func_div) as usize,
        cyclomatic_max: (size_score / cyc_div) as u32,
        cognitive_avg: size_score / cog_div,
        halstead_effort: size_score * halstead_mul,
    }
}

impl FileRanker for ComplexityRanker {
    type Metric = CompositeComplexityScore;

    fn compute_score(&self, file_path: &Path) -> Self::Metric {
        let ext = file_path.extension().and_then(|s| s.to_str());
        match ext.and_then(language_score_params) {
            Some(params) => score_from_file_size(file_path, params),
            None => CompositeComplexityScore::default(),
        }
    }

    fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
        format!(
            "| {:>4} | {:<50} | {:>9} | {:>14} | {:>13.1} | {:>11.1} | {:>11.1} |",
            rank,
            file,
            metric.function_count,
            metric.cyclomatic_max,
            metric.cognitive_avg,
            metric.halstead_effort,
            metric.total_score
        )
    }

    fn ranking_type(&self) -> &'static str {
        "Complexity"
    }
}

/// Create a complexity ranker from file metrics (more accurate)
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn rank_files_by_complexity(
    file_metrics: &[FileComplexityMetrics],
    limit: usize,
    ranker: &ComplexityRanker,
) -> Vec<(String, CompositeComplexityScore)> {
    let mut rankings: Vec<_> = file_metrics
        .iter()
        .map(|metrics| {
            let score = ranker.calculate_composite_score(metrics);
            (metrics.path.clone(), score)
        })
        .collect();

    // Sort by total score (descending)
    rankings.sort_by(|a, b| {
        b.1.total_score
            .partial_cmp(&a.1.total_score)
            .unwrap_or(Ordering::Equal)
    });

    if limit > 0 {
        rankings.truncate(limit);
    }

    rankings
}
