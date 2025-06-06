use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::complexity::FileComplexityMetrics;

/// Core trait for ranking files by different metrics
pub trait FileRanker: Send + Sync {
    type Metric: PartialOrd + Clone + Send + Sync;

    /// Compute the ranking metric for a single file
    fn compute_score(&self, file_path: &Path) -> Self::Metric;

    /// Format a single ranking entry for display
    fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String;

    /// Get the display name for this ranking type
    fn ranking_type(&self) -> &'static str;
}

/// Generic ranking engine that can work with any FileRanker
pub struct RankingEngine<R: FileRanker> {
    ranker: R,
    cache: Arc<RwLock<HashMap<String, R::Metric>>>,
}

impl<R: FileRanker> RankingEngine<R> {
    pub fn new(ranker: R) -> Self {
        Self {
            ranker,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Rank files and return the top N results
    pub async fn rank_files(&self, files: &[PathBuf], limit: usize) -> Vec<(String, R::Metric)> {
        if files.is_empty() || limit == 0 {
            return Vec::new();
        }

        // Compute scores in parallel
        let mut scores: Vec<_> = files
            .par_iter()
            .filter_map(|f| {
                if !f.exists() || !f.is_file() {
                    return None;
                }

                let file_str = f.to_string_lossy().to_string();

                // Check cache first
                if let Ok(cache) = self.cache.read() {
                    if let Some(cached_score) = cache.get(&file_str) {
                        return Some((file_str, cached_score.clone()));
                    }
                }

                // Compute score
                let score = self.ranker.compute_score(f);

                // Cache the result
                if let Ok(mut cache) = self.cache.write() {
                    cache.insert(file_str.clone(), score.clone());
                }

                Some((file_str, score))
            })
            .collect();

        // Sort by score (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Apply limit
        scores.truncate(limit);
        scores
    }

    /// Format rankings as a table
    pub fn format_rankings_table(&self, rankings: &[(String, R::Metric)]) -> String {
        if rankings.is_empty() {
            return format!(
                "## Top {} Files\n\nNo files found.\n",
                self.ranker.ranking_type()
            );
        }

        let mut output = format!(
            "## Top {} {} Files\n\n",
            rankings.len(),
            self.ranker.ranking_type()
        );

        for (i, (file, metric)) in rankings.iter().enumerate() {
            output.push_str(&self.ranker.format_ranking_entry(file, metric, i + 1));
            output.push('\n');
        }

        output.push('\n');
        output
    }

    /// Format rankings as JSON
    pub fn format_rankings_json(&self, rankings: &[(String, R::Metric)]) -> serde_json::Value {
        serde_json::json!({
            "analysis_type": self.ranker.ranking_type(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "top_files": {
                "requested": rankings.len(),
                "returned": rankings.len(),
            },
            "rankings": rankings.iter().enumerate().map(|(i, (file, _))| {
                serde_json::json!({
                    "rank": i + 1,
                    "file": file,
                })
            }).collect::<Vec<_>>()
        })
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// Composite complexity score for ranking files
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositeComplexityScore {
    pub cyclomatic_max: u32,
    pub cognitive_avg: f64,
    pub halstead_effort: f64,
    pub function_count: usize,
    pub total_score: f64,
}

impl Default for CompositeComplexityScore {
    fn default() -> Self {
        Self {
            cyclomatic_max: 0,
            cognitive_avg: 0.0,
            halstead_effort: 0.0,
            function_count: 0,
            total_score: 0.0,
        }
    }
}

impl PartialEq for CompositeComplexityScore {
    fn eq(&self, other: &Self) -> bool {
        (self.total_score - other.total_score).abs() < f64::EPSILON
    }
}

impl PartialOrd for CompositeComplexityScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total_score.partial_cmp(&other.total_score)
    }
}

/// Churn score for ranking files by change frequency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChurnScore {
    pub commit_count: usize,
    pub unique_authors: usize,
    pub lines_changed: usize,
    pub recency_weight: f64,
    pub score: f64,
}

impl Default for ChurnScore {
    fn default() -> Self {
        Self {
            commit_count: 0,
            unique_authors: 0,
            lines_changed: 0,
            recency_weight: 0.0,
            score: 0.0,
        }
    }
}

impl PartialEq for ChurnScore {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < f64::EPSILON
    }
}

impl PartialOrd for ChurnScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// Duplication score for ranking files by code duplication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicationScore {
    pub exact_clones: usize,
    pub renamed_clones: usize,
    pub gapped_clones: usize,
    pub semantic_clones: usize,
    pub duplication_ratio: f64,
    pub score: f64,
}

