#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;

// =============================================================================
// Tests for CB-130 Agent Context Adoption
// =============================================================================

mod cb130_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb130_no_index() {
        let temp = TempDir::new().unwrap();
        let report = detect_cb130_agent_context_adoption(temp.path());

        assert!(!report.index_exists);
        assert!(report.index_age_hours.is_none());
        assert!(!report.index_stale);
        assert_eq!(report.function_count, 0);
        assert!(!report.claude_md_configured);
    }

    #[test]
    fn test_cb130_with_claude_md_pmat_query() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nUse `pmat query` for code search.\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(report.claude_md_configured);
    }

    #[test]
    fn test_cb130_with_claude_md_mcp_tool() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nUse pmat_query_code tool for search.\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(report.claude_md_configured);
    }

    #[test]
    fn test_cb130_claude_md_no_mention() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nJust some generic instructions.\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(!report.claude_md_configured);
    }

    #[test]
    fn test_cb130_with_index_file() {
        let temp = TempDir::new().unwrap();

        // Create .pmat directory and a dummy index file
        let pmat_dir = temp.path().join(".pmat");
        fs::create_dir_all(&pmat_dir).unwrap();
        // Write some bytes - it won't deserialize but index_exists is checked first
        fs::write(pmat_dir.join("context.idx"), b"dummy").unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(report.index_exists);
        // function_count will be 0 because the index can't be loaded
        assert_eq!(report.function_count, 0);
    }

    #[test]
    fn test_cb130_required_patterns_missing() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nUse pmat query for search.\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        // "pmat query" is present but "NEVER use grep" is missing
        assert!(report.claude_md_configured);
        assert!(report.missing_required_patterns.contains(&"NEVER use grep".to_string()));
    }

    #[test]
    fn test_cb130_all_required_patterns_present() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nNEVER use grep for code search.\nUse `pmat query --faults` instead.\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(report.claude_md_configured);
        assert!(report.missing_required_patterns.is_empty());
    }

    #[test]
    fn test_cb130_forbidden_pattern_detected() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\nSearch with: grep -r \"error\" src/\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        assert!(!report.forbidden_patterns_found.is_empty());
        assert_eq!(report.forbidden_patterns_found[0].pattern, "grep -r");
        assert_eq!(report.forbidden_patterns_found[0].line, 3);
    }

    #[test]
    fn test_cb130_forbidden_pattern_in_negative_example_allowed() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("CLAUDE.md"),
            "# Instructions\n\n# BAD - Don't do this: grep -r \"error\" src/\n",
        )
        .unwrap();

        let report = detect_cb130_agent_context_adoption(temp.path());
        // Should not flag "grep -r" when it's in a negative example context
        assert!(report.forbidden_patterns_found.is_empty());
    }
}

// =============================================================================
// Tests for OIP Tarantula Pattern Detection
// =============================================================================

#[cfg(test)]
mod oip_tarantula_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // CB-120 Tests: NaN-unsafe comparison detection

    #[test]
    fn test_cb120_detects_partial_cmp_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("ml.rs"),
            r#"
fn sort_floats(vec: &mut Vec<f64>) {
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-120");
        assert!(violations[0].description.contains("partial_cmp"));
    }

    #[test]
    fn test_cb120_skips_unwrap_or() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("safe.rs"),
            r#"
fn sort_floats(vec: &mut Vec<f64>) {
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb120_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_sort() {
        let mut v = vec![1.0, 2.0];
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert!(violations.is_empty());
    }

    // CB-121 Tests: Lock poisoning detection

    #[test]
    #[ignore = "OIP detection needs walkdir debugging"]
    fn test_cb121_detects_mutex_lock_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("sync.rs"),
            r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
    *m.lock().unwrap()
}
"#,
        )
        .unwrap();

        let violations = detect_cb121_lock_poisoning(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-121");
    }

    #[test]
    fn test_cb121_skips_into_inner() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("safe.rs"),
            r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
    *m.lock().unwrap_or_else(|e| e.into_inner())
}
"#,
        )
        .unwrap();

        let violations = detect_cb121_lock_poisoning(temp.path());
        assert!(violations.is_empty());
    }

    // CB-122 Tests: Serde deserialization safety

    #[test]
    fn test_cb122_detects_serde_json_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("parser.rs"),
            r#"
fn parse_config(s: &str) -> Config {
    serde_json::from_str(s).unwrap()
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-122");
    }

    #[test]
    fn test_cb122_detects_toml_expect() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn load(s: &str) -> Settings {
    toml::from_str(s).expect("invalid toml")
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_cb122_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let v: Value = serde_json::from_str("{}").unwrap();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert!(violations.is_empty());
    }

    // CB-123 Tests: Undocumented #[ignore] (bare, without reason)

    #[test]
    #[ignore = "OIP detection needs walkdir debugging"]
    fn test_cb123_detects_bare_ignore() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore = "slow test"]
#[test]
fn slow_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-123");
    }

    #[test]
    fn test_cb123_skips_ignore_with_reason() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore = "requires GPU"]
#[test]
fn gpu_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb123_skips_ignore_with_comment() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore] // flaky on CI
#[test]
fn flaky_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert!(violations.is_empty());
    }

    // CB-124 Tests: Coverage threshold enforcement

    #[test]
    fn test_cb124_detects_low_threshold() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 58.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-124");
        assert_eq!(violations[0].severity, Severity::Error);
    }

    #[test]
    fn test_cb124_warns_below_95() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 85.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_cb124_passes_high_threshold() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 95.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// Tests for CB-081 Dependency Count Detection
// =============================================================================

#[cfg(test)]
mod cb081_dependency_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb081_detects_excessive_direct_deps() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with many dependencies (>50)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..60 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();
        fs::write(temp.path().join("Cargo.lock"), "[[package]]\nname = \"test\"").unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 60);
        assert_eq!(report.score, 0);  // >50 direct = score 0
        assert!(!report.violations.is_empty());
        assert_eq!(report.violations[0].pattern_id, "CB-081-A");
    }

    #[test]
    fn test_cb081_moderate_deps() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with moderate dependencies (30-40)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..35 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

        // Create Cargo.lock with 180 packages (between 150-200)
        let mut lock = String::new();
        for _ in 0..180 {
            lock.push_str("[[package]]\nname = \"pkg\"\n");
        }
        fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 35);
        assert_eq!(report.transitive_count, 180);
        assert_eq!(report.score, 3);  // 30-40 direct, 150-200 transitive = 3
    }

    #[test]
    fn test_cb081_low_deps_excellent() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with few dependencies (<=20)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..15 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

        // Create Cargo.lock with few packages (<=100)
        let mut lock = String::new();
        for _ in 0..80 {
            lock.push_str("[[package]]\nname = \"pkg\"\n");
        }
        fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 15);
        assert_eq!(report.transitive_count, 80);
        assert_eq!(report.score, 5);  // <=20 direct, <=100 transitive = 5
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_cb081_excludes_dev_dependencies() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with few regular deps but many dev-deps
        let deps = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
