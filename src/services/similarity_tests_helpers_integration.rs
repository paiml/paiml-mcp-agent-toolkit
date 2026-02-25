// Internal Helper Method Tests (via SimilarityDetector)

#[test]
fn test_normalize_whitespace() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let normalized = detector.normalize_whitespace("  hello   world  \n\t  test  ");
    assert_eq!(normalized, "hello world test");
}

#[test]
fn test_normalize_whitespace_empty() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let normalized = detector.normalize_whitespace("");
    assert_eq!(normalized, "");
}

#[test]
fn test_is_keyword() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    assert!(detector.is_keyword("fn"));
    assert!(detector.is_keyword("let"));
    assert!(detector.is_keyword("mut"));
    assert!(detector.is_keyword("if"));
    assert!(detector.is_keyword("struct"));
    assert!(!detector.is_keyword("hello"));
    assert!(!detector.is_keyword("variable"));
}

#[test]
fn test_count_tokens() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    assert_eq!(detector.count_tokens("hello world test"), 3);
    assert_eq!(detector.count_tokens(""), 0);
    assert_eq!(detector.count_tokens("   "), 0);
    assert_eq!(detector.count_tokens("one"), 1);
}

#[test]
fn test_hash_content_deterministic() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let hash1 = detector.hash_content("test content");
    let hash2 = detector.hash_content("test content");
    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_content_different() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let hash1 = detector.hash_content("test content 1");
    let hash2 = detector.hash_content("test content 2");
    assert_ne!(hash1, hash2);
}

#[test]
fn test_calculate_similarity() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);

    // Identical strings
    let sim = detector.calculate_similarity("hello", "hello");
    assert!((sim - 1.0).abs() < f64::EPSILON);

    // Completely different
    let sim = detector.calculate_similarity("abc", "xyz");
    assert!(sim < 0.5);

    // Similar strings
    let sim = detector.calculate_similarity("hello", "hallo");
    assert!(sim > 0.5);
}

#[test]
fn test_normalize_identifiers() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let normalized = detector.normalize_identifiers("let myVar = 1;");
    // Keywords should remain, identifiers replaced
    assert!(normalized.contains("let"));
    // The variable name should be replaced with VAR<n>
    assert!(normalized.contains("VAR"));
}

#[test]
fn test_extract_code_blocks() {
    let config = SimilarityConfig {
        min_lines: 2,
        min_tokens: 3,
        ..SimilarityConfig::default()
    };
    let detector = SimilarityDetector::new(config);
    let content = "line1 token1 token2\nline2 token3 token4\nline3 token5 token6\n";
    let blocks = detector.extract_code_blocks(content, 2);
    // Should extract overlapping blocks
    assert!(!blocks.is_empty());
}

#[test]
fn test_extract_code_blocks_empty() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);
    let blocks = detector.extract_code_blocks("", 6);
    assert!(blocks.is_empty());
}

#[test]
fn test_extract_code_blocks_short_content() {
    let config = SimilarityConfig {
        min_lines: 10,
        min_tokens: 50,
        ..SimilarityConfig::default()
    };
    let detector = SimilarityDetector::new(config);
    let blocks = detector.extract_code_blocks("short content", 10);
    // Content too short to meet min_lines requirement
    assert!(blocks.is_empty());
}

#[test]
fn test_generate_recommendations() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);

    // Empty arrays
    let recs = detector.generate_recommendations(&[], &[]);
    assert!(recs.is_empty());

    // With high entropy blocks
    let high_entropy = vec![EntropyBlock {
        location: Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        },
        entropy: 4.5,
        category: "Complex".to_string(),
        suggestion: "Simplify".to_string(),
    }];
    let recs = detector.generate_recommendations(&high_entropy, &[]);
    assert!(!recs.is_empty());
    assert!(recs[0].contains("complex code blocks"));

    // With low entropy blocks
    let low_entropy = vec![EntropyBlock {
        location: Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        },
        entropy: 1.5,
        category: "Repetitive".to_string(),
        suggestion: "Extract".to_string(),
    }];
    let recs = detector.generate_recommendations(&[], &low_entropy);
    assert!(!recs.is_empty());
    assert!(recs[0].contains("repetitive patterns"));
}

#[test]
fn test_generate_recommendations_many_low_entropy() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);

    // More than 5 low entropy blocks triggers utility function recommendation
    let low_entropy: Vec<EntropyBlock> = (0..6)
        .map(|i| EntropyBlock {
            location: Location {
                file: PathBuf::from(format!("test{i}.rs")),
                start_line: 1,
                end_line: 10,
                start_column: None,
                end_column: None,
            },
            entropy: 1.5,
            category: "Repetitive".to_string(),
            suggestion: "Extract".to_string(),
        })
        .collect();

    let recs = detector.generate_recommendations(&[], &low_entropy);
    assert!(recs.len() >= 2);
    assert!(recs.iter().any(|r| r.contains("utility functions")));
}

