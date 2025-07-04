use crate::models::defect_report::{Defect, DefectCategory, Severity};
use crate::services::defect_analyzer::{
    AnalysisContext, AnalysisResult, FileRankingEngine, LineInfo, LineRange, MetricValue,
    RankedFile, SimpleScorer,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Configuration for file ranking
#[derive(Default)]
pub struct RankingConfig {
    /// Maximum number of files to return (0 = all files)
    pub top_files: usize,
    /// Minimum score threshold (files below this are excluded)
    pub min_score: Option<f64>,
}

/// Apply file ranking to analysis results
pub fn apply_file_ranking<T>(
    results: Vec<T>,
    config: &RankingConfig,
    extractor: impl Fn(&T) -> AnalysisResult,
) -> Vec<(T, usize)> {
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
        id: format!("RANK-{:04}", index),
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

/// Helper to create analysis results from common patterns
pub struct AnalysisResultBuilder {
    file_path: PathBuf,
    absolute_path: PathBuf,
    line_start: u32,
    line_end: Option<u32>,
    column_start: u32,
    column_end: Option<u32>,
    metrics: BTreeMap<String, MetricValue>,
    description: String,
    entity_name: Option<String>,
    entity_type: Option<String>,
}

impl AnalysisResultBuilder {
    pub fn new(file_path: PathBuf) -> Self {
        let absolute_path = file_path.clone();
        Self {
            file_path,
            absolute_path,
            line_start: 1,
            line_end: None,
            column_start: 1,
            column_end: None,
            metrics: BTreeMap::new(),
            description: String::new(),
            entity_name: None,
            entity_type: None,
        }
    }

    pub fn with_line_range(mut self, start: u32, end: Option<u32>) -> Self {
        self.line_start = start;
        self.line_end = end;
        self
    }

    pub fn with_column_range(mut self, start: u32, end: Option<u32>) -> Self {
        self.column_start = start;
        self.column_end = end;
        self
    }

    pub fn add_metric(mut self, key: impl Into<String>, value: MetricValue) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    pub fn add_metric_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.metrics.insert(key.into(), MetricValue::Integer(value));
        self
    }

    pub fn add_metric_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), MetricValue::Float(value));
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_entity(mut self, name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        self.entity_name = Some(name.into());
        self.entity_type = Some(entity_type.into());
        self
    }

    pub fn build(self) -> AnalysisResult {
        AnalysisResult {
            file_path: self.file_path,
            absolute_path: self.absolute_path,
            line_range: LineRange {
                start: LineInfo {
                    line: self.line_start,
                    column: self.column_start,
                    byte_offset: 0, // Not computed here
                },
                end: self.line_end.map(|line| LineInfo {
                    line,
                    column: self.column_end.unwrap_or(1),
                    byte_offset: 0,
                }),
            },
            metrics: self.metrics,
            context: AnalysisContext {
                description: self.description,
                entity_name: self.entity_name,
                entity_type: self.entity_type,
            },
        }
    }
}

/// Format a ranked list of files as a table
pub fn format_ranked_files_table(ranked_files: &[RankedFile]) -> String {
    let mut output = String::new();

    // Header
    output.push_str("RANK  SCORE   FILE                                              DEFECTS  CRITICAL  HIGH  MEDIUM  LOW\n");
    output.push_str("----  ------  ------------------------------------------------  -------  --------  ----  ------  ---\n");

    for file in ranked_files {
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for defect in &file.defects {
            match defect.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
            }
        }

        output.push_str(&format!(
            "{:<4}  {:>6.1}  {:<48}  {:>7}  {:>8}  {:>4}  {:>6}  {:>3}\n",
            file.rank,
            file.score,
            file.path
                .display()
                .to_string()
                .chars()
                .take(48)
                .collect::<String>(),
            file.defects.len(),
            critical,
            high,
            medium,
            low
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_result_builder() {
        let result = AnalysisResultBuilder::new(PathBuf::from("test.rs"))
            .with_line_range(10, Some(20))
            .add_metric_int("cyclomatic", 15)
            .add_metric_float("coverage", 85.5)
            .with_description("Complex function")
            .with_entity("process_data", "function")
            .build();

        assert_eq!(result.file_path, PathBuf::from("test.rs"));
        assert_eq!(result.line_range.start.line, 10);
        assert_eq!(result.line_range.end.as_ref().unwrap().line, 20);
        assert_eq!(
            result.metrics.get("cyclomatic"),
            Some(&MetricValue::Integer(15))
        );
        assert_eq!(
            result.metrics.get("coverage"),
            Some(&MetricValue::Float(85.5))
        );
        assert_eq!(result.context.entity_name, Some("process_data".to_string()));
    }

    #[test]
    fn test_severity_computation() {
        let mut metrics = BTreeMap::new();

        // High complexity
        metrics.insert("cyclomatic".to_string(), MetricValue::Integer(55));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Critical);

        // Medium complexity
        metrics.clear();
        metrics.insert("cognitive_complexity".to_string(), MetricValue::Float(15.0));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Medium);

        // Low complexity
        metrics.clear();
        metrics.insert("complexity".to_string(), MetricValue::Integer(5));
        assert_eq!(compute_severity_from_metrics(&metrics), Severity::Low);
    }
}
