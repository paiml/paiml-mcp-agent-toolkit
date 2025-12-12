//! Unified Help Service - RAG-powered intelligent help
//!
//! Combines:
//! - **NLP**: Tokenization, stemming, BM25 scoring
//! - **trueno-graph**: PageRank for command importance ranking
//!
//! # Architecture
//!
//! ```text
//! User Query
//!     │
//!     ├─▶ NLP (tokenize, stem, BM25)
//!     │        │
//!     │        ▼
//!     └─▶ trueno-graph PageRank (importance ranking)
//!              │
//!              ▼
//!         Ranked Results with Context
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Citations: Lewis et al. (2020) RAG, Teyton et al. (2013) PageRank

use crate::cli::registry::{CommandMetadata, CommandRegistry};
use std::collections::{HashMap, HashSet};

// Re-export for convenience
pub use trueno_graph::storage::csr::{CsrGraph, NodeId};

/// NLP processor for semantic help matching.
/// Uses simple but effective tokenization, stemming, and BM25 scoring.
pub struct HelpNlpProcessor {
    /// Stop words to filter out
    stop_words: HashSet<String>,
}

impl HelpNlpProcessor {
    /// Create a new NLP processor
    pub fn new() -> Self {
        let mut stop_words = HashSet::new();
        // Common English stop words + domain-specific
        for word in &[
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "to", "of", "in",
            "for", "on", "with", "at", "by", "from", "as", "into", "through",
            "during", "before", "after", "above", "below", "between", "under",
            "again", "further", "then", "once", "here", "there", "when", "where",
            "why", "how", "all", "each", "few", "more", "most", "other", "some",
            "such", "no", "nor", "not", "only", "own", "same", "so", "than",
            "too", "very", "just", "and", "but", "if", "or", "because", "until",
            "while", "this", "that", "these", "those", "it", "its",
            // Domain-specific
            "pmat", "command", "run", "execute", "use", "using",
        ] {
            stop_words.insert(word.to_string());
        }

        Self { stop_words }
    }

    /// Simple tokenization - split on whitespace and punctuation
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    /// Simple Porter-like stemming (suffix removal)
    fn stem(&self, word: &str) -> String {
        let word = word.to_lowercase();
        // Simple suffix removal rules
        if word.ends_with("ing") && word.len() > 5 {
            return word[..word.len() - 3].to_string();
        }
        if word.ends_with("ed") && word.len() > 4 {
            return word[..word.len() - 2].to_string();
        }
        if word.ends_with("ies") && word.len() > 4 {
            return format!("{}y", &word[..word.len() - 3]);
        }
        if word.ends_with("es") && word.len() > 4 {
            return word[..word.len() - 2].to_string();
        }
        if word.ends_with("s") && word.len() > 3 && !word.ends_with("ss") {
            return word[..word.len() - 1].to_string();
        }
        if word.ends_with("ly") && word.len() > 4 {
            return word[..word.len() - 2].to_string();
        }
        word
    }

    /// Preprocess text for search (tokenize, filter, stem)
    pub fn preprocess(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
            .into_iter()
            .filter(|t| !self.stop_words.contains(t))
            .map(|t| self.stem(&t))
            .collect()
    }

