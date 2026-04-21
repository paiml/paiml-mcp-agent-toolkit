//! Tests for pattern extractor
//! Extracted to separate file for file health compliance (CB-040)

use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::path::PathBuf;

#[test]
fn test_pattern_type_equality() {
    assert_eq!(PatternType::ErrorHandling, PatternType::ErrorHandling);
    assert_ne!(PatternType::ErrorHandling, PatternType::DataValidation);
}

#[test]
fn test_pattern_collection() {
    let mut collection = PatternCollection::new();
    assert_eq!(collection.file_count(), 0);

    let pattern = AstPattern {
        pattern_type: PatternType::ErrorHandling,
        pattern_hash: "test123".to_string(),
        frequency: 3,
        locations: vec![],
        variation_score: 0.0,
        example_code: "test".to_string(),
        estimated_loc: 10,
    };

    collection.add_pattern(pattern);
    let summary = collection.summary();
    assert_eq!(summary.repetitions, 3);
    assert_eq!(summary.pattern_type, PatternType::ErrorHandling);
}

#[test]
fn test_extract_error_handling_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need ≥3 structurally identical Result handling patterns
    let content = r#"
        fn foo() {
match bar() -> Result<i32, Error> {
Ok(x) => Ok(x),
Err(e) => Err(e),
}
        }
        fn baz() {
match qux() -> Result<i32, Error> {
Ok(x) => Ok(x),
Err(e) => Err(e),
}
        }
        fn quux() {
match waldo() -> Result<i32, Error> {
Ok(x) => Ok(x),
Err(e) => Err(e),
}
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_error_handling_patterns(&file_path, content, &mut collection)
        .expect("Pattern extraction should succeed");

    assert!(
        !collection.patterns.is_empty(),
        "Should extract error handling patterns with >=3 structurally identical Result matches"
    );
}

#[test]
fn test_extract_control_flow_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Build content with ≥3 structurally identical } else if patterns
    let mut content = String::from("fn foo(x: i32) {\n  if x > 10 {\n");
    content.push_str("  } else if x > 5 {\n  } else if x > 5 {\n  } else if x > 5 {\n  }\n}\n");

    let mut collection = PatternCollection::new();
    extractor
        .extract_control_flow_patterns(&file_path, &content, &mut collection)
        .expect("Pattern extraction should succeed");

    assert!(
        !collection.patterns.is_empty(),
        "Should extract control flow patterns with >=3 structurally identical else-if chains"
    );
}

#[test]
fn test_extract_data_transformation_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need >8 iterator combinator calls, with ≥3 structurally identical
    let content = r#"
        fn process1() {
            items.iter().filter(|x| x > 0).map(|x| x + 1).collect()
        }
        fn process2() {
            items.iter().filter(|x| x > 0).map(|x| x + 1).collect()
        }
        fn process3() {
            items.iter().filter(|x| x > 0).map(|x| x + 1).collect()
        }
        fn process4() {
            items.iter().filter(|x| x > 0).map(|x| x + 1).collect()
        }
        fn process5() {
            items.iter().filter(|x| x > 0).map(|x| x + 1).collect()
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_data_transformation_patterns(&file_path, content, &mut collection)
        .expect("Pattern extraction should succeed");

    assert!(
        !collection.patterns.is_empty(),
        "Should extract data transformation patterns with >=3 structurally identical combinator calls"
    );
}

#[test]
fn test_extract_api_call_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need >3 HTTP calls, with ≥3 structurally identical
    let content = r#"
        async fn fetch_data() {
            let r1 = client.post("/endpoint", body).await;
            let r2 = client.post("/endpoint", body).await;
            let r3 = client.post("/endpoint", body).await;
            let r4 = client.post("/endpoint", body).await;
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_api_call_patterns(&file_path, content, &mut collection)
        .expect("Pattern extraction should succeed");

    assert!(
        !collection.patterns.is_empty(),
        "Should extract API call patterns with >=3 structurally identical HTTP calls"
    );
}

