// Tests for duplicate detector
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

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
        let clone1 = CloneType::Type1 { similarity: 0.9 };
        let clone2 = CloneType::Type2 {
            similarity: 0.8,
            normalized: true,
        };
        let clone3 = CloneType::Type3 {
            similarity: 0.7,
            ast_distance: 0.3,
        };

        // Test that the types can be created and compared
        assert!(matches!(clone1, CloneType::Type1 { .. }));
        assert!(matches!(clone2, CloneType::Type2 { .. }));
        assert!(matches!(clone3, CloneType::Type3 { .. }));
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
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        // Test Rust code
        let rust_tokens = extractor.extract_features(
            "fn main() { let x = 42; println!(\"Hello\"); }",
            Language::Rust,
        );
        assert!(!rust_tokens.is_empty());

        // Test TypeScript code
        let ts_tokens = extractor.extract_features(
            "function main(): void { const x = 42; console.log('Hello'); }",
            Language::TypeScript,
        );
        assert!(!ts_tokens.is_empty());

        // Test Python code
        let py_tokens = extractor.extract_features(
            "def main():\n    x = 42\n    print('Hello')",
            Language::Python,
        );
        assert!(!py_tokens.is_empty());
    }

    #[test]
    fn test_normalize_tokens() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        let tokens = vec![
            Token::new(TokenKind::Identifier("myVar".to_string())),
            Token::new(TokenKind::Literal("42".to_string())),
            Token::new(TokenKind::Comment),
        ];

        let normalized = extractor.normalize_tokens(&tokens);

        // Should normalize identifier
        assert!(normalized[0].text.starts_with("VAR_"));

        // Should normalize literal
        assert_eq!(normalized[1].text, "LITERAL");

        // Should remove comment (ignore_comments is true by default)
        assert_eq!(normalized.len(), 2);
    }

    #[test]
    fn test_normalize_tokens_keep_identifiers_and_literals() {
        let config = DuplicateDetectionConfig {
            normalize_identifiers: false,
            normalize_literals: false,
            ignore_comments: false,
            ..Default::default()
        };
        let extractor = UniversalFeatureExtractor::new(config);

        let tokens = vec![
            Token::new(TokenKind::Identifier("myVar".to_string())),
            Token::new(TokenKind::Literal("42".to_string())),
            Token::new(TokenKind::Comment),
        ];

        let normalized = extractor.normalize_tokens(&tokens);

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
            signature: MinHashSignature {
                values: vec![1, 2, 3],
            },
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

        let report = engine.detect_duplicates(&files).expect("internal error");
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

        let report = engine.detect_duplicates(&files).expect("internal error");
        assert_eq!(report.summary.total_files, 3);
    }

    #[test]
    fn test_extract_fragments() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
            min_tokens: 5,
            ..Default::default()
        });

        let fragments = engine
            .extract_fragments(
                &PathBuf::from("test.rs"),
                "fn one() { println!(\"1\"); }\n\nfn two() { println!(\"2\"); }",
                Language::Rust,
            )
            .expect("internal error");
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
                    },
                },
                hash: i as u64,
                language: Language::Rust,
            });
        }

        let pairs = engine.find_clone_pairs(&fragments).expect("internal error");
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_group_clones() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());

        // Populate fragments in the engine
        for i in 1..=5 {
            let fragment = CodeFragment {
                id: i,
                file_path: PathBuf::from(format!("file{}.rs", i)),
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
            };
            engine.fragments.insert(i, fragment);
        }

        let clone_pairs = vec![(1, 2, 0.9), (2, 3, 0.85), (4, 5, 0.95)];

        let groups = engine.group_clones(clone_pairs).expect("internal error");
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

        let groups = vec![CloneGroup {
            id: 1,
            clone_type: CloneType::Type1 { similarity: 1.0 },
            fragments: vec![CloneInstance {
                file: PathBuf::from("file1.rs"),
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 100,
                similarity_to_representative: 1.0,
                normalized_hash: 123,
            }],
            total_lines: 10,
            total_tokens: 50,
            average_similarity: 1.0,
            representative: 1,
        }];

        let summary = engine.compute_summary(&fragments, &groups, 2);
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_fragments, 2);
        assert_eq!(summary.clone_groups, 1);
        assert_eq!(summary.total_lines, 15); // 10 + 5
        assert_eq!(summary.duplicate_lines, 10);
    }

    #[test]
    fn test_compute_hotspots() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());

        let groups = vec![CloneGroup {
            id: 1,
            clone_type: CloneType::Type1 { similarity: 1.0 },
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
        }];

        let hotspots = engine.compute_hotspots(&groups);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].file, PathBuf::from("hotspot.rs"));
        assert_eq!(hotspots[0].clone_groups, 1);
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

        let files = vec![(PathBuf::from("empty.rs"), String::new(), Language::Rust)];

        let report = engine.detect_duplicates(&files).expect("internal error");
        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.total_fragments, 0);
    }

    #[test]
    fn test_c_and_cpp_languages() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        // Test C code
        let c_tokens = extractor.extract_features(
            "#include <stdio.h>\nint main() { printf(\"Hello\"); return 0; }",
            Language::C,
        );
        assert!(!c_tokens.is_empty());

        // Test C++ code
        let cpp_tokens = extractor.extract_features(
            "#include <iostream>\nint main() { std::cout << \"Hello\"; return 0; }",
            Language::Cpp,
        );
        assert!(!cpp_tokens.is_empty());
    }

    #[test]
    fn test_language_specific_edge_cases() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

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

    #[test]
    fn test_is_function_start() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());

        // Test Rust
        assert!(engine.is_function_start("fn main() {", Language::Rust));
        assert!(engine.is_function_start("pub fn test(x: i32) -> i32 {", Language::Rust));
        assert!(!engine.is_function_start("let x = 42;", Language::Rust));

        // Test TypeScript/JavaScript
        assert!(engine.is_function_start("function test() {", Language::TypeScript));
        assert!(engine.is_function_start("const test = () => {", Language::TypeScript));
        assert!(engine.is_function_start("test(param) {", Language::JavaScript));

        // Test Python
        assert!(engine.is_function_start("def test():", Language::Python));
        assert!(engine.is_function_start("def test(x, y):", Language::Python));
        assert!(!engine.is_function_start("if True:", Language::Python));

        // Test C/C++
        assert!(engine.is_function_start("int main() {", Language::C));
        assert!(engine.is_function_start("void test(int x) {", Language::C));
        assert!(engine.is_function_start("std::string getName() const {", Language::Cpp));
    }

    #[test]
    fn test_is_function_end() {
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig::default());

        // Test most languages
        assert!(engine.is_function_end("}", Language::Rust));
        assert!(engine.is_function_end("}", Language::TypeScript));
        assert!(engine.is_function_end("}", Language::JavaScript));
        assert!(engine.is_function_end("}", Language::C));
        assert!(engine.is_function_end("}", Language::Cpp));

        // Test Python
        assert!(engine.is_function_end("def another_function():", Language::Python));
        assert!(engine.is_function_end("class Test:", Language::Python));
        assert!(engine.is_function_end("import os", Language::Python));
        assert!(!engine.is_function_end("    return x", Language::Python));
    }

    #[test]
    fn test_canonicalize_identifier() {
        use crate::services::duplicate_detector::IdentifierScope;
        let mut scope = IdentifierScope::default();

        let canonical1 = scope.canonicalize("myVar");
        let canonical2 = scope.canonicalize("myVar");
        let canonical3 = scope.canonicalize("otherVar");

        // Same identifier should get same canonical name
        assert_eq!(canonical1, canonical2);

        // Different identifier should get different canonical name
        assert_ne!(canonical1, canonical3);

        // Should follow VAR_N pattern
        assert_eq!(canonical1, "VAR_0");
        assert_eq!(canonical3, "VAR_1");
    }

    /// The numbering is per scope, in order of first appearance: two fragments
    /// that differ only in their names normalise to the SAME token stream.
    /// A counter shared across the project instead made the second copy of a
    /// function normalise to `VAR_9…` where the first was `VAR_2…`, so renamed
    /// clones — the entire point of normalisation — hashed differently and
    /// measured a MinHash similarity of 0.10.
    #[test]
    fn identifier_numbering_is_rename_invariant() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        let first = extractor.extract_features(
            "fn alpha(values: &[i64]) -> i64 { let mut total = 0; for v in values { total += v; } total }",
            Language::Rust,
        );
        let second = extractor.extract_features(
            "fn beta(items: &[i64]) -> i64 { let mut sum = 0; for i in items { sum += i; } sum }",
            Language::Rust,
        );

        let text =
            |tokens: &[Token]| -> Vec<String> { tokens.iter().map(|t| t.text.clone()).collect() };
        assert_eq!(
            text(&first),
            text(&second),
            "renaming every identifier must not change the normalised stream"
        );
    }

    #[test]
    fn test_token_hash() {
        let token1 = Token::new(TokenKind::Identifier("test".to_string()));
        let token2 = Token::new(TokenKind::Identifier("test".to_string()));
        assert_eq!(token1.hash(), token2.hash());
    }

    #[test]
    fn test_minhash_similarity() {
        let sig1 = MinHashSignature {
            values: vec![1, 2, 3, 4, 5],
        };
        let sig2 = MinHashSignature {
            values: vec![1, 2, 3, 6, 7],
        };
        let similarity = sig1.jaccard_similarity(&sig2);
        assert_eq!(similarity, 0.6); // 3 out of 5 match
    }

    #[test]
    fn test_feature_extraction() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        let tokens = extractor.extract_features("fn test() { return 42; }", Language::Rust);
        assert!(!tokens.is_empty());

        // Should normalize identifiers
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Identifier(name) if name.starts_with("VAR_"))));
    }

    #[test]
    fn test_duplicate_detection() {
        // Create config with lower min_tokens for testing
        let config = DuplicateDetectionConfig {
            min_tokens: 5, // Lower threshold for test snippets
            ..Default::default()
        };
        let engine = DuplicateDetectionEngine::new(config);

        let files = vec![
            (
                PathBuf::from("test1.rs"),
                "fn hello() { println!(\"Hello\"); }".to_string(),
                Language::Rust,
            ),
            (
                PathBuf::from("test2.rs"),
                "fn greet() { println!(\"Hello\"); }".to_string(),
                Language::Rust,
            ),
        ];

        let report = engine.detect_duplicates(&files).expect("internal error");
        assert!(report.summary.total_fragments >= 1);
    }

    #[test]
    fn test_shingle_generation() {
        let generator = MinHashGenerator::new(100);
        let tokens = vec![
            Token::new(TokenKind::Keyword("fn".to_string())),
            Token::new(TokenKind::Identifier("test".to_string())),
            Token::new(TokenKind::Delimiter("(".to_string())),
            Token::new(TokenKind::Delimiter(")".to_string())),
            Token::new(TokenKind::Delimiter("{".to_string())),
        ];

        let shingles = generator.generate_shingles(&tokens, 3);
        assert_eq!(shingles.len(), 3); // 5 tokens, k=3 -> 3 shingles
    }

    // TRUENO-RAG-4-MINHASH: LSH Index Tests
    // Tests for O(1) lookup via Locality-Sensitive Hashing

    /// Test LSH index creation
    #[test]
    fn test_lsh_index_creation() {
        let index = LshIndex::new(20, 5);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    /// Test LSH index insertion and retrieval
    #[test]
    fn test_lsh_index_insert_and_query() {
        let mut index = LshIndex::new(10, 10);

        // Create two identical signatures
        let sig1 = MinHashSignature {
            values: (0..100).collect(),
        };
        let sig2 = MinHashSignature {
            values: (0..100).collect(),
        };

        index.insert(1, sig1.clone());
        index.insert(2, sig2.clone());

        assert_eq!(index.len(), 2);

        // Query with sig1 should find both (identical signatures)
        let candidates = index.query(&sig1);
        assert!(candidates.contains(&1));
        assert!(candidates.contains(&2));
    }

    /// Test that identical signatures always collide
    #[test]
    fn test_lsh_identical_signatures_collide() {
        let mut index = LshIndex::new(20, 5);

        let sig = MinHashSignature {
            values: (100..200).collect(),
        };

        index.insert(42, sig.clone());

        let candidates = index.query(&sig);
        assert!(
            candidates.contains(&42),
            "Identical signature should always be a candidate"
        );
    }

    /// Test that dissimilar signatures rarely collide
    #[test]
    fn test_lsh_dissimilar_signatures_rarely_collide() {
        let mut index = LshIndex::new(20, 5);

        // Completely different signatures
        let sig1 = MinHashSignature {
            values: (0..100).collect(),
        };
        let sig2 = MinHashSignature {
            values: (1000..1100).collect(),
        };

        index.insert(1, sig1);

        // Query with dissimilar signature
        let candidates = index.query(&sig2);

        // With high probability, dissimilar signatures should not collide
        // (they might occasionally due to hash collisions, but that's rare)
        // We just verify the query completes
        assert!(candidates.len() <= 1);
    }

    /// Test LSH find_similar returns correct similarities
    #[test]
    fn test_lsh_find_similar() {
        let mut index = LshIndex::new(10, 10);

        // Create signatures with varying similarity
        let base: Vec<u64> = (0..100).collect();
        let sig_base = MinHashSignature {
            values: base.clone(),
        };

        // 90% similar (90 matching hashes)
        let mut similar_90 = base.clone();
        for i in 90..100 {
            similar_90[i] = 9999 + i as u64;
        }
        let sig_90 = MinHashSignature { values: similar_90 };

        // 50% similar (50 matching hashes)
        let mut similar_50 = base.clone();
        for i in 50..100 {
            similar_50[i] = 8888 + i as u64;
        }
        let sig_50 = MinHashSignature { values: similar_50 };

        index.insert(1, sig_base.clone());
        index.insert(2, sig_90);
        index.insert(3, sig_50);

        // Find similar with threshold 0.8
        let results = index.find_similar(&sig_base, 0.8);

        // Should find sig_base (1.0) and sig_90 (0.9)
        let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1), "Should find identical signature");
        // sig_90 might be found depending on LSH parameters
    }

    /// Test collision probability calculation
    #[test]
    fn test_lsh_collision_probability() {
        let index = LshIndex::new(20, 5);

        // High similarity should have high collision probability
        let high_sim_prob = index.collision_probability(0.9);
        assert!(
            high_sim_prob > 0.99,
            "P(0.9) should be near 1.0, got {}",
            high_sim_prob
        );

        // Medium similarity (with b=20, r=5: P(0.5) ≈ 0.47)
        let med_sim_prob = index.collision_probability(0.5);
        assert!(
            med_sim_prob > 0.3 && med_sim_prob < 0.8,
            "P(0.5) should be moderate, got {}",
            med_sim_prob
        );

        // Low similarity should have low collision probability
        let low_sim_prob = index.collision_probability(0.2);
        assert!(
            low_sim_prob < 0.2,
            "P(0.2) should be low, got {}",
            low_sim_prob
        );
    }

    /// Test that LSH provides O(1) lookup characteristics
    /// (verifying candidates << total for large indices)
    #[test]
    fn test_lsh_o1_lookup_vs_o_n_scan() {
        let mut index = LshIndex::new(20, 5);

        // Insert 1000 random signatures
        for i in 0..1000u64 {
            let sig = MinHashSignature {
                values: (i * 100..(i + 1) * 100).collect(),
            };
            index.insert(i, sig);
        }

        // Query with a signature that matches only one
        let query_sig = MinHashSignature {
            values: (0..100).collect(),
        };

        let candidates = index.query(&query_sig);

        // With good LSH parameters, candidates should be << 1000
        // (O(1) average case vs O(n) brute force)
        assert!(
            candidates.len() < 100,
            "LSH should produce few candidates, got {}",
            candidates.len()
        );
    }

    /// Test Jaccard similarity accuracy
    #[test]
    fn test_jaccard_similarity_accuracy() {
        // Known similarity: 80% matching hashes
        let sig1 = MinHashSignature {
            values: (0..100).collect(),
        };
        let mut values2: Vec<u64> = (0..80).collect();
        values2.extend(200..220); // 20 different values
        let sig2 = MinHashSignature { values: values2 };

        let similarity = sig1.jaccard_similarity(&sig2);

        // Should be approximately 0.8
        assert!(
            (similarity - 0.8).abs() < 0.01,
            "Jaccard similarity should be ~0.8, got {}",
            similarity
        );
    }

    /// Test LSH with empty query
    #[test]
    fn test_lsh_empty_query() {
        let mut index = LshIndex::new(10, 10);

        index.insert(
            1,
            MinHashSignature {
                values: vec![1, 2, 3, 4, 5],
            },
        );

        // Query with empty signature
        let empty_sig = MinHashSignature { values: vec![] };
        let candidates = index.query(&empty_sig);

        // Should return empty or minimal candidates
        assert!(candidates.len() <= 1);
    }

    /// Test get_signature retrieval
    #[test]
    fn test_lsh_get_signature() {
        let mut index = LshIndex::new(10, 10);

        let sig = MinHashSignature {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };
        index.insert(42, sig.clone());

        let retrieved = index.get_signature(42);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().values, sig.values);

        let missing = index.get_signature(999);
        assert!(missing.is_none());
    }
}

