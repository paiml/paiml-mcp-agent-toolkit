#![cfg_attr(coverage_nightly, coverage(off))]
//! Query Engine for Agent Context
//!
//! Provides semantic search with quality filtering over the function index.

use super::{AgentContextIndex, FunctionEntry};
use crate::models::churn::FileChurnMetrics;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Deduplicate a list of strings while preserving first-seen order
fn dedup_ordered(items: &[&str]) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|s| seen.insert(**s))
        .map(|s| s.to_string())
        .collect()
}

/// Check if function is a test (test_ prefix or in tests/ directory)
fn is_test_function(func: &FunctionEntry) -> bool {
    func.function_name.starts_with("test_")
        || func.file_path.starts_with("tests/")
        || func.file_path.contains("/tests/")
        || func.file_path.contains("_tests.")
        || func.file_path.contains("_test.")
}

/// Ranking strategy for query results
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RankBy {
    /// Rank by semantic relevance (default)
    #[default]
    Relevance,
    /// Rank by PageRank (most important/called functions first)
    PageRank,
    /// Rank by degree centrality (most connected functions)
    Centrality,
    /// Rank by in-degree (most called by others)
    InDegree,
}

impl std::str::FromStr for RankBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "relevance" | "rel" => Ok(RankBy::Relevance),
            "pagerank" | "pr" | "importance" => Ok(RankBy::PageRank),
            "centrality" | "degree" => Ok(RankBy::Centrality),
            "indegree" | "callers" => Ok(RankBy::InDegree),
            _ => Err(format!("Unknown rank-by: '{}'. Valid: relevance, pagerank, centrality, indegree", s)),
        }
    }
}

/// Query options for filtering results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryOptions {
    /// Maximum number of results
    pub limit: usize,
    /// Minimum TDG grade (A, B, C, D, F)
    pub min_grade: Option<String>,
    /// Maximum cyclomatic complexity
    pub max_complexity: Option<u32>,
    /// Maximum lines of code
    pub max_loc: Option<u32>,
    /// Filter by language
    pub language: Option<String>,
    /// Filter by file path pattern
    pub path_pattern: Option<String>,
    /// Include full source code in results
    pub include_source: bool,
    /// Ranking strategy (default: relevance)
    #[serde(default)]
    pub rank_by: RankBy,
    /// Minimum PageRank score filter
    pub min_pagerank: Option<f32>,
}

/// A search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// File path relative to project root
    pub file_path: String,
    /// Function name
    pub function_name: String,
    /// Full function signature
    pub signature: String,
    /// Type of definition (function, struct, enum, trait, type)
    #[serde(default)]
    pub definition_type: String,
    /// Documentation comment
    pub doc_comment: Option<String>,
    /// Starting line number
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Programming language
    pub language: String,
    /// TDG score
    pub tdg_score: f32,
    /// TDG grade
    pub tdg_grade: String,
    /// Cyclomatic complexity
    pub complexity: u32,
    /// Big-O estimate
    pub big_o: String,
    /// SATD count
    pub satd_count: u32,
    /// Lines of code
    pub loc: u32,
    /// Relevance score (0-1)
    pub relevance_score: f32,
    /// Full source (if requested)
    pub source: Option<String>,
    /// Function names this function calls
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<String>,
    /// Function names that call this function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub called_by: Vec<String>,
    /// PageRank score (importance based on call graph)
    #[serde(default)]
    pub pagerank: f32,
    /// In-degree (number of direct callers)
    #[serde(default)]
    pub in_degree: u32,
    /// Out-degree (number of direct callees)
    #[serde(default)]
    pub out_degree: u32,
    /// Git commit count for the file (churn indicator)
    #[serde(default)]
    pub commit_count: u32,
    /// Churn score (0.0-1.0, higher = more volatile)
    #[serde(default)]
    pub churn_score: f32,
    /// Number of code clones (duplicate instances)
    #[serde(default)]
    pub clone_count: u32,
    /// Duplication score (0.0-1.0, higher = more duplicated)
    #[serde(default)]
    pub duplication_score: f32,
    /// Pattern diversity (0.0-1.0, higher = more unique patterns, lower = more repetitive)
    #[serde(default)]
    pub pattern_diversity: f32,
    /// Fault pattern annotations from batuta bug-hunter (mutation targets, boundary conditions)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fault_annotations: Vec<String>,
}

impl QueryResult {
    /// Create from function entry with relevance score
    pub fn from_entry(entry: &FunctionEntry, relevance: f32, include_source: bool) -> Self {
        Self {
            file_path: entry.file_path.clone(),
            function_name: entry.function_name.clone(),
            signature: entry.signature.clone(),
            definition_type: format!("{:?}", entry.definition_type).to_lowercase(),
            doc_comment: entry.doc_comment.clone(),
            start_line: entry.start_line,
            end_line: entry.end_line,
            language: entry.language.clone(),
            tdg_score: entry.quality.tdg_score,
            tdg_grade: entry.quality.tdg_grade.clone(),
            complexity: entry.quality.complexity,
            big_o: entry.quality.big_o.clone(),
            satd_count: entry.quality.satd_count,
            loc: entry.quality.loc,
            relevance_score: relevance,
            source: if include_source {
                Some(entry.source.clone())
            } else {
                None
            },
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: entry.commit_count,
            churn_score: entry.churn_score,
            clone_count: entry.clone_count,
            duplication_score: entry.clone_count as f32 / 10.0, // Normalize
            pattern_diversity: entry.pattern_diversity,
            fault_annotations: entry.fault_annotations.clone(),
        }
    }

    /// Create from function entry with caller/callee context and graph metrics
    pub fn from_entry_with_context(
        entry: &FunctionEntry,
        func_idx: usize,
        index: &AgentContextIndex,
        relevance: f32,
        include_source: bool,
    ) -> Self {
        let mut result = Self::from_entry(entry, relevance, include_source);

        // Add graph metrics if available
        if func_idx < index.graph_metrics.len() {
            let metrics = &index.graph_metrics[func_idx];
            result.pagerank = metrics.pagerank;
            result.in_degree = metrics.in_degree;
            result.out_degree = metrics.out_degree;
        }

        // Deduplicate while preserving first-seen order
        result.calls = dedup_ordered(&index.get_calls(func_idx));
        // Separate production callers from test callers, cap display
        let all_callers = dedup_ordered(&index.get_called_by(func_idx));
        let (prod, tests): (Vec<_>, Vec<_>) = all_callers
            .into_iter()
            .partition(|name| !name.starts_with("test_"));
        const MAX_CALLERS: usize = 10;
        if prod.len() > MAX_CALLERS {
            result.called_by = prod[..MAX_CALLERS].to_vec();
            result
                .called_by
                .push(format!("(+{} more)", prod.len() - MAX_CALLERS));
        } else {
            result.called_by = prod;
        }
        if !tests.is_empty() {
            result
                .called_by
                .push(format!("(+{} tests)", tests.len()));
        }
        result
    }

    /// Format for display
    pub fn format_display(&self) -> String {
        format!(
            "{}:{} - {}\n  Signature: {}\n  TDG: {} ({:.1}) | Complexity: {} | Big-O: {}\n  Doc: {}\n  Relevance: {:.2}",
            self.file_path,
            self.start_line,
            self.function_name,
            self.signature,
            self.tdg_grade,
            self.tdg_score,
            self.complexity,
            self.big_o,
            self.doc_comment.as_deref().unwrap_or("(none)"),
            self.relevance_score
        )
    }
}

