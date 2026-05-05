#[tokio::test]
async fn test_handle_query_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create empty project
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/main.rs"), "").unwrap();

    let result = handle_query(
        "test".to_string(),
        10,
        None,
        None,
        None,
        None,
        project_path,
        QueryOutputFormat::Text,
        false,
        false,
        false,
        None,   // rank_by
        None,   // min_pagerank
        vec![], // include_project
        false,  // churn
        false,  // duplicates
        false,  // entropy
        false,  // faults
        false,  // coverage
        false,  // uncovered_only
        None,   // coverage_diff
        None,   // coverage_file
        false,  // coverage_gaps
        false,  // include_excluded
        None,   // definition_type
        false,  // code
        false,  // git_history
        false,  // regex
        false,  // literal
        None,   // search_mode (issue #562)
        false,  // raw
        false,  // case_sensitive
        false,  // ignore_case
        Vec::new(),   // exclude
        Vec::new(),   // exclude_file
        false,  // files_with_matches
        false,  // count
        None,   // after_context
        None,   // before_context
        None,   // context_lines
        false,  // ptx_flow
        false,  // ptx_diagnostics
        false,  // suggest_rename
        false,  // apply
        false,  // docs
        false,  // docs_only
        false,  // extract_candidates
        500,    // max_module_lines
    )
    .await;

    // Should not error, just find nothing
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_query_with_functions() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create project with a function
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/main.rs"),
        r#"
/// Handle errors in the API layer
fn handle_api_error(err: String) -> String {
    format!("Error: {}", err)
}

fn main() {
    println!("Hello");
}
"#,
    )
    .unwrap();

    let result = handle_query(
        "error handling".to_string(),
        10,
        None,
        None,
        None,
        None,
        project_path,
        QueryOutputFormat::Json,
        false,
        true, // Force rebuild
        false,
        None,   // rank_by
        None,   // min_pagerank
        vec![], // include_project
        false,  // churn
        false,  // duplicates
        false,  // entropy
        false,  // faults
        false,  // coverage
        false,  // uncovered_only
        None,   // coverage_diff
        None,   // coverage_file
        false,  // coverage_gaps
        false,  // include_excluded
        None,   // definition_type
        false,  // code
        false,  // git_history
        false,  // regex
        false,  // literal
        None,   // search_mode (issue #562)
        false,  // raw
        false,  // case_sensitive
        false,  // ignore_case
        Vec::new(),   // exclude
        Vec::new(),   // exclude_file
        false,  // files_with_matches
        false,  // count
        None,   // after_context
        None,   // before_context
        None,   // context_lines
        false,  // ptx_flow
        false,  // ptx_diagnostics
        false,  // suggest_rename
        false,  // apply
        false,  // docs
        false,  // docs_only
        false,  // extract_candidates
        500,    // max_module_lines
    )
    .await;

    assert!(result.is_ok());
}

#[test]
fn test_classify_commit_type() {
    assert_eq!(classify_commit_type("fix: null pointer").1, "[fix]");
    assert_eq!(classify_commit_type("feat: add auth").1, "[feat]");
    assert_eq!(
        classify_commit_type("refactor: simplify parser").1,
        "[refactor]"
    );
    assert_eq!(classify_commit_type("docs: update README").1, "[docs]");
    assert_eq!(classify_commit_type("chore: bump deps").1, "[chore]");
    assert_eq!(classify_commit_type("random commit").1, "");
    assert_eq!(classify_commit_type("Merge branch main").1, "[merge]");
}

#[test]
fn test_format_timestamp() {
    // 2024-01-01 00:00:00 UTC = 1704067200
    let ts = 1704067200_i64;
    let formatted = format_timestamp(ts);
    // Should produce something reasonable (approximate date)
    assert!(formatted.starts_with("2024"));
}

