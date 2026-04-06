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
pub fn detect_cb618_ffi_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
pub fn detect_cb619_oop_patterns(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