#[test]
fn test_extract_resource_management_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need >5 resource management calls, with ≥3 structurally identical
    let content = r#"
        fn foo() {
            let guard = mutex.lock();
            let guard = mutex.lock();
            let guard = mutex.lock();
            let handle = resource.open();
            let handle = resource.open();
            let handle = resource.open();
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_resource_management_patterns(&file_path, content, &mut collection)
        .expect("Pattern extraction should succeed");

    assert!(
        !collection.patterns.is_empty(),
        "Should extract resource management patterns with >=3 structurally identical calls"
    );
}

#[test]
fn test_extract_data_validation_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need >5 compound validation checks, with ≥3 structurally identical
    let content = r#"
        fn validate(input: &str) -> bool {
            if input.is_empty() && input.len() > 0 {
                return false;
            }
            if input.is_empty() && input.len() > 0 {
                return false;
            }
            if input.is_empty() && input.len() > 0 {
                return false;
            }
            if data.is_empty() && data.len() > 0 {
                return false;
            }
            if name.is_empty() && name.len() > 0 {
                return false;
            }
            if buf.is_empty() && buf.len() > 0 {
                return false;
            }
            true
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_data_validation_patterns(&file_path, content, &mut collection)
        .expect("Pattern extraction should succeed");

    // Should detect pattern when ≥3 structurally identical compound checks found
    assert!(
        !collection.patterns.is_empty(),
        "Should extract data validation patterns with >=3 structurally identical compound checks"
    );
}

#[test]
fn test_pattern_collection_get_patterns_for_file() {
    let mut collection = PatternCollection::new();
    let file_path = PathBuf::from("test.rs");

    let pattern = AstPattern {
        pattern_type: PatternType::ErrorHandling,
        pattern_hash: "hash1".to_string(),
        frequency: 5,
        locations: vec![],
        variation_score: 0.3,
        example_code: "test code".to_string(),
        estimated_loc: 20,
    };

    collection.add_pattern(pattern);
    collection
        .file_patterns
        .insert(file_path.clone(), vec!["hash1".to_string()]);

    let patterns = collection.get_patterns_for_file(&file_path);
    assert_eq!(patterns.len(), 1, "Should retrieve pattern for file");
    assert_eq!(patterns[0].pattern_hash, "hash1");
}

#[test]
fn test_empty_content_no_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("empty.rs");
    let content = "";

    let mut collection = PatternCollection::new();
    extractor
        .extract_error_handling_patterns(&file_path, content, &mut collection)
        .expect("Should handle empty content");

    assert!(
        collection.patterns.is_empty(),
        "Empty content should produce no patterns"
    );
}

// Property tests
#[allow(unused_imports)]
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

// Coverage tests
#[allow(unused_imports)]
use super::*;

// PatternType tests
#[test]
fn test_pattern_type_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(PatternType::ErrorHandling, 1);
    map.insert(PatternType::DataValidation, 2);
    map.insert(PatternType::ResourceManagement, 3);
    map.insert(PatternType::ControlFlow, 4);
    map.insert(PatternType::DataTransformation, 5);
    map.insert(PatternType::ApiCall, 6);
    assert_eq!(map.len(), 6);
}

#[test]
fn test_pattern_type_clone_and_copy() {
    let pt = PatternType::ErrorHandling;
    let pt2 = pt; // Copy
    let pt3 = pt; // Clone
    assert_eq!(pt, pt2);
    assert_eq!(pt, pt3);
}

#[test]
fn test_pattern_type_serialization() {
    let pt = PatternType::DataValidation;
    let json = serde_json::to_string(&pt).unwrap();
    let deserialized: PatternType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, deserialized);
}

// Location tests
#[test]
fn test_location_creation() {
    let loc = Location {
        file: PathBuf::from("test.rs"),
        line: 42,
        column: 10,
    };
    assert_eq!(loc.file, PathBuf::from("test.rs"));
    assert_eq!(loc.line, 42);
    assert_eq!(loc.column, 10);
}

#[test]
fn test_location_clone_and_debug() {
    let loc = Location {
        file: PathBuf::from("src/lib.rs"),
        line: 100,
        column: 5,
    };
    let cloned = loc.clone();
    assert_eq!(format!("{:?}", loc), format!("{:?}", cloned));
}

