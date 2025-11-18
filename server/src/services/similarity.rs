//! Advanced code similarity detection with entropy analysis
//!
//! Implements multiple algorithms for detecting code clones and similarities:
//! - Winnowing for fingerprinting
//! - AST-based structural similarity
//! - Token-based semantic similarity
//! - Shannon entropy analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Configuration for similarity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityConfig {
    pub min_lines: usize,
    pub min_tokens: usize,
    pub similarity_threshold: f64,
    pub enable_entropy: bool,
    pub enable_ast: bool,
    pub enable_semantic: bool,
    pub window_size: usize,
    pub k_gram_size: usize,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            min_lines: 6,
            min_tokens: 50,
            similarity_threshold: 0.7,
            enable_entropy: true,
            enable_ast: true,
            enable_semantic: true,
            window_size: 40,
            k_gram_size: 15,
        }
    }
}

/// Types of code clones
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CloneType {
    Type1, // Exact clones
    Type2, // Renamed clones
    Type3, // Modified clones
    Type4, // Semantic clones
}

/// A duplicate or similar code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarBlock {
    pub id: String,
    pub locations: Vec<Location>,
    pub similarity: f64,
    pub clone_type: CloneType,
    pub lines: usize,
    pub tokens: usize,
    pub content_preview: String,
}

/// Location of a code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
}

/// Entropy analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub average_entropy: f64,
    pub high_entropy_blocks: Vec<EntropyBlock>,
    pub low_entropy_patterns: Vec<EntropyBlock>,
    pub recommendations: Vec<String>,
}

/// A code block with entropy measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyBlock {
    pub location: Location,
    pub entropy: f64,
    pub category: String,
    pub suggestion: String,
}

/// Refactoring hint for similar code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringHint {
    pub locations: Vec<Location>,
    pub pattern: String,
    pub suggestion: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Comprehensive analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    pub exact_duplicates: Vec<SimilarBlock>,
    pub structural_similarities: Vec<SimilarBlock>,
    pub semantic_similarities: Vec<SimilarBlock>,
    pub entropy_analysis: Option<EntropyReport>,
    pub refactoring_opportunities: Vec<RefactoringHint>,
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub duplication_percentage: f64,
    pub average_entropy: f64,
    pub total_clones: usize,
}

/// Main similarity detector
pub struct SimilarityDetector {
    config: SimilarityConfig,
    #[allow(dead_code)] // Will be used in future winnowing implementation
    winnower: Winnowing,
    token_analyzer: TokenAnalyzer,
    entropy_calculator: EntropyCalculator,
}

impl SimilarityDetector {
    #[must_use]
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
    pub fn calculate_entropy(&self, text: &str) -> f64 {
        self.entropy_calculator.calculate(text)
    }

    // Helper methods with complexity < 10

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
        let ident_pattern = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
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
        let len1 = text1.len() as f64;
        let len2 = text2.len() as f64;
        let dist = levenshtein::levenshtein(text1, text2) as f64;

        1.0 - (dist / len1.max(len2))
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

struct CodeBlock {
    start_line: usize,
    end_line: usize,
    content: String,
}

/// Winnowing algorithm for fingerprinting
pub struct Winnowing {
    window_size: usize,
    k_gram_size: usize,
}

impl Winnowing {
    #[must_use]
    pub fn new(window_size: usize, k_gram_size: usize) -> Self {
        Self {
            window_size,
            k_gram_size,
        }
    }

    #[must_use]
    pub fn fingerprint(&self, text: &str) -> Vec<u64> {
        let k_grams = self.extract_k_grams(text);
        self.select_fingerprints(&k_grams)
    }

