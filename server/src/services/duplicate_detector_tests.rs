//! Comprehensive tests for duplicate_detector module to achieve 80% code coverage

#[cfg(test)]
mod tests {
    use super::super::duplicate_detector::*;
    use std::path::PathBuf;
    use std::collections::HashMap;

    // Helper function to create test tokens
    fn create_test_tokens(code: &str) -> Vec<Token> {
        code.split_whitespace()
            .map(|word| {
                if matches!(word, "fn" | "let" | "if" | "else" | "return") {
                    Token::new(TokenKind::Keyword(word.to_string()))
                } else if word.chars().all(|c| c.is_numeric()) {
                    Token::new(TokenKind::Literal(word.to_string()))
                } else if matches!(word, "(" | ")" | "{" | "}" | ";" | ",") {
                    Token::new(TokenKind::Delimiter(word.to_string()))
                } else if matches!(word, "+" | "-" | "*" | "/" | "=" | "==") {
                    Token::new(TokenKind::Operator(word.to_string()))
                } else {
                    Token::new(TokenKind::Identifier(word.to_string()))
                }
            })
            .collect()
    }

    #[test]
    fn test_token_creation_and_hash() {
        let token = Token::new(TokenKind::Identifier("test".to_string()));
        assert_eq!(token.text, "test");
        assert!(matches!(token.kind, TokenKind::Identifier(_)));
        
        let hash = token.hash();
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_all_token_kinds() {
        let identifier = Token::new(TokenKind::Identifier("var".to_string()));
        assert_eq!(identifier.text, "var");
        
        let literal = Token::new(TokenKind::Literal("42".to_string()));
        assert_eq!(literal.text, "42");
        
        let keyword = Token::new(TokenKind::Keyword("fn".to_string()));
        assert_eq!(keyword.text, "fn");
        
        let operator = Token::new(TokenKind::Operator("+".to_string()));
        assert_eq!(operator.text, "+");
        
        let delimiter = Token::new(TokenKind::Delimiter("{".to_string()));
        assert_eq!(delimiter.text, "{");
        
        let comment = Token::new(TokenKind::Comment);
        assert_eq!(comment.text, "//");
        
        let whitespace = Token::new(TokenKind::Whitespace);
        assert_eq!(whitespace.text, " ");
    }

    #[test]
    fn test_minhash_signature_jaccard_similarity() {
        let sig1 = MinHashSignature {
            values: vec![1, 2, 3, 4, 5],
        };
        
        let sig2 = MinHashSignature {
            values: vec![1, 2, 3, 6, 7],
        };
        
        let similarity = sig1.jaccard_similarity(&sig2);
        assert_eq!(similarity, 0.6); // 3 matches out of 5
        
        // Test identical signatures
        let similarity_same = sig1.jaccard_similarity(&sig1);
        assert_eq!(similarity_same, 1.0);
        
        // Test completely different signatures
        let sig3 = MinHashSignature {
            values: vec![10, 20, 30, 40, 50],
        };
        let similarity_diff = sig1.jaccard_similarity(&sig3);
        assert_eq!(similarity_diff, 0.0);
    }

    #[test]
    fn test_clone_type_display() {
        assert_eq!(format!("{}", CloneType::Exact), "Type-1 (Exact)");
        assert_eq!(format!("{}", CloneType::Renamed), "Type-2 (Renamed)");
        assert_eq!(format!("{}", CloneType::NearMiss), "Type-3 (Near-miss)");
        assert_eq!(format!("{}", CloneType::Semantic), "Type-4 (Semantic)");
    }

    #[test]
    fn test_duplicate_detection_config_default() {
        let config = DuplicateDetectionConfig::default();
        assert_eq!(config.min_tokens, 50);
        assert_eq!(config.similarity_threshold, 0.70);
        assert_eq!(config.shingle_size, 5);
        assert_eq!(config.num_hash_functions, 200);
        assert_eq!(config.num_bands, 20);
        assert_eq!(config.rows_per_band, 10);
        assert!(config.normalize_identifiers);
        assert!(config.normalize_literals);
        assert!(config.ignore_comments);
        assert_eq!(config.min_group_size, 2);
    }

    #[test]
    fn test_universal_feature_extractor() {
        let extractor = UniversalFeatureExtractor::new();
        
        // Test Rust code
        let rust_tokens = extractor.extract_features(
            "fn main() { let x = 42; println!(\"Hello\"); }",
            Language::Rust
        );
        assert!(!rust_tokens.is_empty());
        
        // Test TypeScript code
        let ts_tokens = extractor.extract_features(
            "function main(): void { const x = 42; console.log('Hello'); }",
            Language::TypeScript
        );
        assert!(!ts_tokens.is_empty());
        
        // Test Python code
        let py_tokens = extractor.extract_features(
            "def main():\n    x = 42\n    print('Hello')",
            Language::Python
        );
        assert!(!py_tokens.is_empty());
    }

    #[test]
    fn test_extract_identifiers() {
        let extractor = UniversalFeatureExtractor::new();
        
        // Test various identifier patterns
        let code = "let variable_name = 42; const CONSTANT = true; function myFunction() {}";
        let identifiers = extractor.extract_identifiers(code);
        
        assert!(identifiers.contains(&"variable_name".to_string()));
        assert!(identifiers.contains(&"CONSTANT".to_string()));
        assert!(identifiers.contains(&"myFunction".to_string()));
    }

    #[test]
    fn test_extract_literals() {
        let extractor = UniversalFeatureExtractor::new();
        
        // Test various literal patterns
        let code = r#"42 3.14 'single' "double" true false 0xFF 0b101"#;
        let literals = extractor.extract_literals(code);
        
        assert!(literals.contains(&"42".to_string()));
        assert!(literals.contains(&"3.14".to_string()));
        assert!(literals.contains(&"single".to_string()));
        assert!(literals.contains(&"double".to_string()));
        assert!(literals.contains(&"true".to_string()));
        assert!(literals.contains(&"false".to_string()));
    }

    #[test]
    fn test_normalize_tokens() {
        let extractor = UniversalFeatureExtractor::new();
        let config = DuplicateDetectionConfig::default();
        
        let tokens = vec![
            Token::new(TokenKind::Identifier("myVar".to_string())),
            Token::new(TokenKind::Literal("42".to_string())),
            Token::new(TokenKind::Comment),
        ];
        
        let normalized = extractor.normalize_tokens(tokens, &config);
        
        // Should normalize identifier
        assert!(normalized[0].text.starts_with("VAR_"));
        
        // Should normalize literal
        assert_eq!(normalized[1].text, "LITERAL");
        
        // Should remove comment (ignore_comments is true by default)
        assert_eq!(normalized.len(), 2);
    }

    #[test]
    fn test_normalize_tokens_keep_identifiers_and_literals() {
        let extractor = UniversalFeatureExtractor::new();
        let config = DuplicateDetectionConfig {
            normalize_identifiers: false,
            normalize_literals: false,
            ignore_comments: false,
            ..Default::default()
        };
        
        let tokens = vec![
            Token::new(TokenKind::Identifier("myVar".to_string())),
            Token::new(TokenKind::Literal("42".to_string())),
            Token::new(TokenKind::Comment),
        ];
        
        let normalized = extractor.normalize_tokens(tokens.clone(), &config);
        
        // Should keep original identifier
        assert_eq!(normalized[0].text, "myVar");
        
        // Should keep original literal
        assert_eq!(normalized[1].text, "42");
        
        // Should keep comment
        assert_eq!(normalized.len(), 3);
    }

    #[test]
    fn test_minhash_generator() {
        let generator = MinHashGenerator::new(100);
        assert_eq!(generator.seeds.len(), 100);
        
        // Test shingle generation
        let tokens = create_test_tokens("fn test ( ) { return 42 ; }");
        let shingles = generator.generate_shingles(&tokens, 3);
        assert_eq!(shingles.len(), tokens.len().saturating_sub(2)); // n - k + 1
        
        // Test signature computation
        let signature = generator.compute_signature(&shingles);
        assert_eq!(signature.values.len(), 100);
    }

    #[test]
    fn test_code_fragment_creation() {
        let fragment = CodeFragment {
            id: 1,
            file_path: PathBuf::from("test.rs"),
            start_line: 10,
            end_line: 20,
            start_column: 0,
            end_column: 80,
            raw_content: "test content".to_string(),
            tokens: vec![],
            normalized_tokens: vec![],
            signature: MinHashSignature { values: vec![1, 2, 3] },
            hash: 12345,
            language: Language::Rust,
        };
        
        assert_eq!(fragment.id, 1);
        assert_eq!(fragment.file_path, PathBuf::from("test.rs"));
        assert_eq!(fragment.start_line, 10);
        assert_eq!(fragment.end_line, 20);
    }

    #[test]
    fn test_duplicate_detection_engine_basic() {
        let config = DuplicateDetectionConfig {
            min_tokens: 5,
            similarity_threshold: 0.5,
            ..Default::default()
        };
        
        let engine = DuplicateDetectionEngine::new(config);
        
        // Test with identical code
        let files = vec![
            (
                PathBuf::from("file1.rs"),
                "fn test() { let x = 42; return x; }".to_string(),
                Language::Rust,
            ),
            (
                PathBuf::from("file2.rs"),
                "fn test() { let x = 42; return x; }".to_string(),
                Language::Rust,
            ),
        ];
        
        let report = engine.detect_duplicates(&files).unwrap();
        assert!(report.summary.clone_groups > 0);
        assert!(report.summary.duplication_ratio > 0.0);
    }

    #[test]
    fn test_duplicate_detection_different_languages() {
        let config = DuplicateDetectionConfig {
            min_tokens: 5,
            similarity_threshold: 0.6,
            ..Default::default()
        };
        
        let engine = DuplicateDetectionEngine::new(config);
        
        let files = vec![
            (
                PathBuf::from("test.rs"),
                "fn calculate(x: i32) -> i32 { x * 2 }".to_string(),
                Language::Rust,
            ),
            (
                PathBuf::from("test.ts"),
                "function calculate(x: number): number { return x * 2; }".to_string(),
                Language::TypeScript,
            ),
            (
                PathBuf::from("test.py"),
                "def calculate(x): return x * 2".to_string(),
                Language::Python,
            ),
        ];
        
        let report = engine.detect_duplicates(&files).unwrap();
        assert_eq!(report.summary.total_files, 3);
    }

    #[test]
    fn test_extract_fragments() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
            min_tokens: 5,
            ..Default::default()
        });
        
        let file = (
            PathBuf::from("test.rs"),
            "fn one() { println!(\"1\"); }\n\nfn two() { println!(\"2\"); }".to_string(),
            Language::Rust,
        );
        
        let fragments = engine.extract_fragments(&[file]).unwrap();
        assert!(!fragments.is_empty());
    }

    #[test]
    fn test_find_clone_pairs_with_lsh() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
        
        // Create test fragments with similar signatures
        let mut fragments = vec![];
        for i in 0..5 {
            fragments.push(CodeFragment {
                id: i as u64,
                file_path: PathBuf::from(format!("file{}.rs", i)),
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 100,
                raw_content: format!("content {}", i),
                tokens: vec![],
                normalized_tokens: vec![],
                signature: MinHashSignature {
                    values: if i < 3 {
                        vec![1, 2, 3, 4, 5] // Similar signatures for first 3
                    } else {
                        vec![10, 20, 30, 40, 50] // Different for last 2
                    }
                },
                hash: i as u64,
                language: Language::Rust,
            });
        }
        
        let pairs = engine.find_clone_pairs(&fragments).unwrap();
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_group_clones() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
        
        let clone_pairs = vec![
            (1, 2, 0.9),
            (2, 3, 0.85),
            (4, 5, 0.95),
        ];
        
        let groups = engine.group_clones(clone_pairs).unwrap();
        assert_eq!(groups.len(), 2); // Should form 2 groups: {1,2,3} and {4,5}
    }

    #[test]
    fn test_compute_summary() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
        
        let fragments = vec![
            CodeFragment {
                id: 1,
                file_path: PathBuf::from("file1.rs"),
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 100,
                raw_content: String::new(),
                tokens: vec![],
                normalized_tokens: vec![],
                signature: MinHashSignature { values: vec![] },
                hash: 0,
                language: Language::Rust,
            },
            CodeFragment {
                id: 2,
                file_path: PathBuf::from("file2.rs"),
                start_line: 1,
                end_line: 5,
                start_column: 0,
                end_column: 50,
                raw_content: String::new(),
                tokens: vec![],
                normalized_tokens: vec![],
                signature: MinHashSignature { values: vec![] },
                hash: 0,
                language: Language::Rust,
            },
        ];
        
        let groups = vec![
            CloneGroup {
                id: 1,
                clone_type: CloneType::Exact,
                fragments: vec![
                    CloneInstance {
                        file: PathBuf::from("file1.rs"),
                        start_line: 1,
                        end_line: 10,
                        start_column: 0,
                        end_column: 100,
                        similarity_to_representative: 1.0,
                        normalized_hash: 123,
                    },
                ],
                total_lines: 10,
                total_tokens: 50,
                average_similarity: 1.0,
                representative: 1,
            },
        ];
        
        let summary = engine.compute_summary(&fragments, &groups);
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_fragments, 2);
        assert_eq!(summary.clone_groups, 1);
        assert_eq!(summary.total_lines, 15); // 10 + 5
        assert_eq!(summary.duplicate_lines, 10);
    }

    #[test]
    fn test_find_hotspots() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());
        
        let groups = vec![
            CloneGroup {
                id: 1,
                clone_type: CloneType::Exact,
                fragments: vec![
                    CloneInstance {
                        file: PathBuf::from("hotspot.rs"),
                        start_line: 1,
                        end_line: 10,
                        start_column: 0,
                        end_column: 100,
                        similarity_to_representative: 1.0,
                        normalized_hash: 123,
                    },
                    CloneInstance {
                        file: PathBuf::from("hotspot.rs"),
                        start_line: 20,
                        end_line: 30,
                        start_column: 0,
                        end_column: 100,
                        similarity_to_representative: 0.9,
                        normalized_hash: 124,
                    },
                ],
                total_lines: 20,
                total_tokens: 100,
                average_similarity: 0.95,
                representative: 1,
            },
        ];
        
        let hotspots = engine.find_hotspots(&groups);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].file_path, PathBuf::from("hotspot.rs"));
        assert_eq!(hotspots[0].clone_count, 2);
    }

    #[test]
    fn test_is_significant_fragment() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
            min_tokens: 10,
            ..Default::default()
        });
        
        // Test too few tokens
        let small_fragment = CodeFragment {
            id: 1,
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 1,
            start_column: 0,
            end_column: 10,
            raw_content: "x = 1".to_string(),
            tokens: vec![
                Token::new(TokenKind::Identifier("x".to_string())),
                Token::new(TokenKind::Operator("=".to_string())),
                Token::new(TokenKind::Literal("1".to_string())),
            ],
            normalized_tokens: vec![],
            signature: MinHashSignature { values: vec![] },
            hash: 0,
            language: Language::Rust,
        };
        
        assert!(!engine.is_significant_fragment(&small_fragment));
        
        // Test significant fragment
        let significant_fragment = CodeFragment {
            id: 2,
            file_path: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 5,
            start_column: 0,
            end_column: 100,
            raw_content: "fn test() { let x = 1; let y = 2; return x + y; }".to_string(),
            tokens: create_test_tokens("fn test ( ) { let x = 1 ; let y = 2 ; return x + y ; }"),
            normalized_tokens: vec![],
            signature: MinHashSignature { values: vec![] },
            hash: 0,
            language: Language::Rust,
        };
        
        assert!(engine.is_significant_fragment(&significant_fragment));
    }

    #[test]
    fn test_find_representative() {
        let mut representative = HashMap::new();
        representative.insert(1, 1);
        representative.insert(2, 1);
        representative.insert(3, 2);
        
        assert_eq!(
            DuplicateDetectionEngine::find_representative(&representative, 3),
            1
        );
        assert_eq!(
            DuplicateDetectionEngine::find_representative(&representative, 1),
            1
        );
        
        // Test non-existent ID
        assert_eq!(
            DuplicateDetectionEngine::find_representative(&representative, 999),
            999
        );
    }

    #[test]
    fn test_empty_file_handling() {
        let config = DuplicateDetectionConfig::default();
        let engine = DuplicateDetectionEngine::new(config);
        
        let files = vec![
            (PathBuf::from("empty.rs"), String::new(), Language::Rust),
        ];
        
        let report = engine.detect_duplicates(&files).unwrap();
        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_fragments, 0);
    }

    #[test]
    fn test_c_and_cpp_languages() {
        let extractor = UniversalFeatureExtractor::new();
        
        // Test C code
        let c_tokens = extractor.extract_features(
            "#include <stdio.h>\nint main() { printf(\"Hello\"); return 0; }",
            Language::C
        );
        assert!(!c_tokens.is_empty());
        
        // Test C++ code
        let cpp_tokens = extractor.extract_features(
            "#include <iostream>\nint main() { std::cout << \"Hello\"; return 0; }",
            Language::Cpp
        );
        assert!(!cpp_tokens.is_empty());
    }
    
    #[test]
    fn test_language_specific_edge_cases() {
        let extractor = UniversalFeatureExtractor::new();
        
        // JavaScript template literals
        let js_code = "const msg = `Hello ${name}!`;";
        let js_tokens = extractor.extract_features(js_code, Language::JavaScript);
        assert!(!js_tokens.is_empty());
        
        // Python f-strings
        let py_code = "msg = f'Hello {name}!'";
        let py_tokens = extractor.extract_features(py_code, Language::Python);
        assert!(!py_tokens.is_empty());
        
        // Rust lifetime annotations
        let rust_code = "fn test<'a>(x: &'a str) -> &'a str { x }";
        let rust_tokens = extractor.extract_features(rust_code, Language::Rust);
        assert!(!rust_tokens.is_empty());
    }
}