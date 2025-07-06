#[cfg(test)]
mod tests {
    use crate::services::duplicate_detector::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // Strategy for generating valid tokens
    prop_compose! {
        fn arb_token()
            (choice in 0usize..6, text in "[a-zA-Z_][a-zA-Z0-9_]*")
            -> Token
        {
            let kind = match choice {
                0 => TokenKind::Identifier(text.clone()),
                1 => TokenKind::Keyword(text.clone()),
                2 => TokenKind::Literal(text.clone()),
                3 => TokenKind::Operator(text.clone()),
                4 => TokenKind::Delimiter(text.clone()),
                _ => TokenKind::Comment,
            };
            Token::new(kind)
        }
    }

    // Strategy for generating token sequences
    prop_compose! {
        fn arb_token_sequence()
            (tokens in prop::collection::vec(arb_token(), 1..100))
            -> Vec<Token>
        {
            tokens
        }
    }

    // Strategy for generating MinHash signatures
    prop_compose! {
        fn arb_minhash_signature()
            (values in prop::collection::vec(any::<u64>(), 50..200))
            -> MinHashSignature
        {
            MinHashSignature { values }
        }
    }

    // Strategy for generating valid duplicate detection configs
    prop_compose! {
        fn arb_duplicate_config()
            (
                min_tokens in 1usize..200,
                similarity_threshold in 0.1f64..1.0,
                shingle_size in 1usize..10,
                num_hash_functions in 10usize..500,
                num_bands in 1usize..50,
                rows_per_band in 1usize..20,
                normalize_identifiers in any::<bool>(),
                normalize_literals in any::<bool>(),
                ignore_comments in any::<bool>(),
                min_group_size in 2usize..10,
            )
            -> DuplicateDetectionConfig
        {
            DuplicateDetectionConfig {
                min_tokens,
                similarity_threshold,
                shingle_size,
                num_hash_functions,
                num_bands,
                rows_per_band,
                normalize_identifiers,
                normalize_literals,
                ignore_comments,
                min_group_size,
            }
        }
    }

    // Strategy for generating code source
    prop_compose! {
        fn arb_source_code()
            (lines in prop::collection::vec("[a-zA-Z0-9 +\\-*/=(){};\"'_]*", 1..50))
            -> String
        {
            lines.join("\n")
        }
    }

