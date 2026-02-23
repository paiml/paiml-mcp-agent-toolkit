#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Publishing and project configuration checks.
//!
//! - CB-500: Publish Hygiene
//! - CB-503: Clippy Configuration
//! - CB-504: Deny Configuration
//! - CB-505: Workspace Lint Hygiene
//! - CB-509: Feature Gate Coverage

use super::utilities::walkdir_files_with_ext;
use crate::cli::handlers::comply_cb_detect::types::*;
use std::fs;
use std::path::Path;

/// CB-500: Publish Hygiene - missing `exclude` in Cargo.toml
pub fn detect_cb500_publish_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    let has_exclude = content.contains("exclude = [") || content.contains("exclude=[");
    let has_include = content.contains("include = [") || content.contains("include=[");

    if !has_exclude && !has_include {
        violations.push(CbPatternViolation {
            pattern_id: "CB-500".to_string(),
            file: "Cargo.toml".to_string(),
            line: 1,
            description: "Missing `exclude` field - published crate may include unnecessary files"
                .to_string(),
            severity: Severity::Warning,
        });
    }

    if has_include && has_exclude {
        violations.push(CbPatternViolation {
            pattern_id: "CB-500".to_string(),
            file: "Cargo.toml".to_string(),
            line: 1,
            description: "Both `include` and `exclude` present - Cargo ignores `exclude` when `include` is set"
                .to_string(),
            severity: Severity::Warning,
        });
    }

    if has_exclude {
        let critical_patterns = [
            "target/",
            ".profraw",
            ".profdata",
            ".vscode/",
            ".idea/",
            ".pmat",
            "proptest-regressions",
        ];
        let matched = critical_patterns
            .iter()
            .filter(|p| content.contains(*p))
            .count();
        if matched < 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-500".to_string(),
                file: "Cargo.toml".to_string(),
                line: 1,
                description: format!(
                    "Only {matched}/7 critical patterns in exclude (target/, .profraw, .profdata, .vscode/, .idea/, .pmat, proptest-regressions)"
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

/// CB-503: Clippy Configuration - missing .clippy.toml
pub fn detect_cb503_clippy_config(project_path: &Path) -> Vec<CbPatternViolation> {
    let clippy_toml = project_path.join(".clippy.toml");
    let clippy_toml_alt = project_path.join("clippy.toml");
    let mut violations = Vec::new();

    if !clippy_toml.exists() && !clippy_toml_alt.exists() {
        violations.push(CbPatternViolation {
            pattern_id: "CB-503".to_string(),
            file: ".clippy.toml".to_string(),
            line: 0,
            description: "No clippy configuration file found".to_string(),
            severity: Severity::Info,
        });
    } else {
        let path = if clippy_toml.exists() {
            &clippy_toml
        } else {
            &clippy_toml_alt
        };
        if let Ok(content) = fs::read_to_string(path) {
            if !content.contains("disallowed-methods") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-503".to_string(),
                    file: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    line: 0,
                    description: "Clippy config missing `disallowed-methods` section".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-504: Deny Configuration - missing deny.toml for supply chain security
pub fn detect_cb504_deny_config(project_path: &Path) -> Vec<CbPatternViolation> {
    let deny_toml = project_path.join("deny.toml");
    if deny_toml.exists() {
        return Vec::new();
    }
    vec![CbPatternViolation {
        pattern_id: "CB-504".to_string(),
        file: "deny.toml".to_string(),
        line: 0,
        description: "No cargo-deny configuration for supply chain security".to_string(),
        severity: Severity::Info,
    }]
}

/// CB-505: Workspace Lint Hygiene - missing [lints] or [workspace.lints]
pub fn detect_cb505_workspace_lint_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let has_workspace_lints =
        content.contains("[workspace.lints]") || content.contains("[workspace.lints.");
    let has_lints = content.contains("[lints]") || content.contains("[lints.");

    if has_workspace_lints || has_lints {
        return Vec::new();
    }

    vec![CbPatternViolation {
        pattern_id: "CB-505".to_string(),
        file: "Cargo.toml".to_string(),
        line: 1,
        description: "Missing [lints] section - no project-wide lint configuration".to_string(),
        severity: Severity::Warning,
    }]
}

/// CB-509: Feature Gate Coverage - features defined but never tested
pub fn detect_cb509_feature_gate_coverage(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Count features defined
    let in_features = content
        .lines()
        .skip_while(|l| !l.starts_with("[features]"))
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| l.contains('='))
        .count();

    if in_features == 0 {
        return Vec::new();
    }

    // Check for CI matrix testing features
    let ci_dir = project_path.join(".github/workflows");
    let has_feature_matrix = if ci_dir.exists() {
        walkdir_files_with_ext(&ci_dir, "yml")
            .unwrap_or_default()
            .iter()
            .chain(
                walkdir_files_with_ext(&ci_dir, "yaml")
                    .unwrap_or_default()
                    .iter(),
            )
            .any(|f| {
                fs::read_to_string(f)
                    .map(|c| c.contains("features") || c.contains("--features"))
                    .unwrap_or(false)
            })
    } else {
        false
    };

    if !has_feature_matrix && in_features > 3 {
        vec![CbPatternViolation {
            pattern_id: "CB-509".to_string(),
            file: "Cargo.toml".to_string(),
            line: 0,
            description: format!(
                "{in_features} features defined but no CI feature matrix testing detected"
            ),
            severity: Severity::Info,
        }]
    } else {
        Vec::new()
    }
}
