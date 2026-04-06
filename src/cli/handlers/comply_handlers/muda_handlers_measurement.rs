/// Overproduction waste: dead code percentage
/// Uses cached dead-code analysis if available
fn measure_overproduction(project_path: &Path) -> f64 {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cache_path = project_path.join(".pmat/dead-code-cache.json");
    if let Ok(content) = std::fs::read_to_string(&cache_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Look for dead code percentage in cache
            if let Some(pct) = json.get("dead_code_percentage").and_then(|v| v.as_f64()) {
                // Scale: 0% dead = 0 waste, 10%+ dead = 100 waste
                return (pct * 10.0).clamp(0.0, 100.0);
            }
            // Alternative: count dead items
            if let Some(items) = json.get("dead_items").and_then(|v| v.as_array()) {
                let count = items.len();
                return ((count as f64) * 2.0).clamp(0.0, 100.0);
            }
        }
    }
    0.0 // No data = assume no waste (conservative)
}

/// Waiting waste: slow tests and builds
/// Checks cached test timing and build metrics
fn measure_waiting(project_path: &Path) -> f64 {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut score = 0.0;

    // Check test timing from hooks cache
    let metrics_path = project_path.join(".pmat/hooks-cache/metrics.json");
    if let Ok(content) = std::fs::read_to_string(&metrics_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Check test-fast duration
            if let Some(test_ms) = json
                .get("test-fast")
                .and_then(|v| v.get("duration_ms"))
                .and_then(|v| v.as_f64())
            {
                let test_secs = test_ms / 1000.0;
                // Scale: <30s = 0 waste, >300s = 100 waste
                score += ((test_secs - 30.0) / 270.0 * 100.0).clamp(0.0, 100.0) * 0.5;
            }
            // Check lint duration
            if let Some(lint_ms) = json
                .get("lint")
                .and_then(|v| v.get("duration_ms"))
                .and_then(|v| v.as_f64())
            {
                let lint_secs = lint_ms / 1000.0;
                // Scale: <10s = 0 waste, >60s = 100 waste
                score += ((lint_secs - 10.0) / 50.0 * 100.0).clamp(0.0, 100.0) * 0.5;
            }
        }
    }

    score.clamp(0.0, 100.0)
}

/// Inventory waste: stale SATD markers (TODO/FIXME/HACK)
/// Counts SATD markers as inventory that should be processed
fn measure_inventory(project_path: &Path) -> f64 {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Check for SATD count in cached analysis
    let satd_cache = project_path.join(".pmat/satd-cache.json");
    if let Ok(content) = std::fs::read_to_string(&satd_cache) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(count) = json.get("total_count").and_then(|v| v.as_u64()) {
                // Scale: 0 SATD = 0 waste, 50+ SATD = 100 waste
                return ((count as f64) * 2.0).clamp(0.0, 100.0);
            }
        }
    }

    // Quick heuristic: count SATD markers in source
    let count = count_satd_markers(project_path);
    ((count as f64) * 2.0).clamp(0.0, 100.0)
}

/// Count SATD markers in Rust source files (quick heuristic)
fn count_satd_markers(project_path: &Path) -> usize {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0;
    }

    collect_rs_source_files(&src_dir)
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .map(|content| count_satd_in_content(&content))
        .sum()
}

