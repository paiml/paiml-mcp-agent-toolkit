//! Query Engine for Agent Context
//!
//! Provides semantic search with quality filtering over the function index.

use super::{AgentContextIndex, FunctionEntry};
use serde::{Deserialize, Serialize};

/// Check if function is a test (test_ prefix or in tests/ directory)
fn is_test_function(func: &FunctionEntry) -> bool {
    func.function_name.starts_with("test_")
        || func.file_path.starts_with("tests/")
        || func.file_path.contains("/tests/")
        || func.file_path.contains("_tests.")
        || func.file_path.contains("_test.")
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
}

impl QueryResult {
    /// Create from function entry with relevance score
    pub fn from_entry(entry: &FunctionEntry, relevance: f32, include_source: bool) -> Self {
        Self {
            file_path: entry.file_path.clone(),
            function_name: entry.function_name.clone(),
            signature: entry.signature.clone(),
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
        }
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

impl AgentContextIndex {
    /// Query the index with semantic search
    ///
    /// # Arguments
    /// * `query` - Natural language query
    /// * `options` - Query options for filtering
    ///
    /// # Returns
    /// Ranked list of matching functions
    pub fn query(&self, query: &str, options: QueryOptions) -> Result<Vec<QueryResult>, String> {
        if query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let limit = if options.limit == 0 { 10 } else { options.limit };

        // Calculate relevance scores using term matching
        let scores = self.calculate_relevance_scores(query)?;

        // Combine with quality score for final ranking
        let mut ranked: Vec<(usize, f32)> = scores
            .into_iter()
            .enumerate()
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

                (idx, combined)
            })
            .collect();

        // Sort by combined score (descending)
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<QueryResult> = ranked
            .into_iter()
            .take(limit)
            .map(|(idx, score)| {
                QueryResult::from_entry(&self.functions[idx], score, options.include_source)
            })
            .collect();

        Ok(results)
    }

    /// Get function by file and name
    pub fn get_function(&self, file_path: &str, function_name: &str) -> Option<QueryResult> {
        self.functions
            .iter()
            .find(|f| f.file_path == file_path && f.function_name == function_name)
            .map(|f| QueryResult::from_entry(f, 1.0, true))
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
            .enumerate()
            .filter(|(idx, _)| *idx != ref_idx) // Exclude self
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<QueryResult> = ranked
            .into_iter()
            .take(limit)
            .map(|(idx, score)| QueryResult::from_entry(&self.functions[idx], score, false))
            .collect();

        Ok(results)
    }

    /// Calculate term-based relevance scores using pre-computed lowercase corpus.
    ///
    /// Uses pre-computed `corpus_lower` to avoid per-query `.to_lowercase()` on
    /// all 42K+ documents. Linear scan with TF scoring for quality ranking.
    fn calculate_relevance_scores(&self, query: &str) -> Result<Vec<f32>, String> {
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
            return Ok(vec![0.0; self.corpus.len()]);
        }

        // Score each document using pre-computed lowercase corpus
        let mut scores = vec![0.0f32; self.corpus.len()];

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
                scores[doc_idx] = term_score / query_terms.len() as f32;
            }
        }

        // Normalize scores to 0-1 range
        let max_score = scores.iter().cloned().fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for score in &mut scores {
                *score /= max_score;
            }
        }

        Ok(scores)
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
        output.push_str(&format!(
            "**Quality:** TDG {} ({:.1}) | Complexity: {} | Big-O: {}\n\n",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        ));

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("**Documentation:** {}\n\n", doc));
        }

        output.push_str(&format!("**Relevance:** {:.2}\n\n", r.relevance_score));
        output.push_str("---\n\n");
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
        output.push_str(&format!(
            "   TDG: {} ({:.1}) | Complexity: {} | Big-O: {}\n",
            r.tdg_grade, r.tdg_score, r.complexity, r.big_o
        ));

        if let Some(doc) = &r.doc_comment {
            output.push_str(&format!("   Doc: {}\n", doc));
        }

        output.push_str(&format!("   Relevance: {:.2}\n\n", r.relevance_score));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_context::QualityMetrics;

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
            },
            checksum: "abc123".to_string(),
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
}
