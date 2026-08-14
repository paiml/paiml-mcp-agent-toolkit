//! `path_pattern` glob-filter regression tests.
//!
//! In its own file, not at the end of `engine_scoring.rs`: that file is
//! `include!`d into `engine.rs` ahead of `engine_search.rs`, so a test module
//! at its end puts items after a test module in the expanded source
//! (`clippy::items_after_test_module`, denied by `ci / lint`).

//! `pmat_query_code`'s schema advertises `path_pattern` as a "Path glob
//! pattern filter", but the filter was `file_path.contains(pattern)`: on the
//! real index `src/tdg` returned five hits while `src/tdg/*`, `**/tdg/**`
//! and `*tdg*` all returned zero — a silent empty set, not an error.
use super::*;
use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{IndexManifest, QualityMetrics};
use std::collections::HashMap;

fn entry(file_path: &str) -> FunctionEntry {
    FunctionEntry {
        file_path: file_path.to_string(),
        function_name: "f".to_string(),
        signature: "fn f()".to_string(),
        definition_type: DefinitionType::default(),
        doc_comment: None,
        source: "fn f() {}".to_string(),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics {
            tdg_score: 1.0,
            tdg_grade: "A".to_string(),
            complexity: 1,
            cognitive_complexity: 1,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 1,
            commit_count: 0,
            churn_score: 0.0,
            contract_level: None,
            contract_equation: None,
        },
        checksum: "chk".to_string(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
        linked_definition: None,
    }
}

/// `passes_filters` reads only `self.functions[idx]`, so the rest of the
/// index can stay empty.
fn index_of(paths: &[&str]) -> AgentContextIndex {
    AgentContextIndex {
        functions: paths.iter().map(|p| entry(p)).collect(),
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: Vec::new(),
        corpus_lower: Vec::new(),
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: Vec::new(),
        project_root: std::path::PathBuf::from("/test"),
        manifest: IndexManifest {
            version: "1.2.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: paths.len(),
            file_count: paths.len(),
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 1.0,
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    }
}

fn kept(paths: &[&str], pattern: &str) -> Vec<String> {
    let index = index_of(paths);
    let options = QueryOptions {
        path_pattern: Some(pattern.to_string()),
        ..QueryOptions::default()
    };
    (0..paths.len())
        .filter(|i| index.passes_filters(*i, &options))
        .map(|i| paths[i].to_string())
        .collect()
}

const PATHS: [&str; 3] = [
    "src/tdg/scorers/consistency.rs",
    "src/tdg/mod.rs",
    "src/cli/handlers/score_handler.rs",
];

#[test]
fn test_globstar_pattern_is_not_an_empty_set() {
    // Under the substring filter this matched nothing at all.
    assert_eq!(
        kept(&PATHS, "**/tdg/**"),
        vec![
            "src/tdg/scorers/consistency.rs".to_string(),
            "src/tdg/mod.rs".to_string()
        ]
    );
}

#[test]
fn test_single_star_does_not_cross_a_path_separator() {
    // Glob semantics: `src/tdg/*` is the files directly under src/tdg.
    assert_eq!(
        kept(&PATHS, "src/tdg/*"),
        vec!["src/tdg/mod.rs".to_string()]
    );
}

#[test]
fn test_plain_prefix_still_behaves_as_a_substring() {
    // A pattern with no metacharacter must keep matching the way callers
    // relied on before the glob support went in.
    assert_eq!(
        kept(&PATHS, "src/tdg"),
        vec![
            "src/tdg/scorers/consistency.rs".to_string(),
            "src/tdg/mod.rs".to_string()
        ]
    );
}
