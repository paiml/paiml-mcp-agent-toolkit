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
