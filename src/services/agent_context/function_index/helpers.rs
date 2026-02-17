#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::*;
use crate::services::semantic::{CodeChunk, Language};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Parse `siblings` array from workspace.toml content.
///
/// Handles: `siblings = ["../aprender", "../trueno"]`
/// Minimal parser — no full TOML dependency needed for one key.
pub(super) fn parse_workspace_siblings(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("siblings") {
            let rest = rest.trim().strip_prefix('=').unwrap_or("").trim();
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                return inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Build a corpus document string for a single function entry.
///
/// Used by find_similar() when corpus was not pre-built (SQLite load path).
pub(crate) fn build_corpus_entry(func: &FunctionEntry) -> String {
    format!(
        "{name} {name} {sig} {sig} {doc} {doc} {path} {idents}",
        name = func.function_name,
        sig = func.signature,
        doc = func.doc_comment.as_deref().unwrap_or(""),
        path = func.file_path,
        idents = extract_identifiers(&func.source)
    )
}

/// Build name_index, file_index, and corpus from functions.
pub(crate) fn build_indices(functions: &[FunctionEntry]) -> BuildIndicesResult {
    build_indices_impl(functions, true)
}

/// Build name_index and file_index only (skip corpus construction).
///
/// Used by SQLite load path where FTS5 handles search, saving ~36MB
/// of corpus string allocation for 90K functions.
pub(crate) fn build_indices_without_corpus(functions: &[FunctionEntry]) -> BuildIndicesResult {
    build_indices_impl(functions, false)
}

fn build_indices_impl(functions: &[FunctionEntry], include_corpus: bool) -> BuildIndicesResult {
    let mut result = BuildIndicesResult {
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: if include_corpus {
            Vec::with_capacity(functions.len())
        } else {
            Vec::new()
        },
    };

    for (idx, func) in functions.iter().enumerate() {
        // Cap name_index entries per name to prevent pathological sizes
        // for common names like "new" (can have 10,000+ entries)
        let name_entries = result
            .name_index
            .entry(func.function_name.clone())
            .or_default();
        if name_entries.len() < 100 {
            name_entries.push(idx);
        }
        result
            .file_index
            .entry(func.file_path.clone())
            .or_default()
            .push(idx);

        if include_corpus {
            result.corpus.push(build_corpus_entry(func));
        }
    }

    result
}

/// Compute SHA256 hash of file content
pub(super) fn compute_file_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Populate cached annotations for all functions during index build.
/// Computes: git churn, code clones, pattern diversity, fault patterns.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git + filesystem for churn/clones
#[allow(clippy::cast_possible_truncation)]
pub(super) fn populate_cached_annotations(
    functions: &mut [FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
    project_root: &std::path::Path,
) {
    eprintln!("Computing annotations for {} functions...", functions.len());

    // 1. Git churn: get commit counts per file
    let file_commits = get_file_commit_counts(project_root, file_index.keys());
    let max_commits = file_commits.values().copied().max().unwrap_or(1) as f32;
    eprintln!(
        "  Git churn: {} files with commits (max={})",
        file_commits.len(),
        max_commits as u32
    );

    // 2. Detect duplicate/similar functions (by normalized source hash)
    let clone_groups = detect_code_clones(functions);
    eprintln!("  Clones: {} functions with duplicates", clone_groups.len());

    // 3. Compute pattern diversity per file
    let file_diversity = compute_file_pattern_diversity(functions, file_index);
    eprintln!("  Diversity: {} files analyzed", file_diversity.len());

    // 4. Detect fault patterns in source code
    let fault_patterns = detect_fault_patterns(functions);
    eprintln!(
        "  Faults: {} functions with patterns",
        fault_patterns.len()
    );

    // Apply annotations to functions
    let mut churn_applied = 0;
    let mut clone_applied = 0;
    let mut diversity_applied = 0;
    let mut fault_applied = 0;

    for (i, func) in functions.iter_mut().enumerate() {
        // Churn data
        if let Some(&commits) = file_commits.get(&func.file_path) {
            func.commit_count = commits;
            func.churn_score = commits as f32 / max_commits;
            churn_applied += 1;
        }

        // Clone count
        if let Some(&count) = clone_groups.get(&i) {
            func.clone_count = count;
            clone_applied += 1;
        }

        // Pattern diversity (from file-level)
        if let Some(&diversity) = file_diversity.get(&func.file_path) {
            func.pattern_diversity = diversity;
            diversity_applied += 1;
        }

        // Fault annotations
        if let Some(faults) = fault_patterns.get(&i) {
            func.fault_annotations = faults.clone();
            fault_applied += 1;
        }
    }

    eprintln!(
        "  Applied: churn={}, clones={}, diversity={}, faults={}",
        churn_applied, clone_applied, diversity_applied, fault_applied
    );
}

/// Match a git log file path against the known file set, handling path migrations.
fn match_git_path(line: &str, files: &std::collections::HashSet<&String>) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Exact match
    if files.contains(&trimmed.to_string()) {
        return Some(trimmed.to_string());
    }
    // Handle path migrations (e.g., server/src/foo.rs -> src/foo.rs)
    let normalized = trimmed.strip_prefix("server/").unwrap_or(trimmed);
    if files.contains(&normalized.to_string()) {
        return Some(normalized.to_string());
    }
    None
}

/// Get commit counts per file from git log
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git process
pub(super) fn get_file_commit_counts<'a>(
    project_root: &std::path::Path,
    files: impl Iterator<Item = &'a String>,
) -> HashMap<String, u32> {
    let files: std::collections::HashSet<_> = files.collect();
    if files.is_empty() {
        return HashMap::new();
    }

    let output = std::process::Command::new("git")
        .args(["log", "--format=", "--name-only", "--since=1 year ago"])
        .current_dir(project_root)
        .output();

    let Ok(output) = output else { return HashMap::new() };
    if !output.status.success() {
        return HashMap::new();
    }

    let mut result = HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = match_git_path(line, &files) {
            *result.entry(path).or_insert(0) += 1;
        }
    }
    result
}

