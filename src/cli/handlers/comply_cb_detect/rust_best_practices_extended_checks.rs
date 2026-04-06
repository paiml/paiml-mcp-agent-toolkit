// CB-522 through CB-527: Pattern-based best practice detectors
// Detects: untested path normalization, external config heuristics,
// incomplete enum matches, hardcoded field names, single-path resolution,
// and incomplete classification chains.

/// CB-522: Untested Path Normalization - path manipulation without edge case handling
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb522_untested_path_normalization(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    // Path manipulation patterns that indicate URL/path normalization
    let path_manip_patterns = [
        ".strip_prefix(\"http",
        ".replace(\"//\"",
        ".replace(\"resolve/\"",
        "split(\"://\")",
        "trim_start_matches(\"http",
        "Url::parse(",
    ];

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let mut path_manip_count = 0u32;
        let mut first_line = 0usize;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            if path_manip_patterns.iter().any(|p| trimmed.contains(p)) {
                if path_manip_count == 0 {
                    first_line = i;
                }
                path_manip_count += 1;
            }
        }

        // Multiple path manipulations in one file suggest complex URL/path normalization
        if path_manip_count >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-522".to_string(),
                file,
                line: first_line + 1,
                description: format!(
                    "{path_manip_count} path/URL manipulation operations — verify edge cases (double slashes, web URLs, relative paths) are tested"
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

/// CB-523: External Config Over Embedded Metadata - filesystem heuristics instead of embedded data
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb523_external_config_over_embedded(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // Filesystem heuristic patterns
    let fs_heuristic_patterns = [".with_file_name(", ".with_extension("];
    let config_discovery = [
        "config.json",
        "tokenizer.json",
        "generation_config",
        "model.json",
        "params.json",
        "hyperparams",
    ];

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Detect: path.with_file_name("config.json") or similar sibling file discovery
            let has_fs_heuristic = fs_heuristic_patterns.iter().any(|p| trimmed.contains(p));
            let has_config_discovery = config_discovery.iter().any(|p| trimmed.contains(p));

            if has_fs_heuristic && has_config_discovery {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-523".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "External config discovery via filesystem heuristic — prefer embedded metadata if available".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-524: Incomplete Enum Match Coverage - wildcard matches on project enums across functions
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb524_incomplete_enum_match(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // Track: for each file, count match blocks that use _ => catch-all
    // If a file has many _ => catch-all arms with different concrete return types,
    // it's likely dispatching on the same enum inconsistently
    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let mut wildcard_match_count = 0u32;
        let mut wildcard_lines: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Count _ => arms that return concrete values (not errors)
            if trimmed.starts_with("_ =>") || trimmed.starts_with("_=>") {
                let after = trimmed
                    .trim_start_matches("_ =>")
                    .trim_start_matches("_=>")
                    .trim();

                // Skip error/none/panic patterns — these are deliberate catch-alls
                let safe_patterns = [
                    "Err(",
                    "None",
                    "unreachable!",
                    "panic!",
                    "return Err",
                    "bail!",
                    "todo!",
                    "unimplemented!",
                    "Default::default()",
                ];
                let is_safe = after.is_empty()
                    || after == "{"
                    || after == "}"
                    || after == "},"
                    || safe_patterns.iter().any(|p| after.contains(p));

                if !is_safe {
                    wildcard_match_count += 1;
                    wildcard_lines.push(i + 1);
                }
            }
        }

        // If a single file has 3+ wildcard match arms with concrete returns,
        // it's dispatching on an enum in multiple places with catch-all defaults
        if wildcard_match_count >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-524".to_string(),
                file,
                line: wildcard_lines.first().copied().unwrap_or(0),
                description: format!(
                    "{wildcard_match_count} catch-all match arms with concrete defaults in single file (lines: {}) — enum variants may be inconsistently handled",
                    wildcard_lines.iter().take(5).map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-525: Hardcoded Field Names Without Aliases - JSON .get("field") chains without fallbacks
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb525_hardcoded_field_names(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let json_get_re = regex::Regex::new(r#"\.get\(\s*""#).expect("valid regex");

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Per-function: count .get("field") calls without .or_else fallback
        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut get_count: u32 = 0;
        let mut has_or_fallback = false;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && fn_start.is_none()
            {
                fn_start = Some(i);
                fn_depth = 0;
                get_count = 0;
                has_or_fallback = false;
            }

            if fn_start.is_some() {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if !trimmed.starts_with("//") {
                    if json_get_re.is_match(trimmed) {
                        get_count += 1;
                    }
                    if trimmed.contains(".or_else(") || trimmed.contains(".or(") {
                        has_or_fallback = true;
                    }
                }

                if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                    // 5+ .get("field") without any .or_else/.or fallback alias support
                    if get_count >= 5 && !has_or_fallback {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-525".to_string(),
                            file: file.clone(),
                            line: fn_start.unwrap_or(0) + 1,
                            description: format!(
                                "{get_count} hardcoded .get(\"field\") calls without alias fallbacks — schemas with alternative field names will fail silently"
                            ),
                            severity: Severity::Info,
                        });
                    }
                    fn_start = None;
                }
            }
        }
    }

    violations
}

/// CB-526: Single-Path File Resolution - file lookup without fallback search
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb526_single_path_resolution(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Pattern: path.join("specific_file.ext").exists() without fallback
            // or: path.join("specific_file.ext") followed by read without exists check
            if trimmed.contains(".join(\"") && trimmed.contains(".exists()") {
                // Check if there's a fallback on same or next line
                let next_trimmed = lines.get(i + 1).map(|l| l.trim()).unwrap_or("");
                let has_fallback = trimmed.contains("||")
                    || trimmed.contains(".or_else")
                    || next_trimmed.contains("||")
                    || next_trimmed.contains("else {")
                    || next_trimmed.contains(".parent()");

                if !has_fallback {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-526".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: "Single-path file resolution without fallback — consider parent directory or recursive search".to_string(),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    violations
}

/// CB-527: Incomplete Pattern List - contains()/starts_with() classification chains
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb527_incomplete_pattern_list(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let classification_re =
        regex::Regex::new(r#"\.contains\(\s*"[a-z_]+"\s*\)\s*\|\|"#).expect("valid regex");

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Look for chains of .contains("x") || .contains("y") || ... — classification patterns
        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Count contains() calls chained with ||
            let chain_count = classification_re.find_iter(trimmed).count();

            // Only check continuation on next line if current line starts a chain
            let next_chain = if chain_count > 0 {
                lines
                    .get(i + 1)
                    .map(|l| classification_re.find_iter(l.trim()).count())
                    .unwrap_or(0)
            } else {
                0
            };

            let total = chain_count + next_chain;

            if total >= 3 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-527".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: format!(
                        "Classification chain with {total}+ .contains() patterns — may be incomplete; consider a centralized pattern registry"
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}
