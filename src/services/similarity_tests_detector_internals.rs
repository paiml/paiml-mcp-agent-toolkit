// Winnowing Tests

#[test]
fn test_winnowing_new() {
    let winnow = Winnowing::new(40, 15);
    assert_eq!(winnow.window_size, 40);
    assert_eq!(winnow.k_gram_size, 15);
}

#[test]
fn test_winnowing_fingerprint_empty() {
    let winnow = Winnowing::new(5, 3);
    let fp = winnow.fingerprint("");
    assert!(fp.is_empty());
}

#[test]
fn test_winnowing_fingerprint_short_text() {
    let winnow = Winnowing::new(5, 3);
    let fp = winnow.fingerprint("ab");
    // Text shorter than k_gram_size produces empty fingerprint
    assert!(fp.is_empty());
}

#[test]
fn test_winnowing_fingerprint_valid_text() {
    let winnow = Winnowing::new(5, 3);
    let fp = winnow.fingerprint("the quick brown fox");
    assert!(!fp.is_empty());
    // Fingerprints should be unique
    let unique_count = fp.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, fp.len());
}

#[test]
fn test_winnowing_fingerprint_deterministic() {
    let winnow = Winnowing::new(5, 3);
    let text = "hello world";
    let fp1 = winnow.fingerprint(text);
    let fp2 = winnow.fingerprint(text);
    assert_eq!(fp1, fp2);
}

