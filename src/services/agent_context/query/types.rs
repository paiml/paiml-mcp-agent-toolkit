#![cfg_attr(coverage_nightly, coverage(off))]

use super::coverage_exclusion::CoverageExclusion;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry, GraphMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Deduplicate a list of strings while preserving first-seen order
pub(super) fn dedup_ordered(items: &[&str]) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|s| seen.insert(**s))
        .map(|s| s.to_string())
        .collect()
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
    /// Rank by coverage impact ROI (missed_lines * pagerank * 1/complexity)
    Impact,
    /// Rank by cross-project importance (PageRank boosted by cross-project callers)
    CrossProject,
    /// Rank by TDG churn-weighted priority (tdg_score * (1 + churn_score))
    Priority,
}

impl std::str::FromStr for RankBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "relevance" | "rel" => Ok(RankBy::Relevance),
            "pagerank" | "pr" | "importance" => Ok(RankBy::PageRank),
            "centrality" | "degree" => Ok(RankBy::Centrality),
            "indegree" | "callers" => Ok(RankBy::InDegree),
            "impact" | "roi" | "coverage" => Ok(RankBy::Impact),
            "cross-project" | "crossproject" | "xproject" => Ok(RankBy::CrossProject),
            "priority" | "tdg-priority" | "churn-priority" => Ok(RankBy::Priority),
            _ => Err(format!("Unknown rank-by: '{}'. Valid: relevance, pagerank, centrality, indegree, impact, cross-project, priority", s)),
        }
    }
}

/// Search mode for query
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchMode {
    /// Semantic TF-scoring (default)
    #[default]
    Semantic,
    /// Regex pattern matching against source/signature/name
    Regex,
    /// Exact literal string matching
    Literal,
}

/// Case sensitivity mode
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseSensitivity {
    /// Smart-case: case-sensitive if query contains uppercase (like rg)
    #[default]
    Smart,
    /// Always case-sensitive
    Sensitive,
    /// Always case-insensitive
    Insensitive,
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
    /// Search mode: semantic, regex, or literal
    #[serde(default)]
    pub search_mode: SearchMode,
    /// Case sensitivity control
    #[serde(default)]
    pub case_sensitivity: CaseSensitivity,
    /// Exclude results matching this content pattern
    pub exclude_pattern: Option<String>,
    /// Exclude results from files matching these glob patterns (repeatable)
    pub exclude_file_pattern: Vec<String>,
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
    /// Line coverage percentage (0.0-100.0)
    #[serde(default)]
    pub line_coverage_pct: f32,
    /// Number of covered (instrumented + executed) lines
    #[serde(default)]
    pub lines_covered: u32,
    /// Total instrumented lines in function
    #[serde(default)]
    pub lines_total: u32,
    /// Number of uncovered (instrumented but not executed) lines
    #[serde(default)]
    pub missed_lines: u32,
    /// ROI impact score: missed_lines * pagerank * (1/complexity)
    #[serde(default)]
    pub impact_score: f32,
    /// Coverage status: "no_data", "uncovered", "partial", "full"
    #[serde(default)]
    pub coverage_status: String,
    /// Coverage change vs baseline (positive = improved, negative = regressed)
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub coverage_diff: f32,
    /// Why this function is excluded from coverage tracking (if any)
    #[serde(default, skip_serializing_if = "CoverageExclusion::is_none")]
    pub coverage_exclusion: CoverageExclusion,
    /// Whether this result was filtered out by coverage exclusion detection
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub coverage_excluded: bool,
    /// Number of callers from different projects (workspace cross-project search)
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub cross_project_callers: u32,
    /// I/O classification: "PURE" or "IO"
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub io_classification: String,
    /// Detected I/O patterns (PRINT, FS, PROCESS, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub io_patterns: Vec<String>,
    /// Suggested extraction module name
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub suggested_module: String,
}

fn is_zero_f32(v: &f32) -> bool {
    *v == 0.0
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
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
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: CoverageExclusion::None,
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
        }
    }

    /// Create from function entry with graph metrics but NO call graph.
    ///
    /// Used by coverage-gaps mode where call graph data is not displayed.
    /// Avoids 71K × 2 HashMap lookups + Vec clones for unused data.
    pub fn from_entry_with_metrics(
        entry: &FunctionEntry,
        func_idx: usize,
        graph_metrics: &[GraphMetrics],
        relevance: f32,
    ) -> Self {
        let mut result = Self::from_entry(entry, relevance, false);
        if func_idx < graph_metrics.len() {
            let metrics = &graph_metrics[func_idx];
            result.pagerank = metrics.pagerank;
            result.in_degree = metrics.in_degree;
            result.out_degree = metrics.out_degree;
        }
        result
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

        // Cross-project caller count (non-zero only in workspace mode)
        result.cross_project_callers = index.count_cross_project_callers(func_idx);

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
            result.called_by.push(format!("(+{} tests)", tests.len()));
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
