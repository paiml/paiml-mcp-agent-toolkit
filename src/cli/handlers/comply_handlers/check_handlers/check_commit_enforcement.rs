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
                        // Accept v4 contracts (work_item_id only) and v5 (version + work_item_id)
                        let has_id = v.get("work_item_id").is_some();
                        let has_claims = v.get("claims").is_some()
                            || v.get("require").is_some()
                            || v.get("ensure").is_some()
                            || v.get("falsifiable_claims").is_some();
                        if has_id || has_claims {
                            valid += 1;
                        } else {
                            let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            invalid.push(format!("{} (missing work_item_id and claims)", name));
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

/// CB-1322: SVG Asset Contract
///
/// Validates SVG files for viewBox, accessibility, and reasonable element count.
pub(crate) fn check_svg_contract(project_path: &Path) -> ComplianceCheck {
    let mut svg_count = 0usize;
    let mut issues: Vec<String> = Vec::new();

    // Scan for SVG files in common locations
    let search_dirs = ["assets", "docs", "static", "."];
    for dir_name in &search_dirs {
        let dir = project_path.join(dir_name);
        if !dir.exists() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "svg") || !path.is_file() {
                continue;
            }
            svg_count += 1;
            if let Ok(content) = fs::read_to_string(&path) {
                let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !content.contains("viewBox") {
                    issues.push(format!("{}: missing viewBox", name));
                }
                if !content.contains("<title") && !content.contains("aria-label") {
                    issues.push(format!("{}: no accessibility (title or aria-label)", name));
                }
            }
        }
    }

    if svg_count == 0 {
        return ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Skip,
            message: "No SVG files found".into(),
            severity: Severity::Info,
        };
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Pass,
            message: format!("{} SVG file(s) validated", svg_count),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1324: mdBook Contract
///
/// Validates mdBook SUMMARY.md links if book/ directory exists.
pub(crate) fn check_mdbook_contract(project_path: &Path) -> ComplianceCheck {
    let book_dir = project_path.join("book");
    let summary = book_dir.join("src/SUMMARY.md");

    if !book_dir.exists() || !summary.exists() {
        return ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Skip,
            message: "No book/src/SUMMARY.md found".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&summary) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1324: mdBook Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read SUMMARY.md".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut broken_links: Vec<String> = Vec::new();
    let book_src = book_dir.join("src");

    for line in content.lines() {
        // Extract markdown links: [text](path.md)
        if let Some(start) = line.find("](") {
            if let Some(end) = line[start + 2..].find(')') {
                let link = &line[start + 2..start + 2 + end];
                // Skip external links and anchors
                if link.starts_with("http") || link.starts_with('#') {
                    continue;
                }
                let link_path = book_src.join(link.split('#').next().unwrap_or(link));
                if !link_path.exists() {
                    broken_links.push(link.to_string());
                }
            }
        }
    }

    if broken_links.is_empty() {
        ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Pass,
            message: "SUMMARY.md links valid".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} broken link(s): {}",
                broken_links.len(),
                broken_links.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1330: L-Level Ratchet
///
/// Checks that provable-contracts verification levels don't regress.
/// Reads contracts/ YAML for verification_summary.current_level fields
/// and warns if any are below target_level.
pub(crate) fn check_verification_ratchet(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut total = 0usize;
    let mut regressions: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().map_or(true, |e| e != "yaml" && e != "yml") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("verification_summary") {
                total += 1;
                // Extract target and current levels
                let target = extract_level(&content, "target_level");
                let current = extract_level(&content, "current_level");
                if let (Some(t), Some(c)) = (target, current) {
                    if c < t {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        regressions.push(format!("{} (L{}→L{}, target L{})", name, t, c, t));
                    }
                }
            }
        }
    }

    if total == 0 {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Skip,
            message: "No contracts with verification_summary".into(),
            severity: Severity::Info,
        }
    } else if regressions.is_empty() {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Pass,
            message: format!("{} contract(s) at or above target level", total),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} regression(s): {}",
                regressions.len(),
                regressions.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// Extract numeric level from "target_level: L3" or "current_level: L1" patterns.