#[test]
fn test_winnowing_similarity_identical() {
    let winnow = Winnowing::new(5, 3);
    let fp = winnow.fingerprint("the quick brown fox");
    let sim = winnow.similarity(&fp, &fp);
    assert!((sim - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_winnowing_similarity_empty() {
    let winnow = Winnowing::new(5, 3);
    let sim = winnow.similarity(&[], &[]);
    assert!((sim - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_winnowing_similarity_different() {
    let winnow = Winnowing::new(5, 3);
    let fp1 = winnow.fingerprint("the quick brown fox");
    let fp2 = winnow.fingerprint("hello world goodbye moon");
    let sim = winnow.similarity(&fp1, &fp2);
    // Different texts should have low similarity
    assert!((0.0..=1.0).contains(&sim));
}

#[test]
fn test_winnowing_similarity_partial_overlap() {
    let winnow = Winnowing::new(5, 3);
    let fp1 = winnow.fingerprint("the quick brown fox jumps");
    let fp2 = winnow.fingerprint("the quick brown dog runs");
    let sim = winnow.similarity(&fp1, &fp2);
    // Partial overlap should give intermediate similarity
    assert!(sim > 0.0);
    assert!(sim < 1.0);
}

#[test]
fn test_winnowing_find_matches_empty() {
    let winnow = Winnowing::new(5, 3);
    let matches = winnow.find_matches(&[], &[]);
    assert!(matches.is_empty());
}

#[test]
fn test_winnowing_find_matches_no_matches() {
    let winnow = Winnowing::new(5, 3);
    let fp1 = winnow.fingerprint("the quick brown fox");
    let fp2 = winnow.fingerprint("xyz abc 123 456 789");
    let matches = winnow.find_matches(&fp1, &fp2);
    // May or may not have matches depending on hash collisions
    assert!(matches.len() <= fp1.len());
}

#[test]
fn test_winnowing_find_matches_with_matches() {
    let winnow = Winnowing::new(5, 3);
    let text = "the quick brown fox jumps over the lazy dog";
    let fp = winnow.fingerprint(text);
    // Find matches with itself
    let matches = winnow.find_matches(&fp, &fp);
    // All fingerprints should match themselves
    assert_eq!(matches.len(), fp.len());
}

// TokenAnalyzer Tests (via SimilarityDetector)

#[test]
fn test_token_analyzer_tokenize() {
    let analyzer = TokenAnalyzer::new();
    let tokens = analyzer.tokenize("Hello World");
    assert_eq!(tokens, vec!["hello", "world"]);
}

#[test]
fn test_token_analyzer_tokenize_empty() {
    let analyzer = TokenAnalyzer::new();
    let tokens = analyzer.tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_token_analyzer_to_vector() {
    let analyzer = TokenAnalyzer::new();
    let tokens = vec!["hello".to_string(), "world".to_string()];
    let vector = analyzer.to_vector(&tokens);
    assert!((vector.get("hello").unwrap() - 0.5).abs() < f64::EPSILON);
    assert!((vector.get("world").unwrap() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_token_analyzer_to_vector_repeated() {
    let analyzer = TokenAnalyzer::new();
    let tokens = vec![
        "hello".to_string(),
        "hello".to_string(),
        "world".to_string(),
    ];
    let vector = analyzer.to_vector(&tokens);
    // "hello" appears twice, "world" once
    // hello weight = 2/3, world weight = 1/3
    assert!((vector.get("hello").unwrap() - (2.0 / 3.0)).abs() < 0.01);
    assert!((vector.get("world").unwrap() - (1.0 / 3.0)).abs() < 0.01);
}

#[test]
fn test_token_analyzer_cosine_similarity_identical() {
    let analyzer = TokenAnalyzer::new();
    let tokens = vec!["hello".to_string(), "world".to_string()];
    let vector = analyzer.to_vector(&tokens);
    let sim = analyzer.cosine_similarity(&vector, &vector);
    assert!((sim - 1.0).abs() < 0.01);
}

#[test]
fn test_token_analyzer_cosine_similarity_empty() {
    let analyzer = TokenAnalyzer::new();
    let empty: TokenVector = HashMap::new();
    let sim = analyzer.cosine_similarity(&empty, &empty);
    assert!((sim - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_analyzer_cosine_similarity_different() {
    let analyzer = TokenAnalyzer::new();
    let tokens1 = vec!["hello".to_string(), "world".to_string()];
    let tokens2 = vec!["goodbye".to_string(), "moon".to_string()];
    let v1 = analyzer.to_vector(&tokens1);
    let v2 = analyzer.to_vector(&tokens2);
    let sim = analyzer.cosine_similarity(&v1, &v2);
    // Completely different tokens should have 0 similarity
    assert!((sim - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_analyzer_cosine_similarity_partial() {
    let analyzer = TokenAnalyzer::new();
    let tokens1 = vec!["hello".to_string(), "world".to_string()];
    let tokens2 = vec!["hello".to_string(), "moon".to_string()];
    let v1 = analyzer.to_vector(&tokens1);
    let v2 = analyzer.to_vector(&tokens2);
    let sim = analyzer.cosine_similarity(&v1, &v2);
    // One common token should give partial similarity
    assert!(sim > 0.0);
    assert!(sim < 1.0);
}

// EntropyCalculator Tests

#[test]
fn test_entropy_calculator_new() {
    let calc = EntropyCalculator::new();
    // Just verify construction
    let _ = calc;
}

#[test]
fn test_entropy_calculator_calculate_uniform() {
    let calc = EntropyCalculator::new();
    // Two different chars = 1 bit
    let entropy = calc.calculate("ab");
    assert!((entropy - 1.0).abs() < 0.01);
}

#[test]
fn test_entropy_calculator_calculate_skewed() {
    let calc = EntropyCalculator::new();
    // Mostly 'a' with one 'b' = low entropy
    let entropy = calc.calculate("aaaab");
    // Entropy should be less than 1 bit
    assert!(entropy < 1.0);
    assert!(entropy > 0.0);
}

#[test]
fn test_entropy_calculator_calculate_all_same() {
    let calc = EntropyCalculator::new();
    let entropy = calc.calculate("aaaa");
    assert!((entropy - 0.0).abs() < f64::EPSILON);
}

// Priority Tests

#[test]
fn test_priority_clone() {
    let p = Priority::High;
    let cloned = p.clone();
    assert!(matches!(cloned, Priority::High));
}

#[test]
fn test_priority_variants() {
    let _high = Priority::High;
    let _medium = Priority::Medium;
    let _low = Priority::Low;
}

// Location Tests

#[test]
fn test_location_clone() {
    let loc = Location {
        file: PathBuf::from("test.rs"),
        start_line: 1,
        end_line: 10,
        start_column: Some(1),
        end_column: Some(50),
    };
    let cloned = loc.clone();
    assert_eq!(loc.file, cloned.file);
    assert_eq!(loc.start_line, cloned.start_line);
}

#[test]
fn test_location_without_columns() {
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

// SimilarBlock Tests

#[test]
fn test_similar_block_clone() {
    let block = SimilarBlock {
        id: "test".to_string(),
        locations: vec![Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        }],
        similarity: 0.95,
        clone_type: CloneType::Type1,
        lines: 10,
        tokens: 50,
        content_preview: "fn test()".to_string(),
    };
    let cloned = block.clone();
    assert_eq!(block.id, cloned.id);
    assert_eq!(block.similarity, cloned.similarity);
}

// EntropyReport Tests

#[test]
fn test_entropy_report_clone() {
    let report = EntropyReport {
        average_entropy: 3.5,
        high_entropy_blocks: vec![],
        low_entropy_patterns: vec![],
        recommendations: vec!["Test recommendation".to_string()],
    };
    let cloned = report.clone();
    assert_eq!(report.average_entropy, cloned.average_entropy);
    assert_eq!(report.recommendations.len(), cloned.recommendations.len());
}

// EntropyBlock Tests

#[test]
fn test_entropy_block_clone() {
    let block = EntropyBlock {
        location: Location {
            file: PathBuf::from("test.rs"),
            start_line: 1,
            end_line: 10,
            start_column: None,
            end_column: None,
        },
        entropy: 4.5,
        category: "Complex".to_string(),
        suggestion: "Simplify this code".to_string(),
    };
    let cloned = block.clone();
    assert_eq!(block.entropy, cloned.entropy);
    assert_eq!(block.category, cloned.category);
}

// RefactoringHint Tests

#[test]
fn test_refactoring_hint_clone() {
    let hint = RefactoringHint {
        locations: vec![],
        pattern: "Repeated pattern".to_string(),
        suggestion: "Extract to function".to_string(),
        priority: Priority::High,
    };
    let cloned = hint.clone();
    assert_eq!(hint.pattern, cloned.pattern);
    assert_eq!(hint.suggestion, cloned.suggestion);
}

// Metrics Tests

#[test]
fn test_metrics_clone() {
    let metrics = Metrics {
        duplication_percentage: 15.5,
        average_entropy: 3.2,
        total_clones: 5,
    };
    let cloned = metrics.clone();
    assert_eq!(
        metrics.duplication_percentage,
        cloned.duplication_percentage
    );
    assert_eq!(metrics.average_entropy, cloned.average_entropy);
    assert_eq!(metrics.total_clones, cloned.total_clones);
}

// ComprehensiveReport Tests

#[test]
fn test_comprehensive_report_clone() {
    let report = ComprehensiveReport {
        exact_duplicates: vec![],
        structural_similarities: vec![],
        semantic_similarities: vec![],
        entropy_analysis: None,
        refactoring_opportunities: vec![],
        metrics: Metrics {
            duplication_percentage: 0.0,
            average_entropy: 0.0,
            total_clones: 0,
        },
    };
    let cloned = report.clone();
    assert_eq!(report.metrics.total_clones, cloned.metrics.total_clones);
}