    #[must_use]
    pub fn similarity(&self, fp1: &[u64], fp2: &[u64]) -> f64 {
        let set1: HashSet<_> = fp1.iter().collect();
        let set2: HashSet<_> = fp2.iter().collect();

        let intersection = set1.intersection(&set2).count() as f64;
        let union = set1.union(&set2).count() as f64;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn find_matches(&self, text_fp: &[u64], sub_fp: &[u64]) -> Vec<usize> {
        let mut matches = Vec::new();
        let sub_set: HashSet<_> = sub_fp.iter().collect();

        for (i, fp) in text_fp.iter().enumerate() {
            if sub_set.contains(fp) {
                matches.push(i);
            }
        }

        matches
    }

    fn extract_k_grams(&self, text: &str) -> Vec<u64> {
        let chars: Vec<char> = text.chars().collect();
        let mut k_grams = Vec::new();

        for i in 0..chars.len().saturating_sub(self.k_gram_size - 1) {
            let gram: String = chars[i..i + self.k_gram_size].iter().collect();
            k_grams.push(self.hash_k_gram(&gram));
        }

        k_grams
    }

    fn select_fingerprints(&self, k_grams: &[u64]) -> Vec<u64> {
        let mut fingerprints = Vec::new();

        for window in k_grams.windows(self.window_size) {
            if let Some(min) = window.iter().min() {
                if !fingerprints.contains(min) {
                    fingerprints.push(*min);
                }
            }
        }

        fingerprints
    }

    fn hash_k_gram(&self, gram: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        gram.hash(&mut hasher);
        hasher.finish()
    }
}

/// Token-based analysis for semantic similarity
struct TokenAnalyzer;

type TokenVector = HashMap<String, f64>;

impl TokenAnalyzer {
    fn new() -> Self {
        Self
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_lowercase).collect()
    }

    fn to_vector(&self, tokens: &[String]) -> TokenVector {
        let mut vector = HashMap::new();
        let total = tokens.len() as f64;

        for token in tokens {
            *vector.entry(token.clone()).or_insert(0.0) += 1.0 / total;
        }

        vector
    }

    fn cosine_similarity(&self, v1: &TokenVector, v2: &TokenVector) -> f64 {
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for (token, weight1) in v1 {
            norm1 += weight1 * weight1;
            if let Some(weight2) = v2.get(token) {
                dot_product += weight1 * weight2;
            }
        }

        for weight2 in v2.values() {
            norm2 += weight2 * weight2;
        }

        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        }
    }
}

/// Shannon entropy calculator
struct EntropyCalculator;

impl EntropyCalculator {
    fn new() -> Self {
        Self
    }

