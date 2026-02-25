// RRF fusion tests
// Included from rrf_fusion.rs — shares parent module scope (no `use` imports here)

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
            make_doc("func_b", 0.95, "git"), // Higher in git
            make_doc("func_a", 0.85, "git"),
            make_doc("func_d", 0.75, "git"), // Only in git
        ];

        let results = fusion.fuse(vec![("code", code_results), ("git", git_results)], 10);

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

        let code_results = vec![RankedDocument {
            id: "func_a".to_string(),
            original_score: 0.9,
            source: "code".to_string(),
            metadata: DocumentMetadata {
                path: "src/main.rs".to_string(),
                name: "func_a".to_string(),
                line_or_timestamp: 42,
                related_commits: vec![],
            },
        }];

        let results = fusion.fuse(vec![("code", code_results)], 10);

        assert_eq!(results[0].metadata.path, "src/main.rs");
        assert_eq!(results[0].metadata.line_or_timestamp, 42);
    }

    #[test]
    fn test_rrf_tracks_sources() {
        let fusion = RrfFusion::new();

        let code_results = vec![make_doc("func_a", 0.9, "code")];
        let git_results = vec![make_doc("func_a", 0.8, "git")];

        let results = fusion.fuse(vec![("code", code_results), ("git", git_results)], 10);

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

        assert!(
            low_k_diff > high_k_diff,
            "Lower k should amplify rank differences"
        );
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
        assert!(
            (mrr - 0.5).abs() < 0.001,
            "Second match should have MRR=0.5"
        );

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
            make_doc("target", 0.7, "code"), // Ground truth
        ];

        // Fused results: ground truth at position 1
        let fused = fusion.fuse(
            vec![
                ("code", primary.clone()),
                ("git", vec![make_doc("target", 0.95, "git")]), // Boost target
            ],
            10,
        );

        let ground_truth = vec!["target".to_string()];
        let (improvement, _, fused_mrr) =
            fusion.calculate_improvement(&fused, &primary, &ground_truth);

        // Fused should have better MRR
        assert!(fused_mrr > 0.0);
        assert!(
            improvement > 0.0,
            "Fusion should improve ranking for target"
        );
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

        let fused = fusion.fuse(vec![("code", primary.clone()), ("git", git_results)], 10);

        let ground_truth = vec!["target".to_string()];
        let (improvement, primary_mrr, fused_mrr) =
            fusion.calculate_improvement(&fused, &primary, &ground_truth);

        // Falsification: RRF MUST improve relevance when git has better signal
        // Target was #2 in code-only (MRR=0.5), should improve with git boost
        assert!(
            fused_mrr >= primary_mrr,
            "FALSIFIED: RRF did not improve relevance with complementary sources. \
             primary_mrr={}, fused_mrr={}, improvement={}",
            primary_mrr,
            fused_mrr,
            improvement
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

        let fused = fusion.fuse(vec![("code", primary.clone()), ("git", git_results)], 10);

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
    fn test_rrf_default_trait() {
        let fusion = RrfFusion::default();
        // default should use k=60
        let results = fusion.fuse(vec![("code", vec![make_doc("a", 0.9, "code")])], 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_improvement_zero_primary_mrr() {
        let fusion = RrfFusion::new();

        // Primary has no matching ground truth
        let primary = vec![make_doc("other", 0.9, "code")];

        // Fused has the target
        let fused = fusion.fuse(
            vec![
                ("code", vec![make_doc("other", 0.9, "code")]),
                ("git", vec![make_doc("target", 0.95, "git")]),
            ],
            10,
        );

        let ground_truth = vec!["target".to_string()];
        let (improvement, primary_mrr, fused_mrr) =
            fusion.calculate_improvement(&fused, &primary, &ground_truth);

        assert_eq!(primary_mrr, 0.0);
        assert!(fused_mrr > 0.0);
        assert_eq!(improvement, 1.0); // Infinite improvement case
    }

    #[test]
    fn test_improvement_both_zero() {
        let fusion = RrfFusion::new();

        let primary = vec![make_doc("other", 0.9, "code")];
        let fused = fusion.fuse(vec![("code", primary.clone())], 10);

        let ground_truth = vec!["nonexistent".to_string()];
        let (improvement, primary_mrr, fused_mrr) =
            fusion.calculate_improvement(&fused, &primary, &ground_truth);

        assert_eq!(primary_mrr, 0.0);
        assert_eq!(fused_mrr, 0.0);
        assert_eq!(improvement, 0.0);
    }

    #[test]
    fn test_mrr_empty_inputs() {
        assert_eq!(
            RrfFusion::mean_reciprocal_rank(&[], &["a".to_string()]),
            0.0
        );
        assert_eq!(
            RrfFusion::mean_reciprocal_rank(&["a".to_string()], &[]),
            0.0
        );
    }

    #[test]
    fn test_mrr_multiple_ground_truth() {
        let results = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let truth = vec!["a".to_string(), "c".to_string()]; // at ranks 1 and 3
        let mrr = RrfFusion::mean_reciprocal_rank(&results, &truth);
        // (1/1 + 1/3) / 2 = 0.6667
        assert!((mrr - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_primary_source_selection() {
        let fusion = RrfFusion::new();

        // Code has higher score for func_a
        let code_results = vec![make_doc("func_a", 0.9, "code")];
        let git_results = vec![make_doc("func_a", 0.3, "git")];

        let results = fusion.fuse(vec![("code", code_results), ("git", git_results)], 10);

        assert_eq!(results[0].primary_source, "code");
    }
}