/// Detect code clones by normalized source hash
#[allow(clippy::cast_possible_truncation)]
pub(super) fn detect_code_clones(functions: &[FunctionEntry]) -> HashMap<usize, u32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut result = HashMap::new();
    let mut hash_to_indices: HashMap<u64, Vec<usize>> = HashMap::new();

    for (i, func) in functions.iter().enumerate() {
        // Normalize source: remove whitespace, lowercase identifiers
        let normalized = normalize_source(&func.source);

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        let hash = hasher.finish();

        hash_to_indices.entry(hash).or_default().push(i);
    }

    // Mark functions that have clones (more than 1 with same hash)
    for indices in hash_to_indices.values() {
        if indices.len() > 1 {
            let count = indices.len() as u32;
            for &idx in indices {
                result.insert(idx, count);
            }
        }
    }

    result
}

/// Normalize source code for clone detection
pub(super) fn normalize_source(source: &str) -> String {
    source
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// Compute pattern diversity per file (unique AST patterns / total patterns)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_file_pattern_diversity(
    functions: &[FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();

    for (file_path, indices) in file_index {
        if indices.is_empty() {
            continue;
        }

        // Count unique patterns in file based on function signatures
        let mut patterns = std::collections::HashSet::new();
        for &idx in indices {
            if let Some(func) = functions.get(idx) {
                // Extract pattern: return type + param count + complexity bucket
                let pattern = format!(
                    "{}:{}:{}",
                    extract_return_type(&func.signature),
                    count_params(&func.signature),
                    func.quality.complexity / 5 // bucket by 5
                );
                patterns.insert(pattern);
            }
        }

        let diversity = patterns.len() as f32 / indices.len() as f32;
        result.insert(file_path.clone(), diversity);
    }

    result
}

/// Extract return type from signature (simplified)
pub(super) fn extract_return_type(sig: &str) -> &str {
    if sig.contains("->") {
        sig.split("->").last().unwrap_or("void").trim()
    } else {
        "void"
    }
}

/// Count parameters in signature
pub(super) fn count_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        if let Some(end) = sig.find(')') {
            let params = &sig[start + 1..end];
            if params.trim().is_empty() {
                return 0;
            }
            return params.split(',').count();
        }
    }
    0
}

