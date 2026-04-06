/// Detect coverage gaming patterns in a project
pub fn detect_coverage_gaming(project_path: &Path) -> Result<GamingDetectionResult> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut result = GamingDetectionResult {
        patterns_checked: vec![
            "cfg(not(coverage))".to_string(),
            "cfg(not(tarpaulin))".to_string(),
            "cfg(not(llvm_cov))".to_string(),
            "coverage exclusion comments".to_string(),
        ],
        ..Default::default()
    };

    // Walk through source files
    for entry in walkdir::WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_excluded_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_source_file(path) {
            result.files_scanned += 1;

            if let Ok(content) = std::fs::read_to_string(path) {
                // Check for cfg(not(coverage)) patterns
                check_cfg_patterns(path, &content, &mut result.violations);

                // Check for coverage exclusion comments
                check_exclusion_comments(path, &content, &mut result.violations);
            }
        }
    }

    // Check for codecov.yml changes
    check_codecov_changes(project_path, &mut result.violations)?;

    Ok(result)
}

/// Update raw string tracking state and determine if the line should be skipped.
/// Returns (should_skip, new_in_raw_string_state).
fn update_raw_string_state(trimmed: &str, in_raw_string: bool) -> (bool, bool) {
    if in_raw_string {
        let closed = trimmed.contains("\"#");
        return (true, !closed);
    }
    if !trimmed.contains("r#\"") {
        return (false, false);
    }
    // Line starts a raw string; check if it also closes on the same line
    let is_single_line = trimmed.contains("\"#") && trimmed.rfind("\"#") > trimmed.find("r#\"");
    (true, !is_single_line)
}

/// Check if a trimmed line should be skipped (comments or string literals).
fn is_non_attribute_line(trimmed: &str) -> bool {
    // Skip comments and doc comments
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return true;
    }
    // Skip lines where the pattern appears inside a string literal
    trimmed.contains('"') && !trimmed.starts_with("#[") && !trimmed.starts_with("#!")
}

/// Check for cfg(not(...)) coverage exclusion patterns
fn check_cfg_patterns(path: &Path, content: &str, violations: &mut Vec<GamingViolation>) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let patterns = [
        ("cfg(not(coverage))", GamingPattern::CfgNotCoverage),
        ("cfg(not(tarpaulin))", GamingPattern::CfgNotTarpaulin),
        ("cfg(not(llvm_cov))", GamingPattern::CfgNotLlvmCov),
        (
            "cfg(not(tarpaulin_include))",
            GamingPattern::CfgNotTarpaulin,
        ),
    ];

    let mut in_raw_string = false;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        let (skip, new_state) = update_raw_string_state(trimmed, in_raw_string);
        in_raw_string = new_state;
        if skip {
            continue;
        }

        if is_non_attribute_line(trimmed) {
            continue;
        }

        for (pattern, gaming_type) in &patterns {
            if line.contains(pattern)
                || line.contains(&pattern.replace("(", " ("))
                || line.contains(&format!("#[{}]", pattern))
            {
                violations.push(GamingViolation {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    pattern: gaming_type.clone(),
                    severity: Severity::Critical,
                    explanation: format!(
                        "Found {} at line {} - this excludes code from coverage",
                        pattern,
                        line_num + 1
                    ),
                });
            }
        }
    }
}

/// Check for coverage exclusion comment patterns
fn check_exclusion_comments(path: &Path, content: &str, violations: &mut Vec<GamingViolation>) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let comment_patterns = [
        "// LCOV_EXCL_START",
        "// LCOV_EXCL_STOP",
        "// LCOV_EXCL_LINE",
        "/* LCOV_EXCL_START */",
        "/* LCOV_EXCL_STOP */",
        "// coverage:ignore",
        "// istanbul ignore",
        "// c8 ignore",
        "#[no_coverage]",
        "#[coverage(off)]",
    ];

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        // Skip conditional cfg_attr coverage attributes — these are legitimate
        // Rust patterns that only activate under specific feature flags
        if trimmed.contains("cfg_attr(") && trimmed.contains("coverage(off)") {
            continue;
        }
        for pattern in &comment_patterns {
            if line.contains(pattern) {
                violations.push(GamingViolation {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    pattern: GamingPattern::CoverageExclusionComment,
                    severity: Severity::Warning,
                    explanation: format!(
                        "Found coverage exclusion comment '{}' at line {}",
                        pattern,
                        line_num + 1
                    ),
                });
            }
        }
    }
}

