// UnifiedHelpService implementation and levenshtein distance utility

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
        if doc.len() <= 100 {
            doc.to_string()
        } else {
            format!("{}...", doc.get(..100).unwrap_or(doc))
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
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}