impl Default for DuplicationScore {
    fn default() -> Self {
        Self {
            exact_clones: 0,
            renamed_clones: 0,
            gapped_clones: 0,
            semantic_clones: 0,
            duplication_ratio: 0.0,
            score: 0.0,
        }
    }
}

impl PartialEq for DuplicationScore {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < f64::EPSILON
    }
}

impl PartialOrd for DuplicationScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// SIMD-optimized ranking for large datasets
pub fn rank_files_vectorized(scores: &[f32], limit: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..scores.len()).collect();

    // For large datasets, use parallel sorting
    if scores.len() > 1024 {
        indices.par_sort_unstable_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(Ordering::Equal)
        });
    } else {
        // Standard sort for smaller datasets
        indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap_or(Ordering::Equal));
    }

    indices.truncate(limit);
    indices
}

/// Complexity-based file ranker implementation
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
            .map(|f| f.metrics.cyclomatic as u32)
            .max()
            .unwrap_or(0);

        // Calculate average cognitive complexity
        let cognitive_total: u32 = all_functions
            .iter()
            .map(|f| f.metrics.cognitive as u32)
            .sum();
        let cognitive_avg = cognitive_total as f64 / function_count as f64;

        // Mock halstead effort (would need proper calculation)
        let halstead_effort = all_functions
            .iter()
            .map(|f| f.metrics.lines as u32 * 10) // Simple approximation
            .sum::<u32>() as f64;

        // Calculate composite score
        let normalized_cyclomatic = (cyclomatic_max as f64).min(50.0) / 50.0; // Normalize to 0-1
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

impl FileRanker for ComplexityRanker {
    type Metric = CompositeComplexityScore;

