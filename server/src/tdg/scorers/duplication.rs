use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use super::{Scorer, walk_tree, get_node_text};

pub struct DuplicationDetector {
    min_token_sequence: usize,
    similarity_threshold: f32,
}

impl DuplicationDetector {
    pub fn new() -> Self {
        Self {
            min_token_sequence: 50,
            similarity_threshold: 0.85,
        }
    }
    
    fn extract_token_sequences(&self, root: Node, source: &str) -> Vec<TokenSequence> {
        let mut sequences = Vec::new();
        let mut current_tokens = Vec::new();
        let mut start_byte = 0;
        
        walk_tree(root, |node| {
            if self.is_significant_node(node) {
                let token = self.node_to_token(node, source);
                if current_tokens.is_empty() {
                    start_byte = node.start_byte();
                }
                current_tokens.push(token);
                
                if current_tokens.len() >= self.min_token_sequence {
                    sequences.push(TokenSequence {
                        tokens: current_tokens.clone(),
                        start_byte,
                        end_byte: node.end_byte(),
                    });
                }
            } else if node.kind() == ";" || node.kind() == "{" || node.kind() == "}" {
                current_tokens.clear();
            }
        });
        
        sequences
    }
    
    fn is_significant_node(&self, node: Node) -> bool {
        !matches!(
            node.kind(),
            "comment" | "line_comment" | "block_comment" | 
            "whitespace" | "(" | ")" | "{" | "}" | "[" | "]" | ";" | ","
        )
    }
    
    fn node_to_token(&self, node: Node, source: &str) -> Token {
        Token {
            kind: node.kind().to_string(),
            text: get_node_text(node, source).to_string(),
            normalized: self.normalize_token(node, source),
        }
    }
    
    fn normalize_token(&self, node: Node, source: &str) -> String {
        match node.kind() {
            "identifier" if !self.is_type_name(node, source) => "$VAR".to_string(),
            "string_literal" | "string" | "raw_string_literal" => "$STR".to_string(),
            "integer_literal" | "float_literal" | "number" => "$NUM".to_string(),
            _ => get_node_text(node, source).to_string(),
        }
    }
    
    fn is_type_name(&self, node: Node, source: &str) -> bool {
        if let Some(parent) = node.parent() {
            matches!(
                parent.kind(),
                "type_identifier" | "generic_type" | "reference_type" | "pointer_type"
            )
        } else {
            let text = get_node_text(node, source);
            text.chars().next().map_or(false, |c| c.is_uppercase())
        }
    }
    
    fn find_exact_clones(&self, sequences: &[TokenSequence]) -> CloneSet {
        let mut clones = CloneSet::new();
        let mut seen = HashMap::new();
        
        for seq in sequences {
            let hash = self.hash_sequence(&seq.tokens);
            seen.entry(hash)
                .or_insert_with(Vec::new)
                .push(seq.clone());
        }
        
        for (_, group) in seen {
            if group.len() > 1 {
                clones.add_clone(CloneType::Exact, group);
            }
        }
        
        clones
    }
    
    fn find_renamed_clones(&self, sequences: &[TokenSequence]) -> CloneSet {
        let mut clones = CloneSet::new();
        let mut normalized_map = HashMap::new();
        
        for seq in sequences {
            let normalized: Vec<String> = seq.tokens.iter()
                .map(|t| t.normalized.clone())
                .collect();
            let hash = self.hash_normalized(&normalized);
            normalized_map.entry(hash)
                .or_insert_with(Vec::new)
                .push(seq.clone());
        }
        
        for (_, group) in normalized_map {
            if group.len() > 1 {
                clones.add_clone(CloneType::Renamed, group);
            }
        }
        
        clones
    }
    
    fn find_modified_clones(&self, sequences: &[TokenSequence]) -> CloneSet {
        let mut clones = CloneSet::new();
        
        for i in 0..sequences.len() {
            for j in i + 1..sequences.len() {
                let similarity = self.calculate_similarity(&sequences[i], &sequences[j]);
                if similarity >= self.similarity_threshold && similarity < 1.0 {
                    clones.add_clone(
                        CloneType::Modified,
                        vec![sequences[i].clone(), sequences[j].clone()]
                    );
                }
            }
        }
        
        clones
    }
    
    fn calculate_similarity(&self, seq1: &TokenSequence, seq2: &TokenSequence) -> f32 {
        let normalized1: Vec<String> = seq1.tokens.iter().map(|t| t.normalized.clone()).collect();
        let normalized2: Vec<String> = seq2.tokens.iter().map(|t| t.normalized.clone()).collect();
        
        let lcs_length = self.longest_common_subsequence(&normalized1, &normalized2);
        let max_length = normalized1.len().max(normalized2.len()) as f32;
        
        if max_length > 0.0 {
            lcs_length as f32 / max_length
        } else {
            0.0
        }
    }
    
