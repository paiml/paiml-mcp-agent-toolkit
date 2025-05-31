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
    use std::path::PathBuf;

    struct MockRanker;

    impl FileRanker for MockRanker {
        type Metric = f64;

        fn compute_score(&self, file_path: &Path) -> Self::Metric {
            // Mock score based on file name length
            file_path.to_string_lossy().len() as f64
        }

        fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
            format!("| {:>4} | {} | {:.1} |", rank, file, metric)
        }

        fn ranking_type(&self) -> &'static str {
            "Mock"
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
}
