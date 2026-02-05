// Reciprocal Rank Fusion (GH-RAG-006)
// Toyota Way: Jidoka - Automation with quality built-in
// Citation: [SIGIR-2022] SPLADE v2 - Sparse Lexical and Expansion Model
// Spec: docs/specifications/git-history-rag-integration.md

use std::collections::HashMap;

/// Reciprocal Rank Fusion (RRF) implementation
/// Formula: score(d) = Σ 1/(k + rank(d))
/// Citation: [SIGIR-2022] Formal, T., et al. "SPLADE v2"
pub struct RrfFusion {
    /// Constant k in RRF formula (default: 60)
    /// Higher k values smooth out the impact of rank differences
    k: f32,
}

/// A document with a ranking score
#[derive(Debug, Clone)]
pub struct RankedDocument {
    /// Unique identifier (file_path:function_name or commit_hash)
    pub id: String,
    /// Original relevance score (for display)
    pub original_score: f32,
    /// Source of this result ("code" or "git")
    pub source: String,
    /// Additional metadata
    pub metadata: DocumentMetadata,
}

/// Metadata carried through fusion
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// File path (for code) or commit hash (for git)
    pub path: String,
    /// Function name (for code) or commit subject (for git)
    pub name: String,
    /// Line number (for code) or timestamp (for git as i64)
    pub line_or_timestamp: i64,
    /// Related commit hashes (for code results enriched with git)
    pub related_commits: Vec<String>,
}

/// Result of RRF fusion
#[derive(Debug, Clone)]
pub struct FusedResult {
    /// Document identifier
    pub id: String,
    /// Combined RRF score
    pub rrf_score: f32,
    /// Original scores from each source
    pub source_scores: HashMap<String, f32>,
    /// Metadata from the highest-scoring source
    pub metadata: DocumentMetadata,
    /// Source that contributed most
    pub primary_source: String,
}

impl Default for RrfFusion {
    fn default() -> Self {
        Self::new()
    }
}

impl RrfFusion {
    /// Create a new RRF fusion instance with default k=60
    pub fn new() -> Self {
        Self { k: 60.0 }
    }

    /// Create with custom k value
    pub fn with_k(k: f32) -> Self {
        Self { k }
    }

    /// Fuse multiple ranked lists using RRF
    ///
    /// # Arguments
    /// * `lists` - Vec of (source_name, ranked_documents)
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Fused results sorted by RRF score
    pub fn fuse(&self, lists: Vec<(&str, Vec<RankedDocument>)>, limit: usize) -> Vec<FusedResult> {
        let mut scores: HashMap<String, FusedResultBuilder> = HashMap::new();

        // Process each ranked list
        for (source, docs) in lists {
            for (rank, doc) in docs.into_iter().enumerate() {
                let rrf_contribution = 1.0 / (self.k + (rank as f32) + 1.0);

                let entry = scores.entry(doc.id.clone()).or_insert_with(|| FusedResultBuilder {
                    id: doc.id.clone(),
                    total_rrf: 0.0,
                    source_scores: HashMap::new(),
                    best_metadata: doc.metadata.clone(),
                    best_score: 0.0,
                    primary_source: source.to_string(),
                });

                entry.total_rrf += rrf_contribution;
                entry.source_scores.insert(source.to_string(), doc.original_score);

                // Track best source for metadata
                if doc.original_score > entry.best_score {
                    entry.best_score = doc.original_score;
                    entry.best_metadata = doc.metadata;
                    entry.primary_source = source.to_string();
                }
            }
        }

        // Convert to results and sort by RRF score
        let mut results: Vec<FusedResult> = scores
            .into_values()
            .map(|b| FusedResult {
                id: b.id,
                rrf_score: b.total_rrf,
                source_scores: b.source_scores,
                metadata: b.best_metadata,
                primary_source: b.primary_source,
            })
            .collect();

        results.sort_by(|a, b| b.rrf_score.partial_cmp(&a.rrf_score).unwrap_or(std::cmp::Ordering::Equal));

        results.truncate(limit);
        results
    }

    /// Calculate RRF improvement over single-source search
    /// Returns: (improvement_ratio, baseline_mrr, fused_mrr)
    pub fn calculate_improvement(
        &self,
        fused_results: &[FusedResult],
        primary_results: &[RankedDocument],
        ground_truth_ids: &[String],
    ) -> (f32, f32, f32) {
        let primary_mrr = Self::mean_reciprocal_rank(
            &primary_results.iter().map(|d| d.id.clone()).collect::<Vec<_>>(),
            ground_truth_ids,
        );

        let fused_mrr = Self::mean_reciprocal_rank(
            &fused_results.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            ground_truth_ids,
        );

        let improvement = if primary_mrr > 0.0 {
            (fused_mrr - primary_mrr) / primary_mrr
        } else if fused_mrr > 0.0 {
            1.0 // Infinite improvement
        } else {
            0.0
        };

        (improvement, primary_mrr, fused_mrr)
    }