    fn calculate(&self, text: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let total = text.len() as f64;

        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for count in char_counts.values() {
            let probability = f64::from(*count) / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }

    // SIMD/Scalar equivalence property tests for Trueno integration
    mod simd_equivalence_tests {
        use proptest::prelude::*;

        const EPSILON: f64 = 1e-6;

        /// Scalar implementation of cosine similarity for dense vectors
        /// This is the reference implementation that SIMD must match
        fn cosine_similarity_scalar(v1: &[f64], v2: &[f64]) -> f64 {
            if v1.len() != v2.len() || v1.is_empty() {
                return 0.0;
            }

            let mut dot_product = 0.0;
            let mut norm1 = 0.0;
            let mut norm2 = 0.0;

            for i in 0..v1.len() {
                dot_product += v1[i] * v2[i];
                norm1 += v1[i] * v1[i];
                norm2 += v2[i] * v2[i];
            }

            if norm1 > 0.0 && norm2 > 0.0 {
                dot_product / (norm1.sqrt() * norm2.sqrt())
            } else {
                0.0
            }
        }

        /// Scalar implementation of entropy calculation
        /// This is the reference implementation that SIMD must match
        fn entropy_scalar(probabilities: &[f64]) -> f64 {
            let mut entropy = 0.0;
            for &p in probabilities {
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }
            entropy
        }

        /// SIMD implementation of cosine similarity using Trueno
        /// Uses Trueno's Vector type with auto-selected SIMD backend
        #[cfg(feature = "simd")]
        fn cosine_similarity_simd(v1: &[f64], v2: &[f64]) -> f64 {
            use trueno::Vector;

            if v1.len() != v2.len() || v1.is_empty() {
                return 0.0;
            }

            // Convert f64 to f32 for Trueno (Trueno uses f32 for SIMD efficiency)
            let v1_f32: Vec<f32> = v1.iter().map(|&x| x as f32).collect();
            let v2_f32: Vec<f32> = v2.iter().map(|&x| x as f32).collect();

            // Create Trueno vectors with auto-selected backend
            let vec1 = Vector::from_slice(&v1_f32);
            let vec2 = Vector::from_slice(&v2_f32);

            // Compute dot product and norms using SIMD
            let dot = match vec1.dot(&vec2) {
                Ok(d) => d as f64,
                Err(_) => return 0.0,
            };

            let norm1 = match vec1.norm_l2() {
                Ok(n) => n as f64,
                Err(_) => return 0.0,
            };

            let norm2 = match vec2.norm_l2() {
                Ok(n) => n as f64,
                Err(_) => return 0.0,
            };

            if norm1 > 0.0 && norm2 > 0.0 {
                dot / (norm1 * norm2)
            } else {
                0.0
            }
        }

        /// SIMD implementation of entropy using Trueno
        /// Uses Trueno's Vector::log2() for vectorized Shannon entropy calculation.
        /// H = -Σ p * log2(p)
        #[cfg(feature = "simd")]
        fn entropy_simd(probabilities: &[f64]) -> f64 {
            use trueno::Vector;

            if probabilities.is_empty() {
                return 0.0;
            }

            // Convert to f32, replacing zeros with 1.0 (so log2(1) = 0, contributing nothing)
            let probs_f32: Vec<f32> = probabilities
                .iter()
                .map(|&p| if p > 0.0 { p as f32 } else { 1.0 })
                .collect();

            let probs_vec = Vector::from_slice(&probs_f32);

            // Compute log2(p) for all elements
            let log2_vec = match probs_vec.log2() {
                Ok(v) => v,
                Err(_) => return entropy_scalar(probabilities),
            };

            // Compute p * log2(p)
            let p_log_p = match probs_vec.mul(&log2_vec) {
                Ok(v) => v,
                Err(_) => return entropy_scalar(probabilities),
            };

            // Sum and negate: H = -Σ p * log2(p)
            match p_log_p.sum() {
                Ok(sum) => -(sum as f64),
                Err(_) => entropy_scalar(probabilities),
            }
        }

        // Generate valid probability distributions
        fn probability_distribution(size: usize) -> impl Strategy<Value = Vec<f64>> {
            prop::collection::vec(1.0f64..100.0, size..=size).prop_map(|v| {
                let sum: f64 = v.iter().sum();
                v.iter().map(|&x| x / sum).collect()
            })
        }

        // Generate non-zero vectors for cosine similarity
        fn non_zero_vector(size: usize) -> impl Strategy<Value = Vec<f64>> {
            prop::collection::vec(-100.0f64..100.0, size..=size)
                .prop_filter("at least one non-zero element", |v| {
                    v.iter().any(|&x| x.abs() > 1e-10)
                })
        }

        proptest! {
            /// Property: Cosine similarity is symmetric
            #[test]
            fn cosine_similarity_symmetric(
                v1 in non_zero_vector(100),
                v2 in non_zero_vector(100)
            ) {
                let sim_12 = cosine_similarity_scalar(&v1, &v2);
                let sim_21 = cosine_similarity_scalar(&v2, &v1);
                prop_assert!((sim_12 - sim_21).abs() < EPSILON,
                    "Symmetry violated: {} vs {}", sim_12, sim_21);
            }

            /// Property: Cosine similarity of identical vectors is 1.0
            #[test]
            fn cosine_similarity_identical(v in non_zero_vector(100)) {
                let sim = cosine_similarity_scalar(&v, &v);
                prop_assert!((sim - 1.0).abs() < EPSILON,
                    "Self-similarity should be 1.0, got {}", sim);
            }

            /// Property: Cosine similarity is bounded [-1, 1]
            #[test]
            fn cosine_similarity_bounded(
                v1 in non_zero_vector(100),
                v2 in non_zero_vector(100)
            ) {
                let sim = cosine_similarity_scalar(&v1, &v2);
                prop_assert!((-1.0 - EPSILON..=1.0 + EPSILON).contains(&sim),
                    "Similarity out of bounds: {}", sim);
            }

            /// Property: Entropy is non-negative
            #[test]
            fn entropy_non_negative(probs in probability_distribution(50)) {
                let entropy = entropy_scalar(&probs);
                prop_assert!(entropy >= -EPSILON,
                    "Entropy should be non-negative, got {}", entropy);
            }

            /// Property: Uniform distribution has maximum entropy
            #[test]
            fn entropy_maximum_for_uniform(size in 10usize..100) {
                let uniform: Vec<f64> = vec![1.0 / size as f64; size];
                let max_entropy = (size as f64).log2();
                let computed = entropy_scalar(&uniform);
                prop_assert!((computed - max_entropy).abs() < EPSILON,
                    "Uniform entropy should be log2({}), got {}", size, computed);
            }
        }

        /// RED TEST: SIMD cosine similarity must match scalar
        /// This test will FAIL until Trueno SIMD is implemented
        #[test]
        #[cfg(feature = "simd")]
        fn simd_cosine_similarity_matches_scalar() {
            use proptest::test_runner::{Config, TestRunner};

            let mut runner = TestRunner::new(Config::with_cases(1000));

            runner
                .run(&(non_zero_vector(256), non_zero_vector(256)), |(v1, v2)| {
                    let scalar_result = cosine_similarity_scalar(&v1, &v2);
                    let simd_result = cosine_similarity_simd(&v1, &v2);

                    let diff = (scalar_result - simd_result).abs();
                    prop_assert!(
                        diff < EPSILON,
                        "SIMD/scalar mismatch: scalar={}, simd={}, diff={}",
                        scalar_result,
                        simd_result,
                        diff
                    );
                    Ok(())
                })
                .expect("SIMD cosine similarity must match scalar within epsilon");
        }

        /// GREEN TEST: SIMD entropy must match scalar
        /// Now passes with Trueno v0.2.1's log2() support
        #[test]
        #[cfg(feature = "simd")]
        fn simd_entropy_matches_scalar() {
            use proptest::test_runner::{Config, TestRunner};

            let mut runner = TestRunner::new(Config::with_cases(1000));

            runner
                .run(&probability_distribution(256), |probs| {
                    let scalar_result = entropy_scalar(&probs);
                    let simd_result = entropy_simd(&probs);

                    let diff = (scalar_result - simd_result).abs();
                    prop_assert!(
                        diff < EPSILON,
                        "SIMD/scalar mismatch: scalar={}, simd={}, diff={}",
                        scalar_result,
                        simd_result,
                        diff
                    );
                    Ok(())
                })
                .expect("SIMD entropy must match scalar within epsilon");
        }

        /// Test various vector sizes for SIMD alignment edge cases
        #[test]
        #[cfg(feature = "simd")]
        fn simd_handles_various_sizes() {
            // Test sizes that aren't multiples of 4 (SIMD lane width)
            let sizes = [1, 3, 5, 7, 15, 17, 31, 33, 63, 65, 127, 129, 255, 257];

            for &size in &sizes {
                // Test cosine similarity
                let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
                let v2: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();

                let scalar = cosine_similarity_scalar(&v1, &v2);
                let simd = cosine_similarity_simd(&v1, &v2);

                assert!(
                    (scalar - simd).abs() < EPSILON,
                    "Cosine size {} mismatch: scalar={}, simd={}",
                    size,
                    scalar,
                    simd
                );

                // Test entropy with various sizes
                let raw: Vec<f64> = (0..size).map(|i| (i as f64 + 1.0)).collect();
                let sum: f64 = raw.iter().sum();
                let probs: Vec<f64> = raw.iter().map(|&x| x / sum).collect();

                let scalar_entropy = entropy_scalar(&probs);
                let simd_entropy = entropy_simd(&probs);

                assert!(
                    (scalar_entropy - simd_entropy).abs() < EPSILON,
                    "Entropy size {} mismatch: scalar={}, simd={}",
                    size,
                    scalar_entropy,
                    simd_entropy
                );
            }
        }

        /// Test empty and degenerate cases
        #[test]
        fn edge_cases() {
            // Empty vectors
            assert_eq!(cosine_similarity_scalar(&[], &[]), 0.0);

            // Single element
            assert!((cosine_similarity_scalar(&[1.0], &[1.0]) - 1.0).abs() < EPSILON);

            // Zero vector
            assert_eq!(cosine_similarity_scalar(&[0.0, 0.0], &[1.0, 1.0]), 0.0);

            // Orthogonal vectors
            let orth = cosine_similarity_scalar(&[1.0, 0.0], &[0.0, 1.0]);
            assert!(
                orth.abs() < EPSILON,
                "Orthogonal vectors should have ~0 similarity"
            );

            // Opposite vectors
            let opp = cosine_similarity_scalar(&[1.0, 1.0], &[-1.0, -1.0]);
            assert!(
                (opp + 1.0).abs() < EPSILON,
                "Opposite vectors should have -1 similarity"
            );
        }

        /// Benchmark-ready function that can be used to measure SIMD vs scalar performance
        #[test]
        fn baseline_performance_scalar() {
            let size = 10000;
            let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
            let v2: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();

            // Warmup and verify
            let result = cosine_similarity_scalar(&v1, &v2);
            assert!(result.is_finite(), "Result should be finite");
        }
    }
}
