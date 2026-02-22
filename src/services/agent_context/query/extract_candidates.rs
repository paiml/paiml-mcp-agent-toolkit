#![cfg_attr(coverage_nightly, coverage(off))]
//! Extract Candidates Analysis (Issue #235)
//!
//! Scans function source for I/O patterns, groups by name prefix / call graph
//! clusters, and suggests module extractions for large files.

use super::types::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── I/O Pattern Categories ──────────────────────────────────────────────────

/// I/O pattern categories with their detection strings
const IO_PATTERNS: &[(&str, &[&str])] = &[
    ("PRINT", &["println!", "print!"]),
    ("EPRINT", &["eprintln!", "eprint!"]),
    ("WRITE", &["write!", "writeln!"]),
    (
        "FS",
        &["std::fs::", "File::open", "File::create", "OpenOptions"],
    ),
    ("PROCESS", &["std::process::Command", "Command::new"]),
    ("STDIN", &["std::io::stdin"]),
    ("STDOUT", &["stdout()"]),
    ("STDERR", &["stderr()"]),
    ("HTTP", &["reqwest::", "hyper::"]),
    ("NET", &["tokio::net::", "TcpStream", "UdpSocket"]),
    ("DB", &["sqlx::", "rusqlite::", "Connection::open"]),
];

/// Classify a function's source code as PURE or IO.
///
/// Returns the classification string and a list of detected I/O pattern labels.
pub(crate) fn classify_io(source: &str) -> (String, Vec<String>) {
    let mut patterns = Vec::new();
    for (label, markers) in IO_PATTERNS {
        if markers.iter().any(|m| source.contains(m)) {
            patterns.push(label.to_string());
        }
    }
    if patterns.is_empty() {
        ("PURE".to_string(), patterns)
    } else {
        ("IO".to_string(), patterns)
    }
}

/// Classify all results in-place, updating io_classification and io_patterns.
pub(crate) fn classify_all_results(results: &mut [QueryResult]) {
    for r in results.iter_mut() {
        if let Some(ref source) = r.source {
            let (class, patterns) = classify_io(source);
            r.io_classification = class;
            r.io_patterns = patterns;
        } else {
            r.io_classification = "PURE".to_string();
        }
    }
}

// ── Grouping ────────────────────────────────────────────────────────────────

/// Extract the name prefix before the first `_` (must be > 2 chars).
fn extract_prefix(name: &str) -> Option<&str> {
    let prefix = name.split('_').next()?;
    if prefix.len() > 2 {
        Some(prefix)
    } else {
        None
    }
}

/// Group functions by name prefix (requires 3+ members per group, functions only).
pub(crate) fn group_by_prefix(results: &[QueryResult]) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in results.iter().enumerate() {
        if r.definition_type != "function" {
            continue;
        }
        if let Some(prefix) = extract_prefix(&r.function_name) {
            groups.entry(prefix.to_string()).or_default().push(i);
        }
    }
    // Only keep groups with 3+ members
    groups.retain(|_, indices| indices.len() >= 3);
    groups
}

/// Group co-located functions that call each other (3+ members).
///
/// Two functions are in the same cluster if they are in the same file
/// and one calls the other (or they share callees).
pub(crate) fn group_by_call_cluster(results: &[QueryResult]) -> HashMap<String, Vec<usize>> {
    let mut by_file: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, r) in results.iter().enumerate() {
        if r.definition_type == "function" {
            by_file.entry(&r.file_path).or_default().push(i);
        }
    }

    let mut clusters: HashMap<String, Vec<usize>> = HashMap::new();
    for (file, indices) in &by_file {
        if indices.len() < 3 {
            continue;
        }
        let names: HashMap<&str, usize> = indices
            .iter()
            .map(|&i| (results[i].function_name.as_str(), i))
            .collect();

        let mut visited = vec![false; indices.len()];
        for (local_idx, &global_idx) in indices.iter().enumerate() {
            if visited[local_idx] {
                continue;
            }
            visited[local_idx] = true;
            let mut cluster = vec![global_idx];
            collect_neighbors(
                &results[global_idx].calls,
                global_idx,
                &names,
                indices,
                &mut visited,
                &mut cluster,
            );
            collect_neighbors(
                &results[global_idx].called_by,
                global_idx,
                &names,
                indices,
                &mut visited,
                &mut cluster,
            );
            if cluster.len() >= 3 {
                let key = format!("{}::cluster_{}", file, results[global_idx].function_name);
                clusters.insert(key, cluster);
            }
        }
    }
    clusters
}

