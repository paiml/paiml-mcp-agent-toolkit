#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::{CaseSensitivity, QueryOptions, QueryResult, SearchMode};
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use regex::Regex;
use std::collections::HashSet;

/// Check if function is a test (test_ prefix or in tests/ directory)
pub(super) fn is_test_function(func: &FunctionEntry) -> bool {
    func.function_name.starts_with("test_")
        || func.file_path.starts_with("tests/")
        || func.file_path.contains("/tests/")
        || func.file_path.contains("_tests.")
        || func.file_path.contains("_test.")
}

/// Parse `file:` and `fn:` prefixes from a query string.
///
/// Extracts optional scope filters:
/// - `file:query.rs error handling` -> (Some("query.rs"), None, "error handling")
/// - `fn:handle_ auth` -> (None, Some("handle_"), "auth")
/// - `file:foo.rs fn:bar baz` -> (Some("foo.rs"), Some("bar"), "baz")
pub(super) fn parse_query_prefixes(query: &str) -> (Option<String>, Option<String>, String) {
    let mut file_filter = None;
    let mut fn_filter = None;
    let mut remaining_parts = Vec::new();

    for token in query.split_whitespace() {
        if let Some(pattern) = token.strip_prefix("file:") {
            if !pattern.is_empty() {
                file_filter = Some(pattern.to_string());
            }
        } else if let Some(pattern) = token.strip_prefix("fn:") {
            if !pattern.is_empty() {
                fn_filter = Some(pattern.to_string());
            }
        } else {
            remaining_parts.push(token);
        }
    }

    let remaining = remaining_parts.join(" ");
    (file_filter, fn_filter, remaining)
}