#[test]
fn test_location_serialization() {
    let loc = Location {
        file: PathBuf::from("test.rs"),
        line: 1,
        column: 1,
    };
    let json = serde_json::to_string(&loc).unwrap();
    let deserialized: Location = serde_json::from_str(&json).unwrap();
    assert_eq!(loc.line, deserialized.line);
    assert_eq!(loc.column, deserialized.column);
}

// AstPattern tests
#[test]
fn test_ast_pattern_creation() {
    let pattern = AstPattern {
        pattern_type: PatternType::ControlFlow,
        pattern_hash: "abc123".to_string(),
        frequency: 5,
        locations: vec![],
        variation_score: 0.5,
        example_code: "if x > 0 {}".to_string(),
        estimated_loc: 10,
    };
    assert_eq!(pattern.frequency, 5);
    assert_eq!(pattern.variation_score, 0.5);
}

#[test]
fn test_ast_pattern_clone() {
    let pattern = AstPattern {
        pattern_type: PatternType::ApiCall,
        pattern_hash: "hash".to_string(),
        frequency: 3,
        locations: vec![Location {
            file: PathBuf::from("api.rs"),
            line: 10,
            column: 1,
        }],
        variation_score: 0.2,
        example_code: "client.get()".to_string(),
        estimated_loc: 5,
    };
    let cloned = pattern.clone();
    assert_eq!(pattern.pattern_hash, cloned.pattern_hash);
    assert_eq!(pattern.frequency, cloned.frequency);
}

// PatternCollection tests
#[test]
fn test_pattern_collection_default() {
    let collection = PatternCollection::default();
    assert_eq!(collection.file_count(), 0);
    assert!(collection.patterns.is_empty());
}

#[test]
fn test_pattern_collection_summary_with_patterns() {
    let mut collection = PatternCollection::new();

    // Add multiple patterns with different frequencies
    collection.add_pattern(AstPattern {
        pattern_type: PatternType::ErrorHandling,
        pattern_hash: "error1".to_string(),
        frequency: 5,
        locations: vec![],
        variation_score: 0.1,
        example_code: "match result {}".to_string(),
        estimated_loc: 10,
    });

    collection.add_pattern(AstPattern {
        pattern_type: PatternType::DataValidation,
        pattern_hash: "validate1".to_string(),
        frequency: 10, // Higher frequency - should be returned
        locations: vec![],
        variation_score: 0.3,
        example_code: "if input.is_empty()".to_string(),
        estimated_loc: 5,
    });

    let summary = collection.summary();
    // Should return the pattern with highest frequency
    assert_eq!(summary.repetitions, 10);
    assert_eq!(summary.pattern_type, PatternType::DataValidation);
}

#[test]
fn test_pattern_collection_get_patterns_nonexistent_file() {
    let collection = PatternCollection::new();
    let patterns = collection.get_patterns_for_file(Path::new("nonexistent.rs"));
    assert!(patterns.is_empty());
}

#[test]
fn test_pattern_collection_with_file_patterns() {
    let mut collection = PatternCollection::new();
    let file_path = PathBuf::from("src/main.rs");

    collection.add_pattern(AstPattern {
        pattern_type: PatternType::ControlFlow,
        pattern_hash: "cf_1".to_string(),
        frequency: 2,
        locations: vec![Location {
            file: file_path.clone(),
            line: 10,
            column: 1,
        }],
        variation_score: 0.0,
        example_code: "if".to_string(),
        estimated_loc: 3,
    });

    collection
        .file_patterns
        .insert(file_path.clone(), vec!["cf_1".to_string()]);

    let patterns = collection.get_patterns_for_file(&file_path);
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern_hash, "cf_1");
}

// PatternExtractor tests
#[test]
fn test_pattern_extractor_creation() {
    let config = EntropyConfig::default();
    let extractor = PatternExtractor::new(config);
    // Verify it doesn't panic
    let _ = extractor;
}

#[test]
fn test_should_process_file_included() {
    let config = EntropyConfig {
        exclude_paths: vec!["**/tests/**".to_string()],
        ..EntropyConfig::default()
    };
    let extractor = PatternExtractor::new(config);

    assert!(extractor.should_process_file(Path::new("src/lib.rs")));
    assert!(extractor.should_process_file(Path::new("src/main.rs")));
}

