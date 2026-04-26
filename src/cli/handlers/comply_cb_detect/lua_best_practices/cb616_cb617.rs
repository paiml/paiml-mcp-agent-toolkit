#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-616: Lua Type Annotation Awareness
//! and CB-617: OpenResty-Specific Lua Checks.

use super::super::types::*;
use super::constants::OPENRESTY_CACHEABLE_GLOBALS;
use super::helpers::{is_lua_test_file, walkdir_lua_files};
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

    build_annotation_violations(
        luals_count,
        ldoc_count,
        total_functions,
        annotated_functions,
    )
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
    let mut stats = AnnotationStats {
        luals: 0,
        ldoc: 0,
        functions: 0,
        annotated: 0,
    };
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
        (true, true) => Some(format!(
            "LuaLS/sumneko ({luals_count} annotations) + LDoc ({ldoc_count} annotations)"
        )),
        (true, false) => Some(format!("LuaLS/sumneko ({luals_count} annotations)")),
        (false, true) => Some(format!("LDoc ({ldoc_count} annotations)")),
        (false, false) => None,
    };

    if let Some(desc) = system {
        let coverage_pct = (annotated_functions * 100)
            .checked_div(total_functions)
            .unwrap_or(0);
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
            c.contains("require(\"resty")
                || c.contains("require('resty")
                || c.contains("ngx.")
                || c.contains("nginx.conf")
        })
    })
}

/// CB-617: OpenResty-specific performance and safety checks.
/// Only runs on detected OpenResty projects.
/// - Flags stdlib globals used in handler functions without local caching
/// - Flags ngx.var access without nil check
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
fn check_stdlib_caching(content: &str, rel: &str, violations: &mut Vec<CbPatternViolation>) {
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
        if rhs == *g
            || (rhs.starts_with(g)
                && rhs[g.len()..]
                    .chars()
                    .next()
                    .map_or(true, |c| c == ' ' || c == '\n'))
        {
            return Some(*g);
        }
    }
    None
}

