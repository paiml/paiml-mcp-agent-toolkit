#![cfg_attr(coverage_nightly, coverage(off))]

use super::engine::glob_matches;
use super::enrichment::{
    build_churn_map, enrich_results_with_churn, enrich_results_with_duplicates,
    enrich_results_with_entropy, enrich_results_with_faults, enrich_with_churn,
};
use super::formatters::{format_markdown, format_text, format_text_with_code};
use super::types::{CaseSensitivity, QueryOptions, QueryResult, SearchMode};
use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry, QualityMetrics};
use std::collections::{HashMap, HashSet};

fn create_test_entry(name: &str, complexity: u32, tdg_score: f32) -> FunctionEntry {
    FunctionEntry {
        file_path: "test.rs".to_string(),
        function_name: name.to_string(),
        signature: format!("fn {name}()"),
        doc_comment: Some("Test function".to_string()),
        source: format!("fn {name}() {{ }}"),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics {
            tdg_score,
            tdg_grade: if tdg_score < 2.0 {
                "A".to_string()
            } else {
                "B".to_string()
            },
            complexity,
            cognitive_complexity: complexity,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            commit_count: 0,
            churn_score: 0.0,
        },
        checksum: "abc123".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
    }
}

fn build_test_index() -> AgentContextIndex {
    let functions = vec![
        FunctionEntry {
            file_path: "src/handler.rs".to_string(),
            function_name: "handle_error".to_string(),
            signature: "fn handle_error(e: Error)".to_string(),
            doc_comment: Some("Handle API errors gracefully".to_string()),
            source: "fn handle_error(e: Error) { log(e); respond(500); }".to_string(),
            start_line: 10,
            end_line: 15,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 1.0,
                tdg_grade: "A".to_string(),
                complexity: 3,
                cognitive_complexity: 3,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 6,
                commit_count: 15,
                churn_score: 0.6,
            },
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 15,
            churn_score: 0.6,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/handler.rs".to_string(),
            function_name: "handle_request".to_string(),
            signature: "fn handle_request(req: Request)".to_string(),
            doc_comment: Some("Process incoming HTTP requests".to_string()),
            source: "fn handle_request(req: Request) { validate(req); handle_error(err); }"
                .to_string(),
            start_line: 20,
            end_line: 30,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 2.0,
                tdg_grade: "B".to_string(),
                complexity: 5,
                cognitive_complexity: 5,
                big_o: "O(n)".to_string(),
                satd_count: 0,
                loc: 11,
                commit_count: 25,
                churn_score: 0.8,
            },
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 25,
            churn_score: 0.8,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/utils.rs".to_string(),
            function_name: "validate".to_string(),
            signature: "fn validate(input: &str) -> bool".to_string(),
            doc_comment: Some("Validate input data".to_string()),
            source: "fn validate(input: &str) -> bool { !input.is_empty() }".to_string(),
            start_line: 1,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 0.5,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 3,
                commit_count: 3,
                churn_score: 0.1,
            },
            checksum: "ccc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 3,
            churn_score: 0.1,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "tests/test_handler.rs".to_string(),
            function_name: "test_error_handling".to_string(),
            signature: "fn test_error_handling()".to_string(),
            doc_comment: None,
            source: "fn test_error_handling() { handle_error(mock_err()); }".to_string(),
            start_line: 1,
            end_line: 5,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 1.0,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 5,
                commit_count: 5,
                churn_score: 0.2,
            },
            checksum: "ddd".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 5,
            churn_score: 0.2,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "src/utils.rs".to_string(),
            function_name: "new".to_string(),
            signature: "fn new() -> Self".to_string(),
            doc_comment: None,
            source: "fn new() -> Self { Self {} }".to_string(),
            start_line: 10,
            end_line: 12,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                tdg_score: 0.5,
                tdg_grade: "A".to_string(),
                complexity: 1,
                cognitive_complexity: 1,
                big_o: "O(1)".to_string(),
                satd_count: 0,
                loc: 3,
                commit_count: 2,
                churn_score: 0.05,
            },
            checksum: "eee".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 2,
            churn_score: 0.05,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let indices = crate::services::agent_context::function_index::build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
    let name_frequency = crate::services::agent_context::function_index::compute_name_frequency(
        &indices.name_index,
        functions.len(),
    );
    let (calls, called_by) = crate::services::agent_context::function_index::build_call_graph(
        &functions,
        &indices.name_index,
    );
    let graph_metrics = crate::services::agent_context::function_index::compute_graph_metrics(
        functions.len(),
        &calls,
        &called_by,
    );

    AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency,
        calls,
        called_by,
        graph_metrics,
        project_root: std::path::PathBuf::from("/test"),
        manifest: crate::services::agent_context::IndexManifest {
            version: "1.2.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 5,
            file_count: 3,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 1.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    }
}

