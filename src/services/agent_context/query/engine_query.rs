impl AgentContextIndex {
    /// Query the index with semantic search
    ///
    /// Supports scope-aware prefixes:
    /// - `file:query.rs error handling` - search only in matching files
    /// - `fn:handle_ auth` - search only functions matching name prefix
    ///
    /// # Arguments
    /// * `query` - Natural language query (with optional file:/fn: prefixes)
    /// * `options` - Query options for filtering
    ///
    /// # Returns
    /// Ranked list of matching functions
    #[allow(clippy::cast_possible_truncation)]
    pub fn query(&self, query: &str, options: QueryOptions) -> Result<Vec<QueryResult>, String> {
        let limit = if options.limit == 0 {
            10
        } else {
            options.limit
        };

        // Empty query → browse all functions sorted by PageRank (for enrichment-only flags)
        if query.trim().is_empty() {
            return Ok(self.browse_all(limit, &options));
        }

        // Parse scope prefixes
        let (file_filter, fn_filter, remaining_query) = parse_query_prefixes(query);

        // Determine candidate set based on scope prefixes
        let candidates: Option<Vec<usize>> = match (&file_filter, &fn_filter) {
            (Some(file_pat), Some(fn_pat)) => {
                // Both filters: intersect file and name matches
                let file_candidates: HashSet<usize> = self
                    .file_index
                    .iter()
                    .filter(|(path, _)| path.contains(file_pat.as_str()))
                    .flat_map(|(_, indices)| indices.iter().copied())
                    .collect();
                let fn_candidates: Vec<usize> = self
                    .name_index
                    .iter()
                    .filter(|(name, _)| name.starts_with(fn_pat.as_str()))
                    .flat_map(|(_, indices)| indices.iter().copied())
                    .filter(|idx| file_candidates.contains(idx))
                    .collect();
                Some(fn_candidates)
            }
            (Some(file_pat), None) => {
                // File filter only: use file_index for O(1)-ish lookup
                let indices: Vec<usize> = self
                    .file_index
                    .iter()
                    .filter(|(path, _)| path.contains(file_pat.as_str()))
                    .flat_map(|(_, indices)| indices.iter().copied())
                    .collect();
                Some(indices)
            }
            (None, Some(fn_pat)) => {
                // Function name filter: use name_index
                let indices: Vec<usize> = self
                    .name_index
                    .iter()
                    .filter(|(name, _)| name.starts_with(fn_pat.as_str()))
                    .flat_map(|(_, indices)| indices.iter().copied())
                    .collect();
                Some(indices)
            }
            (None, None) => None, // Full corpus scan
        };

        // Use remaining query for scoring, or original if no prefixes found
        let search_query = if remaining_query.is_empty() {
            query
        } else {
            &remaining_query
        };

        // Calculate relevance scores based on search mode
        let scores = match options.search_mode {
            SearchMode::Regex => {
                self.calculate_regex_scores(search_query, candidates.as_deref(), &options)?
            }
            SearchMode::Literal => {
                self.calculate_literal_scores(search_query, candidates.as_deref(), &options)?
            }
            SearchMode::Semantic => {
                if let Some(ref candidate_indices) = candidates {
                    self.calculate_relevance_scores_scoped(search_query, candidate_indices)?
                } else {
                    self.calculate_relevance_scores(search_query)?
                }
            }
        };

        let mut ranked = self.rank_scores(scores, &options);
        self.sort_ranked(&mut ranked, &options);

        let results: Vec<QueryResult> = ranked
            .into_iter()
            .take(limit)
            .map(|(idx, score)| {
                QueryResult::from_entry_with_context(
                    &self.functions[idx],
                    idx,
                    self,
                    score,
                    options.include_source,
                )
            })
            .collect();

        Ok(results)
    }

    fn rank_scores(
        &self,
        scores: Vec<(usize, f32)>,
        options: &QueryOptions,
    ) -> Vec<(usize, f32)> {
        let use_quality = options.search_mode == SearchMode::Semantic;
        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .filter(|(idx, _)| self.passes_filters(*idx, options))
            .map(|(idx, relevance)| {
                if !use_quality {
                    return (idx, relevance);
                }
                (idx, self.apply_quality_weighting(idx, relevance))
            })
            .collect();

        if let Some(min_pr) = options.min_pagerank {
            ranked.retain(|(idx, _)| {
                idx < &self.graph_metrics.len() && self.graph_metrics[*idx].pagerank >= min_pr
            });
        }
        ranked
    }

    fn apply_quality_weighting(&self, idx: usize, relevance: f32) -> f32 {
        let func = &self.functions[idx];
        let quality_factor = 1.0 - (func.quality.tdg_score / 10.0);
        let mut combined = relevance * 0.7 + quality_factor * 0.3;
        if is_test_function(func) {
            combined *= 0.6;
        }
        let freq = self.name_frequency.get(&func.function_name).copied().unwrap_or(0.0);
        if freq > 0.001 {
            combined *= (1.0 - freq).max(0.3);
        }
        combined
    }

    fn sort_ranked(&self, ranked: &mut Vec<(usize, f32)>, options: &QueryOptions) {
        match options.rank_by {
            super::types::RankBy::Relevance | super::types::RankBy::Impact => {
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            super::types::RankBy::PageRank => {
                ranked.sort_by(|a, b| {
                    let pr_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.pagerank);
                    let pr_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.pagerank);
                    pr_b.partial_cmp(&pr_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::Centrality => {
                ranked.sort_by(|a, b| {
                    let c_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.centrality);
                    let c_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.centrality);
                    c_b.partial_cmp(&c_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::InDegree => {
                ranked.sort_by(|a, b| {
                    let in_a = self.graph_metrics.get(a.0).map_or(0, |m| m.in_degree);
                    let in_b = self.graph_metrics.get(b.0).map_or(0, |m| m.in_degree);
                    in_b.cmp(&in_a)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::CrossProject => {
                ranked.sort_by(|a, b| {
                    let score_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.pagerank)
                        * (1.0 + 0.5 * self.count_cross_project_callers(a.0) as f32);
                    let score_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.pagerank)
                        * (1.0 + 0.5 * self.count_cross_project_callers(b.0) as f32);
                    score_b
                        .partial_cmp(&score_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::Priority => {
                ranked.sort_by(|a, b| {
                    let priority_a = self.functions.get(a.0).map_or(0.0, |f| {
                        f.quality.tdg_score * (1.0 + f.quality.churn_score)
                    });
                    let priority_b = self.functions.get(b.0).map_or(0.0, |f| {
                        f.quality.tdg_score * (1.0 + f.quality.churn_score)
                    });
                    priority_b
                        .partial_cmp(&priority_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
        }
    }

    /// Browse all functions sorted by PageRank (for enrichment-only queries).
    fn browse_all(&self, limit: usize, options: &QueryOptions) -> Vec<QueryResult> {
        let mut indexed: Vec<(usize, f32)> = self
            .graph_metrics
            .iter()
            .enumerate()
            .filter(|(i, _)| self.passes_filters(*i, options))
            .map(|(i, m)| (i, m.pagerank))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed
            .into_iter()
            .take(limit)
            .filter_map(|(idx, pr)| {
                self.functions.get(idx).map(|f| {
                    QueryResult::from_entry_with_context(
                        f,
                        idx,
                        self,
                        pr,
                        options.include_source,
                    )
                })
            })
            .collect()
    }

    /// Get function by file and name
    pub fn get_function(&self, file_path: &str, function_name: &str) -> Option<QueryResult> {
        self.functions
            .iter()
            .enumerate()
            .find(|(_, f)| f.file_path == file_path && f.function_name == function_name)
            .map(|(idx, f)| QueryResult::from_entry_with_context(f, idx, self, 1.0, true))
    }

    /// Find similar functions
    pub fn find_similar(
        &self,
        file_path: &str,
        function_name: &str,
        limit: usize,
    ) -> Result<Vec<QueryResult>, String> {
        use crate::services::agent_context::function_index::helpers::build_corpus_entry;

        // Find the reference function
        let ref_idx = self
            .functions
            .iter()
            .position(|f| f.file_path == file_path && f.function_name == function_name)
            .ok_or_else(|| format!("Function not found: {file_path}::{function_name}"))?;

        // Get reference document (build on-the-fly when corpus is empty, e.g. SQLite load)
        let ref_doc = if ref_idx < self.corpus.len() {
            self.corpus[ref_idx].clone()
        } else {
            build_corpus_entry(&self.functions[ref_idx])
        };

        // Calculate similarity to all other functions
        let scores = self.calculate_relevance_scores(&ref_doc)?;

        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .filter(|(idx, _)| *idx != ref_idx) // Exclude self
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<QueryResult> = ranked
            .into_iter()
            .take(limit)
            .map(|(idx, score)| {
                QueryResult::from_entry_with_context(&self.functions[idx], idx, self, score, false)
            })
            .collect();

        Ok(results)
    }
}