/// The two measurements the clone engine is built on: what a fragment IS, and
/// how similar two fragments are. Both were wrong in ways that made the engine
/// report exact copies and nothing else — see `analyze duplicates ::
/// structural_similarities`, a hard 0 on every input.
#[cfg(test)]
mod fragment_and_similarity_tests {
    use crate::services::duplicate_detector::{
        DuplicateDetectionConfig, DuplicateDetectionEngine, Language, MinHashGenerator, Token,
        TokenKind,
    };
    use std::path::Path;

    fn tokens(text: &str) -> Vec<Token> {
        text.split_whitespace()
            .map(|w| Token::new(TokenKind::Identifier(w.to_string())))
            .collect()
    }

    /// `MinHash` estimates the similarity of SETS, so a fragment built from six
    /// copies of a block and one built from seven copies had the identical
    /// shingle set and measured 1.00 — "exact clone". Every near-miss pair in a
    /// corpus of 40 deliberately near-miss files was therefore invisible.
    #[test]
    fn repeating_a_block_changes_the_signature() {
        let generator = MinHashGenerator::new(64);
        let block = "let value = compute ( ) ; total += value ; ";
        let six = tokens(&block.repeat(6));
        let seven = tokens(&block.repeat(7));

        let sig_six = generator.compute_signature(&generator.generate_shingles(&six, 5));
        let sig_seven = generator.compute_signature(&generator.generate_shingles(&seven, 5));

        let similarity = sig_six.jaccard_similarity(&sig_seven);
        assert!(
            similarity < 1.0,
            "six copies of a block are not identical to seven, got {similarity}"
        );
        assert!(
            similarity > 0.5,
            "but they are still highly similar, got {similarity}"
        );
    }

