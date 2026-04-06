#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-600: Implicit Globals and CB-601: Nil-Unsafe Access detection.

use super::super::types::*;
use super::detection_helpers::{
    collect_known_locals, count_braces, extract_implicit_global, starts_with_lua_keyword,
};
use super::helpers::{
    compute_lua_production_lines, count_consecutive_field_access, is_in_lua_string,
    is_lua_test_file, walkdir_lua_files,
};
use std::fs;
use std::path::Path;

/// CB-600: Implicit Globals -- assignment without `local` keyword.
/// Based on luacheck W111/W113.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb600_implicit_globals(project_path: &Path) -> Vec<CbPatternViolation> {
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

        // Collect all known local identifiers: function params, for-loop vars, local decls
        let known_locals = collect_known_locals(&prod_lines);

        // Track brace depth: assignments inside { } are table constructor fields, not globals
        let mut brace_depth: i32 = 0;

        for (line_num, trimmed) in &prod_lines {
            let (opens, closes) = count_braces(trimmed);
            // Apply opens before checking (a line like `{ key = val }` starts inside)
            brace_depth += opens;

            if brace_depth <= 0 && !starts_with_lua_keyword(trimmed) {
                if let Some(lhs) = extract_implicit_global(trimmed) {
                    // Skip identifiers known to be local (params, loop vars, local decls)
                    if !known_locals.contains(lhs) {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-600".to_string(),
                            file: rel.clone(),
                            line: *line_num,
                            description: format!(
                                "Implicit global `{lhs}` — missing `local` keyword"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            brace_depth -= closes;
            brace_depth = brace_depth.max(0);
        }
    }

    violations
}

/// CB-601: Nil-Unsafe Access -- chained calls on function returns or deep field access.
/// Based on Luau type system and LuaTaint taint analysis.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb601_nil_unsafe_access(project_path: &Path) -> Vec<CbPatternViolation> {
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

        for (line_num, trimmed) in &prod_lines {
            // Pattern 1: function return chained -- `):` or `).`
            if (trimmed.contains(").") || trimmed.contains("):"))
                && !is_in_lua_string(trimmed, ").")
                && !is_in_lua_string(trimmed, "):")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-601".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Nil-unsafe: chained access on function return value".to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Pattern 2: 3+ consecutive field accesses (a.b.c.d)
            if count_consecutive_field_access(trimmed) >= 4 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-601".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Nil-unsafe: deep field access chain (3+ levels)".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}