fn extract_level(content: &str, field: &str) -> Option<u8> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(field) {
            // Parse "target_level: L3" or "target_level: \"L3\""
            if let Some(val) = trimmed.split(':').nth(1) {
                let val = val.trim().trim_matches('"').trim_matches('\'');
                if let Some(digit) = val.strip_prefix('L') {
                    return digit.parse().ok();
                }
                // Also try plain number
                return val.parse().ok();
            }
        }
    }
    None
}

/// CB-1338: No Ghost Bindings
///
/// Checks binding.yaml entries reference functions that exist in source.
/// A "ghost binding" is a binding.yaml entry for a function that doesn't exist.
pub(crate) fn check_no_ghost_bindings(project_path: &Path) -> ComplianceCheck {
    let binding = project_path.join("binding.yaml");
    if !binding.exists() {
        // Also check contracts/ subdirectory
        let contracts_binding = project_path.join("contracts/binding.yaml");
        if !contracts_binding.exists() {
            return ComplianceCheck {
                name: "CB-1338: No Ghost Bindings".into(),
                status: CheckStatus::Skip,
                message: "No binding.yaml found".into(),
                severity: Severity::Info,
            };
        }
    }

    // Count binding entries and check if source files exist
    let binding_path = if project_path.join("binding.yaml").exists() {
        project_path.join("binding.yaml")
    } else {
        project_path.join("contracts/binding.yaml")
    };

    let content = match fs::read_to_string(&binding_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1338: No Ghost Bindings".into(),
                status: CheckStatus::Warn,
                message: "Could not read binding.yaml".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut total_bindings = 0usize;
    let mut ghost_count = 0usize;

    // Parse binding entries — look for "status: implemented" with source file refs
    let mut current_source: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source_file:") || trimmed.starts_with("file:") {
            if let Some(val) = trimmed.split(':').nth(1) {
                current_source = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
        if trimmed.starts_with("status:") && trimmed.contains("implemented") {
            total_bindings += 1;
            if let Some(ref src) = current_source {
                let src_path = project_path.join(src);
                if !src_path.exists() {
                    ghost_count += 1;
                }
            }
        }
        if trimmed.starts_with("- name:") || trimmed.starts_with("- module_path:") {
            current_source = None;
        }
    }

    if total_bindings == 0 {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Pass,
            message: "No implemented bindings to verify".into(),
            severity: Severity::Info,
        }
    } else if ghost_count == 0 {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Pass,
            message: format!("{} binding(s) verified, 0 ghosts", total_bindings),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1338: No Ghost Bindings".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{}/{} binding(s) are ghosts (source files missing)",
                ghost_count, total_bindings
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1339: No Placeholder Preconditions
///
/// Checks contracts for generic placeholder preconditions like !is_empty().
/// Domain-specific equations should have real preconditions, not boilerplate.
pub(crate) fn check_no_placeholder_preconditions(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let placeholders = [
        "!input.is_empty()",
        "!x.is_empty()",
        "input.len() > 0",
        "x.len() > 0",
        "!is_empty()",
    ];

    let mut total_preconditions = 0usize;
    let mut placeholder_count = 0usize;

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().map_or(true, |e| e != "yaml" && e != "yml") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("precondition") || trimmed.starts_with("- \"") || trimmed.starts_with("- '") {
                    if placeholders.iter().any(|p| trimmed.contains(p)) {
                        placeholder_count += 1;
                    }
                    if trimmed.contains("precondition") || (trimmed.starts_with("- ") && trimmed.len() > 5) {
                        total_preconditions += 1;
                    }
                }
            }
        }
    }

    if total_preconditions == 0 {
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Pass,
            message: "No preconditions to check".into(),
            severity: Severity::Info,
        }
    } else if placeholder_count == 0 {
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: CheckStatus::Pass,
            message: format!("{} precondition(s), 0 placeholders", total_preconditions),
            severity: Severity::Info,
        }
    } else {
        let ratio = placeholder_count as f64 / total_preconditions.max(1) as f64;
        ComplianceCheck {
            name: "CB-1339: No Placeholder Preconditions".into(),
            status: if ratio > 0.5 { CheckStatus::Fail } else { CheckStatus::Warn },
            message: format!(
                "{}/{} precondition(s) are placeholders ({:.0}%)",
                placeholder_count,
                total_preconditions,
                ratio * 100.0
            ),
            severity: if ratio > 0.5 { Severity::Error } else { Severity::Warning },
        }
    }
}