/// Parse `file:` and `fn:` prefixes from a query string.
///
/// Extracts optional scope filters:
/// - `file:query.rs error handling` -> (Some("query.rs"), None, "error handling")
/// - `fn:handle_ auth` -> (None, Some("handle_"), "auth")
/// - `file:foo.rs fn:bar baz` -> (Some("foo.rs"), Some("bar"), "baz")
fn parse_query_prefixes(query: &str) -> (Option<String>, Option<String>, String) {
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
                let file_candidates: std::collections::HashSet<usize> = self
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

        // Calculate relevance scores
        let scores = if let Some(ref candidate_indices) = candidates {
            // Scoped scoring: only score candidate functions
            self.calculate_relevance_scores_scoped(search_query, candidate_indices)?
        } else {
            self.calculate_relevance_scores(search_query)?
        };

        // Combine with quality score for final ranking
        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .filter(|(idx, _)| self.passes_filters(*idx, &options))
            .map(|(idx, relevance)| {
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
            RankBy::Relevance => {
                // Default: sort by combined relevance score (descending)
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            RankBy::PageRank => {
                // Sort by PageRank (most important first), using relevance as tiebreaker
                ranked.sort_by(|a, b| {
                    let pr_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.pagerank);
                    let pr_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.pagerank);
                    pr_b.partial_cmp(&pr_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            RankBy::Centrality => {
                // Sort by degree centrality (most connected first)
                ranked.sort_by(|a, b| {
                    let c_a = self.graph_metrics.get(a.0).map_or(0.0, |m| m.centrality);
                    let c_b = self.graph_metrics.get(b.0).map_or(0.0, |m| m.centrality);
                    c_b.partial_cmp(&c_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
            }
            RankBy::InDegree => {
                // Sort by in-degree (most called functions first)
                ranked.sort_by(|a, b| {
                    let in_a = self.graph_metrics.get(a.0).map_or(0, |m| m.in_degree);
                    let in_b = self.graph_metrics.get(b.0).map_or(0, |m| m.in_degree);
                    in_b.cmp(&in_a)
                        .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
                });
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
    fn calculate_relevance_scores(&self, query: &str) -> Result<Vec<(usize, f32)>, String> {
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
    fn calculate_relevance_scores_scoped(
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

        true
    }
}

/// Format results as JSON
pub fn format_json(results: &[QueryResult]) -> Result<String, String> {
    serde_json::to_string_pretty(results).map_err(|e| format!("JSON serialization failed: {e}"))
}

/// Format results as markdown
pub fn format_markdown(results: &[QueryResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Search Results ({} functions)\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!("## {}. `{}`\n\n", i + 1, r.function_name));
        output.push_str(&format!(
            "**Location:** `{}:{}` ({} lines)\n\n",
            r.file_path, r.start_line, r.loc
        ));
        output.push_str(&format!("**Signature:**\n```\n{}\n```\n\n", r.signature));
        // Quality metrics with SATD warning
        let mut quality = format!(
            "**Quality:** TDG {} ({:.1}) | Complexity: {} | Big-O: {}",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        );
        if r.satd_count > 0 {
            quality.push_str(&format!(" | ⚠️ **SATD: {}**", r.satd_count));
        }
        // Add churn indicator for volatile files
        if r.churn_score > 0.5 {
            quality.push_str(&format!(" | 🔥 **Hot: {} commits ({:.0}%)**", r.commit_count, r.churn_score * 100.0));
        } else if r.commit_count > 0 {
            quality.push_str(&format!(" | Commits: {}", r.commit_count));
        }
        // Add duplication indicator
        if r.clone_count > 0 {
            quality.push_str(&format!(" | 📋 **Clones: {} ({:.0}%)**", r.clone_count, r.duplication_score * 100.0));
        }
        // Add entropy indicator for low pattern diversity
        if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
            quality.push_str(&format!(" | 🔄 **Repetitive ({:.0}%)**", (1.0 - r.pattern_diversity) * 100.0));
        }
        output.push_str(&quality);
        output.push_str("\n\n");

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("**Documentation:** {}\n\n", doc));
        }

        if !r.calls.is_empty() {
            output.push_str(&format!("**Calls:** {}\n\n", r.calls.join(", ")));
        }
        if !r.called_by.is_empty() {
            output.push_str(&format!("**Called by:** {}\n\n", r.called_by.join(", ")));
        }

        // Show graph metrics if significant
        if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
            output.push_str(&format!(
                "**Graph:** PageRank {:.6} | In-Degree: {} | Out-Degree: {}\n\n",
                r.pagerank, r.in_degree, r.out_degree
            ));
        }

        output.push_str(&format!("**Relevance:** {:.2}\n\n", r.relevance_score));
        output.push_str("---\n\n");
    }

    output
}

/// Format results as text with inline source code (agent-friendly)
/// Uses syntect for rich syntax highlighting
pub fn format_text_with_code(results: &[QueryResult]) -> String {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    let mut output = String::new();

    // Load syntax definitions and theme
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    for r in results.iter() {
        // Header with rich colors
        // Cyan for file path, yellow for function name, green for TDG, magenta for Big-O
        output.push_str(&format!(
            "\x1b[36m{}\x1b[0m:\x1b[33m{}-{}\x1b[0m │ \x1b[1;37m{}\x1b[0m │ TDG: \x1b[32m{}\x1b[0m │ \x1b[35m{}\x1b[0m\n",
            r.file_path, r.start_line, r.end_line, r.function_name, r.tdg_grade, r.big_o
        ));

        // Metrics line - always show key metrics for agent decision-making
        let mut metrics = Vec::new();

        // Core metrics (always show)
        metrics.push(format!("C:{}", r.complexity));
        metrics.push(format!("L:{}", r.loc));

        // PageRank - show importance score (higher = more central to codebase)
        if r.pagerank > 0.0 {
            let pr_scaled = r.pagerank * 10000.0;
            if pr_scaled >= 10.0 {
                metrics.push(format!("\x1b[1;36m★{:.0}\x1b[0m", pr_scaled));
            } else if pr_scaled >= 1.0 {
                metrics.push(format!("★{:.1}", pr_scaled));
            }
        }

        // In-degree (callers) - shows how widely used
        if r.in_degree >= 5 {
            metrics.push(format!("\x1b[1;32m↓{}\x1b[0m", r.in_degree));
        } else if r.in_degree > 0 {
            metrics.push(format!("↓{}", r.in_degree));
        }

        // Churn - git volatility (commit count and churn score)
        if r.commit_count > 0 {
            if r.churn_score > 0.7 {
                metrics.push(format!("\x1b[1;31m🔥{}c {:.0}%\x1b[0m", r.commit_count, r.churn_score * 100.0));
            } else if r.churn_score > 0.3 {
                metrics.push(format!("{}c {:.0}%", r.commit_count, r.churn_score * 100.0));
            } else {
                metrics.push(format!("{}c", r.commit_count));
            }
        }

        // Pattern diversity / entropy (lower = more repetitive code patterns)
        if r.pattern_diversity > 0.0 {
            if r.pattern_diversity < 0.3 {
                metrics.push(format!("\x1b[2m🔄{:.0}%\x1b[0m", r.pattern_diversity * 100.0));
            } else if r.pattern_diversity > 0.8 {
                metrics.push(format!("H:{:.0}%", r.pattern_diversity * 100.0));
            }
        }

        // SATD (tech debt markers)
        if r.satd_count > 0 {
            metrics.push(format!("\x1b[1;33m⚠{}\x1b[0m", r.satd_count));
        }

        // Clone count (duplicates)
        if r.clone_count > 0 {
            metrics.push(format!("\x1b[1;35m📋{}\x1b[0m", r.clone_count));
        }

        // Fault annotations (Tarantula-style defect suspiciousness)
        if !r.fault_annotations.is_empty() {
            // Show count and first annotation type
            let first = r.fault_annotations.first().map_or("", |s| {
                s.split(':').next().unwrap_or(s)
            });
            metrics.push(format!("\x1b[1;91m🐛{}:{}\x1b[0m", r.fault_annotations.len(), first));
        }

        output.push_str(&format!("   \x1b[2m{}\x1b[0m\n", metrics.join(" │ ")));

        // Doc comment (important context for understanding intent)
        if let Some(doc) = &r.doc_comment {
            // Truncate long docs, show first line
            let first_line = doc.lines().next().unwrap_or(doc);
            let truncated = if first_line.len() > 100 {
                format!("{}...", &first_line[..97])
            } else {
                first_line.to_string()
            };
            output.push_str(&format!("   \x1b[3;37m/// {}\x1b[0m\n", truncated));
        }

        // Call graph (useful for navigation)
        if !r.calls.is_empty() || !r.called_by.is_empty() {
            let mut graph_parts = Vec::new();
            if !r.calls.is_empty() {
                let calls_str = if r.calls.len() <= 5 {
                    r.calls.join(", ")
                } else {
                    format!("{}, (+{} more)", r.calls[..5].join(", "), r.calls.len() - 5)
                };
                graph_parts.push(format!("calls: {}", calls_str));
            }
            if !r.called_by.is_empty() {
                let called_str = if r.called_by.len() <= 3 {
                    r.called_by.join(", ")
                } else {
                    format!("{}, (+{} more)", r.called_by[..3].join(", "), r.called_by.len() - 3)
                };
                graph_parts.push(format!("← {}", called_str));
            }
            output.push_str(&format!("   \x1b[2;36m{}\x1b[0m\n", graph_parts.join(" │ ")));
        }

        // Fault annotations with red/yellow warning colors
        for fault in &r.fault_annotations {
            if fault.contains("Boundary") || fault.contains("condition") {
                output.push_str(&format!("\x1b[1;33m⚠️  {}\x1b[0m\n", fault)); // Yellow for boundary
            } else if fault.contains("Arithmetic") {
                output.push_str(&format!("\x1b[1;31m⚠️  {}\x1b[0m\n", fault)); // Red for arithmetic
            } else {
                output.push_str(&format!("\x1b[1;35m⚠️  {}\x1b[0m\n", fault)); // Magenta for others
            }
        }

        // Source code with syntax highlighting
        if let Some(source) = &r.source {
            // Detect language from file extension
            let syntax = ps
                .find_syntax_by_extension(
                    r.file_path
                        .rsplit('.')
                        .next()
                        .unwrap_or("rs"),
                )
                .unwrap_or_else(|| ps.find_syntax_plain_text());

            let mut h = HighlightLines::new(syntax, theme);

            for line in LinesWithEndings::from(source) {
                match h.highlight_line(line, &ps) {
                    Ok(ranges) => {
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                        output.push_str(&escaped);
                    }
                    Err(_) => {
                        output.push_str(line);
                    }
                }
            }

            if !source.ends_with('\n') {
                output.push('\n');
            }
            // Reset colors after code block
            output.push_str("\x1b[0m");
        } else {
            output.push_str("\x1b[2m// (use --include-source to see code)\x1b[0m\n");
        }

        output.push('\n');
    }

    output
}

/// Format results as text
pub fn format_text(results: &[QueryResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!("Found {} functions:\n\n", results.len()));

    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "{}. {}:{} - {}\n",
            i + 1,
            r.file_path,
            r.start_line,
            r.function_name
        ));
        output.push_str(&format!("   Signature: {}\n", r.signature));
        // Core metrics line
        let mut metrics = format!(
            "   TDG: {} ({:.1}) | Complexity: {} | Big-O: {}",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        );

        // Add SATD warning if technical debt markers exist
        if r.satd_count > 0 {
            metrics.push_str(&format!(" | ⚠️ SATD: {}", r.satd_count));
        }

        // Add LOC for large functions
        if r.loc > 50 {
            metrics.push_str(&format!(" | LOC: {}", r.loc));
        }

        // Add churn indicator for volatile files
        if r.churn_score > 0.5 {
            metrics.push_str(&format!(" | 🔥 Hot: {} commits ({:.0}%)", r.commit_count, r.churn_score * 100.0));
        } else if r.commit_count > 0 {
            metrics.push_str(&format!(" | Commits: {}", r.commit_count));
        }

        // Add duplication indicator
        if r.clone_count > 0 {
            metrics.push_str(&format!(" | 📋 Clones: {} ({:.0}%)", r.clone_count, r.duplication_score * 100.0));
        }

        // Add entropy indicator for low pattern diversity
        if r.pattern_diversity > 0.0 && r.pattern_diversity < 0.3 {
            metrics.push_str(&format!(" | 🔄 Repetitive ({:.0}%)", (1.0 - r.pattern_diversity) * 100.0));
        }

        output.push_str(&metrics);
        output.push('\n');

        // Add fault annotations if present
        if !r.fault_annotations.is_empty() {
            for fault in &r.fault_annotations {
                output.push_str(&format!("   ⚠️ {}\n", fault));
            }
        }

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("   Doc: {}\n", doc));
        }

        if !r.calls.is_empty() {
            output.push_str(&format!("   Calls: {}\n", r.calls.join(", ")));
        }
        if !r.called_by.is_empty() {
            output.push_str(&format!("   Called by: {}\n", r.called_by.join(", ")));
        }

        // Show graph metrics if significant
        if r.pagerank > 0.0 || r.in_degree > 0 || r.out_degree > 0 {
            output.push_str(&format!(
                "   Graph: PageRank {:.6} | In-Degree: {} | Out-Degree: {}\n",
                r.pagerank, r.in_degree, r.out_degree
            ));
        }

        output.push_str(&format!("   Relevance: {:.2}\n\n", r.relevance_score));
    }

    output
}

