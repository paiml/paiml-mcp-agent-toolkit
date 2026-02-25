// Enrichment and formatter tests: churn, clones, entropy, faults, graph metrics,
// ranking, empty/edge cases.  Lines 870–1355 of the original tests.rs.

#[test]
fn test_enrich_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    // Initially no churn data
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score - 0.0).abs() < 0.01);

    // Build churn map
    let mut churn_map = HashMap::new();
    churn_map.insert("test.rs".to_string(), (42u32, 0.75f32));

    // Enrich results
    enrich_with_churn(&mut results, &churn_map);

    // Verify churn data was applied
    assert_eq!(results[0].commit_count, 42);
    assert!((results[0].churn_score - 0.75).abs() < 0.01);
}

#[test]
fn test_enrich_with_churn_no_match() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    // Churn map with different file
    let mut churn_map = HashMap::new();
    churn_map.insert("other.rs".to_string(), (100u32, 0.9f32));

    // Enrich results - should not match
    enrich_with_churn(&mut results, &churn_map);

    // Verify churn data was NOT changed (no match)
    assert_eq!(results[0].commit_count, 0);
    assert!((results[0].churn_score - 0.0).abs() < 0.01);
}

#[test]
fn test_format_text_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 25;
    result.churn_score = 0.8;

    let text = format_text(&[result]);
    assert!(text.contains("🔥 Hot"));
    assert!(text.contains("25 commits"));
    assert!(text.contains("80%"));
}

#[test]
fn test_format_text_with_code_shows_metrics() {
    let entry = create_test_entry("test_func", 15, 3.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.satd_count = 2;
    result.commit_count = 30;
    result.churn_score = 0.7;

    let text = format_text_with_code(&[result], None);
    // Should show complexity (format: "C:15" without space)
    assert!(text.contains("C:15"), "missing complexity");
    // Should show SATD warning as "⚠2" (warning symbol + count)
    assert!(text.contains("⚠2"), "missing SATD");
    // Should show churn (high commit count shown as "🔥" indicator)
    assert!(
        text.contains("🔥") || text.contains("30"),
        "missing churn indicator"
    );
    // Should show function name in header
    assert!(text.contains("test_func"), "missing function name");
}

#[test]
fn test_format_text_with_code_minimal_metrics() {
    let entry = create_test_entry("simple_func", 3, 1.0);
    let result = QueryResult::from_entry(&entry, 0.9, true);

    let text = format_text_with_code(&[result], None);
    // Should still show complexity (format: "C:3" without space)
    assert!(text.contains("C:3"), "missing complexity");
    // Should NOT show SATD (is 0)
    assert!(!text.contains("SATD"), "should not show SATD when 0");
    // Should NOT show churn (is 0)
    assert!(!text.contains("commits"), "should not show churn when 0");
}

#[test]
fn test_format_text_with_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 5;
    result.churn_score = 0.2; // Below 0.5 threshold

    let text = format_text(&[result]);
    assert!(!text.contains("🔥 Hot"));
    assert!(text.contains("5c")); // low churn shows compact "5c" format
}

#[test]
fn test_format_markdown_with_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 30;
    result.churn_score = 0.9;

    let md = format_markdown(&[result]);
    assert!(md.contains("🔥 **Hot"));
    assert!(md.contains("30 commits"));
    assert!(md.contains("90%"));
}

#[test]
fn test_query_rank_by_pagerank() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::PageRank,
                ..Default::default()
            },
        )
        .unwrap();
    // Should return results ordered by PageRank
    assert!(!results.is_empty());
    // Verify descending PageRank order
    for w in results.windows(2) {
        assert!(
            w[0].pagerank >= w[1].pagerank || (w[0].pagerank - w[1].pagerank).abs() < 1e-6,
            "PageRank not descending: {} vs {}",
            w[0].pagerank,
            w[1].pagerank,
        );
    }
}

#[test]
fn test_query_rank_by_centrality() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::Centrality,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_query_rank_by_indegree() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                rank_by: RankBy::InDegree,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // Verify descending in-degree order
    for w in results.windows(2) {
        assert!(
            w[0].in_degree >= w[1].in_degree,
            "InDegree not descending: {} vs {}",
            w[0].in_degree,
            w[1].in_degree,
        );
    }
}

#[test]
fn test_query_min_pagerank_filter() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_pagerank: Some(0.0),
                ..Default::default()
            },
        )
        .unwrap();
    // All results should have pagerank >= 0.0 (all pass)
    for r in &results {
        assert!(r.pagerank >= 0.0);
    }

    // Very high threshold should filter everything
    let results_strict = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_pagerank: Some(1.0),
                ..Default::default()
            },
        )
        .unwrap();
    // No function will have pagerank >= 1.0
    assert!(results_strict.is_empty());
}