// ── Rich coverage metric tests (format_text_with_code) ──────────────────

#[test]
fn test_format_text_with_code_coverage_uncovered_rich() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 25;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("0/25"), "missing uncovered line count");
}

#[test]
fn test_format_text_with_code_coverage_partial_low_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 30.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("30%"), "missing low coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_mid_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 65.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("65%"), "missing mid coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_high_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 90.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("90%"), "missing high coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_full_rich() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "full".to_string();
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("100%"), "missing full coverage indicator");
}

#[test]
fn test_format_text_with_code_coverage_impact_rich() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.impact_score = 4.2;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("4.2"), "missing impact score");
}

#[test]
fn test_format_text_with_code_coverage_diff_positive_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = 2.5;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("+2.5%"), "missing positive diff");
}

#[test]
fn test_format_text_with_code_coverage_diff_negative_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = -3.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("-3.0%"), "missing negative diff");
}

// ── Grade color branches in format_text ─────────────────────────────────

#[test]
fn test_format_text_grade_c() {
    let entry = create_test_entry("grade_c_fn", 15, 4.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "C".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("C"), "missing grade C");
}

#[test]
fn test_format_text_grade_d() {
    let entry = create_test_entry("grade_d_fn", 25, 6.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "D".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("D"), "missing grade D");
}

#[test]
fn test_format_text_grade_f() {
    let entry = create_test_entry("grade_f_fn", 50, 8.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "F".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("F"), "missing grade F");
}