/// Enrich query results with churn metrics from pre-computed file churn data.
///
/// Maps file-level churn to function-level results. Since churn is computed
/// per-file (not per-function), all functions in the same file share the
/// same churn metrics.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `file_churn` - Map of relative file path -> churn metrics
///
/// # Example
/// ```rust,no_run
/// use pmat::services::agent_context::{enrich_with_churn, QueryResult};
/// use std::collections::HashMap;
///
/// let mut results = vec![/* ... */];
/// let churn_map: HashMap<String, (u32, f32)> = HashMap::new();
/// enrich_with_churn(&mut results, &churn_map);
/// ```
pub fn enrich_with_churn(results: &mut [QueryResult], file_churn: &HashMap<String, (u32, f32)>) {
    for result in results.iter_mut() {
        if let Some((commit_count, churn_score)) = file_churn.get(&result.file_path) {
            result.commit_count = *commit_count;
            result.churn_score = *churn_score;
        }
    }
}

/// Build a churn lookup map from FileChurnMetrics.
///
/// Converts a slice of file churn metrics into a HashMap keyed by relative path
/// for O(1) lookup during result enrichment.
pub fn build_churn_map(metrics: &[FileChurnMetrics]) -> HashMap<String, (u32, f32)> {
    metrics
        .iter()
        .map(|m| (m.relative_path.clone(), (m.commit_count as u32, m.churn_score)))
        .collect()
}

/// Compute churn for files in query results.
///
/// Uses git log to compute churn metrics for files referenced in query results.
/// This is a convenience function for on-demand churn enrichment.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for git operations
/// * `period_days` - Number of days to look back in git history
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git + IncrementalChurnAnalyzer
pub async fn enrich_results_with_churn(
    results: &mut [QueryResult],
    project_root: &Path,
    period_days: u32,
) -> Result<(), String> {
    use crate::services::incremental_churn::IncrementalChurnAnalyzer;

    if results.is_empty() {
        return Ok(());
    }

    // Collect unique files from results
    let files: Vec<std::path::PathBuf> = results
        .iter()
        .map(|r| project_root.join(&r.file_path))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Analyze churn for these files
    let analyzer = IncrementalChurnAnalyzer::new(project_root.to_path_buf());
    let analysis = analyzer
        .analyze_incremental(files, period_days)
        .await
        .map_err(|e| format!("Churn analysis failed: {e}"))?;

    // Build lookup map
    let churn_map = build_churn_map(&analysis.files);

    // Enrich results
    enrich_with_churn(results, &churn_map);

    Ok(())
}

/// Enrich query results with duplicate detection data.
///
/// Detects code clones using MinHash + LSH for O(1) similarity matching.
/// Results are enriched with clone_count and duplication_score.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for file access
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires filesystem + DuplicateDetectionEngine
pub async fn enrich_results_with_duplicates(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    use crate::services::duplicate_detector::{DuplicateDetectionConfig, DuplicateDetectionEngine, Language};

    if results.is_empty() {
        return Ok(());
    }

    // Collect unique files and their content
    let mut files_to_analyze = Vec::new();
    let mut file_contents: HashMap<String, String> = HashMap::new();

    for result in results.iter() {
        if file_contents.contains_key(&result.file_path) {
            continue;
        }

        let full_path = project_root.join(&result.file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            file_contents.insert(result.file_path.clone(), content);
        }
    }

    // Determine language and build file list
    for (path, content) in &file_contents {
        let lang = if path.ends_with(".rs") {
            Language::Rust
        } else if path.ends_with(".ts") || path.ends_with(".tsx") {
            Language::TypeScript
        } else if path.ends_with(".js") || path.ends_with(".jsx") {
            Language::JavaScript
        } else if path.ends_with(".py") {
            Language::Python
        } else if path.ends_with(".c") {
            Language::C
        } else if path.ends_with(".cpp") || path.ends_with(".cc") || path.ends_with(".cxx") {
            Language::Cpp
        } else if path.ends_with(".kt") {
            Language::Kotlin
        } else {
            continue; // Skip unsupported languages
        };

        files_to_analyze.push((std::path::PathBuf::from(path), content.clone(), lang));
    }

    if files_to_analyze.is_empty() {
        return Ok(());
    }

    // Run duplicate detection with relaxed settings for function-level analysis
    let config = DuplicateDetectionConfig {
        min_tokens: 20, // Lower threshold to catch function-level duplicates
        similarity_threshold: 0.65,
        ..Default::default()
    };

    let engine = DuplicateDetectionEngine::new(config);
    let report = engine
        .detect_duplicates(&files_to_analyze)
        .map_err(|e| format!("Duplicate detection failed: {e}"))?;

    // Build file -> duplication metrics map
    let mut file_duplication: HashMap<String, (u32, f32)> = HashMap::new();
    for group in &report.groups {
        for fragment in &group.fragments {
            let path_str = fragment.file.to_string_lossy().to_string();
            let entry = file_duplication.entry(path_str).or_insert((0, 0.0));
            entry.0 += 1; // clone count
            entry.1 = entry.1.max(group.average_similarity as f32); // max similarity
        }
    }

    // Enrich results
    for result in results.iter_mut() {
        if let Some((clone_count, dup_score)) = file_duplication.get(&result.file_path) {
            result.clone_count = *clone_count;
            result.duplication_score = *dup_score;
        }
    }

    Ok(())
}

