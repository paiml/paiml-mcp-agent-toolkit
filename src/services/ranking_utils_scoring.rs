/// Apply file ranking to analysis results
pub fn apply_file_ranking<T>(
    results: Vec<T>,
    config: &RankingConfig,
    extractor: impl Fn(&T) -> AnalysisResult,
) -> Vec<(T, usize)> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    if config.top_files == 0 && config.min_score.is_none() {
        // No ranking needed, return all with rank 1
        return results
            .into_iter()
            .enumerate()
            .map(|(i, r)| (r, i + 1))
            .collect();
    }

    // Convert to defects for ranking
    let defects: Vec<Defect> = results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let analysis = extractor(result);
            result_to_defect(&analysis, i)
        })
        .collect();

    // Rank files
    let engine = FileRankingEngine::new(Box::new(SimpleScorer));
    let ranked = engine.rank_files(defects, config.top_files);

    // Create a mapping of file paths to ranks
    let rank_map: BTreeMap<PathBuf, usize> = ranked.into_iter().map(|r| (r.path, r.rank)).collect();

    // Filter and sort results based on ranking
    let mut ranked_results: Vec<(T, usize)> = results
        .into_iter()
        .filter_map(|result| {
            let analysis = extractor(&result);
            rank_map
                .get(&analysis.file_path)
                .map(|&rank| (result, rank))
        })
        .collect();

    // Sort by rank
    ranked_results.sort_by_key(|(_, rank)| *rank);

    ranked_results
}

/// Convert an analysis result to a defect for ranking purposes
fn result_to_defect(result: &AnalysisResult, index: usize) -> Defect {
    // Compute severity based on metrics
    let severity = compute_severity_from_metrics(&result.metrics);

    // Build metrics HashMap
    let mut metrics = std::collections::HashMap::new();
    for (key, value) in &result.metrics {
        if let MetricValue::Float(f) = value {
            metrics.insert(key.clone(), *f);
        } else if let MetricValue::Integer(i) = value {
            metrics.insert(key.clone(), *i as f64);
        }
    }

    Defect {
        id: format!("RANK-{index:04}"),
        severity,
        category: DefectCategory::Complexity, // Default category for ranking
        file_path: result.file_path.clone(),
        line_start: result.line_range.start.line,
        line_end: result.line_range.end.as_ref().map(|e| e.line),
        column_start: Some(result.line_range.start.column),
        column_end: result.line_range.end.as_ref().map(|e| e.column),
        message: result.context.description.clone(),
        rule_id: "ranking".to_string(),
        fix_suggestion: None,
        metrics,
    }
}

/// Compute severity from metrics for ranking
fn compute_severity_from_metrics(metrics: &BTreeMap<String, MetricValue>) -> Severity {
    // Look for common complexity metrics
    let complexity_score = metrics
        .iter()
        .filter_map(|(k, v)| {
            if k.contains("complexity") || k.contains("cyclomatic") || k.contains("cognitive") {
                match v {
                    MetricValue::Integer(i) => Some(*i as f64),
                    MetricValue::Float(f) => Some(*f),
                    _ => None,
                }
            } else {
                None
            }
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    if complexity_score > 50.0 {
        Severity::Critical
    } else if complexity_score > 20.0 {
        Severity::High
    } else if complexity_score > 10.0 {
        Severity::Medium
    } else {
        Severity::Low
    }
}
