// ============================================================================
// v4.0 provable contracts: Formal Proof Verification
// ============================================================================

/// Count sorry occurrences in Lean source, respecting comments and word boundaries.
/// Handles: line comments (--), nested block comments (/- ... -/), inline block comments,
/// and word-boundary checking to avoid false positives from identifiers like `sorry_helper`.
fn count_lean_sorry_in_source(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment = 0i32;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--") {
            continue;
        }

        // Strip block comments inline for same-line /- ... -/ handling
        let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);

        if in_block_comment > 0 {
            continue;
        }

        if contains_sorry_word_boundary(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips block comment content from a line, updating nesting depth.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Checks if line contains "sorry" as a standalone word (not part of an identifier).
fn contains_sorry_word_boundary(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";
    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok = abs_idx == 0
                || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    || bytes[abs_idx + sorry.len()] == b'_');
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}

/// Test formal proof verification: count sorry occurrences in .lean files
fn test_formal_proof_verification(
    project_path: &Path,
    max_sorry_count: usize,
) -> Result<FalsificationResult> {
    print!("Scanning .lean files for sorry... ");

    let mut total_sorry = 0usize;
    let mut sorry_files = Vec::new();

    // Walk project looking for .lean files
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "lean") {
            if let Ok(content) = std::fs::read_to_string(path) {
                let count = count_lean_sorry_in_source(&content);
                if count > 0 {
                    total_sorry += count;
                    sorry_files.push(path.to_path_buf());
                }
            }
        }
    }

    if sorry_files.is_empty() && total_sorry == 0 {
        return Ok(FalsificationResult::passed(
            "No .lean files with sorry found".to_string(),
        ));
    }

    if total_sorry <= max_sorry_count {
        Ok(FalsificationResult::passed(format!(
            "{} sorry occurrence(s) within threshold (max: {})",
            total_sorry, max_sorry_count
        )))
    } else {
        Ok(FalsificationResult::failed(
            format!(
                "{} sorry occurrence(s) exceed threshold (max: {}), in: {}",
                total_sorry,
                max_sorry_count,
                sorry_files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EvidenceType::FileList(sorry_files),
        ))
    }
}

// ============================================================================
// v2.6 comply spec: SATD, Dead Code, Per-File Coverage, Lint Gate
// ============================================================================

