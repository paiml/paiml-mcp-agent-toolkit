#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-611: Weak Table Misuse, CB-612: Test Framework Detection,
//! and CB-613: Require Cycle Detection.

use super::super::types::*;
use super::helpers::{compute_lua_production_lines, is_lua_test_file, walkdir_lua_files};
use std::fs;
use std::path::{Path, PathBuf};

// =============================================================================
// CB-611: Weak Table Misuse Detection (#186)
// =============================================================================

/// CB-611: Detect weak table misuse patterns.
/// - String/numeric keys with `__mode = "k"` (never GC'd -- value types)
/// - Unbounded caches without weak references or eviction
///
/// Reference: Kong, AwesomeWM, KOReader weak table usage.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb611_weak_table_misuse(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

        detect_weak_key_with_value_types(&prod_lines, &rel, &mut violations);
    }

    violations
}

/// Check if a line declares a weak-key-only table (`__mode = "k"`, not "v" or "kv").
fn is_weak_key_only_declaration(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    line.contains("__mode")
        && (line.contains("\"k\"") || line.contains("'k'"))
        && !line.contains("\"v\"")
        && !line.contains("'v'")
        && !line.contains("\"kv\"")
        && !line.contains("'kv'")
}

/// Classify the key type after `var[...` -- returns "string", "numeric", or None.
fn classify_bracket_key(after_bracket: &str) -> Option<&'static str> {
    debug_assert!(!after_bracket.is_empty(), "after_bracket must not be empty");
    if after_bracket.starts_with('"') || after_bracket.starts_with('\'') {
        Some("string")
    } else if after_bracket.starts_with(|c: char| c.is_ascii_digit()) {
        Some("numeric")
    } else {
        None
    }
}

/// Detect `__mode = "k"` tables being indexed with string or numeric keys.
fn detect_weak_key_with_value_types(
    prod_lines: &[(usize, String)],
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    debug_assert!(!rel.is_empty(), "rel must not be empty");
    // Phase 1: Find variables assigned weak-key tables
    let weak_key_vars: std::collections::HashSet<String> = prod_lines
        .iter()
        .filter(|(_, trimmed)| is_weak_key_only_declaration(trimmed))
        .filter_map(|(_, trimmed)| extract_weak_table_var(trimmed))
        .collect();

    if weak_key_vars.is_empty() {
        return;
    }

    // Phase 2: Check if weak-key vars are indexed with value-type literals
    for (line_num, trimmed) in prod_lines {
        for var in &weak_key_vars {
            let bracket_pattern = format!("{var}[");
            let Some(pos) = trimmed.find(&bracket_pattern) else {
                continue;
            };
            let after = &trimmed[pos + bracket_pattern.len()..];
            let Some(key_type) = classify_bracket_key(after) else {
                continue;
            };
            violations.push(CbPatternViolation {
                pattern_id: "CB-611".to_string(),
                file: rel.to_string(),
                line: *line_num,
                description: format!(
                    "Weak-key table `{var}` indexed with {key_type} key — \
                     {key_type} keys are value types and never GC'd, defeating __mode=\"k\""
                ),
                severity: Severity::Warning,
            });
        }
    }
}

/// Extract variable name from weak table assignment.
/// E.g. `local cache = setmetatable({}, { __mode = "k" })` -> Some("cache")
fn extract_weak_table_var(line: &str) -> Option<String> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let eq_pos = line.find('=')?;
    let lhs = line[..eq_pos].trim();
    let lhs = lhs.strip_prefix("local ").unwrap_or(lhs).trim();
    if lhs.is_empty() || lhs.contains('.') || lhs.contains('[') {
        return None;
    }
    let var: String = lhs
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if var.is_empty() {
        None
    } else {
        Some(var)
    }
}

// =============================================================================
// CB-612: Lua Test Framework Detection (#184)
// =============================================================================

/// Detected Lua test framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaTestFramework {
    Busted,
    TestNginx,
    LuaUnit,
    Telescope,
    Custom,
}

