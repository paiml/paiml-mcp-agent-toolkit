#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-604: Unused Variables and CB-605: String Concat in Loop detection.

use super::super::types::*;
use super::detection_helpers::collect_local_declarations;
use super::helpers::{
    compute_lua_production_lines, contains_concat_operator, contains_identifier, is_in_lua_string,
    is_lua_test_file, walkdir_lua_files,
};
use std::fs;
use std::path::Path;

/// CB-604: Unused Variables -- `local var = ...` where var is never referenced again.
/// Based on luacheck W211.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb604_unused_variables(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let declarations = collect_local_declarations(&prod_lines);

        for (line_num, var_name) in &declarations {
            let count = prod_lines
                .iter()
                .filter(|(_, l)| contains_identifier(l, var_name))
                .count();
            if count <= 1 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-604".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: format!(
                        "Unused variable `{var_name}` — prefix with `_` if intentional"
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-605: String Concat in Loop -- `..` operator inside for/while/repeat (O(n^2)).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb605_string_concat_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut loop_depth: i32 = 0;

        for (line_num, trimmed) in &prod_lines {
            if trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("repeat")
            {
                loop_depth += 1;
            }

            if loop_depth > 0
                && contains_concat_operator(trimmed)
                && !is_in_lua_string(trimmed, "..")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-605".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "String concatenation (`..`) in loop — O(n²), use table.concat()"
                        .to_string(),
                    severity: Severity::Info,
                });
            }

            if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") {
                loop_depth = (loop_depth - 1).max(0);
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp_with(file: &str, content: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(file), content).unwrap();
        dir
    }

    // ── detect_cb604_unused_variables ───────────────────────────────────────

    #[test]
    fn test_detect_cb604_no_files_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb604_unused_variables(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb604_unused_variable_flagged() {
        let tmp = tmp_with("a.lua", "local unused_var = 42\nreturn 1\n");
        let v = detect_cb604_unused_variables(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-604");
        assert!(v[0].description.contains("unused_var"));
    }

    #[test]
    fn test_detect_cb604_used_variable_clean() {
        let tmp = tmp_with(
            "a.lua",
            "local used_var = 42\nprint(used_var)\nreturn used_var\n",
        );
        let v = detect_cb604_unused_variables(tmp.path());
        // used_var appears 3 times → not flagged
        assert!(!v.iter().any(|x| x.description.contains("used_var")));
    }

    #[test]
    fn test_detect_cb604_test_files_skipped() {
        // *_spec.lua / *_test.lua are skipped by is_lua_test_file
        let tmp = tmp_with("a_spec.lua", "local unused = 1\n");
        // Test files aren't analyzed for CB-604 — should be empty
        let v = detect_cb604_unused_variables(tmp.path());
        assert!(v.is_empty());
    }

    // ── detect_cb605_string_concat_in_loop ──────────────────────────────────

    #[test]
    fn test_detect_cb605_no_files_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb605_string_concat_in_loop(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb605_concat_in_for_loop_flagged() {
        let tmp = tmp_with(
            "a.lua",
            "local s = \"\"\nfor i = 1, 10 do\n  s = s .. i\nend\n",
        );
        let v = detect_cb605_string_concat_in_loop(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-605");
    }

    #[test]
    fn test_detect_cb605_concat_in_while_loop_flagged() {
        let tmp = tmp_with(
            "a.lua",
            "local s = \"\"\nlocal i = 0\nwhile i < 5 do\n  s = s .. i\n  i = i + 1\nend\n",
        );
        let v = detect_cb605_string_concat_in_loop(tmp.path());
        assert!(!v.is_empty());
    }

    #[test]
    fn test_detect_cb605_concat_in_repeat_loop_flagged() {
        let tmp = tmp_with(
            "a.lua",
            "local s = \"\"\nlocal i = 0\nrepeat\n  s = s .. i\n  i = i + 1\nuntil i > 3\n",
        );
        let v = detect_cb605_string_concat_in_loop(tmp.path());
        assert!(!v.is_empty());
    }

    #[test]
    fn test_detect_cb605_concat_outside_loop_clean() {
        let tmp = tmp_with("a.lua", "local s = \"a\" .. \"b\"\n");
        assert!(detect_cb605_string_concat_in_loop(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb605_no_concat_in_loop_clean() {
        let tmp = tmp_with("a.lua", "for i = 1, 10 do\n  print(i)\nend\n");
        assert!(detect_cb605_string_concat_in_loop(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb605_loop_depth_tracks_nested() {
        // nested loops; concat inside inner loop still has loop_depth > 0
        let tmp = tmp_with(
            "a.lua",
            "for i = 1, 3 do\n  for j = 1, 3 do\n    local x = i .. j\n  end\nend\n",
        );
        let v = detect_cb605_string_concat_in_loop(tmp.path());
        assert!(!v.is_empty());
    }
}