proptest = "1.0"
quickcheck = "1.0"
tokio-test = "0.4"
"#;
        fs::write(temp.path().join("Cargo.toml"), deps).unwrap();
        fs::write(temp.path().join("Cargo.lock"), "[[package]]\nname = \"test\"").unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        // Only counts [dependencies], not [dev-dependencies]
        assert_eq!(report.direct_count, 2);
    }

    #[test]
    fn test_cb081_no_cargo_toml() {
        let temp = TempDir::new().unwrap();
        // No Cargo.toml

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 0);
        assert_eq!(report.transitive_count, 0);
    }

    // =========================================================================
    // CB-400/401/402 bashrs integration tests
    // =========================================================================

    #[test]
    fn test_cb400_no_git_hooks_dir() {
        let temp = TempDir::new().unwrap();
        // No .git/hooks directory
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "No hooks dir should return empty");
    }

    #[test]
    fn test_cb400_empty_git_hooks_dir() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git/hooks")).unwrap();
        // Empty hooks dir - no hook files
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "Empty hooks dir should return empty");
    }

    #[test]
    fn test_cb400_sample_hooks_ignored() {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        // Sample hooks should be ignored
        fs::write(hooks_dir.join("pre-commit.sample"), "#!/bin/bash\necho test").unwrap();
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "Sample hooks should be ignored");
    }

    #[test]
    fn test_cb401_no_makefile() {
        let temp = TempDir::new().unwrap();
        // No Makefile
        let violations = detect_cb401_makefile_quality(temp.path());
        assert!(violations.is_empty(), "No Makefile should return empty");
    }

    #[test]
    fn test_cb402_no_shell_scripts() {
        let temp = TempDir::new().unwrap();
        // No shell scripts
        let violations = detect_cb402_shell_script_quality(temp.path());
        assert!(violations.is_empty(), "No shell scripts should return empty");
    }

    #[test]
    fn test_cb402_target_dir_excluded() {
        let temp = TempDir::new().unwrap();
        // Shell script in target/ should be ignored
        let target_dir = temp.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("test.sh"), "#!/bin/bash\necho test").unwrap();
        let violations = detect_cb402_shell_script_quality(temp.path());
        assert!(violations.is_empty(), "Scripts in target/ should be ignored");
    }

    #[test]
    fn test_parse_bashrs_json_array() {
        let json = r#"[{"code":"SC2086","message":"Double quote","line":5,"severity":"warning"}]"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "SC2086");
        assert_eq!(result[0].line, 5);
    }

    #[test]
    fn test_parse_bashrs_json_object() {
        let json = r#"{"diagnostics":[{"code":"SC2046","message":"Quote this","line":3,"severity":"error"}]}"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "SC2046");
        assert_eq!(result[0].severity, "error");
    }

    #[test]
    fn test_parse_bashrs_json_invalid() {
        let json = "not valid json";
        let result = parse_bashrs_json_output(json).unwrap();
        assert!(result.is_empty(), "Invalid JSON should return empty");
    }

    #[test]
    fn test_parse_bashrs_json_empty_array() {
        let json = "[]";
        let result = parse_bashrs_json_output(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_bashrs_json_multiple_issues() {
        let json = r#"[
            {"code":"SC2086","message":"Double quote","line":5,"severity":"warning"},
            {"code":"SC2046","message":"Quote this","line":10,"severity":"error"},
            {"code":"SC2116","message":"Useless echo","line":15,"severity":"info"}
        ]"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].code, "SC2086");
        assert_eq!(result[1].code, "SC2046");
        assert_eq!(result[2].code, "SC2116");
    }
}

// =============================================================================
// Tests for CB-600 Lua Best Practices Detection (PMAT-487)
// =============================================================================