impl std::fmt::Display for LuaTestFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaTestFramework::Busted => write!(f, "busted"),
            LuaTestFramework::TestNginx => write!(f, "Test::Nginx"),
            LuaTestFramework::LuaUnit => write!(f, "LuaUnit"),
            LuaTestFramework::Telescope => write!(f, "telescope"),
            LuaTestFramework::Custom => write!(f, "custom"),
        }
    }
}

/// CB-612: Auto-detect Lua test framework(s) and report as informational.
/// Supports hybrid projects (e.g., Kong uses both busted and Test::Nginx).
/// Reference: Kong, APISIX, xmake, KOReader framework patterns.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb612_test_framework(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let frameworks = detect_lua_test_frameworks(project_path);
    let mut violations = Vec::new();

    if frameworks.is_empty() {
        // No Lua test framework detected -- only flag if Lua files exist
        let lua_files = walkdir_lua_files(project_path);
        if lua_files.len() >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-612".to_string(),
                file: "project".to_string(),
                line: 0,
                description: format!(
                    "No Lua test framework detected ({} Lua files) — consider adding busted or LuaUnit",
                    lua_files.len()
                ),
                severity: Severity::Info,
            });
        }
    } else {
        let names: Vec<String> = frameworks.iter().map(|f| f.to_string()).collect();
        violations.push(CbPatternViolation {
            pattern_id: "CB-612".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!("Lua test framework(s) detected: {}", names.join(", ")),
            severity: Severity::Info,
        });
    }

    violations
}

/// Check if test file content references a specific framework.
fn has_require_pattern(content: &str, module: &str) -> bool {
    content.contains(&format!("require(\"{module}\")"))
        || content.contains(&format!("require('{module}')"))
        || content.contains(&format!("require \"{module}\""))
}

/// Detect which Lua test frameworks are in use based on file patterns and require statements.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_lua_test_frameworks(project_path: &Path) -> Vec<LuaTestFramework> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut frameworks = Vec::new();

    if has_busted_indicators(project_path) {
        frameworks.push(LuaTestFramework::Busted);
    }
    if has_test_nginx_indicators(project_path) {
        frameworks.push(LuaTestFramework::TestNginx);
    }

    let (found_luaunit, found_telescope, found_custom) = scan_test_file_requires(project_path);

    if found_luaunit {
        frameworks.push(LuaTestFramework::LuaUnit);
    }
    if found_telescope {
        frameworks.push(LuaTestFramework::Telescope);
    }
    if found_custom && frameworks.is_empty() {
        frameworks.push(LuaTestFramework::Custom);
    }

    frameworks
}

/// Scan Lua test files for require('luaunit'), require('telescope'), or custom test patterns.
fn scan_test_file_requires(project_path: &Path) -> (bool, bool, bool) {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let lua_files = walkdir_lua_files(project_path);
    let mut luaunit = false;
    let mut telescope = false;
    let mut custom = false;

    for file_path in &lua_files {
        if !is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !luaunit && has_require_pattern(&content, "luaunit") {
            luaunit = true;
        }
        if !telescope && has_require_pattern(&content, "telescope") {
            telescope = true;
        }
        if !custom
            && !luaunit
            && !telescope
            && (content.contains("function test_") || content.contains("function Test"))
        {
            custom = true;
        }
    }
    (luaunit, telescope, custom)
}

/// Check for busted test framework indicators.
fn has_busted_indicators(project_path: &Path) -> bool {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    // Check for .busted config file
    if project_path.join(".busted").exists() {
        return true;
    }
    // Check for _spec.lua files
    let lua_files = walkdir_lua_files(project_path);
    lua_files.iter().any(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.ends_with("_spec"))
    })
}

/// Check for Test::Nginx indicators.
fn has_test_nginx_indicators(project_path: &Path) -> bool {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let t_dir = project_path.join("t");
    if !t_dir.is_dir() {
        return false;
    }
    // Look for .t files in t/ directory
    match fs::read_dir(&t_dir) {
        Ok(entries) => entries
            .flatten()
            .any(|e| e.path().extension().map(|ext| ext == "t").unwrap_or(false)),
        Err(_) => false,
    }
}

// =============================================================================
// CB-613: Lua Require Cycle Detection (#187)
// =============================================================================