/// Check for changes to .codecov.yml that add exclusions
fn check_codecov_changes(
    project_path: &Path,
    violations: &mut Vec<GamingViolation>,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let codecov_path = project_path.join(".codecov.yml");
    let codecov_yaml = project_path.join("codecov.yml");

    let codecov_file = if codecov_path.exists() {
        Some(codecov_path)
    } else if codecov_yaml.exists() {
        Some(codecov_yaml)
    } else {
        None
    };

    if let Some(codecov_file) = codecov_file {
        // Check if file was modified in current work
        if let Ok(output) = std::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD~1"])
            .current_dir(project_path)
            .output()
        {
            let changed_files = String::from_utf8_lossy(&output.stdout);
            let codecov_rel = codecov_file
                .strip_prefix(project_path)
                .unwrap_or(&codecov_file)
                .to_string_lossy();

            if changed_files.contains(&*codecov_rel) {
                // Check what was added
                if let Ok(content) = std::fs::read_to_string(&codecov_file) {
                    // Look for ignore patterns
                    if content.contains("ignore:")
                        || content.contains("exclude:")
                        || content.contains("paths:")
                    {
                        violations.push(GamingViolation {
                            file: codecov_file,
                            line: 0,
                            pattern: GamingPattern::NewCodecovExclusion(
                                "Modified codecov config".to_string(),
                            ),
                            severity: Severity::Critical,
                            explanation:
                                "codecov.yml was modified during this work item - verify no gaming"
                                    .to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detect test file deletions by comparing against baseline manifest
pub fn detect_test_deletions(
    project_path: &Path,
    baseline_files: &HashSet<PathBuf>,
) -> Vec<GamingViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    for baseline_file in baseline_files {
        let full_path = project_path.join(baseline_file);

        // Check if it was a test file
        let is_test = baseline_file.to_string_lossy().contains("test")
            || baseline_file.to_string_lossy().contains("_test")
            || baseline_file.to_string_lossy().contains("/tests/");

        if is_test && !full_path.exists() {
            violations.push(GamingViolation {
                file: baseline_file.clone(),
                line: 0,
                pattern: GamingPattern::TestFileDeletion,
                severity: Severity::Critical,
                explanation: format!(
                    "Test file {} was deleted during this work item",
                    baseline_file.display()
                ),
            });
        }
    }

    violations
}

/// Detect critical file removals (CUDA, AVX, etc.)
pub fn detect_critical_file_removals(
    project_path: &Path,
    baseline_files: &HashSet<PathBuf>,
) -> Vec<GamingViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    for baseline_file in baseline_files {
        let full_path = project_path.join(baseline_file);
        let ext = baseline_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Critical file types that cannot be removed
        let is_critical = matches!(ext, "cu" | "cuh")  // CUDA
            || baseline_file.to_string_lossy().contains("simd")
            || baseline_file.to_string_lossy().contains("avx")
            || baseline_file.to_string_lossy().contains("neon");

        if is_critical && !full_path.exists() {
            violations.push(GamingViolation {
                file: baseline_file.clone(),
                line: 0,
                pattern: GamingPattern::CriticalFileRemoved,
                severity: Severity::Critical,
                explanation: format!(
                    "Critical file {} (CUDA/SIMD) was removed during this work item",
                    baseline_file.display()
                ),
            });
        }
    }

    violations
}

/// Check if path is an excluded directory
fn is_excluded_dir(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let path_str = path.to_string_lossy();
    path_str.contains("/target/")
        || path_str.contains("/.git/")
        || path_str.contains("/node_modules/")
        || path_str.contains("/.pmat-")
        || path_str.contains("/vendor/")
}

/// Check if file is a source file we should scan
fn is_source_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let ext = path.extension().and_then(|e| e.to_str());
    matches!(
        ext,
        Some(
            "rs" | "cu"
                | "cuh"
                | "c"
                | "cpp"
                | "h"
                | "hpp"
                | "py"
                | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "go"
        )
    )
}

/// Run meta-falsification check (verify the detector itself is working)
pub fn run_meta_falsification(_project_path: &Path) -> Result<bool> {
    debug_assert!(_project_path.exists(), "_project_path must exist: {}", _project_path.display());
    // Create a temporary test pattern to verify detection
    let test_content = r#"
        // This is a meta-test pattern
        #[cfg(not(coverage))]
        fn hidden_function() {}
    "#;

    // Meta-check: verify the detector can identify this known pattern
    let mut violations = Vec::new();
    check_cfg_patterns(Path::new("meta-test.rs"), test_content, &mut violations);

    // We SHOULD find exactly one violation
    if violations.len() == 1 && violations[0].pattern == GamingPattern::CfgNotCoverage {
        Ok(true) // Meta-check passed: detector is working
    } else {
        Ok(false) // Meta-check FAILED: detector is broken!
    }
}