    /// Calculate Mean Reciprocal Rank (MRR)
    fn mean_reciprocal_rank(results: &[String], ground_truth: &[String]) -> f32 {
        if results.is_empty() || ground_truth.is_empty() {
            return 0.0;
        }

        let mut total_rr = 0.0;
        let mut count = 0;

        for truth in ground_truth {
            for (rank, result) in results.iter().enumerate() {
                if result == truth {
                    total_rr += 1.0 / (rank as f32 + 1.0);
                    count += 1;
                    break;
                }
            }
        }

        if count > 0 {
            total_rr / count as f32
        } else {
            0.0
        }
    }
}

/// Builder for accumulating RRF scores
struct FusedResultBuilder {
    id: String,
    total_rrf: f32,
    source_scores: HashMap<String, f32>,
    best_metadata: DocumentMetadata,
    best_score: f32,
    primary_source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(id: &str, score: f32, source: &str) -> RankedDocument {
        RankedDocument {
            id: id.to_string(),
            original_score: score,
            source: source.to_string(),
            metadata: DocumentMetadata {
                path: id.to_string(),
                name: id.to_string(),
                line_or_timestamp: 0,
                related_commits: vec![],
            },
        }
    }

    #[test]
    fn test_rrf_basic_fusion() {
        let fusion = RrfFusion::new();

        let code_results = vec![
            make_doc("func_a", 0.9, "code"),
            make_doc("func_b", 0.8, "code"),
            make_doc("func_c", 0.7, "code"),
        ];

        let git_results = vec![
            make_doc("func_b", 0.95, "git"),  // Higher in git
            make_doc("func_a", 0.85, "git"),
            make_doc("func_d", 0.75, "git"),  // Only in git
        ];

        let results = fusion.fuse(
            vec![("code", code_results), ("git", git_results)],
            10,
        );

        // func_a and func_b should rank highest (present in both)
        assert!(!results.is_empty());

        // Documents in both lists should have higher scores
        let top_ids: Vec<&str> = results.iter().take(2).map(|r| r.id.as_str()).collect();
        assert!(top_ids.contains(&"func_a") || top_ids.contains(&"func_b"));
    }

