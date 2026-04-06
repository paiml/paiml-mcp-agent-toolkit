// CB-001 and CB-002: WGSL bounds checking and barrier divergence detection
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

/// Check if any of the preceding 5 lines contain a bounds check (an `if` with `<` or `>=`).
fn has_bounds_check_nearby(content_lines: &[&str], line_num: usize) -> bool {
    content_lines[..line_num]
        .iter()
        .rev()
        .take(5)
        .any(|l| l.contains("if") && (l.contains('<') || l.contains(">=")))
}

/// Check a single WGSL file for array accesses without preceding bounds checks (CB-001).
fn check_wgsl_file_for_bounds_violations(entry: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(entry.exists(), "entry must exist: {}", entry.display());
    let mut violations = Vec::new();
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return violations,
    };
    let content_lines: Vec<&str> = content.lines().collect();
    let file_path = entry.display().to_string();

    for (line_num, line) in content_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains('[')
            && trimmed.contains(']')
            && !has_bounds_check_nearby(&content_lines, line_num)
        {
            violations.push(CbPatternViolation {
                pattern_id: "CB-001".to_string(),
                file: file_path.clone(),
                line: line_num + 1,
                description: "WGSL array access without bounds check".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// Scan for CB-001 (WGSL without bounds checking)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb001_wgsl_no_bounds_check(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    // Look for .wgsl files
    let src_dir = project_path.join("src");
    let shaders_dir = project_path.join("shaders");

    for dir in [src_dir, shaders_dir] {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = walkdir_wgsl_files(&dir) {
            for entry in entries {
                violations.extend(check_wgsl_file_for_bounds_violations(&entry));
            }
        }
    }

    violations
}

pub(super) fn walkdir_wgsl_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_wgsl_files(&path)?);
        } else if path.extension().map(|e| e == "wgsl").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Check a single line for barrier usage inside a conditional block (CB-002).
/// Returns a violation if the line contains a barrier call while `in_conditional` is true.
fn check_line_for_barrier(
    trimmed: &str,
    in_conditional: bool,
    line_num: usize,
    file_path: &str,
) -> Option<CbPatternViolation> {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    if in_conditional
        && (trimmed.contains("workgroupBarrier") || trimmed.contains("storageBarrier"))
    {
        Some(CbPatternViolation {
            pattern_id: "CB-002".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: "WGSL barrier inside conditional (divergence risk)".to_string(),
            severity: Severity::Critical,
        })
    } else {
        None
    }
}

/// Check a single WGSL file for barrier divergence violations (CB-002).
/// Reads the file, walks lines tracking conditional depth, and returns violations.
fn check_wgsl_file_for_barrier_divergence(entry: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(entry.exists(), "entry must exist: {}", entry.display());
    let mut violations = Vec::new();
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return violations,
    };
    let file_path = entry.display().to_string();
    let mut in_conditional = false;
    let mut conditional_depth = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track conditional blocks
        if trimmed.starts_with("if") || trimmed.starts_with("else") {
            in_conditional = true;
        }
        if in_conditional {
            conditional_depth += trimmed.matches('{').count();
            conditional_depth = conditional_depth.saturating_sub(trimmed.matches('}').count());
            if conditional_depth == 0 {
                in_conditional = false;
            }
        }

        // Check for barrier inside conditional
        if let Some(v) = check_line_for_barrier(trimmed, in_conditional, line_num, &file_path) {
            violations.push(v);
        }
    }

    violations
}

/// Scan for CB-002 (WGSL barrier divergence)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb002_wgsl_barrier_divergence(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    let shaders_dir = project_path.join("shaders");

    for dir in [src_dir, shaders_dir] {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = walkdir_wgsl_files(&dir) {
            for entry in entries {
                violations.extend(check_wgsl_file_for_barrier_divergence(&entry));
            }
        }
    }

    violations
}