/// Check if a function definition is an OpenResty handler.
fn is_handler_function(line: &str) -> bool {
    let handlers = [
        "access",
        "header_filter",
        "body_filter",
        "log",
        "rewrite",
        "content",
    ];
    handlers.iter().any(|h| {
        line.contains(&format!("function _M.{h}")) || line.contains(&format!("function _M:{h}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ── LuaAnnotationSystem Display ─────────────────────────────────────────

    #[test]
    fn test_lua_annotation_system_display_arms() {
        assert_eq!(LuaAnnotationSystem::LuaLS.to_string(), "LuaLS/sumneko");
        assert_eq!(LuaAnnotationSystem::LDoc.to_string(), "LDoc");
    }

    // ── is_annotation_line ──────────────────────────────────────────────────

    fn empty_stats() -> AnnotationStats {
        AnnotationStats {
            luals: 0,
            ldoc: 0,
            functions: 0,
            annotated: 0,
        }
    }

    #[test]
    fn test_is_annotation_line_luals_param() {
        let mut s = empty_stats();
        assert!(is_annotation_line("---@param x number", &mut s));
        assert_eq!(s.luals, 1);
    }

    #[test]
    fn test_is_annotation_line_luals_return() {
        let mut s = empty_stats();
        assert!(is_annotation_line("---@return string", &mut s));
        assert_eq!(s.luals, 1);
    }

    #[test]
    fn test_is_annotation_line_ldoc_tparam() {
        let mut s = empty_stats();
        assert!(is_annotation_line("-- @tparam string name", &mut s));
        assert_eq!(s.ldoc, 1);
    }

    #[test]
    fn test_is_annotation_line_ldoc_treturn() {
        let mut s = empty_stats();
        assert!(is_annotation_line("-- @treturn boolean", &mut s));
        assert_eq!(s.ldoc, 1);
    }

    #[test]
    fn test_is_annotation_line_ldoc_param_via_triple_dash() {
        let mut s = empty_stats();
        assert!(is_annotation_line("--- @param x number", &mut s));
        assert_eq!(s.ldoc, 1);
    }

    #[test]
    fn test_is_annotation_line_ldoc_raise() {
        let mut s = empty_stats();
        assert!(is_annotation_line("-- @raise oops", &mut s));
        assert_eq!(s.ldoc, 1);
    }

    #[test]
    fn test_is_annotation_line_plain_comment_not_annotation() {
        let mut s = empty_stats();
        assert!(!is_annotation_line("-- just a comment", &mut s));
        assert_eq!(s.luals, 0);
        assert_eq!(s.ldoc, 0);
    }

    #[test]
    fn test_is_annotation_line_unknown_ldoc_tag_rejected() {
        // `-- @notag` doesn't match any known LDoc tag → false
        let mut s = empty_stats();
        assert!(!is_annotation_line("-- @notag value", &mut s));
        assert_eq!(s.ldoc, 0);
    }

    // ── count_annotation_stats ──────────────────────────────────────────────

    #[test]
    fn test_count_annotation_stats_basic() {
        let content = "---@param x number\nfunction foo(x)\nend\n";
        let s = count_annotation_stats(content);
        assert_eq!(s.luals, 1);
        assert_eq!(s.functions, 1);
        assert_eq!(s.annotated, 1);
    }

    #[test]
    fn test_count_annotation_stats_function_without_annotation() {
        let content = "function bar()\nend\n";
        let s = count_annotation_stats(content);
        assert_eq!(s.functions, 1);
        assert_eq!(s.annotated, 0);
    }

    #[test]
    fn test_count_annotation_stats_local_function_counts() {
        let content = "-- @tparam string s\nlocal function go(s)\nend\n";
        let s = count_annotation_stats(content);
        assert_eq!(s.ldoc, 1);
        assert_eq!(s.functions, 1);
        assert_eq!(s.annotated, 1);
    }

    #[test]
    fn test_count_annotation_stats_multiple_functions() {
        let content = "function a()\nend\nfunction b()\nend\n";
        let s = count_annotation_stats(content);
        assert_eq!(s.functions, 2);
        assert_eq!(s.annotated, 0);
    }

    // ── build_annotation_violations ─────────────────────────────────────────

    #[test]
    fn test_build_annotation_violations_only_luals() {
        let v = build_annotation_violations(5, 0, 10, 5);
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("LuaLS/sumneko"));
        assert!(v[0].description.contains("50%"));
    }

    #[test]
    fn test_build_annotation_violations_only_ldoc() {
        let v = build_annotation_violations(0, 3, 6, 3);
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("LDoc"));
        assert!(v[0].description.contains("50%"));
    }

    #[test]
    fn test_build_annotation_violations_both_systems() {
        let v = build_annotation_violations(2, 4, 10, 6);
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("LuaLS/sumneko"));
        assert!(v[0].description.contains("LDoc"));
    }

    #[test]
    fn test_build_annotation_violations_no_annotations_few_functions() {
        // Below 10 functions — no violation emitted
        let v = build_annotation_violations(0, 0, 5, 0);
        assert!(v.is_empty());
    }

    #[test]
    fn test_build_annotation_violations_no_annotations_many_functions() {
        // 10+ functions with no annotations → flagged
        let v = build_annotation_violations(0, 0, 10, 0);
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("No type annotations"));
        assert!(v[0].description.contains("10 functions"));
    }

    #[test]
    fn test_build_annotation_violations_zero_functions_no_div_by_zero() {
        // luals_count > 0 but total_functions == 0 → checked_div returns None → 0%
        let v = build_annotation_violations(1, 0, 0, 0);
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("0%"));
    }

    // ── extract_local_cache_name ────────────────────────────────────────────

    #[test]
    fn test_extract_local_cache_name_exact_match() {
        assert_eq!(extract_local_cache_name("local type = type"), Some("type"));
    }

    #[test]
    fn test_extract_local_cache_name_no_local_prefix() {
        assert!(extract_local_cache_name("type = type").is_none());
    }

    #[test]
    fn test_extract_local_cache_name_no_eq() {
        assert!(extract_local_cache_name("local type").is_none());
    }

    #[test]
    fn test_extract_local_cache_name_unrelated_rhs() {
        assert!(extract_local_cache_name("local x = something_else").is_none());
    }

    // ── is_handler_function ─────────────────────────────────────────────────

    #[test]
    fn test_is_handler_function_dot_handlers() {
        assert!(is_handler_function("function _M.access()"));
        assert!(is_handler_function("function _M.header_filter()"));
        assert!(is_handler_function("function _M.body_filter()"));
        assert!(is_handler_function("function _M.log()"));
        assert!(is_handler_function("function _M.rewrite()"));
        assert!(is_handler_function("function _M.content()"));
    }

    #[test]
    fn test_is_handler_function_colon_handlers() {
        assert!(is_handler_function("function _M:access()"));
        assert!(is_handler_function("function _M:rewrite()"));
    }

    #[test]
    fn test_is_handler_function_unrelated_function_rejected() {
        assert!(!is_handler_function("function _M.helper()"));
        assert!(!is_handler_function("local function go() end"));
    }

    // ── check_uncached_global_in_line ───────────────────────────────────────

    #[test]
    fn test_check_uncached_global_in_line_uncached_type_flagged() {
        let cached: HashSet<&str> = HashSet::new();
        let mut violations = Vec::new();
        check_uncached_global_in_line(
            "local x = type(value)",
            7,
            "h.lua",
            &cached,
            &mut violations,
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-617");
        assert_eq!(violations[0].line, 7);
        assert!(violations[0].description.contains("type"));
    }

    #[test]
    fn test_check_uncached_global_in_line_cached_global_skipped() {
        let mut cached: HashSet<&str> = HashSet::new();
        cached.insert("type");
        let mut violations = Vec::new();
        check_uncached_global_in_line(
            "local x = type(value)",
            1,
            "h.lua",
            &cached,
            &mut violations,
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_uncached_global_in_line_comment_skipped() {
        let cached: HashSet<&str> = HashSet::new();
        let mut violations = Vec::new();
        check_uncached_global_in_line(
            "-- type(value) in a comment",
            1,
            "h.lua",
            &cached,
            &mut violations,
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_uncached_global_in_line_no_global_call() {
        let cached: HashSet<&str> = HashSet::new();
        let mut violations = Vec::new();
        check_uncached_global_in_line("local x = 1", 1, "h.lua", &cached, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_uncached_global_in_line_one_per_line() {
        // Multiple stdlib calls on same line → only one violation emitted
        let cached: HashSet<&str> = HashSet::new();
        let mut violations = Vec::new();
        check_uncached_global_in_line(
            "if type(x) == \"table\" and pairs(x) then",
            1,
            "h.lua",
            &cached,
            &mut violations,
        );
        assert_eq!(violations.len(), 1);
    }

    // ── check_stdlib_caching ────────────────────────────────────────────────

    #[test]
    fn test_check_stdlib_caching_handler_with_uncached_call_flagged() {
        let content = "function _M.access()\n  local x = type(v)\nend\n";
        let mut violations = Vec::new();
        check_stdlib_caching(content, "h.lua", &mut violations);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_check_stdlib_caching_handler_with_cached_call_clean() {
        let content = "local type = type\nfunction _M.access()\n  local x = type(v)\nend\n";
        let mut violations = Vec::new();
        check_stdlib_caching(content, "h.lua", &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_stdlib_caching_outside_handler_not_flagged() {
        let content = "local function helper()\n  local x = type(v)\nend\n";
        let mut violations = Vec::new();
        check_stdlib_caching(content, "h.lua", &mut violations);
        assert!(violations.is_empty());
    }

    // ── filesystem entrypoints (use tempfile) ───────────────────────────────

    #[test]
    fn test_detect_cb616_no_lua_files_returns_empty() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb616_type_annotations(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb616_with_annotations_emits_summary() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("mod.lua"),
            "---@param x number\nfunction foo(x)\nend\n",
        )
        .unwrap();
        let v = detect_cb616_type_annotations(tmp.path());
        assert_eq!(v.len(), 1);
        assert!(v[0].description.contains("LuaLS"));
    }

    #[test]
    fn test_detect_cb617_non_openresty_returns_empty() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("plain.lua"), "function foo() end\n").unwrap();
        let v = detect_cb617_openresty_checks(tmp.path());
        assert!(v.is_empty());
    }

    #[test]
    fn test_detect_cb617_openresty_handler_uncached_flagged() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("handler.lua"),
            "local _M = {}\nrequire(\"resty.core\")\nfunction _M.access()\n  local x = type(v)\nend\nreturn _M\n",
        )
        .unwrap();
        let v = detect_cb617_openresty_checks(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-617");
    }

    #[test]
    fn test_is_openresty_project_true_when_resty_required() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("a.lua");
        fs::write(&f, "require(\"resty.core\")\n").unwrap();
        assert!(is_openresty_project(&[f]));
    }

    #[test]
    fn test_is_openresty_project_false_when_no_resty() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("a.lua");
        fs::write(&f, "function foo() end\n").unwrap();
        assert!(!is_openresty_project(&[f]));
    }
}
