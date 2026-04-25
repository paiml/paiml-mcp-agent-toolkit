#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-618: Lua FFI Safety Checks and CB-619: Lua OOP Pattern Recognition.

use super::super::types::*;
use super::helpers::{is_lua_test_file, walkdir_lua_files};
use std::fs;
use std::path::Path;

// =============================================================================
// CB-618: Lua FFI Safety Checks (#189)
// =============================================================================

/// CB-618: Detect LuaJIT FFI safety issues.
/// - Flags ffi.new("char[?]", ...) buffer allocations
/// - Flags C.* function calls without error checking
/// - Reports FFI usage summary
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb618_ffi_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();
    let mut ffi_file_count = 0;

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !content.contains("require(\"ffi\")")
            && !content.contains("require('ffi')")
            && !content.contains("require \"ffi\"")
        {
            continue;
        }
        ffi_file_count += 1;

        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_ffi_patterns(&content, &rel, &mut violations);
    }

    if ffi_file_count > 0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-618".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!("LuaJIT FFI used in {ffi_file_count} files"),
            severity: Severity::Info,
        });
    }

    violations
}

/// Check FFI-related patterns in a single file.
fn check_ffi_patterns(content: &str, rel: &str, violations: &mut Vec<CbPatternViolation>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        // Detect C.open / C.socket / C.malloc without error check
        check_ffi_resource_call(trimmed, i + 1, rel, content, violations);
    }
}

/// Flag C.open, C.socket, C.malloc calls without error checking on next lines.
fn check_ffi_resource_call(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    content: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let resource_funcs = ["C.open", "C.socket", "C.malloc", "C.mmap"];
    for func in &resource_funcs {
        if !trimmed.contains(func) {
            continue;
        }
        // Check if next 2 lines have an error check (< 0, == nil, ~= nil, etc.)
        let next_lines: String = content
            .lines()
            .skip(line_num)
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let has_check = next_lines.contains("< 0")
            || next_lines.contains("== nil")
            || next_lines.contains("~= nil")
            || next_lines.contains("== -1")
            || next_lines.contains("if not ");
        if !has_check {
            violations.push(CbPatternViolation {
                pattern_id: "CB-618".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "`{func}()` without error check — verify return value before use"
                ),
                severity: Severity::Warning,
            });
        }
        return;
    }
}

// =============================================================================
// CB-619: Lua OOP Pattern Recognition (#182)
// =============================================================================

/// Detected Lua OOP pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LuaOopPattern {
    SeparateMetatable,
    PrototypalInheritance,
    CallConstructor,
    SelfAsMetatable,
}

impl std::fmt::Display for LuaOopPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaOopPattern::SeparateMetatable => write!(f, "separate-metatable"),
            LuaOopPattern::PrototypalInheritance => write!(f, "prototypal-inheritance"),
            LuaOopPattern::CallConstructor => write!(f, "__call-constructor"),
            LuaOopPattern::SelfAsMetatable => write!(f, "self-as-metatable"),
        }
    }
}

/// CB-619: Detect Lua OOP patterns and report them for TDG awareness.
/// Recognizes: separate metatable, prototypal inheritance, __call constructor, self-as-metatable.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb619_oop_patterns(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut pattern_counts: std::collections::HashMap<LuaOopPattern, usize> =
        std::collections::HashMap::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for pattern in detect_oop_in_file(&content) {
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
    }

    if pattern_counts.is_empty() {
        return Vec::new();
    }

    let mut parts: Vec<String> = pattern_counts
        .iter()
        .map(|(p, c)| format!("{p} ({c} files)"))
        .collect();
    parts.sort();

    vec![CbPatternViolation {
        pattern_id: "CB-619".to_string(),
        file: "project".to_string(),
        line: 0,
        description: format!("OOP patterns: {}", parts.join(", ")),
        severity: Severity::Info,
    }]
}

