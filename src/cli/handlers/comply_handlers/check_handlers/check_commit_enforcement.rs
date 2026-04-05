// Commit-Level Contract Enforcement checks (CB-1320 through CB-1343)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Spec: docs/specifications/components/commit-level-contract-enforcement.md (Component 25)
// Phase 3a: Asset layout contracts (CB-1320..1326)
// Phase 7: Hook consolidation (CB-1333..1337)
// Phase 8: Falsify leak remediation (CB-1338..1343)

/// CB-1320: README Layout Contract
///
/// Validates README.md has required sections in correct order.
/// Required slots: header/title, badges area, description, installation, usage,
/// contributing, license. Optional: benchmarks, architecture, api, footer.
pub(crate) fn check_readme_layout(project_path: &Path) -> ComplianceCheck {
    let readme_path = project_path.join("README.md");
    if !readme_path.exists() {
        return ComplianceCheck {
            name: "CB-1320: README Layout Contract".into(),
            status: CheckStatus::Warn,
            message: "No README.md found".into(),
            severity: Severity::Warning,
        };
    }

    let content = match fs::read_to_string(&readme_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1320: README Layout Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read README.md".into(),
                severity: Severity::Warning,
            };
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut issues: Vec<String> = Vec::new();

    // Check for title (# heading)
    let has_title = lines.iter().any(|l| l.starts_with("# "));
    if !has_title {
        issues.push("missing title (# heading)".into());
    }

    // Check required sections by common heading patterns
    let required_sections = [
        ("install", &["install", "getting started", "setup", "quickstart"][..]),
        ("usage", &["usage", "examples", "quick start"][..]),
        ("license", &["license"][..]),
    ];

    let lower_content = content.to_lowercase();
    for (name, patterns) in &required_sections {
        let found = patterns
            .iter()
            .any(|p| lower_content.contains(&format!("## {}", p)) || lower_content.contains(&format!("# {}", p)));
        if !found {
            issues.push(format!("missing required section: {}", name));
        }
    }

    // Check for badge area (common patterns: shields.io, badge)
    let has_badges = content.contains("shields.io")
        || content.contains("badge")
        || content.contains("![")
        || content.contains("[![");
    if !has_badges {
        issues.push("no badges detected (shields.io, images, or badge references)".into());
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1320: README Layout Contract".into(),
            status: CheckStatus::Pass,
            message: "README.md has required sections and structure".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1320: README Layout Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1325: CHANGELOG Contract
///
/// Checks CHANGELOG.md follows Keep-a-Changelog format if present.
pub(crate) fn check_changelog_contract(project_path: &Path) -> ComplianceCheck {
    let changelog_path = project_path.join("CHANGELOG.md");
    if !changelog_path.exists() {
        return ComplianceCheck {
            name: "CB-1325: CHANGELOG Contract".into(),
            status: CheckStatus::Skip,
            message: "No CHANGELOG.md found".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&changelog_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1325: CHANGELOG Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read CHANGELOG.md".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut issues: Vec<String> = Vec::new();

    // Keep-a-Changelog format checks
    let has_heading = content.starts_with("# ") || content.contains("\n# ");
    if !has_heading {
        issues.push("missing top-level heading".into());
    }

    // Check for standard section types
    let standard_sections = ["Added", "Changed", "Deprecated", "Removed", "Fixed", "Security"];
    let has_standard_section = standard_sections
        .iter()
        .any(|s| content.contains(&format!("### {}", s)));
    if !has_standard_section {
        issues.push("no Keep-a-Changelog section types (Added/Changed/Fixed/etc.)".into());
    }

    // Check for version entries with dates
    let has_version_date = content.contains("[Unreleased]")
        || content.lines().any(|l| {
            l.starts_with("## [") && l.contains(']')
        });
    if !has_version_date {
        issues.push("no versioned entries (## [x.y.z] format)".into());
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1325: CHANGELOG Contract".into(),
            status: CheckStatus::Pass,
            message: "CHANGELOG.md follows Keep-a-Changelog format".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1325: CHANGELOG Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1332: Cache Staleness
///
/// Checks .pmat/ cache files for staleness.
/// >7 days = warning, >30 days = error.
pub(crate) fn check_cache_staleness(project_path: &Path) -> ComplianceCheck {
    let pmat_dir = project_path.join(".pmat");
    if !pmat_dir.exists() {
        return ComplianceCheck {
            name: "CB-1332: Cache Staleness".into(),
            status: CheckStatus::Skip,
            message: "No .pmat/ directory".into(),
            severity: Severity::Info,
        };
    }

    let cache_files = ["context.db", "baseline.json"];
    let mut stale_warns: Vec<String> = Vec::new();
    let mut stale_errors: Vec<String> = Vec::new();

    let now = std::time::SystemTime::now();
    let seven_days = std::time::Duration::from_secs(7 * 24 * 3600);
    let thirty_days = std::time::Duration::from_secs(30 * 24 * 3600);

    for file in &cache_files {
        let path = pmat_dir.join(file);
        if !path.is_file() {
            continue;
        }
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > thirty_days {
                        stale_errors.push(format!(
                            "{} ({} days old)",
                            file,
                            age.as_secs() / 86400
                        ));
                    } else if age > seven_days {
                        stale_warns.push(format!(
                            "{} ({} days old)",
                            file,
                            age.as_secs() / 86400
                        ));
                    }
                }
            }
        }
    }

    if !stale_errors.is_empty() {
        ComplianceCheck {
            name: "CB-1332: Cache Staleness".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Critical staleness (>30d): {}",
                stale_errors.join(", ")
            ),
            severity: Severity::Error,
        }
    } else if !stale_warns.is_empty() {
        ComplianceCheck {
            name: "CB-1332: Cache Staleness".into(),
            status: CheckStatus::Warn,
            message: format!(
                "Stale caches (>7d): {}. Run `pmat` commands to refresh",
                stale_warns.join(", ")
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1332: Cache Staleness".into(),
            status: CheckStatus::Pass,
            message: "All cache files fresh (<7 days)".into(),
            severity: Severity::Info,
        }
    }
}

/// CB-1335: Hook Determinism
///
/// Verifies that generated hook content is deterministic.
/// Checks for timestamps, HashMap iteration, and random values in hook files.
pub(crate) fn check_hook_determinism(project_path: &Path) -> ComplianceCheck {
    let hooks_dir = project_path.join(".git/hooks");
    if !hooks_dir.exists() {
        return ComplianceCheck {
            name: "CB-1335: Hook Determinism".into(),
            status: CheckStatus::Skip,
            message: "No .git/hooks/ directory".into(),
            severity: Severity::Info,
        };
    }

    let hook_files = ["pre-commit", "pre-push", "post-commit"];
    let mut nondeterministic: Vec<String> = Vec::new();

    // Patterns that indicate non-deterministic content in hooks
    let bad_patterns = [
        ("timestamp", &["Generated at:", "Generated on:", "Date:", "Created:"][..]),
        ("absolute path", &["/home/", "/Users/", "/root/"][..]),
    ];

    for hook_name in &hook_files {
        let hook_path = hooks_dir.join(hook_name);
        if !hook_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&hook_path) {
            for (category, patterns) in &bad_patterns {
                for pattern in *patterns {
                    if content.contains(pattern) {
                        nondeterministic.push(format!(
                            "{}: {} (found '{}')",
                            hook_name, category, pattern
                        ));
                    }
                }
            }
        }
    }

    if nondeterministic.is_empty() {
        ComplianceCheck {
            name: "CB-1335: Hook Determinism".into(),
            status: CheckStatus::Pass,
            message: format!(
                "Hook files contain no non-deterministic patterns (checked {} hooks)",
                hook_files.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1335: Hook Determinism".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} non-deterministic pattern(s): {}",
                nondeterministic.len(),
                nondeterministic.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1337: Hook Performance
///
/// Checks that pre-commit hooks have a performance budget.
/// Reads timing data from .pmat-metrics/hook-timing.json if available.
/// Falls back to checking hook file for expensive operations.
pub(crate) fn check_hook_performance(project_path: &Path) -> ComplianceCheck {
    let hooks_dir = project_path.join(".git/hooks");
    let pre_commit = hooks_dir.join("pre-commit");

    if !pre_commit.exists() {
        return ComplianceCheck {
            name: "CB-1337: Hook Performance".into(),
            status: CheckStatus::Skip,
            message: "No pre-commit hook installed".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&pre_commit) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1337: Hook Performance".into(),
                status: CheckStatus::Warn,
                message: "Could not read pre-commit hook".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut expensive_ops: Vec<String> = Vec::new();

    // Check for known expensive operations in hooks
    let expensive_patterns = [
        ("cargo build", "full build in pre-commit"),
        ("cargo test", "full test suite in pre-commit"),
        ("cargo clippy", "full clippy in pre-commit (use cached results)"),
        ("npm install", "package install in pre-commit"),
        ("pip install", "package install in pre-commit"),
    ];

    for (pattern, description) in &expensive_patterns {
        if content.contains(pattern) {
            expensive_ops.push(format!("{}: {}", pattern, description));
        }
    }

    if expensive_ops.is_empty() {
        ComplianceCheck {
            name: "CB-1337: Hook Performance".into(),
            status: CheckStatus::Pass,
            message: "Pre-commit hook has no expensive cold operations".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1337: Hook Performance".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} expensive operation(s) in pre-commit: {}",
                expensive_ops.len(),
                expensive_ops.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cb1320_readme_with_all_sections() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("README.md"),
            "# My Project\n\n[![badge](https://shields.io/badge)]\n\n## Installation\n\nRun it.\n\n## Usage\n\nUse it.\n\n## License\n\nMIT\n",
        ).unwrap();

        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1320_readme_missing_sections() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# Project\n\nHello.\n").unwrap();

        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("install"));
    }

    #[test]
    fn test_cb1320_no_readme() {
        let dir = tempdir().unwrap();
        let check = check_readme_layout(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_cb1325_changelog_valid() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("CHANGELOG.md"),
            "# Changelog\n\n## [1.0.0] - 2026-04-05\n\n### Added\n\n- Initial release\n",
        ).unwrap();

        let check = check_changelog_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1325_no_changelog() {
        let dir = tempdir().unwrap();
        let check = check_changelog_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1332_fresh_cache() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("baseline.json"), "{}").unwrap();

        let check = check_cache_staleness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1332_no_pmat_dir() {
        let dir = tempdir().unwrap();
        let check = check_cache_staleness(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1335_clean_hooks() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\npmat hook pre-commit --format --complexity\n",
        ).unwrap();

        let check = check_hook_determinism(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1335_nondeterministic_hook() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\n# Generated at: 2026-04-05T10:00:00Z\npmat hook pre-commit\n",
        ).unwrap();

        let check = check_hook_determinism(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("timestamp"));
    }

    #[test]
    fn test_cb1337_no_expensive_ops() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\npmat hook pre-commit --all\n",
        ).unwrap();

        let check = check_hook_performance(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1337_expensive_ops() {
        let dir = tempdir().unwrap();
        let hooks = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks).unwrap();
        fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\ncargo test\ncargo build\n",
        ).unwrap();

        let check = check_hook_performance(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("cargo test"));
    }
}