#[test]
fn test_rankby_from_str() {
    assert_eq!("relevance".parse::<RankBy>().unwrap(), RankBy::Relevance);
    assert_eq!("rel".parse::<RankBy>().unwrap(), RankBy::Relevance);
    assert_eq!("pagerank".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("pr".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("importance".parse::<RankBy>().unwrap(), RankBy::PageRank);
    assert_eq!("centrality".parse::<RankBy>().unwrap(), RankBy::Centrality);
    assert_eq!("degree".parse::<RankBy>().unwrap(), RankBy::Centrality);
    assert_eq!("indegree".parse::<RankBy>().unwrap(), RankBy::InDegree);
    assert_eq!("callers".parse::<RankBy>().unwrap(), RankBy::InDegree);
    assert_eq!("impact".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert_eq!("roi".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert_eq!("coverage".parse::<RankBy>().unwrap(), RankBy::Impact);
    assert_eq!(
        "cross-project".parse::<RankBy>().unwrap(),
        RankBy::CrossProject
    );
    assert_eq!(
        "crossproject".parse::<RankBy>().unwrap(),
        RankBy::CrossProject
    );
    assert_eq!("xproject".parse::<RankBy>().unwrap(), RankBy::CrossProject);
    assert!("invalid".parse::<RankBy>().is_err());
}

#[test]
fn test_format_text_with_clones() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.clone_count = 3;
    result.duplication_score = 0.85;

    let text = format_text(&[result]);
    assert!(text.contains("📋 Clones: 3"), "missing clones in text");
    assert!(text.contains("85%"), "missing duplication score");
}

#[test]
fn test_format_text_with_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.pattern_diversity = 0.2;

    let text = format_text(&[result]);
    assert!(text.contains("🔄 Repetitive"), "missing entropy in text");
}

#[test]
fn test_format_text_with_fault_annotations() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.fault_annotations = vec![
        "BH001: Boundary condition at line 10".to_string(),
        "BH002: Arithmetic overflow at line 15".to_string(),
    ];

    let text = format_text(&[result]);
    assert!(text.contains("BH001"), "missing fault annotation");
    assert!(text.contains("BH002"), "missing fault annotation");
}

#[test]
fn test_format_text_with_graph_metrics() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.pagerank = 0.05;
    result.in_degree = 3;
    result.out_degree = 2;

    let text = format_text(&[result]);
    assert!(text.contains("PageRank"), "missing graph metrics");
    assert!(text.contains("In-Degree"), "missing in-degree");
}

#[test]
fn test_format_text_with_large_loc() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.loc = 100;

    let text = format_text(&[result]);
    assert!(text.contains("LOC: 100"), "missing LOC for large function");
}

#[test]
fn test_format_markdown_with_clones_and_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.clone_count = 2;
    result.duplication_score = 0.7;
    result.pattern_diversity = 0.15;
    result.pagerank = 0.01;
    result.in_degree = 5;
    result.out_degree = 3;
    result.doc_comment = Some("Test doc".to_string());

    let md = format_markdown(&[result]);
    assert!(md.contains("📋 **Clones:"), "missing clones in markdown");
    assert!(
        md.contains("🔄 **Repetitive"),
        "missing entropy in markdown"
    );
    assert!(md.contains("**Graph:**"), "missing graph metrics");
    assert!(md.contains("**Documentation:**"), "missing doc comment");
}

#[test]
fn test_format_markdown_with_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, false);
    result.commit_count = 3;
    result.churn_score = 0.2;

    let md = format_markdown(&[result]);
    assert!(md.contains("Commits: 3"), "missing low churn commits");
    assert!(!md.contains("🔥"), "should not show fire for low churn");
}

#[test]
fn test_format_text_with_code_clones_and_faults() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.clone_count = 3;
    result.fault_annotations = vec![
        "BH001: Boundary condition at line 10".to_string(),
        "BH002: Arithmetic overflow at line 15".to_string(),
        "BH003: Other pattern at line 20".to_string(),
    ];

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("📋"), "missing clone indicator");
    assert!(text.contains("🐛"), "missing fault indicator");
}

#[test]
fn test_format_text_with_code_call_graph_truncation() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    // More than 5 calls -> should truncate
    result.calls = (0..8).map(|i| format!("func_{i}")).collect();
    // More than 3 called_by -> should truncate
    result.called_by = (0..6).map(|i| format!("caller_{i}")).collect();

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("(+3 more)"), "calls not truncated at 5");
    assert!(text.contains("(+3 more)"), "called_by not truncated at 3");
}

#[test]
fn test_format_text_with_code_doc_truncation() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.doc_comment = Some("A".repeat(150)); // >100 chars

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("..."), "long doc comment not truncated");
}

#[test]
fn test_format_text_with_code_no_source() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.9, false);

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("source hidden"), "missing hint for no source");
}

#[test]
fn test_format_text_with_code_high_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.005; // scaled: 50 -> >= 10 threshold

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("★"), "missing high pagerank star");
}

#[test]
fn test_format_text_with_code_medium_pagerank() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.0005; // scaled: 5 -> >= 1 threshold

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("★"), "missing medium pagerank star");
}

#[test]
fn test_format_text_with_code_high_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 10;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("↓10"), "missing high in-degree");
}

#[test]
fn test_format_text_with_code_low_indegree() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.in_degree = 2;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("↓2"), "missing low in-degree");
}

#[test]
fn test_format_text_with_code_medium_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 15;
    result.churn_score = 0.4;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("15c"), "missing medium churn");
    assert!(text.contains("40%"), "missing churn percentage");
}

#[test]
fn test_format_text_with_code_low_churn() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 3;
    result.churn_score = 0.1;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("3c"), "missing low churn");
}

#[test]
fn test_format_text_with_code_high_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.9;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("H:90%"), "missing high entropy indicator");
}

#[test]
fn test_format_text_with_code_low_entropy() {
    let entry = create_test_entry("test_func", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pattern_diversity = 0.2;

    let text = format_text_with_code(&[result], None);
    assert!(text.contains("🔄"), "missing low entropy indicator");
}

#[test]
fn test_format_json_with_empty_results() {
    let json = format_json(&[]).unwrap();
    assert_eq!(json.trim(), "[]");
}

#[test]
fn test_format_markdown_empty() {
    let md = format_markdown(&[]);
    assert!(md.contains("0 functions"));
}

#[test]
fn test_format_text_empty() {
    let text = format_text(&[]);
    assert!(text.contains("Found 0 functions"));
}

#[test]
fn test_format_text_with_code_empty() {
    let text = format_text_with_code(&[], None);
    assert!(text.is_empty());
}
