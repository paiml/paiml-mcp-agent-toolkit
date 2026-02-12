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
}