/// Test SATD detection: find new TODO/FIXME/HACK markers since baseline
async fn test_satd_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Detecting SATD markers... ");

    // Run pmat analyze satd
    let output = Command::new("pmat")
        .args([
            "analyze",
            "satd",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get current SATD count
                let current_count = json
                    .get("total_count")
                    .or_else(|| json.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                // Check for new SATD since baseline (compare with git diff)
                let new_satd = detect_new_satd_since_baseline(project_path, baseline_commit)?;

                if new_satd.is_empty() {
                    Ok(FalsificationResult::passed(format!(
                        "No new SATD markers ({} existing)",
                        current_count
                    )))
                } else {
                    let paths: Vec<PathBuf> = new_satd.iter().map(|(p, _)| p.clone()).collect();
                    let details: Vec<String> = new_satd
                        .iter()
                        .take(5)
                        .map(|(p, marker)| format!("{}: {}", p.display(), marker))
                        .collect();
                    Ok(FalsificationResult::failed(
                        format!(
                            "{} new SATD marker(s): {}",
                            new_satd.len(),
                            details.join("; ")
                        ),
                        EvidenceType::FileList(paths),
                    ))
                }
            } else {
                Ok(FalsificationResult::passed(
                    "SATD check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "SATD analyzer not available".to_string(),
        )),
    }
}

/// Check if a trimmed line is a regular SATD-eligible comment.
/// Must start with `//` but NOT `///` or `//!`, and must not be
/// a SECURITY/SAFETY annotation.
fn is_satd_comment(trimmed: &str) -> bool {
    if !trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    let comment_text = trimmed[2..].trim_start();
    !comment_text.starts_with("SECURITY:") && !comment_text.starts_with("SAFETY:")
}

/// Extract SATD markers from a single added line.
/// Returns one entry per matching pattern found in the line.
fn extract_satd_markers(
    line_content: &str,
    file: &Path,
    satd_patterns: &[&str],
) -> Vec<(PathBuf, String)> {
    let trimmed = line_content.trim();
    satd_patterns
        .iter()
        .filter(|pattern| trimmed.contains(*pattern))
        .map(|pattern| {
            let marker = line_content
                .split(pattern)
                .nth(1)
                .map(|s| format!("{}{}", pattern, s.chars().take(50).collect::<String>()))
                .unwrap_or_else(|| pattern.to_string());
            (file.to_path_buf(), marker)
        })
        .collect()
}

/// Detect new SATD markers by comparing git diff
fn detect_new_satd_since_baseline(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<Vec<(PathBuf, String)>> {
    let mut new_satd = Vec::new();

    // Get diff of added lines since baseline
    let output = Command::new("git")
        .args(["diff", "-U0", baseline_commit, "HEAD", "--", "*.rs"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        return Ok(new_satd);
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    let satd_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"];

    let mut current_file: Option<PathBuf> = None;

    for line in diff.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
            continue;
        }

        // Skip non-added lines and the "+++ b/" header (already handled above)
        let Some(line_content) = line.strip_prefix('+') else {
            continue;
        };
        if line.starts_with("+++") {
            continue;
        }

        let trimmed = line_content.trim();
        if !is_satd_comment(trimmed) {
            continue;
        }

        if let Some(ref file) = current_file {
            new_satd.extend(extract_satd_markers(line_content, file, &satd_patterns));
        }
    }

    Ok(new_satd)
}

/// Test dead code detection: find new unreachable code since baseline
async fn test_dead_code_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Detecting dead code... ");

    // Run pmat analyze dead-code
    let output = Command::new("pmat")
        .args([
            "analyze",
            "dead-code",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get dead code items
                let dead_items = json
                    .get("dead_code")
                    .or_else(|| json.get("items"))
                    .and_then(|items| items.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                // For now, we report any dead code found
                // Future: compare with baseline to only flag NEW dead code
                if dead_items == 0 {
                    Ok(FalsificationResult::passed(
                        "No dead code detected".to_string(),
                    ))
                } else {
                    // Check if these are new since baseline
                    let changed_files = get_changed_files(project_path, baseline_commit)?;
                    let dead_in_changed: usize = json
                        .get("dead_code")
                        .or_else(|| json.get("items"))
                        .and_then(|items| items.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter(|item| {
                                    item.get("file")
                                        .and_then(|f| f.as_str())
                                        .map(|f| changed_files.iter().any(|cf| cf.ends_with(f)))
                                        .unwrap_or(false)
                                })
                                .count()
                        })
                        .unwrap_or(0);

                    if dead_in_changed == 0 {
                        Ok(FalsificationResult::passed(format!(
                            "{} existing dead code items (none in changed files)",
                            dead_items
                        )))
                    } else {
                        Ok(FalsificationResult::failed(
                            format!("{} dead code item(s) in changed files", dead_in_changed),
                            EvidenceType::NumericComparison {
                                actual: dead_in_changed as f64,
                                threshold: 0.0,
                            },
                        ))
                    }
                }
            } else {
                Ok(FalsificationResult::passed(
                    "Dead code check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "Dead code analyzer not available".to_string(),
        )),
    }
}

/// Get list of changed files since baseline
fn get_changed_files(project_path: &Path, baseline_commit: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline_commit, "HEAD"])
        .current_dir(project_path)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    } else {
        Ok(Vec::new())
    }
}

/// Check if a file should be skipped for per-file coverage checks.
fn is_excluded_from_per_file_coverage(filename: &str) -> bool {
    filename.contains("/tests/") || filename.contains("_test.rs") || filename.contains("/target/")
}

