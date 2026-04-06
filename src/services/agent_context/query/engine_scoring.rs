impl AgentContextIndex {
    /// Calculate relevance scores for all documents.
    ///
    /// Uses FTS5 BM25 when SQLite index is available (O(1) per-term lookup),
    /// falls back to TF-only O(n) scan otherwise.
    ///
    /// Returns (index, score) pairs for all documents with non-zero scores.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn calculate_relevance_scores(
        &self,
        query: &str,
    ) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(!query.is_empty(), "query must not be empty");
        // Try FTS5 BM25 search first (fast path)
        if let Some(ref db_path) = self.db_path {
            if let Ok(results) = self.calculate_relevance_scores_fts5(db_path, query) {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
            // Fall through to TF scan if FTS5 fails or returns empty
        }

        self.calculate_relevance_scores_tf(query)
    }

    /// BM25 scoring via FTS5 inverted index (Robertson & Zaragoza, 2009).
    ///
    /// O(1) per-term lookup with built-in IDF weighting, Porter stemming,
    /// and stop word filtering. Returns up to 500 results for downstream
    /// quality weighting and filtering.
    fn calculate_relevance_scores_fts5(
        &self,
        db_path: &std::path::Path,
        query: &str,
    ) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(!query.is_empty(), "query must not be empty");
        use crate::services::agent_context::function_index::sqlite_backend::{
            fts5_search, open_db,
        };
        let conn = open_db(db_path)?;
        // Return more than final limit so downstream filters (grade, test, quality) have candidates
        fts5_search(&conn, query, 500)
    }

    /// Legacy TF-only scoring via O(n) corpus scan.
    ///
    /// Used as fallback when no SQLite FTS5 index is available.
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_relevance_scores_tf(&self, query: &str) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(!query.is_empty(), "query must not be empty");
        if self.corpus.is_empty() {
            return Ok(Vec::new());
        }

        let query_terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut max_score = 0.0f32;

        for (doc_idx, doc_lower) in self.corpus_lower.iter().enumerate() {
            let mut term_score = 0.0f32;
            let mut term_count = 0;
            let doc_len_factor = 1.0 + (self.corpus[doc_idx].len() as f32).ln();

            for term in &query_terms {
                let count = doc_lower.matches(term.as_str()).count() as f32;
                if count > 0.0 {
                    let tf = (1.0 + count.ln()) / doc_len_factor;
                    term_score += tf;
                    term_count += 1;
                }
            }

            if term_count > 0 {
                let score = term_score / query_terms.len() as f32;
                if score > 0.0 {
                    max_score = max_score.max(score);
                    results.push((doc_idx, score));
                }
            }
        }

        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }

    /// Calculate relevance scores for a scoped subset of documents.
    ///
    /// Only scores the candidate indices instead of the full 42K corpus.
    #[allow(clippy::cast_possible_truncation)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn calculate_relevance_scores_scoped(
        &self,
        query: &str,
        candidates: &[usize],
    ) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(!query.is_empty(), "query must not be empty");
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let query_terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        if query_terms.is_empty() {
            // No remaining query terms: return all candidates with equal score
            return Ok(candidates.iter().map(|&idx| (idx, 1.0)).collect());
        }

        let mut results = Vec::new();
        let mut max_score = 0.0f32;

        for &doc_idx in candidates {
            if doc_idx >= self.corpus_lower.len() {
                continue;
            }
            let doc_lower = &self.corpus_lower[doc_idx];
            let mut term_score = 0.0f32;
            let mut term_count = 0;
            let doc_len_factor = 1.0 + (self.corpus[doc_idx].len() as f32).ln();

            for term in &query_terms {
                let count = doc_lower.matches(term.as_str()).count() as f32;
                if count > 0.0 {
                    let tf = (1.0 + count.ln()) / doc_len_factor;
                    term_score += tf;
                    term_count += 1;
                }
            }

            if term_count > 0 {
                let score = term_score / query_terms.len() as f32;
                if score > 0.0 {
                    max_score = max_score.max(score);
                    results.push((doc_idx, score));
                }
            }
        }

        // Normalize
        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }

    /// Check if function passes filter options
    fn passes_filters(&self, idx: usize, options: &QueryOptions) -> bool {
        debug_assert!(true, "contract: passes_filters");
        let func = &self.functions[idx];

        // Grade filter
        if let Some(min_grade) = &options.min_grade {
            let grade_order = ["A", "B", "C", "D", "F"];
            let min_idx = grade_order.iter().position(|g| *g == min_grade);
            let func_idx = grade_order
                .iter()
                .position(|g| *g == func.quality.tdg_grade.as_str());

            if let (Some(min_i), Some(func_i)) = (min_idx, func_idx) {
                if func_i > min_i {
                    return false;
                }
            }
        }

        // Complexity filter
        if let Some(max_complexity) = options.max_complexity {
            if func.quality.complexity > max_complexity {
                return false;
            }
        }

        // LOC filter
        if let Some(max_loc) = options.max_loc {
            if func.quality.loc > max_loc {
                return false;
            }
        }

        // Language filter
        if let Some(lang) = &options.language {
            if !func.language.eq_ignore_ascii_case(lang) {
                return false;
            }
        }

        // Path pattern filter
        if let Some(pattern) = &options.path_pattern {
            if !func.file_path.contains(pattern) {
                return false;
            }
        }

        // Exclude content patterns (like grep -v, repeatable)
        for exclude in &options.exclude_pattern {
            let exclude_lower = exclude.to_lowercase();
            let haystack =
                format!("{} {} {}", func.function_name, func.signature, func.source).to_lowercase();
            if haystack.contains(&exclude_lower) {
                return false;
            }
        }

        // Exclude file patterns (like rg --glob '!pattern', repeatable)
        for exclude_file in &options.exclude_file_pattern {
            if func.file_path.contains(exclude_file) || glob_matches(exclude_file, &func.file_path)
            {
                return false;
            }
        }

        true
    }
}