/// Enrich query results with entropy/pattern diversity metrics.
///
/// Analyzes code for repetitive patterns using AST-based pattern extraction.
/// Low pattern diversity indicates code that could benefit from refactoring.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for analysis
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires EntropyAnalyzer + filesystem
pub async fn enrich_results_with_entropy(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    use crate::entropy::{EntropyAnalyzer, EntropyConfig};

    if results.is_empty() {
        return Ok(());
    }

    // Run entropy analysis on the project
    let config = EntropyConfig::default();
    let analyzer = EntropyAnalyzer::with_config(config);
    let report = analyzer
        .analyze(project_root)
        .await
        .map_err(|e| format!("Entropy analysis failed: {e}"))?;

    // Get overall pattern diversity
    let overall_diversity = report.entropy_metrics.pattern_diversity as f32;

    // Build file -> pattern count map from violations
    let mut file_pattern_count: HashMap<String, usize> = HashMap::new();
    for violation in &report.actionable_violations {
        for file in &violation.affected_files {
            let path_str = file
                .strip_prefix(project_root)
                .unwrap_or(file)
                .to_string_lossy()
                .to_string();
            *file_pattern_count.entry(path_str).or_insert(0) += 1;
        }
    }

    // Calculate per-file diversity (inverse of pattern repetition)
    let max_patterns = file_pattern_count.values().max().copied().unwrap_or(1) as f32;

    // Enrich results
    for result in results.iter_mut() {
        if let Some(&pattern_count) = file_pattern_count.get(&result.file_path) {
            // Lower diversity = more repetitive patterns
            result.pattern_diversity = 1.0 - (pattern_count as f32 / max_patterns).min(1.0);
        } else {
            // No violations = high diversity (good)
            result.pattern_diversity = overall_diversity;
        }
    }

    Ok(())
}