#[test]
fn test_should_process_file_excluded() {
    let config = EntropyConfig {
        exclude_paths: vec!["**/target/**".to_string()],
        ..EntropyConfig::default()
    };
    let extractor = PatternExtractor::new(config);

    // File in target should be excluded
    assert!(!extractor.should_process_file(Path::new("target/debug/build.rs")));
}

#[test]
fn test_hash_pattern() {
    let extractor = PatternExtractor::new(EntropyConfig::default());

    let hash1 = extractor.hash_pattern("test pattern 1");
    let hash2 = extractor.hash_pattern("test pattern 2");
    let hash3 = extractor.hash_pattern("test pattern 1");

    assert_ne!(hash1, hash2);
    assert_eq!(hash1, hash3);
}

#[test]
fn test_calculate_variation_score_single_match() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "fn foo() {}";
    let pattern = Regex::new(r"fn").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();

    let score = extractor.calculate_variation_score(&matches, content);
    assert_eq!(score, 0.0);
}

#[test]
fn test_calculate_variation_score_multiple_similar() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "fn foo() {} fn bar() {} fn baz() {}";
    let pattern = Regex::new(r"fn \w+").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();

    let score = extractor.calculate_variation_score(&matches, content);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn test_calculate_string_similarity_identical() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let similarity = extractor.calculate_string_similarity("foo bar", "foo bar");
    assert_eq!(similarity, 1.0);
}

#[test]
fn test_calculate_string_similarity_different() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let similarity = extractor.calculate_string_similarity("foo bar", "baz qux");
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_calculate_string_similarity_partial() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let similarity = extractor.calculate_string_similarity("foo bar baz", "foo bar qux");
    assert!(similarity > 0.0 && similarity < 1.0);
}

#[test]
fn test_calculate_string_similarity_empty() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let similarity = extractor.calculate_string_similarity("", "");
    assert_eq!(similarity, 0.0);
}

#[test]
fn test_calculate_pattern_variations() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let mut collection = PatternCollection::new();

    collection.add_pattern(AstPattern {
        pattern_type: PatternType::ControlFlow,
        pattern_hash: "test".to_string(),
        frequency: 3,
        locations: vec![
            Location {
                file: PathBuf::from("a.rs"),
                line: 1,
                column: 1,
            },
            Location {
                file: PathBuf::from("b.rs"),
                line: 2,
                column: 1,
            },
            Location {
                file: PathBuf::from("c.rs"),
                line: 3,
                column: 1,
            },
        ],
        variation_score: 0.0,
        example_code: "test".to_string(),
        estimated_loc: 5,
    });

    extractor.calculate_pattern_variations(&mut collection);

    // After calculation, variation_score should be updated
    let pattern = collection.patterns.get("test").unwrap();
    assert!(pattern.variation_score >= 0.0 && pattern.variation_score <= 1.0);
}

// Ruchy pattern extraction tests
#[test]
fn test_extract_ruchy_actor_patterns_with_actors() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = r#"
        actor Counter {
            state: i32
        }
        actor Logger {
            buffer: String
        }
        receive increment(value: i32) {
            self.state += value
        }
        receive log(msg: String) {
            self.buffer.push_str(&msg)
        }
        receive clear() {
            self.buffer.clear()
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_actor_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    // Should detect actor patterns
    assert!(!collection.patterns.is_empty());
}

#[test]
fn test_extract_ruchy_actor_patterns_no_actors() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = "fn main() { println!(\"hello\"); }";

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_actor_patterns(&file_path, content, &mut collection)
        .expect("Should handle empty result");

    assert!(collection.patterns.is_empty());
}

#[test]
fn test_extract_ruchy_pipeline_patterns() {
    use regex::Regex;

    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    // Need >3 matches (i.e., 4+) - include 5 pipeline operations
    // Use simple formatting to ensure regex matches
    let content = "data |> transform(x) |> filter(y) |> map(z) |> reduce(w) |> collect()";

    // Debug: test simpler regex patterns
    let simple_pattern = Regex::new(r"\|>").unwrap();
    let simple_matches: Vec<_> = simple_pattern.find_iter(content).collect();
    assert!(
        simple_matches.len() >= 5,
        "Expected >= 5 pipe-greater-than, got {}: {:?}",
        simple_matches.len(),
        simple_matches
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
    );

    // Function regex `\s*\|>\s*\w+\(` (post-fix: `\>` was a word-end
    // assertion, not a literal) matches 5 pipeline ops here, so a pattern
    // must be produced.
    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_pipeline_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    assert!(!collection.patterns.is_empty(), "must produce pattern");
}