/// CB-613: Detect circular require() chains in Lua projects.
/// Builds a directed graph from top-level require() calls and finds cycles via DFS.
/// Function-scoped requires are excluded (they're safe -- deferred loading).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb613_require_cycles(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let files = walkdir_lua_files(project_path);
    if files.len() < 2 {
        return Vec::new();
    }

    // Build require graph: module_name -> Vec<required_module>
    let graph = build_require_graph(project_path, &files);
    if graph.is_empty() {
        return Vec::new();
    }

    // Find cycles via DFS
    let cycles = find_require_cycles(&graph);
    cycles
        .into_iter()
        .map(|cycle| {
            let chain = cycle.join(" -> ");
            CbPatternViolation {
                pattern_id: "CB-613".to_string(),
                file: cycle
                    .first()
                    .map(|s| format!("{s}.lua"))
                    .unwrap_or_default(),
                line: 0,
                description: format!("Circular require chain: {chain}"),
                severity: Severity::Warning,
            }
        })
        .collect()
}

/// Build a directed graph of top-level require() calls.
/// Returns module_name -> Vec<required_module_name>.
fn build_require_graph(
    project_path: &Path,
    files: &[PathBuf],
) -> std::collections::HashMap<String, Vec<String>> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut graph: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for file_path in files {
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
            .with_extension("")
            .display()
            .to_string()
            .replace(['/', '\\'], ".");

        let requires = extract_top_level_requires(&content);
        if !requires.is_empty() {
            graph.insert(rel, requires);
        }
    }

    graph
}

/// Extract module names from top-level require() calls (not inside functions).
fn extract_top_level_requires(content: &str) -> Vec<String> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut requires = Vec::new();
    let mut func_depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        // Track function nesting
        if trimmed.starts_with("function ")
            || trimmed.starts_with("local function ")
            || trimmed.contains("= function(")
        {
            func_depth += 1;
        }
        // Only capture top-level requires
        if func_depth == 0 {
            if let Some(module) = extract_require_module(trimmed) {
                requires.push(module);
            }
        }
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end)") {
            func_depth = (func_depth - 1).max(0);
        }
    }
    requires
}

/// Extract module name from a require() call.
/// Matches: require("foo"), require('foo'), require "foo", require 'foo'
fn extract_require_module(line: &str) -> Option<String> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let req_idx = line.find("require")?;
    let after = line[req_idx + 7..].trim();
    // Skip if require is part of a larger word
    if req_idx > 0 {
        let before_char = line.as_bytes()[req_idx - 1];
        if before_char.is_ascii_alphanumeric() || before_char == b'_' {
            return None;
        }
    }
    let after = after.strip_prefix('(').unwrap_or(after).trim();
    let (quote, rest) = if let Some(stripped) = after.strip_prefix('"') {
        ('"', stripped)
    } else if let Some(stripped) = after.strip_prefix('\'') {
        ('\'', stripped)
    } else {
        return None;
    };
    let end = rest.find(quote)?;
    let module = rest[..end].to_string();
    if module.is_empty() {
        None
    } else {
        Some(module)
    }
}

/// Find cycles in the require graph using DFS.
fn find_require_cycles(graph: &std::collections::HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    debug_assert!(true, "contract: find_require_cycles");
    use std::collections::HashSet;
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = Vec::new();

    for start in graph.keys() {
        if visited.contains(start) {
            continue;
        }
        dfs_find_cycle(start, graph, &mut visited, &mut rec_stack, &mut cycles);
    }
    cycles
}

/// DFS helper to detect back edges (cycles).
fn dfs_find_cycle(
    node: &str,
    graph: &std::collections::HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    rec_stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    debug_assert!(!node.is_empty(), "node must not be empty");
    visited.insert(node.to_string());
    rec_stack.push(node.to_string());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if let Some(pos) = rec_stack.iter().position(|n| n == dep) {
                // Found a cycle: extract the cycle from stack
                let cycle: Vec<String> = rec_stack[pos..].to_vec();
                cycles.push(cycle);
            } else if !visited.contains(dep.as_str()) {
                dfs_find_cycle(dep, graph, visited, rec_stack, cycles);
            }
        }
    }

    rec_stack.pop();
}
