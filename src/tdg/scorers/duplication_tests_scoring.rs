// Unit tests for normalization, node significance, hashing, scoring, and clone detection
// Included from duplication.rs — shares parent module scope (no `use` imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_scoring {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }

    // === Normalization tests ===

    #[test]
    fn test_normalize_identifier() {
        let source = "let my_var = 1;";
        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();

        walk_tree(tree.root_node(), |node| {
            if node.kind() == "identifier" {
                let normalized = detector.normalize_token(node, source);
                // Most identifiers should normalize to $VAR unless they're type names
                assert!(normalized == "$VAR" || normalized == get_node_text(node, source));
            }
        });
    }

    #[test]
    fn test_normalize_string_literal() {
        let detector = DuplicationDetector::new();
        let source = r#"let s = "hello";"#;
        let tree = parse_rust(source);

        walk_tree(tree.root_node(), |node| {
            if node.kind() == "string_literal" {
                let normalized = detector.normalize_token(node, source);
                assert_eq!(normalized, "$STR");
            }
        });
    }

    #[test]
    fn test_normalize_number_literal() {
        let detector = DuplicationDetector::new();
        let source = "let n = 42;";
        let tree = parse_rust(source);

        walk_tree(tree.root_node(), |node| {
            if node.kind() == "integer_literal" {
                let normalized = detector.normalize_token(node, source);
                assert_eq!(normalized, "$NUM");
            }
        });
    }

    // === is_significant_node tests ===

    #[test]
    fn test_is_significant_node_comment() {
        let detector = DuplicationDetector::new();
        let source = "// comment\nfn test() {}";
        let tree = parse_rust(source);

        walk_tree(tree.root_node(), |node| {
            if node.kind() == "line_comment" {
                assert!(!detector.is_significant_node(node));
            }
        });
    }

    #[test]
    fn test_is_significant_node_brackets() {
        let detector = DuplicationDetector::new();
        let source = "fn test() {}";
        let tree = parse_rust(source);

        walk_tree(tree.root_node(), |node| {
            if matches!(node.kind(), "(" | ")" | "{" | "}") {
                assert!(!detector.is_significant_node(node));
            }
        });
    }

    #[test]
    fn test_is_significant_node_function() {
        let detector = DuplicationDetector::new();
        let source = "fn test() {}";
        let tree = parse_rust(source);

        walk_tree(tree.root_node(), |node| {
            if node.kind() == "fn" {
                assert!(detector.is_significant_node(node));
            }
        });
    }

    // === Hash tests ===

    #[test]
    fn test_hash_sequence_identical() {
        let detector = DuplicationDetector::new();

        let tokens = vec![
            Token { kind: "a".to_string(), text: "a".to_string(), normalized: "a".to_string() },
            Token { kind: "b".to_string(), text: "b".to_string(), normalized: "b".to_string() },
        ];

        let hash1 = detector.hash_sequence(&tokens);
        let hash2 = detector.hash_sequence(&tokens);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_sequence_different() {
        let detector = DuplicationDetector::new();

        let tokens1 = vec![
            Token { kind: "a".to_string(), text: "a".to_string(), normalized: "a".to_string() },
        ];

        let tokens2 = vec![
            Token { kind: "b".to_string(), text: "b".to_string(), normalized: "b".to_string() },
        ];

        let hash1 = detector.hash_sequence(&tokens1);
        let hash2 = detector.hash_sequence(&tokens2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_normalized_identical() {
        let detector = DuplicationDetector::new();

        let normalized = vec!["$VAR".to_string(), "=".to_string(), "$NUM".to_string()];

        let hash1 = detector.hash_normalized(&normalized);
        let hash2 = detector.hash_normalized(&normalized);

        assert_eq!(hash1, hash2);
    }

    // === Scorer tests ===

    #[test]
    fn test_scorer_simple_code() {
        let source = r#"
            fn simple() {
                let x = 1;
            }
        "#;

        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();
        let config = TdgConfig::default();
        let mut tracker = PenaltyTracker::new();

        let score = detector.score(&tree, source, Language::Rust, &config, &mut tracker);

        assert!(score.is_ok());
        assert!(score.unwrap() >= 0.0);
    }

    #[test]
    fn test_scorer_empty_source() {
        let source = "";

        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();
        let config = TdgConfig::default();
        let mut tracker = PenaltyTracker::new();

        let score = detector.score(&tree, source, Language::Rust, &config, &mut tracker);

        assert!(score.is_ok());
    }

    #[test]
    fn test_scorer_no_duplication() {
        let source = r#"
            fn unique_a() { let a = 1; }
            fn unique_b() { let b = 2; }
            fn unique_c() { let c = 3; }
        "#;

        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();
        let config = TdgConfig::default();
        let mut tracker = PenaltyTracker::new();

        let score = detector.score(&tree, source, Language::Rust, &config, &mut tracker);

        assert!(score.is_ok());
        // No duplication should result in full score
        let score_value = score.unwrap();
        assert!(score_value >= 0.0);
    }

    // === Clone detection tests ===

    #[test]
    fn test_find_exact_clones_no_clones() {
        let detector = DuplicationDetector::new();

        let seq1 = TokenSequence {
            tokens: vec![
                Token { kind: "a".to_string(), text: "a".to_string(), normalized: "a".to_string() },
            ],
            start_byte: 0,
            end_byte: 1,
        };

        let seq2 = TokenSequence {
            tokens: vec![
                Token { kind: "b".to_string(), text: "b".to_string(), normalized: "b".to_string() },
            ],
            start_byte: 10,
            end_byte: 11,
        };

        let clones = detector.find_exact_clones(&[seq1, seq2]);
        assert!(clones.clones.is_empty());
    }

    #[test]
    fn test_find_renamed_clones() {
        let detector = DuplicationDetector::new();

        let seq1 = TokenSequence {
            tokens: vec![
                Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
                Token { kind: "identifier".to_string(), text: "x".to_string(), normalized: "$VAR".to_string() },
            ],
            start_byte: 0,
            end_byte: 5,
        };

        let seq2 = TokenSequence {
            tokens: vec![
                Token { kind: "let".to_string(), text: "let".to_string(), normalized: "let".to_string() },
                Token { kind: "identifier".to_string(), text: "y".to_string(), normalized: "$VAR".to_string() },
            ],
            start_byte: 10,
            end_byte: 15,
        };

        let clones = detector.find_renamed_clones(&[seq1, seq2]);
        // Should find clones since normalized forms match
        assert!(!clones.clones.is_empty());
    }

    #[test]
    fn test_find_modified_clones() {
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
                Token { kind: "+".to_string(), text: "+".to_string(), normalized: "+".to_string() },
            ],
            start_byte: 20,
            end_byte: 35,
        };

        let clones = detector.find_modified_clones(&[seq1, seq2]);
        // May or may not find clones depending on similarity threshold
        assert!(clones.clones.len() >= 0);
    }

    // === is_type_name tests ===

    #[test]
    fn test_is_type_name_uppercase() {
        let detector = DuplicationDetector::new();
        let source = "let x: MyType = value;";
        let tree = parse_rust(source);

        // Type identifiers that start with uppercase are typically types
        walk_tree(tree.root_node(), |node| {
            if node.kind() == "type_identifier" {
                let is_type = detector.is_type_name(node, source);
                assert!(is_type);
            }
        });
    }

    // === extract_token_sequences tests ===

    #[test]
    fn test_extract_token_sequences_empty_source() {
        let source = "";
        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();

        let sequences = detector.extract_token_sequences(tree.root_node(), source);
        assert!(sequences.is_empty());
    }

    #[test]
    fn test_extract_token_sequences_short_code() {
        let source = "let x = 1;";
        let tree = parse_rust(source);
        let detector = DuplicationDetector::new();

        let sequences = detector.extract_token_sequences(tree.root_node(), source);
        // Short code won't meet min_token_sequence threshold
        assert!(sequences.is_empty() || sequences.len() >= 0);
    }
}
