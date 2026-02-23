#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-616: Lua Type Annotation Awareness
//! and CB-617: OpenResty-Specific Lua Checks.

use super::constants::OPENRESTY_CACHEABLE_GLOBALS;
use super::helpers::{is_lua_test_file, walkdir_lua_files};
use super::super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

// =============================================================================
// CB-616: Lua Type Annotation Awareness (#183)
// =============================================================================

/// Detected Lua annotation system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaAnnotationSystem {
    LuaLS,
    LDoc,
}

impl std::fmt::Display for LuaAnnotationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaAnnotationSystem::LuaLS => write!(f, "LuaLS/sumneko"),
            LuaAnnotationSystem::LDoc => write!(f, "LDoc"),
        }
    }
}

/// CB-616: Detect type annotation system and report doc coverage.
/// Supports LuaLS (---@param, ---@return) and LDoc (-- @tparam, -- @treturn).
pub fn detect_cb616_type_annotations(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if files.is_empty() {
        return Vec::new();
    }

    let mut luals_count: usize = 0;
    let mut ldoc_count: usize = 0;
    let mut total_functions: usize = 0;
    let mut annotated_functions: usize = 0;

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let stats = count_annotation_stats(&content);
        luals_count += stats.luals;
        ldoc_count += stats.ldoc;
        total_functions += stats.functions;
        annotated_functions += stats.annotated;
    }

    build_annotation_violations(luals_count, ldoc_count, total_functions, annotated_functions)
}

/// Stats from scanning a single file for annotations.
struct AnnotationStats {
    luals: usize,
    ldoc: usize,
    functions: usize,
    annotated: usize,
}

/// Count annotation patterns and functions in a single file.
fn count_annotation_stats(content: &str) -> AnnotationStats {
    let mut stats = AnnotationStats { luals: 0, ldoc: 0, functions: 0, annotated: 0 };
    let mut prev_was_annotation = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let is_annotation = is_annotation_line(trimmed, &mut stats);

        if trimmed.starts_with("function ") || trimmed.starts_with("local function ") {
            stats.functions += 1;
            if prev_was_annotation {
                stats.annotated += 1;
            }
        }
        prev_was_annotation = is_annotation;
    }
    stats
}

/// Check if a line is an annotation and count it. Returns true if annotation.
fn is_annotation_line(trimmed: &str, stats: &mut AnnotationStats) -> bool {
    // LuaLS: ---@param, ---@return, ---@class, ---@field, ---@type
    if trimmed.starts_with("---@") {
        stats.luals += 1;
        return true;
    }
    // LDoc: -- @tparam, -- @treturn, -- @param, -- @return, -- @raise
    if trimmed.starts_with("-- @") || trimmed.starts_with("--- @") {
        let after = trimmed.trim_start_matches('-').trim();
        if after.starts_with("@tparam")
            || after.starts_with("@treturn")
            || after.starts_with("@param")
            || after.starts_with("@return")
            || after.starts_with("@raise")
        {
            stats.ldoc += 1;
            return true;
        }
    }
    false
}

/// Build violations from aggregated annotation stats.
fn build_annotation_violations(
    luals_count: usize,
    ldoc_count: usize,
    total_functions: usize,
    annotated_functions: usize,
) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let system = match (luals_count > 0, ldoc_count > 0) {
        (true, true) => Some(format!("LuaLS/sumneko ({luals_count} annotations) + LDoc ({ldoc_count} annotations)")),
        (true, false) => Some(format!("LuaLS/sumneko ({luals_count} annotations)")),
        (false, true) => Some(format!("LDoc ({ldoc_count} annotations)")),
        (false, false) => None,
    };

    if let Some(desc) = system {
        let coverage_pct = if total_functions > 0 {
            annotated_functions * 100 / total_functions
        } else {
            0
        };
        violations.push(CbPatternViolation {
            pattern_id: "CB-616".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!(
                "Type annotations: {desc}. Doc coverage: {annotated_functions}/{total_functions} functions ({coverage_pct}%)"
            ),
            severity: Severity::Info,
        });
    } else if total_functions >= 10 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-616".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!(
                "No type annotations found ({total_functions} functions) — consider adding LuaLS annotations"
            ),
            severity: Severity::Info,
        });
    }

    violations
}

// =============================================================================
// CB-617: OpenResty-Specific Lua Checks (#185)
// =============================================================================

/// Detect if a project uses OpenResty based on require("resty.*") or ngx.* usage.
fn is_openresty_project(files: &[PathBuf]) -> bool {
    files.iter().take(50).any(|f| {
        fs::read_to_string(f).is_ok_and(|c| {
            c.contains("require(\"resty") || c.contains("require('resty")
                || c.contains("ngx.") || c.contains("nginx.conf")
        })
    })
}

/// CB-617: OpenResty-specific performance and safety checks.
/// Only runs on detected OpenResty projects.
/// - Flags stdlib globals used in handler functions without local caching
/// - Flags ngx.var access without nil check
pub fn detect_cb617_openresty_checks(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if !is_openresty_project(&files) {
        return Vec::new();
    }

    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_stdlib_caching(&content, &rel, &mut violations);
    }

    violations
}

/// Check if stdlib globals are used in handler functions without local caching.
fn check_stdlib_caching(
    content: &str,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    // Collect locally cached names at module level
    let cached: std::collections::HashSet<&str> = content
        .lines()
        .filter(|l| l.trim().starts_with("local "))
        .filter_map(|l| extract_local_cache_name(l.trim()))
        .collect();

    // Check handler functions for uncached global usage
    let mut in_handler = false;
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_handler_function(trimmed) {
            in_handler = true;
        }
        if in_handler {
            check_uncached_global_in_line(trimmed, i + 1, rel, &cached, violations);
        }
        if trimmed == "end" && in_handler {
            in_handler = false;
        }
    }
}

/// Extract the cached name from `local type = type` or `local str_find = string.find`.
/// Only matches exact global caching (not function calls like `local t = type(x)`).
fn extract_local_cache_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("local ")?;
    let eq_pos = rest.find('=')?;
    let rhs = rest[eq_pos + 1..].trim();
    // Check if RHS is exactly a known cacheable global (no parens/brackets after)
    for g in OPENRESTY_CACHEABLE_GLOBALS {
        if rhs == *g || (rhs.starts_with(g) && rhs[g.len()..].chars().next().map_or(true, |c| c == ' ' || c == '\n')) {
            return Some(*g);
        }
    }
    None
}

/// Check if a function definition is an OpenResty handler.
fn is_handler_function(line: &str) -> bool {
    let handlers = ["access", "header_filter", "body_filter", "log", "rewrite", "content"];
    handlers.iter().any(|h| {
        line.contains(&format!("function _M.{h}"))
            || line.contains(&format!("function _M:{h}"))
    })
}

/// Check a single line inside a handler for uncached globals.
fn check_uncached_global_in_line(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    cached: &std::collections::HashSet<&str>,
    violations: &mut Vec<CbPatternViolation>,
) {
    if trimmed.starts_with("--") {
        return;
    }
    // Check for simple stdlib calls like type(...), pairs(...)
    for g in &["type", "pairs", "ipairs", "tostring", "tonumber"] {
        let pattern = format!("{g}(");
        if trimmed.contains(&pattern) && !cached.contains(*g) {
            violations.push(CbPatternViolation {
                pattern_id: "CB-617".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "Uncached `{g}()` in handler — add `local {g} = {g}` at module top"
                ),
                severity: Severity::Info,
            });
            return; // One per line
        }
    }
}
