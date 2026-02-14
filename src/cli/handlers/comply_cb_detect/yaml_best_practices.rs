#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-950 Series: YAML Best Practices Detection
//!
//! Pattern-based YAML defect detection for `pmat comply check`.
//! Targets CI/CD configurations, Kubernetes manifests, and IaC files.
//! Based on: YAML 1.2 spec (Ben-Kiki, Evans & Net, 2009), OWASP secret detection.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for YAML files.
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", ".pmat", "vendor", "build", "dist",
    "__pycache__", ".venv",
];

/// YAML "truthy" strings that cause subtle bugs when unquoted.
const TRUTHY_STRINGS: &[&str] = &[
    "yes", "no", "on", "off", "true", "false",
    "Yes", "No", "On", "Off", "True", "False",
    "YES", "NO", "ON", "OFF", "TRUE", "FALSE",
    "y", "n", "Y", "N",
];

/// CI/CD YAML keys that legitimately require native booleans (not quoted strings).
/// GitHub Actions: `if`, `fail-fast`, `continue-on-error`, `required`, `cancel-in-progress`
/// GitLab CI: `allow_failure`
/// Kubernetes: `readOnly`, `privileged`
const NATIVE_BOOLEAN_KEYS: &[&str] = &[
    // GitHub Actions
    "if", "fail-fast", "continue-on-error", "required", "cancel-in-progress",
    // GitLab CI
    "allow_failure",
    // Kubernetes
    "readOnly", "privileged",
    // PMAT roadmap schema (boolean fields parsed as native bool)
    "active", "draft",
];

/// Secret-indicating key patterns (case-insensitive).
const SECRET_KEY_PATTERNS: &[&str] = &[
    "password", "secret", "token", "api_key", "apikey", "api-key",
    "private_key", "privatekey", "private-key", "access_key", "accesskey",
    "aws_secret", "credentials", "auth_token",
];

/// Known non-secret keys that contain secret-pattern substrings (e.g. "token").
/// These are common ML/LLM inference parameters and permission scopes, not credentials.
const SECRET_KEY_ALLOWLIST: &[&str] = &[
    "max_tokens",
    "num_tokens",
    "context_tokens",
    "token_limit",
    "total_tokens",
    "completion_tokens",
    "prompt_tokens",
    "max_output_tokens",
    "max_new_tokens",
    "token_count",
    "tokens_per_second",
    // GitHub Actions permission scopes (not secrets)
    "id-token",
    "id_token",
];

// =============================================================================
// File walking
// =============================================================================

/// Walk directory recursively for `.yaml`/`.yml` files.
pub fn walkdir_yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_yaml_recursive(dir, &mut files);
    files
}

fn walk_yaml_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_yaml_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "yaml" | "yml"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Compute production lines (strip YAML comments).
pub fn compute_yaml_production_lines(content: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Strip inline comments (but not inside quotes)
        let line_content = strip_yaml_inline_comment(trimmed);
        if !line_content.is_empty() {
            result.push((i + 1, line_content));
        }
    }
    result
}

fn strip_yaml_inline_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double => {
                // Must be preceded by whitespace
                if i > 0 && bytes[i - 1] == b' ' {
                    return line[..i].trim_end().to_string();
                }
            }
            _ => {}
        }
    }
    line.to_string()
}

// =============================================================================
// CB-950: Truthy String Ambiguity
// =============================================================================

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

        if max_depth > 8 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-951".to_string(),
                file: rel.clone(),
                line: max_depth_line,
                description: format!(
                    "Excessive nesting depth {} (threshold: 8) — consider restructuring",
                    max_depth
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

pub fn detect_cb952_missing_required_fields(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        // Only check GitHub Actions workflows
        let rel_path = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path);
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

pub fn detect_cb953_unpinned_action(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_yaml_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let rel_path = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path);
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
                let version = &action_ref[at_pos + 1..];
                if version == "main"
                    || version == "master"
                    || version == "latest"
                    || version == "dev"
                {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-953".to_string(),
                        file: rel.clone(),
                        line: i + 1,
                        description: format!(
                            "Action `{}` pinned to branch `{}` — use version tag or SHA",
                            &action_ref[..at_pos],
                            version
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

pub fn detect_cb954_plaintext_secret(project_path: &Path) -> Vec<CbPatternViolation> {
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
                let is_allowlisted = SECRET_KEY_ALLOWLIST
                    .iter()
                    .any(|a| key == *a);
                let is_secret_key = !is_allowlisted
                    && SECRET_KEY_PATTERNS.iter().any(|p| key.contains(p));

                if is_secret_key {
                    // Allow references to env vars or secrets
                    if value.starts_with("${{")
                        || value.starts_with("${")
                        || value.starts_with("$")
                        || value == "\"\""
                        || value == "''"
                        || value.is_empty()
                        || value == "null"
                        || value == "~"
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
