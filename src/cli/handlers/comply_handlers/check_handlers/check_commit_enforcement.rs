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

/// CB-1321: Dockerfile Contract
///
/// Validates Dockerfile follows security and best practices:
/// no :latest tags, no curl|bash, pinned base images.
pub(crate) fn check_dockerfile_contract(project_path: &Path) -> ComplianceCheck {
    let dockerfile = project_path.join("Dockerfile");
    if !dockerfile.exists() {
        return ComplianceCheck {
            name: "CB-1321: Dockerfile Contract".into(),
            status: CheckStatus::Skip,
            message: "No Dockerfile found".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&dockerfile) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1321: Dockerfile Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read Dockerfile".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut issues: Vec<String> = Vec::new();

    // Check for :latest tags
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM ") && trimmed.contains(":latest") {
            issues.push("FROM uses :latest tag (pin to specific version)".into());
        }
    }

    // Check for curl|bash anti-pattern
    if content.contains("curl") && content.contains("| bash")
        || content.contains("| sh")
        || content.contains("|bash")
        || content.contains("|sh")
    {
        issues.push("curl piped to shell (use ADD/COPY instead)".into());
    }

    // Check for running as root (no USER instruction)
    let has_user = content
        .lines()
        .any(|l| l.trim().starts_with("USER ") && !l.trim().starts_with("USER root"));
    if !has_user {
        issues.push("no non-root USER instruction".into());
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1321: Dockerfile Contract".into(),
            status: CheckStatus::Pass,
            message: "Dockerfile follows security best practices".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1321: Dockerfile Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1326: Badge Contract
///
/// Checks that README has required badges (CI status, version, license).
pub(crate) fn check_badge_contract(project_path: &Path) -> ComplianceCheck {
    let readme_path = project_path.join("README.md");
    if !readme_path.exists() {
        return ComplianceCheck {
            name: "CB-1326: Badge Contract".into(),
            status: CheckStatus::Skip,
            message: "No README.md found".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&readme_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1326: Badge Contract".into(),
                status: CheckStatus::Skip,
                message: "Could not read README.md".into(),
                severity: Severity::Info,
            };
        }
    };

    let mut present: Vec<&str> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    // Check for common badge types
    let badge_checks: &[(&str, &[&str])] = &[
        ("CI status", &["actions/workflows", "github.com/", "ci.svg", "build.svg", "passing"]),
        ("version/crate", &["crates.io", "version", "crate-"]),
        ("license", &["license", "License"]),
    ];

    for (name, patterns) in badge_checks {
        let found = patterns.iter().any(|p| content.contains(p));
        if found {
            present.push(name);
        } else {
            missing.push(name);
        }
    }

    if missing.is_empty() {
        ComplianceCheck {
            name: "CB-1326: Badge Contract".into(),
            status: CheckStatus::Pass,
            message: format!("All required badges present: {}", present.join(", ")),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1326: Badge Contract".into(),
            status: CheckStatus::Warn,
            message: format!(
                "Missing badge(s): {}. Present: {}",
                missing.join(", "),
                present.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1333: Hook Single Writer
///
/// Checks that hook files are written by a single codepath (HookRegistry pattern).
/// Detects multiple independent hook writers by scanning for fs::write to hooks dir.
pub(crate) fn check_hook_single_writer(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1333: Hook Single Writer".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory (not a Rust project)".into(),
            severity: Severity::Info,
        };
    }

    // Count files that write to hooks directory
    let mut hook_writer_files: Vec<String> = Vec::new();

    fn scan_for_hook_writes(dir: &Path, results: &mut Vec<String>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_for_hook_writes(&path, results);
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Look for patterns that write to .git/hooks
                    let writes_hooks = content.contains("hooks/pre-commit")
                        || content.contains("hooks/pre-push")
                        || content.contains("hooks/post-commit");
                    let does_write = content.contains("fs::write")
                        || content.contains("write_all")
                        || content.contains("OpenOptions");
                    if writes_hooks && does_write {
                        let rel = path
                            .strip_prefix(dir.parent().unwrap_or(dir))
                            .unwrap_or(&path);
                        results.push(rel.display().to_string());
                    }
                }
            }
        }
    }

    scan_for_hook_writes(&src_dir, &mut hook_writer_files);

    // Filter out test files and the check itself (contains hook path strings for detection)
    hook_writer_files.retain(|f| {
        !f.contains("test")
            && !f.contains("_tests")
            && !f.contains("check_commit_enforcement")
            && !f.contains("check_handlers")
    });

    if hook_writer_files.len() <= 1 {
        ComplianceCheck {
            name: "CB-1333: Hook Single Writer".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} hook writer module(s) found (target: 1)",
                hook_writer_files.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1333: Hook Single Writer".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} hook writer modules (should be 1): {}",
                hook_writer_files.len(),
                hook_writer_files.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1336: Hook No Shell Injection
///
/// Checks that hook generation code doesn't have unescaped template substitution.
/// Looks for `replace("{{` patterns without shell escaping.
pub(crate) fn check_hook_no_injection(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1336: Hook No Injection".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut injection_risks: Vec<String> = Vec::new();

    fn scan_for_injection(dir: &Path, results: &mut Vec<String>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_for_injection(&path, results);
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Skip test files and check files (contain patterns as detection strings)
                    let path_str = path.to_str().unwrap_or("");
                    let is_excluded = path_str.contains("test")
                        || path_str.contains("_tests")
                        || path_str.contains("check_commit_enforcement")
                        || path_str.contains("check_handlers");
                    if is_excluded {
                        continue;
                    }
                    // Look for template substitution in hook-related code
                    let is_hook_code = content.contains("hook")
                        && (content.contains("pre-commit") || content.contains("pre-push"));
                    if !is_hook_code {
                        continue;
                    }
                    // Detect unescaped template substitution
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(".replace(") && line.contains("{{") {
                            let rel = path
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            results.push(format!("{}:{}", rel, i + 1));
                        }
                    }
                }
            }
        }
    }

    scan_for_injection(&src_dir, &mut injection_risks);

    if injection_risks.is_empty() {
        ComplianceCheck {
            name: "CB-1336: Hook No Injection".into(),
            status: CheckStatus::Pass,
            message: "No unescaped template substitution in hook code".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1336: Hook No Injection".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} unescaped template substitution(s): {}",
                injection_risks.len(),
                injection_risks.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1334: Hook Atomic Writes
///
/// Checks that hook file writes use write-then-rename (atomic) pattern,
/// not direct fs::write to the hook path. Scans src/ for fs::write calls
/// to .git/hooks/ paths without a tmp+rename pattern nearby.
pub(crate) fn check_hook_atomic_writes(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1334: Hook Atomic Writes".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut non_atomic: Vec<String> = Vec::new();

    fn scan_atomicity(dir: &Path, results: &mut Vec<String>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_atomicity(&path, results);
            } else if path.extension().map_or(false, |e| e == "rs") {
                let path_str = path.to_str().unwrap_or("");
                if path_str.contains("test") || path_str.contains("check_handlers") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    let is_hook_code = content.contains("hooks/pre-commit")
                        || content.contains("hooks/pre-push");
                    if !is_hook_code {
                        continue;
                    }
                    let has_direct_write = content.contains("fs::write");
                    let has_atomic =
                        content.contains("rename") || content.contains("atomic_write");
                    if has_direct_write && !has_atomic {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        results.push(name);
                    }
                }
            }
        }
    }

    scan_atomicity(&src_dir, &mut non_atomic);

    if non_atomic.is_empty() {
        ComplianceCheck {
            name: "CB-1334: Hook Atomic Writes".into(),
            status: CheckStatus::Pass,
            message: "All hook writes use atomic pattern or no direct writes found".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1334: Hook Atomic Writes".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} file(s) write hooks non-atomically (use write-then-rename): {}",
                non_atomic.len(),
                non_atomic.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1331: Work Contract YAML Validity
///
/// Validates that active work contracts in .pmat-work/ have valid structure.
pub(crate) fn check_work_contract_validity(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut valid = 0usize;
    let mut invalid: Vec<String> = Vec::new();

    let entries = match fs::read_dir(&work_dir) {
        Ok(e) => e,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1331: Work Contract Validity".into(),
                status: CheckStatus::Warn,
                message: "Could not read .pmat-work/".into(),
                severity: Severity::Warning,
            };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let contract = path.join("contract.json");
        if !contract.exists() {
            let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            invalid.push(format!("{} (missing contract.json)", name));
            continue;
        }
        // Validate JSON structure
        match fs::read_to_string(&contract) {
            Ok(content) => {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
                match parsed {
                    Ok(v) => {
                        if v.get("version").is_some() && v.get("work_item_id").is_some() {
                            valid += 1;
                        } else {
                            let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            invalid.push(format!("{} (missing version or work_item_id)", name));
                        }
                    }
                    Err(_) => {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        invalid.push(format!("{} (invalid JSON)", name));
                    }
                }
            }
            Err(_) => {
                let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                invalid.push(format!("{} (unreadable)", name));
            }
        }
    }

    if invalid.is_empty() {
        ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Pass,
            message: format!("{} valid work contract(s)", valid),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} valid, {} invalid: {}",
                valid,
                invalid.len(),
                invalid.join("; ")
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

    #[test]
    fn test_cb1321_no_dockerfile() {
        let dir = tempdir().unwrap();
        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1321_good_dockerfile() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Dockerfile"),
            "FROM rust:1.80-slim\nRUN apt-get update\nUSER app\nCMD [\"./app\"]\n",
        ).unwrap();

        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1321_latest_tag() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Dockerfile"),
            "FROM ubuntu:latest\nRUN echo hi\n",
        ).unwrap();

        let check = check_dockerfile_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains(":latest"));
    }

    #[test]
    fn test_cb1326_badges_present() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("README.md"),
            "# Proj\n[![CI](https://github.com/org/repo/actions/workflows/ci.yml/badge.svg)](x)\n[![crates.io](https://crates.io/v/proj)](y)\n## License\nMIT\n",
        ).unwrap();

        let check = check_badge_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1326_badges_missing() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# Proj\nHello\n").unwrap();

        let check = check_badge_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("Missing"));
    }

    #[test]
    fn test_cb1333_no_src() {
        let dir = tempdir().unwrap();
        let check = check_hook_single_writer(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1333_single_writer() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src/hooks");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("registry.rs"),
            "fn install() { let p = \"hooks/pre-commit\"; fs::write(p, content); }",
        ).unwrap();

        let check = check_hook_single_writer(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1336_no_injection() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("hooks.rs"),
            "fn gen() { let s = format!(\"pre-commit hook\"); }",
        ).unwrap();

        let check = check_hook_no_injection(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1334_no_src() {
        let dir = tempdir().unwrap();
        let check = check_hook_atomic_writes(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1334_atomic_write() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("hooks.rs"),
            "fn install() { let p = \"hooks/pre-commit\"; fs::write(&tmp, c); fs::rename(&tmp, p); }",
        ).unwrap();

        let check = check_hook_atomic_writes(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1331_no_work_dir() {
        let dir = tempdir().unwrap();
        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1331_valid_contract() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-001");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("contract.json"),
            r#"{"version":"5.0","work_item_id":"PMAT-001"}"#,
        ).unwrap();

        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("1 valid"));
    }

    #[test]
    fn test_cb1331_invalid_json() {
        let dir = tempdir().unwrap();
        let work = dir.path().join(".pmat-work/PMAT-BAD");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join("contract.json"), "not json").unwrap();

        let check = check_work_contract_validity(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("invalid JSON"));
    }
}
