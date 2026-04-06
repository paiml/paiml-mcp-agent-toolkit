// RRF scoring methods — fuse(), calculate_improvement(), mean_reciprocal_rank()
// Included from rrf_fusion.rs — shares parent module scope (no `use` imports here)

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

                let entry = scores
                    .entry(doc.id.clone())
                    .or_insert_with(|| FusedResultBuilder {
                        id: doc.id.clone(),
                        total_rrf: 0.0,
                        source_scores: HashMap::new(),
                        best_metadata: doc.metadata.clone(),
                        best_score: 0.0,
                        primary_source: source.to_string(),
                    });

                entry.total_rrf += rrf_contribution;
                entry
                    .source_scores
                    .insert(source.to_string(), doc.original_score);

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

        results.sort_by(|a, b| {
            b.rrf_score
                .partial_cmp(&a.rrf_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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
            &primary_results
                .iter()
                .map(|d| d.id.clone())
                .collect::<Vec<_>>(),
            ground_truth_ids,
        );

        let fused_mrr = Self::mean_reciprocal_rank(
            &fused_results
                .iter()
                .map(|r| r.id.clone())
                .collect::<Vec<_>>(),
            ground_truth_ids,
        );

        let improvement = if primary_mrr > 0.0 {
            (fused_mrr - primary_mrr) / primary_mrr
        } else if fused_mrr > 0.0 {
            1.0 // Infinite improvement
        } else {
            0.0
        };

        debug_assert!(primary_mrr >= 0.0 && primary_mrr <= 1.0, "MRR out of range: {}", primary_mrr);
        debug_assert!(fused_mrr >= 0.0 && fused_mrr <= 1.0, "MRR out of range: {}", fused_mrr);
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