/// Enrich query results with batuta fault pattern annotations.
///
/// Runs batuta bug-hunter falsify to detect mutation targets and boundary conditions.
/// Results are enriched with fault_annotations containing any detected issues.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for analysis
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires pmat subprocess
pub async fn enrich_results_with_faults(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    use std::process::Command;

    if results.is_empty() {
        return Ok(());
    }

    // Run batuta bug-hunter falsify in JSON mode
    let output = Command::new("batuta")
        .args([
            "bug-hunter",
            "falsify",
            "--format",
            "json",
            "--target",
            ".",
        ])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("Failed to run batuta: {e}"))?;

    if !output.status.success() {
        // batuta returns exit code 2 for help/usage, which is fine
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("Usage:") {
            return Err(format!("batuta failed: {stderr}"));
        }
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to find the JSON object in the output (batuta may print warnings first)
    let json_start = stdout.find('{');
    let json_str = match json_start {
        Some(start) => &stdout[start..],
        None => return Ok(()), // No JSON output, no findings
    };

    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse batuta output: {e}"))?;

    // Extract findings
    let findings = match parsed.get("findings").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return Ok(()), // No findings
    };

    // Build file:line -> findings map
    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding
            .get("file")
            .and_then(|f| f.as_str())
            .unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        // Normalize path (remove leading ./)
        let normalized_file = file.strip_prefix("./").unwrap_or(file);

        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);

        fault_map.entry(key).or_default().push(annotation);
    }

    // Enrich results with fault annotations
    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            // Filter to faults within the function's line range
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;

            let relevant_faults: Vec<_> = faults
                .iter()
                .filter(|f| {
                    // Extract line number from annotation and check if in function range
                    if let Some(line_part) = f.split("at line ").last() {
                        if let Ok(line) = line_part.parse::<usize>() {
                            return line >= func_start && line <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();

            if !relevant_faults.is_empty() {
                result.fault_annotations = relevant_faults;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_context::function_index::DefinitionType;
    use crate::services::agent_context::QualityMetrics;
    use std::collections::HashMap;

    fn create_test_entry(name: &str, complexity: u32, tdg_score: f32) -> FunctionEntry {
        FunctionEntry {
            file_path: "test.rs".to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            doc_comment: Some("Test function".to_string()),
            source: format!("fn {name}() {{ }}"),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score,
                tdg_grade: if tdg_score < 2.0 {
                    "A".to_string()
                } else {
                    "B".to_string()
                },
                complexity,
                cognitive_complexity: complexity,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 10,
                commit_count: 0,
                churn_score: 0.0,
            },
            checksum: "abc123".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        }
    }

    #[test]
    fn test_query_result_from_entry() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);

        assert_eq!(result.function_name, "test_func");
        assert_eq!(result.complexity, 5);
        assert!((result.tdg_score - 1.5).abs() < 0.01);
        assert!(result.source.is_none());
    }

    #[test]
    fn test_query_result_with_source() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, true);

        assert!(result.source.is_some());
    }

    #[test]
    fn test_format_display() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        let display = result.format_display();

        assert!(display.contains("test_func"));
        assert!(display.contains("Complexity: 5"));
    }

    #[test]
    fn test_format_text() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        let text = format_text(&[result]);

        assert!(text.contains("Found 1 functions"));
        assert!(text.contains("test_func"));
    }

    #[test]
    fn test_format_markdown() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        let md = format_markdown(&[result]);

        assert!(md.contains("# Search Results"));
        assert!(md.contains("`test_func`"));
    }

    #[test]
    fn test_format_json() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        let json = format_json(&[result]).unwrap();

        assert!(json.contains("\"function_name\": \"test_func\""));
    }

    #[test]
    fn test_parse_query_prefixes_file_only() {
        let (file, func, remaining) = parse_query_prefixes("file:query.rs error handling");
        assert_eq!(file, Some("query.rs".to_string()));
        assert_eq!(func, None);
        assert_eq!(remaining, "error handling");
    }

    #[test]
    fn test_parse_query_prefixes_fn_only() {
        let (file, func, remaining) = parse_query_prefixes("fn:handle_ auth");
        assert_eq!(file, None);
        assert_eq!(func, Some("handle_".to_string()));
        assert_eq!(remaining, "auth");
    }

    #[test]
    fn test_parse_query_prefixes_both() {
        let (file, func, remaining) =
            parse_query_prefixes("file:foo.rs fn:bar baz");
        assert_eq!(file, Some("foo.rs".to_string()));
        assert_eq!(func, Some("bar".to_string()));
        assert_eq!(remaining, "baz");
    }

    #[test]
    fn test_parse_query_prefixes_none() {
        let (file, func, remaining) = parse_query_prefixes("error handling");
        assert_eq!(file, None);
        assert_eq!(func, None);
        assert_eq!(remaining, "error handling");
    }

    #[test]
    fn test_parse_query_prefixes_empty_value() {
        let (file, func, remaining) = parse_query_prefixes("file: fn: hello");
        assert_eq!(file, None);
        assert_eq!(func, None);
        assert_eq!(remaining, "hello");
    }

    #[test]
    fn test_query_result_has_calls_fields() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        assert!(result.calls.is_empty());
        assert!(result.called_by.is_empty());
    }

    #[test]
    fn test_format_text_with_calls() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.calls = vec!["helper_func".to_string()];
        result.called_by = vec!["main".to_string()];
        let text = format_text(&[result]);
        assert!(text.contains("Calls: helper_func"));
        assert!(text.contains("Called by: main"));
    }

    #[test]
    fn test_format_markdown_with_calls() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.calls = vec!["helper_func".to_string(), "other".to_string()];
        let md = format_markdown(&[result]);
        assert!(md.contains("**Calls:** helper_func, other"));
    }

    #[test]
    fn test_format_json_skips_empty_calls() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);
        let json = format_json(&[result]).unwrap();
        // Empty calls/called_by should not appear in JSON (skip_serializing_if)
        assert!(!json.contains("\"calls\""));
        assert!(!json.contains("\"called_by\""));
    }

    /// Build a small in-memory index for testing query paths
    fn build_test_index() -> AgentContextIndex {
        let functions = vec![
            FunctionEntry {
                file_path: "src/handler.rs".to_string(),
                function_name: "handle_error".to_string(),
                signature: "fn handle_error(e: Error)".to_string(),
                doc_comment: Some("Handle API errors gracefully".to_string()),
                source: "fn handle_error(e: Error) { log(e); respond(500); }".to_string(),
                start_line: 10,
                end_line: 15,
                language: "Rust".to_string(),
                quality: QualityMetrics {
                    tdg_score: 1.0,
                    tdg_grade: "A".to_string(),
                    complexity: 3,
                    cognitive_complexity: 3,
                    big_o: "O(1)".to_string(),
                    satd_count: 0,
                    loc: 6,
                    commit_count: 15,
                    churn_score: 0.6,
                },
                checksum: "aaa".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 15,
                churn_score: 0.6,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "src/handler.rs".to_string(),
                function_name: "handle_request".to_string(),
                signature: "fn handle_request(req: Request)".to_string(),
                doc_comment: Some("Process incoming HTTP requests".to_string()),
                source: "fn handle_request(req: Request) { validate(req); handle_error(err); }"
                    .to_string(),
                start_line: 20,
                end_line: 30,
                language: "Rust".to_string(),
                quality: QualityMetrics {
                    tdg_score: 2.0,
                    tdg_grade: "B".to_string(),
                    complexity: 5,
                    cognitive_complexity: 5,
                    big_o: "O(n)".to_string(),
                    satd_count: 0,
                    loc: 11,
                    commit_count: 25,
                    churn_score: 0.8,
                },
                checksum: "bbb".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 25,
                churn_score: 0.8,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "src/utils.rs".to_string(),
                function_name: "validate".to_string(),
                signature: "fn validate(input: &str) -> bool".to_string(),
                doc_comment: Some("Validate input data".to_string()),
                source: "fn validate(input: &str) -> bool { !input.is_empty() }".to_string(),
                start_line: 1,
                end_line: 3,
                language: "Rust".to_string(),
                quality: QualityMetrics {
                    tdg_score: 0.5,
                    tdg_grade: "A".to_string(),
                    complexity: 1,
                    cognitive_complexity: 1,
                    big_o: "O(1)".to_string(),
                    satd_count: 0,
                    loc: 3,
                    commit_count: 3,
                    churn_score: 0.1,
                },
                checksum: "ccc".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 3,
                churn_score: 0.1,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "tests/test_handler.rs".to_string(),
                function_name: "test_error_handling".to_string(),
                signature: "fn test_error_handling()".to_string(),
                doc_comment: None,
                source: "fn test_error_handling() { handle_error(mock_err()); }".to_string(),
                start_line: 1,
                end_line: 5,
                language: "Rust".to_string(),
                quality: QualityMetrics {
                    tdg_score: 1.0,
                    tdg_grade: "A".to_string(),
                    complexity: 1,
                    cognitive_complexity: 1,
                    big_o: "O(1)".to_string(),
                    satd_count: 0,
                    loc: 5,
                    commit_count: 5,
                    churn_score: 0.2,
                },
                checksum: "ddd".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 5,
                churn_score: 0.2,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
            FunctionEntry {
                file_path: "src/utils.rs".to_string(),
                function_name: "new".to_string(),
                signature: "fn new() -> Self".to_string(),
                doc_comment: None,
                source: "fn new() -> Self { Self {} }".to_string(),
                start_line: 10,
                end_line: 12,
                language: "Rust".to_string(),
                quality: QualityMetrics {
                    tdg_score: 0.5,
                    tdg_grade: "A".to_string(),
                    complexity: 1,
                    cognitive_complexity: 1,
                    big_o: "O(1)".to_string(),
                    satd_count: 0,
                    loc: 3,
                    commit_count: 2,
                    churn_score: 0.05,
                },
                checksum: "eee".to_string(),
                definition_type: DefinitionType::default(),
                commit_count: 2,
                churn_score: 0.05,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            },
        ];

        let indices = crate::services::agent_context::function_index::build_indices(&functions);
        let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
        let name_frequency = crate::services::agent_context::function_index::compute_name_frequency(
            &indices.name_index,
            functions.len(),
        );
        let (calls, called_by) =
            crate::services::agent_context::function_index::build_call_graph(&functions, &indices.name_index);
        let graph_metrics =
            crate::services::agent_context::function_index::compute_graph_metrics(functions.len(), &calls, &called_by);

        AgentContextIndex {
            functions,
            name_index: indices.name_index,
            file_index: indices.file_index,
            corpus: indices.corpus,
            corpus_lower,
            name_frequency,
            calls,
            called_by,
            graph_metrics,
            project_root: std::path::PathBuf::from("/test"),
            manifest: crate::services::agent_context::IndexManifest {
                version: "1.2.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 5,
                file_count: 3,
                languages: vec!["Rust".to_string()],
                avg_tdg_score: 1.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        }
    }

    #[test]
    fn test_query_empty_query_returns_error() {
        let index = build_test_index();
        let result = index.query("", QueryOptions::default());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_query_basic_search() {
        let index = build_test_index();
        let results = index
            .query(
                "error handling",
                QueryOptions {
                    limit: 5,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        // handle_error should rank high for "error handling"
        assert_eq!(results[0].function_name, "handle_error");
    }

    #[test]
    fn test_query_with_file_scope() {
        let index = build_test_index();
        let results = index
            .query(
                "file:utils.rs validate",
                QueryOptions {
                    limit: 5,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        // All results must be from utils.rs
        for r in &results {
            assert!(r.file_path.contains("utils.rs"), "unexpected file: {}", r.file_path);
        }
    }

    #[test]
    fn test_query_with_fn_scope() {
        let index = build_test_index();
        let results = index
            .query(
                "fn:handle_ request",
                QueryOptions {
                    limit: 5,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        // All results must have function names starting with "handle_"
        for r in &results {
            assert!(
                r.function_name.starts_with("handle_"),
                "unexpected fn: {}",
                r.function_name
            );
        }
    }

    #[test]
    fn test_query_with_both_scopes() {
        let index = build_test_index();
        let results = index
            .query(
                "file:handler.rs fn:handle_ error",
                QueryOptions {
                    limit: 5,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.file_path.contains("handler.rs"));
            assert!(r.function_name.starts_with("handle_"));
        }
    }

    #[test]
    fn test_query_grade_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    min_grade: Some("A".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        // Only grade A results
        for r in &results {
            assert_eq!(r.tdg_grade, "A", "expected A grade, got {}", r.tdg_grade);
        }
    }

    #[test]
    fn test_query_complexity_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    max_complexity: Some(3),
                    ..Default::default()
                },
            )
            .unwrap();
        for r in &results {
            assert!(r.complexity <= 3, "complexity {} exceeds max 3", r.complexity);
        }
    }

    #[test]
    fn test_query_language_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "validate",
                QueryOptions {
                    limit: 10,
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        for r in &results {
            assert_eq!(r.language, "Rust");
        }
    }

    #[test]
    fn test_query_path_pattern_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    path_pattern: Some("src/".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        for r in &results {
            assert!(r.file_path.contains("src/"));
        }
    }

    #[test]
    fn test_query_loc_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    max_loc: Some(5),
                    ..Default::default()
                },
            )
            .unwrap();
        for r in &results {
            assert!(r.loc <= 5, "loc {} exceeds max 5", r.loc);
        }
    }

    #[test]
    fn test_query_test_function_demotion() {
        let index = build_test_index();
        let results = index
            .query(
                "error handling",
                QueryOptions {
                    limit: 10,
                    ..Default::default()
                },
            )
            .unwrap();
        // test_error_handling should be ranked lower than handle_error
        let handle_pos = results.iter().position(|r| r.function_name == "handle_error");
        let test_pos = results
            .iter()
            .position(|r| r.function_name == "test_error_handling");
        if let (Some(h), Some(t)) = (handle_pos, test_pos) {
            assert!(h < t, "production fn should rank higher than test fn");
        }
    }

    #[test]
    fn test_query_generic_name_demotion() {
        let index = build_test_index();
        let results = index
            .query(
                "new",
                QueryOptions {
                    limit: 10,
                    ..Default::default()
                },
            )
            .unwrap();
        // "new" is a common name - if it appears, its score should be demoted
        if let Some(new_result) = results.iter().find(|r| r.function_name == "new") {
            // Score should be < 1.0 due to name frequency demotion
            assert!(new_result.relevance_score < 1.0);
        }
    }

    #[test]
    fn test_query_include_source() {
        let index = build_test_index();
        let results = index
            .query(
                "validate",
                QueryOptions {
                    limit: 1,
                    include_source: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        assert!(results[0].source.is_some());
    }

    #[test]
    fn test_query_zero_limit_defaults_to_10() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 0,
                    ..Default::default()
                },
            )
            .unwrap();
        // Should not panic with limit=0, should default to 10
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_query_results_have_calls() {
        let index = build_test_index();
        let results = index
            .query(
                "handle_request",
                QueryOptions {
                    limit: 1,
                    ..Default::default()
                },
            )
            .unwrap();
        if let Some(r) = results.iter().find(|r| r.function_name == "handle_request") {
            // handle_request calls validate and handle_error
            assert!(!r.calls.is_empty(), "expected calls to be populated");
        }
    }

    #[test]
    fn test_get_function() {
        let index = build_test_index();
        let result = index.get_function("src/handler.rs", "handle_error");
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.function_name, "handle_error");
        assert_eq!(r.file_path, "src/handler.rs");
        assert!(r.source.is_some()); // get_function always includes source
    }

    #[test]
    fn test_get_function_not_found() {
        let index = build_test_index();
        let result = index.get_function("nonexistent.rs", "foo");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_similar() {
        let index = build_test_index();
        let results = index
            .find_similar("src/handler.rs", "handle_error", 3)
            .unwrap();
        // Should find similar functions (handle_request is similar)
        assert!(!results.is_empty());
        // Should not include self
        assert!(results.iter().all(|r| !(r.file_path == "src/handler.rs" && r.function_name == "handle_error")));
    }

    #[test]
    fn test_find_similar_not_found() {
        let index = build_test_index();
        let result = index.find_similar("nonexistent.rs", "foo", 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_scoped_scoring_empty_candidates() {
        let index = build_test_index();
        let results = index
            .calculate_relevance_scores_scoped("test", &[])
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scoped_scoring_no_query_terms() {
        let index = build_test_index();
        // Only special chars = no query terms after tokenization
        let results = index
            .calculate_relevance_scores_scoped("!@#$%", &[0, 1])
            .unwrap();
        // Returns all candidates with equal score when no terms
        assert_eq!(results.len(), 2);
        assert!((results[0].1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_full_scoring_empty_corpus() {
        let index = AgentContextIndex {
            functions: vec![],
            name_index: HashMap::new(),
            file_index: HashMap::new(),
            corpus: vec![],
            corpus_lower: vec![],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![],
            project_root: std::path::PathBuf::from("/test"),
            manifest: crate::services::agent_context::IndexManifest {
                version: "1.2.0".to_string(),
                built_at: "2025-01-01T00:00:00Z".to_string(),
                project_root: "/test".to_string(),
                function_count: 0,
                file_count: 0,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        };
        let results = index.calculate_relevance_scores("test").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_full_scoring_no_query_terms() {
        let index = build_test_index();
        let results = index.calculate_relevance_scores("!!!").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_dedup_ordered() {
        let items = vec!["a", "b", "a", "c", "b", "d"];
        let result = dedup_ordered(&items);
        assert_eq!(result, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_dedup_ordered_empty() {
        let items: Vec<&str> = vec![];
        let result = dedup_ordered(&items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_is_test_function() {
        let mut entry = create_test_entry("test_something", 1, 0.5);
        assert!(is_test_function(&entry));

        entry.function_name = "handle_request".to_string();
        entry.file_path = "src/handler.rs".to_string();
        assert!(!is_test_function(&entry));

        entry.file_path = "tests/integration.rs".to_string();
        assert!(is_test_function(&entry));

        entry.file_path = "src/tests/mod.rs".to_string();
        assert!(is_test_function(&entry));

        entry.file_path = "src/handler_tests.rs".to_string();
        assert!(is_test_function(&entry));

        entry.file_path = "src/handler_test.rs".to_string();
        assert!(is_test_function(&entry));
    }

    #[test]
    fn test_called_by_test_summarization() {
        let mut index = build_test_index();
        // Simulate many test callers for function 0
        let mut callers = vec![1usize]; // one production caller
        for i in 10..25 {
            // Add fake test function indices
            index.functions.push(FunctionEntry {
                file_path: "tests/t.rs".to_string(),
                function_name: format!("test_case_{i}"),
                signature: format!("fn test_case_{i}()"),
                doc_comment: None,
                source: "fn test() {}".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: format!("t{i}"),
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            });
            callers.push(index.functions.len() - 1);
        }
        index.called_by.insert(0, callers);
        // Rebuild name_index for the new functions
        for (i, f) in index.functions.iter().enumerate() {
            index.name_index.entry(f.function_name.clone()).or_default().push(i);
        }

        let result = QueryResult::from_entry_with_context(
            &index.functions[0],
            0,
            &index,
            0.9,
            false,
        );
        // Should have production caller + test summary
        assert!(result.called_by.iter().any(|s| s.contains("tests)")));
        // Should not list individual test_case_N names
        assert!(!result.called_by.iter().any(|s| s.starts_with("test_case_")));
    }

    #[test]
    fn test_called_by_production_cap() {
        let mut index = build_test_index();
        // Simulate >10 production callers for function 0
        let mut callers = Vec::new();
        for i in 10..25 {
            index.functions.push(FunctionEntry {
                file_path: "src/callers.rs".to_string(),
                function_name: format!("caller_{i}"),
                signature: format!("fn caller_{i}()"),
                doc_comment: None,
                source: "fn caller() {}".to_string(),
                start_line: 1,
                end_line: 1,
                language: "Rust".to_string(),
                quality: QualityMetrics::default(),
                checksum: format!("c{i}"),
                definition_type: DefinitionType::default(),
                commit_count: 0,
                churn_score: 0.0,
                clone_count: 0,
                pattern_diversity: 0.0,
                fault_annotations: Vec::new(),
            });
            callers.push(index.functions.len() - 1);
            index
                .name_index
                .entry(format!("caller_{i}"))
                .or_default()
                .push(index.functions.len() - 1);
        }
        index.called_by.insert(0, callers);

        let result = QueryResult::from_entry_with_context(
            &index.functions[0],
            0,
            &index,
            0.9,
            false,
        );
        // Should cap at 10 + "(+N more)"
        assert!(result.called_by.iter().any(|s| s.contains("more)")));
        // Total entries should be 10 visible + 1 summary = 11
        assert!(result.called_by.len() <= 12);
    }

    #[test]
    fn test_enrich_with_churn() {
        use super::enrich_with_churn;

        let entry = create_test_entry("test_func", 5, 1.5);
        let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

        // Initially no churn data
        assert_eq!(results[0].commit_count, 0);
        assert!((results[0].churn_score - 0.0).abs() < 0.01);

        // Build churn map
        let mut churn_map = HashMap::new();
        churn_map.insert("test.rs".to_string(), (42u32, 0.75f32));

        // Enrich results
        enrich_with_churn(&mut results, &churn_map);

        // Verify churn data was applied
        assert_eq!(results[0].commit_count, 42);
        assert!((results[0].churn_score - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_enrich_with_churn_no_match() {
        use super::enrich_with_churn;

        let entry = create_test_entry("test_func", 5, 1.5);
        let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

        // Churn map with different file
        let mut churn_map = HashMap::new();
        churn_map.insert("other.rs".to_string(), (100u32, 0.9f32));

        // Enrich results - should not match
        enrich_with_churn(&mut results, &churn_map);

        // Verify churn data was NOT changed (no match)
        assert_eq!(results[0].commit_count, 0);
        assert!((results[0].churn_score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_format_text_with_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.commit_count = 25;
        result.churn_score = 0.8;

        let text = format_text(&[result]);
        assert!(text.contains("🔥 Hot"));
        assert!(text.contains("25 commits"));
        assert!(text.contains("80%"));
    }

    #[test]
    fn test_format_text_with_code_shows_metrics() {
        let entry = create_test_entry("test_func", 15, 3.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.satd_count = 2;
        result.commit_count = 30;
        result.churn_score = 0.7;

        let text = format_text_with_code(&[result]);
        // Should show complexity (format: "C:15" without space)
        assert!(text.contains("C:15"), "missing complexity");
        // Should show SATD warning as "⚠2" (warning symbol + count)
        assert!(text.contains("⚠2"), "missing SATD");
        // Should show churn (high commit count shown as "🔥" indicator)
        assert!(text.contains("🔥") || text.contains("30"), "missing churn indicator");
        // Should show function name in header
        assert!(text.contains("test_func"), "missing function name");
    }

    #[test]
    fn test_format_text_with_code_minimal_metrics() {
        let entry = create_test_entry("simple_func", 3, 1.0);
        let result = QueryResult::from_entry(&entry, 0.9, true);

        let text = format_text_with_code(&[result]);
        // Should still show complexity (format: "C:3" without space)
        assert!(text.contains("C:3"), "missing complexity");
        // Should NOT show SATD (is 0)
        assert!(!text.contains("SATD"), "should not show SATD when 0");
        // Should NOT show churn (is 0)
        assert!(!text.contains("commits"), "should not show churn when 0");
    }

    #[test]
    fn test_format_text_with_low_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.commit_count = 5;
        result.churn_score = 0.2; // Below 0.5 threshold

        let text = format_text(&[result]);
        assert!(!text.contains("🔥 Hot"));
        assert!(text.contains("Commits: 5"));
    }

    #[test]
    fn test_format_markdown_with_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.commit_count = 30;
        result.churn_score = 0.9;

        let md = format_markdown(&[result]);
        assert!(md.contains("🔥 **Hot"));
        assert!(md.contains("30 commits"));
        assert!(md.contains("90%"));
    }

    #[test]
    fn test_query_rank_by_pagerank() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    rank_by: RankBy::PageRank,
                    ..Default::default()
                },
            )
            .unwrap();
        // Should return results ordered by PageRank
        assert!(!results.is_empty());
        // Verify descending PageRank order
        for w in results.windows(2) {
            assert!(
                w[0].pagerank >= w[1].pagerank || (w[0].pagerank - w[1].pagerank).abs() < 1e-6,
                "PageRank not descending: {} vs {}",
                w[0].pagerank,
                w[1].pagerank,
            );
        }
    }

    #[test]
    fn test_query_rank_by_centrality() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    rank_by: RankBy::Centrality,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_query_rank_by_indegree() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    rank_by: RankBy::InDegree,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!results.is_empty());
        // Verify descending in-degree order
        for w in results.windows(2) {
            assert!(
                w[0].in_degree >= w[1].in_degree,
                "InDegree not descending: {} vs {}",
                w[0].in_degree,
                w[1].in_degree,
            );
        }
    }

    #[test]
    fn test_query_min_pagerank_filter() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    min_pagerank: Some(0.0),
                    ..Default::default()
                },
            )
            .unwrap();
        // All results should have pagerank >= 0.0 (all pass)
        for r in &results {
            assert!(r.pagerank >= 0.0);
        }

        // Very high threshold should filter everything
        let results_strict = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    min_pagerank: Some(1.0),
                    ..Default::default()
                },
            )
            .unwrap();
        // No function will have pagerank >= 1.0
        assert!(results_strict.is_empty());
    }

    #[test]
    fn test_rankby_from_str() {
        assert_eq!("relevance".parse::<RankBy>().unwrap(), RankBy::Relevance);
        assert_eq!("rel".parse::<RankBy>().unwrap(), RankBy::Relevance);
        assert_eq!("pagerank".parse::<RankBy>().unwrap(), RankBy::PageRank);
        assert_eq!("pr".parse::<RankBy>().unwrap(), RankBy::PageRank);
        assert_eq!("importance".parse::<RankBy>().unwrap(), RankBy::PageRank);
        assert_eq!("centrality".parse::<RankBy>().unwrap(), RankBy::Centrality);
        assert_eq!("degree".parse::<RankBy>().unwrap(), RankBy::Centrality);
        assert_eq!("indegree".parse::<RankBy>().unwrap(), RankBy::InDegree);
        assert_eq!("callers".parse::<RankBy>().unwrap(), RankBy::InDegree);
        assert!("invalid".parse::<RankBy>().is_err());
    }

    #[test]
    fn test_format_text_with_clones() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.clone_count = 3;
        result.duplication_score = 0.85;

        let text = format_text(&[result]);
        assert!(text.contains("📋 Clones: 3"), "missing clones in text");
        assert!(text.contains("85%"), "missing duplication score");
    }

    #[test]
    fn test_format_text_with_entropy() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.pattern_diversity = 0.2;

        let text = format_text(&[result]);
        assert!(text.contains("🔄 Repetitive"), "missing entropy in text");
    }

    #[test]
    fn test_format_text_with_fault_annotations() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.fault_annotations = vec![
            "BH001: Boundary condition at line 10".to_string(),
            "BH002: Arithmetic overflow at line 15".to_string(),
        ];

        let text = format_text(&[result]);
        assert!(text.contains("BH001"), "missing fault annotation");
        assert!(text.contains("BH002"), "missing fault annotation");
    }

    #[test]
    fn test_format_text_with_graph_metrics() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.pagerank = 0.05;
        result.in_degree = 3;
        result.out_degree = 2;

        let text = format_text(&[result]);
        assert!(text.contains("PageRank"), "missing graph metrics");
        assert!(text.contains("In-Degree"), "missing in-degree");
    }

    #[test]
    fn test_format_text_with_large_loc() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.loc = 100;

        let text = format_text(&[result]);
        assert!(text.contains("LOC: 100"), "missing LOC for large function");
    }

    #[test]
    fn test_format_markdown_with_clones_and_entropy() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.clone_count = 2;
        result.duplication_score = 0.7;
        result.pattern_diversity = 0.15;
        result.pagerank = 0.01;
        result.in_degree = 5;
        result.out_degree = 3;
        result.doc_comment = Some("Test doc".to_string());

        let md = format_markdown(&[result]);
        assert!(md.contains("📋 **Clones:"), "missing clones in markdown");
        assert!(md.contains("🔄 **Repetitive"), "missing entropy in markdown");
        assert!(md.contains("**Graph:**"), "missing graph metrics");
        assert!(md.contains("**Documentation:**"), "missing doc comment");
    }

    #[test]
    fn test_format_markdown_with_low_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, false);
        result.commit_count = 3;
        result.churn_score = 0.2;

        let md = format_markdown(&[result]);
        assert!(md.contains("Commits: 3"), "missing low churn commits");
        assert!(!md.contains("🔥"), "should not show fire for low churn");
    }

    #[test]
    fn test_format_text_with_code_clones_and_faults() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.clone_count = 3;
        result.fault_annotations = vec![
            "BH001: Boundary condition at line 10".to_string(),
            "BH002: Arithmetic overflow at line 15".to_string(),
            "BH003: Other pattern at line 20".to_string(),
        ];

        let text = format_text_with_code(&[result]);
        assert!(text.contains("📋"), "missing clone indicator");
        assert!(text.contains("🐛"), "missing fault indicator");
    }

    #[test]
    fn test_format_text_with_code_call_graph_truncation() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        // More than 5 calls -> should truncate
        result.calls = (0..8).map(|i| format!("func_{i}")).collect();
        // More than 3 called_by -> should truncate
        result.called_by = (0..6).map(|i| format!("caller_{i}")).collect();

        let text = format_text_with_code(&[result]);
        assert!(text.contains("(+3 more)"), "calls not truncated at 5");
        assert!(text.contains("(+3 more)"), "called_by not truncated at 3");
    }

    #[test]
    fn test_format_text_with_code_doc_truncation() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.doc_comment = Some("A".repeat(150)); // >100 chars

        let text = format_text_with_code(&[result]);
        assert!(text.contains("..."), "long doc comment not truncated");
    }

    #[test]
    fn test_format_text_with_code_no_source() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let result = QueryResult::from_entry(&entry, 0.9, false);

        let text = format_text_with_code(&[result]);
        assert!(text.contains("--include-source"), "missing hint for no source");
    }

    #[test]
    fn test_format_text_with_code_high_pagerank() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.pagerank = 0.005; // scaled: 50 -> >= 10 threshold

        let text = format_text_with_code(&[result]);
        assert!(text.contains("★"), "missing high pagerank star");
    }

    #[test]
    fn test_format_text_with_code_medium_pagerank() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.pagerank = 0.0005; // scaled: 5 -> >= 1 threshold

        let text = format_text_with_code(&[result]);
        assert!(text.contains("★"), "missing medium pagerank star");
    }

    #[test]
    fn test_format_text_with_code_high_indegree() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.in_degree = 10;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("↓10"), "missing high in-degree");
    }

    #[test]
    fn test_format_text_with_code_low_indegree() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.in_degree = 2;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("↓2"), "missing low in-degree");
    }

    #[test]
    fn test_format_text_with_code_medium_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.commit_count = 15;
        result.churn_score = 0.4;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("15c"), "missing medium churn");
        assert!(text.contains("40%"), "missing churn percentage");
    }

    #[test]
    fn test_format_text_with_code_low_churn() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.commit_count = 3;
        result.churn_score = 0.1;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("3c"), "missing low churn");
    }

    #[test]
    fn test_format_text_with_code_high_entropy() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.pattern_diversity = 0.9;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("H:90%"), "missing high entropy indicator");
    }

    #[test]
    fn test_format_text_with_code_low_entropy() {
        let entry = create_test_entry("test_func", 5, 1.5);
        let mut result = QueryResult::from_entry(&entry, 0.9, true);
        result.pattern_diversity = 0.2;

        let text = format_text_with_code(&[result]);
        assert!(text.contains("🔄"), "missing low entropy indicator");
    }

    #[test]
    fn test_format_json_with_empty_results() {
        let json = format_json(&[]).unwrap();
        assert_eq!(json.trim(), "[]");
    }

    #[test]
    fn test_format_markdown_empty() {
        let md = format_markdown(&[]);
        assert!(md.contains("0 functions"));
    }

    #[test]
    fn test_format_text_empty() {
        let text = format_text(&[]);
        assert!(text.contains("Found 0 functions"));
    }

    #[test]
    fn test_format_text_with_code_empty() {
        let text = format_text_with_code(&[]);
        assert!(text.is_empty());
    }

    #[test]
    fn test_query_combined_filters() {
        let index = build_test_index();
        let results = index
            .query(
                "handle",
                QueryOptions {
                    limit: 10,
                    min_grade: Some("A".to_string()),
                    max_complexity: Some(3),
                    language: Some("Rust".to_string()),
                    path_pattern: Some("src/".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        for r in &results {
            assert_eq!(r.tdg_grade, "A");
            assert!(r.complexity <= 3);
            assert_eq!(r.language, "Rust");
            assert!(r.file_path.contains("src/"));
        }
    }

    #[test]
    fn test_build_churn_map() {
        use super::build_churn_map;
        use crate::models::churn::FileChurnMetrics;

        let metrics = vec![
            FileChurnMetrics {
                path: std::path::PathBuf::from("src/foo.rs"),
                relative_path: "src/foo.rs".to_string(),
                commit_count: 10,
                unique_authors: vec!["author1".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.5,
                last_modified: chrono::Utc::now(),
                first_seen: chrono::Utc::now(),
            },
            FileChurnMetrics {
                path: std::path::PathBuf::from("src/bar.rs"),
                relative_path: "src/bar.rs".to_string(),
                commit_count: 25,
                unique_authors: vec!["author2".to_string()],
                additions: 200,
                deletions: 100,
                churn_score: 0.8,
                last_modified: chrono::Utc::now(),
                first_seen: chrono::Utc::now(),
            },
        ];

        let map = build_churn_map(&metrics);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("src/foo.rs"), Some(&(10, 0.5)));
        assert_eq!(map.get("src/bar.rs"), Some(&(25, 0.8)));
    }

    #[test]
    fn test_from_entry_with_context_basic() {
        use crate::services::agent_context::function_index::{
            AgentContextIndex, GraphMetrics, IndexManifest,
        };
        use std::path::PathBuf;

        let entry = create_test_entry("my_func", 5, 1.5);
        let index = AgentContextIndex {
            functions: vec![entry.clone()],
            name_index: HashMap::new(),
            file_index: HashMap::new(),
            corpus: vec!["my_func".to_string()],
            corpus_lower: vec!["my_func".to_string()],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![GraphMetrics {
                pagerank: 0.42,
                centrality: 0.3,
                in_degree: 5,
                out_degree: 2,
            }],
            project_root: PathBuf::from("/tmp"),
            manifest: IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "test".to_string(),
                project_root: "/tmp".to_string(),
                function_count: 0,
                file_count: 0,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        };

        let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
        assert!((result.pagerank - 0.42).abs() < 0.001);
        assert_eq!(result.in_degree, 5);
        assert_eq!(result.out_degree, 2);
    }

    #[test]
    fn test_from_entry_with_context_out_of_bounds() {
        use crate::services::agent_context::function_index::{
            AgentContextIndex, IndexManifest,
        };
        use std::path::PathBuf;

        let entry = create_test_entry("my_func", 5, 1.5);
        let index = AgentContextIndex {
            functions: vec![entry.clone()],
            name_index: HashMap::new(),
            file_index: HashMap::new(),
            corpus: vec!["my_func".to_string()],
            corpus_lower: vec!["my_func".to_string()],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![], // empty - out of bounds
            project_root: PathBuf::from("/tmp"),
            manifest: IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "test".to_string(),
                project_root: "/tmp".to_string(),
                function_count: 0,
                file_count: 0,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        };

        let result = QueryResult::from_entry_with_context(&entry, 99, &index, 0.9, false);
        // Should not panic, pagerank stays 0
        assert!((result.pagerank - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_from_entry_with_context_callers_capping() {
        use crate::services::agent_context::function_index::{
            AgentContextIndex, GraphMetrics, IndexManifest,
        };
        use std::path::PathBuf;

        let entry = create_test_entry("target", 5, 1.5);
        // Create 15 caller functions + 3 test callers
        let mut functions = vec![entry.clone()];
        let mut called_by_map = HashMap::new();
        let mut callers = vec![];
        for i in 0..15 {
            functions.push(create_test_entry(&format!("caller_{}", i), 1, 0.5));
            callers.push(i + 1); // indices 1..=15
        }
        for i in 0..3 {
            functions.push(create_test_entry(&format!("test_caller_{}", i), 1, 0.5));
            callers.push(16 + i); // indices 16..=18
        }
        called_by_map.insert(0usize, callers);

        let index = AgentContextIndex {
            functions,
            name_index: HashMap::new(),
            file_index: HashMap::new(),
            corpus: vec![],
            corpus_lower: vec![],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: called_by_map,
            graph_metrics: vec![GraphMetrics {
                pagerank: 0.1,
                centrality: 0.0,
                in_degree: 18,
                out_degree: 0,
            }],
            project_root: PathBuf::from("/tmp"),
            manifest: IndexManifest {
                version: "1.3.0".to_string(),
                built_at: "test".to_string(),
                project_root: "/tmp".to_string(),
                function_count: 0,
                file_count: 0,
                languages: vec![],
                avg_tdg_score: 0.0,
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
        };

        let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
        // Should have 10 prod callers + "(+5 more)" + "(+3 tests)" = 12 entries
        assert!(result.called_by.len() <= 12);
        // Should have the capping message
        let has_more = result
            .called_by
            .iter()
            .any(|s| s.starts_with("(+") && s.ends_with("more)"));
        assert!(has_more, "Should have (+N more) message");
        let has_tests = result
            .called_by
            .iter()
            .any(|s| s.contains("tests)"));
        assert!(has_tests, "Should have (+N tests) message");
    }

    #[test]
    fn test_dedup_ordered_preserves_first() {
        let input: Vec<&str> = vec!["a", "b", "c", "b", "d", "c", "e"];
        let deduped = dedup_ordered(&input);
        assert_eq!(deduped, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_dedup_ordered_empty_input() {
        let input: Vec<&str> = vec![];
        let deduped = dedup_ordered(&input);
        assert!(deduped.is_empty());
    }
}