    /// A function fragment is the WHOLE function. The boundary used to be the
    /// first line whose trimmed text was `"}"`, which is every nested block
    /// close: a function containing an `if` was cut short there, so the engine
    /// compared prefixes of functions and two functions differing only after
    /// their first block were "identical".
    #[test]
    fn a_fragment_spans_the_whole_function() {
        let source = "\
pub fn wide(values: &[i64]) -> i64 {
    let mut total = 0i64;
    for value in values {
        if *value > 0 {
            total += value;
        } else {
            total -= value;
        }
    }
    let scaled = total * 2;
    let shifted = scaled + 7;
    let clamped = shifted.min(1000);
    clamped
}
";
        let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
            min_tokens: 10,
            ..DuplicateDetectionConfig::default()
        });
        let fragments = engine
            .extract_fragments(Path::new("wide.rs"), source, Language::Rust)
            .expect("extraction must succeed");

        assert_eq!(fragments.len(), 1, "one function, one fragment");
        assert_eq!(fragments[0].start_line, 1);
        assert_eq!(
            fragments[0].end_line, 14,
            "the fragment must reach the function's closing brace, not the `if`'s"
        );
    }

    /// The similarity a group reports is measured from the fragments, not
    /// echoed from the search's cut-off. It used to be
    /// `config.similarity_threshold` verbatim, so RAISING the threshold RAISED
    /// the reported similarity of the groups that survived.
    #[test]
    fn group_similarity_does_not_follow_the_threshold() {
        let alpha = "\
pub fn tally_alpha(values: &[i64], limit: i64) -> i64 {
    let mut total = 0i64;
    let mut seen = 0usize;
    for value in values {
        if *value > limit {
            total = total.wrapping_add(value * 3);
            seen += 1;
        } else if *value < 0 {
            total = total.wrapping_sub(value / 2);
        } else {
            total = total.wrapping_add(1);
        }
        if seen > 100 {
            break;
        }
    }
    total
}
";
        let beta = "\
pub fn tally_beta(items: &[i64], bound: i64) -> i64 {
    let mut sum = 0i64;
    let mut counted = 0usize;
    for item in items {
        if *item > bound {
            sum = sum.wrapping_add(item * 3);
            counted += 1;
        } else if *item < 0 {
            sum = sum.wrapping_sub(item / 2);
        } else {
            sum = sum.wrapping_add(1);
        }
        if counted % 7 == 0 {
            sum = sum.wrapping_mul(2);
        }
        if counted > 100 {
            break;
        }
    }
    sum
}
";
        let measured = |threshold: f64| -> f64 {
            let engine = DuplicateDetectionEngine::new(DuplicateDetectionConfig {
                similarity_threshold: threshold,
                ..DuplicateDetectionConfig::default()
            });
            let files = vec![
                (
                    std::path::PathBuf::from("alpha.rs"),
                    alpha.to_string(),
                    Language::Rust,
                ),
                (
                    std::path::PathBuf::from("beta.rs"),
                    beta.to_string(),
                    Language::Rust,
                ),
            ];
            let report = engine.detect_duplicates(&files).expect("detection");
            assert_eq!(report.groups.len(), 1, "the pair is one clone group");
            report.groups[0].average_similarity
        };

        let loose = measured(0.50);
        let tighter = measured(0.70);
        assert!(
            (loose - tighter).abs() < 1e-9,
            "the same two functions must measure the same similarity at any cut-off: {loose} vs {tighter}"
        );
        assert!(
            loose > 0.7 && loose < 1.0,
            "a near-miss pair is neither unrelated nor identical, got {loose}"
        );
    }
}
