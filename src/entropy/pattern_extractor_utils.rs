static NORMALIZE_STRING: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r#""[^"]*""#).expect("valid regex"));
static NORMALIZE_NUM: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\b\d+\b").expect("valid regex"));
static NORMALIZE_IDENT: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\b[a-zA-Z_]\w*\b").expect("valid regex"));
static NORMALIZE_WS: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\s+").expect("valid regex"));
static NORMALIZE_KEYWORDS: std::sync::LazyLock<std::collections::HashSet<&'static str>> =
    std::sync::LazyLock::new(|| {
        [
            "if", "else", "match", "for", "while", "let", "mut", "fn", "return", "true", "false",
            "self", "Ok", "Err", "Some", "None", "Result", "Option", "Vec", "String", "impl",
            "pub", "struct", "enum", "async", "await", "unsafe", "use", "mod", "const", "static",
            "type", "where", "trait", "loop", "break", "continue", "ref", "in", "as", "crate",
            "super", "dyn", "move", "extern", "STR", "N",
        ]
        .into_iter()
        .collect()
    });

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
    ///
    /// The four regexes and the keyword set are process statics
    /// (`NORMALIZE_*` / `NORMALIZE_KEYWORDS`). They used to be rebuilt on every
    /// call, which was affordable only because the caller looked at 20 matches
    /// per file at most; that cap is gone, so this now runs tens of thousands of
    /// times per project.
    fn normalize_code_snippet(snippet: &str) -> String {
        let trimmed = snippet.trim();
        // Replace string literals with placeholder
        let normalized = NORMALIZE_STRING.replace_all(trimmed, "\"STR\"");
        // Replace numeric literals with placeholder
        let normalized = NORMALIZE_NUM.replace_all(&normalized, "N");
        // Replace identifiers with placeholder, preserving keywords.
        // Split on word boundaries, check each token against keyword set.
        let normalized = NORMALIZE_IDENT.replace_all(&normalized, |caps: &regex::Captures| {
            let word = caps.get(0).expect("group 0").as_str();
            if NORMALIZE_KEYWORDS.contains(word) {
                word.to_string()
            } else {
                "IDENT".to_string()
            }
        });
        // Collapse whitespace
        NORMALIZE_WS.replace_all(&normalized, " ").to_string()
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

    /// Distinct source lines a set of matches occupies, as `Location`s.
    ///
    /// Uncapped and deduped. The Ruchy extractors each stopped recording
    /// locations after the 8th, 10th or 15th match, so their location lists were
    /// silently truncated totals; `estimated_loc` was separately a per-construct
    /// constant ("Each pipeline operation is ~2 lines"). Both are replaced by
    /// this one measurement.
    fn distinct_line_locations<'m>(
        file_path: &Path,
        content: &str,
        matches: impl IntoIterator<Item = &'m regex::Match<'m>>,
    ) -> Vec<Location> {
        let mut lines: Vec<usize> = matches
            .into_iter()
            .map(|m| content.get(..m.start()).unwrap_or_default().lines().count() + 1)
            .collect();
        lines.sort_unstable();
        lines.dedup();
        lines
            .into_iter()
            .map(|line| Location {
                file: file_path.to_owned(),
                line,
                column: 1,
            })
            .collect()
    }

    /// Group matches by structural hash and produce AstPatterns for groups with
    /// >= `min_group_size` structurally identical occurrences.
    fn group_by_structural_hash(
        &self,
        matches: &[regex::Match],
        content: &str,
        file_path: &Path,
        pattern_type: PatternType,
        min_group_size: usize,
        collection: &mut PatternCollection,
    ) {
        // BTreeMap so groups are emitted in hash order; with a HashMap the order
        // in which patterns were handed to `add_pattern` varied per process.
        let mut groups: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();

        // A TOTAL THAT IS SECRETLY A CAP: this loop was `matches.iter().take(20)`.
        // Every match past the twentieth in a file was simply not counted, so the
        // reported number stopped measuring the input.
        for m in matches {
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
            // One occurrence per distinct source line. Two matches on one line
            // (`.map(..).filter(..)`) are one line of code, and `merge_pattern`
            // dedups locations by (file, line, column) anyway — counting raw
            // matches made `frequency` exceed the location list after a merge.
            let mut lines: Vec<usize> = group.iter().map(|(line, _)| *line).collect();
            lines.sort_unstable();
            lines.dedup();

            if lines.len() < min_group_size {
                continue;
            }

            // `locations` was `.take(10)`, so the list disagreed with the count
            // that heads it as soon as a file had 11 copies.
            let locations: Vec<Location> = lines
                .iter()
                .map(|line| Location {
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
                // Was `group.len().min(10)`. One file with N identical copies
                // reported N for N <= 10 and then 10 forever: 11, 12, 20, 40 and
                // 100 copies all came out as "repeated 10 times", while
                // estimated_loc kept tracking the real N — so at N=100 the
                // message read "DataValidation pattern repeated 10 times (saves
                // 43 lines)", two halves of one sentence disagreeing about how
                // much code there is.
                frequency: lines.len(),
                locations,
                variation_score: 0.0, // Structurally identical = no variation
                example_code,
                // MEASURED: the source lines these occurrences actually sit on.
                // This was `group.len() * loc_per_match`, where loc_per_match was
                // a per-construct guess (5, 3, 4, 6, 2, 3 lines) that nothing
                // measured. With the per-file cap removed the guess showed: 100
                // one-line validation checks in a 102-line file were reported as
                // "saves 237 lines (232.4% of analyzed code)" — a part 2.3x its
                // own whole.
                estimated_loc: lines.len(),
            };

            collection.add_pattern(pattern);
        }
    }
}