/// Detect fault patterns in function source
pub(super) fn detect_fault_patterns(functions: &[FunctionEntry]) -> HashMap<usize, Vec<String>> {
    let mut result = HashMap::new();

    let patterns = [
        ("unwrap()", "UNWRAP"),
        ("expect(", "EXPECT"),
        ("panic!", "PANIC"),
        ("unsafe {", "UNSAFE"),
        ("unsafe{", "UNSAFE"),
        (".clone()", "CLONE"),
        ("// TODO", "TODO"),
        ("// FIXME", "FIXME"),
        ("// HACK", "HACK"),
        ("// XXX", "XXX"),
        ("unimplemented!", "UNIMPL"),
        ("todo!", "TODO_MACRO"),
        ("unreachable!", "UNREACHABLE"),
    ];

    for (i, func) in functions.iter().enumerate() {
        let mut faults = Vec::new();
        let src = &func.source;

        for (pattern, label) in &patterns {
            if src.contains(pattern) {
                faults.push(label.to_string());
            }
        }

        if !faults.is_empty() {
            faults.sort();
            faults.dedup();
            result.insert(i, faults);
        }
    }

    result
}

/// Compute name frequency for generic name demotion.
///
/// Returns a map of function_name -> fraction of total functions with that name.
/// High-frequency names like `new`, `default`, `from` get demoted in search results.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn compute_name_frequency(
    name_index: &HashMap<String, Vec<usize>>,
    total: usize,
) -> HashMap<String, f32> {
    if total == 0 {
        return HashMap::new();
    }
    name_index
        .iter()
        .map(|(name, indices)| (name.clone(), indices.len() as f32 / total as f32))
        .collect()
}

/// Check if a function name is too generic for meaningful call graph edges.
///
/// Common method names like `new`, `from`, `clone` appear in thousands of types,
/// creating O(n^2) spurious edges. Excluding them reduces call graph size by ~99%
/// for large repos (e.g., 58GB -> <100MB for 230K-function repos).
pub(crate) fn is_generic_callee(name: &str) -> bool {
    matches!(
        name,
        "new" | "from" | "into" | "default" | "clone" | "fmt"
            | "len" | "push" | "pop" | "get" | "set" | "insert" | "remove"
            | "unwrap" | "expect" | "map" | "and_then" | "or_else" | "ok" | "err"
            | "to_string" | "to_owned" | "as_ref" | "as_mut" | "borrow"
            | "iter" | "collect" | "filter" | "fold" | "next"
            | "write" | "read" | "flush" | "close" | "open"
            | "is_empty" | "contains" | "starts_with" | "ends_with"
            | "display" | "debug" | "hash" | "cmp" | "partial_cmp"
            | "serialize" | "deserialize" | "drop"
            | "init" | "run" | "start" | "stop" | "build" | "parse" | "format"
            | "test" | "setup" | "teardown" | "assert" | "verify" | "check"
            // Lua stdlib (prevent noise edges)
            | "require" | "print" | "pairs" | "ipairs" | "type" | "error"
            | "pcall" | "xpcall" | "select" | "rawget" | "rawset" | "rawlen"
            | "tostring" | "tonumber" | "setmetatable" | "getmetatable"
            | "table" | "string" | "math" | "coroutine" | "unpack"
    )
}

/// Check if a code chunk is a test function or from a test file.
///
/// Used to exclude test code from the index at build time, reducing index size
/// by 25-70% for test-heavy repos.
pub(crate) fn is_test_chunk(chunk_name: &str, file_path: &str) -> bool {
    // File-level: skip *_test.rs, *_tests.rs, tests/ directories
    if file_path.contains("/tests/")
        || file_path.ends_with("_test.rs")
        || file_path.ends_with("_tests.rs")
    {
        return true;
    }
    // Function-level: skip test_ prefixed functions
    if chunk_name.starts_with("test_") {
        return true;
    }
    false
}

/// Build caller/callee graph by matching identifiers in source against function names.
///
/// For each function, extracts identifiers from its source and checks if they match
/// any known function name. If a match is found (and it's not a self-reference),
/// records a call edge. Generic method names (new, clone, etc.) are excluded to
/// prevent O(n^2) edge explosion.
/// Extract meaningful identifiers from source code for call graph analysis.
fn extract_call_identifiers(source: &str) -> Vec<&str> {
    source
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() >= 3 && !is_keyword(s) && !is_generic_callee(s))
        .collect()
}

/// Record call edges from a single caller to its callees.
fn record_call_edges(
    caller_idx: usize,
    idents: &[&str],
    name_index: &HashMap<String, Vec<usize>>,
    calls: &mut HashMap<usize, Vec<usize>>,
    called_by: &mut HashMap<usize, Vec<usize>>,
) {
    let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for ident in idents {
        let Some(callee_indices) = name_index.get(*ident) else { continue };
        for &callee_idx in callee_indices {
            if callee_idx != caller_idx && seen.insert(callee_idx) {
                calls.entry(caller_idx).or_default().push(callee_idx);
                called_by.entry(callee_idx).or_default().push(caller_idx);
            }
        }
    }
}