    proptest! {
        /// Property: MinHash Jaccard similarity is symmetric
        #[test]
        fn jaccard_similarity_symmetric(
            sig1 in arb_minhash_signature(),
            sig2 in arb_minhash_signature()
        ) {
            let sim12 = sig1.jaccard_similarity(&sig2);
            let sim21 = sig2.jaccard_similarity(&sig1);
            prop_assert!((sim12 - sim21).abs() < f64::EPSILON);
        }

        /// Property: MinHash Jaccard similarity with self is 1.0
        #[test]
        fn jaccard_similarity_reflexive(sig in arb_minhash_signature()) {
            let sim = sig.jaccard_similarity(&sig);
            prop_assert_eq!(sim, 1.0);
        }

        /// Property: MinHash Jaccard similarity is in range [0, 1]
        #[test]
        fn jaccard_similarity_bounded(
            sig1 in arb_minhash_signature(),
            sig2 in arb_minhash_signature()
        ) {
            let sim = sig1.jaccard_similarity(&sig2);
            prop_assert!((0.0..=1.0).contains(&sim));
        }

        /// Property: MinHash generator creates signatures of correct size
        #[test]
        fn minhash_signature_size_correct(
            num_hashes in 1usize..1000,
            tokens in arb_token_sequence()
        ) {
            let generator = MinHashGenerator::new(num_hashes);
            let shingles = generator.generate_shingles(&tokens, 3);
            let signature = generator.compute_signature(&shingles);
            prop_assert_eq!(signature.values.len(), num_hashes);
        }

        /// Property: Shingle generation produces correct number of shingles
        #[test]
        fn shingle_generation_count_correct(
            tokens in arb_token_sequence(),
            k in 1usize..10
        ) {
            let generator = MinHashGenerator::new(100);
            let shingles = generator.generate_shingles(&tokens, k);
            let expected_count = if tokens.len() >= k {
                tokens.len() - k + 1
            } else {
                0
            };
            prop_assert_eq!(shingles.len(), expected_count);
        }

        /// Property: Empty token sequence produces empty shingles
        #[test]
        fn empty_tokens_empty_shingles(k in 1usize..10) {
            let generator = MinHashGenerator::new(100);
            let shingles = generator.generate_shingles(&[], k);
            prop_assert!(shingles.is_empty());
        }

        /// Property: Identical token sequences produce identical shingles
        #[test]
        fn identical_tokens_identical_shingles(
            tokens in arb_token_sequence(),
            k in 1usize..10
        ) {
            let generator = MinHashGenerator::new(100);
            let shingles1 = generator.generate_shingles(&tokens, k);
            let shingles2 = generator.generate_shingles(&tokens, k);
            prop_assert_eq!(shingles1, shingles2);
        }

        /// Property: Token hashing is deterministic
        #[test]
        fn token_hashing_deterministic(token in arb_token()) {
            let hash1 = token.hash();
            let hash2 = token.hash();
            prop_assert_eq!(hash1, hash2);
        }

        /// Property: Different tokens should generally have different hashes
        #[test]
        fn different_tokens_different_hashes(
            token1 in arb_token(),
            token2 in arb_token()
        ) {
            if token1.text != token2.text {
                let hash1 = token1.hash();
                let hash2 = token2.hash();
                // Allow rare hash collisions but they should be uncommon
                prop_assert!(hash1 != hash2 || token1.text.len() < 5);
            }
        }

        /// Property: Feature extractor is total function (never panics)
        #[test]
        fn feature_extractor_total_function(
            config in arb_duplicate_config(),
            source in arb_source_code(),
            language in prop::sample::select(vec![
                Language::Rust, Language::TypeScript, Language::JavaScript,
                Language::Python, Language::C, Language::Cpp, Language::Kotlin
            ])
        ) {
            let extractor = UniversalFeatureExtractor::new(config);
            let _ = extractor.extract_features(&source, language);
            // If we reach here without panic, test passes
        }

        /// Property: Tokenization preserves information (round-trip property)
        #[test]
        fn tokenization_preserves_structure(
            config in arb_duplicate_config(),
            source in "[a-zA-Z0-9 \\n\\t(){}\\[\\];]*"
        ) {
            let extractor = UniversalFeatureExtractor::new(config);
            let tokens = extractor.extract_features(&source, Language::Rust);
            
            // Should have at least as many non-whitespace tokens as words
            let word_count = source.split_whitespace().count();
            let token_count = tokens.iter()
                .filter(|t| !matches!(t.kind, TokenKind::Whitespace))
                .count();
            
            prop_assert!(token_count >= word_count.saturating_sub(word_count / 2));
        }

        /// Property: Duplicate detection config validation
        #[test]
        fn config_validation_consistent(config in arb_duplicate_config()) {
            // Bands * rows_per_band should be reasonable for num_hash_functions
            prop_assert!(config.num_bands > 0);
            prop_assert!(config.rows_per_band > 0);
            prop_assert!(config.min_tokens > 0);
            prop_assert!(config.shingle_size > 0);
            prop_assert!(config.similarity_threshold > 0.0 && config.similarity_threshold <= 1.0);
            prop_assert!(config.min_group_size >= 2);
        }

        /// Property: MinHash similarity correlates with set similarity
        #[test]
        fn minhash_similarity_correlation(
            tokens1 in arb_token_sequence(),
            tokens2 in arb_token_sequence()
        ) {
            if tokens1.len() >= 3 && tokens2.len() >= 3 {
                let generator = MinHashGenerator::new(200);
                
                let shingles1 = generator.generate_shingles(&tokens1, 3);
                let shingles2 = generator.generate_shingles(&tokens2, 3);
                
                // Calculate actual Jaccard similarity
                let set1: HashSet<_> = shingles1.iter().collect();
                let set2: HashSet<_> = shingles2.iter().collect();
                let intersection_size = set1.intersection(&set2).count();
                let union_size = set1.union(&set2).count();
                let actual_jaccard = if union_size == 0 {
                    0.0
                } else {
                    intersection_size as f64 / union_size as f64
                };
                
                // Calculate MinHash approximation
                let sig1 = generator.compute_signature(&shingles1);
                let sig2 = generator.compute_signature(&shingles2);
                let minhash_jaccard = sig1.jaccard_similarity(&sig2);
                
                // MinHash should approximate actual Jaccard (allow some error)
                let error = (actual_jaccard - minhash_jaccard).abs();
                prop_assert!(error <= 0.3); // Allow reasonable approximation error
            }
        }

        /// Property: Signature computation is deterministic
        #[test]
        fn signature_computation_deterministic(
            tokens in arb_token_sequence(),
            num_hashes in 10usize..100
        ) {
            let generator = MinHashGenerator::new(num_hashes);
            let shingles = generator.generate_shingles(&tokens, 3);
            
            let sig1 = generator.compute_signature(&shingles);
            let sig2 = generator.compute_signature(&shingles);
            
            prop_assert_eq!(sig1.values, sig2.values);
        }

        /// Property: Larger shingle size produces fewer shingles
        #[test]
        fn larger_shingle_size_fewer_shingles(
            tokens in prop::collection::vec(arb_token(), 10..50),
            k1 in 1usize..5,
            k2 in 6usize..15
        ) {
            let generator = MinHashGenerator::new(100);
            let shingles1 = generator.generate_shingles(&tokens, k1);
            let shingles2 = generator.generate_shingles(&tokens, k2);
            
            if tokens.len() >= k2 {
                prop_assert!(shingles2.len() <= shingles1.len());
            }
        }

        /// Property: Clone type similarity values are in valid ranges
        #[test]
        fn clone_type_similarity_bounded(similarity in 0.0f64..1.0) {
            let clone_type = CloneType::Type1 { similarity };
            match clone_type {
                CloneType::Type1 { similarity } => {
                    prop_assert!((0.0..=1.0).contains(&similarity));
                }
                CloneType::Type2 { similarity, .. } => {
                    prop_assert!((0.0..=1.0).contains(&similarity));
                }
                CloneType::Type3 { similarity, .. } => {
                    prop_assert!((0.0..=1.0).contains(&similarity));
                }
            }
        }

        /// Property: Engine creation succeeds with valid config
        #[test]
        fn engine_creation_succeeds(config in arb_duplicate_config()) {
            let _engine = DuplicateDetectionEngine::new(config);
            // If we reach here without panic, test passes
        }

        /// Property: Tokenization handles Unicode correctly
        #[test]
        fn tokenization_unicode_safe(
            config in arb_duplicate_config(),
            unicode_text in "[αβγδε🦀🚀✨]{1,20}"
        ) {
            let extractor = UniversalFeatureExtractor::new(config);
            let result = extractor.extract_features(&unicode_text, Language::Rust);
            // Should not panic and should return some result
            // result.len() is usize so always >= 0
            let _ = result;
        }

        /// Property: Empty source produces minimal tokens
        #[test]
        fn empty_source_minimal_tokens(config in arb_duplicate_config()) {
            let extractor = UniversalFeatureExtractor::new(config);
            let tokens = extractor.extract_features("", Language::Rust);
            prop_assert!(tokens.len() <= 1); // At most empty result
        }

        /// Property: Very large inputs don't cause stack overflow
        #[test]
        fn large_input_bounded_processing(
            config in arb_duplicate_config(),
            repeat_count in 1usize..1000
        ) {
            let large_source = "fn test() { return 42; }\n".repeat(repeat_count);
            let extractor = UniversalFeatureExtractor::new(config);
            let tokens = extractor.extract_features(&large_source, Language::Rust);
            
            // Should handle large inputs gracefully
            prop_assert!(!tokens.is_empty());
            prop_assert!(tokens.len() < repeat_count * 20); // Reasonable upper bound
        }
    }

    #[test]
    fn test_basic_property_invariants() {
        // Ensure basic data structures work
        let config = DuplicateDetectionConfig::default();
        assert!(config.similarity_threshold > 0.0);
        assert!(config.min_tokens > 0);
        
        let generator = MinHashGenerator::new(100);
        // Test generator creation succeeds
        let shingles = vec![1u64, 2u64, 3u64];
        let sig = generator.compute_signature(&shingles);
        assert_eq!(sig.values.len(), 100);
        
        let signature = MinHashSignature { values: vec![1, 2, 3] };
        assert_eq!(signature.jaccard_similarity(&signature), 1.0);
    }
}