/// Extract the line coverage percentage from a single llvm-cov file entry.
fn extract_file_line_coverage(file_entry: &serde_json::Value) -> f64 {
    file_entry
        .get("summary")
        .and_then(|s| s.get("lines"))
        .and_then(|l| l.get("percent"))
        .and_then(|p| p.as_f64())
        .unwrap_or(100.0)
}

/// Parse llvm-cov JSON and return files whose coverage is below `threshold`.
///
/// Each entry is `(filename, coverage_pct)`. Test files and generated files
/// are excluded.
fn collect_files_below_threshold(json: &serde_json::Value, threshold: f64) -> Vec<(PathBuf, f64)> {
    let data = match json.get("data").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return Vec::new(),
    };

    data.iter()
        .filter_map(|file_data| file_data.get("files").and_then(|f| f.as_array()))
        .flatten()
        .filter_map(|file| {
            let filename = file
                .get("filename")
                .and_then(|f| f.as_str())
                .unwrap_or("unknown");
            if is_excluded_from_per_file_coverage(filename) {
                return None;
            }
            let coverage = extract_file_line_coverage(file);
            if coverage < threshold {
                Some((PathBuf::from(filename), coverage))
            } else {
                None
            }
        })
        .collect()
}

/// Build a FalsificationResult from the list of files below coverage threshold.
fn build_per_file_coverage_result(
    files_below_threshold: Vec<(PathBuf, f64)>,
    threshold: f64,
) -> FalsificationResult {
    if files_below_threshold.is_empty() {
        return FalsificationResult::passed(format!("All files >= {:.1}% coverage", threshold));
    }

    let paths: Vec<PathBuf> = files_below_threshold
        .iter()
        .map(|(p, _)| p.clone())
        .collect();
    let details: Vec<String> = files_below_threshold
        .iter()
        .take(10)
        .map(|(p, cov)| format!("{}: {:.1}%", p.display(), cov))
        .collect();
    FalsificationResult::failed(
        format!(
            "{} file(s) below {:.1}% threshold: {}",
            files_below_threshold.len(),
            threshold,
            details.join(", ")
        ),
        EvidenceType::FileList(paths),
    )
}

/// Test per-file coverage: all files must meet threshold
async fn test_per_file_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking per-file coverage... ");

    // Try to read per-file coverage from llvm-cov output
    let coverage_json = project_path.join("target/llvm-cov/coverage.json");

    if !coverage_json.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No per-file coverage data (run 'make coverage'), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let files_below = collect_files_below_threshold(&json, threshold);
    Ok(build_per_file_coverage_result(files_below, threshold))
}

/// Test lint pass: O(1) - reads from cached lint status
async fn test_lint_pass(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading lint cache... ");

    // O(1): Read from cache instead of running make lint
    // Try primary cache location, then fallback to work-item and .pmat directories
    if let Some(cache) = read_cached_metric(project_path, "lint-status.json")
        .or_else(|| read_lint_cache_fallback(project_path))
    {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Lint cache too old ({} min). Run 'make lint' first.",
                    cache.age_minutes
                ),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // Validate 'passed' field exists — reject malformed cache (Popperian Audit v2.1)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid lint cache (missing 'passed' field). Re-run 'make lint'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = if cache.is_stale_warn {
            format!(
                " (cached {} min ago, consider re-running)",
                cache.age_minutes
            )
        } else {
            format!(" (cached {} min ago)", cache.age_minutes)
        };

        if passed {
            return Ok(FalsificationResult::passed(format!("PASSED{}", stale_note)));
        } else {
            let errors = cache
                .value
                .get("error_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} lint errors{}", errors, stale_note),
                EvidenceType::NumericComparison {
                    actual: errors as f64,
                    threshold: 0.0,
                },
            ));
        }
    }

    // No cache - check if Makefile exists and suggest running lint
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return Ok(FalsificationResult::passed(
            "No Makefile found (skipping lint check)".to_string(),
        ));
    }

    // No cache available - block until user runs make lint
    Ok(FalsificationResult::failed(
        "No lint cache. Run 'make lint' first (O(1) requirement)".to_string(),
        EvidenceType::BooleanCheck(false),
    ))
}
