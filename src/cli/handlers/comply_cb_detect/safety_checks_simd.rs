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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb021 simd without target feature.
pub fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
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

#[cfg(test)]
mod safety_checks_simd_tests {
    //! Covers safety_checks_simd.rs pure helpers (55 uncov on broad, 0% cov).
    use super::*;

    // ── mark_function_body: brace tracking ──

    #[test]
    fn test_mark_function_body_single_line_braces_closes_immediately() {
        let lines = vec!["fn inline() { body(); }", "after();"];
        let mut protected: HashSet<usize> = HashSet::new();
        mark_function_body(&lines, 0, &mut protected);
        assert!(protected.contains(&0));
        assert!(!protected.contains(&1), "after() line must not be protected");
    }

    #[test]
    fn test_mark_function_body_multi_line_tracks_nested_braces() {
        // fn on line 0; body spans 0..=4; line 5 is outside.
        let lines = vec![
            "fn example() {",
            "    if cond {",
            "        inner();",
            "    }",
            "}",
            "outside();",
        ];
        let mut protected = HashSet::new();
        mark_function_body(&lines, 0, &mut protected);
        for i in 0..=4 {
            assert!(protected.contains(&i), "line {i} must be protected");
        }
        assert!(!protected.contains(&5));
    }

    // ── compute_target_feature_protected_lines ──

    #[test]
    fn test_compute_target_feature_protected_lines_empty_when_no_attr() {
        let lines = vec!["fn foo() {}", "fn bar() {}"];
        assert!(compute_target_feature_protected_lines(&lines).is_empty());
    }

    #[test]
    fn test_compute_target_feature_protected_lines_marks_target_feature_fn_body() {
        let lines = vec![
            "#[target_feature(enable = \"avx2\")]",
            "unsafe fn avx_fn() {",
            "    _mm256_add_ps(a, b);",
            "}",
            "fn other() {}",
        ];
        let prot = compute_target_feature_protected_lines(&lines);
        // Lines 1..=3 (the fn body) must be protected.
        for i in 1..=3 {
            assert!(prot.contains(&i), "line {i} must be protected");
        }
    }

    #[test]
    fn test_compute_target_feature_protected_lines_handles_cfg_target_feature() {
        // `#[cfg(... target_feature ...)]` also counts as protection.
        let lines = vec![
            "#[cfg(target_feature = \"avx2\")]",
            "fn avx_fn() {",
            "    body();",
            "}",
        ];
        let prot = compute_target_feature_protected_lines(&lines);
        assert!(prot.contains(&1));
        assert!(prot.contains(&3));
    }

    // ── detect_cb021 top-level: missing src/ → empty ──

    #[test]
    fn test_detect_cb021_missing_src_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_cb021_simd_without_target_feature(tmp.path());
        assert!(v.is_empty());
    }

    // ── check_file_for_simd_violations: read error = empty ──

    #[test]
    fn test_check_file_for_simd_violations_missing_file_returns_empty() {
        let missing = std::path::Path::new("/tmp/pmat_nope_xyz_0xC0FFEE.rs");
        let v = check_file_for_simd_violations(missing);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_file_for_simd_violations_clean_file_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.rs");
        std::fs::write(&f, "fn add(a: f32, b: f32) -> f32 { a + b }\n").unwrap();
        let v = check_file_for_simd_violations(&f);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_file_for_simd_violations_protected_fn_body_not_flagged() {
        // SIMD intrinsic inside a #[target_feature]-protected fn → no violation.
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.rs");
        std::fs::write(
            &f,
            "#[target_feature(enable = \"avx2\")]\nunsafe fn x() {\n    _mm256_add_ps(a, b);\n}\n",
        )
        .unwrap();
        let v = check_file_for_simd_violations(&f);
        assert!(
            v.is_empty(),
            "protected fn body must suppress CB-021: {v:?}"
        );
    }
}
