// Core analysis functions for defect prediction
// Included from defect_prediction_helpers.rs - shares parent module scope

#[allow(dead_code)]
pub struct DefectPredictionConfig {
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
}

#[allow(dead_code)]
pub struct DefectAnalysisResult {
    pub file_metrics: Vec<FileMetrics>,
    pub filtered_predictions: Vec<(String, DefectScore)>,
    pub analysis_time: std::time::Duration,
}

/// Discover source files for defect analysis
pub async fn discover_source_files_for_defect_analysis(
    project_path: &Path,
    config: &DefectPredictionConfig,
) -> Result<Vec<(PathBuf, String, usize)>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let mut discovery_config = FileDiscoveryConfig::default();

    if let Some(exclude_pattern) = &config.exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery =
        ProjectFileDiscovery::new(project_path.to_path_buf()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        if let Some(include_pattern) = &config.include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let lines_of_code = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();

            if lines_of_code >= config.min_lines {
                analyzed_files.push((file_path, content, lines_of_code));
            }
        }
    }

    Ok(analyzed_files)
}

/// Calculate simple complexity metric from source code
#[must_use]
pub fn calculate_simple_complexity(content: &str) -> u32 {
    let mut complexity = 1u32;

    for line in content.lines() {
        let trimmed = line.trim();
        complexity += count_line_complexity(trimmed);
    }

    complexity
}

fn count_line_complexity(line: &str) -> u32 {
    let mut line_complexity = 0u32;

    line_complexity += count_conditional_statements(line);
    line_complexity += count_loop_statements(line);
    line_complexity += count_pattern_matching(line);
    line_complexity += count_logical_operators(line);
    line_complexity += count_exception_handling(line);

    line_complexity
}

fn count_conditional_statements(line: &str) -> u32 {
    u32::from(line.starts_with("if ") || line.starts_with("else if"))
}

fn count_loop_statements(line: &str) -> u32 {
    u32::from(line.starts_with("for ") || line.starts_with("while "))
}

fn count_pattern_matching(line: &str) -> u32 {
    u32::from(
        line.starts_with("match ")
            || line.starts_with("switch ")
            || line.contains("=>")
            || line.starts_with("case "),
    )
}

fn count_logical_operators(line: &str) -> u32 {
    u32::from(line.contains("&&") || line.contains("||"))
}

fn count_exception_handling(line: &str) -> u32 {
    u32::from(line.starts_with("catch") || line.starts_with("except"))
}

/// Calculate simple churn score based on file content
#[must_use]
pub fn calculate_simple_churn_score(content: &str, lines_of_code: usize) -> f32 {
    // Simple heuristic based on comments and file size
    let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
    let comment_lines = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('#')
        })
        .count();

    let comment_ratio = comment_lines as f32 / lines_of_code.max(1) as f32;
    let todo_factor = (todo_count as f32 * 0.1).min(1.0);

    // Higher churn for files with many TODOs or low comment ratio
    (1.0 - comment_ratio) * 0.5 + todo_factor * 0.5
}

/// Collect metrics for all files
#[must_use]
pub fn collect_file_metrics(analyzed_files: &[(PathBuf, String, usize)]) -> Vec<FileMetrics> {
    let mut file_metrics = Vec::new();

    for (file_path, content, lines_of_code) in analyzed_files {
        let cyclomatic_complexity = calculate_simple_complexity(content);
        let cognitive_complexity = (cyclomatic_complexity as f32 * 1.3) as u32;
        let churn_score = calculate_simple_churn_score(content, *lines_of_code);

        let afferent_coupling = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("use ")
                    || line.trim_start().starts_with("import ")
                    || line.trim_start().starts_with("#include")
            })
            .count() as f32;

        let metrics = FileMetrics {
            file_path: file_path.to_string_lossy().to_string(),
            churn_score,
            complexity: cyclomatic_complexity as f32,
            duplicate_ratio: 0.0,
            afferent_coupling,
            efferent_coupling: 0.0,
            lines_of_code: *lines_of_code,
            cyclomatic_complexity,
            cognitive_complexity,
        };

        file_metrics.push(metrics);
    }

    file_metrics
}

/// Filter predictions based on configuration
#[must_use]
pub fn filter_predictions(
    predictions: Vec<(String, DefectScore)>,
    config: &DefectPredictionConfig,
) -> Vec<(String, DefectScore)> {
    let mut filtered_predictions = predictions;

    if !config.include_low_confidence {
        filtered_predictions.retain(|(_, score)| score.confidence >= config.confidence_threshold);
    }

    if config.high_risk_only {
        filtered_predictions.retain(|(_, score)| score.probability >= 0.7);
    }

    // Sort by probability (highest first)
    filtered_predictions.sort_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .expect("internal error")
    });

    filtered_predictions
}

/// Calculate risk distribution
pub struct RiskDistribution {
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
}

#[must_use]
pub fn calculate_risk_distribution(predictions: &[(String, DefectScore)]) -> RiskDistribution {
    RiskDistribution {
        high_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability >= 0.7)
            .count(),
        medium_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability >= 0.3 && score.probability < 0.7)
            .count(),
        low_risk_count: predictions
            .iter()
            .filter(|(_, score)| score.probability < 0.3)
            .count(),
    }
}