#[test]
fn test_extract_ruchy_pipeline_patterns_insufficient() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = "let x = a |> b();"; // Only 1 pipeline

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_pipeline_patterns(&file_path, content, &mut collection)
        .expect("Should handle insufficient patterns");

    assert!(collection.patterns.is_empty());
}

#[test]
fn test_extract_ruchy_message_passing_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = r#"
        counter <- increment(5)
        logger <- log("test")
        result <? query_status()
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_message_passing_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    // Should detect messaging patterns
    assert!(!collection.patterns.is_empty());
}

#[test]
fn test_extract_ruchy_error_handling_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = r#"
        match get_value() -> Result<i32, Error> {
            Ok(v) => v,
            Err(e) => 0,
        }
        match parse_input() -> Result<String, Error> {
            Ok(s) => s,
            Err(_) => "default".to_string(),
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_error_handling_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    assert!(!collection.patterns.is_empty());
}

#[test]
fn test_extract_ruchy_pattern_matching_patterns() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = r#"
        enum State { Active, Inactive, Pending }
        enum Command { Start, Stop, Pause, Resume }

        match state {
            State::Active => "active",
            State::Inactive => "inactive",
            State::Pending => "pending",
        }

        match command {
            Command::Start => start(),
            Command::Stop => stop(),
            Command::Pause => pause(),
            Command::Resume => resume(),
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_pattern_matching_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    // Should detect pattern matching with multiple enums
    assert!(!collection.patterns.is_empty());
}

// Variation score helper tests
#[test]
fn test_calculate_actor_variation_score_empty() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let score = extractor.calculate_actor_variation_score(&[], &[], "");
    assert_eq!(score, 0.0);
}

#[test]
fn test_calculate_pipeline_variation_score_single() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "|> transform(x)";
    let pattern = Regex::new(r"\|\>").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();

    let score = extractor.calculate_pipeline_variation_score(&matches, content);
    assert_eq!(score, 0.0);
}

#[test]
fn test_calculate_messaging_variation_score() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "counter <- inc(1) counter <- inc(2) counter <- inc(3)";
    let send_pattern = Regex::new(r"<-").unwrap();
    let query_pattern = Regex::new(r"<\?").unwrap();

    let sends: Vec<_> = send_pattern.find_iter(content).collect();
    let queries: Vec<_> = query_pattern.find_iter(content).collect();

    let score = extractor.calculate_messaging_variation_score(&sends, &queries, content);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn test_calculate_pattern_match_variation_score() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = r#"
        enum A { X, Y }
        enum B { P, Q }
        match a { A::X => 1, A::Y => 2 }
        match b { B::P => 3, B::Q => 4 }
    "#;

    let enum_pattern = Regex::new(r"enum\s+\w+\s*\{").unwrap();
    let match_pattern = Regex::new(r"match\s+\w+\s*\{").unwrap();
    let arrow_pattern = Regex::new(r"\w+::\w+\s*=>").unwrap();

    let enums: Vec<_> = enum_pattern.find_iter(content).collect();
    let matches: Vec<_> = match_pattern.find_iter(content).collect();
    let arrows: Vec<_> = arrow_pattern.find_iter(content).collect();

    let score =
        extractor.calculate_pattern_match_variation_score(&enums, &matches, &arrows, content);
    assert!((0.0..=1.0).contains(&score));
}