#[cfg(test)]
mod cb600_lua_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // Helper function tests
    // =========================================================================

    #[test]
    fn test_is_lua_test_file() {
        assert!(is_lua_test_file(std::path::Path::new("foo_test.lua")));
        assert!(is_lua_test_file(std::path::Path::new("bar_spec.lua")));
        assert!(is_lua_test_file(std::path::Path::new("test_baz.lua")));
        assert!(is_lua_test_file(std::path::Path::new("tests/util.lua")));
        assert!(is_lua_test_file(std::path::Path::new("spec/helper.lua")));
        assert!(!is_lua_test_file(std::path::Path::new("app.lua")));
        assert!(!is_lua_test_file(std::path::Path::new("module.lua")));
    }

    #[test]
    fn test_compute_lua_production_lines_filters_comments() {
        let content = "-- comment\nlocal x = 1\n--[[ block\ncomment ]]\nlocal y = 2\n";
        let lines = compute_lua_production_lines(content);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].1, "local x = 1");
        assert_eq!(lines[1].1, "local y = 2");
    }

    #[test]
    fn test_walkdir_lua_files_skips_git() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("hook.lua"), "local x = 1").unwrap();
        fs::write(temp.path().join("app.lua"), "local y = 2").unwrap();
        let files = walkdir_lua_files(temp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap() == "app.lua");
    }

    // =========================================================================
    // CB-600: Implicit Globals
    // =========================================================================

    #[test]
    fn test_cb600_detects_implicit_global() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "counter = 0\nlocal x = 1\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-600");
        assert!(violations[0].description.contains("counter"));
    }

    #[test]
    fn test_cb600_skips_local_vars() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local counter = 0\nlocal name = 'test'\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb600_skips_std_globals() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "print = custom_print\n").unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb600_skips_table_field_assignment() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "M.name = 'test'\nself.value = 42\ntbl[key] = true\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb600_skips_table_constructor_fields() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local t = {\n  id = id,\n  name = name,\n  score = 100,\n}\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "table constructor fields are not globals: {:?}", violations);
    }

    #[test]
    fn test_cb600_skips_inline_table_constructor() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local t = { id = id, name = name }\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "inline table constructor fields are not globals: {:?}", violations);
    }

    #[test]
    fn test_cb600_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app_test.lua"), "counter = 0\n").unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb600_no_lua_files_empty() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.rs"), "fn main() {}").unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb600_skips_function_params() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "function M.levenshtein(a, b)\n  a = a or \"\"\n  b = b or \"\"\nend\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "function params should not be flagged: {:?}", violations);
    }

    #[test]
    fn test_cb600_skips_for_loop_vars() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "for i = 1, 10 do\n  i = i + 1\nend\nfor k, v in pairs(t) do\n  k = tostring(k)\nend\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "for-loop vars should not be flagged: {:?}", violations);
    }

    #[test]
    fn test_cb600_skips_local_decl_reassignment() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local result = 0\nresult = result + 1\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "local var reassignment should not be flagged: {:?}", violations);
    }

    #[test]
    fn test_cb600_skips_multi_local_decl() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local a, b = 1, 2\na = a + 1\nb = b + 1\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert!(violations.is_empty(), "multi-local reassignment should not be flagged: {:?}", violations);
    }

    #[test]
    fn test_cb600_still_detects_true_globals() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "function foo(x)\n  x = x + 1\nend\nglobal_thing = 42\n",
        )
        .unwrap();
        let violations = detect_cb600_implicit_globals(temp.path());
        assert_eq!(violations.len(), 1, "true global should still be caught");
        assert!(violations[0].description.contains("global_thing"));
    }

    // =========================================================================
    // CB-601: Nil-Unsafe Access
    // =========================================================================

    #[test]
    fn test_cb601_detects_chained_call() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local name = get_user().name\n",
        )
        .unwrap();
        let violations = detect_cb601_nil_unsafe_access(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-601");
        assert!(violations[0].description.contains("chained"));
    }

    #[test]
    fn test_cb601_detects_deep_chain() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local x = config.server.host.port\n",
        )
        .unwrap();
        let violations = detect_cb601_nil_unsafe_access(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("deep field"));
    }

    #[test]
    fn test_cb601_shallow_access_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local x = config.host\nlocal y = tbl.key\n",
        )
        .unwrap();
        let violations = detect_cb601_nil_unsafe_access(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb601_string_key_with_dots_not_false_positive() {
        // ["H.N.S.W."] and ["C.I.C.D."] are string-literal table keys,
        // not deep field access chains
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            concat!(
                "local corrections = {}\n",
                "corrections[\"H.N.S.W.\"] = \"HNSW\"\n",
                "corrections[\"C.I.C.D.\"] = \"CICD\"\n",
                "corrections['R.A.G.'] = \"RAG\"\n",
            ),
        )
        .unwrap();
        let violations = detect_cb601_nil_unsafe_access(temp.path());
        assert!(
            violations.is_empty(),
            "Dots inside string-literal table keys should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_cb601_real_deep_chain_still_detected() {
        // Ensure real deep chains are still caught after the string fix
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local x = a.b.c.d\n",
        )
        .unwrap();
        let violations = detect_cb601_nil_unsafe_access(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("deep field"));
    }

    #[test]
    fn test_count_consecutive_field_access_skips_strings() {
        // Dots inside strings don't count
        assert!(count_consecutive_field_access("tbl[\"H.N.S.W.\"] = 1") < 4);
        assert!(count_consecutive_field_access("x['a.b.c.d.e'] = 1") < 4);
        // Real chains still count
        assert_eq!(count_consecutive_field_access("a.b.c.d"), 4);
        assert_eq!(count_consecutive_field_access("a.b.c"), 3);
        // Mixed: bracket access counts as 1 level but its contents don't add depth
        assert!(count_consecutive_field_access("tbl[\"key\"].field") < 4);
    }

    // =========================================================================
    // CB-602: pcall Error Handling
    // =========================================================================

    #[test]
    fn test_cb602_uncaptured_pcall() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "pcall(dangerous_fn)\n").unwrap();
        let violations = detect_cb602_pcall_error_handling(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("not captured"));
    }

    #[test]
    fn test_cb602_unchecked_status() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local ok, result = pcall(fn_call)\nlocal x = result\n",
        )
        .unwrap();
        let violations = detect_cb602_pcall_error_handling(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("not checked"));
    }

    #[test]
    fn test_cb602_checked_pcall_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local ok, result = pcall(fn_call)\nif not ok then\n  error(result)\nend\n",
        )
        .unwrap();
        let violations = detect_cb602_pcall_error_handling(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb602_prefixed_variable_not_false_positive() {
        // Pattern: local wrap_ok = pcall(obj.method, obj, ...) / if wrap_ok then
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            concat!(
                "local wrap_ok, wrap_err = pcall(obj.wrap, obj, config)\n",
                "if wrap_ok then\n",
                "  config = {applied = true}\n",
                "end\n",
            ),
        )
        .unwrap();
        let violations = detect_cb602_pcall_error_handling(temp.path());
        assert!(
            violations.is_empty(),
            "pcall with prefixed var checked on next line should not be flagged: {:?}",
            violations
        );
    }

    #[test]
    fn test_cb602_multiple_prefixed_vars_pass() {
        // All 4 false positive patterns from CB-602 audit
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            concat!(
                "local lint_ok, lint_result = pcall(obj.lint, obj, source)\n",
                "if lint_ok then\n",
                "  process(lint_result)\n",
                "end\n",
                "\n",
                "local qe_ok, qe_result = pcall(query_engine.execute, query_engine, q)\n",
                "if not qe_ok then\n",
                "  error(qe_result)\n",
                "end\n",
                "\n",
                "local export_ok, data = pcall(obj.export, obj, fmt)\n",
                "if export_ok then\n",
                "  save(data)\n",
                "end\n",
            ),
        )
        .unwrap();
        let violations = detect_cb602_pcall_error_handling(temp.path());
        assert!(
            violations.is_empty(),
            "All prefixed pcall vars checked on next line: {:?}",
            violations
        );
    }

    #[test]
    fn test_cb602_extract_pcall_status_var() {
        assert_eq!(
            extract_pcall_status_var("local ok, err = pcall(fn)"),
            Some("ok".to_string())
        );
        assert_eq!(
            extract_pcall_status_var("local wrap_ok, wrap_err = pcall(obj.method, obj)"),
            Some("wrap_ok".to_string())
        );
        assert_eq!(
            extract_pcall_status_var("status = pcall(fn)"),
            Some("status".to_string())
        );
        assert_eq!(
            extract_pcall_status_var("pcall(fn)"),
            None
        );
    }

    // =========================================================================
    // CB-603: Deprecated/Dangerous API
    // =========================================================================

    #[test]
    fn test_cb603_detects_loadstring() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "local fn = loadstring(code)\n").unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("loadstring"));
    }

    #[test]
    fn test_cb603_detects_os_execute() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "os.execute(cmd)\n").unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("os.execute"));
    }

    #[test]
    fn test_cb603_skips_comments() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "-- loadstring(code)\nlocal x = 1\n",
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb603_hardcoded_string_is_info_severity() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "os.execute(\"make clean\")\n",
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Info, "Hardcoded string arg should be Info");
        assert!(violations[0].description.contains("hardcoded"));
    }

    #[test]
    fn test_cb603_concatenation_is_warning() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "os.execute(\"rm -rf \" .. user_input)\n",
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning, "Concatenation should be Warning");
        assert!(violations[0].description.contains("command injection"));
    }

    #[test]
    fn test_cb603_variable_arg_is_warning() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "os.execute(cmd)\n",
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_cb603_inline_suppression() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            concat!(
                "os.execute(\"make build\") -- pmat:ignore CB-603\n",
                "os.execute(\"make test\")\n",
            ),
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert_eq!(violations.len(), 1, "Suppressed line should not be flagged");
        assert_eq!(violations[0].line, 2, "Only unsuppressed line should be flagged");
    }

    #[test]
    fn test_cb603_bare_pmat_ignore_suppresses_all() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "io.popen(\"ls\") -- pmat:ignore\n",
        )
        .unwrap();
        let violations = detect_cb603_deprecated_dangerous_api(temp.path());
        assert!(violations.is_empty(), "Bare pmat:ignore should suppress all");
    }

    // =========================================================================
    // CB-604: Unused Variables
    // =========================================================================

    #[test]
    fn test_cb604_detects_unused_var() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local unused = compute()\nlocal used = 1\nprint(used)\n",
        )
        .unwrap();
        let violations = detect_cb604_unused_variables(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("unused"));
    }

    #[test]
    fn test_cb604_underscore_prefix_skipped() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "local _ignored = compute()\n").unwrap();
        let violations = detect_cb604_unused_variables(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb604_used_var_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local name = get_name()\nprint(name)\n",
        )
        .unwrap();
        let violations = detect_cb604_unused_variables(temp.path());
        assert!(violations.is_empty());
    }

    // =========================================================================
    // CB-605: String Concat in Loop
    // =========================================================================

    #[test]
    fn test_cb605_detects_concat_in_loop() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "for i = 1, 10 do\n  result = result .. item\nend\n",
        )
        .unwrap();
        let violations = detect_cb605_string_concat_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("concatenation"));
    }

    #[test]
    fn test_cb605_concat_outside_loop_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "local msg = greeting .. name\n").unwrap();
        let violations = detect_cb605_string_concat_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    // =========================================================================
    // CB-606: Missing Module Return
    // =========================================================================

    #[test]
    fn test_cb606_detects_missing_return() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("module.lua"),
            "local M = {}\nfunction M.hello()\n  print('hi')\nend\n",
        )
        .unwrap();
        let violations = detect_cb606_missing_module_return(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("return M"));
    }

    #[test]
    fn test_cb606_return_present_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("module.lua"),
            "local M = {}\nfunction M.hello()\n  print('hi')\nend\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb606_missing_module_return(temp.path());
        assert!(violations.is_empty());
    }

    // =========================================================================
    // CB-607: Colon/Dot Confusion
    // =========================================================================

    #[test]
    fn test_cb607_detects_mixed_style() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "player:move()\nplayer.jump()\n",
        )
        .unwrap();
        let violations = detect_cb607_colon_dot_confusion(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("player"));
    }

    #[test]
    fn test_cb607_consistent_usage_passes() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "player:move()\nplayer:jump()\n",
        )
        .unwrap();
        let violations = detect_cb607_colon_dot_confusion(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb607_std_library_skipped() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "math.floor(x)\nmath.ceil(y)\nstring.format('hi')\n",
        )
        .unwrap();
        let violations = detect_cb607_colon_dot_confusion(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// Tests for CB-700 SQL Best Practices
// =============================================================================

mod cb700_sql_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb700_no_sql_files_empty() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb700_select_star(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb700_detects_select_star() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT * FROM users WHERE active = 1;\n",
        )
        .unwrap();
        let violations = detect_cb700_select_star(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-700");
    }

    #[test]
    fn test_cb700_allows_count_star() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT COUNT(*) FROM users;\n",
        )
        .unwrap();
        let violations = detect_cb700_select_star(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb700_allows_explicit_columns() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT id, name, email FROM users;\n",
        )
        .unwrap();
        let violations = detect_cb700_select_star(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb701_detects_update_without_where() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("dangerous.sql"),
            "UPDATE users SET active = 0;\n",
        )
        .unwrap();
        let violations = detect_cb701_missing_where(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-701");
        assert!(matches!(violations[0].severity, Severity::Error));
    }

    #[test]
    fn test_cb701_allows_update_with_where() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("safe.sql"),
            "UPDATE users SET active = 0 WHERE id = 5;\n",
        )
        .unwrap();
        let violations = detect_cb701_missing_where(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb701_detects_delete_without_where() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("dangerous.sql"),
            "DELETE FROM users;\n",
        )
        .unwrap();
        let violations = detect_cb701_missing_where(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-701");
    }

    #[test]
    fn test_cb702_detects_implicit_join() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT u.name FROM users u, orders o WHERE u.id = o.user_id;\n",
        )
        .unwrap();
        let violations = detect_cb702_implicit_join(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-702");
    }

    #[test]
    fn test_cb702_allows_explicit_join() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT u.name FROM users u JOIN orders o ON u.id = o.user_id;\n",
        )
        .unwrap();
        let violations = detect_cb702_implicit_join(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb704_detects_many_joins() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("query.sql"),
            "SELECT a.x FROM a JOIN b ON a.id = b.id JOIN c ON b.id = c.id JOIN d ON c.id = d.id JOIN e ON d.id = e.id;\n",
        )
        .unwrap();
        let violations = detect_cb704_missing_index_hint(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-704");
    }

    #[test]
    fn test_sql_test_file_excluded() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("tests");
        fs::create_dir_all(&test_dir).unwrap();
        fs::write(
            test_dir.join("test_queries.sql"),
            "SELECT * FROM users;\n",
        )
        .unwrap();
        let violations = detect_cb700_select_star(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_compute_sql_production_lines() {
        let content = "-- This is a comment\nSELECT id FROM users; -- inline comment\n/* block */\nINSERT INTO t VALUES(1);\n";
        let lines = compute_sql_production_lines(content);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].1.contains("SELECT"));
        assert!(lines[1].1.contains("INSERT"));
    }

    #[test]
    fn test_walkdir_sql_files_skips_git() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("hooks.sql"), "SELECT 1;\n").unwrap();
        fs::write(temp.path().join("real.sql"), "SELECT 1;\n").unwrap();
        let files = walkdir_sql_files(temp.path());
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_cb705_detects_n_plus_1_query() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.py"),
            "users = db.query('SELECT * FROM users')\nfor user in users:\n    cursor.execute('SELECT * FROM orders WHERE user_id=' + str(user.id))\n",
        )
        .unwrap();
        let violations = detect_cb705_n_plus_1_query(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-705");
    }

    #[test]
    fn test_cb705_no_false_positive_outside_loop() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.py"),
            "result = cursor.execute('SELECT * FROM users WHERE id = 1')\n",
        )
        .unwrap();
        let violations = detect_cb705_n_plus_1_query(temp.path());
        assert_eq!(violations.len(), 0);
    }
}

