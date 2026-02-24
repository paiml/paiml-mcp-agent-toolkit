// Unit tests for DuplicationDetector — data structures, similarity, LCS
// Included from duplication.rs — shares parent module scope (no `use` imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }

    // === DuplicationDetector tests ===

    #[test]
    fn test_duplication_detector_new() {
        let detector = DuplicationDetector::new();
        assert_eq!(detector.min_token_sequence, 50);
        assert_eq!(detector.similarity_threshold, 0.85);
    }

    #[test]
    fn test_duplication_detector_category() {
        let detector = DuplicationDetector::new();
        assert_eq!(detector.category(), MetricCategory::Duplication);
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

    #[test]
    fn test_similarity_identical_sequences() {
        let detector = DuplicationDetector::new();

        let tokens = vec![
            Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
            Token { kind: "identifier".to_string(), text: "x".to_string(), normalized: "$VAR".to_string() },
        ];

        let seq1 = TokenSequence {
            tokens: tokens.clone(),
            start_byte: 0,
            end_byte: 10,
        };

        let seq2 = TokenSequence {
            tokens,
            start_byte: 20,
            end_byte: 30,
        };

        let similarity = detector.calculate_similarity(&seq1, &seq2);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_similarity_completely_different_sequences() {
        let detector = DuplicationDetector::new();

        let seq1 = TokenSequence {
            tokens: vec![
                Token { kind: "fn".to_string(), text: "fn".to_string(), normalized: "fn".to_string() },
            ],
            start_byte: 0,
            end_byte: 2,
        };

        let seq2 = TokenSequence {
            tokens: vec![
                Token { kind: "struct".to_string(), text: "struct".to_string(), normalized: "struct".to_string() },
            ],
            start_byte: 10,
            end_byte: 16,
        };

        let similarity = detector.calculate_similarity(&seq1, &seq2);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_longest_common_subsequence() {
        let detector = DuplicationDetector::new();

        let seq1 = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let seq2 = vec!["a".to_string(), "c".to_string(), "d".to_string()];

        let lcs = detector.longest_common_subsequence(&seq1, &seq2);
        assert_eq!(lcs, 3); // "a", "c", "d"
    }

    #[test]
    fn test_longest_common_subsequence_empty() {
        let detector = DuplicationDetector::new();

        let seq1: Vec<String> = vec![];
        let seq2: Vec<String> = vec![];

        let lcs = detector.longest_common_subsequence(&seq1, &seq2);
        assert_eq!(lcs, 0);
    }

    #[test]
    fn test_longest_common_subsequence_no_match() {
        let detector = DuplicationDetector::new();

        let seq1 = vec!["a".to_string(), "b".to_string()];
        let seq2 = vec!["c".to_string(), "d".to_string()];

        let lcs = detector.longest_common_subsequence(&seq1, &seq2);
        assert_eq!(lcs, 0);
    }

    // === Token tests ===

    #[test]
    fn test_token_creation() {
        let token = Token {
            kind: "identifier".to_string(),
            text: "my_variable".to_string(),
            normalized: "$VAR".to_string(),
        };

        assert_eq!(token.kind, "identifier");
        assert_eq!(token.text, "my_variable");
        assert_eq!(token.normalized, "$VAR");
    }

    #[test]
    fn test_token_clone() {
        let token = Token {
            kind: "let".to_string(),
            text: "let".to_string(),
            normalized: "let".to_string(),
        };

        let cloned = token.clone();
        assert_eq!(token.kind, cloned.kind);
        assert_eq!(token.text, cloned.text);
        assert_eq!(token.normalized, cloned.normalized);
    }

    // === TokenSequence tests ===

    #[test]
    fn test_token_sequence_creation() {
        let seq = TokenSequence {
            tokens: vec![],
            start_byte: 10,
            end_byte: 50,
        };

        assert!(seq.tokens.is_empty());
        assert_eq!(seq.start_byte, 10);
        assert_eq!(seq.end_byte, 50);
    }

    #[test]
    fn test_token_sequence_with_tokens() {
        let tokens = vec![
            Token { kind: "fn".to_string(), text: "fn".to_string(), normalized: "fn".to_string() },
            Token { kind: "identifier".to_string(), text: "main".to_string(), normalized: "$VAR".to_string() },
        ];

        let seq = TokenSequence {
            tokens,
            start_byte: 0,
            end_byte: 7,
        };

        assert_eq!(seq.tokens.len(), 2);
    }

    // === CloneSet tests ===

    #[test]
    fn test_clone_set_new() {
        let clone_set = CloneSet::new();
        assert!(clone_set.clones.is_empty());
    }

    #[test]
    fn test_clone_set_add_clone() {
        let mut clone_set = CloneSet::new();

        let seq = TokenSequence {
            tokens: vec![
                Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
            ],
            start_byte: 0,
            end_byte: 3,
        };

        clone_set.add_clone(CloneType::Exact, vec![seq.clone(), seq]);

        assert_eq!(clone_set.clones.len(), 1);
    }

    #[test]
    fn test_clone_set_total_tokens() {
        let mut clone_set = CloneSet::new();

        let seq1 = TokenSequence {
            tokens: vec![
                Token { kind: "a".to_string(), text: "a".to_string(), normalized: "a".to_string() },
                Token { kind: "b".to_string(), text: "b".to_string(), normalized: "b".to_string() },
            ],
            start_byte: 0,
            end_byte: 2,
        };

        let seq2 = TokenSequence {
            tokens: vec![
                Token { kind: "c".to_string(), text: "c".to_string(), normalized: "c".to_string() },
            ],
            start_byte: 10,
            end_byte: 11,
        };

        clone_set.add_clone(CloneType::Exact, vec![seq1, seq2]);

        // 2 tokens + 1 token = 3 total
        assert_eq!(clone_set.total_tokens(), 3);
    }

    #[test]
    fn test_clone_set_total_tokens_empty() {
        let clone_set = CloneSet::new();
        assert_eq!(clone_set.total_tokens(), 0);
    }
}
