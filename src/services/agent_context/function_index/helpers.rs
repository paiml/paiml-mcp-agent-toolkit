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

/// Build name_index, file_index, and corpus from functions.
pub(crate) fn build_indices(functions: &[FunctionEntry]) -> BuildIndicesResult {
    let mut result = BuildIndicesResult {
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: Vec::with_capacity(functions.len()),
    };

    for (idx, func) in functions.iter().enumerate() {
        result
            .name_index
            .entry(func.function_name.clone())
            .or_default()
            .push(idx);
        result
            .file_index
            .entry(func.file_path.clone())
            .or_default()
            .push(idx);

        let doc = format!(
            "{name} {name} {sig} {sig} {doc} {doc} {path} {idents}",
            name = func.function_name,
            sig = func.signature,
            doc = func.doc_comment.as_deref().unwrap_or(""),
            path = func.file_path,
            idents = extract_identifiers(&func.source)
        );
        result.corpus.push(doc);
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

/// Get commit counts per file from git log
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git process
pub(super) fn get_file_commit_counts<'a>(
    project_root: &std::path::Path,
    files: impl Iterator<Item = &'a String>,
) -> HashMap<String, u32> {
    let mut result = HashMap::new();

    // Collect unique files
    let files: std::collections::HashSet<_> = files.collect();
    if files.is_empty() {
        return result;
    }

    // Get all file changes from git log (fast batch operation)
    let output = std::process::Command::new("git")
        .args(["log", "--format=", "--name-only", "--since=1 year ago"])
        .current_dir(project_root)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Try exact match first
                if files.contains(&line.to_string()) {
                    *result.entry(line.to_string()).or_insert(0) += 1;
                    continue;
                }

                // Handle path migrations (e.g., server/src/foo.rs -> src/foo.rs)
                let normalized = line.strip_prefix("server/").unwrap_or(line);

                if files.contains(&normalized.to_string()) {
                    *result.entry(normalized.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    result
}

/// Detect code clones by normalized source hash
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

/// Build caller/callee graph by matching identifiers in source against function names.
///
/// For each function, extracts identifiers from its source and checks if they match
/// any known function name. If a match is found (and it's not a self-reference),
/// records a call edge.
pub(crate) fn build_call_graph(
    functions: &[FunctionEntry],
    name_index: &HashMap<String, Vec<usize>>,
) -> (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) {
    let mut calls: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut called_by: HashMap<usize, Vec<usize>> = HashMap::new();

    for (caller_idx, func) in functions.iter().enumerate() {
        // Extract identifiers from source
        let idents: Vec<&str> = func
            .source
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() >= 3 && !is_keyword(s))
            .collect();

        let mut seen_callees: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for ident in &idents {
            if let Some(callee_indices) = name_index.get(*ident) {
                for &callee_idx in callee_indices {
                    // Skip self-references and duplicates
                    if callee_idx == caller_idx || seen_callees.contains(&callee_idx) {
                        continue;
                    }
                    seen_callees.insert(callee_idx);
                    calls.entry(caller_idx).or_default().push(callee_idx);
                    called_by.entry(callee_idx).or_default().push(caller_idx);
                }
            }
        }
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
pub(crate) fn compute_graph_metrics(
    num_functions: usize,
    calls: &HashMap<usize, Vec<usize>>,
    called_by: &HashMap<usize, Vec<usize>>,
) -> Vec<GraphMetrics> {
    if num_functions == 0 {
        return Vec::new();
    }

    let damping = 0.85_f32;
    let iterations = 20;
    let initial_score = 1.0 / num_functions as f32;

    // Initialize PageRank scores
    let mut pagerank: Vec<f32> = vec![initial_score; num_functions];
    let mut new_pagerank: Vec<f32> = vec![0.0; num_functions];

    // Iterative PageRank computation
    for _ in 0..iterations {
        // Reset new scores with teleportation probability
        for score in new_pagerank.iter_mut() {
            *score = (1.0 - damping) / num_functions as f32;
        }

        // Distribute scores from callers to callees
        for (caller_idx, callees) in calls {
            if callees.is_empty() {
                continue;
            }
            let contribution = damping * pagerank[*caller_idx] / callees.len() as f32;
            for &callee_idx in callees {
                if callee_idx < num_functions {
                    new_pagerank[callee_idx] += contribution;
                }
            }
        }

        // Handle dangling nodes (functions that don't call anything)
        // Their PageRank distributes evenly to all nodes
        let mut dangling_sum = 0.0_f32;
        for idx in 0..num_functions {
            if !calls.contains_key(&idx) || calls.get(&idx).map_or(true, |c| c.is_empty()) {
                dangling_sum += pagerank[idx];
            }
        }
        let dangling_contribution = damping * dangling_sum / num_functions as f32;
        for score in new_pagerank.iter_mut() {
            *score += dangling_contribution;
        }

        // Swap for next iteration
        std::mem::swap(&mut pagerank, &mut new_pagerank);
    }

    // Build GraphMetrics for each function
    let mut metrics: Vec<GraphMetrics> = Vec::with_capacity(num_functions);
    for idx in 0..num_functions {
        let in_degree = called_by.get(&idx).map_or(0, |v| v.len()) as u32;
        let out_degree = calls.get(&idx).map_or(0, |v| v.len()) as u32;
        let centrality = (in_degree + out_degree) as f32 / (2.0 * num_functions as f32).max(1.0);

        metrics.push(GraphMetrics {
            pagerank: pagerank[idx],
            centrality,
            in_degree,
            out_degree,
        });
    }

    metrics
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
        _ => None,
    }
}

/// Extract quality metrics from a code chunk
pub(super) fn extract_quality_metrics(chunk: &CodeChunk, _full_content: &str) -> QualityMetrics {
    let loc = chunk.content.lines().count() as u32;

    // Count control flow complexity (simple heuristic)
    let complexity = count_complexity(&chunk.content);

    // Count SATD markers
    let satd_count = count_satd_markers(&chunk.content);

    // Estimate Big-O from control flow
    let big_o = estimate_big_o(&chunk.content);

    // Calculate TDG score (simplified - real implementation uses full TDG)
    let tdg_score = calculate_simple_tdg(complexity, satd_count, loc);
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

/// Count SATD markers
pub(super) fn count_satd_markers(source: &str) -> u32 {
    let upper = source.to_uppercase();
    let mut count = 0;

    for marker in ["TODO", "FIXME", "HACK", "XXX", "BUG", "OPTIMIZE"] {
        count += upper.matches(marker).count() as u32;
    }

    count
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
pub(super) fn calculate_simple_tdg(complexity: u32, satd_count: u32, loc: u32) -> f32 {
    let mut score = 0.0f32;

    // Complexity penalty (0-4 points)
    score += (complexity as f32 / 10.0).min(4.0);

    // SATD penalty (0-2 points)
    score += (satd_count as f32).min(2.0);

    // LOC penalty (0-2 points for > 50 lines)
    if loc > 50 {
        score += ((loc - 50) as f32 / 50.0).min(2.0);
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
pub(super) fn extract_doc_comment(content: &str, start_line: usize) -> Option<String> {
    if start_line <= 1 {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut doc_lines = Vec::new();

    // Look backwards from function start for doc comments
    for i in (0..start_line - 1).rev() {
        let line = lines.get(i)?.trim();

        if line.starts_with("///") || line.starts_with("//!") {
            doc_lines.push(line.trim_start_matches("///").trim_start_matches("//!").trim());
        } else if line.starts_with("/**") || line.starts_with("/*") {
            // Block comment
            break;
        } else if line.starts_with('*') {
            // Inside block comment
            doc_lines.push(line.trim_start_matches('*').trim());
        } else if line.is_empty() || line.starts_with("#[") || line.starts_with('@') {
            // Empty line or attribute - continue
            continue;
        } else {
            break;
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join(" "))
    }
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
    )
}
