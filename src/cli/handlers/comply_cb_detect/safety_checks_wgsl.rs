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

#[cfg(test)]
mod safety_checks_wgsl_tests {
    //! Covers safety_checks_wgsl.rs (86 uncov on broad, 0% cov).
    use super::*;

    // ── has_bounds_check_nearby ──

    #[test]
    fn test_has_bounds_check_nearby_if_less_than_within_5_lines_true() {
        let lines = vec!["if i < 10 {", "    // body", "arr[i]"];
        assert!(has_bounds_check_nearby(&lines, 2));
    }

    #[test]
    fn test_has_bounds_check_nearby_if_gte_within_5_lines_true() {
        let lines = vec!["if i >= 0 {", "    // body", "arr[i]"];
        assert!(has_bounds_check_nearby(&lines, 2));
    }

    #[test]
    fn test_has_bounds_check_nearby_no_if_returns_false() {
        let lines = vec!["let x = 1;", "let y = 2;", "arr[i]"];
        assert!(!has_bounds_check_nearby(&lines, 2));
    }

    #[test]
    fn test_has_bounds_check_nearby_if_without_comparison_returns_false() {
        // `if` but no `<` nor `>=` → false.
        let lines = vec!["if cond {", "arr[i]"];
        assert!(!has_bounds_check_nearby(&lines, 1));
    }

    #[test]
    fn test_has_bounds_check_nearby_out_of_window_returns_false() {
        // bounds check 6 lines back → outside the 5-line window.
        let lines = vec![
            "if i < 10 {",
            "a", "b", "c", "d", "e", "arr[i]",
        ];
        assert!(!has_bounds_check_nearby(&lines, 6));
    }

    // ── check_wgsl_file_for_bounds_violations ──

    #[test]
    fn test_check_bounds_violations_array_without_guard_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.wgsl");
        std::fs::write(&f, "let x = arr[i];\n").unwrap();
        let v = check_wgsl_file_for_bounds_violations(&f);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-001");
    }

    #[test]
    fn test_check_bounds_violations_with_guard_not_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.wgsl");
        std::fs::write(&f, "if i < 10 {\n    let x = arr[i];\n}\n").unwrap();
        let v = check_wgsl_file_for_bounds_violations(&f);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_bounds_violations_missing_file_returns_empty() {
        let missing = std::path::Path::new("/tmp/pmat_missing_wgsl_xyz.wgsl");
        let v = check_wgsl_file_for_bounds_violations(missing);
        assert!(v.is_empty());
    }

    // ── walkdir_wgsl_files ──

    #[test]
    fn test_walkdir_wgsl_files_empty_dir_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let out = walkdir_wgsl_files(tmp.path()).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_walkdir_wgsl_files_finds_nested_wgsl_ignores_others() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.wgsl"), "").unwrap();
        let nested = tmp.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("b.wgsl"), "").unwrap();
        std::fs::write(nested.join("c.rs"), "").unwrap();
        let out = walkdir_wgsl_files(tmp.path()).unwrap();
        assert_eq!(out.len(), 2, "2 .wgsl files, 0 .rs");
    }

    // ── check_line_for_barrier ──

    #[test]
    fn test_check_line_for_barrier_not_in_conditional_returns_none() {
        let v = check_line_for_barrier("workgroupBarrier();", false, 0, "a.wgsl");
        assert!(v.is_none());
    }

    #[test]
    fn test_check_line_for_barrier_in_conditional_workgroup_detected() {
        let v = check_line_for_barrier("workgroupBarrier();", true, 5, "a.wgsl");
        let v = v.unwrap();
        assert_eq!(v.pattern_id, "CB-002");
        assert_eq!(v.line, 6);
        assert!(matches!(v.severity, Severity::Critical));
    }

    #[test]
    fn test_check_line_for_barrier_in_conditional_storage_detected() {
        let v = check_line_for_barrier("storageBarrier();", true, 0, "a.wgsl");
        assert!(v.is_some());
    }

    #[test]
    fn test_check_line_for_barrier_no_barrier_keyword_in_conditional_none() {
        let v = check_line_for_barrier("let x = 1;", true, 0, "a.wgsl");
        assert!(v.is_none());
    }

    // ── check_wgsl_file_for_barrier_divergence ──

    #[test]
    fn test_barrier_divergence_missing_file_returns_empty() {
        let missing = std::path::Path::new("/tmp/pmat_missing_wgsl_b.wgsl");
        let v = check_wgsl_file_for_barrier_divergence(missing);
        assert!(v.is_empty());
    }

    #[test]
    fn test_barrier_divergence_barrier_inside_if_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("b.wgsl");
        std::fs::write(&f, "if cond {\n    workgroupBarrier();\n}\n").unwrap();
        let v = check_wgsl_file_for_barrier_divergence(&f);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-002");
    }

    #[test]
    fn test_barrier_divergence_barrier_outside_conditional_not_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("b.wgsl");
        std::fs::write(&f, "fn kernel() {\n    workgroupBarrier();\n}\n").unwrap();
        let v = check_wgsl_file_for_barrier_divergence(&f);
        assert!(v.is_empty());
    }

    // ── detect_cb001 + detect_cb002 top-level: missing dirs → empty ──

    #[test]
    fn test_detect_cb001_missing_dirs_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_cb001_wgsl_no_bounds_check(tmp.path());
        assert!(v.is_empty());
    }

    #[test]
    fn test_detect_cb002_missing_dirs_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_cb002_wgsl_barrier_divergence(tmp.path());
        assert!(v.is_empty());
    }
}
