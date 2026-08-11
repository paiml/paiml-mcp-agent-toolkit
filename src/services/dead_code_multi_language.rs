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

    /// #720: `total_files` must be a FILE count. The caller used to report
    /// `total_functions.max(1)` as its file count, so this 1-file / 3-function
    /// Python project printed "Files Analyzed | 3".
    #[test]
    fn test_total_files_is_a_file_count_not_a_function_count() {
        let temp = create_test_python_project();
        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        assert_eq!(
            result.total_files, 1,
            "one .py file was walked, but total_files reported {}",
            result.total_files
        );
        // `main` is deliberately skipped by the extractor, leaving
        // used_function + unused_function.
        assert_eq!(
            result.total_functions, 2,
            "sanity: the fixture yields 2 counted functions"
        );
        assert_ne!(
            result.total_files, result.total_functions,
            "a function count must never be reported as a file count"
        );
    }

    /// #720: two C files, two functions -- the two counts happen to be equal in
    /// the other fixture, so this one separates them in the opposite direction.
    #[test]
    fn test_total_files_counts_every_walked_file() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("main.c"),
            "int main() { used_function(); return 0; }\n",
        )
        .unwrap();
        std::fs::write(temp.path().join("a.c"), "void used_function() {}\n").unwrap();
        std::fs::write(temp.path().join("b.c"), "void dead_one() {}\n").unwrap();

        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        assert_eq!(
            result.total_files, 3,
            "three .c files were walked, got {}",
            result.total_files
        );
    }

    /// A nonexistent path used to be reported as "not supported for language:
    /// unknown" — a language verdict for a path that was never read.
    #[test]
    fn test_missing_path_is_a_path_error_not_a_language_verdict() {
        let err = analyze_dead_code_multi_language(Path::new(
            "/does/not/exist/pmat-dead-code-missing-path",
        ))
        .expect_err("a missing path must be an error");
        let msg = err.to_string();
        assert!(msg.contains("Path not found"), "{msg}");
        assert!(
            !msg.contains("unknown"),
            "must not report a detected language for a path that does not exist: {msg}"
        );
    }

    /// One unsupported dominant language used to abort the whole run, even when
    /// every supported file under the path was analysable.
    #[test]
    fn test_unsupported_dominant_language_still_analyses_supported_files() {
        let temp = TempDir::new().unwrap();
        // Enough TypeScript to win language detection...
        for i in 0..5 {
            std::fs::write(
                temp.path().join(format!("app{i}.ts")),
                "export function f(): number { return 1; }\n",
            )
            .unwrap();
        }
        // ...plus one analysable Python file.
        std::fs::write(
            temp.path().join("main.py"),
            "def main():\n    used()\n\ndef used():\n    pass\n\ndef dead_one():\n    pass\n",
        )
        .unwrap();

        let result = analyze_dead_code_multi_language(temp.path())
            .expect("supported files are present, so the run must not abort");
        assert_eq!(result.language, "python");
        assert!(
            result.dead_functions.iter().any(|d| d.name == "dead_one"),
            "the Python file must actually have been analysed: {result:?}"
        );
    }

    /// ...but a tree with nothing analysable in it must still say so.
    #[test]
    fn test_no_supported_files_still_errors() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("app.ts"),
            "export function f(): number { return 1; }\n",
        )
        .unwrap();
        let err =
            analyze_dead_code_multi_language(temp.path()).expect_err("nothing analysable here");
        assert!(err.to_string().contains("no rust, c, cpp, python or lua"));
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
