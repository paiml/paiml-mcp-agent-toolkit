// CB-021: SIMD intrinsics without #[target_feature] detection
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

/// Mark all lines in a function body as protected, starting from the `fn` declaration line.
fn mark_function_body(lines: &[&str], fn_line: usize, protected: &mut HashSet<usize>) {
    let mut depth: usize = 0;
    let mut entered_body = false;
    for k in fn_line..lines.len() {
        depth += lines[k].matches('{').count();
        if depth > 0 {
            entered_body = true;
        }
        depth = depth.saturating_sub(lines[k].matches('}').count());
        protected.insert(k);
        if entered_body && depth == 0 {
            break;
        }
    }
}

pub(super) fn compute_target_feature_protected_lines(lines: &[&str]) -> HashSet<usize> {
    let mut protected = HashSet::new();
    for (i, line) in lines.iter().enumerate() {
        let is_protected = line.trim().starts_with("#[target_feature")
            || (line.contains("#[cfg(") && line.contains("target_feature"));
        if !is_protected {
            continue;
        }
        // Find the function this attribute applies to and mark its body
        for j in i..lines.len() {
            if lines[j].contains("fn ") {
                mark_function_body(lines, j, &mut protected);
                break;
            }
        }
    }
    protected
}

fn check_file_for_simd_violations(entry: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(entry.exists(), "entry must exist: {}", entry.display());
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let lines: Vec<&str> = content.lines().collect();
    let test_lines = compute_test_code_lines(&lines);
    let protected_lines = compute_target_feature_protected_lines(&lines);
    let file_path = entry.display().to_string();
    let mut violations = Vec::new();
    for (line_num, line) in lines.iter().enumerate() {
        if test_lines.contains(&line_num) || protected_lines.contains(&line_num) {
            continue;
        }
        for &(pattern, kind) in SIMD_INTRINSIC_PATTERNS.iter().chain(PORTABLE_SIMD_PATTERNS) {
            if line.contains(pattern) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-021".to_string(),
                    file: file_path.clone(),
                    line: line_num + 1,
                    description: format!("{kind} {pattern} without #[target_feature]"),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}

pub fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return vec![];
    }
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    entries
        .iter()
        .flat_map(|e| check_file_for_simd_violations(e))
        .collect()
}
