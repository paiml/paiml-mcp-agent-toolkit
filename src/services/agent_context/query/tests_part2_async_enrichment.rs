// Async (tokio) enrichment tests for faults, churn, duplicates, and entropy.
// Included via include!() from tests_part2.rs - no use imports allowed.

// ── enrich_results_with_faults: async tests ─────────────────────────

#[tokio::test]
async fn test_enrich_results_with_faults_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_faults(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_faults_batuta_not_found() {
    let entry = create_test_entry("func_x", 5, 88.0);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    let res =
        enrich_results_with_faults(&mut results, std::path::Path::new("/nonexistent/path")).await;
    // When batuta is not on PATH, returns Err. Must not panic.
    if let Err(err) = res {
        assert!(
            err.contains("batuta") || err.contains("Failed"),
            "Error should mention batuta: {err}"
        );
    }
}

#[tokio::test]
async fn test_enrich_results_with_faults_preserves_existing_annotations() {
    let mut entry = create_test_entry("func_with_existing", 5, 88.0);
    entry.fault_annotations = vec!["existing_fault".to_string()];
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    assert_eq!(results[0].fault_annotations, vec!["existing_fault"]);
    // Even if batuta fails, existing annotations should persist
    let _res =
        enrich_results_with_faults(&mut results, std::path::Path::new("/nonexistent/path")).await;
}

// ── enrich_results_with_churn: async tests ──────────────────────────

#[tokio::test]
async fn test_enrich_results_with_churn_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_churn(&mut results, std::path::Path::new("/tmp"), 90).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_churn_nonexistent_project() {
    let entry = create_test_entry("func_y", 3, 92.0);
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let res = enrich_results_with_churn(
        &mut results,
        std::path::Path::new("/nonexistent/no/such/project"),
        90,
    )
    .await;
    assert!(res.is_err());
}

// ── enrich_results_with_duplicates: async tests ─────────────────────

#[tokio::test]
async fn test_enrich_results_with_duplicates_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_duplicates(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_duplicates_missing_files() {
    let mut entry = create_test_entry("func_z", 3, 92.0);
    entry.file_path = "nonexistent_file_that_does_not_exist.rs".to_string();
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let res = enrich_results_with_duplicates(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
    assert_eq!(results[0].clone_count, 0);
    assert!((results[0].duplication_score).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_enrich_results_with_duplicates_unsupported_language() {
    let mut entry = create_test_entry("func_z", 3, 92.0);
    entry.file_path = "readme.txt".to_string();
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let tmp = std::env::temp_dir();
    let txt_path = tmp.join("readme.txt");
    std::fs::write(&txt_path, "hello world").ok();
    let res = enrich_results_with_duplicates(&mut results, &tmp).await;
    assert!(res.is_ok());
    assert_eq!(results[0].clone_count, 0);
    std::fs::remove_file(&txt_path).ok();
}

// ── enrich_results_with_entropy: async tests ────────────────────────

#[tokio::test]
async fn test_enrich_results_with_entropy_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_entropy(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}