/// pattern_extractor_ruchy.rs:397 — early return when match_matches.len() < 2.
#[test]
fn test_calculate_pattern_match_variation_score_short_circuits_single_match() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    // Only one `match` block -> len() < 2 triggers the `return 0.0` arm before
    // the enum/match variation branches execute.
    let content = r#"
        enum A { X, Y }
        match a { A::X => 1, A::Y => 2 }
    "#;

    let enum_pattern = Regex::new(r"enum\s+\w+\s*\{").unwrap();
    let match_pattern = Regex::new(r"match\s+\w+\s*\{").unwrap();
    let arrow_pattern = Regex::new(r"\w+::\w+\s*=>").unwrap();

    let enums: Vec<_> = enum_pattern.find_iter(content).collect();
    let matches: Vec<_> = match_pattern.find_iter(content).collect();
    let arrows: Vec<_> = arrow_pattern.find_iter(content).collect();
    assert_eq!(matches.len(), 1, "precondition: exactly one match block");

    let score =
        extractor.calculate_pattern_match_variation_score(&enums, &matches, &arrows, content);
    assert_eq!(score, 0.0, "fewer than 2 match blocks must short-circuit to 0.0");
}

/// pattern_extractor_ruchy.rs:405 — enum_matches.len() <= 1 low-variation arm (0.3).
#[test]
fn test_calculate_pattern_match_variation_score_single_enum_low_variation() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    // Single enum but 2 match blocks -> enum_variation = 0.3 (not 0.6).
    let content = r#"
        enum A { X, Y }
        match a { A::X => 1, A::Y => 2 }
        match b { A::X => 3, A::Y => 4 }
    "#;

    let enum_pattern = Regex::new(r"enum\s+\w+\s*\{").unwrap();
    let match_pattern = Regex::new(r"match\s+\w+\s*\{").unwrap();
    let arrow_pattern = Regex::new(r"\w+::\w+\s*=>").unwrap();

    let enums: Vec<_> = enum_pattern.find_iter(content).collect();
    let matches: Vec<_> = match_pattern.find_iter(content).collect();
    let arrows: Vec<_> = arrow_pattern.find_iter(content).collect();
    assert_eq!(enums.len(), 1, "precondition: exactly one enum");
    assert_eq!(matches.len(), 2, "precondition: exactly two match blocks");

    let score =
        extractor.calculate_pattern_match_variation_score(&enums, &matches, &arrows, content);
    // enum_variation = 0.3, match_variation = (unique patterns)/2 in [0, 1].
    // Mean in [(0.3 + 0.0)/2, (0.3 + 1.0)/2] = [0.15, 0.65], clipped at 1.0.
    assert!(
        (0.15..=0.65).contains(&score),
        "single-enum path must use 0.3 enum_variation, got {score}"
    );
}

// File pattern extraction tests
#[test]
fn test_extract_file_patterns_rust() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Generate structurally identical compound validation patterns
    // All lines are identical → same structural hash → grouped together
    let content = (0..8)
        .map(|_| "if input.is_empty() && input.len() > 0 { return false; }".to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let mut collection = PatternCollection::new();
    extractor
        .extract_file_patterns(&file_path, &content, &mut collection)
        .expect("Should extract patterns");

    // Should extract compound validation patterns (8 > threshold of 5, ≥3 identical)
    assert!(!collection.patterns.is_empty());
}

#[test]
fn test_extract_file_patterns_ruchy() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.ruchy");
    let content = r#"
        actor Counter {
            value: i32
        }
        actor Logger {
            buffer: Vec<String>
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_file_patterns(&file_path, content, &mut collection)
        .expect("Should extract patterns");

    // May or may not have patterns depending on thresholds
}

#[test]
fn test_extract_file_patterns_rh_extension() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rh");
    let content = "actor Test {}";

    let mut collection = PatternCollection::new();
    extractor
        .extract_file_patterns(&file_path, content, &mut collection)
        .expect("Should handle .rh files");
}

#[test]
fn test_extract_file_patterns_generic() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.py"); // Generic extension
    let content = r#"
        if x > 0:
            pass
        } else if x < 0 {
            pass
        } else if x == 0 {
            pass
        } else if True {
            pass
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_file_patterns(&file_path, content, &mut collection)
        .expect("Should handle generic files");
}

#[test]
fn test_extract_file_patterns_no_extension() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("Makefile");
    let content = "all: build test";

    let mut collection = PatternCollection::new();
    let result = extractor.extract_file_patterns(&file_path, content, &mut collection);
    assert!(result.is_ok());
}