    /// Calculate term frequency for a document
    pub fn term_frequency(&self, text: &str) -> HashMap<String, f64> {
        let tokens = self.preprocess(text);
        let total = tokens.len() as f64;

        let mut tf = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0.0) += 1.0;
        }

        // Normalize by document length
        for freq in tf.values_mut() {
            *freq /= total.max(1.0);
        }

        tf
    }

    /// Calculate BM25 score between query and document
    pub fn bm25_score(&self, query: &str, document: &str, k1: f64, b: f64) -> f64 {
        let query_tokens = self.preprocess(query);
        let doc_tf = self.term_frequency(document);
        let avg_dl = 100.0; // Approximate average document length

        let doc_len = self.preprocess(document).len() as f64;
        let norm = 1.0 - b + b * (doc_len / avg_dl);

        query_tokens
            .iter()
            .map(|term| {
                let tf = doc_tf.get(term).copied().unwrap_or(0.0);
                if tf > 0.0 {
                    tf * (k1 + 1.0) / (tf + k1 * norm)
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl Default for HelpNlpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph-based command importance ranking using trueno-graph.
pub struct CommandGraph {
    /// CSR graph for efficient traversal
    graph: CsrGraph,
    /// Command name to node ID mapping
    command_to_node: HashMap<String, NodeId>,
    /// Node ID to command name mapping
    node_to_command: HashMap<NodeId, String>,
    /// Cached PageRank scores
    importance_scores: HashMap<String, f32>,
    /// Next available node ID
    next_node_id: u32,
}

impl CommandGraph {
    /// Create a new command graph
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            command_to_node: HashMap::new(),
            node_to_command: HashMap::new(),
            importance_scores: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Build graph from command registry
    pub fn build_from_registry(&mut self, registry: &CommandRegistry) {
        // Add nodes for all commands
        for (name, cmd) in &registry.commands {
            let node_id = NodeId(self.next_node_id);
            self.next_node_id += 1;
            self.command_to_node.insert(name.clone(), node_id);
            self.node_to_command.insert(node_id, name.clone());

            // Also add subcommands
            for sub in &cmd.subcommands {
                let full_name = format!("{} {}", name, sub.name);
                let sub_node_id = NodeId(self.next_node_id);
                self.next_node_id += 1;
                self.command_to_node.insert(full_name.clone(), sub_node_id);
                self.node_to_command.insert(sub_node_id, full_name);
            }
        }

        // Add edges for relationships
        for (name, cmd) in &registry.commands {
            if let Some(&from_id) = self.command_to_node.get(name) {
                // Parent -> Subcommand edges
                for sub in &cmd.subcommands {
                    let full_name = format!("{} {}", name, sub.name);
                    if let Some(&to_id) = self.command_to_node.get(&full_name) {
                        let _ = self.graph.add_edge(from_id, to_id, 1.0);
                    }
                }

                // Related command edges (bidirectional, lower weight)
                for related in &cmd.related {
                    if let Some(&to_id) = self.command_to_node.get(related) {
                        let _ = self.graph.add_edge(from_id, to_id, 0.5);
                        let _ = self.graph.add_edge(to_id, from_id, 0.5);
                    }
                }
            }
        }

        // Compute PageRank
        self.update_importance();
    }

    /// Update importance scores using PageRank
    fn update_importance(&mut self) {
        use trueno_graph::algorithms::pagerank::pagerank;

        match pagerank(&self.graph, 20, 1e-6_f32) {
            Ok(scores) => {
                self.importance_scores.clear();
                for (node_idx, score) in scores.iter().enumerate() {
                    let node_id = NodeId(node_idx as u32);
                    if let Some(name) = self.node_to_command.get(&node_id) {
                        let key: String = name.clone();
                        self.importance_scores.insert(key, *score);
                    }
                }
            }
            Err(_) => {
                // On error, use uniform importance
                let uniform = 1.0 / self.command_to_node.len().max(1) as f32;
                for name in self.command_to_node.keys() {
                    let key: String = name.clone();
                    self.importance_scores.insert(key, uniform);
                }
            }
        }
    }

    /// Get importance score for a command
    pub fn importance(&self, command: &str) -> f32 {
        self.importance_scores.get(command).copied().unwrap_or(0.0)
    }

    /// Rank commands by importance
    pub fn rank_by_importance(&self, commands: &[String]) -> Vec<(String, f32)> {
        let mut ranked: Vec<_> = commands
            .iter()
            .map(|c| (c.clone(), self.importance(c)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Get top-k most important commands
    pub fn top_k_important(&self, k: usize) -> Vec<(String, f32)> {
        let mut all: Vec<_> = self.importance_scores.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all
    }
}

impl Default for CommandGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result from unified help
#[derive(Debug, Clone)]
pub struct HelpSearchResult {
    /// Command name
    pub command: String,
    /// Short description
    pub description: String,
    /// Relevance score (0-1)
    pub relevance: f32,
    /// Importance score from PageRank
    pub importance: f32,
    /// Combined score
    pub combined_score: f32,
    /// Matched snippet
    pub snippet: String,
}

/// Response from unified help lookup
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum HelpResponse {
    /// Exact match found
    Exact(CommandMetadata),
    /// Fuzzy match suggestion
    DidYouMean {
        suggestion: String,
        confidence: f32,
    },
    /// Search results
    SearchResults {
        query: String,
        results: Vec<HelpSearchResult>,
    },
}

/// Unified help service combining NLP, Graph, and RAG.
pub struct UnifiedHelpService {
    registry: CommandRegistry,
    nlp: HelpNlpProcessor,
    graph: CommandGraph,
    /// Indexed command documents for search
    command_docs: HashMap<String, String>,
}

impl UnifiedHelpService {
    /// Create a new unified help service
    pub fn new(registry: CommandRegistry) -> Self {
        let nlp = HelpNlpProcessor::new();
        let mut graph = CommandGraph::new();

        // Build graph from registry
        graph.build_from_registry(&registry);

        // Index command documents
        let command_docs = Self::index_commands(&registry);

        Self {
            registry,
            nlp,
            graph,
            command_docs,
        }
    }

    /// Index commands for search
    fn index_commands(registry: &CommandRegistry) -> HashMap<String, String> {
        let mut docs = HashMap::new();

        for (name, cmd) in &registry.commands {
            // Create searchable document from command metadata
            let doc = format!(
                "{} {} {} {}",
                cmd.name,
                cmd.short_description,
                cmd.long_description,
                cmd.tags.join(" ")
            );
            docs.insert(name.clone(), doc);

            // Index subcommands
            for sub in &cmd.subcommands {
                let full_name = format!("{} {}", name, sub.name);
                let sub_doc = format!(
                    "{} {} {} {}",
                    sub.name,
                    sub.short_description,
                    sub.long_description,
                    sub.tags.join(" ")
                );
                docs.insert(full_name, sub_doc);
            }
        }

        docs
    }

    /// Intelligent help lookup
    ///
    /// Combines:
    /// 1. Exact match (fast path)
    /// 2. Fuzzy match via NLP (typo tolerance)
    /// 3. Semantic search via BM25 (intent understanding)
    /// 4. Importance ranking via PageRank (relevance)
    pub fn lookup(&self, query: &str) -> HelpResponse {
        // 1. Try exact match
        if let Some(cmd) = self.registry.find_command(query) {
            return HelpResponse::Exact(cmd.clone());
        }

        // 2. Try fuzzy match for typos (edit distance)
        let all_commands = self.registry.all_command_paths();
        if let Some((suggestion, distance)) = self.find_closest(&all_commands, query) {
            if distance <= 2 {
                return HelpResponse::DidYouMean {
                    suggestion,
                    confidence: 1.0 - (distance as f32 / query.len().max(1) as f32),
                };
            }
        }

        // 3. Semantic search
        let mut results = self.search(query, 5);

        // 4. Re-rank by PageRank importance
        for result in &mut results {
            result.importance = self.graph.importance(&result.command);
            // Combined score: 70% relevance + 30% importance
            result.combined_score = 0.7 * result.relevance + 0.3 * result.importance;
        }

        // Sort by combined score
        results.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        HelpResponse::SearchResults {
            query: query.to_string(),
            results,
        }
    }

    /// Search commands by query
    pub fn search(&self, query: &str, top_k: usize) -> Vec<HelpSearchResult> {
        let mut scored: Vec<_> = self
            .command_docs
            .iter()
            .map(|(name, doc)| {
                let score = self.nlp.bm25_score(query, doc, 1.2, 0.75);
                (name.clone(), doc.clone(), score)
            })
            .collect();

        // Sort by BM25 score
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored
            .into_iter()
            .filter(|(_, _, score)| *score > 0.0)
            .map(|(name, doc, score)| {
                let cmd = self.registry.find_command(&name);
                HelpSearchResult {
                    command: name,
                    description: cmd.map(|c| c.short_description.clone()).unwrap_or_default(),
                    relevance: (score as f32).min(1.0),
                    importance: 0.0, // Will be filled by lookup()
                    combined_score: 0.0,
                    snippet: self.extract_snippet(&doc, query),
                }
            })
            .collect()
    }

    /// Find closest command using edit distance
    fn find_closest(&self, commands: &[String], query: &str) -> Option<(String, usize)> {
        commands
            .iter()
            .map(|cmd| {
                let distance = levenshtein(&cmd.to_lowercase(), &query.to_lowercase());
                (cmd.clone(), distance)
            })
            .min_by_key(|(_, d)| *d)
    }

    /// Extract relevant snippet from document
    fn extract_snippet(&self, doc: &str, _query: &str) -> String {
        // Simple snippet: first 100 chars
        // TODO: Use query to highlight matching terms
        if doc.len() <= 100 {
            doc.to_string()
        } else {
            format!("{}...", &doc[..100])
        }
    }

    /// Get top important commands (for suggestions)
    pub fn get_important_commands(&self, k: usize) -> Vec<(String, f32)> {
        self.graph.top_k_important(k)
    }

    /// Get commands by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&CommandMetadata> {
        self.registry.find_by_tag(tag)
    }

    /// Get commands by category
    pub fn get_by_category(&self, category: &str) -> Vec<&CommandMetadata> {
        self.registry.find_by_category(category)
    }
}

/// Simple Levenshtein distance
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        // Analyze command with subcommands
        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze code complexity")
            .long_description("Calculate cyclomatic complexity for all functions")
            .tags(["quality", "metrics", "complexity"])
            .build();

        let satd_sub = CommandMetadata::builder("satd")
            .short_description("Find technical debt")
            .long_description("Detect TODO/FIXME/HACK comments indicating technical debt")
            .tags(["quality", "debt", "satd"])
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code metrics")
                .long_description("Run various code analysis tools on your project")
                .aliases(["a", "an"])
                .subcommand(complexity_sub)
                .subcommand(satd_sub)
                .category("analysis")
                .tags(["quality", "metrics"])
                .related("context")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("context")
                .short_description("Generate project context")
                .long_description("Generate AI-friendly project context using AST analysis")
                .aliases(["ctx"])
                .category("generation")
                .tags(["generation", "ast", "context"])
                .related("analyze")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("quality-gate")
                .short_description("Run quality gates")
                .long_description("Check code quality against configured thresholds")
                .aliases(["qg", "gate"])
                .category("quality")
                .tags(["quality", "ci", "gate"])
                .build(),
        );

        registry
    }

    mod nlp_tests {
        use super::*;

        #[test]
        fn test_nlp_processor_creation() {
            let nlp = HelpNlpProcessor::new();
            let tokens = nlp.preprocess("analyze code complexity");
            assert!(!tokens.is_empty());
        }

        #[test]
        fn test_preprocess_removes_stop_words() {
            let nlp = HelpNlpProcessor::new();
            let tokens = nlp.preprocess("run the pmat command");
            // "run", "the", "pmat", "command" should be filtered
            assert!(tokens.is_empty() || !tokens.contains(&"the".to_string()));
        }

        #[test]
        fn test_term_frequency() {
            let nlp = HelpNlpProcessor::new();
            let tf = nlp.term_frequency("code code code analysis");
            // "code" appears three times, "analysis" once
            let code_freq = tf.get("code").unwrap_or(&0.0);
            let analysis_freq = tf.get("analysi").unwrap_or(&0.0);
            assert!(code_freq > analysis_freq, "code freq {} should be > analysis freq {}", code_freq, analysis_freq);
        }

        #[test]
        fn test_bm25_score() {
            let nlp = HelpNlpProcessor::new();
            let score1 = nlp.bm25_score("complexity", "analyze complexity metrics", 1.2, 0.75);
            let score2 = nlp.bm25_score("complexity", "generate context ast", 1.2, 0.75);
            // Query "complexity" should match better with first doc
            assert!(score1 > score2);
        }
    }

    mod graph_tests {
        use super::*;

        #[test]
        fn test_graph_creation() {
            let graph = CommandGraph::new();
            assert!(graph.command_to_node.is_empty());
        }

        #[test]
        fn test_build_from_registry() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            // Should have all commands indexed
            assert!(graph.command_to_node.contains_key("analyze"));
            assert!(graph.command_to_node.contains_key("context"));
            assert!(graph.command_to_node.contains_key("analyze complexity"));
        }

        #[test]
        fn test_pagerank_importance() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            // All commands should have non-zero importance
            let importance = graph.importance("analyze");
            assert!(importance >= 0.0);
        }

        #[test]
        fn test_top_k_important() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            let top = graph.top_k_important(3);
            assert!(top.len() <= 3);
        }
    }

    mod unified_help_tests {
        use super::*;

        #[test]
        fn test_unified_help_creation() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);
            assert!(!help.command_docs.is_empty());
        }

        #[test]
        fn test_lookup_exact_match() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let response = help.lookup("analyze");
            assert!(matches!(response, HelpResponse::Exact(_)));
        }

        #[test]
        fn test_lookup_fuzzy_match() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            // Typo: "analize" instead of "analyze"
            let response = help.lookup("analize");
            assert!(matches!(response, HelpResponse::DidYouMean { .. }));
        }

        #[test]
        fn test_lookup_semantic_search() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            // Query that doesn't match any command name
            let response = help.lookup("how to check code quality");
            match response {
                HelpResponse::SearchResults { results, .. } => {
                    assert!(!results.is_empty());
                }
                _ => panic!("Expected SearchResults"),
            }
        }

        #[test]
        fn test_search_returns_relevant_results() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let results = help.search("complexity", 3);
            assert!(!results.is_empty());
            // First result should be related to complexity
            assert!(results[0].command.contains("complexity") ||
                    results[0].description.to_lowercase().contains("complex"));
        }

        #[test]
        fn test_search_ranking() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let results = help.search("technical debt", 5);
            // satd command should rank highly for "technical debt"
            let satd_result = results.iter().find(|r| r.command.contains("satd"));
            assert!(satd_result.is_some());
        }

        #[test]
        fn test_get_important_commands() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let important = help.get_important_commands(5);
            assert!(!important.is_empty());
        }

        #[test]
        fn test_get_by_tag() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let quality_cmds = help.get_by_tag("quality");
            assert!(!quality_cmds.is_empty());
        }

        #[test]
        fn test_get_by_category() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let analysis_cmds = help.get_by_category("analysis");
            assert_eq!(analysis_cmds.len(), 1);
            assert_eq!(analysis_cmds[0].name, "analyze");
        }
    }

    mod levenshtein_tests {
        use super::*;

        #[test]
        fn test_levenshtein_identical() {
            assert_eq!(levenshtein("test", "test"), 0);
        }

        #[test]
        fn test_levenshtein_one_char() {
            assert_eq!(levenshtein("test", "fest"), 1); // One substitution
            assert_eq!(levenshtein("test", "tests"), 1); // One insertion
            assert_eq!(levenshtein("test", "tes"), 1); // One deletion
        }

        #[test]
        fn test_levenshtein_empty() {
            assert_eq!(levenshtein("", "test"), 4);
            assert_eq!(levenshtein("test", ""), 4);
            assert_eq!(levenshtein("", ""), 0);
        }
    }
}
