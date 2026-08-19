/// CB-1326: Badge Contract
///
/// Checks that README has required badges (CI status, version, license).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Look for patterns that write to .git/hooks OUTSIDE #[cfg(test)]
                    let writes_hooks = content.contains("hooks/pre-commit")
                        || content.contains("hooks/pre-push")
                        || content.contains("hooks/post-commit");
                    if !writes_hooks { continue; }
                    // Scan line-by-line, skipping test modules
                    let mut pending_test = false;
                    let mut in_test_module = false;
                    let mut brace_depth_at_test = 0i32;
                    let mut brace_depth = 0i32;
                    let mut found_prod_write = false;
                    for line in content.lines() {
                        let t = line.trim();
                        if t.contains("#[cfg(test)]") {
                            pending_test = true;
                        }
                        let old_depth = brace_depth;
                        let (opens, closes) = count_braces_outside_literals(line);
                        brace_depth += (opens - closes) as i32;
                        if pending_test && brace_depth > old_depth {
                            in_test_module = true;
                            pending_test = false;
                            brace_depth_at_test = old_depth;
                        }
                        if in_test_module && brace_depth <= brace_depth_at_test {
                            in_test_module = false;
                        }
                        if in_test_module { continue; }
                        if t.starts_with("//") { continue; }
                        // Count write operations targeting hook paths or tmp paths
                        // (for atomic writes). Excludes writes to arbitrary user paths
                        // like output_path (prompt generators mention hooks in strings).
                        let is_write = t.contains("fs::write") || t.contains("write_all")
                            || t.contains("OpenOptions") || t.contains("fs::rename");
                        let targets_hook = t.contains("hook_path") || t.contains("precommit")
                            || t.contains("pre_commit") || t.contains("hooks/")
                            || t.contains("tmp_path");
                        if is_write && targets_hook {
                            found_prod_write = true;
                            break;
                        }
                    }
                    if found_prod_write {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            } else if path.extension().is_some_and(|e| e == "rs") {
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
                    // Detect unescaped template substitution (skip if shell_escape is used)
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(".replace(") && line.contains("{{") {
                            // Skip if the substitution value is escaped
                            if line.contains("shell_escape") || line.contains("escape(") {
                                continue;
                            }
                            // Skip numeric-only substitutions (bool/float .to_string())
                            if line.contains(".to_string()") && !line.contains("mode") && !line.contains("path") {
                                continue;
                            }
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

/// Does this source file write a hook path directly, in production code?
///
/// Split out of `check_hook_atomic_writes`'s directory walk so that neither
/// half carries the other's nesting: the walk is a walk, and this is the
/// judgement. Behaviour is unchanged — a file qualifies when it mentions a
/// hook path, writes with `fs::write`, has no `rename`/`atomic_write` anywhere,
/// and the write is reached outside comments and outside `#[cfg(test)]`.
fn writes_a_hook_non_atomically(content: &str) -> bool {
    let is_hook_code =
        content.contains("hooks/pre-commit") || content.contains("hooks/pre-push");
    if !is_hook_code {
        return false;
    }
    let has_direct_write = content.contains("fs::write");
    let has_atomic = content.contains("rename") || content.contains("atomic_write");
    if !has_direct_write || has_atomic {
        return false;
    }
    let mut pending_test = false;
    let mut in_test_module = false;
    let mut brace_depth_at_test = 0i32;
    let mut brace_depth = 0i32;
    for line in content.lines() {
        let t = line.trim();
        if t.contains("#[cfg(test)]") {
            pending_test = true;
        }
        let old_depth = brace_depth;
        let (opens, closes) = count_braces_outside_literals(line);
        brace_depth += (opens - closes) as i32;
        if pending_test && brace_depth > old_depth {
            in_test_module = true;
            pending_test = false;
            brace_depth_at_test = old_depth;
        }
        if in_test_module && brace_depth <= brace_depth_at_test {
            in_test_module = false;
        }
        if in_test_module || t.starts_with("//") || t.starts_with("///") {
            continue;
        }
        if t.contains("fs::write")
            && (t.contains("hook_path")
                || t.contains("precommit")
                || t.contains("pre_commit")
                || t.contains("hooks/"))
        {
            return true;
        }
    }
    false
}

/// CB-1334: Hook Atomic Writes
///
/// Checks that hook file writes use write-then-rename (atomic) pattern,
/// not direct fs::write to the hook path. Scans src/ for fs::write calls
/// to .git/hooks/ paths without a tmp+rename pattern nearby.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
                continue;
            }
            if !path.extension().is_some_and(|e| e == "rs") {
                continue;
            }
            let path_str = path.to_str().unwrap_or("");
            if path_str.contains("test") || path_str.contains("check_handlers") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if writes_a_hook_non_atomically(&content) {
                let name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                results.push(name);
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