#[test]
fn test_calculate_duplication_percentage() {
    let config = SimilarityConfig::default();
    let detector = SimilarityDetector::new(config);

    // Empty files
    let files: Vec<(PathBuf, String)> = vec![];
    let duplicates: Vec<SimilarBlock> = vec![];
    let pct = detector.calculate_duplication_percentage(&files, &duplicates);
    assert!((pct - 0.0).abs() < f64::EPSILON);

    // Files with no duplicates
    let files = vec![(
        PathBuf::from("test.rs"),
        "line1\nline2\nline3\n".to_string(),
    )];
    let pct = detector.calculate_duplication_percentage(&files, &[]);
    assert!((pct - 0.0).abs() < f64::EPSILON);

    // Files with some duplicates
    let duplicates = vec![SimilarBlock {
        id: "test".to_string(),
        locations: vec![
            Location {
                file: PathBuf::from("file1.rs"),
                start_line: 1,
                end_line: 5,
                start_column: None,
                end_column: None,
            },
            Location {
                file: PathBuf::from("file2.rs"),
                start_line: 1,
                end_line: 5,
                start_column: None,
                end_column: None,
            },
        ],
        similarity: 1.0,
        clone_type: CloneType::Type1,
        lines: 5,
        tokens: 20,
        content_preview: "preview".to_string(),
    }];
    let files = vec![
        (
            PathBuf::from("file1.rs"),
            "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n".to_string(),
        ),
        (
            PathBuf::from("file2.rs"),
            "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n".to_string(),
        ),
    ];
    let pct = detector.calculate_duplication_percentage(&files, &duplicates);
    // 5 lines duplicated * 2 locations = 10 lines, 20 total lines = 50%
    assert!((pct - 50.0).abs() < f64::EPSILON);
}

// Serialization Tests

#[test]
fn test_similarity_config_serialization() {
    let config = SimilarityConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SimilarityConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.min_lines, deserialized.min_lines);
}

#[test]
fn test_clone_type_serialization() {
    let ct = CloneType::Type1;
    let json = serde_json::to_string(&ct).unwrap();
    let deserialized: CloneType = serde_json::from_str(&json).unwrap();
    assert_eq!(ct, deserialized);
}

#[test]
fn test_similar_block_serialization() {
    let block = SimilarBlock {
        id: "test".to_string(),
        locations: vec![],
        similarity: 0.95,
        clone_type: CloneType::Type1,
        lines: 10,
        tokens: 50,
        content_preview: "fn test()".to_string(),
    };
    let json = serde_json::to_string(&block).unwrap();
    let deserialized: SimilarBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block.id, deserialized.id);
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
    assert_eq!(loc.start_column, deserialized.start_column);
}

#[test]
fn test_priority_serialization() {
    for priority in [Priority::High, Priority::Medium, Priority::Low] {
        let json = serde_json::to_string(&priority).unwrap();
        let deserialized: Priority = serde_json::from_str(&json).unwrap();
        // Compare by serialization since Priority doesn't implement PartialEq
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[test]
fn test_metrics_serialization() {
    let metrics = Metrics {
        duplication_percentage: 15.5,
        average_entropy: 3.2,
        total_clones: 5,
    };
    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: Metrics = serde_json::from_str(&json).unwrap();
    assert_eq!(metrics.total_clones, deserialized.total_clones);
}

// Integration-style Tests

#[test]
fn test_full_workflow_with_real_code() {
    let config = SimilarityConfig {
        min_lines: 3,
        min_tokens: 10,
        similarity_threshold: 0.7,
        enable_entropy: true,
        enable_ast: true,
        enable_semantic: true,
        window_size: 10,
        k_gram_size: 5,
    };
    let detector = SimilarityDetector::new(config);

    let rust_code1 = r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    let result = a + b;
    println!("Sum: {}", result);
    result
}
"#;

    let rust_code2 = r#"
fn calculate_sum(x: i32, y: i32) -> i32 {
    let result = x + y;
    println!("Sum: {}", result);
    result
}
"#;

    let files = vec![
        (PathBuf::from("file1.rs"), rust_code1.to_string()),
        (PathBuf::from("file2.rs"), rust_code2.to_string()),
    ];

    let report = detector.comprehensive_analysis(&files);

    // Verify report structure
    assert!(report.entropy_analysis.is_some());
    assert!(report.metrics.average_entropy >= 0.0);
    assert!(report.metrics.duplication_percentage >= 0.0);
}

#[test]
fn test_winnowing_integration() {
    let winnow = Winnowing::new(10, 5);

    let doc1 = "This is a test document with some shared content that should be detected.";
    let doc2 = "This is another document with some shared content but also unique parts.";

    let fp1 = winnow.fingerprint(doc1);
    let fp2 = winnow.fingerprint(doc2);

    let similarity = winnow.similarity(&fp1, &fp2);

    // Documents share some content, so similarity should be > 0
    assert!(similarity > 0.0);
    // But they're not identical, so similarity should be < 1
    assert!(similarity < 1.0);
}

#[test]
fn test_multiple_files_detection() {
    let config = SimilarityConfig {
        min_lines: 2,
        min_tokens: 5,
        similarity_threshold: 0.6,
        ..SimilarityConfig::default()
    };
    let detector = SimilarityDetector::new(config);

    let files = vec![
        (
            PathBuf::from("file1.rs"),
            "fn test() { let x = 1; let y = 2; }\n".repeat(5),
        ),
        (
            PathBuf::from("file2.rs"),
            "fn test() { let a = 1; let b = 2; }\n".repeat(5),
        ),
        (
            PathBuf::from("file3.rs"),
            "fn different() { println!(\"hello\"); }\n".repeat(5),
        ),
    ];

    let exact = detector.detect_exact_duplicates(&files);
    let structural = detector.detect_structural_similarity(&files, 0.6);
    let semantic = detector.detect_semantic_similarity(&files, 0.6);

    // Just verify no panics and valid output
    let _ = exact.len();
    let _ = structural.len();
    let _ = semantic.len();
}