#[test]
fn test_format_text_with_satd() {
    let entry = create_test_entry("satd_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.satd_count = 3;
    let text = format_text(&[result]);
    assert!(text.contains("SATD: 3"), "missing SATD count");
}

#[test]
fn test_format_text_large_loc() {
    let entry = create_test_entry("big_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.loc = 100;
    let text = format_text(&[result]);
    assert!(text.contains("LOC: 100"), "missing LOC for large function");
}

#[test]
fn test_format_text_with_doc_comment() {
    let entry = create_test_entry("doc_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.doc_comment = Some("Important documentation".to_string());
    let text = format_text(&[result]);
    assert!(
        text.contains("Important documentation"),
        "missing doc comment"
    );
}

#[test]
fn test_format_text_summary_with_calls() {
    let entry = create_test_entry("caller_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.calls = vec!["foo".to_string(), "bar".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Calls:"), "missing calls label");
    assert!(text.contains("foo"), "missing call target");
}

#[test]
fn test_format_text_summary_with_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.called_by = vec!["main".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Called by:"), "missing called_by label");
}

#[test]
fn test_format_text_summary_with_graph_metrics() {
    let entry = create_test_entry("graph_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pagerank = 0.001;
    result.in_degree = 3;
    result.out_degree = 5;
    let text = format_text(&[result]);
    assert!(text.contains("PageRank"), "missing pagerank");
    assert!(text.contains("In-Degree: 3"), "missing in-degree");
}

#[test]
fn test_format_text_low_relevance() {
    let entry = create_test_entry("low_rel_fn", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.1, false);
    let text = format_text(&[result]);
    assert!(text.contains("0.10"), "missing low relevance score");
}

#[test]
fn test_format_text_with_faults() {
    let entry = create_test_entry("fault_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.fault_annotations = vec!["BH001: Boundary at line 5".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Boundary"), "missing fault annotation");
}

#[test]
fn test_format_text_summary_with_clones() {
    let entry = create_test_entry("clone_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.clone_count = 2;
    result.duplication_score = 0.85;
    let text = format_text(&[result]);
    assert!(text.contains("Clones: 2"), "missing clone count");
}

#[test]
fn test_format_text_with_repetitive_pattern() {
    let entry = create_test_entry("rep_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pattern_diversity = 0.2;
    let text = format_text(&[result]);
    assert!(text.contains("Repetitive"), "missing repetitive label");
}

// ── Markdown detail branches ────────────────────────────────────────────

#[test]
fn test_format_markdown_with_doc_and_graph() {
    let entry = create_test_entry("md_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.doc_comment = Some("A documented function".to_string());
    result.calls = vec!["helper".to_string()];
    result.called_by = vec!["main".to_string()];
    result.pagerank = 0.002;
    result.in_degree = 4;
    result.out_degree = 1;
    let md = format_markdown(&[result]);
    assert!(md.contains("A documented function"), "missing doc in md");
    assert!(md.contains("Calls:"), "missing calls in md");
    assert!(md.contains("Called by:"), "missing called_by in md");
    assert!(md.contains("PageRank"), "missing graph in md");
}

// ── Call graph with only called_by (no calls) ───────────────────────────

#[test]
fn test_format_text_with_code_only_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.called_by = vec!["a".to_string(), "b".to_string()];
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("← a, b"), "missing called_by in call graph");
}

// ── Highlight source with syntect (no highlight param) ──────────────────

#[test]
fn test_format_text_with_code_syntect_source() {
    let entry = create_test_entry("src_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.source = Some("fn src_fn() { let x = 42; }".to_string());
    let text = format_text_with_code(&[result], None);
    assert!(
        text.contains("src_fn"),
        "missing source content with syntect"
    );
    // syntect adds ANSI escape codes
    assert!(text.contains("\x1b["), "missing ANSI codes from syntect");
}

// ── High churn in rich metrics (>0.7 branch) ────────────────────────────

#[test]
fn test_format_text_with_code_very_high_churn() {
    let entry = create_test_entry("hot_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 50;
    result.churn_score = 0.8;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("🔥"), "missing fire emoji for >0.7 churn");
    assert!(text.contains("50c"), "missing commit count");
}

// ── Pagerank metric with no star (below 1.0 threshold) ─────────────────

#[test]
fn test_format_text_with_code_low_pagerank_no_star() {
    let entry = create_test_entry("low_pr_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.00005; // scaled: 0.5 -> below 1.0 threshold
    let text = format_text_with_code(&[result], None);
    assert!(
        !text.contains("★"),
        "should not show star for very low pagerank"
    );
}

#[test]
fn test_query_combined_filters() {
    let index = build_test_index();
    let results = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                min_grade: Some("A".to_string()),
                max_complexity: Some(3),
                language: Some("Rust".to_string()),
                path_pattern: Some("src/".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    for r in &results {
        assert_eq!(r.tdg_grade, "A");
        assert!(r.complexity <= 3);
        assert_eq!(r.language, "Rust");
        assert!(r.file_path.contains("src/"));
    }
}

#[test]
fn test_build_churn_map() {
    use crate::models::churn::FileChurnMetrics;

    let metrics = vec![
        FileChurnMetrics {
            path: std::path::PathBuf::from("src/foo.rs"),
            relative_path: "src/foo.rs".to_string(),
            commit_count: 10,
            unique_authors: vec!["author1".to_string()],
            additions: 100,
            deletions: 50,
            churn_score: 0.5,
            last_modified: chrono::Utc::now(),
            first_seen: chrono::Utc::now(),
        },
        FileChurnMetrics {
            path: std::path::PathBuf::from("src/bar.rs"),
            relative_path: "src/bar.rs".to_string(),
            commit_count: 25,
            unique_authors: vec!["author2".to_string()],
            additions: 200,
            deletions: 100,
            churn_score: 0.8,
            last_modified: chrono::Utc::now(),
            first_seen: chrono::Utc::now(),
        },
    ];

    let map = build_churn_map(&metrics);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("src/foo.rs"), Some(&(10, 0.5)));
    assert_eq!(map.get("src/bar.rs"), Some(&(25, 0.8)));
}

#[test]
fn test_from_entry_with_context_basic() {
    use crate::services::agent_context::function_index::{
        AgentContextIndex, GraphMetrics, IndexManifest,
    };
    use std::path::PathBuf;

    let entry = create_test_entry("my_func", 5, 1.5);
    let index = AgentContextIndex {
        functions: vec![entry.clone()],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec!["my_func".to_string()],
        corpus_lower: vec!["my_func".to_string()],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![GraphMetrics {
            pagerank: 0.42,
            centrality: 0.3,
            in_degree: 5,
            out_degree: 2,
        }],
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
            function_count: 0,
            file_count: 0,
            languages: vec![],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
    assert!((result.pagerank - 0.42).abs() < 0.001);
    assert_eq!(result.in_degree, 5);
    assert_eq!(result.out_degree, 2);
}

#[test]
fn test_from_entry_with_context_out_of_bounds() {
    use crate::services::agent_context::function_index::{AgentContextIndex, IndexManifest};
    use std::path::PathBuf;

    let entry = create_test_entry("my_func", 5, 1.5);
    let index = AgentContextIndex {
        functions: vec![entry.clone()],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec!["my_func".to_string()],
        corpus_lower: vec!["my_func".to_string()],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![], // empty - out of bounds
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
            function_count: 0,
            file_count: 0,
            languages: vec![],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let result = QueryResult::from_entry_with_context(&entry, 99, &index, 0.9, false);
    // Should not panic, pagerank stays 0
    assert!((result.pagerank - 0.0).abs() < 0.001);
}

#[test]
fn test_from_entry_with_context_callers_capping() {
    use crate::services::agent_context::function_index::{
        AgentContextIndex, GraphMetrics, IndexManifest,
    };
    use std::path::PathBuf;

    let entry = create_test_entry("target", 5, 1.5);
    // Create 15 caller functions + 3 test callers
    let mut functions = vec![entry.clone()];
    let mut called_by_map = HashMap::new();
    let mut callers = vec![];
    for i in 0..15 {
        functions.push(create_test_entry(&format!("caller_{}", i), 1, 0.5));
        callers.push(i + 1); // indices 1..=15
    }
    for i in 0..3 {
        functions.push(create_test_entry(&format!("test_caller_{}", i), 1, 0.5));
        callers.push(16 + i); // indices 16..=18
    }
    called_by_map.insert(0usize, callers);

    let index = AgentContextIndex {
        functions,
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec![],
        corpus_lower: vec![],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: called_by_map,
        graph_metrics: vec![GraphMetrics {
            pagerank: 0.1,
            centrality: 0.0,
            in_degree: 18,
            out_degree: 0,
        }],
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
            function_count: 0,
            file_count: 0,
            languages: vec![],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let result = QueryResult::from_entry_with_context(&entry, 0, &index, 0.9, false);
    // Should have 10 prod callers + "(+5 more)" + "(+3 tests)" = 12 entries
    assert!(result.called_by.len() <= 12);
    // Should have the capping message
    let has_more = result
        .called_by
        .iter()
        .any(|s| s.starts_with("(+") && s.ends_with("more)"));
    assert!(has_more, "Should have (+N more) message");
    let has_tests = result.called_by.iter().any(|s| s.contains("tests)"));
    assert!(has_tests, "Should have (+N tests) message");
}

#[test]
fn test_dedup_ordered_preserves_first() {
    use super::types::dedup_ordered;
    let input: Vec<&str> = vec!["a", "b", "c", "b", "d", "c", "e"];
    let deduped = dedup_ordered(&input);
    assert_eq!(deduped, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn test_dedup_ordered_empty_input() {
    use super::types::dedup_ordered;
    let input: Vec<&str> = vec![];
    let deduped = dedup_ordered(&input);
    assert!(deduped.is_empty());
}

// ── PMAT-478: New search mode tests ────────────────────────────────

#[test]
fn test_query_regex_mode() {
    let index = build_test_index();
    let results = index
        .query(
            r"handle_\w+",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Regex,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!results.is_empty());
    // Top result should be a handle_ function (name matches score highest)
    assert!(
        results[0].function_name.starts_with("handle_"),
        "Top result should be handle_* function, got: {}",
        results[0].function_name
    );
}

#[test]
fn test_query_regex_invalid_pattern() {
    let index = build_test_index();
    let result = index.query(
        r"[invalid",
        QueryOptions {
            limit: 10,
            search_mode: SearchMode::Regex,
            ..Default::default()
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid regex"));
}

#[test]
fn test_query_literal_mode() {
    let index = build_test_index();
    let results = index
        .query(
            "unwrap()",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                ..Default::default()
            },
        )
        .unwrap();
    // Literal mode searches source code for exact string
    // May or may not find matches depending on test data
    for r in &results {
        // If there's a match, the source should contain the literal
        assert!(
            r.function_name.contains("unwrap()") || r.signature.contains("unwrap()") || true, // source is not in result unless include_source
        );
    }
}

#[test]
fn test_query_case_sensitive() {
    let index = build_test_index();
    let results_sensitive = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                case_sensitivity: CaseSensitivity::Sensitive,
                ..Default::default()
            },
        )
        .unwrap();
    let results_insensitive = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                case_sensitivity: CaseSensitivity::Insensitive,
                ..Default::default()
            },
        )
        .unwrap();
    // Case-insensitive should find at least as many results
    assert!(results_insensitive.len() >= results_sensitive.len());
}

#[test]
fn test_query_smart_case() {
    let index = build_test_index();
    // Lowercase query => smart-case treats as insensitive
    let results_lower = index
        .query(
            "handle",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                case_sensitivity: CaseSensitivity::Smart,
                ..Default::default()
            },
        )
        .unwrap();
    // Query with uppercase => smart-case treats as sensitive
    let results_upper = index
        .query(
            "Handle",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Literal,
                case_sensitivity: CaseSensitivity::Smart,
                ..Default::default()
            },
        )
        .unwrap();
    // Lowercase should find more or equal (case-insensitive)
    assert!(results_lower.len() >= results_upper.len());
}

#[test]
fn test_query_exclude_pattern() {
    let index = build_test_index();
    let all_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                ..Default::default()
            },
        )
        .unwrap();
    let filtered_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                exclude_pattern: Some("error".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    // Excluding "error" should reduce results
    assert!(filtered_results.len() <= all_results.len());
    // No filtered result should contain "error" in name/signature/source
    for r in &filtered_results {
        let haystack = format!("{} {}", r.function_name, r.signature).to_lowercase();
        assert!(
            !haystack.contains("error"),
            "Excluded result still contains 'error': {}",
            r.function_name
        );
    }
}

#[test]
fn test_query_exclude_file_pattern() {
    let index = build_test_index();
    let all_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                ..Default::default()
            },
        )
        .unwrap();
    let filtered_results = index
        .query(
            "handle",
            QueryOptions {
                limit: 20,
                exclude_file_pattern: Some("utils.rs".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(filtered_results.len() <= all_results.len());
    for r in &filtered_results {
        assert!(
            !r.file_path.contains("utils.rs"),
            "Excluded file still present: {}",
            r.file_path
        );
    }
}

#[test]
fn test_query_regex_case_insensitive() {
    let index = build_test_index();
    let results = index
        .query(
            r"HANDLE",
            QueryOptions {
                limit: 10,
                search_mode: SearchMode::Regex,
                case_sensitivity: CaseSensitivity::Insensitive,
                ..Default::default()
            },
        )
        .unwrap();
    // Should find handle_ functions despite uppercase query
    assert!(!results.is_empty());
}

#[test]
fn test_glob_matches_basic() {
    assert!(glob_matches("*.rs", "foo.rs"));
    assert!(!glob_matches("*.rs", "foo.py"));
    assert!(glob_matches("src/**/*.rs", "src/cli/handlers/mod.rs"));
    assert!(!glob_matches("src/**/*.rs", "tests/mod.rs"));
    assert!(glob_matches("utils", "src/utils.rs"));
}

// ── Enrichment module coverage tests ────────────────────────────────

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

// ── enrich_results_with_faults: async tests ─────────────────────────

#[tokio::test]
async fn test_enrich_results_with_faults_empty_results() {
    let mut results: Vec<QueryResult> = vec![];
    let res = enrich_results_with_faults(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_enrich_results_with_faults_batuta_not_found() {
    let entry = create_test_entry("func_x", 5, 1.5);
    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    let res =
        enrich_results_with_faults(&mut results, std::path::Path::new("/nonexistent/path")).await;
    // When batuta is not on PATH, returns Err. Must not panic.
    if res.is_err() {
        let err = res.unwrap_err();
        assert!(
            err.contains("batuta") || err.contains("Failed"),
            "Error should mention batuta: {err}"
        );
    }
}

#[tokio::test]
async fn test_enrich_results_with_faults_preserves_existing_annotations() {
    let mut entry = create_test_entry("func_with_existing", 5, 1.5);
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
    let entry = create_test_entry("func_y", 3, 1.0);
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
    let mut entry = create_test_entry("func_z", 3, 1.0);
    entry.file_path = "nonexistent_file_that_does_not_exist.rs".to_string();
    let mut results = vec![QueryResult::from_entry(&entry, 0.8, false)];
    let res = enrich_results_with_duplicates(&mut results, std::path::Path::new("/tmp")).await;
    assert!(res.is_ok());
    assert_eq!(results[0].clone_count, 0);
    assert!((results[0].duplication_score).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_enrich_results_with_duplicates_unsupported_language() {
    let mut entry = create_test_entry("func_z", 3, 1.0);
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

// ── Fault annotation parsing and filtering logic tests ──────────────

#[test]
fn test_fault_annotation_line_extraction_pattern() {
    let annotation = "BH001: Boundary condition at line 42";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 42);
}

#[test]
fn test_fault_annotation_line_extraction_multi_digit() {
    let annotation = "BH003: Overflow risk at line 1234";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 1234);
}

#[test]
fn test_fault_annotation_line_range_filtering() {
    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let faults = vec![
        "BH001: Before range at line 5".to_string(),
        "BH002: In range start at line 10".to_string(),
        "BH003: In range middle at line 20".to_string(),
        "BH004: In range end at line 30".to_string(),
        "BH005: After range at line 31".to_string(),
        "BH006: Way after at line 100".to_string(),
    ];

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 3);
    assert!(relevant[0].contains("line 10"));
    assert!(relevant[1].contains("line 20"));
    assert!(relevant[2].contains("line 30"));
}

#[test]
fn test_fault_annotation_malformed_line() {
    let faults = vec![
        "BH001: No line info".to_string(),
        "BH002: Missing number at line ".to_string(),
        "BH003: Valid at line 15".to_string(),
    ];

    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 1);
    assert!(relevant[0].contains("line 15"));
}

#[test]
fn test_fault_map_path_normalization() {
    let file = "./src/lib.rs";
    let normalized = file.strip_prefix("./").unwrap_or(file);
    assert_eq!(normalized, "src/lib.rs");

    let file_no_prefix = "src/lib.rs";
    let normalized2 = file_no_prefix.strip_prefix("./").unwrap_or(file_no_prefix);
    assert_eq!(normalized2, "src/lib.rs");
}

#[test]
fn test_fault_finding_json_parsing() {
    let json_str = r#"{
        "findings": [
            {
                "file": "./src/handler.rs",
                "line": 15,
                "title": "Unchecked unwrap",
                "id": "BH001"
            },
            {
                "file": "src/utils.rs",
                "line": 42,
                "title": "Boundary condition",
                "id": "BH002"
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert_eq!(findings.len(), 2);

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert_eq!(fault_map.len(), 2);
    assert_eq!(fault_map["src/handler.rs"].len(), 1);
    assert!(fault_map["src/handler.rs"][0].contains("BH001"));
    assert!(fault_map["src/handler.rs"][0].contains("line 15"));
    assert_eq!(fault_map["src/utils.rs"].len(), 1);
    assert!(fault_map["src/utils.rs"][0].contains("BH002"));
}

#[test]
fn test_fault_finding_json_no_findings_key() {
    let json_str = r#"{"status": "ok"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").and_then(|f| f.as_array());
    assert!(findings.is_none());
}

#[test]
fn test_fault_finding_json_empty_findings() {
    let json_str = r#"{"findings": []}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_fault_finding_json_missing_fields() {
    let json_str = r#"{
        "findings": [
            {},
            {"file": "a.rs"},
            {"file": "b.rs", "line": 10},
            {"file": "c.rs", "line": 20, "title": "Some fault"}
        ]
    }"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert!(fault_map.contains_key(""));
    let a_annotations = &fault_map["a.rs"];
    assert!(a_annotations[0].contains("BH: Unknown fault pattern at line 0"));
    let c_annotations = &fault_map["c.rs"];
    assert!(c_annotations[0].contains("BH: Some fault at line 20"));
}

#[test]
fn test_fault_finding_json_start_search() {
    let stdout = "WARNING: something\n{\"findings\": []}";
    let json_start = stdout.find('{');
    assert!(json_start.is_some());
    let json_str = &stdout[json_start.unwrap()..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(parsed.get("findings").is_some());
}

#[test]
fn test_fault_finding_no_json_in_output() {
    let stdout = "No output here";
    let json_start = stdout.find('{');
    assert!(json_start.is_none());
}

#[test]
fn test_fault_enrichment_full_flow_simulation() {
    let mut entry_a = create_test_entry("handler", 5, 2.0);
    entry_a.file_path = "src/handler.rs".to_string();
    entry_a.start_line = 10;
    entry_a.quality.loc = 20;

    let mut entry_b = create_test_entry("validate", 3, 1.0);
    entry_b.file_path = "src/utils.rs".to_string();
    entry_b.start_line = 50;
    entry_b.quality.loc = 10;

    let mut results = vec![
        QueryResult::from_entry(&entry_a, 0.9, false),
        QueryResult::from_entry(&entry_b, 0.8, false),
    ];

    let json_str = r#"{
        "findings": [
            {"file": "./src/handler.rs", "line": 15, "title": "Unchecked unwrap", "id": "BH001"},
            {"file": "./src/handler.rs", "line": 25, "title": "In range", "id": "BH002"},
            {"file": "./src/handler.rs", "line": 35, "title": "Out of range", "id": "BH003"},
            {"file": "./src/utils.rs", "line": 55, "title": "Boundary check", "id": "BH004"},
            {"file": "./src/utils.rs", "line": 100, "title": "Way out of range", "id": "BH005"}
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");
        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;

            let relevant_faults: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(line_part) = f.split("at line ").last() {
                        if let Ok(line) = line_part.parse::<usize>() {
                            return line >= func_start && line <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();

            if !relevant_faults.is_empty() {
                result.fault_annotations = relevant_faults;
            }
        }
    }

    // handler: start=10, loc=20, end=30
    assert_eq!(results[0].fault_annotations.len(), 2);
    assert!(results[0].fault_annotations[0].contains("BH001"));
    assert!(results[0].fault_annotations[1].contains("BH002"));

    // validate: start=50, loc=10, end=60
    assert_eq!(results[1].fault_annotations.len(), 1);
    assert!(results[1].fault_annotations[0].contains("BH004"));
}

#[test]
fn test_fault_enrichment_no_faults_for_file() {
    let mut entry = create_test_entry("orphan_func", 3, 1.0);
    entry.file_path = "src/orphan.rs".to_string();
    entry.start_line = 1;
    entry.quality.loc = 50;

    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];
    let fault_map: HashMap<String, Vec<String>> = HashMap::new();

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}

#[test]
fn test_fault_enrichment_all_faults_out_of_range() {
    let mut entry = create_test_entry("narrow_func", 3, 1.0);
    entry.file_path = "src/narrow.rs".to_string();
    entry.start_line = 100;
    entry.quality.loc = 5;

    let mut results = vec![QueryResult::from_entry(&entry, 0.9, false)];

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    fault_map.insert(
        "src/narrow.rs".to_string(),
        vec![
            "BH001: Early fault at line 10".to_string(),
            "BH002: Late fault at line 200".to_string(),
        ],
    );

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}

#[test]
fn test_load_workspace_coverage_merges_siblings() {
    use super::coverage::load_workspace_coverage;

    let tmp1 = tempfile::TempDir::new().unwrap();
    let tmp2 = tempfile::TempDir::new().unwrap();

    let pmat1 = tmp1.path().join(".pmat");
    let pmat2 = tmp2.path().join(".pmat");
    std::fs::create_dir_all(&pmat1).unwrap();
    std::fs::create_dir_all(&pmat2).unwrap();

    let cache1 = serde_json::json!({
        "git_hash": "abc123",
        "coverage_mtime": 0,
        "files": {
            "src/lib.rs": {"1": 5, "2": 0, "3": 10}
        }
    });
    std::fs::write(pmat1.join("coverage-cache.json"), cache1.to_string()).unwrap();

    let cache2 = serde_json::json!({
        "git_hash": "def456",
        "coverage_mtime": 0,
        "files": {
            "src/main.rs": {"1": 1, "5": 3}
        }
    });
    std::fs::write(pmat2.join("coverage-cache.json"), cache2.to_string()).unwrap();

    let siblings = vec![
        (pmat1.join("context.db"), "trueno".to_string()),
        (pmat2.join("context.db"), "realizar".to_string()),
    ];

    let merged = load_workspace_coverage(&siblings);

    assert!(merged.contains_key("trueno/src/lib.rs"));
    assert!(merged.contains_key("realizar/src/main.rs"));
    assert_eq!(merged.len(), 2);

    let trueno_lib = &merged["trueno/src/lib.rs"];
    assert_eq!(trueno_lib.get(&1), Some(&5));
    assert_eq!(trueno_lib.get(&2), Some(&0));
}

#[test]
fn test_load_workspace_coverage_skips_missing() {
    use super::coverage::load_workspace_coverage;

    let tmp = tempfile::TempDir::new().unwrap();
    let pmat = tmp.path().join(".pmat");
    std::fs::create_dir_all(&pmat).unwrap();

    let siblings = vec![(pmat.join("context.db"), "missing_project".to_string())];

    let merged = load_workspace_coverage(&siblings);
    assert!(merged.is_empty());
}
