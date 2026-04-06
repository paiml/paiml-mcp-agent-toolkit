// SimilarityDetector implementation: public API for clone detection,
// entropy analysis, refactoring opportunities, and comprehensive reports.
// Also contains private helper methods for block extraction, normalization,
// hashing, and similarity computation.

impl SimilarityDetector {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: SimilarityConfig) -> Self {
        Self {
            winnower: Winnowing::new(config.window_size, config.k_gram_size),
            token_analyzer: TokenAnalyzer::new(),
            entropy_calculator: EntropyCalculator::new(),
            config,
        }
    }

    /// Detect exact duplicates
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_exact_duplicates(&self, files: &[(PathBuf, String)]) -> Vec<SimilarBlock> {
        let mut hash_map: HashMap<u64, Vec<(PathBuf, usize, usize, String)>> = HashMap::new();

        for (path, content) in files {
            let blocks = self.extract_code_blocks(content, self.config.min_lines);
            for block in blocks {
                let normalized = self.normalize_whitespace(&block.content);
                let hash = self.hash_content(&normalized);
                hash_map.entry(hash).or_default().push((
                    path.clone(),
                    block.start_line,
                    block.end_line,
                    block.content,
                ));
            }
        }

        self.build_duplicate_blocks(hash_map, CloneType::Type1)
    }

    /// Detect structural similarity using AST normalization
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_structural_similarity(
        &self,
        files: &[(PathBuf, String)],
        threshold: f64,
    ) -> Vec<SimilarBlock> {
        let mut normalized_blocks = Vec::new();

        for (path, content) in files {
            let blocks = self.extract_code_blocks(content, self.config.min_lines);
            for block in blocks {
                let normalized = self.normalize_identifiers(&block.content);
                normalized_blocks.push((path.clone(), block, normalized));
            }
        }

        self.find_similar_blocks(normalized_blocks, threshold, CloneType::Type2)
    }

    /// Detect semantic similarity using token analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_semantic_similarity(
        &self,
        files: &[(PathBuf, String)],
        threshold: f64,
    ) -> Vec<SimilarBlock> {
        let mut token_vectors = Vec::new();

        for (path, content) in files {
            let blocks = self.extract_code_blocks(content, self.config.min_lines);
            for block in blocks {
                let tokens = self.token_analyzer.tokenize(&block.content);
                let vector = self.token_analyzer.to_vector(&tokens);
                token_vectors.push((path.clone(), block, vector));
            }
        }

        self.find_semantic_matches(token_vectors, threshold, CloneType::Type4)
    }

    /// Analyze entropy of code blocks
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn analyze_entropy(&self, files: &[(PathBuf, String)]) -> EntropyReport {
        let mut all_entropies = Vec::new();
        let mut high_entropy = Vec::new();
        let mut low_entropy = Vec::new();

        for (path, content) in files {
            let blocks = self.extract_code_blocks(content, self.config.min_lines);
            for block in blocks {
                let entropy = self.calculate_entropy(&block.content);
                all_entropies.push(entropy);

                let location = Location {
                    file: path.clone(),
                    start_line: block.start_line,
                    end_line: block.end_line,
                    start_column: None,
                    end_column: None,
                };

                if entropy > 4.0 {
                    high_entropy.push(EntropyBlock {
                        location,
                        entropy,
                        category: "Complex".to_string(),
                        suggestion: "Consider breaking down this complex code".to_string(),
                    });
                } else if entropy < 2.0 {
                    low_entropy.push(EntropyBlock {
                        location,
                        entropy,
                        category: "Repetitive".to_string(),
                        suggestion: "Extract repeated pattern into reusable function".to_string(),
                    });
                }
            }
        }

        let avg_entropy = if all_entropies.is_empty() {
            0.0
        } else {
            all_entropies.iter().sum::<f64>() / all_entropies.len() as f64
        };

        let recommendations = self.generate_recommendations(&high_entropy, &low_entropy);

        EntropyReport {
            average_entropy: avg_entropy,
            high_entropy_blocks: high_entropy,
            low_entropy_patterns: low_entropy,
            recommendations,
        }
    }

    /// Find refactoring opportunities
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn find_refactoring_opportunities(
        &self,
        files: &[(PathBuf, String)],
    ) -> Vec<RefactoringHint> {
        let mut hints = Vec::new();

        // Find similar patterns
        let structural = self.detect_structural_similarity(files, 0.8);
        for similar in structural {
            if similar.locations.len() > 2 {
                hints.push(RefactoringHint {
                    locations: similar.locations,
                    pattern: "Repeated code structure".to_string(),
                    suggestion: "Extract common pattern into shared function".to_string(),
                    priority: Priority::High,
                });
            }
        }

        // Find semantic duplicates
        let semantic = self.detect_semantic_similarity(files, 0.7);
        for similar in semantic {
            hints.push(RefactoringHint {
                locations: similar.locations,
                pattern: "Semantically equivalent code".to_string(),
                suggestion: "Consolidate implementations".to_string(),
                priority: Priority::Medium,
            });
        }

        hints
    }

    /// Perform comprehensive analysis
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn comprehensive_analysis(&self, files: &[(PathBuf, String)]) -> ComprehensiveReport {
        let exact = self.detect_exact_duplicates(files);
        let structural = self.detect_structural_similarity(files, self.config.similarity_threshold);
        let semantic = self.detect_semantic_similarity(files, self.config.similarity_threshold);
        let entropy = if self.config.enable_entropy {
            Some(self.analyze_entropy(files))
        } else {
            None
        };
        let refactoring = self.find_refactoring_opportunities(files);

        let total_clones = exact.len() + structural.len() + semantic.len();
        let duplication_percentage = self.calculate_duplication_percentage(files, &exact);
        let average_entropy = entropy.as_ref().map_or(0.0, |e| e.average_entropy);

        ComprehensiveReport {
            exact_duplicates: exact,
            structural_similarities: structural,
            semantic_similarities: semantic,
            entropy_analysis: entropy,
            refactoring_opportunities: refactoring,
            metrics: Metrics {
                duplication_percentage,
                average_entropy,
                total_clones,
            },
        }
    }

    /// Calculate Shannon entropy
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn calculate_entropy(&self, text: &str) -> f64 {
        let result = self.entropy_calculator.calculate(text);
        result
    }

    // --- Private helper methods ---

    fn extract_code_blocks(&self, content: &str, min_lines: usize) -> Vec<CodeBlock> {
        let lines: Vec<&str> = content.lines().collect();
        let mut blocks = Vec::new();

        for i in 0..lines.len().saturating_sub(min_lines - 1) {
            let block_lines = &lines[i..i + min_lines];
            let block_content = block_lines.join("\n");

            if self.count_tokens(&block_content) >= self.config.min_tokens {
                blocks.push(CodeBlock {
                    start_line: i + 1,
                    end_line: i + min_lines,
                    content: block_content,
                });
            }
        }

        blocks
    }

    fn normalize_whitespace(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn normalize_identifiers(&self, text: &str) -> String {
        // Simple identifier normalization - replace with placeholders
        let mut result = text.to_string();
        let ident_pattern =
            regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").expect("internal error");
        let mut counter = 0;

        for mat in ident_pattern.find_iter(text) {
            if !self.is_keyword(mat.as_str()) {
                counter += 1;
                result = result.replace(mat.as_str(), &format!("VAR{counter}"));
            }
        }

        result
    }

    fn hash_content(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn is_keyword(&self, word: &str) -> bool {
        matches!(
            word,
            "fn" | "let"
                | "mut"
                | "if"
                | "else"
                | "match"
                | "for"
                | "while"
                | "loop"
                | "return"
                | "use"
                | "pub"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "mod"
        )
    }

    fn build_duplicate_blocks(
        &self,
        hash_map: HashMap<u64, Vec<(PathBuf, usize, usize, String)>>,
        clone_type: CloneType,
    ) -> Vec<SimilarBlock> {
        let mut blocks = Vec::new();

        for (hash, locations) in hash_map {
            if locations.len() > 1 {
                let content = &locations[0].3;
                let lines = content.lines().count();
                let tokens = self.count_tokens(content);

                blocks.push(SimilarBlock {
                    id: format!("{hash:x}"),
                    locations: locations
                        .iter()
                        .map(|(path, start, end, _)| Location {
                            file: path.clone(),
                            start_line: *start,
                            end_line: *end,
                            start_column: None,
                            end_column: None,
                        })
                        .collect(),
                    similarity: 1.0,
                    clone_type,
                    lines,
                    tokens,
                    content_preview: content.lines().take(3).collect::<Vec<_>>().join("\n"),
                });
            }
        }

        blocks
    }

    fn find_similar_blocks(
        &self,
        normalized: Vec<(PathBuf, CodeBlock, String)>,
        threshold: f64,
        clone_type: CloneType,
    ) -> Vec<SimilarBlock> {
        let mut similar = Vec::new();

        for i in 0..normalized.len() {
            for j in i + 1..normalized.len() {
                let sim = self.calculate_similarity(&normalized[i].2, &normalized[j].2);
                if sim >= threshold {
                    similar.push(SimilarBlock {
                        id: format!("sim_{}", similar.len()),
                        locations: vec![
                            Location {
                                file: normalized[i].0.clone(),
                                start_line: normalized[i].1.start_line,
                                end_line: normalized[i].1.end_line,
                                start_column: None,
                                end_column: None,
                            },
                            Location {
                                file: normalized[j].0.clone(),
                                start_line: normalized[j].1.start_line,
                                end_line: normalized[j].1.end_line,
                                start_column: None,
                                end_column: None,
                            },
                        ],
                        similarity: sim,
                        clone_type,
                        lines: normalized[i].1.content.lines().count(),
                        tokens: self.count_tokens(&normalized[i].1.content),
                        content_preview: normalized[i]
                            .1
                            .content
                            .lines()
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    });
                }
            }
        }

        similar
    }

    fn find_semantic_matches(
        &self,
        vectors: Vec<(PathBuf, CodeBlock, TokenVector)>,
        threshold: f64,
        clone_type: CloneType,
    ) -> Vec<SimilarBlock> {
        let mut matches = Vec::new();

        for i in 0..vectors.len() {
            for j in i + 1..vectors.len() {
                let sim = self
                    .token_analyzer
                    .cosine_similarity(&vectors[i].2, &vectors[j].2);
                if sim >= threshold {
                    matches.push(SimilarBlock {
                        id: format!("sem_{}", matches.len()),
                        locations: vec![
                            Location {
                                file: vectors[i].0.clone(),
                                start_line: vectors[i].1.start_line,
                                end_line: vectors[i].1.end_line,
                                start_column: None,
                                end_column: None,
                            },
                            Location {
                                file: vectors[j].0.clone(),
                                start_line: vectors[j].1.start_line,
                                end_line: vectors[j].1.end_line,
                                start_column: None,
                                end_column: None,
                            },
                        ],
                        similarity: sim,
                        clone_type,
                        lines: vectors[i].1.content.lines().count(),
                        tokens: self.count_tokens(&vectors[i].1.content),
                        content_preview: vectors[i]
                            .1
                            .content
                            .lines()
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    });
                }
            }
        }

        matches
    }

    fn calculate_similarity(&self, text1: &str, text2: &str) -> f64 {
        // Use aprender's edit_distance_similarity (replaces levenshtein crate)
        // Returns normalized similarity: 1.0 = identical, 0.0 = completely different
        aprender::text::similarity::edit_distance_similarity(text1, text2).unwrap_or(0.0)
    }

    fn calculate_duplication_percentage(
        &self,
        files: &[(PathBuf, String)],
        duplicates: &[SimilarBlock],
    ) -> f64 {
        let total_lines: usize = files
            .iter()
            .map(|(_, content)| content.lines().count())
            .sum();

        let duplicate_lines: usize = duplicates.iter().map(|d| d.lines * d.locations.len()).sum();

        if total_lines > 0 {
            (duplicate_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        }
    }

    fn generate_recommendations(
        &self,
        high_entropy: &[EntropyBlock],
        low_entropy: &[EntropyBlock],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !high_entropy.is_empty() {
            recommendations.push(format!(
                "Found {} complex code blocks that should be simplified",
                high_entropy.len()
            ));
        }

        if !low_entropy.is_empty() {
            recommendations.push(format!(
                "Found {} repetitive patterns that could be extracted",
                low_entropy.len()
            ));
        }

        if low_entropy.len() > 5 {
            recommendations
                .push("Consider creating utility functions for common patterns".to_string());
        }

        recommendations
    }
}