pub(crate) fn build_call_graph(
    functions: &[FunctionEntry],
    name_index: &HashMap<String, Vec<usize>>,
) -> (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) {
    let mut calls: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut called_by: HashMap<usize, Vec<usize>> = HashMap::new();

    for (caller_idx, func) in functions.iter().enumerate() {
        let idents = extract_call_identifiers(&func.source);
        record_call_edges(caller_idx, &idents, name_index, &mut calls, &mut called_by);
    }

    (calls, called_by)
}

/// Compute graph metrics (PageRank, centrality) for each function.
///
/// Uses a simplified PageRank algorithm:
/// - Damping factor: 0.85
/// - Iterations: 20 (sufficient for convergence)
/// - Initial score: 1/N for each node
///
/// PageRank represents "importance" - functions that are transitively called
/// by many other functions will have higher scores.
/// Run one iteration of PageRank: distribute scores from callers to callees.
#[allow(clippy::cast_possible_truncation)]
fn pagerank_iteration(
    pagerank: &[f32],
    new_pagerank: &mut [f32],
    calls: &HashMap<usize, Vec<usize>>,
    damping: f32,
    num_functions: usize,
) {
    let teleport = (1.0 - damping) / num_functions as f32;
    new_pagerank.iter_mut().for_each(|s| *s = teleport);

    // Distribute scores along call edges
    for (caller_idx, callees) in calls {
        if !callees.is_empty() {
            let contribution = damping * pagerank[*caller_idx] / callees.len() as f32;
            for &callee_idx in callees {
                if callee_idx < num_functions {
                    new_pagerank[callee_idx] += contribution;
                }
            }
        }
    }

    // Dangling nodes: distribute their rank evenly to all nodes
    let dangling_sum: f32 = (0..num_functions)
        .filter(|idx| calls.get(idx).map_or(true, |c| c.is_empty()))
        .map(|idx| pagerank[idx])
        .sum();
    let dangling_contrib = damping * dangling_sum / num_functions as f32;
    new_pagerank.iter_mut().for_each(|s| *s += dangling_contrib);
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn compute_graph_metrics(
    num_functions: usize,
    calls: &HashMap<usize, Vec<usize>>,
    called_by: &HashMap<usize, Vec<usize>>,
) -> Vec<GraphMetrics> {
    if num_functions == 0 {
        return Vec::new();
    }

    let mut pagerank = vec![1.0 / num_functions as f32; num_functions];
    let mut new_pagerank = vec![0.0; num_functions];

    for _ in 0..20 {
        pagerank_iteration(&pagerank, &mut new_pagerank, calls, 0.85, num_functions);
        std::mem::swap(&mut pagerank, &mut new_pagerank);
    }

    (0..num_functions)
        .map(|idx| {
            let in_degree = called_by.get(&idx).map_or(0, |v| v.len()) as u32;
            let out_degree = calls.get(&idx).map_or(0, |v| v.len()) as u32;
            let centrality =
                (in_degree + out_degree) as f32 / (2.0 * num_functions as f32).max(1.0);
            GraphMetrics { pagerank: pagerank[idx], centrality, in_degree, out_degree }
        })
        .collect()
}

/// Check if directory should be ignored
pub(super) fn is_ignored_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    matches!(
        name,
        "target"
            | "node_modules"
            | ".git"
            | ".pmat"
            | "__pycache__"
            | "venv"
            | ".venv"
            | "dist"
            | "build"
            | ".next"
            | ".cache"
            | "vendor"
            | "third_party"
            | "third-party"
            | "external"
            | "deps"
            | "book"
            | "theme"
            | "fixtures"
            | ".cargo"
    )
}

/// Detect language from file extension
pub(super) fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
        "py" => Some(Language::Python),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        "go" => Some(Language::Go),
        "lua" => Some(Language::Lua),
        _ => None,
    }
}