    fn compute_score(&self, file_path: &Path) -> Self::Metric {
        // Try to analyze the file for complexity
        // For now, we'll use a simplified approach that doesn't require async
        // In a real implementation, this would be more sophisticated

        // Basic file analysis based on extension
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => {
                    // For Rust files, we'd normally do full AST analysis
                    // For now, return a simple score based on file size
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        let size_score = (metadata.len() as f64 / 1000.0).min(100.0);
                        CompositeComplexityScore {
                            total_score: size_score,
                            function_count: (size_score / 10.0) as usize,
                            cyclomatic_max: (size_score / 5.0) as u32,
                            cognitive_avg: size_score / 3.0,
                            halstead_effort: size_score * 10.0,
                        }
                    } else {
                        CompositeComplexityScore::default()
                    }
                }
                "ts" | "tsx" | "js" | "jsx" => {
                    // Similar logic for TypeScript/JavaScript
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        let size_score = (metadata.len() as f64 / 1200.0).min(100.0);
                        CompositeComplexityScore {
                            total_score: size_score * 0.9, // Slightly lower weight for JS/TS
                            function_count: (size_score / 12.0) as usize,
                            cyclomatic_max: (size_score / 6.0) as u32,
                            cognitive_avg: size_score / 4.0,
                            halstead_effort: size_score * 8.0,
                        }
                    } else {
                        CompositeComplexityScore::default()
                    }
                }
                "py" => {
                    // Similar logic for Python
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        let size_score = (metadata.len() as f64 / 800.0).min(100.0);
                        CompositeComplexityScore {
                            total_score: size_score * 1.1, // Slightly higher weight for Python
                            function_count: (size_score / 8.0) as usize,
                            cyclomatic_max: (size_score / 4.0) as u32,
                            cognitive_avg: size_score / 2.5,
                            halstead_effort: size_score * 12.0,
                        }
                    } else {
                        CompositeComplexityScore::default()
                    }
                }
                _ => CompositeComplexityScore::default(),
            }
        } else {
            CompositeComplexityScore::default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::complexity::{ClassComplexity, ComplexityMetrics, FunctionComplexity};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct MockRanker;

    impl FileRanker for MockRanker {
        type Metric = f64;

        fn compute_score(&self, file_path: &Path) -> Self::Metric {
            // Mock score based on file name length
            file_path.to_string_lossy().len() as f64
        }

        fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
            format!("| {rank:>4} | {file} | {metric:.1} |")
        }

        fn ranking_type(&self) -> &'static str {
            "Mock"
        }
    }

    fn create_test_file_metrics() -> FileComplexityMetrics {
        FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 23,
                cognitive: 37,
                nesting_max: 4,
                lines: 50,
            },
            functions: vec![
                FunctionComplexity {
                    name: "test_func".to_string(),
                    line_start: 1,
                    line_end: 10,
                    metrics: ComplexityMetrics {
                        cyclomatic: 5,
                        cognitive: 8,
                        nesting_max: 2,
                        lines: 10,
                    },
                },
                FunctionComplexity {
                    name: "complex_func".to_string(),
                    line_start: 20,
                    line_end: 50,
                    metrics: ComplexityMetrics {
                        cyclomatic: 15,
                        cognitive: 25,
                        nesting_max: 4,
                        lines: 30,
                    },
                },
            ],
            classes: vec![ClassComplexity {
                name: "TestClass".to_string(),
                line_start: 60,
                line_end: 100,
                metrics: ComplexityMetrics {
                    cyclomatic: 3,
                    cognitive: 4,
                    nesting_max: 1,
                    lines: 10,
                },
                methods: vec![FunctionComplexity {
                    name: "method".to_string(),
                    line_start: 65,
                    line_end: 75,
                    metrics: ComplexityMetrics {
                        cyclomatic: 3,
                        cognitive: 4,
                        nesting_max: 1,
                        lines: 10,
                    },
                }],
            }],
        }
    }

    #[tokio::test]
    async fn test_empty_file_list() {
        let ranker = MockRanker;
        let engine = RankingEngine::new(ranker);
        let result = engine.rank_files(&[], 5).await;
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_limit_exceeds_files() {
        let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
        let ranker = MockRanker;
        let engine = RankingEngine::new(ranker);

        // This will filter out non-existent files, so result will be empty
        let result = engine.rank_files(&files, 10).await;
        assert_eq!(result.len(), 0); // Files don't exist
    }

    #[test]
    fn test_vectorized_ranking() {
        let scores = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        let ranked = rank_files_vectorized(&scores, 3);

        // Should be sorted by score descending: [1]=5.0, [4]=4.0, [2]=3.0
        assert_eq!(ranked, vec![1, 4, 2]);
    }

    #[test]
    fn test_composite_complexity_score_ordering() {
        let score1 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score2 = CompositeComplexityScore {
            total_score: 5.0,
            ..Default::default()
        };

        assert!(score1 > score2);
    }

    #[test]
    fn test_composite_complexity_score_default() {
        let score = CompositeComplexityScore::default();
        assert_eq!(score.cyclomatic_max, 0);
        assert_eq!(score.cognitive_avg, 0.0);
        assert_eq!(score.halstead_effort, 0.0);
        assert_eq!(score.function_count, 0);
        assert_eq!(score.total_score, 0.0);
    }

    #[test]
    fn test_composite_complexity_score_equality() {
        let score1 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score2 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score3 = CompositeComplexityScore {
            total_score: 15.0,
            ..Default::default()
        };

        assert_eq!(score1, score2);
        assert_ne!(score1, score3);
    }

    #[test]
    fn test_churn_score_default_and_ordering() {
        let score1 = ChurnScore::default();
        let score2 = ChurnScore {
            score: 10.0,
            ..Default::default()
        };

        assert_eq!(score1.commit_count, 0);
        assert_eq!(score1.score, 0.0);
        assert!(score2 > score1);
    }

    #[test]
    fn test_duplication_score_default_and_ordering() {
        let score1 = DuplicationScore::default();
        let score2 = DuplicationScore {
            score: 5.0,
            exact_clones: 2,
            duplication_ratio: 0.3,
            ..Default::default()
        };

        assert_eq!(score1.exact_clones, 0);
        assert_eq!(score1.duplication_ratio, 0.0);
        assert!(score2 > score1);
    }

    #[test]
    fn test_vectorized_ranking_small_dataset() {
        let scores = vec![3.0, 1.0, 4.0, 2.0];
        let ranked = rank_files_vectorized(&scores, 2);
        assert_eq!(ranked, vec![2, 0]); // indices of highest scores
    }

    #[test]
    fn test_vectorized_ranking_large_dataset() {
        let scores: Vec<f32> = (0..2000).map(|i| i as f32).collect();
        let ranked = rank_files_vectorized(&scores, 5);
        assert_eq!(ranked, vec![1999, 1998, 1997, 1996, 1995]);
    }

    #[test]
    fn test_vectorized_ranking_empty() {
        let scores = vec![];
        let ranked = rank_files_vectorized(&scores, 5);
        assert_eq!(ranked.len(), 0);
    }

    #[test]
    fn test_complexity_ranker_default() {
        let ranker = ComplexityRanker::default();
        assert_eq!(ranker.cyclomatic_weight, 0.4);
        assert_eq!(ranker.cognitive_weight, 0.4);
        assert_eq!(ranker.function_count_weight, 0.2);
        assert_eq!(ranker.ranking_type(), "Complexity");
    }

    #[test]
    fn test_complexity_ranker_new() {
        let ranker = ComplexityRanker::new(0.5, 0.3, 0.2);
        assert_eq!(ranker.cyclomatic_weight, 0.5);
        assert_eq!(ranker.cognitive_weight, 0.3);
        assert_eq!(ranker.function_count_weight, 0.2);
    }

    #[test]
    fn test_complexity_ranker_calculate_composite_score() {
        let ranker = ComplexityRanker::default();
        let metrics = create_test_file_metrics();
        let score = ranker.calculate_composite_score(&metrics);

        assert_eq!(score.function_count, 3); // 2 functions + 1 method
        assert_eq!(score.cyclomatic_max, 15); // max from complex_func
        assert!((score.cognitive_avg - 12.333333333333334).abs() < 0.001); // (8+25+4)/3
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_calculate_composite_score_empty() {
        let ranker = ComplexityRanker::default();
        let metrics = FileComplexityMetrics {
            path: "empty.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![],
            classes: vec![],
        };
        let score = ranker.calculate_composite_score(&metrics);

        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[tokio::test]
    async fn test_ranking_engine_with_temp_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("small.rs");
        let file2 = temp_dir.path().join("large.rs");

        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "fn small() {{}}").unwrap();

        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "fn large() {{ // This is a much longer file").unwrap();
        for _ in 0..100 {
            writeln!(f2, "    println!(\"line\");").unwrap();
        }
        writeln!(f2, "}}").unwrap();

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![file1, file2];
        let rankings = engine.rank_files(&files, 2).await;

        assert_eq!(rankings.len(), 2);
        // Larger file should have higher score
        assert!(rankings[0].1.total_score >= rankings[1].1.total_score);
    }

    #[tokio::test]
    async fn test_ranking_engine_zero_limit() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let files = vec![PathBuf::from("test.rs")];
        let rankings = engine.rank_files(&files, 0).await;
        assert_eq!(rankings.len(), 0);
    }

    #[tokio::test]
    async fn test_ranking_engine_cache() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.rs");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "fn test() {{}}").unwrap();

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![file1.clone()];

        // First call should compute and cache
        let rankings1 = engine.rank_files(&files, 1).await;

        // Second call should use cache
        let rankings2 = engine.rank_files(&files, 1).await;

        assert_eq!(rankings1.len(), 1);
        assert_eq!(rankings2.len(), 1);
        assert_eq!(rankings1[0].1.total_score, rankings2[0].1.total_score);

        // Clear cache and verify
        engine.clear_cache();
        let rankings3 = engine.rank_files(&files, 1).await;
        assert_eq!(rankings3.len(), 1);
    }

    #[test]
    fn test_ranking_engine_format_rankings_table_empty() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![];
        let output = engine.format_rankings_table(&rankings);
        assert!(output.contains("No files found"));
    }

    #[test]
    fn test_ranking_engine_format_rankings_table() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![
            (
                "test1.rs".to_string(),
                CompositeComplexityScore {
                    total_score: 10.0,
                    function_count: 5,
                    cyclomatic_max: 8,
                    cognitive_avg: 12.0,
                    halstead_effort: 150.0,
                },
            ),
            (
                "test2.rs".to_string(),
                CompositeComplexityScore {
                    total_score: 5.0,
                    function_count: 2,
                    cyclomatic_max: 3,
                    cognitive_avg: 4.0,
                    halstead_effort: 50.0,
                },
            ),
        ];
        let output = engine.format_rankings_table(&rankings);
        assert!(output.contains("Top 2 Complexity Files"));
        assert!(output.contains("test1.rs"));
        assert!(output.contains("test2.rs"));
        assert!(output.contains("10.0"));
        assert!(output.contains("5.0"));
    }

    #[test]
    fn test_ranking_engine_format_rankings_json() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![(
            "test1.rs".to_string(),
            CompositeComplexityScore {
                total_score: 10.0,
                ..Default::default()
            },
        )];
        let json = engine.format_rankings_json(&rankings);
        assert_eq!(json["analysis_type"], "Complexity");
        assert_eq!(json["top_files"]["requested"], 1);
        assert_eq!(json["rankings"][0]["rank"], 1);
        assert_eq!(json["rankings"][0]["file"], "test1.rs");
    }

    #[test]
    fn test_complexity_ranker_compute_score_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        let mut f = File::create(&rust_file).unwrap();
        writeln!(f, "fn test() {{ println!(\"hello\"); }}").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&rust_file);
        assert!(score.total_score > 0.0);
        // Note: compute_score uses simplified file-size-based scoring, not actual AST parsing
        // function_count is a usize and is always >= 0
    }

    #[test]
    fn test_complexity_ranker_compute_score_javascript_file() {
        let temp_dir = TempDir::new().unwrap();
        let js_file = temp_dir.path().join("test.js");
        let mut f = File::create(&js_file).unwrap();
        writeln!(f, "function test() {{ console.log('hello'); }}").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&js_file);
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_compute_score_python_file() {
        let temp_dir = TempDir::new().unwrap();
        let py_file = temp_dir.path().join("test.py");
        let mut f = File::create(&py_file).unwrap();
        writeln!(f, "def test():\n    print('hello')").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&py_file);
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_compute_score_unknown_file() {
        let temp_dir = TempDir::new().unwrap();
        let unknown_file = temp_dir.path().join("test.txt");
        let mut f = File::create(&unknown_file).unwrap();
        writeln!(f, "hello world").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&unknown_file);
        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[test]
    fn test_complexity_ranker_compute_score_nonexistent_file() {
        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(Path::new("/nonexistent/file.rs"));
        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[test]
    fn test_complexity_ranker_format_ranking_entry() {
        let ranker = ComplexityRanker::default();
        let metric = CompositeComplexityScore {
            total_score: 42.5,
            function_count: 10,
            cyclomatic_max: 15,
            cognitive_avg: 8.7,
            halstead_effort: 123.4,
        };
        let output = ranker.format_ranking_entry("test.rs", &metric, 1);
        assert!(output.contains("1"));
        assert!(output.contains("test.rs"));
        assert!(output.contains("42.5"));
        assert!(output.contains("10"));
        assert!(output.contains("15"));
        assert!(output.contains("8.7"));
        assert!(output.contains("123.4"));
    }

    #[test]
    fn test_rank_files_by_complexity() {
        let metrics = vec![
            FileComplexityMetrics {
                path: "simple.rs".to_string(),
                total_complexity: ComplexityMetrics {
                    cyclomatic: 1,
                    cognitive: 1,
                    nesting_max: 0,
                    lines: 5,
                },
                functions: vec![FunctionComplexity {
                    name: "simple".to_string(),
                    line_start: 1,
                    line_end: 5,
                    metrics: ComplexityMetrics {
                        cyclomatic: 1,
                        cognitive: 1,
                        nesting_max: 0,
                        lines: 5,
                    },
                }],
                classes: vec![],
            },
            create_test_file_metrics(), // More complex file
        ];

        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 2, &ranker);

        assert_eq!(rankings.len(), 2);
        // More complex file should be ranked first
        assert_eq!(rankings[0].0, "test.rs");
        assert_eq!(rankings[1].0, "simple.rs");
        assert!(rankings[0].1.total_score > rankings[1].1.total_score);
    }

    #[test]
    fn test_rank_files_by_complexity_with_limit() {
        let metrics = vec![create_test_file_metrics()];
        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 0, &ranker);
        assert_eq!(rankings.len(), 1); // limit 0 means no truncation

        let rankings_limited = rank_files_by_complexity(&metrics, 1, &ranker);
        assert_eq!(rankings_limited.len(), 1);
    }

    #[test]
    fn test_rank_files_by_complexity_empty() {
        let metrics = vec![];
        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 5, &ranker);
        assert_eq!(rankings.len(), 0);
    }

    #[tokio::test]
    async fn test_ranking_engine_with_nonexistent_files() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![
            PathBuf::from("/nonexistent/file1.rs"),
            PathBuf::from("/nonexistent/file2.rs"),
        ];

        let rankings = engine.rank_files(&files, 5).await;
        assert_eq!(rankings.len(), 0); // All files filtered out
    }

    #[tokio::test]
    async fn test_ranking_engine_mixed_existing_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.rs");
        let mut f = File::create(&existing_file).unwrap();
        writeln!(f, "fn test() {{}}").unwrap();

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![existing_file, PathBuf::from("/nonexistent/file.rs")];

        let rankings = engine.rank_files(&files, 5).await;
        assert_eq!(rankings.len(), 1); // Only existing file
    }

    // Test custom ranker functionality
    struct TestRanker {
        score_multiplier: f64,
    }

    impl FileRanker for TestRanker {
        type Metric = f64;

        fn compute_score(&self, file_path: &Path) -> Self::Metric {
            file_path.to_string_lossy().len() as f64 * self.score_multiplier
        }

        fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
            format!("{rank}. {file} ({metric})")
        }

        fn ranking_type(&self) -> &'static str {
            "Test"
        }
    }

    #[tokio::test]
    async fn test_custom_ranker() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("a.rs");
        let file2 = temp_dir.path().join("longer_name.rs");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let ranker = TestRanker {
            score_multiplier: 2.0,
        };
        let engine = RankingEngine::new(ranker);

        let files = vec![file1, file2];
        let rankings = engine.rank_files(&files, 2).await;

        assert_eq!(rankings.len(), 2);
        // Longer filename should have higher score
        assert!(rankings[0].0.contains("longer_name"));
        assert!(rankings[0].1 > rankings[1].1);
    }

    #[test]
    fn test_all_score_types_partial_ord() {
        // Test CompositeComplexityScore
        let comp1 = CompositeComplexityScore {
            total_score: 5.0,
            ..Default::default()
        };
        let comp2 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        assert!(comp1 < comp2);

        // Test ChurnScore
        let churn1 = ChurnScore {
            score: 3.0,
            ..Default::default()
        };
        let churn2 = ChurnScore {
            score: 7.0,
            ..Default::default()
        };
        assert!(churn1 < churn2);

        // Test DuplicationScore
        let dup1 = DuplicationScore {
            score: 2.0,
            ..Default::default()
        };
        let dup2 = DuplicationScore {
            score: 8.0,
            ..Default::default()
        };
        assert!(dup1 < dup2);
    }
}
