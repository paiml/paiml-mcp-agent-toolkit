// CB-950 through CB-954 YAML best practices detectors.
// Included by yaml_best_practices.rs — do NOT add `use` imports or `#!` attributes.

// =============================================================================
// CB-950: Truthy String Ambiguity
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb950 truthy ambiguity.
pub fn detect_cb950_truthy_ambiguity(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let raw_lines: Vec<&str> = content.lines().collect();
        let prod_lines = compute_yaml_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            // Honor inline # pmat:ignore directives
            if let Some(raw) = raw_lines.get(line_num.wrapping_sub(1)) {
                if raw.contains("# pmat:ignore") {
                    continue;
                }
            }
            // Pattern: key: value where value is an unquoted truthy string
            if let Some(colon_pos) = line.find(": ") {
                let key = line[..colon_pos].trim();
                let value = line[colon_pos + 2..].trim();
                // Skip CI/CD keys that require native booleans
                if NATIVE_BOOLEAN_KEYS.contains(&key) {
                    continue;
                }
                // Check if value is an unquoted truthy string
                if !value.starts_with('"')
                    && !value.starts_with('\'')
                    && TRUTHY_STRINGS.contains(&value)
                {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-950".to_string(),
                        file: rel.clone(),
                        line: *line_num,
                        description: format!(
                            "Unquoted truthy string `{}` — quote to prevent implicit boolean conversion",
                            value
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-951: Excessive Nesting
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb951 excessive nesting.
pub fn detect_cb951_excessive_nesting(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut max_depth = 0;
        let mut max_depth_line = 1;

        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            // Calculate indentation depth (spaces / 2)
            let indent = line.len() - line.trim_start().len();
            let depth = indent / 2;
            if depth > max_depth {
                max_depth = depth;
                max_depth_line = i + 1;
            }
        }

        // Contract YAML files legitimately nest to 12+ levels
        // (proof_obligations → items → constraints → sub-fields)
        let threshold = if rel.starts_with("contracts/") || rel.contains("/contracts/") {
            14
        } else {
            8
        };

        if max_depth > threshold {
            violations.push(CbPatternViolation {
                pattern_id: "CB-951".to_string(),
                file: rel.clone(),
                line: max_depth_line,
                description: format!(
                    "Excessive nesting depth {} (threshold: {}) — consider restructuring",
                    max_depth, threshold
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

// =============================================================================
// CB-952: Missing Required Fields (GitHub Actions specific)
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb952 missing required fields.
pub fn detect_cb952_missing_required_fields(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        // Only check GitHub Actions workflows
        let rel_path = file_path.strip_prefix(project_path).unwrap_or(file_path);
        let rel = rel_path.display().to_string();
        if !rel.contains(".github/workflows") {
            continue;
        }

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let has_name = content.lines().any(|l| l.starts_with("name:"));
        let has_on = content
            .lines()
            .any(|l| l.starts_with("on:") || l.starts_with("on ") || l.trim() == "on:");
        let has_jobs = content.lines().any(|l| l.starts_with("jobs:"));

        if !has_name {
            violations.push(CbPatternViolation {
                pattern_id: "CB-952".to_string(),
                file: rel.clone(),
                line: 1,
                description: "GitHub Actions workflow missing `name:` field".to_string(),
                severity: Severity::Warning,
            });
        }
        if !has_on {
            violations.push(CbPatternViolation {
                pattern_id: "CB-952".to_string(),
                file: rel.clone(),
                line: 1,
                description: "GitHub Actions workflow missing `on:` trigger".to_string(),
                severity: Severity::Warning,
            });
        }
        if !has_jobs {
            violations.push(CbPatternViolation {
                pattern_id: "CB-952".to_string(),
                file: rel.clone(),
                line: 1,
                description: "GitHub Actions workflow missing `jobs:` section".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

// =============================================================================
// CB-953: Unpinned Action Version
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb953 unpinned action.
pub fn detect_cb953_unpinned_action(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let rel_path = file_path.strip_prefix(project_path).unwrap_or(file_path);
        let rel = rel_path.display().to_string();
        if !rel.contains(".github/workflows") && !rel.contains(".github/actions") {
            continue;
        }

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("uses:") && !trimmed.starts_with("- uses:") {
                continue;
            }

            // Extract the action reference
            let action_ref = trimmed
                .trim_start_matches("- ")
                .trim_start_matches("uses:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            // Check if pinned to branch instead of tag/SHA
            if let Some(at_pos) = action_ref.find('@') {
                let action_name = &action_ref[..at_pos];
                let version = &action_ref[at_pos + 1..];

                // Org-internal reusable workflows pinned to main are acceptable
                // (the org controls the workflow source)
                let is_org_internal = action_name.contains("/.github/");

                if !is_org_internal
                    && (version == "main"
                        || version == "master"
                        || version == "latest"
                        || version == "dev")
                {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-953".to_string(),
                        file: rel.clone(),
                        line: i + 1,
                        description: format!(
                            "Action `{}` pinned to branch `{}` — use version tag or SHA",
                            action_name, version
                        ),
                        severity: Severity::Warning,
                    });
                }
            } else if !action_ref.starts_with("./") && !action_ref.is_empty() {
                // No @ at all (not a local action)
                violations.push(CbPatternViolation {
                    pattern_id: "CB-953".to_string(),
                    file: rel.clone(),
                    line: i + 1,
                    description: format!(
                        "Action `{}` has no version pin — add @vX or @SHA",
                        action_ref
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-954: Secret in Plain Text
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb954 plaintext secret.
pub fn detect_cb954_plaintext_secret(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        // Only scan CI/CD and config YAML for secrets, not docs/roadmaps/contracts
        let is_ci_or_config = rel.contains(".github/")
            || rel.contains("docker")
            || rel.contains("deploy")
            || rel.ends_with(".pmat.yaml")
            || rel.ends_with(".pmat-gates.toml")
            || rel.contains("ci/")
            || rel.contains("config");
        if !is_ci_or_config {
            continue;
        }

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            // Honor inline # pmat:ignore directives
            if line.contains("# pmat:ignore") {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(": ") {
                let key = trimmed[..colon_pos].trim().to_lowercase();
                let value = trimmed[colon_pos + 2..].trim();

                // Check if key matches secret patterns but is not in allowlist
                let is_allowlisted = SECRET_KEY_ALLOWLIST.iter().any(|a| key == *a);
                let is_secret_key =
                    !is_allowlisted && SECRET_KEY_PATTERNS.iter().any(|p| key.contains(p));

                if is_secret_key {
                    // Allow references to env vars, secrets, and GHA inherit
                    if value.starts_with("${{")
                        || value.starts_with("${")
                        || value.starts_with("$")
                        || value == "\"\""
                        || value == "''"
                        || value.is_empty()
                        || value == "null"
                        || value == "~"
                        || value == "inherit" // GHA: `secrets: inherit` is standard
                    {
                        continue;
                    }

                    violations.push(CbPatternViolation {
                        pattern_id: "CB-954".to_string(),
                        file: rel.clone(),
                        line: i + 1,
                        description: format!(
                            "Possible plaintext secret in `{}` — use environment variable or secret reference",
                            key
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    violations
}