/// Extract quality metrics from a code chunk
#[allow(clippy::cast_possible_truncation)]
pub(super) fn extract_quality_metrics(chunk: &CodeChunk, _full_content: &str) -> QualityMetrics {
    let loc = chunk.content.lines().count() as u32;

    // Count control flow complexity (simple heuristic)
    let complexity = count_complexity(&chunk.content);

    // Count SATD markers
    let satd_count = count_satd_markers(&chunk.content);

    // Estimate Big-O from control flow
    let big_o = estimate_big_o(&chunk.content);

    // Exempt enums/structs/traits from LOC penalty — they're declarations, not logic
    use crate::services::semantic::ChunkType;
    let effective_loc = match chunk.chunk_type {
        ChunkType::Enum | ChunkType::Struct | ChunkType::Trait | ChunkType::TypeAlias => 0,
        _ => loc,
    };
    let tdg_score = calculate_simple_tdg(complexity, satd_count, effective_loc);
    let tdg_grade = score_to_grade(tdg_score);

    QualityMetrics {
        tdg_score,
        tdg_grade,
        complexity,
        cognitive_complexity: complexity, // Simplified: use same as cyclomatic
        big_o,
        satd_count,
        loc,
        commit_count: 0,  // Populated later by churn enrichment
        churn_score: 0.0, // Populated later by churn enrichment
    }
}

/// Count cyclomatic complexity (simplified)
pub(super) fn count_complexity(source: &str) -> u32 {
    let mut complexity = 1u32; // Base complexity

    // Count decision points
    for line in source.lines() {
        let trimmed = line.trim();

        // Control flow keywords
        if trimmed.starts_with("if ")
            || trimmed.starts_with("else if ")
            || trimmed.contains(" if ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("loop ")
            || trimmed.contains("&&")
            || trimmed.contains("||")
            || trimmed.contains("? ")
        {
            complexity += 1;
        }

        // Match arms
        if trimmed.contains("=>") && !trimmed.starts_with("//") {
            complexity += 1;
        }
    }

    complexity
}

/// Count SATD markers in implementation comments only.
/// Excludes doc comments (/// and //!), string literals, and identifiers.
/// Only counts markers that represent genuine self-admitted technical debt.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn count_satd_markers(source: &str) -> u32 {
    let mut count = 0u32;
    let mut in_block_comment = false;
    let mut in_raw_string = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip lines inside raw string literals
        if update_raw_string_state(trimmed, &mut in_raw_string) {
            continue;
        }

        // Track block comment state
        if in_block_comment {
            count += count_markers_in_line(trimmed);
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            in_block_comment = true;
            count += count_markers_in_line(trimmed);
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        // Skip doc comments (/// and //!) — these describe behavior, not debt
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }

        count += count_markers_in_comment(trimmed);
    }

    count
}

/// Count SATD markers in a single line (used for block comments).
fn count_markers_in_line(line: &str) -> u32 {
    let upper = line.to_uppercase();
    let mut count = 0u32;
    for marker in ["TODO", "FIXME", "HACK", "OPTIMIZE"] {
        count += upper.matches(marker).count() as u32;
    }
    count
}

/// Count SATD markers in inline comment portion of a line.
/// Skips if // is inside a string literal (odd quote count before //).
fn count_markers_in_comment(trimmed: &str) -> u32 {
    let Some(comment_start) = trimmed.find("//") else {
        return 0;
    };
    let before = &trimmed[..comment_start];
    if before.chars().filter(|&c| c == '"').count() % 2 != 0 {
        return 0;
    }
    count_markers_in_line(&trimmed[comment_start..])
}

/// Track raw string literal state. Returns true if line should be skipped.
fn update_raw_string_state(trimmed: &str, in_raw_string: &mut bool) -> bool {
    if *in_raw_string {
        if trimmed.contains("\"#") || trimmed.ends_with('"') {
            *in_raw_string = false;
        }
        return true;
    }
    if let Some(pos) = trimmed.find("r#\"") {
        let after_open = &trimmed[pos + 3..];
        if !after_open.contains("\"#") {
            *in_raw_string = true;
        }
        return true;
    }
    false
}

/// Estimate Big-O from control flow
pub(super) fn estimate_big_o(source: &str) -> String {
    let mut current_nesting = 0;
    let mut max_nesting = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("loop ")
        {
            current_nesting += 1;
            max_nesting = max_nesting.max(current_nesting);
        }

        if trimmed == "}" && current_nesting > 0 {
            current_nesting -= 1;
        }
    }

    match max_nesting {
        0 => "O(1)".to_string(),
        1 => "O(n)".to_string(),
        2 => "O(n^2)".to_string(),
        3 => "O(n^3)".to_string(),
        n => format!("O(n^{n})"),
    }
}