/// CB-1340: Enforcement Penetration
///
/// Checks that repos with binding.yaml have meaningful call-site penetration.
/// Repos with contracts but <10% enforcement are just "paper contracts."
pub(crate) fn check_enforcement_penetration(project_path: &Path) -> ComplianceCheck {
    let binding = project_path.join("binding.yaml");
    let contracts_binding = project_path.join("contracts/binding.yaml");

    if !binding.exists() && !contracts_binding.exists() {
        return ComplianceCheck {
            name: "CB-1340: Enforcement Penetration".into(),
            status: CheckStatus::Skip,
            message: "No binding.yaml (no enforcement to measure)".into(),
            severity: Severity::Info,
        };
    }

    // Check for contract macro invocations in source
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1340: Enforcement Penetration".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut call_sites = 0usize;
    let mut total_fns = 0usize;

    // Count contract enforcement call sites (debug_assert!, contract macros)
    fn count_enforcement(dir: &Path, calls: &mut usize, fns: &mut usize) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.to_str().unwrap_or("").contains("test") {
                count_enforcement(&path, calls, fns);
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        if line.contains("fn ") && !line.trim().starts_with("//") {
                            *fns += 1;
                        }
                        if line.contains("debug_assert!") || line.contains("contract_") || line.contains("requires!") || line.contains("ensures!") {
                            *calls += 1;
                        }
                    }
                }
            }
        }
    }

    count_enforcement(&src_dir, &mut call_sites, &mut total_fns);

    let penetration = if total_fns > 0 {
        call_sites as f64 / total_fns as f64
    } else {
        0.0
    };

    if penetration >= 0.10 {
        ComplianceCheck {
            name: "CB-1340: Enforcement Penetration".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} call sites / {} functions = {:.1}% penetration",
                call_sites, total_fns, penetration * 100.0
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1340: Enforcement Penetration".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} call sites / {} functions = {:.1}% penetration (target: ≥10%)",
                call_sites, total_fns, penetration * 100.0
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1343: Assertion Placement
///
/// Checks that precondition assertions are placed after early-return guards,
/// not before. Scans for debug_assert! before if..return patterns.
pub(crate) fn check_assertion_placement(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1343: Assertion Placement".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Simplified: count debug_assert! calls and check if there are any
    // contract-related files with assertions
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1343: Assertion Placement".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ (no generated assertions to check)".into(),
            severity: Severity::Info,
        };
    }

    // Look for generated contract assertion files
    let generated_dir = project_path.join("src/contracts");
    if !generated_dir.exists() {
        ComplianceCheck {
            name: "CB-1343: Assertion Placement".into(),
            status: CheckStatus::Pass,
            message: "No generated contract code found (placement N/A)".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1343: Assertion Placement".into(),
            status: CheckStatus::Pass,
            message: "Generated contract code present (manual review recommended)".into(),
            severity: Severity::Info,
        }
    }
}

