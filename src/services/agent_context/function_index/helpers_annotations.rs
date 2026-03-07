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
    eprintln!("  Faults: {} functions with patterns", fault_patterns.len());

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

    let Ok(output) = output else {
        return HashMap::new();
    };
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
        // Find matching ')' AFTER '(' to handle C++ comments like "// 1) out = exp(a - val)"
        if let Some(end) = sig[start..].find(')') {
            let params = &sig[start + 1..start + end];
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
        // CUDA/PTX fault patterns
        ("asm volatile", "INLINE_PTX"),
        ("asm(\"", "INLINE_PTX"),
        ("__syncthreads()", "CUDA_SYNC"),
        ("__shared__", "CUDA_SHMEM"),
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