/// Collect unvisited neighbors from a call/caller list into the cluster.
fn collect_neighbors(
    edges: &[String],
    origin: usize,
    names: &HashMap<&str, usize>,
    indices: &[usize],
    visited: &mut [bool],
    cluster: &mut Vec<usize>,
) {
    for edge in edges {
        let Some(&global) = names.get(edge.as_str()) else {
            continue;
        };
        if global == origin {
            continue;
        }
        let local = indices.iter().position(|&i| i == global).unwrap_or(0);
        if !visited[local] {
            visited[local] = true;
            cluster.push(global);
        }
    }
}

/// Find the longest common prefix among a set of strings, trimmed to underscore boundary.
#[allow(dead_code)]
pub(crate) fn longest_common_prefix(names: &[&str]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let first = names[0];
    let len = names[1..]
        .iter()
        .fold(first.len(), |acc, name| common_prefix_len(first, name, acc));
    let prefix = &first[..len];
    match prefix.rfind('_') {
        Some(pos) => first[..pos].to_string(),
        None => prefix.to_string(),
    }
}

fn common_prefix_len(a: &str, b: &str, max: usize) -> usize {
    let limit = max.min(b.len());
    a.bytes()
        .zip(b.bytes())
        .take(limit)
        .position(|(x, y)| x != y)
        .unwrap_or(limit)
}

// ── Extraction Output Types ─────────────────────────────────────────────────

/// A candidate function for extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExtractionCandidate {
    pub function_name: String,
    pub file_path: String,
    pub start_line: usize,
    pub loc: u32,
    pub io_classification: String,
    pub io_patterns: Vec<String>,
    pub complexity: u32,
    pub tdg_grade: String,
}

/// A group of functions suggested for extraction into a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExtractionGroup {
    pub module_name: String,
    pub source_file: String,
    pub functions: Vec<ExtractionCandidate>,
    pub total_loc: u32,
    pub pure_count: usize,
    pub io_count: usize,
    pub grouping_signal: String,
}

/// Build extraction groups from prefix and cluster groupings.
///
/// Merges both grouping signals, respects `max_module_lines`, and produces
/// sorted output (largest groups first).
pub(crate) fn build_extraction_groups(
    results: &[QueryResult],
    prefix_groups: &HashMap<String, Vec<usize>>,
    cluster_groups: &HashMap<String, Vec<usize>>,
    max_module_lines: usize,
) -> Vec<ExtractionGroup> {
    let mut groups = Vec::new();

    // Process prefix groups
    for (prefix, indices) in prefix_groups {
        let group = build_group(results, indices, prefix, "prefix", max_module_lines);
        if let Some(g) = group {
            groups.push(g);
        }
    }

    // Process cluster groups (skip if already covered by prefix)
    let prefix_indices: std::collections::HashSet<usize> = prefix_groups
        .values()
        .flat_map(|v| v.iter().copied())
        .collect();

    for (cluster_name, indices) in cluster_groups {
        // Skip if most members are already in a prefix group
        let overlap = indices
            .iter()
            .filter(|i| prefix_indices.contains(i))
            .count();
        if overlap > indices.len() / 2 {
            continue;
        }

        let module_name = cluster_name
            .rsplit("::")
            .next()
            .unwrap_or(cluster_name)
            .to_string();
        let group = build_group(
            results,
            indices,
            &module_name,
            "call_cluster",
            max_module_lines,
        );
        if let Some(g) = group {
            groups.push(g);
        }
    }

    // Sort by total LOC descending (biggest extraction targets first)
    groups.sort_by(|a, b| b.total_loc.cmp(&a.total_loc));
    groups
}

