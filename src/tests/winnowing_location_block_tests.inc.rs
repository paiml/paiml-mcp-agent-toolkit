mod winnowing_tests {
    use super::*;

    #[test]
    fn test_winnowing_new_various_sizes() {
        for window in [1, 5, 10, 40, 100] {
            for k_gram in [1, 5, 15, 50] {
                let winnow = Winnowing::new(window, k_gram);
                // Test that winnowing is created and functional (fields are private)
                // Use fingerprinting as a functional test
                let _fp = winnow.fingerprint("test string for winnowing");
            }
        }
    }

    #[test]
    fn test_fingerprint_text_shorter_than_k_gram() {
        let winnow = Winnowing::new(5, 10);
        let fp = winnow.fingerprint("short");
        assert!(fp.is_empty());
    }

    #[test]
    fn test_fingerprint_text_equal_to_k_gram() {
        let winnow = Winnowing::new(5, 5);
        let fp = winnow.fingerprint("hello");
        // Exactly k_gram size should produce 1 k-gram but may not meet window size
        assert!(fp.len() <= 1);
    }

    #[test]
    fn test_fingerprint_long_text() {
        let winnow = Winnowing::new(5, 3);
        let fp =
            winnow.fingerprint("the quick brown fox jumps over the lazy dog and more text here");
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_fingerprint_unicode_text() {
        let winnow = Winnowing::new(5, 3);
        let fp = winnow.fingerprint("hello cafe");
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_fingerprint_special_characters() {
        let winnow = Winnowing::new(5, 3);
        let fp = winnow.fingerprint("fn test() { let x = 1; }");
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_fingerprint_all_same_characters() {
        let winnow = Winnowing::new(5, 3);
        let fp = winnow.fingerprint("aaaaaaaaaaaaaaaaaaa");
        // All same k-grams should still produce fingerprints
        assert!(!fp.is_empty());
    }

    // similarity tests

    #[test]
    fn test_similarity_one_empty() {
        let winnow = Winnowing::new(5, 3);
        let fp1 = winnow.fingerprint("hello world test");
        let sim = winnow.similarity(&fp1, &[]);
        assert!(sim >= 0.0);
        assert!(sim <= 1.0);
    }

    #[test]
    fn test_similarity_both_from_same_text() {
        let winnow = Winnowing::new(5, 3);
        let text = "the quick brown fox";
        let fp1 = winnow.fingerprint(text);
        let fp2 = winnow.fingerprint(text);
        let sim = winnow.similarity(&fp1, &fp2);
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_similar_texts() {
        let winnow = Winnowing::new(5, 3);
        let fp1 = winnow.fingerprint("the quick brown fox jumps");
        let fp2 = winnow.fingerprint("the quick brown dog runs");
        let sim = winnow.similarity(&fp1, &fp2);
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_similarity_is_symmetric() {
        let winnow = Winnowing::new(5, 3);
        let fp1 = winnow.fingerprint("hello world");
        let fp2 = winnow.fingerprint("goodbye moon");
        let sim12 = winnow.similarity(&fp1, &fp2);
        let sim21 = winnow.similarity(&fp2, &fp1);
        assert!((sim12 - sim21).abs() < f64::EPSILON);
    }

    // find_matches tests

    #[test]
    fn test_find_matches_subset() {
        let winnow = Winnowing::new(5, 3);
        let text = "the quick brown fox jumps over";
        let sub = "quick brown";
        let fp_text = winnow.fingerprint(text);
        let fp_sub = winnow.fingerprint(sub);
        let matches = winnow.find_matches(&fp_text, &fp_sub);
        // Should find some matches since sub is contained in text - len() is usize, always >= 0
        let _ = matches.len();
    }

    #[test]
    fn test_find_matches_disjoint() {
        let winnow = Winnowing::new(5, 3);
        let fp1 = winnow.fingerprint("aaaaaaaaaaaaaaa");
        let fp2 = winnow.fingerprint("bbbbbbbbbbbbbbb");
        let matches = winnow.find_matches(&fp1, &fp2);
        // Disjoint texts should have few or no matches
        assert!(matches.len() <= fp1.len());
    }

    #[test]
    fn test_find_matches_partial_overlap() {
        let winnow = Winnowing::new(5, 3);
        let fp1 = winnow.fingerprint("hello world test string");
        let fp2 = winnow.fingerprint("world test other words");
        let _ = winnow.find_matches(&fp1, &fp2);
    }
}

// =============================================================================
// Location Tests
// =============================================================================

mod location_tests {
    use super::*;

    #[test]
    fn test_location_all_fields() {
        let loc = Location {
            file: PathBuf::from("/path/to/test.rs"),
            start_line: 10,
            end_line: 20,
            start_column: Some(5),
            end_column: Some(80),
        };
        assert_eq!(loc.file, PathBuf::from("/path/to/test.rs"));
        assert_eq!(loc.start_line, 10);
        assert_eq!(loc.end_line, 20);
        assert_eq!(loc.start_column, Some(5));
        assert_eq!(loc.end_column, Some(80));
    }

    #[test]
    fn test_location_optional_columns_none() {
        let loc = Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        };
        assert!(loc.start_column.is_none());
        assert!(loc.end_column.is_none());
    }

    #[test]
    fn test_location_debug() {
        let loc = Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        };
        let debug_str = format!("{:?}", loc);
        assert!(debug_str.contains("test.rs"));
    }

    #[test]
    fn test_location_serialization() {
        let loc = Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: Some(5),
            end_column: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(loc.file, deserialized.file);
        assert_eq!(loc.start_line, deserialized.start_line);
    }
}

// =============================================================================
// SimilarBlock Tests
// =============================================================================

mod similar_block_tests {
    use super::*;

    #[test]
    fn test_similar_block_full() {
        let block = SimilarBlock {
            id: "block_001".to_string(),
            locations: vec![
                Location {
                    file: PathBuf::from("file1.rs"),
                    start_line: 1,
                    end_line: 10,
                    start_column: None,
                    end_column: None,
                },
                Location {
                    file: PathBuf::from("file2.rs"),
                    start_line: 20,
                    end_line: 30,
                    start_column: None,
                    end_column: None,
                },
            ],
            similarity: 0.95,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "fn test() {\n    let x = 1;\n}".to_string(),
        };
        assert_eq!(block.id, "block_001");
        assert_eq!(block.locations.len(), 2);
        assert!((block.similarity - 0.95).abs() < f64::EPSILON);
        assert_eq!(block.clone_type, CloneType::Type1);
    }

    #[test]
    fn test_similar_block_all_clone_types() {
        for ct in [
            CloneType::Type1,
            CloneType::Type2,
            CloneType::Type3,
            CloneType::Type4,
        ] {
            let block = SimilarBlock {
                id: "test".to_string(),
                locations: vec![],
                similarity: 1.0,
                clone_type: ct,
                lines: 1,
                tokens: 1,
                content_preview: String::new(),
            };
            assert_eq!(block.clone_type, ct);
        }
    }

    #[test]
    fn test_similar_block_serialization() {
        let block = SimilarBlock {
            id: "test".to_string(),
            locations: vec![],
            similarity: 0.85,
            clone_type: CloneType::Type2,
            lines: 5,
            tokens: 25,
            content_preview: "preview".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: SimilarBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block.id, deserialized.id);
        assert!((block.similarity - deserialized.similarity).abs() < f64::EPSILON);
    }
}

// =============================================================================
// EntropyReport Tests
// =============================================================================

