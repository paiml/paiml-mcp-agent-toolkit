// Synchronous enrichment tests: enrich_with_churn and build_churn_map edge cases.
// Included via include!() from tests_part2.rs - no use imports allowed.

// ── enrich_with_churn: additional edge cases ────────────────────────

#[test]
fn test_enrich_with_churn_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let churn_map = HashMap::new();
    enrich_with_churn(&mut results, &churn_map);
    assert!(results.is_empty());
}

#[test]
fn test_enrich_with_churn_empty_map() {
    let entry = create_test_entry("func_a", 3, 1.0);
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let churn_map = HashMap::new();
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score).abs() < f32::EPSILON);
}

#[test]
fn test_enrich_with_churn_multiple_results_same_file() {
    let mut entry_a = create_test_entry("func_a", 3, 1.0);
    entry_a.file_path = "src/lib.rs".to_string();
    let mut entry_b = create_test_entry("func_b", 5, 2.0);
    entry_b.file_path = "src/lib.rs".to_string();
    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.8, false),
        QueryResult::from_entry(&entry_b, 0.7, false),
    ];
    let mut churn_map = HashMap::new();
    churn_map.insert("src/lib.rs".to_string(), (20u32, 0.65f32));
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 20);
    assert!((results[0].churn_score - 0.65).abs() < 0.01);
    assert_eq!(results[1].commit_count, 20);
    assert!((results[1].churn_score - 0.65).abs() < 0.01);
}

#[test]
fn test_enrich_with_churn_mixed_match_and_miss() {
    let mut entry_a = create_test_entry("func_a", 3, 1.0);
    entry_a.file_path = "src/known.rs".to_string();
    let mut entry_b = create_test_entry("func_b", 5, 2.0);
    entry_b.file_path = "src/unknown.rs".to_string();
    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.8, false),
        QueryResult::from_entry(&entry_b, 0.7, false),
    ];
    let mut churn_map = HashMap::new();
    churn_map.insert("src/known.rs".to_string(), (50u32, 0.99f32));
    enrich_with_churn(&mut results, &churn_map);
    assert_eq!(results[0].commit_count, 50);
    assert!((results[0].churn_score - 0.99).abs() < 0.01);
    assert_eq!(results[1].commit_count, 0);
    assert!((results[1].churn_score).abs() < f32::EPSILON);
}

// ── build_churn_map: additional edge cases ──────────────────────────

#[test]
fn test_build_churn_map_empty() {
    let metrics: Vec<crate::models::churn::FileChurnMetrics> = vec![];
    let map = build_churn_map(&metrics);
    assert!(map.is_empty());
}

#[test]
fn test_build_churn_map_single_entry() {
    use crate::models::churn::FileChurnMetrics;
    let metrics = vec![FileChurnMetrics {
        path: std::path::PathBuf::from("src/main.rs"),
        relative_path: "src/main.rs".to_string(),
        commit_count: 7,
        unique_authors: vec!["alice".to_string()],
        additions: 50,
        deletions: 10,
        churn_score: 0.3,
        last_modified: chrono::Utc::now(),
        first_seen: chrono::Utc::now(),
    }];
    let map = build_churn_map(&metrics);
    assert_eq!(map.len(), 1);
    let (count, score) = map["src/main.rs"];
    assert_eq!(count, 7);
    assert!((score - 0.3).abs() < 0.01);
}

#[test]
fn test_build_churn_map_commit_count_u32_conversion() {
    use crate::models::churn::FileChurnMetrics;
    let metrics = vec![FileChurnMetrics {
        path: std::path::PathBuf::from("big.rs"),
        relative_path: "big.rs".to_string(),
        commit_count: 1000,
        unique_authors: vec![],
        additions: 0,
        deletions: 0,
        churn_score: 1.0,
        last_modified: chrono::Utc::now(),
        first_seen: chrono::Utc::now(),
    }];
    let map = build_churn_map(&metrics);
    assert_eq!(map["big.rs"].0, 1000);
}