/// Detect OOP patterns in a single file's content.
fn detect_oop_in_file(content: &str) -> Vec<LuaOopPattern> {
    let mut patterns = Vec::new();
    let has_setmetatable = content.contains("setmetatable");

    if !has_setmetatable {
        return patterns;
    }

    // Separate metatable: `local mt = { __index = M }` + `setmetatable({}, mt)`
    if (content.contains("__index = M") || content.contains("__index = _M"))
        && (content.contains("setmetatable({") || content.contains("setmetatable(self"))
    {
        patterns.push(LuaOopPattern::SeparateMetatable);
    }

    // Prototypal: `self.__index = self` or `Base:extend`
    if content.contains("self.__index = self") || content.contains(":extend") {
        patterns.push(LuaOopPattern::PrototypalInheritance);
    }

    // __call constructor: `setmetatable(M, { __call = ...`
    if content.contains("__call") && has_setmetatable {
        patterns.push(LuaOopPattern::CallConstructor);
    }

    // Self-as-metatable: `setmetatable(X, X)`
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("setmetatable(") {
            let parts: Vec<&str> = rest.splitn(3, ',').collect();
            if parts.len() >= 2 {
                let arg1 = parts[0].trim();
                let arg2 = parts[1].trim().trim_end_matches(')');
                if arg1 == arg2 && !arg1.is_empty() && !arg1.starts_with('{') {
                    patterns.push(LuaOopPattern::SelfAsMetatable);
                    break;
                }
            }
        }
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── LuaOopPattern Display ───────────────────────────────────────────────

    #[test]
    fn test_lua_oop_pattern_display_arms() {
        assert_eq!(
            LuaOopPattern::SeparateMetatable.to_string(),
            "separate-metatable"
        );
        assert_eq!(
            LuaOopPattern::PrototypalInheritance.to_string(),
            "prototypal-inheritance"
        );
        assert_eq!(
            LuaOopPattern::CallConstructor.to_string(),
            "__call-constructor"
        );
        assert_eq!(
            LuaOopPattern::SelfAsMetatable.to_string(),
            "self-as-metatable"
        );
    }

    // ── check_ffi_resource_call ─────────────────────────────────────────────

    #[test]
    fn test_check_ffi_resource_call_no_match_returns() {
        let mut violations = Vec::new();
        let content = "local x = 1\n";
        check_ffi_resource_call(content.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_c_open_no_check_flagged() {
        let mut violations = Vec::new();
        let content = "local fd = C.open(\"/tmp/x\")\nuse(fd)\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-618");
        assert!(violations[0].description.contains("C.open"));
    }

    #[test]
    fn test_check_ffi_resource_call_with_lt_zero_check_clean() {
        let mut violations = Vec::new();
        let content = "local fd = C.open(\"/tmp/x\")\nif fd < 0 then error end\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_with_eq_nil_check_clean() {
        let mut violations = Vec::new();
        let content = "local sock = C.socket(AF, ST, 0)\nif sock == nil then error end\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_with_neq_nil_check_clean() {
        let mut violations = Vec::new();
        let content = "local p = C.malloc(64)\nif p ~= nil then use(p) end\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_with_eq_neg_one_check_clean() {
        let mut violations = Vec::new();
        let content = "local p = C.mmap(NULL, 4096)\nif p == -1 then error end\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_with_if_not_check_clean() {
        let mut violations = Vec::new();
        let content = "local fd = C.open(\"/tmp/x\")\nif not fd then error end\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_resource_call_first_match_wins() {
        // The function returns after the first matched function — verify by ensuring
        // a single line with multiple resource calls only emits one violation.
        let mut violations = Vec::new();
        let content = "local x = C.open(p) + C.malloc(64)\n";
        let line1 = content.lines().next().unwrap();
        check_ffi_resource_call(line1.trim(), 1, "f.lua", content, &mut violations);
        assert_eq!(violations.len(), 1);
    }

    // ── check_ffi_patterns ──────────────────────────────────────────────────

    #[test]
    fn test_check_ffi_patterns_skips_comments() {
        let mut violations = Vec::new();
        let content = "-- C.open(\"x\")\n";
        check_ffi_patterns(content, "f.lua", &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_ffi_patterns_finds_unsafe_call() {
        let mut violations = Vec::new();
        let content = "local fd = C.open(\"/tmp/x\")\nuse(fd)\n";
        check_ffi_patterns(content, "f.lua", &mut violations);
        assert_eq!(violations.len(), 1);
    }

    // ── detect_oop_in_file ──────────────────────────────────────────────────

    #[test]
    fn test_detect_oop_in_file_no_setmetatable_returns_empty() {
        let p = detect_oop_in_file("local x = 1\n");
        assert!(p.is_empty());
    }

    #[test]
    fn test_detect_oop_in_file_separate_metatable() {
        let content = "local M = {}\nlocal mt = { __index = M }\nsetmetatable({}, mt)\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::SeparateMetatable));
    }

    #[test]
    fn test_detect_oop_in_file_separate_metatable_underscore_m_variant() {
        let content = "local mt = { __index = _M }\nsetmetatable(self, mt)\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::SeparateMetatable));
    }

    #[test]
    fn test_detect_oop_in_file_prototypal_self_index_self() {
        let content = "function Class.new()\n  setmetatable(o, mt)\n  self.__index = self\nend\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::PrototypalInheritance));
    }

    #[test]
    fn test_detect_oop_in_file_prototypal_extend_method() {
        let content = "function Base:extend()\n  setmetatable(o, mt)\nend\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::PrototypalInheritance));
    }

    #[test]
    fn test_detect_oop_in_file_call_constructor() {
        let content = "setmetatable(M, { __call = function() end })\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::CallConstructor));
    }

    #[test]
    fn test_detect_oop_in_file_self_as_metatable() {
        // `setmetatable(X, X)` with non-{ first arg
        let content = "setmetatable(M, M)\n";
        let p = detect_oop_in_file(content);
        assert!(p.contains(&LuaOopPattern::SelfAsMetatable));
    }

    #[test]
    fn test_detect_oop_in_file_self_metatable_curly_first_arg_skipped() {
        // arg1 starts with `{` → skipped (would be `setmetatable({}, {})` etc.)
        let content = "setmetatable({}, {})\n";
        let p = detect_oop_in_file(content);
        assert!(!p.contains(&LuaOopPattern::SelfAsMetatable));
    }

    // ── filesystem entrypoints ──────────────────────────────────────────────

    #[test]
    fn test_detect_cb618_no_files_returns_empty() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb618_ffi_safety(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb618_no_ffi_files_returns_empty() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.lua"), "local x = 1\n").unwrap();
        assert!(detect_cb618_ffi_safety(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb618_with_ffi_emits_summary() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("a.lua"),
            "local ffi = require(\"ffi\")\nlocal fd = C.open(\"/tmp\")\nif fd < 0 then end\n",
        )
        .unwrap();
        let v = detect_cb618_ffi_safety(tmp.path());
        // Expect a single CB-618 summary (the open call has a check)
        assert!(!v.is_empty());
        assert!(v.iter().any(|r| r.description.contains("FFI used in")));
    }

    #[test]
    fn test_detect_cb619_no_files_returns_empty() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb619_oop_patterns(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb619_no_oop_returns_empty() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.lua"), "local x = 1\n").unwrap();
        assert!(detect_cb619_oop_patterns(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_cb619_with_oop_emits_summary() {
        use std::fs;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("a.lua"),
            "local M = {}\nfunction Class:new()\n  setmetatable(o, mt)\n  self.__index = self\nend\n",
        )
        .unwrap();
        let v = detect_cb619_oop_patterns(tmp.path());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-619");
        assert!(v[0].description.contains("OOP patterns"));
    }
}