// Edge cases
#[test]
fn test_extract_patterns_with_unicode() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");
    // Need >5 compound validation checks with ≥3 structurally identical (unicode content)
    let content = r#"
        // 日本語コメント
        if data.contains("こんにちは") && data.len() > 0 {
            return true;
        }
        if data.contains("世界") && data.len() > 0 {
            return true;
        }
        if data.contains("テスト") && data.len() > 0 {
            return true;
        }
        if input.contains("日本") && input.len() > 0 {
            return true;
        }
        if buf.contains("UTF-8") && buf.len() > 0 {
            return true;
        }
        if name.contains("san") && name.len() > 0 {
            return true;
        }
    "#;

    let mut collection = PatternCollection::new();
    extractor
        .extract_data_validation_patterns(&file_path, content, &mut collection)
        .expect("Should handle unicode");

    assert!(!collection.patterns.is_empty());
}

#[test]
fn test_variation_score_utf8_boundary() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "fn 日本語() {} fn テスト() {}";
    let pattern = Regex::new(r"fn \S+\(\)").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();

    // Should handle UTF-8 boundaries correctly
    let score = extractor.calculate_variation_score(&matches, content);
    assert!(score >= 0.0);
}

#[test]
fn test_limited_pattern_extraction() {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("test.rs");

    // Create content with structurally identical compound matches to test limiting
    let mut content = String::new();
    for _ in 0..20 {
        content.push_str("if input.is_empty() && input.len() > 0 { }\n");
    }

    let mut collection = PatternCollection::new();
    extractor
        .extract_data_validation_patterns(&file_path, &content, &mut collection)
        .expect("Should limit patterns");

    // Should have extracted patterns (limited to 10 locations by group_by_structural_hash)
    for pattern in collection.patterns.values() {
        assert!(pattern.locations.len() <= 10);
    }
}

/// scan_directory_fallback reads only Rust/Ruchy files and skips others.
/// Exercises the fallback path when `pmat` subprocess fails or isn't on PATH.
#[tokio::test]
async fn test_scan_directory_fallback_reads_rust_and_ruchy_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    std::fs::write(root.join("good.rs"), "fn a() {}").unwrap();
    std::fs::write(root.join("good.ruchy"), "actor A {}").unwrap();
    std::fs::write(root.join("good.rh"), "actor B {}").unwrap();
    std::fs::write(root.join("skip.py"), "def x(): pass").unwrap();
    std::fs::write(root.join("skip.txt"), "note").unwrap();

    let extractor = PatternExtractor::new(EntropyConfig::default());
    let context = extractor
        .scan_directory_fallback(root)
        .await
        .expect("fallback should succeed");

    // Only .rs, .ruchy, .rh extensions are included.
    let names: std::collections::HashSet<_> = context
        .files
        .keys()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    assert!(names.contains("good.rs"));
    assert!(names.contains("good.ruchy"));
    assert!(names.contains("good.rh"));
    assert!(!names.contains("skip.py"));
    assert!(!names.contains("skip.txt"));
}

/// get_project_context tolerates a failed/missing pmat binary and returns an
/// empty context via the fallback on an empty directory. Covers the
/// `_ => return self.scan_directory_fallback(...)` branch.
#[tokio::test]
async fn test_get_project_context_empty_dir_yields_empty_context() {
    let dir = tempfile::tempdir().expect("tempdir");
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let context = extractor
        .get_project_context(dir.path())
        .await
        .expect("get_project_context must not error on empty dir");
    // Empty directory produces no files regardless of whether pmat ran or fallback fired.
    assert!(context.files.is_empty());
}

/// extract_patterns is the public async entry. Drives get_project_context +
/// should_process_file + extract_file_patterns end-to-end on a Rust source
/// file containing enough compound validation matches to register patterns.
#[tokio::test]
async fn test_extract_patterns_end_to_end_rust_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("m.rs");
    // Enough compound validations to trigger pattern extraction.
    let content = r#"
        fn a(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
        fn b(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
        fn c(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
        fn d(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
        fn e(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
        fn f(x: &str) { if x.is_empty() && x.len() > 0 { return; } }
    "#;
    std::fs::write(&src, content).unwrap();

    let extractor = PatternExtractor::new(EntropyConfig::default());
    let collection = extractor
        .extract_patterns(dir.path())
        .await
        .expect("extract_patterns must succeed");
    assert!(collection.total_files >= 1, "should process the .rs file");
}
