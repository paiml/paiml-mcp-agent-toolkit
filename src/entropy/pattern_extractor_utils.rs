impl PatternExtractor {
    /// Calculate variation score for pattern matches
    fn calculate_variation_score(&self, matches: &[regex::Match], content: &str) -> f64 {
        if matches.len() <= 1 {
            return 0.0;
        }

        // Simple variation calculation based on context differences
        let contexts: Vec<String> = matches
            .iter()
            .take(5)
            .map(|m| {
                let start = m.start().saturating_sub(20);
                let end = (m.end() + 20).min(content.len());

                // Ensure we're on char boundaries for UTF-8 safety
                let start_char = content
                    .char_indices()
                    .find(|(i, _)| *i >= start)
                    .map_or(start, |(i, _)| i);
                let end_char = content
                    .char_indices()
                    .rev()
                    .find(|(i, _)| *i <= end)
                    .map_or(end, |(i, c)| i + c.len_utf8());

                content
                    .get(start_char..end_char)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();

        // Calculate similarity between contexts
        let mut total_similarity = 0.0;
        let mut comparisons = 0;

        for i in 0..contexts.len() {
            for j in (i + 1)..contexts.len() {
                let similarity = self.calculate_string_similarity(&contexts[i], &contexts[j]);
                total_similarity += similarity;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            1.0 - (total_similarity / f64::from(comparisons)) // Higher variation = less similarity
        } else {
            0.0
        }
    }

    /// Calculate string similarity (simplified Jaccard similarity)
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = s1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = s2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate how much patterns vary from each other.
    /// For Rust patterns (created via group_by_structural_hash), variation_score is already
    /// set correctly at creation time (0.0 = structurally identical). For Ruchy patterns
    /// (created with per-method variation calculators), variation_score is also already set.
    /// This method is now a no-op; the old heuristic (locations.len / 10) was overriding
    /// structural hash results with meaningless values.
    fn calculate_pattern_variations(&self, _collection: &mut PatternCollection) {}

    /// Create a hash for a pattern to identify similar ones
    fn hash_pattern(&self, ast_data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        ast_data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Normalize a code snippet for structural comparison.
    /// Strips variable names, normalizes whitespace, replaces identifiers with
    /// placeholders so that structurally identical code produces the same hash
    /// regardless of variable naming.
    fn normalize_code_snippet(snippet: &str) -> String {
        use regex::Regex;

        let trimmed = snippet.trim();
        // Replace string literals with placeholder
        let re_string = Regex::new(r#""[^"]*""#).expect("valid regex");
        let normalized = re_string.replace_all(trimmed, "\"STR\"");
        // Replace numeric literals with placeholder
        let re_num = Regex::new(r"\b\d+\b").expect("valid regex");
        let normalized = re_num.replace_all(&normalized, "N");
        // Replace identifiers with placeholder, preserving keywords.
        // Split on word boundaries, check each token against keyword set.
        let re_ident = Regex::new(r"\b[a-zA-Z_]\w*\b").expect("valid regex");
        let keywords: std::collections::HashSet<&str> = [
            "if", "else", "match", "for", "while", "let", "mut", "fn", "return", "true", "false",
            "self", "Ok", "Err", "Some", "None", "Result", "Option", "Vec", "String", "impl",
            "pub", "struct", "enum", "async", "await", "unsafe", "use", "mod", "const", "static",
            "type", "where", "trait", "loop", "break", "continue", "ref", "in", "as", "crate",
            "super", "dyn", "move", "extern", "STR", "N",
        ]
        .into_iter()
        .collect();
        let normalized = re_ident.replace_all(&normalized, |caps: &regex::Captures| {
            let word = caps.get(0).expect("group 0").as_str();
            if keywords.contains(word) {
                word.to_string()
            } else {
                "IDENT".to_string()
            }
        });
        // Collapse whitespace
        let re_ws = Regex::new(r"\s+").expect("valid regex");
        re_ws.replace_all(&normalized, " ").to_string()
    }

    /// Extract the current line containing a regex match for structural comparison.
    fn extract_match_context(content: &str, m: &regex::Match) -> String {
        let line_start = content[..m.start()].rfind('\n').map_or(0, |p| p + 1);
        let line_end = content[m.end()..]
            .find('\n')
            .map_or(content.len(), |p| m.end() + p);
        content
            .get(line_start..line_end)
            .unwrap_or_default()
            .to_string()
    }

    /// Group matches by structural hash and produce AstPatterns for groups with >= min_group_size
    /// structurally identical matches.
    #[allow(clippy::too_many_arguments)]
    fn group_by_structural_hash(
        &self,
        matches: &[regex::Match],
        content: &str,
        file_path: &Path,
        pattern_type: PatternType,
        min_group_size: usize,
        loc_per_match: usize,
        collection: &mut PatternCollection,
    ) {
        // BTreeMap so groups are emitted in hash order; with a HashMap the order
        // in which patterns were handed to `add_pattern` varied per process.
        let mut groups: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();

        for m in matches.iter().take(20) {
            let context = Self::extract_match_context(content, m);
            let normalized = Self::normalize_code_snippet(&context);
            let structural_hash = self.hash_pattern(&normalized);
            let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
            groups
                .entry(structural_hash)
                .or_default()
                .push((line_num, context));
        }

        for (hash, group) in &groups {
            if group.len() >= min_group_size {
                let locations: Vec<Location> = group
                    .iter()
                    .take(10)
                    .map(|(line, _)| Location {
                        file: file_path.to_owned(),
                        line: *line,
                        column: 1,
                    })
                    .collect();

                let example_code = group
                    .first()
                    .map(|(_, ctx)| ctx.chars().take(100).collect::<String>())
                    .unwrap_or_default();

                let pattern = AstPattern {
                    pattern_type,
                    pattern_hash: hash.clone(),
                    frequency: group.len().min(10),
                    locations,
                    variation_score: 0.0, // Structurally identical = no variation
                    example_code,
                    estimated_loc: group.len() * loc_per_match,
                };

                collection.add_pattern(pattern);
            }
        }
    }
}