    #[test]
    fn test_rrf_single_source() {
        let fusion = RrfFusion::new();

        let code_results = vec![
            make_doc("func_a", 0.9, "code"),
            make_doc("func_b", 0.8, "code"),
        ];

        let results = fusion.fuse(vec![("code", code_results)], 10);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "func_a"); // Highest ranked
        assert_eq!(results[1].id, "func_b");
    }

    #[test]
    fn test_rrf_preserves_metadata() {
        let fusion = RrfFusion::new();

        let code_results = vec![
            RankedDocument {
                id: "func_a".to_string(),
                original_score: 0.9,
                source: "code".to_string(),
                metadata: DocumentMetadata {
                    path: "src/main.rs".to_string(),
                    name: "func_a".to_string(),
                    line_or_timestamp: 42,
                    related_commits: vec![],
                },
            },
        ];

        let results = fusion.fuse(vec![("code", code_results)], 10);

        assert_eq!(results[0].metadata.path, "src/main.rs");
        assert_eq!(results[0].metadata.line_or_timestamp, 42);
    }

    #[test]
    fn test_rrf_tracks_sources() {
        let fusion = RrfFusion::new();

        let code_results = vec![make_doc("func_a", 0.9, "code")];
        let git_results = vec![make_doc("func_a", 0.8, "git")];

        let results = fusion.fuse(
            vec![("code", code_results), ("git", git_results)],
            10,
        );

        assert!(results[0].source_scores.contains_key("code"));
        assert!(results[0].source_scores.contains_key("git"));
    }

    #[test]
    fn test_rrf_empty_lists() {
        let fusion = RrfFusion::new();

        let results = fusion.fuse(vec![], 10);
        assert!(results.is_empty());

        let results = fusion.fuse(vec![("code", vec![])], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_respects_limit() {
        let fusion = RrfFusion::new();

        let code_results: Vec<RankedDocument> = (0..20)
            .map(|i| make_doc(&format!("func_{}", i), 1.0 - i as f32 * 0.01, "code"))
            .collect();

        let results = fusion.fuse(vec![("code", code_results)], 5);

        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_rrf_custom_k() {
        let fusion_low_k = RrfFusion::with_k(10.0);
        let fusion_high_k = RrfFusion::with_k(100.0);

        let code_results = vec![
            make_doc("func_a", 0.9, "code"),
            make_doc("func_b", 0.1, "code"),
        ];

        let low_k_results = fusion_low_k.fuse(vec![("code", code_results.clone())], 10);
        let high_k_results = fusion_high_k.fuse(vec![("code", code_results)], 10);

        // Lower k amplifies rank differences
        let low_k_diff = low_k_results[0].rrf_score - low_k_results[1].rrf_score;
        let high_k_diff = high_k_results[0].rrf_score - high_k_results[1].rrf_score;

        assert!(low_k_diff > high_k_diff, "Lower k should amplify rank differences");
    }

    #[test]
    fn test_mrr_calculation() {
        // Perfect ranking
        let results = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let truth = vec!["a".to_string()];
        let mrr = RrfFusion::mean_reciprocal_rank(&results, &truth);
        assert!((mrr - 1.0).abs() < 0.001, "First match should have MRR=1.0");

        // Second position
        let truth = vec!["b".to_string()];
        let mrr = RrfFusion::mean_reciprocal_rank(&results, &truth);
        assert!((mrr - 0.5).abs() < 0.001, "Second match should have MRR=0.5");

        // Not found
        let truth = vec!["z".to_string()];
        let mrr = RrfFusion::mean_reciprocal_rank(&results, &truth);
        assert!((mrr - 0.0).abs() < 0.001, "No match should have MRR=0.0");
    }

    #[test]
    fn test_improvement_calculation() {
        let fusion = RrfFusion::new();

        // Primary results: ground truth at position 3
        let primary = vec![
            make_doc("a", 0.9, "code"),
            make_doc("b", 0.8, "code"),
            make_doc("target", 0.7, "code"),  // Ground truth
        ];

        // Fused results: ground truth at position 1
        let fused = fusion.fuse(
            vec![
                ("code", primary.clone()),
                ("git", vec![make_doc("target", 0.95, "git")]),  // Boost target
            ],
            10,
        );

        let ground_truth = vec!["target".to_string()];
        let (improvement, _, fused_mrr) = fusion.calculate_improvement(&fused, &primary, &ground_truth);

        // Fused should have better MRR
        assert!(fused_mrr > 0.0);
        assert!(improvement > 0.0, "Fusion should improve ranking for target");
    }

    // Falsification Test F3: RRF Fusion Improves or Maintains Relevance
    // When both sources have relevant results
    #[test]
    fn falsify_rrf_improves_with_relevant_sources() {
        let fusion = RrfFusion::new();

        // Code search: target is at rank 2
        let primary = vec![
            make_doc("other", 0.9, "code"),
            make_doc("target", 0.8, "code"),
        ];

        // Git search: target is at rank 1 (git has better signal for this query)
        let git_results = vec![
            make_doc("target", 0.95, "git"),
            make_doc("other", 0.85, "git"),
        ];

        let fused = fusion.fuse(
            vec![("code", primary.clone()), ("git", git_results)],
            10,
        );

        let ground_truth = vec!["target".to_string()];
        let (improvement, primary_mrr, fused_mrr) = fusion.calculate_improvement(&fused, &primary, &ground_truth);

        // Falsification: RRF MUST improve relevance when git has better signal
        // Target was #2 in code-only (MRR=0.5), should improve with git boost
        assert!(
            fused_mrr >= primary_mrr,
            "FALSIFIED: RRF did not improve relevance with complementary sources. \
             primary_mrr={}, fused_mrr={}, improvement={}",
            primary_mrr, fused_mrr, improvement
        );
    }

    // Test that RRF with only irrelevant git results doesn't catastrophically fail
    #[test]
    fn test_rrf_with_noisy_secondary_source() {
        let fusion = RrfFusion::new();

        // Code search has the target at rank 1
        let primary = vec![
            make_doc("target", 0.9, "code"),
            make_doc("other", 0.8, "code"),
        ];

        // Git has only noise (no overlap with code results)
        let git_results = vec![
            make_doc("noise1", 0.95, "git"),
            make_doc("noise2", 0.90, "git"),
        ];

        let fused = fusion.fuse(
            vec![("code", primary.clone()), ("git", git_results)],
            10,
        );

        // Target should still be in results (RRF doesn't remove documents)
        let has_target = fused.iter().any(|r| r.id == "target");
        assert!(has_target, "Target should still appear in fused results");

        // Target should be in top 3 even with noise
        let target_rank = fused.iter().position(|r| r.id == "target").unwrap();
        assert!(
            target_rank <= 2,
            "Target should remain in top 3 despite noise, got rank {}",
            target_rank + 1
        );
    }

    #[test]
    fn test_primary_source_selection() {
        let fusion = RrfFusion::new();

        // Code has higher score for func_a
        let code_results = vec![make_doc("func_a", 0.9, "code")];
        let git_results = vec![make_doc("func_a", 0.3, "git")];

        let results = fusion.fuse(
            vec![("code", code_results), ("git", git_results)],
            10,
        );

        assert_eq!(results[0].primary_source, "code");
    }
}