/// Simple glob matching: supports `*` and `**` patterns
pub(super) fn glob_matches(pattern: &str, path: &str) -> bool {
    if pattern.contains('*') {
        // Convert glob to regex: handle ** before * using placeholder to avoid
        // the second replace clobbering the .* from the first
        let regex_str = pattern
            .replace('.', "\\.")
            .replace("**/", "\x00GLOBSTAR\x00")
            .replace("**", "\x00GLOBSTAR2\x00")
            .replace('*', "[^/]*")
            .replace("\x00GLOBSTAR\x00", "(.*/)?")
            .replace("\x00GLOBSTAR2\x00", ".*");
        Regex::new(&format!("^{regex_str}$"))
            .map_or(false, |re| re.is_match(path))
    } else {
        path.contains(pattern)
    }
}

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
    pub fn query(&self, query: &str, options: QueryOptions) -> Result<Vec<QueryResult>, String> {
        if query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let limit = if options.limit == 0 { 10 } else { options.limit };

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

        // Combine with quality score for final ranking
        let use_quality_weighting = options.search_mode == SearchMode::Semantic;
        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .filter(|(idx, _)| self.passes_filters(*idx, &options))
            .map(|(idx, relevance)| {
                if !use_quality_weighting {
                    // Regex/literal: pure match scoring, no quality weighting
                    return (idx, relevance);
                }
                let func = &self.functions[idx];
                // Combine relevance with quality
                // Higher TDG score = worse quality, so invert
                let quality_factor = 1.0 - (func.quality.tdg_score / 10.0);
                let mut combined = relevance * 0.7 + quality_factor * 0.3;

                // Demote test functions so production code ranks higher
                if is_test_function(func) {
                    combined *= 0.6;
                }

                // Demote generic/common function names
                let freq = self
                    .name_frequency
                    .get(&func.function_name)
                    .copied()
                    .unwrap_or(0.0);
                if freq > 0.001 {
                    // name appears in >0.1% of functions
                    combined *= (1.0 - freq).max(0.3); // floor at 0.3x
                }

                (idx, combined)
            })
            .collect();

        // Apply min_pagerank filter if specified
        if let Some(min_pr) = options.min_pagerank {
            ranked.retain(|(idx, _)| {
                idx < &self.graph_metrics.len() && self.graph_metrics[*idx].pagerank >= min_pr
            });
        }

        // Sort based on rank_by strategy
        match options.rank_by {
            super::types::RankBy::Relevance => {
                // Default: sort by combined relevance score (descending)
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            super::types::RankBy::PageRank => {
                // Sort by PageRank (most important first), using relevance as tiebreaker
                ranked.sort_by(|a, b| {
                    let pr_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.pagerank);
                    let pr_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.pagerank);
                    pr_b.partial_cmp(&pr_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::Centrality => {
                // Sort by degree centrality (most connected first)
                ranked.sort_by(|a, b| {
                    let c_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.centrality);
                    let c_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.centrality);
                    c_b.partial_cmp(&c_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::InDegree => {
                // Sort by in-degree (most called functions first)
                ranked.sort_by(|a, b| {
                    let in_a = self.graph_metrics.get(a.0).map_or(0, |m| m.in_degree);
                    let in_b = self.graph_metrics.get(b.0).map_or(0, |m| m.in_degree);
                    in_b.cmp(&in_a)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            super::types::RankBy::Impact => {
                // Impact ranking is applied post-enrichment in query_handler.rs
                // Fall back to relevance ordering here
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        // Take top results with caller/callee context
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
        // Find the reference function
        let ref_idx = self
            .functions
            .iter()
            .position(|f| f.file_path == file_path && f.function_name == function_name)
            .ok_or_else(|| format!("Function not found: {file_path}::{function_name}"))?;

        // Get the reference document
        let ref_doc = &self.corpus[ref_idx];

        // Calculate similarity to all other functions
        let scores = self.calculate_relevance_scores(ref_doc)?;

        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .filter(|(idx, _)| *idx != ref_idx) // Exclude self
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<QueryResult> = ranked
            .into_iter()
            .take(limit)
            .map(|(idx, score)| {
                QueryResult::from_entry_with_context(
                    &self.functions[idx],
                    idx,
                    self,
                    score,
                    false,
                )
            })
            .collect();

        Ok(results)
    }

    /// Calculate term-based relevance scores for all documents.
    ///
    /// Returns (index, score) pairs for all documents with non-zero scores.
    pub(crate) fn calculate_relevance_scores(&self, query: &str) -> Result<Vec<(usize, f32)>, String> {
        if self.corpus.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize query
        let query_terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        // Score each document using pre-computed lowercase corpus
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

        // Normalize scores to 0-1 range
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
    pub(crate) fn calculate_relevance_scores_scoped(
        &self,
        query: &str,
        candidates: &[usize],
    ) -> Result<Vec<(usize, f32)>, String> {
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

        // Exclude content pattern (like grep -v)
        if let Some(exclude) = &options.exclude_pattern {
            let exclude_lower = exclude.to_lowercase();
            let haystack = format!(
                "{} {} {}",
                func.function_name, func.signature, func.source
            )
            .to_lowercase();
            if haystack.contains(&exclude_lower) {
                return false;
            }
        }

        // Exclude file pattern (like rg --glob '!pattern')
        if let Some(exclude_file) = &options.exclude_file_pattern {
            if func.file_path.contains(exclude_file)
                || glob_matches(exclude_file, &func.file_path)
            {
                return false;
            }
        }

        true
    }

    /// Regex-based scoring: match pattern against source/signature/name
    fn calculate_regex_scores(
        &self,
        pattern: &str,
        candidates: Option<&[usize]>,
        options: &QueryOptions,
    ) -> Result<Vec<(usize, f32)>, String> {
        let case_insensitive = match options.case_sensitivity {
            CaseSensitivity::Insensitive => true,
            CaseSensitivity::Sensitive => false,
            CaseSensitivity::Smart => !pattern.chars().any(|c| c.is_uppercase()),
        };

        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|e| format!("Invalid regex pattern: {e}"))?;

        let iter: Box<dyn Iterator<Item = usize>> = match candidates {
            Some(indices) => Box::new(indices.iter().copied()),
            None => Box::new(0..self.functions.len()),
        };

        let mut results = Vec::new();
        for idx in iter {
            if idx >= self.functions.len() {
                continue;
            }
            let func = &self.functions[idx];
            // Count matches across name, signature, source, and file path
            let name_matches = re.find_iter(&func.function_name).count();
            let sig_matches = re.find_iter(&func.signature).count();
            let source_matches = re.find_iter(&func.source).count();
            let path_matches = re.find_iter(&func.file_path).count();
            let total = name_matches + sig_matches + source_matches + path_matches;
            if total > 0 {
                // Score: weight name matches highest, then signature, then path, then source
                let score = (name_matches as f32 * 3.0
                    + sig_matches as f32 * 2.0
                    + path_matches as f32 * 1.5
                    + source_matches as f32)
                    / (1.0 + func.source.len() as f32 / 1000.0);
                results.push((idx, score));
            }
        }

        // Normalize
        let max_score = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }

    /// Literal string scoring: exact match against source/signature/name
    fn calculate_literal_scores(
        &self,
        needle: &str,
        candidates: Option<&[usize]>,
        options: &QueryOptions,
    ) -> Result<Vec<(usize, f32)>, String> {
        let case_insensitive = match options.case_sensitivity {
            CaseSensitivity::Insensitive => true,
            CaseSensitivity::Sensitive => false,
            CaseSensitivity::Smart => !needle.chars().any(|c| c.is_uppercase()),
        };

        let needle_cmp = if case_insensitive {
            needle.to_lowercase()
        } else {
            needle.to_string()
        };

        let iter: Box<dyn Iterator<Item = usize>> = match candidates {
            Some(indices) => Box::new(indices.iter().copied()),
            None => Box::new(0..self.functions.len()),
        };

        let mut results = Vec::new();
        for idx in iter {
            if idx >= self.functions.len() {
                continue;
            }
            let func = &self.functions[idx];
            let (name, sig, source) = if case_insensitive {
                (
                    func.function_name.to_lowercase(),
                    func.signature.to_lowercase(),
                    func.source.to_lowercase(),
                )
            } else {
                (
                    func.function_name.clone(),
                    func.signature.clone(),
                    func.source.clone(),
                )
            };

            let name_matches = name.matches(&needle_cmp).count();
            let sig_matches = sig.matches(&needle_cmp).count();
            let source_matches = source.matches(&needle_cmp).count();
            let file_path_cmp = if case_insensitive {
                func.file_path.to_lowercase()
            } else {
                func.file_path.clone()
            };
            let path_matches = file_path_cmp.matches(&needle_cmp).count();
            let total = name_matches + sig_matches + source_matches + path_matches;
            if total > 0 {
                let score = (name_matches as f32 * 3.0
                    + sig_matches as f32 * 2.0
                    + path_matches as f32 * 1.5
                    + source_matches as f32)
                    / (1.0 + func.source.len() as f32 / 1000.0);
                results.push((idx, score));
            }
        }

        // Normalize
        let max_score = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }
}