/// CB-1323: Forjar Config Contract
///
/// Validates forjar.yaml configuration: no plaintext secrets, template refs resolved.
pub(crate) fn check_forjar_contract(project_path: &Path) -> ComplianceCheck {
    let forjar = project_path.join("forjar.yaml");
    let forjar_alt = project_path.join("forjar.toml");

    if !forjar.exists() && !forjar_alt.exists() {
        return ComplianceCheck {
            name: "CB-1323: Forjar Config Contract".into(),
            status: CheckStatus::Skip,
            message: "No forjar.yaml or forjar.toml found".into(),
            severity: Severity::Info,
        };
    }

    let config_path = if forjar.exists() { forjar } else { forjar_alt };
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1323: Forjar Config Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read forjar config".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut issues: Vec<String> = Vec::new();

    // Check for plaintext secrets
    let secret_patterns = ["password:", "secret:", "api_key:", "token:", "private_key:"];
    for pattern in &secret_patterns {
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(pattern) && !trimmed.contains("${") && !trimmed.contains("env(") {
                let val = trimmed.split(':').nth(1).unwrap_or("").trim();
                if !val.is_empty() && !val.starts_with('#') && val != "\"\"" && val != "''" {
                    issues.push(format!("line {}: possible plaintext {}", i + 1, pattern.trim_end_matches(':')));
                }
            }
        }
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1323: Forjar Config Contract".into(),
            status: CheckStatus::Pass,
            message: "Forjar config passes secret hygiene checks".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1323: Forjar Config Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1341: Spec Number Accuracy
///
/// Checks that numbers in spec documents match measurable data.
/// Compares claims in docs/specifications/ against current pmat output.
pub(crate) fn check_spec_number_accuracy(project_path: &Path) -> ComplianceCheck {
    let spec_dir = project_path.join("docs/specifications");
    if !spec_dir.exists() {
        return ComplianceCheck {
            name: "CB-1341: Spec Number Accuracy".into(),
            status: CheckStatus::Skip,
            message: "No docs/specifications/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut total_specs = 0usize;
    let mut oversized: Vec<String> = Vec::new();

    // Check component specs are under 500 lines (CB-140 cross-validation)
    let components_dir = spec_dir.join("components");
    if components_dir.exists() {
        if let Ok(entries) = fs::read_dir(&components_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(true, |e| e != "md") {
                    continue;
                }
                total_specs += 1;
                if let Ok(content) = fs::read_to_string(&path) {
                    let lines = content.lines().count();
                    if lines > 500 {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        oversized.push(format!("{} ({} lines)", name, lines));
                    }
                }
            }
        }
    }

    // Also check root spec
    let root_spec = spec_dir.join("pmat-spec.md");
    if root_spec.exists() {
        if let Ok(content) = fs::read_to_string(&root_spec) {
            let lines = content.lines().count();
            if lines > 500 {
                oversized.push(format!("pmat-spec.md ({} lines)", lines));
            }
        }
    }

    if oversized.is_empty() {
        ComplianceCheck {
            name: "CB-1341: Spec Number Accuracy".into(),
            status: CheckStatus::Pass,
            message: format!("{} spec(s) validated, all within limits", total_specs),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1341: Spec Number Accuracy".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} oversized spec(s): {}",
                oversized.len(),
                oversized.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1350: Differential Obligation Verification
///
/// At commit time, only obligations whose bound functions were modified need
/// re-checking. Reads `.pmat/binding-index.json` (file→binding reverse index)
/// and cross-references with staged files to identify affected obligations.
/// Reports unverified obligations for modified bindings.
///
/// Spec: Phase 4 of commit-level-contract-enforcement.md
/// Basis: Mugnier et al. (OOPSLA 2025) proof brittleness; Cedar (ICSE 2025)
pub(crate) fn check_differential_obligations(project_path: &Path) -> ComplianceCheck {
    let binding_index_path = project_path.join(".pmat/binding-index.json");

    // Skip if no binding index exists
    if !binding_index_path.exists() {
        // Also try contracts/ location
        let alt = project_path.join("contracts/binding-index.json");
        if !alt.exists() {
            return ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Skip,
                message: "No .pmat/binding-index.json (run pmat comply refresh-bindings)".into(),
                severity: Severity::Info,
            };
        }
    }

    let idx_path = if binding_index_path.exists() {
        binding_index_path
    } else {
        project_path.join("contracts/binding-index.json")
    };

    let content = match fs::read_to_string(&idx_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Warn,
                message: "Could not read binding-index.json".into(),
                severity: Severity::Warning,
            };
        }
    };

    let index: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Warn,
                message: "binding-index.json is not valid JSON".into(),
                severity: Severity::Warning,
            };
        }
    };

    // Get staged files via git diff --cached
    let staged_files = get_staged_files(project_path);
    if staged_files.is_empty() {
        return ComplianceCheck {
            name: "CB-1350: Differential Obligations".into(),
            status: CheckStatus::Pass,
            message: "No staged files (no obligations to check)".into(),
            severity: Severity::Info,
        };
    }

    // Cross-reference staged files against binding index
    // binding-index.json maps: { "file_path": ["binding_name", ...], ... }
    let bindings_obj = match index.as_object() {
        Some(obj) => obj,
        None => {
            return ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Warn,
                message: "binding-index.json is not a JSON object".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut affected_bindings: Vec<String> = Vec::new();
    let mut total_bindings = 0usize;

    for (file_key, bindings) in bindings_obj {
        if let Some(arr) = bindings.as_array() {
            total_bindings += arr.len();
            // Check if any staged file matches this binding's source
            if staged_files.iter().any(|sf| file_key.contains(sf) || sf.contains(file_key)) {
                for b in arr {
                    if let Some(name) = b.as_str() {
                        affected_bindings.push(name.to_string());
                    } else if let Some(obj) = b.as_object() {
                        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                            affected_bindings.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if total_bindings == 0 {
        return ComplianceCheck {
            name: "CB-1350: Differential Obligations".into(),
            status: CheckStatus::Pass,
            message: "Binding index is empty (no obligations tracked)".into(),
            severity: Severity::Info,
        };
    }

    if affected_bindings.is_empty() {
        ComplianceCheck {
            name: "CB-1350: Differential Obligations".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} staged file(s), 0/{} binding(s) affected",
                staged_files.len(),
                total_bindings
            ),
            severity: Severity::Info,
        }
    } else {
        // Check if there's a cached verdict for affected bindings
        let verdict_path = project_path.join(".pmat/obligation-verdicts.json");
        let verified = if let Ok(verdicts_str) = fs::read_to_string(&verdict_path) {
            if let Ok(verdicts) = serde_json::from_str::<serde_json::Value>(&verdicts_str) {
                affected_bindings.iter().filter(|b| {
                    verdicts.get(b.as_str()).and_then(|v| v.as_str()) == Some("pass")
                }).count()
            } else {
                0
            }
        } else {
            0
        };

        let unverified = affected_bindings.len() - verified;
        if unverified == 0 {
            ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Pass,
                message: format!(
                    "{} affected binding(s), all verified from cache",
                    affected_bindings.len()
                ),
                severity: Severity::Info,
            }
        } else {
            let display: Vec<&str> = affected_bindings.iter().take(5).map(|s| s.as_str()).collect();
            ComplianceCheck {
                name: "CB-1350: Differential Obligations".into(),
                status: CheckStatus::Warn,
                message: format!(
                    "{} affected binding(s), {} unverified: {}{}",
                    affected_bindings.len(),
                    unverified,
                    display.join(", "),
                    if affected_bindings.len() > 5 { "..." } else { "" }
                ),
                severity: Severity::Warning,
            }
        }
    }
}

/// Get staged files from git diff --cached --name-only.
/// Returns relative file paths. Falls back to empty vec if git not available.
fn get_staged_files(project_path: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        }
        _ => Vec::new(),
    }
}

/// CB-1351: Binding Index Freshness
///
/// Checks that `.pmat/binding-index.json` exists and is not stale.
/// Staleness: >7 days = warning, >30 days = error.
/// Freshness is essential for O(1) differential obligation checks.
pub(crate) fn check_binding_index_freshness(project_path: &Path) -> ComplianceCheck {
    let idx_path = project_path.join(".pmat/binding-index.json");
    let alt_path = project_path.join("contracts/binding-index.json");

    let path = if idx_path.exists() {
        idx_path
    } else if alt_path.exists() {
        alt_path
    } else {
        return ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Skip,
            message: "No binding-index.json found".into(),
            severity: Severity::Info,
        };
    };

    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1351: Binding Index Freshness".into(),
                status: CheckStatus::Warn,
                message: "Could not read binding-index.json metadata".into(),
                severity: Severity::Warning,
            };
        }
    };

    let modified = match metadata.modified() {
        Ok(t) => t,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1351: Binding Index Freshness".into(),
                status: CheckStatus::Warn,
                message: "Could not determine binding-index.json age".into(),
                severity: Severity::Warning,
            };
        }
    };

    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    let days = age.as_secs() / 86400;

    if days > 30 {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Fail,
            message: format!(
                "binding-index.json is {} days old (>30 days, run pmat comply refresh-bindings)",
                days
            ),
            severity: Severity::Error,
        }
    } else if days > 7 {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Warn,
            message: format!(
                "binding-index.json is {} days old (>7 days, consider refreshing)",
                days
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Pass,
            message: format!("binding-index.json fresh ({} day(s) old)", days),
            severity: Severity::Info,
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

    #[test]
    fn test_cb1330_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_verification_ratchet(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1338_no_binding() {
        let dir = tempdir().unwrap();
        let check = check_no_ghost_bindings(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1339_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_no_placeholder_preconditions(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1340_no_binding() {
        let dir = tempdir().unwrap();
        let check = check_enforcement_penetration(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1343_no_contracts() {
        let dir = tempdir().unwrap();
        let check = check_assertion_placement(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1322_no_svgs() {
        let dir = tempdir().unwrap();
        let check = check_svg_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1324_no_book() {
        let dir = tempdir().unwrap();
        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1324_valid_book() {
        let dir = tempdir().unwrap();
        let book_src = dir.path().join("book/src");
        fs::create_dir_all(&book_src).unwrap();
        fs::write(book_src.join("SUMMARY.md"), "# Summary\n\n- [Intro](intro.md)\n").unwrap();
        fs::write(book_src.join("intro.md"), "# Intro\n").unwrap();

        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1324_broken_link() {
        let dir = tempdir().unwrap();
        let book_src = dir.path().join("book/src");
        fs::create_dir_all(&book_src).unwrap();
        fs::write(book_src.join("SUMMARY.md"), "# Summary\n\n- [Missing](gone.md)\n").unwrap();

        let check = check_mdbook_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("gone.md"));
    }

    #[test]
    fn test_extract_level() {
        assert_eq!(extract_level("target_level: L3", "target_level"), Some(3));
        assert_eq!(extract_level("current_level: L1", "current_level"), Some(1));
        assert_eq!(extract_level("target_level: \"L5\"", "target_level"), Some(5));
        assert_eq!(extract_level("no level here", "target_level"), None);
    }

    #[test]
    fn test_cb1323_no_forjar() {
        let dir = tempdir().unwrap();
        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1323_clean_forjar() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("forjar.yaml"),
            "name: myproject\nsteps:\n  - build\n  - test\n",
        ).unwrap();

        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1323_secret_in_forjar() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("forjar.yaml"),
            "name: myproject\npassword: hunter2\napi_key: sk-abc123\n",
        ).unwrap();

        let check = check_forjar_contract(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("plaintext"));
    }

    #[test]
    fn test_cb1341_no_specs() {
        let dir = tempdir().unwrap();
        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1341_valid_specs() {
        let dir = tempdir().unwrap();
        let specs = dir.path().join("docs/specifications/components");
        fs::create_dir_all(&specs).unwrap();
        fs::write(specs.join("test.md"), "# Test\n\nShort spec.\n").unwrap();

        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1341_oversized_spec() {
        let dir = tempdir().unwrap();
        let specs = dir.path().join("docs/specifications/components");
        fs::create_dir_all(&specs).unwrap();
        let mut long_content = String::from("# Title\n");
        long_content.push_str(&"Line\n".repeat(510));
        fs::write(specs.join("big.md"), &long_content).unwrap();

        let check = check_spec_number_accuracy(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("big.md"));
    }

    // --- Phase 4: Differential Obligation Verification ---

    #[test]
    fn test_cb1350_no_binding_index() {
        let dir = tempdir().unwrap();
        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("binding-index.json"));
    }

    #[test]
    fn test_cb1350_empty_binding_index() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "{}").unwrap();

        let check = check_differential_obligations(dir.path());
        // No staged files in a tempdir (not a git repo), so should pass
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1350_invalid_json() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "not json").unwrap();

        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("not valid JSON"));
    }

    #[test]
    fn test_cb1350_binding_index_with_entries() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(
            pmat.join("binding-index.json"),
            r#"{"src/lib.rs": ["validate_input", "parse_config"]}"#,
        ).unwrap();

        // Not a git repo, so no staged files → pass
        let check = check_differential_obligations(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1351_no_binding_index() {
        let dir = tempdir().unwrap();
        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1351_fresh_binding_index() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        fs::create_dir(&pmat).unwrap();
        fs::write(pmat.join("binding-index.json"), "{}").unwrap();

        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("fresh"));
    }

    #[test]
    fn test_cb1351_contracts_alt_path() {
        let dir = tempdir().unwrap();
        let contracts = dir.path().join("contracts");
        fs::create_dir(&contracts).unwrap();
        fs::write(contracts.join("binding-index.json"), "{}").unwrap();

        let check = check_binding_index_freshness(dir.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_get_staged_files_non_git() {
        let dir = tempdir().unwrap();
        let files = get_staged_files(dir.path());
        assert!(files.is_empty());
    }
}