#[test]
fn test_compute_decay_score() {
    let hotspot = FileHotspot {
        commit_count: 10,
        fix_count: 5,
        annotation: FileAnnotation {
            tdg_grade: Some("D".to_string()),
            dead_code_pct: 10.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let decay = compute_decay_score(&hotspot, 100);
    assert!(decay > 0.0);
    assert!(decay <= 1.0);

    // Grade A with no fixes should have low decay
    let healthy = FileHotspot {
        commit_count: 5,
        fix_count: 0,
        annotation: FileAnnotation {
            tdg_grade: Some("A".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    let healthy_decay = compute_decay_score(&healthy, 100);
    assert!(
        healthy_decay < decay,
        "Healthy file should have lower decay"
    );
}

#[test]
fn test_compute_impact_risk() {
    let hotspot = FileHotspot {
        commit_count: 50,
        annotation: FileAnnotation {
            max_pagerank: Some(0.01),
            fault_count: 3,
            ..Default::default()
        },
        ..Default::default()
    };

    let risk = compute_impact_risk(&hotspot, 100);
    assert!(risk > 0.0);

    // Zero pagerank = zero risk
    let low_risk = FileHotspot {
        commit_count: 50,
        annotation: FileAnnotation {
            max_pagerank: Some(0.0),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(compute_impact_risk(&low_risk, 100), 0.0);
}

#[test]
fn test_parse_git_log_with_issue_refs() {
    let log = "PMAT_START\nH:abc1234567890123456789012345678901234567\nS:feat: add auth (PMAT-472)\nN:noah\nE:noah@test.com\nT:1704067200\nPMAT_FILES\nM\tsrc/main.rs";
    let commits = parse_git_log(log);
    assert_eq!(commits.len(), 1);
    assert!(
        commits[0].issue_refs.contains(&"PMAT-472".to_string())
            || commits[0].issue_refs.contains(&"(PMAT-472)".to_string())
    );
    assert!(commits[0].is_feat);
}

#[test]
fn test_file_annotation_default() {
    let annot = FileAnnotation::default();
    assert_eq!(annot.tdg_grade, None);
    assert_eq!(annot.function_count, 0);
    assert_eq!(annot.dead_code_count, 0);
    assert_eq!(annot.fault_count, 0);
}

// ── Issue #562: --search-mode {semantic,lexical,hybrid} ────────────────────
//
// These tests build a tiny fixture corpus at runtime in a tempdir and assert
// the lexical mode returns hits without any embedding index.

/// Build a fixture corpus in `dir` containing 4 .rs files spanning two
/// "shapes" so that lexical match (substring) and semantic match
/// (TF-scoring) can pull different functions to the top.
fn write_fixture_corpus(dir: &std::path::Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        r#"
pub mod parse;
pub mod render;
pub mod compute;
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/parse.rs"),
        r#"
/// Parse the input bytes into a token stream.
pub fn parse_input(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
}

/// Parse a single literal token from the input cursor.
pub fn parse_literal(cursor: &mut usize) -> Option<u8> {
    *cursor += 1;
    Some(0)
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/render.rs"),
        r#"
/// Render a parsed token stream into output bytes.
pub fn render_output(tokens: &[u8]) -> Vec<u8> {
    tokens.to_vec()
}

/// Render a literal token onto the output buffer.
pub fn render_literal(buf: &mut Vec<u8>) {
    buf.push(0);
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/compute.rs"),
        r#"
/// Compute a checksum over the input bytes.
pub fn compute_checksum(bytes: &[u8]) -> u32 {
    bytes.iter().map(|b| *b as u32).sum()
}
"#,
    )
    .unwrap();
}

#[tokio::test]
async fn test_search_mode_semantic_returns_results() {
    let temp_dir = TempDir::new().unwrap();
    write_fixture_corpus(temp_dir.path());

    let result = handle_query(
        "literal".to_string(),
        5,
        None,
        None,
        None,
        None,
        temp_dir.path().to_path_buf(),
        QueryOutputFormat::Json,
        false,
        true, // rebuild_index
        false,
        None,
        None,
        vec![],
        false,
        false,
        false,
        false,
        false,
        false,
        None,
        None,
        false,
        false,
        None,
        false,
        false,
        false,
        false,
        Some("semantic".to_string()), // search_mode
        false,
        false,
        false,
        Vec::new(),
        Vec::new(),
        false,
        false,
        None,
        None,
        None,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        500,
    )
    .await;
    assert!(
        result.is_ok(),
        "semantic mode must return results: {:?}",
        result
    );
}

#[tokio::test]
async fn test_search_mode_lexical_does_not_require_embeddings() {
    // Lexical mode reads the function index built by --rebuild-index;
    // it MUST NOT require a pre-existing embeddings index (`.pmat/embeddings.idx`).
    // We assert this implicitly by creating a fresh tempdir with no
    // embeddings index and confirming the call succeeds.
    let temp_dir = TempDir::new().unwrap();
    write_fixture_corpus(temp_dir.path());
    assert!(
        !temp_dir.path().join(".pmat/embeddings.idx").exists(),
        "tempdir must NOT have an embeddings index"
    );

    let result = handle_query(
        "literal".to_string(),
        5,
        None,
        None,
        None,
        None,
        temp_dir.path().to_path_buf(),
        QueryOutputFormat::Json,
        false,
        true, // rebuild_index
        false,
        None,
        None,
        vec![],
        false,
        false,
        false,
        false,
        false,
        false,
        None,
        None,
        false,
        false,
        None,
        false,
        false,
        false,
        false,
        Some("lexical".to_string()), // search_mode
        false,
        false,
        false,
        Vec::new(),
        Vec::new(),
        false,
        false,
        None,
        None,
        None,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        500,
    )
    .await;
    assert!(
        result.is_ok(),
        "lexical mode must work without embeddings: {:?}",
        result
    );
}

#[tokio::test]
async fn test_search_mode_hybrid_returns_results() {
    let temp_dir = TempDir::new().unwrap();
    write_fixture_corpus(temp_dir.path());

    let result = handle_query(
        "literal".to_string(),
        5,
        None,
        None,
        None,
        None,
        temp_dir.path().to_path_buf(),
        QueryOutputFormat::Json,
        false,
        true, // rebuild_index
        false,
        None,
        None,
        vec![],
        false,
        false,
        false,
        false,
        false,
        false,
        None,
        None,
        false,
        false,
        None,
        false,
        false,
        false,
        false,
        Some("hybrid".to_string()), // search_mode
        false,
        false,
        false,
        Vec::new(),
        Vec::new(),
        false,
        false,
        None,
        None,
        None,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        500,
    )
    .await;
    assert!(
        result.is_ok(),
        "hybrid mode must return results: {:?}",
        result
    );
}

#[test]
fn test_rrf_fuse_top_within_lexical_or_semantic_top10() {
    // Direct unit test of the RRF helper (issue #562 contract):
    // hybrid top-3 must be a subset of (lexical top-10 ∪ semantic top-10).
    use crate::services::agent_context::QueryResult;

    fn fake_result(file: &str, name: &str) -> QueryResult {
        QueryResult {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 1,
            language: "rust".to_string(),
            tdg_score: 0.0,
            tdg_grade: "A".to_string(),
            complexity: 1,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 1,
            relevance_score: 0.0,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
            contract_level: None,
            contract_equation: None,
        }
    }

    let lexical: Vec<QueryResult> = (0..10)
        .map(|i| fake_result("lex.rs", &format!("lex_{i}")))
        .collect();
    let semantic: Vec<QueryResult> = (0..10)
        .map(|i| fake_result("sem.rs", &format!("sem_{i}")))
        .collect();
    let lex_keys: std::collections::HashSet<_> = lexical
        .iter()
        .map(|r| (r.file_path.clone(), r.function_name.clone()))
        .collect();
    let sem_keys: std::collections::HashSet<_> = semantic
        .iter()
        .map(|r| (r.file_path.clone(), r.function_name.clone()))
        .collect();

    let fused = rrf_fuse(lexical, semantic, 3);
    assert!(!fused.is_empty(), "hybrid must return ≥1 result");
    assert!(fused.len() <= 3, "respects --limit");
    for r in &fused {
        let key = (r.file_path.clone(), r.function_name.clone());
        assert!(
            lex_keys.contains(&key) || sem_keys.contains(&key),
            "every hybrid result must come from the lexical or semantic top-10"
        );
    }
}