// =============================================================================
// Tests for CB-900 Markdown Best Practices
// =============================================================================

mod cb900_markdown_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb900_no_md_files_empty() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb900_broken_internal_link(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb900_detects_broken_link() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("README.md"),
            "# Hello\n\nSee [docs](./nonexistent.md) for more.\n",
        )
        .unwrap();
        let violations = detect_cb900_broken_internal_link(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-900");
    }

    #[test]
    fn test_cb900_allows_valid_link() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("other.md"), "# Other\n").unwrap();
        fs::write(
            temp.path().join("README.md"),
            "# Hello\n\nSee [docs](./other.md) for more.\n",
        )
        .unwrap();
        let violations = detect_cb900_broken_internal_link(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb900_skips_http_links() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("README.md"),
            "# Hello\n\n[link](https://example.com)\n",
        )
        .unwrap();
        let violations = detect_cb900_broken_internal_link(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb901_detects_heading_skip() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\n### Subsection\n\nContent here.\n",
        )
        .unwrap();
        let violations = detect_cb901_heading_hierarchy_skip(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-901");
        assert!(violations[0].description.contains("h1 to h3"));
    }

    #[test]
    fn test_cb901_allows_proper_hierarchy() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\n## Section\n\n### Subsection\n",
        )
        .unwrap();
        let violations = detect_cb901_heading_hierarchy_skip(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb902_detects_missing_alt_text() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\n![](image.png)\n",
        )
        .unwrap();
        let violations = detect_cb902_missing_alt_text(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-902");
    }

    #[test]
    fn test_cb902_allows_alt_text() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\n![A diagram](image.png)\n",
        )
        .unwrap();
        let violations = detect_cb902_missing_alt_text(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb903_detects_bare_url() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\nhttps://example.com\n",
        )
        .unwrap();
        let violations = detect_cb903_bare_url(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-903");
    }

    #[test]
    fn test_cb903_allows_markdown_link() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("doc.md"),
            "# Title\n\n[Example](https://example.com)\n",
        )
        .unwrap();
        let violations = detect_cb903_bare_url(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb904_detects_long_line() {
        let temp = TempDir::new().unwrap();
        let long_line = "x".repeat(150);
        fs::write(
            temp.path().join("doc.md"),
            format!("# Title\n\n{}\n", long_line),
        )
        .unwrap();
        let violations = detect_cb904_long_line(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-904");
    }

    #[test]
    fn test_cb904_allows_code_blocks() {
        let temp = TempDir::new().unwrap();
        let long_line = "x".repeat(150);
        fs::write(
            temp.path().join("doc.md"),
            format!("# Title\n\n```\n{}\n```\n", long_line),
        )
        .unwrap();
        let violations = detect_cb904_long_line(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb904_allows_tables() {
        let temp = TempDir::new().unwrap();
        let long_table = format!("| {} | {} |", "cell".repeat(30), "data".repeat(30));
        fs::write(
            temp.path().join("doc.md"),
            format!("# Title\n\n{}\n", long_table),
        )
        .unwrap();
        let violations = detect_cb904_long_line(temp.path());
        assert_eq!(violations.len(), 0);
    }
}

// =============================================================================
// Tests for CB-950 YAML Best Practices
// =============================================================================

mod cb950_yaml_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb950_no_yaml_files_empty() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb950_truthy_ambiguity(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb950_detects_truthy_string() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("config.yaml"),
            "name: my-app\nenabled: yes\nverbose: no\n",
        )
        .unwrap();
        let violations = detect_cb950_truthy_ambiguity(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-950");
    }

    #[test]
    fn test_cb950_allows_quoted_truthy() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("config.yaml"),
            "name: my-app\nenabled: \"yes\"\nverbose: 'no'\n",
        )
        .unwrap();
        let violations = detect_cb950_truthy_ambiguity(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb951_detects_excessive_nesting() {
        let temp = TempDir::new().unwrap();
        // Create deeply nested YAML (10 levels)
        let mut content = String::new();
        for i in 0..10 {
            let indent = "  ".repeat(i);
            content.push_str(&format!("{}level{}:\n", indent, i));
        }
        fs::write(temp.path().join("deep.yaml"), &content).unwrap();
        let violations = detect_cb951_excessive_nesting(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-951");
    }

    #[test]
    fn test_cb951_allows_moderate_nesting() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("config.yaml"),
            "level0:\n  level1:\n    level2:\n      value: ok\n",
        )
        .unwrap();
        let violations = detect_cb951_excessive_nesting(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb952_detects_missing_gha_fields() {
        let temp = TempDir::new().unwrap();
        let gha_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&gha_dir).unwrap();
        fs::write(
            gha_dir.join("ci.yml"),
            "# No name, no on, no jobs\nsteps:\n  - run: echo hi\n",
        )
        .unwrap();
        let violations = detect_cb952_missing_required_fields(temp.path());
        assert!(violations.len() >= 2); // missing name, on, jobs
    }

    #[test]
    fn test_cb952_passes_valid_workflow() {
        let temp = TempDir::new().unwrap();
        let gha_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&gha_dir).unwrap();
        fs::write(
            gha_dir.join("ci.yml"),
            "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n",
        )
        .unwrap();
        let violations = detect_cb952_missing_required_fields(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb953_detects_unpinned_action() {
        let temp = TempDir::new().unwrap();
        let gha_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&gha_dir).unwrap();
        fs::write(
            gha_dir.join("ci.yml"),
            "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@main\n",
        )
        .unwrap();
        let violations = detect_cb953_unpinned_action(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-953");
    }

    #[test]
    fn test_cb953_allows_pinned_action() {
        let temp = TempDir::new().unwrap();
        let gha_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&gha_dir).unwrap();
        fs::write(
            gha_dir.join("ci.yml"),
            "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n",
        )
        .unwrap();
        let violations = detect_cb953_unpinned_action(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb954_detects_plaintext_secret() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("config.yaml"),
            "database:\n  password: supersecret123\n  host: localhost\n",
        )
        .unwrap();
        let violations = detect_cb954_plaintext_secret(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-954");
        assert!(matches!(violations[0].severity, Severity::Error));
    }

    #[test]
    fn test_cb954_allows_env_reference() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("config.yaml"),
            "database:\n  password: ${{ secrets.DB_PASSWORD }}\n  host: localhost\n",
        )
        .unwrap();
        let violations = detect_cb954_plaintext_secret(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_walkdir_yaml_files_skips_git() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config.yml"), "key: val\n").unwrap();
        fs::write(temp.path().join("real.yaml"), "key: val\n").unwrap();
        let files = walkdir_yaml_files(temp.path());
        assert_eq!(files.len(), 1);
    }
}

// =============================================================================
// Tests for CB-1000 MLOps Model Quality
// =============================================================================

mod cb1000_model_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb1000_no_model_files_empty() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb1000_missing_model_card(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb1000_detects_missing_model_card() {
        let temp = TempDir::new().unwrap();
        let models_dir = temp.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        // Create a minimal GGUF file (just magic bytes)
        let mut gguf_header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
        gguf_header.extend_from_slice(&3u32.to_le_bytes()); // version 3
        gguf_header.extend_from_slice(&10u64.to_le_bytes()); // tensor_count
        gguf_header.extend_from_slice(&5u64.to_le_bytes()); // metadata_count
        gguf_header.resize(64, 0);
        fs::write(models_dir.join("model.gguf"), &gguf_header).unwrap();

        let violations = detect_cb1000_missing_model_card(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1000");
    }

    #[test]
    fn test_cb1000_passes_with_readme() {
        let temp = TempDir::new().unwrap();
        let models_dir = temp.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("model.gguf"), &[0x47, 0x47, 0x55, 0x46, 0, 0, 0, 0]).unwrap();
        fs::write(models_dir.join("README.md"), "# Model Card\n").unwrap();

        let violations = detect_cb1000_missing_model_card(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb1001_detects_oversized_tensor_count() {
        let temp = TempDir::new().unwrap();
        let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
        header.extend_from_slice(&3u32.to_le_bytes()); // version
        header.extend_from_slice(&200_000u64.to_le_bytes()); // oversized tensor_count
        header.extend_from_slice(&0u64.to_le_bytes()); // metadata_count
        header.resize(64, 0);
        fs::write(temp.path().join("bad.gguf"), &header).unwrap();

        let violations = detect_cb1001_oversized_tensor_count(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1001");
        assert!(matches!(violations[0].severity, Severity::Error));
    }

    #[test]
    fn test_cb1001_passes_normal_tensor_count() {
        let temp = TempDir::new().unwrap();
        let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
        header.extend_from_slice(&3u32.to_le_bytes()); // version
        header.extend_from_slice(&500u64.to_le_bytes()); // normal tensor_count
        header.extend_from_slice(&10u64.to_le_bytes()); // metadata_count
        header.resize(64, 0);
        fs::write(temp.path().join("good.gguf"), &header).unwrap();

        let violations = detect_cb1001_oversized_tensor_count(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb1006_detects_sharded_without_index() {
        let temp = TempDir::new().unwrap();
        // Create header bytes for SafeTensors (8-byte length + small JSON)
        let json_header = b"{\"tensor\":{\"dtype\":\"F32\",\"shape\":[1],\"data_offsets\":[0,4]}}";
        let header_len = json_header.len() as u64;
        let mut data = Vec::new();
        data.extend_from_slice(&header_len.to_le_bytes());
        data.extend_from_slice(json_header);
        data.extend_from_slice(&[0u8; 4]); // tensor data

        fs::write(
            temp.path().join("model-00001-of-00002.safetensors"),
            &data,
        )
        .unwrap();
        fs::write(
            temp.path().join("model-00002-of-00002.safetensors"),
            &data,
        )
        .unwrap();

        let violations = detect_cb1006_sharded_without_index(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1006");
    }

    #[test]
    fn test_cb1006_passes_with_index() {
        let temp = TempDir::new().unwrap();
        let json_header = b"{\"tensor\":{\"dtype\":\"F32\",\"shape\":[1],\"data_offsets\":[0,4]}}";
        let header_len = json_header.len() as u64;
        let mut data = Vec::new();
        data.extend_from_slice(&header_len.to_le_bytes());
        data.extend_from_slice(json_header);
        data.extend_from_slice(&[0u8; 4]);

        fs::write(
            temp.path().join("model-00001-of-00002.safetensors"),
            &data,
        )
        .unwrap();
        fs::write(
            temp.path().join("model-00002-of-00002.safetensors"),
            &data,
        )
        .unwrap();
        fs::write(
            temp.path().join("model.safetensors.index.json"),
            "{}",
        )
        .unwrap();

        let violations = detect_cb1006_sharded_without_index(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb1007_detects_large_file() {
        // We can't create a 10GB file in tests, but we can test the threshold logic
        let temp = TempDir::new().unwrap();
        // Create a small file — should NOT trigger
        fs::write(temp.path().join("small.gguf"), &[0u8; 100]).unwrap();
        let violations = detect_cb1007_excessive_file_size(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_walkdir_model_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("model.gguf"), &[0u8; 16]).unwrap();
        fs::write(temp.path().join("weights.safetensors"), &[0u8; 16]).unwrap();
        fs::write(temp.path().join("model.apr"), &[0u8; 16]).unwrap();
        fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

        let files = walkdir_model_files(temp.path());
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_model_format_from_extension() {
        assert_eq!(ModelFormat::from_extension("gguf"), Some(ModelFormat::Gguf));
        assert_eq!(ModelFormat::from_extension("apr"), Some(ModelFormat::Apr));
        assert_eq!(
            ModelFormat::from_extension("safetensors"),
            Some(ModelFormat::SafeTensors)
        );
        assert_eq!(ModelFormat::from_extension("rs"), None);
    }

    #[test]
    fn test_cb1004_detects_missing_architecture() {
        let temp = TempDir::new().unwrap();
        // Create GGUF file without "general.architecture" key
        let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
        header.extend_from_slice(&3u32.to_le_bytes()); // version
        header.extend_from_slice(&10u64.to_le_bytes()); // tensor_count
        header.extend_from_slice(&0u64.to_le_bytes()); // metadata_count
        header.resize(200, 0); // Pad to be > 100 bytes
        fs::write(temp.path().join("model.gguf"), &header).unwrap();

        let violations = detect_cb1004_missing_architecture(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1004");
    }

    #[test]
    fn test_cb1004_passes_with_architecture() {
        let temp = TempDir::new().unwrap();
        let mut header = vec![0x47u8, 0x47, 0x55, 0x46]; // GGUF magic
        header.extend_from_slice(&3u32.to_le_bytes());
        header.extend_from_slice(&10u64.to_le_bytes());
        header.extend_from_slice(&1u64.to_le_bytes()); // 1 metadata entry
        // Add "general.architecture" as a key string
        header.extend_from_slice(b"general.architecture");
        header.resize(200, 0);
        fs::write(temp.path().join("model.gguf"), &header).unwrap();

        let violations = detect_cb1004_missing_architecture(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb1005_detects_size_mismatch() {
        let temp = TempDir::new().unwrap();
        // Create tiny GGUF file claiming F32
        let mut header = vec![0x47u8, 0x47, 0x55, 0x46];
        header.extend_from_slice(&3u32.to_le_bytes());
        header.extend_from_slice(&10u64.to_le_bytes());
        header.extend_from_slice(&0u64.to_le_bytes());
        // File is only ~32 bytes but claims f32
        fs::write(temp.path().join("model-f32.gguf"), &header).unwrap();

        let violations = detect_cb1005_quantization_mismatch(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1005");
    }
}

// =============================================================================
// Tests for CB-800 Scala Best Practices
// =============================================================================

mod cb800_scala_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb800_detects_mutable_collection() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("App.scala"),
            "val cache = mutable.HashMap[String, Int]()\nval items = mutable.Buffer[Int]()",
        )
        .unwrap();

        let violations = detect_cb800_mutable_collection(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-800");
        assert!(violations[0].description.contains("mutable.HashMap"));
    }

    #[test]
    fn test_cb800_allows_import_of_mutable() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.collection.mutable.Map\nval x = Map(\"a\" -> 1)",
        )
        .unwrap();

        let violations = detect_cb800_mutable_collection(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb801_detects_null_literal() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val x: String = null\nval y = if (x == null) \"default\" else x",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-801");
    }

    #[test]
    fn test_cb801_allows_java_interop_null() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "@Nullable val x: String = null",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb801_no_false_positive_on_nullable_identifier() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val nullable = true\nval isNullable = false",
        )
        .unwrap();

        let violations = detect_cb801_null_usage(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb802_detects_wildcard_import() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import com.example.models._\nimport org.apache.spark.sql.*",
        )
        .unwrap();

        let violations = detect_cb802_wildcard_import(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-802");
    }

    #[test]
    fn test_cb802_allows_stdlib_wildcard() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.collection.immutable._\nimport java.util._",
        )
        .unwrap();

        let violations = detect_cb802_wildcard_import(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb803_detects_return_statement() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "def foo(x: Int): Int = {\n  if (x > 0) return x\n  x * -1\n}",
        )
        .unwrap();

        let violations = detect_cb803_return_statement(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-803");
    }

    #[test]
    fn test_cb804_detects_var_declaration() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "var count = 0\nprivate var state = \"init\"",
        )
        .unwrap();

        let violations = detect_cb804_var_declaration(temp.path());
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].pattern_id, "CB-804");
    }

    #[test]
    fn test_cb804_no_false_positive_on_val() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "val count = 0\nprivate val state = \"init\"",
        )
        .unwrap();

        let violations = detect_cb804_var_declaration(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_cb805_detects_blocking_in_future() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "import scala.concurrent.Future\nval f = Future {\n  Thread.sleep(1000)\n  42\n}",
        )
        .unwrap();

        let violations = detect_cb805_blocking_in_future(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-805");
        assert!(violations[0].description.contains("Thread.sleep"));
    }

    #[test]
    fn test_cb805_no_false_positive_outside_future() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("App.scala"),
            "def main(): Unit = {\n  Thread.sleep(1000)\n}",
        )
        .unwrap();

        let violations = detect_cb805_blocking_in_future(temp.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_scala_test_file_detection() {
        use std::path::Path;
        assert!(is_scala_test_file(Path::new("src/test/scala/AppTest.scala")));
        assert!(is_scala_test_file(Path::new("AppSpec.scala")));
        assert!(is_scala_test_file(Path::new("TestHelper.scala")));
        assert!(!is_scala_test_file(Path::new("src/main/scala/App.scala")));
    }

    #[test]
    fn test_scala_production_lines() {
        let content = "// comment\nval x = 1\n/* block */\nval y = 2\n\nval z = 3 // inline";
        let lines = compute_scala_production_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (2, "val x = 1".to_string()));
        assert_eq!(lines[1], (4, "val y = 2".to_string()));
        assert_eq!(lines[2], (6, "val z = 3".to_string()));
    }

    #[test]
    fn test_walkdir_scala_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("App.scala"), "object App").unwrap();
        fs::write(temp.path().join("build.sc"), "// mill build").unwrap();
        fs::write(temp.path().join("code.rs"), "fn main() {}").unwrap();

        let files = walkdir_scala_files(temp.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scala_skips_test_files() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("test");
        fs::create_dir_all(&test_dir).unwrap();
        fs::write(
            test_dir.join("AppTest.scala"),
            "var x = null\nimport foo._\nreturn 42",
        )
        .unwrap();

        // All detectors should skip test files
        assert_eq!(detect_cb800_mutable_collection(temp.path()).len(), 0);
        assert_eq!(detect_cb801_null_usage(temp.path()).len(), 0);
        assert_eq!(detect_cb802_wildcard_import(temp.path()).len(), 0);
        assert_eq!(detect_cb803_return_statement(temp.path()).len(), 0);
        assert_eq!(detect_cb804_var_declaration(temp.path()).len(), 0);
    }
}

// =============================================================================
// Tests for CB-513 through CB-518: Rust Best Practices (Extended)
// =============================================================================

#[cfg(test)]
mod cb513_to_cb518_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ---- CB-513: Silent Error Swallowing ----

    #[test]
    fn test_cb513_detects_unwrap_or_else_discard() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn load_config() -> Config {
    let val = std::env::var("KEY").unwrap_or_else(|_| "default".to_string());
    Config { val }
}
"#,
        )
        .unwrap();

        let violations = detect_cb513_silent_error_swallowing(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-513");
        assert!(violations[0].description.contains("unwrap_or_else"));
    }

    #[test]
    fn test_cb513_detects_map_err_discard() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("io.rs"),
            r#"
fn read_file() -> Result<String, MyError> {
    fs::read_to_string("f.txt").map_err(|_| MyError::IoFailed)
}
"#,
        )
        .unwrap();

        let violations = detect_cb513_silent_error_swallowing(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("map_err"));
    }

    #[test]
    fn test_cb513_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_it() {
        let v = "123".parse::<i32>().unwrap_or_else(|_| 0);
        assert_eq!(v, 123);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb513_silent_error_swallowing(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb513_skips_comments() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
// Note: we could use .unwrap_or_else(|_| default) here
fn foo() -> i32 { 42 }
"#,
        )
        .unwrap();

        let violations = detect_cb513_silent_error_swallowing(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-514: Debug Eprintln Leaks ----

    #[test]
    fn test_cb514_detects_debug_eprintln() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("parser.rs"),
            "fn parse(input: &str) {\n    eprintln!(\"[DEBUG] parsing: {}\", input);\n}\n",
        )
        .unwrap();

        let violations = detect_cb514_debug_eprintln_leaks(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-514");
    }

    #[test]
    fn test_cb514_detects_trace_eprintln() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("net.rs"),
            "fn connect() {\n    eprintln!(\"[TRACE] connecting to server\");\n}\n",
        )
        .unwrap();

        let violations = detect_cb514_debug_eprintln_leaks(temp.path());
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_cb514_allows_normal_eprintln() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("main.rs"),
            "fn main() {\n    eprintln!(\"Error: file not found\");\n}\n",
        )
        .unwrap();

        let violations = detect_cb514_debug_eprintln_leaks(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb514_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        eprintln!(\"[DEBUG] test output\");\n    }\n}\n",
        )
        .unwrap();

        let violations = detect_cb514_debug_eprintln_leaks(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-515: Catch-All Match Default ----

    #[test]
    fn test_cb515_detects_concrete_catch_all() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("arch.rs"),
            r#"
fn get_arch(name: &str) -> Architecture {
    match name {
        "gpt" => Architecture::Gpt,
        "llama" => Architecture::Llama,
        _ => Architecture::Qwen2,
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb515_catch_all_match_default(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-515");
        assert!(violations[0].description.contains("Architecture::Qwen2"));
    }

    #[test]
    fn test_cb515_allows_error_catch_all() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("parse.rs"),
            r#"
fn parse_mode(s: &str) -> Result<Mode, Error> {
    match s {
        "fast" => Ok(Mode::Fast),
        "slow" => Ok(Mode::Slow),
        _ => Err(Error::UnknownMode(s.to_string())),
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb515_catch_all_match_default(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb515_allows_none_catch_all() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lookup.rs"),
            r#"
fn find(key: &str) -> Option<Value> {
    match key {
        "a" => Some(Value::A),
        _ => None,
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb515_catch_all_match_default(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb515_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_match() {
        let result = match "x" {
            "a" => 1,
            _ => 99,
        };
        assert_eq!(result, 99);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb515_catch_all_match_default(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-516: Hardcoded Magic Numbers ----

    #[test]
    fn test_cb516_detects_magic_in_some() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn default_config() -> Config {
    Config {
        rope_theta: Some(10000.0),
        max_seq_len: Some(2048),
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
        assert!(violations.len() >= 1);
        assert_eq!(violations[0].pattern_id, "CB-516");
        assert!(violations[0].description.contains("10000.0"));
    }

    #[test]
    fn test_cb516_skips_const_declarations() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            "const MAX_RETRY: usize = 10000;\nstatic TIMEOUT: u64 = 30000;\n",
        )
        .unwrap();

        let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb516_skips_common_values() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("buf.rs"),
            r#"
fn create_buffer() -> Buffer {
    Buffer { size: Some(1024) }
}
"#,
        )
        .unwrap();

        let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb516_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let x = Some(99999.0);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb516_hardcoded_magic_numbers(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-517: Stale Debug Artifacts ----

    #[test]
    fn test_cb517_detects_atomic_counter() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("metrics.rs"),
            r#"
use std::sync::atomic::AtomicUsize;
static DEBUG_COUNTER: AtomicUsize = AtomicUsize::new(0);
fn process() {
    DEBUG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
"#,
        )
        .unwrap();

        let violations = detect_cb517_stale_debug_artifacts(temp.path());
        assert!(violations.len() >= 1);
        assert_eq!(violations[0].pattern_id, "CB-517");
        assert!(violations[0].description.contains("Atomic"));
    }

    #[test]
    fn test_cb517_detects_allow_unused_static() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("debug.rs"),
            "#[allow(unused)]\nstatic TRACE_LOG: bool = false;\n",
        )
        .unwrap();

        let violations = detect_cb517_stale_debug_artifacts(temp.path());
        assert!(violations.len() >= 1);
        assert!(violations[0].description.contains("allow(unused)"));
    }

    #[test]
    fn test_cb517_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    #[test]
    fn t() {}
}
"#,
        )
        .unwrap();

        let violations = detect_cb517_stale_debug_artifacts(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-518: Expensive Clone in Loop ----

    #[test]
    fn test_cb518_detects_excessive_clones_in_loop() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("process.rs"),
            r#"
fn process(items: &[Item], config: &Config) {
    for item in items {
        let a = config.name.clone();
        let b = config.path.clone();
        let c = config.data.clone();
        let d = config.extra.clone();
        do_work(item, &a, &b, &c, &d);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb518_expensive_clone_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-518");
        assert!(violations[0].description.contains("4 .clone()"));
    }

    #[test]
    fn test_cb518_allows_few_clones() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("small.rs"),
            r#"
fn process(items: &[Item]) {
    for item in items {
        let name = item.name.clone();
        process_name(&name);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb518_expensive_clone_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb518_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        for i in 0..10 {
            let a = "x".to_string().clone();
            let b = "y".to_string().clone();
            let c = "z".to_string().clone();
            let d = "w".to_string().clone();
        }
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb518_expensive_clone_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb518_detects_while_loop() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("iter.rs"),
            r#"
fn drain(queue: &mut Vec<Job>, cfg: &Config) {
    while let Some(job) = queue.pop() {
        let a = cfg.name.clone();
        let b = cfg.path.clone();
        let c = cfg.data.clone();
        let d = cfg.meta.clone();
        run(job, a, b, c, d);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb518_expensive_clone_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
    }
}

// =============================================================================
// Tests for CB-519 through CB-527: Aprender Bug Pattern Detection
// =============================================================================

#[cfg(test)]
mod cb519_to_cb527_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ---- CB-519: Lossy Data Pipeline ----

    #[test]
    fn test_cb519_detects_quantize_dequantize_roundtrip() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("export.rs"),
            r#"
fn convert_tensor(data: &[f32]) -> Vec<u8> {
    let quantized = quantize_q4(data);
    let dequantized = dequantize_q4(&quantized);
    pack_bytes(&dequantized)
}
"#,
        )
        .unwrap();

        let violations = detect_cb519_lossy_data_pipeline(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-519");
        assert!(violations[0].description.contains("quantize"));
    }

    #[test]
    fn test_cb519_detects_encode_decode_roundtrip() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("codec.rs"),
            r#"
fn process(data: &[u8]) -> Vec<u8> {
    let encoded = encode_base64(data);
    let decoded = decode_base64(&encoded);
    decoded
}
"#,
        )
        .unwrap();

        let violations = detect_cb519_lossy_data_pipeline(temp.path());
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_cb519_allows_single_direction() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("quant.rs"),
            r#"
fn compress(data: &[f32]) -> Vec<u8> {
    quantize_q4(data)
}
"#,
        )
        .unwrap();

        let violations = detect_cb519_lossy_data_pipeline(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb519_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_roundtrip() {
        let q = quantize(data);
        let d = dequantize(&q);
        assert_eq!(data, d);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb519_lossy_data_pipeline(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-520: Expensive Init in Hot Path ----

    #[test]
    fn test_cb520_detects_new_in_loop() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("handler.rs"),
            r#"
fn process(items: &[Item]) {
    for item in items {
        let client = HttpClient::new(config);
        let conn = Database::connect("url");
        client.send(item);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb520_expensive_init_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-520");
    }

    #[test]
    fn test_cb520_allows_single_init() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("simple.rs"),
            r#"
fn process(items: &[Item]) {
    for item in items {
        let result = String::new();
        process_item(item, &result);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb520_expensive_init_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb520_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        for i in 0..10 {
            let c = Client::new();
            let d = Database::connect("url");
            let f = File::open("test");
        }
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb520_expensive_init_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-521: Format Detection Without Magic Bytes ----

    #[test]
    fn test_cb521_detects_binary_read_without_magic() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("parser.rs"),
            r#"
fn parse_file(reader: &mut impl Read) -> Result<Header, Error> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let size = u64::from_le_bytes(buf);
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)?;
    Ok(Header { data })
}
"#,
        )
        .unwrap();

        let violations = detect_cb521_format_without_magic_bytes(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-521");
    }

    #[test]
    fn test_cb521_allows_with_magic_check() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("safe_parser.rs"),
            r#"
fn parse_file(reader: &mut impl Read) -> Result<Header, Error> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != FILE_MAGIC {
        return Err(Error::InvalidFormat);
    }
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(Header { size: u64::from_le_bytes(buf) })
}
"#,
        )
        .unwrap();

        let violations = detect_cb521_format_without_magic_bytes(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb521_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let mut buf = [0u8; 8];
        cursor.read_exact(&mut buf).unwrap();
        let val = u64::from_le_bytes(buf);
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb521_format_without_magic_bytes(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-522: Untested Path Normalization ----

    #[test]
    fn test_cb522_detects_path_manipulation_chains() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("uri.rs"),
            r#"
fn normalize_uri(uri: &str) -> String {
    let without_scheme = uri.strip_prefix("http://").unwrap_or(uri);
    let cleaned = without_scheme.replace("//", "/");
    let no_resolve = cleaned.replace("resolve/", "");
    let trimmed = no_resolve.trim_start_matches("http://");
    trimmed.to_string()
}
"#,
        )
        .unwrap();

        let violations = detect_cb522_untested_path_normalization(temp.path());
        assert!(violations.len() >= 1);
        assert_eq!(violations[0].pattern_id, "CB-522");
    }

    #[test]
    fn test_cb522_allows_simple_path_ops() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("simple.rs"),
            r#"
fn get_name(path: &Path) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}
"#,
        )
        .unwrap();

        let violations = detect_cb522_untested_path_normalization(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-523: External Config Over Embedded Metadata ----

    #[test]
    fn test_cb523_detects_sibling_config_discovery() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("loader.rs"),
            r#"
fn load_model(path: &Path) -> Model {
    let config_path = path.with_file_name("config.json");
    let config = fs::read_to_string(config_path).unwrap();
    parse_model(config)
}
"#,
        )
        .unwrap();

        let violations = detect_cb523_external_config_over_embedded(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-523");
    }

    #[test]
    fn test_cb523_allows_non_config_file_ops() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("util.rs"),
            r#"
fn get_log_path(path: &Path) -> PathBuf {
    path.with_file_name("output.log")
}
"#,
        )
        .unwrap();

        let violations = detect_cb523_external_config_over_embedded(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-524: Incomplete Enum Match Coverage ----

    #[test]
    fn test_cb524_detects_multiple_wildcard_matches() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("dispatch.rs"),
            r#"
fn get_name(arch: Architecture) -> &'static str {
    match arch {
        Architecture::Gpt => "gpt",
        Architecture::Llama => "llama",
        _ => "unknown",
    }
}
fn get_layers(arch: Architecture) -> usize {
    match arch {
        Architecture::Gpt => 12,
        _ => 32,
    }
}
fn get_hidden(arch: Architecture) -> usize {
    match arch {
        Architecture::Llama => 4096,
        _ => 768,
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb524_incomplete_enum_match(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-524");
        assert!(violations[0].description.contains("3"));
    }

    #[test]
    fn test_cb524_allows_few_wildcard_matches() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("simple.rs"),
            r#"
fn name(x: Kind) -> &'static str {
    match x {
        Kind::A => "a",
        _ => "other",
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb524_incomplete_enum_match(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb524_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    fn a() -> i32 { match x { X::A => 1, _ => 2 } }
    fn b() -> i32 { match x { X::B => 3, _ => 4 } }
    fn c() -> i32 { match x { X::C => 5, _ => 6 } }
}
"#,
        )
        .unwrap();

        let violations = detect_cb524_incomplete_enum_match(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-525: Hardcoded Field Names Without Aliases ----

    #[test]
    fn test_cb525_detects_many_get_without_fallback() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn parse_config(json: &serde_json::Value) -> Config {
    let hidden = json.get("hidden_size").unwrap();
    let layers = json.get("num_hidden_layers").unwrap();
    let heads = json.get("num_attention_heads").unwrap();
    let vocab = json.get("vocab_size").unwrap();
    let intermediate = json.get("intermediate_size").unwrap();
    Config { hidden, layers, heads, vocab, intermediate }
}
"#,
        )
        .unwrap();

        let violations = detect_cb525_hardcoded_field_names(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-525");
    }

    #[test]
    fn test_cb525_allows_with_or_fallback() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn parse_config(json: &serde_json::Value) -> Config {
    let hidden = json.get("hidden_size").or_else(|| json.get("n_embd")).unwrap();
    let layers = json.get("num_hidden_layers").or_else(|| json.get("n_layer")).unwrap();
    let heads = json.get("num_attention_heads").or_else(|| json.get("n_head")).unwrap();
    let vocab = json.get("vocab_size").unwrap();
    let intermediate = json.get("intermediate_size").or(json.get("n_inner")).unwrap();
    Config { hidden, layers, heads, vocab, intermediate }
}
"#,
        )
        .unwrap();

        let violations = detect_cb525_hardcoded_field_names(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb525_allows_few_gets() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("small.rs"),
            r#"
fn get_name(json: &serde_json::Value) -> String {
    json.get("name").unwrap().as_str().unwrap().to_string()
}
"#,
        )
        .unwrap();

        let violations = detect_cb525_hardcoded_field_names(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-526: Single-Path File Resolution ----

    #[test]
    fn test_cb526_detects_single_path_exists() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("discovery.rs"),
            r#"
fn find_tokenizer(model_path: &Path) -> Option<PathBuf> {
    if model_path.join("tokenizer.json").exists() {
        Some(model_path.join("tokenizer.json"))
    } else {
        None
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb526_single_path_resolution(temp.path());
        assert!(violations.len() >= 1);
        assert_eq!(violations[0].pattern_id, "CB-526");
    }

    #[test]
    fn test_cb526_allows_with_fallback() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("discovery.rs"),
            r#"
fn find_tokenizer(model_path: &Path) -> Option<PathBuf> {
    let tok_path = model_path.join("tokenizer.json");
    if tok_path.exists() || model_path.parent().map(|p| p.join("tokenizer.json").exists()).unwrap_or(false) {
        Some(tok_path)
    } else {
        None
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb526_single_path_resolution(temp.path());
        assert!(violations.is_empty());
    }

    // ---- CB-527: Incomplete Pattern List ----

    #[test]
    fn test_cb527_detects_classification_chain() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("classify.rs"),
            r#"
fn is_embedding(name: &str) -> bool {
    name.contains("embed") || name.contains("wte") || name.contains("wpe") || name.contains("position")
}
"#,
        )
        .unwrap();

        let violations = detect_cb527_incomplete_pattern_list(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-527");
    }

    #[test]
    fn test_cb527_allows_short_chains() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("simple.rs"),
            r#"
fn is_special(name: &str) -> bool {
    name.contains("test") || name.contains("bench")
}
"#,
        )
        .unwrap();

        let violations = detect_cb527_incomplete_pattern_list(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb527_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg(test)]
mod tests {
    fn check(s: &str) -> bool {
        s.contains("a") || s.contains("b") || s.contains("c") || s.contains("d")
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb527_incomplete_pattern_list(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-608: Unchecked nil, err Return Pattern (#181)
// =============================================================================

mod cb608_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb608_detects_unchecked_io_open() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local f = io.open('data.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-608");
        assert!(violations[0].description.contains("io.open"));
    }

    #[test]
    fn test_cb608_passes_when_error_captured() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local f, err = io.open('data.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb608_detects_unchecked_pcall() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "pcall(dangerous_function)\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("pcall"));
    }

    #[test]
    fn test_cb608_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app_test.lua"),
            "local f = io.open('test.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-609: assert() in Library Code (#193)
// =============================================================================

mod cb609_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb609_detects_assert_in_library() {
        let temp = TempDir::new().unwrap();
        // assert on line 8 (past the 5-line require-guard threshold)
        let code = "local M = {}\nlocal utils = require('utils')\n\
                     local fmt = string.format\nlocal insert = table.insert\n\
                     local pairs = pairs\nlocal type = type\n\
                     function M.process(data)\n  assert(type(data) == 'table')\n\
                     return data\nend\nreturn M\n";
        fs::write(temp.path().join("lib.lua"), code).unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(!violations.is_empty(), "Should detect assert() past line 5");
        assert_eq!(violations[0].pattern_id, "CB-609");
    }

    #[test]
    fn test_cb609_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_app.lua"),
            "assert(result == expected)\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb609_skips_module_require_guards() {
        let temp = TempDir::new().unwrap();
        // assert in first 5 lines (module-level guards) should be skipped
        fs::write(
            temp.path().join("lib.lua"),
            "local ok = pcall(require, 'ffi')\nassert(ok, 'FFI required')\nlocal M = {}\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb609_skips_test_framework_assertions() {
        let temp = TempDir::new().unwrap();
        // assert.is_true() is a test framework method, not plain assert()
        fs::write(
            temp.path().join("lib.lua"),
            "local M = {}\nfunction M.run()\n  -- noop\nend\nlocal x = assert.is_true\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-610: String Accumulator in Loop (#190)
// =============================================================================

mod cb610_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb610_detects_string_accumulator() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local result = ''\nfor _, item in ipairs(items) do\n  result = result .. item\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-610");
    }

    #[test]
    fn test_cb610_skips_single_use_concat() {
        let temp = TempDir::new().unwrap();
        // Single-use concat in loop is O(n), not O(n²) — should not be flagged
        fs::write(
            temp.path().join("app.lua"),
            "for _, item in ipairs(items) do\n  log('Processing: ' .. item.name)\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb610_skips_outside_loop() {
        let temp = TempDir::new().unwrap();
        // Accumulator outside loop is fine (not O(n²))
        fs::write(
            temp.path().join("app.lua"),
            "local msg = prefix .. suffix\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb610_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_builder.lua"),
            "local result = ''\nfor _, item in ipairs(items) do\n  result = result .. item\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-611: Weak Table Misuse (#186)
// =============================================================================

mod cb611_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb611_detects_string_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "k" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-611");
        assert!(violations[0].description.contains("string"));
    }

    #[test]
    fn test_cb611_detects_numeric_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("pool.lua"),
            r#"local pool = setmetatable({}, { __mode = "k" })
pool[123] = conn
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("numeric"));
    }

    #[test]
    fn test_cb611_ignores_weak_value_tables() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "v" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_ignores_weak_kv_tables() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "kv" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_allows_table_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("refs.lua"),
            r#"local refs = setmetatable({}, { __mode = "k" })
refs[obj] = true
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "k" })
cache["key"] = val
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-612: Test Framework Detection (#184)
// =============================================================================

mod cb612_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb612_detects_busted_via_spec_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("app_spec.lua"), "describe('app', function() end)\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-612");
        assert!(violations[0].description.contains("busted"));
    }

    #[test]
    fn test_cb612_detects_busted_via_config() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join(".busted"), "return { default = { verbose = true } }\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("busted"));
    }

    #[test]
    fn test_cb612_detects_test_nginx() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::create_dir(temp.path().join("t")).unwrap();
        fs::write(temp.path().join("t/001-basic.t"), "use Test::Nginx::Socket;\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("Test::Nginx"));
    }

    #[test]
    fn test_cb612_detects_hybrid_busted_and_test_nginx() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("handler_spec.lua"), "describe('handler', function() end)\n").unwrap();
        fs::create_dir(temp.path().join("t")).unwrap();
        fs::write(temp.path().join("t/001.t"), "use Test::Nginx;\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("busted"));
        assert!(violations[0].description.contains("Test::Nginx"));
    }

    #[test]
    fn test_cb612_no_framework_few_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("init.lua"), "return {}\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        // Less than 3 Lua files, no warning
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb612_no_framework_many_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("b.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("c.lua"), "return {}\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("No Lua test framework"));
    }

    #[test]
    fn test_cb612_detects_luaunit() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(
            temp.path().join("test_app.lua"),
            "local lu = require('luaunit')\nfunction test_foo() end\n",
        )
        .unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("LuaUnit"));
    }
}