    fn longest_common_subsequence(&self, seq1: &[String], seq2: &[String]) -> usize {
        let m = seq1.len();
        let n = seq2.len();
        let mut dp = vec![vec![0; n + 1]; m + 1];
        
        for i in 1..=m {
            for j in 1..=n {
                if seq1[i - 1] == seq2[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }
        
        dp[m][n]
    }
    
    fn hash_sequence(&self, tokens: &[Token]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for token in tokens {
            token.text.hash(&mut hasher);
        }
        hasher.finish()
    }
    
    fn hash_normalized(&self, normalized: &[String]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for token in normalized {
            token.hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl Scorer for DuplicationDetector {
    fn score(&self, tree: &Tree, source: &str, _language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.duplication;
        let root = tree.root_node();
        
        let sequences = self.extract_token_sequences(root, source);
        if sequences.is_empty() {
            return Ok(points);
        }
        
        let exact_clones = self.find_exact_clones(&sequences);
        let renamed_clones = self.find_renamed_clones(&sequences);
        let modified_clones = self.find_modified_clones(&sequences);
        
        let total_tokens = source.len();
        let duplicate_tokens = 
            exact_clones.total_tokens() +
            (renamed_clones.total_tokens() as f32 * 0.8) as usize +
            (modified_clones.total_tokens() as f32 * 0.5) as usize;
        
        let duplication_ratio = duplicate_tokens as f32 / total_tokens.max(1) as f32;
        
        let penalty = (duplication_ratio * 40.0).min(20.0);
        if penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                format!("duplication_{:.2}", duplication_ratio),
                MetricCategory::Duplication,
                penalty,
                format!("Code duplication: {:.1}%", duplication_ratio * 100.0)
            ) {
                points -= applied;
            }
        }
        
        Ok(points.max(0.0))
    }
    
    fn category(&self) -> MetricCategory {
        MetricCategory::Duplication
    }
}

#[derive(Clone, Debug)]
struct Token {
    kind: String,
    text: String,
    normalized: String,
}

#[derive(Clone, Debug)]
struct TokenSequence {
    tokens: Vec<Token>,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug)]
enum CloneType {
    Exact,
    Renamed,
    Modified,
}

#[derive(Debug)]
struct CloneSet {
    clones: Vec<(CloneType, Vec<TokenSequence>)>,
}

impl CloneSet {
    fn new() -> Self {
        Self { clones: Vec::new() }
    }
    
    fn add_clone(&mut self, clone_type: CloneType, sequences: Vec<TokenSequence>) {
        self.clones.push((clone_type, sequences));
    }
    
    fn total_tokens(&self) -> usize {
        self.clones.iter()
            .map(|(_, sequences)| {
                sequences.iter()
                    .map(|seq| seq.tokens.len())
                    .sum::<usize>()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }
    
    #[test]
    fn test_exact_clone_detection() {
        let source = r#"
            fn process_a(x: i32) -> i32 {
                let result = x * 2;
                if result > 100 {
                    return result + 10;
                }
                result
            }
            
            fn process_b(x: i32) -> i32 {
                let result = x * 2;
                if result > 100 {
                    return result + 10;
                }
                result
            }
        "#;
        
        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();
        let sequences = detector.extract_token_sequences(tree.root_node(), source);
        assert!(!sequences.is_empty());
    }
    
    #[test]
    fn test_similarity_calculation() {
        let detector = DuplicationDetector::new();
        
        let seq1 = TokenSequence {
            tokens: vec![
                Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
                Token { kind: "identifier".to_string(), text: "x".to_string(), normalized: "$VAR".to_string() },
                Token { kind: "=".to_string(), text: "=".to_string(), normalized: "=".to_string() },
                Token { kind: "number".to_string(), text: "1".to_string(), normalized: "$NUM".to_string() },
            ],
            start_byte: 0,
            end_byte: 10,
        };
        
        let seq2 = TokenSequence {
            tokens: vec![
                Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
                Token { kind: "identifier".to_string(), text: "y".to_string(), normalized: "$VAR".to_string() },
                Token { kind: "=".to_string(), text: "=".to_string(), normalized: "=".to_string() },
                Token { kind: "number".to_string(), text: "2".to_string(), normalized: "$NUM".to_string() },
            ],
            start_byte: 20,
            end_byte: 30,
        };
        
        let similarity = detector.calculate_similarity(&seq1, &seq2);
        assert!(similarity > 0.9);
    }
}