fn build_group(
    results: &[QueryResult],
    indices: &[usize],
    name: &str,
    signal: &str,
    max_module_lines: usize,
) -> Option<ExtractionGroup> {
    let mut candidates: Vec<ExtractionCandidate> = indices
        .iter()
        .filter_map(|&i| results.get(i))
        .map(|r| ExtractionCandidate {
            function_name: r.function_name.clone(),
            file_path: r.file_path.clone(),
            start_line: r.start_line,
            loc: r.loc,
            io_classification: r.io_classification.clone(),
            io_patterns: r.io_patterns.clone(),
            complexity: r.complexity,
            tdg_grade: r.tdg_grade.clone(),
        })
        .collect();

    let total_loc: u32 = candidates.iter().map(|c| c.loc).sum();
    if total_loc as usize > max_module_lines {
        // Trim to fit within max_module_lines, keeping highest-LOC functions
        candidates.sort_by(|a, b| b.loc.cmp(&a.loc));
        let mut running = 0u32;
        candidates.retain(|c| {
            running += c.loc;
            (running as usize) <= max_module_lines
        });
    }

    if candidates.len() < 3 {
        return None;
    }

    let pure_count = candidates
        .iter()
        .filter(|c| c.io_classification == "PURE")
        .count();
    let io_count = candidates.len() - pure_count;
    let total_loc: u32 = candidates.iter().map(|c| c.loc).sum();

    let source_file = candidates
        .first()
        .map(|c| c.file_path.clone())
        .unwrap_or_default();

    // Sort candidates by start_line for readable output
    candidates.sort_by_key(|c| c.start_line);

    Some(ExtractionGroup {
        module_name: name.to_string(),
        source_file,
        functions: candidates,
        total_loc,
        pure_count,
        io_count,
        grouping_signal: signal.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, def_type: &str, source: &str, file: &str) -> QueryResult {
        QueryResult {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {}()", name),
            definition_type: def_type.to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            tdg_score: 0.5,
            tdg_grade: "C".to_string(),
            complexity: 5,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.0,
            source: Some(source.to_string()),
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
            coverage_exclusion:
                crate::services::agent_context::query::coverage_exclusion::CoverageExclusion::None,
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
        }
    }

    #[test]
    fn test_classify_io_pure_function() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "PURE");
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_classify_io_print() {
        let source = r#"fn greet() { println!("hello"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"PRINT".to_string()));
    }

    #[test]
    fn test_classify_io_filesystem() {
        let source = r#"fn read_file() { let f = File::open("test.txt"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"FS".to_string()));
    }

    #[test]
    fn test_classify_io_multiple_patterns() {
        let source =
            r#"fn do_stuff() { println!("hi"); let f = File::open("x"); Command::new("ls"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"PRINT".to_string()));
        assert!(patterns.contains(&"FS".to_string()));
        assert!(patterns.contains(&"PROCESS".to_string()));
    }

    #[test]
    fn test_classify_io_database() {
        let source = r#"fn query_db() { rusqlite::Connection::open("db"); }"#;
        let (class, patterns) = classify_io(source);
        assert_eq!(class, "IO");
        assert!(patterns.contains(&"DB".to_string()));
    }

    #[test]
    fn test_group_by_prefix_minimum_three() {
        let results = vec![
            make_result("handle_get", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/handlers.rs"),
        ];
        let groups = group_by_prefix(&results);
        // Only 2 members, should be empty
        assert!(groups.is_empty());

        let results3 = vec![
            make_result("handle_get", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/handlers.rs"),
            make_result("handle_delete", "function", "fn x() {}", "src/handlers.rs"),
        ];
        let groups3 = group_by_prefix(&results3);
        assert!(groups3.contains_key("handle"));
        assert_eq!(groups3["handle"].len(), 3);
    }

    #[test]
    fn test_group_by_prefix_ignores_non_functions() {
        let results = vec![
            make_result("handle_get", "function", "fn x() {}", "src/lib.rs"),
            make_result("handle_post", "function", "fn x() {}", "src/lib.rs"),
            make_result("handle_delete", "function", "fn x() {}", "src/lib.rs"),
            make_result(
                "HandleConfig",
                "struct",
                "struct HandleConfig {}",
                "src/lib.rs",
            ),
        ];
        let groups = group_by_prefix(&results);
        // HandleConfig is a struct, should not be in the "Handle" group
        // But "handle" prefix group should have 3 functions
        assert!(groups.contains_key("handle"));
        assert_eq!(groups["handle"].len(), 3);
    }

    #[test]
    fn test_build_extraction_groups_max_lines() {
        let mut results = vec![
            make_result("parse_header", "function", "fn x() {}", "src/parser.rs"),
            make_result("parse_body", "function", "fn x() {}", "src/parser.rs"),
            make_result("parse_footer", "function", "fn x() {}", "src/parser.rs"),
        ];
        // Set LOC to 200 each = 600 total
        for r in &mut results {
            r.loc = 200;
        }
        classify_all_results(&mut results);

        let prefix_groups = group_by_prefix(&results);
        let cluster_groups = HashMap::new();

        // With max_module_lines=500, should trim
        let groups = build_extraction_groups(&results, &prefix_groups, &cluster_groups, 500);
        // Group should exist but be trimmed to fit within 500 lines
        // 200 + 200 = 400 <= 500, so 2 functions fit, but < 3 means group is dropped
        assert!(groups.is_empty() || groups[0].total_loc <= 500);

        // With max_module_lines=700, all 3 fit
        let groups = build_extraction_groups(&results, &prefix_groups, &cluster_groups, 700);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].functions.len(), 3);
    }

    #[test]
    fn test_longest_common_prefix() {
        assert_eq!(
            longest_common_prefix(&["handle_get", "handle_post", "handle_delete"]),
            "handle"
        );
        assert_eq!(
            longest_common_prefix(&["parse_header", "parse_body"]),
            "parse"
        );
        assert_eq!(longest_common_prefix(&["abc", "def"]), "");
        assert_eq!(longest_common_prefix(&[]), "");
    }
}