/// Calculate simplified TDG score
#[allow(clippy::cast_possible_truncation)]
pub(super) fn calculate_simple_tdg(complexity: u32, satd_count: u32, loc: u32) -> f32 {
    let mut score = 0.0f32;

    // Complexity penalty (0-4 points)
    // Divisor of 25: CC=50 → 2.0 (B boundary). Functions at the pre-commit
    // CC<=30 gate get score=1.2 (safe A). Dispatchers (CC~45) score 1.8 (A).
    // CC=75 → 3.0, CC=100 → 4.0 (cap).
    score += (complexity as f32 / 25.0).min(4.0);

    // SATD penalty (0-2 points, first 2 markers free to reduce false positives)
    // Many functions reference SATD markers descriptively (detector code, enums).
    // 3 SATD → 0.5, 4 → 1.0, 5 → 1.5, 6+ → 2.0.
    score += (satd_count.saturating_sub(2) as f32 * 0.5).min(2.0);

    // LOC penalty (0-2 points for > 200 lines)
    // Threshold at 200: functions under 200 LOC are rarely problematic.
    // Divisor of 200: LOC=400 → 1.0 penalty, LOC=600 → 2.0 (capped).
    if loc > 200 {
        score += ((loc - 200) as f32 / 200.0).min(2.0);
    }

    score.min(10.0)
}

/// Convert TDG score to letter grade
pub(super) fn score_to_grade(score: f32) -> String {
    match score {
        s if s < 2.0 => "A".to_string(),
        s if s < 4.0 => "B".to_string(),
        s if s < 6.0 => "C".to_string(),
        s if s < 8.0 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Extract doc comment from source
/// Classify a line above a function definition for doc comment extraction.
enum DocLineKind<'a> {
    DocComment(&'a str),
    BlockCommentStart,
    BlockCommentBody(&'a str),
    SkipLine, // empty, attribute, annotation
    Other,
}

fn classify_doc_line(line: &str) -> DocLineKind<'_> {
    if line.starts_with("///") || line.starts_with("//!") {
        DocLineKind::DocComment(line.trim_start_matches("///").trim_start_matches("//!").trim())
    } else if line.starts_with("/**") || line.starts_with("/*") {
        DocLineKind::BlockCommentStart
    } else if line.starts_with('*') {
        DocLineKind::BlockCommentBody(line.trim_start_matches('*').trim())
    } else if line.is_empty() || line.starts_with("#[") || line.starts_with('@') {
        DocLineKind::SkipLine
    } else {
        DocLineKind::Other
    }
}

pub(super) fn extract_doc_comment(content: &str, start_line: usize) -> Option<String> {
    if start_line <= 1 {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut doc_lines = Vec::new();

    for i in (0..start_line - 1).rev() {
        let line = lines.get(i)?.trim();
        match classify_doc_line(line) {
            DocLineKind::DocComment(text) => doc_lines.push(text),
            DocLineKind::BlockCommentBody(text) => doc_lines.push(text),
            DocLineKind::BlockCommentStart | DocLineKind::Other => break,
            DocLineKind::SkipLine => continue,
        }
    }

    if doc_lines.is_empty() {
        return None;
    }
    doc_lines.reverse();
    Some(doc_lines.join(" "))
}

/// Extract identifiers from source for better search
pub(super) fn extract_identifiers(source: &str) -> String {
    let mut identifiers = Vec::new();

    for word in source.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let trimmed = word.trim();
        if trimmed.len() >= 3 && !is_keyword(trimmed) {
            identifiers.push(trimmed.to_lowercase());
        }
    }

    identifiers.join(" ")
}

/// Check if word is a language keyword
pub(super) fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "const"
            | "static"
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "async"
            | "await"
            | "true"
            | "false"
            | "self"
            | "Self"
            | "super"
            | "crate"
            | "where"
            | "move"
            | "ref"
            | "dyn"
            | "box"
            | "in"
            | "as"
            | "unsafe"
            | "extern"
            | "macro"
            | "function"
            | "class"
            | "def"
            | "import"
            | "from"
            | "try"
            | "catch"
            | "throw"
            | "new"
            | "this"
            | "var"
            | "void"
            | "int"
            | "str"
            | "bool"
            | "None"
            | "null"
            | "undefined"
            // Lua keywords
            | "local"
            | "then"
            | "end"
            | "elseif"
            | "repeat"
            | "until"
            | "and"
            | "or"
            | "not"
            | "nil"
            | "goto"
    )
}
