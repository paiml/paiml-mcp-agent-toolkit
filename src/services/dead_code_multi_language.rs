//! Multi-Language Dead Code Analysis Module (BUG-004 Fix)
//!
//! This module provides dead code detection across multiple programming languages
//! without requiring Cargo.toml or assuming Rust projects.
//!
//! Fixes:
//! - BUG-004: Dead code analyzer broken for non-Rust projects

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, info};
use walkdir::WalkDir;

// Pre-compiled regex patterns
include!("dead_code_multi_language_regex.rs");

// Types, traits, and dispatch
include!("dead_code_multi_language_types.rs");

// Language strategy implementations
include!("dead_code_multi_language_strategies.rs");

// C, C++, Python analysis helpers
include!("dead_code_multi_language_c_python.rs");

// Lua analysis helpers
include!("dead_code_multi_language_lua.rs");

// Rust-specific analysis helpers
include!("dead_code_multi_language_rust.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_c_dead_code_detection() {
        let temp = create_test_c_project();
        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        eprintln!("C dead code result: {:?}", result);
        eprintln!("Dead functions: {:?}", result.dead_functions);

        assert_eq!(result.language, "c");
        assert_eq!(
            result.total_functions, 2,
            "Should find 2 functions: used_function and unused_function"
        );
        assert_eq!(
            result.dead_functions.len(),
            1,
            "Should find 1 dead function"
        );
        assert_eq!(result.dead_functions[0].name, "unused_function");
    }

    #[test]
    fn test_python_dead_code_detection() {
        let temp = create_test_python_project();
        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        assert_eq!(result.language, "python");
        assert!(!result.dead_functions.is_empty());
    }

    fn create_test_c_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("main.c"),
            "int main() { used_function(); return 0; }\nvoid used_function() {}\nvoid unused_function() {}\n",
        ).unwrap();
        temp
    }

    fn create_test_python_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("main.py"),
            "def main():\n    used_function()\n\ndef used_function():\n    pass\n\ndef unused_function():\n    pass\n",
        ).unwrap();
        std::fs::write(
            temp.path().join("pyproject.toml"),
            "[project]\nname=\"test\"\n",
        )
        .unwrap();
        temp
    }

    #[test]
    fn test_lua_dead_code_detection_basic() {
        let temp = TempDir::new().unwrap();
        // Create a Lua project with used and unused functions
        std::fs::write(
            temp.path().join("main.lua"),
            concat!(
                "local function used_helper()\n",
                "    return 42\n",
                "end\n",
                "\n",
                "local function dead_helper()\n",
                "    return 99\n",
                "end\n",
                "\n",
                "function run()\n",
                "    local x = used_helper()\n",
                "    return x\n",
                "end\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        assert_eq!(defined.len(), 3, "Should find 3 functions");
        assert!(
            called.contains("used_helper"),
            "used_helper should be in calls"
        );
        assert!(
            !called.contains("dead_helper"),
            "dead_helper should NOT be in calls"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(
            dead_names.contains(&"dead_helper"),
            "dead_helper should be dead"
        );
        assert!(
            !dead_names.contains(&"used_helper"),
            "used_helper should not be dead"
        );
    }

    #[test]
    fn test_lua_module_export_awareness() {
        let temp = TempDir::new().unwrap();
        // Module pattern: functions on M are exported via `return M`
        std::fs::write(
            temp.path().join("mymodule.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "function M.public_api()\n",
                "    return M.internal_calc()\n",
                "end\n",
                "\n",
                "function M.internal_calc()\n",
                "    return 42\n",
                "end\n",
                "\n",
                "local function truly_dead()\n",
                "    return 0\n",
                "end\n",
                "\n",
                "return M\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Module functions should be treated as exported (called)
        assert!(
            called.contains("public_api"),
            "M.public_api should be marked as exported"
        );
        assert!(
            called.contains("internal_calc"),
            "M.internal_calc should be marked as exported"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(
            dead_names.contains(&"truly_dead"),
            "truly_dead should be dead"
        );
        assert!(
            !dead_names.contains(&"public_api"),
            "exported funcs should not be dead"
        );
        assert!(
            !dead_names.contains(&"internal_calc"),
            "exported funcs should not be dead"
        );
    }

    #[test]
    fn test_lua_table_field_function_export() {
        let temp = TempDir::new().unwrap();
        // Alternative module pattern: M.name = function(...)
        std::fs::write(
            temp.path().join("alt_module.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "M.handler = function(req)\n",
                "    return req\n",
                "end\n",
                "\n",
                "M.middleware = function(ctx)\n",
                "    return ctx\n",
                "end\n",
                "\n",
                "local function orphan()\n",
                "    return nil\n",
                "end\n",
                "\n",
                "return M\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        assert!(called.contains("handler"), "M.handler should be exported");
        assert!(
            called.contains("middleware"),
            "M.middleware should be exported"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(dead_names.contains(&"orphan"), "orphan should be dead");
        assert_eq!(dead.len(), 1, "Only orphan should be dead");
    }

    #[test]
    fn test_lua_no_module_return_no_exports() {
        let temp = TempDir::new().unwrap();
        // File without module return - no export awareness
        std::fs::write(
            temp.path().join("script.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "function M.something()\n",
                "    return 1\n",
                "end\n",
                "\n",
                "-- no return M at end\n",
                "print(\"hello\")\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Without `return M`, M.something is NOT auto-exported
        assert!(
            !called.contains("something"),
            "Without module return, not auto-exported"
        );
        let dead = find_uncalled_functions(&defined, &called);
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].name, "something");
    }

    #[test]
    fn test_lua_detect_module_return() {
        assert_eq!(
            detect_lua_module_return("return M\n"),
            Some("M".to_string())
        );
        assert_eq!(
            detect_lua_module_return("return MyModule\n"),
            Some("MyModule".to_string())
        );
        assert_eq!(
            detect_lua_module_return("x = 1\nreturn M\n"),
            Some("M".to_string())
        );
        assert_eq!(
            detect_lua_module_return("return M\n-- trailing comment\n"),
            Some("M".to_string())
        );
        assert_eq!(detect_lua_module_return("print('done')\n"), None);
        assert_eq!(detect_lua_module_return("return 1, 2, 3\n"), None);
        assert_eq!(detect_lua_module_return(""), None);
    }

    #[test]
    fn test_lua_test_files_excluded_from_definitions() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("tests")).unwrap();
        std::fs::write(
            temp.path().join("tests/test_main.lua"),
            concat!(
                "local function test_helper()\n",
                "    return true\n",
                "end\n",
                "\n",
                "function test_run()\n",
                "    used_in_prod()\n",
                "end\n",
            ),
        )
        .unwrap();
        std::fs::write(
            temp.path().join("main.lua"),
            concat!("local function used_in_prod()\n", "    return 1\n", "end\n",),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Test file functions should NOT be in defined list
        let def_names: Vec<&str> = defined.iter().map(|d| d.name.as_str()).collect();
        assert!(
            !def_names.contains(&"test_helper"),
            "Test functions excluded"
        );
        assert!(!def_names.contains(&"test_run"), "Test functions excluded");
        assert!(
            def_names.contains(&"used_in_prod"),
            "Prod functions included"
        );

        // But calls FROM test files should still be tracked
        assert!(
            called.contains("used_in_prod"),
            "Calls from tests should be tracked"
        );
    }
}