/// Collect .rs files under a directory, excluding test files.
fn collect_rs_source_files(src_dir: &Path) -> Vec<std::path::PathBuf> {
    debug_assert!(src_dir.exists(), "src_dir must exist: {}", src_dir.display());
    walkdir::WalkDir::new(src_dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Count SATD markers (TODO/FIXME/HACK) in file content.
/// Only counts markers in actual production code comments, not string literals,
/// doc comments, security annotations, or test modules.
fn count_satd_in_content(content: &str) -> usize {
    let mut count = 0;
    let mut in_raw_string = false;
    let mut in_test_module = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Stop counting after #[cfg(test)] — everything below is test code
        if trimmed == "#[cfg(test)]" || trimmed.starts_with("#[cfg(all(test,") {
            in_test_module = true;
        }
        if in_test_module {
            continue;
        }

        // Track raw string boundaries (r#"..."#) to skip embedded comments
        if !in_raw_string && trimmed.contains("r#\"") {
            in_raw_string = true;
            // Check if it closes on the same line
            if trimmed.contains("\"#") && trimmed.rfind("\"#") > trimmed.find("r#\"") {
                in_raw_string = false;
            }
            continue;
        }
        if in_raw_string {
            if trimmed.contains("\"#") {
                in_raw_string = false;
            }
            continue;
        }

        if is_satd_marker(trimmed) {
            count += 1;
        }
    }
    count
}

/// Check if a line is a genuine SATD comment (not a string literal,
/// doc comment, or security annotation).
fn is_satd_marker(trimmed: &str) -> bool {
    // Must be a line comment (not doc comment)
    if !trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }

    // Extract the comment text after //
    let comment = trimmed.get(2..).unwrap_or("");

    // Exclude security annotations — these are hardening notes, not debt
    if comment.trim_start().starts_with("SECURITY:") || comment.trim_start().starts_with("SAFETY:")
    {
        return false;
    }

    // The marker must appear in the comment text itself, not in a string
    // literal that happens to be on this line. If the line has quotes before
    // the marker, it's likely a string literal reference.
    let has_marker =
        comment.contains("TODO") || comment.contains("FIXME") || comment.contains("HACK");
    if !has_marker {
        return false;
    }

    // Exclude lines where the marker appears only inside quotes
    // (test fixtures, string constants referencing SATD patterns)
    let unquoted = strip_quoted_strings(trimmed);
    unquoted.contains("TODO") || unquoted.contains("FIXME") || unquoted.contains("HACK")
}

/// Strip content inside double quotes to avoid matching string literals.
fn strip_quoted_strings(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_quote = false;
    let mut prev_escape = false;
    for ch in s.chars() {
        if ch == '"' && !prev_escape {
            in_quote = !in_quote;
        } else if !in_quote {
            result.push(ch);
        }
        prev_escape = ch == '\\' && !prev_escape;
    }
    result
}

/// Collect top files with dead code from the dead-code cache.
/// Returns up to 5 file paths sorted by dead item count descending.
fn collect_overproduction_files(project_path: &Path) -> Vec<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let cache_path = project_path.join(".pmat/dead-code-cache.json");
    if let Ok(content) = std::fs::read_to_string(&cache_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Try "files" map: { "path": { "dead_count": N } }
            if let Some(files) = json.get("files").and_then(|v| v.as_object()) {
                let mut file_scores: Vec<(&str, u64)> = files
                    .iter()
                    .filter_map(|(path, val)| {
                        let count = val
                            .get("dead_count")
                            .and_then(|c| c.as_u64())
                            .unwrap_or(0);
                        if count > 0 {
                            Some((path.as_str(), count))
                        } else {
                            None
                        }
                    })
                    .collect();
                file_scores.sort_by(|a, b| b.1.cmp(&a.1));
                return file_scores
                    .into_iter()
                    .take(5)
                    .map(|(p, c)| format!("{} ({} dead)", p, c))
                    .collect();
            }
            // Try "dead_items" array: [{ "file": "path", ... }]
            if let Some(items) = json.get("dead_items").and_then(|v| v.as_array()) {
                let mut file_counts: HashMap<&str, usize> = HashMap::new();
                for item in items {
                    if let Some(file) = item.get("file").and_then(|f| f.as_str()) {
                        *file_counts.entry(file).or_insert(0) += 1;
                    }
                }
                let mut sorted: Vec<(&&str, &usize)> = file_counts.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));
                return sorted
                    .into_iter()
                    .take(5)
                    .map(|(p, c)| format!("{} ({} dead)", p, c))
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Collect top files with stale SATD markers (TODO/FIXME/HACK).
/// Returns up to 5 file paths sorted by SATD count descending.
fn collect_inventory_files(project_path: &Path) -> Vec<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return Vec::new();
    }

    let mut file_counts: Vec<(String, usize)> = collect_rs_source_files(&src_dir)
        .iter()
        .filter_map(|p| {
            let content = std::fs::read_to_string(p).ok()?;
            let count = count_satd_in_content(&content);
            if count > 0 {
                let rel = p
                    .strip_prefix(project_path)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string();
                Some((rel, count))
            } else {
                None
            }
        })
        .collect();

    file_counts.sort_by(|a, b| b.1.cmp(&a.1));
    file_counts
        .into_iter()
        .take(5)
        .map(|(p, c)| format!("{} ({} SATD)", p, c))
        .collect()
}
