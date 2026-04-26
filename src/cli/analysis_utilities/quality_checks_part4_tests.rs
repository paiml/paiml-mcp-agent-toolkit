// Wave 36 PR6 — pure-helper coverage for quality_checks_part4.rs.
// Filesystem-bound siblings (analyze_project_files, analyze_complexity_file,
// analyze_file_complexity_async, format_dead_code_output, detect_toolchain)
// are not exercised here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_checks_part4_tests {
    use super::*;
    use std::path::Path;

    // ── build_complexity_thresholds ────────────────────────────────────────

    #[test]
    fn test_build_complexity_thresholds_both_none_returns_defaults_10_15() {
        assert_eq!(build_complexity_thresholds(None, None), (10, 15));
    }

    #[test]
    fn test_build_complexity_thresholds_overrides_used() {
        assert_eq!(build_complexity_thresholds(Some(20), Some(30)), (20, 30));
    }

    #[test]
    fn test_build_complexity_thresholds_partial_override_first() {
        // Only cyclomatic given → cognitive falls back to 15.
        assert_eq!(build_complexity_thresholds(Some(20), None), (20, 15));
    }

    #[test]
    fn test_build_complexity_thresholds_partial_override_second() {
        // Only cognitive given → cyclomatic falls back to 10.
        assert_eq!(build_complexity_thresholds(None, Some(30)), (10, 30));
    }

    // ── get_file_extensions ────────────────────────────────────────────────

    #[test]
    fn test_get_file_extensions_rust() {
        assert_eq!(get_file_extensions(Some("rust")), vec!["rs"]);
    }

    #[test]
    fn test_get_file_extensions_typescript_includes_jsx() {
        // PIN: "typescript" maps to BOTH ts/tsx AND js/jsx (Deno project bias).
        let exts = get_file_extensions(Some("typescript"));
        assert!(exts.contains(&"ts"));
        assert!(exts.contains(&"tsx"));
        assert!(exts.contains(&"js"));
        assert!(exts.contains(&"jsx"));
    }

    #[test]
    fn test_get_file_extensions_deno_aliases_typescript() {
        assert_eq!(
            get_file_extensions(Some("deno")),
            vec!["ts", "tsx", "js", "jsx"]
        );
    }

    #[test]
    fn test_get_file_extensions_javascript_excludes_ts() {
        // PIN: PMAT-BUG-002 — "javascript" must NOT include ts/tsx (those need typescript toolchain).
        let exts = get_file_extensions(Some("javascript"));
        assert_eq!(exts, vec!["js", "jsx"]);
    }

    #[test]
    fn test_get_file_extensions_c_includes_h() {
        assert_eq!(get_file_extensions(Some("c")), vec!["c", "h"]);
    }

    #[test]
    fn test_get_file_extensions_cpp_includes_h_and_hxx() {
        // PIN: PMAT-BUG-004 — cpp toolchain must include `.h` (some C++ headers use it).
        let exts = get_file_extensions(Some("cpp"));
        assert!(exts.contains(&"cpp"));
        assert!(exts.contains(&"h"));
        assert!(exts.contains(&"hpp"));
        assert!(exts.contains(&"hxx"));
    }

    #[test]
    fn test_get_file_extensions_unknown_falls_back_to_rust() {
        // PIN: silent fallback — unknown toolchains coerce to rust, no error.
        assert_eq!(get_file_extensions(Some("klingon")), vec!["rs"]);
    }

    #[test]
    fn test_get_file_extensions_none_returns_all_supported() {
        // PIN: Issue #42 fix — None must scan ALL supported languages, not just rust.
        let exts = get_file_extensions(None);
        assert!(exts.contains(&"rs"));
        assert!(exts.contains(&"py"));
        assert!(exts.contains(&"go"));
        assert!(exts.contains(&"java"));
        assert!(exts.contains(&"swift"));
        assert!(exts.contains(&"lean"));
        assert!(
            exts.len() > 15,
            "all-langs default should be wide: got {} ext(s)",
            exts.len()
        );
    }

    #[test]
    fn test_get_file_extensions_kotlin_includes_kts() {
        let exts = get_file_extensions(Some("kotlin"));
        assert_eq!(exts, vec!["kt", "kts"]);
    }

    #[test]
    fn test_get_file_extensions_csharp_alias_cs() {
        assert_eq!(get_file_extensions(Some("cs")), vec!["cs"]);
        assert_eq!(get_file_extensions(Some("csharp")), vec!["cs"]);
    }

    #[test]
    fn test_get_file_extensions_python_uv_alias() {
        assert_eq!(get_file_extensions(Some("python-uv")), vec!["py"]);
        assert_eq!(get_file_extensions(Some("python")), vec!["py"]);
    }

    // ── is_test_file ───────────────────────────────────────────────────────

    #[test]
    fn test_is_test_file_underscore_test_suffix() {
        assert!(is_test_file("foo_test.rs"));
    }

    #[test]
    fn test_is_test_file_underscore_tests_suffix() {
        assert!(is_test_file("foo_tests.rs"));
    }

    #[test]
    fn test_is_test_file_tests_suffix_no_underscore() {
        assert!(is_test_file("integration_tests.rs"));
        assert!(is_test_file("tests.rs"));
    }

    #[test]
    fn test_is_test_file_test_prefix() {
        assert!(is_test_file("test_foo.rs"));
        assert!(is_test_file("tests_foo.rs"));
    }

    #[test]
    fn test_is_test_file_contains_test_harness() {
        assert!(is_test_file("my_test_harness.rs"));
        assert!(is_test_file("test_helpers_x.rs"));
    }

    #[test]
    fn test_is_test_file_property_test_contains() {
        assert!(is_test_file("foo_property_test.rs"));
        assert!(is_test_file("property_tests_foo.rs"));
    }

    #[test]
    fn test_is_test_file_normal_module_not_test() {
        assert!(!is_test_file("main.rs"));
        assert!(!is_test_file("lib.rs"));
        assert!(!is_test_file("foo.rs"));
    }

    // ── is_example_or_demo_file ───────────────────────────────────────────

    #[test]
    fn test_is_example_or_demo_file_example_prefix() {
        assert!(is_example_or_demo_file("example_foo.rs"));
    }

    #[test]
    fn test_is_example_or_demo_file_demo_prefix() {
        assert!(is_example_or_demo_file("demo_app.rs"));
    }

    #[test]
    fn test_is_example_or_demo_file_contains_underscore_example() {
        assert!(is_example_or_demo_file("foo_example.rs"));
        assert!(is_example_or_demo_file("foo_demo.rs"));
    }

    #[test]
    fn test_is_example_or_demo_file_normal_not_example() {
        assert!(!is_example_or_demo_file("foo.rs"));
        assert!(!is_example_or_demo_file("examples.rs")); // no underscore
    }

    // ── is_benchmark_file ─────────────────────────────────────────────────

    #[test]
    fn test_is_benchmark_file_bench_suffix() {
        assert!(is_benchmark_file("hash_bench.rs"));
        assert!(is_benchmark_file("hash_benchmark.rs"));
    }

    #[test]
    fn test_is_benchmark_file_bench_prefix_with_underscore() {
        // PIN: matcher uses `bench_` not `^bench` — `benchmark.rs` (no underscore) is NOT detected.
        assert!(is_benchmark_file("bench_hash.rs"));
        assert!(is_benchmark_file("benchmark_hash.rs"));
    }

    #[test]
    fn test_is_benchmark_file_benchmark_no_underscore_not_detected() {
        // PIN: bare "benchmark.rs" without underscore separator is NOT flagged.
        assert!(!is_benchmark_file("benchmark.rs"));
    }

    // ── is_mock_or_stub_file ──────────────────────────────────────────────

    #[test]
    fn test_is_mock_or_stub_file_mock_prefix() {
        assert!(is_mock_or_stub_file("mock_db.rs"));
    }

    #[test]
    fn test_is_mock_or_stub_file_stub_prefix() {
        assert!(is_mock_or_stub_file("stub_handler.rs"));
        assert!(is_mock_or_stub_file("stubs_handler.rs"));
    }

    #[test]
    fn test_is_mock_or_stub_file_contains_underscore_mock() {
        assert!(is_mock_or_stub_file("foo_mock.rs"));
        assert!(is_mock_or_stub_file("foo_stub.rs"));
        assert!(is_mock_or_stub_file("foo_stubs.rs"));
    }

    #[test]
    fn test_is_mock_or_stub_file_normal_not_mock() {
        assert!(!is_mock_or_stub_file("handler.rs"));
    }

    // ── is_excluded_filename (composes 4 sub-checks) ──────────────────────

    #[test]
    fn test_is_excluded_filename_test_file() {
        assert!(is_excluded_filename("foo_test.rs"));
    }

    #[test]
    fn test_is_excluded_filename_example() {
        assert!(is_excluded_filename("example_x.rs"));
    }

    #[test]
    fn test_is_excluded_filename_bench() {
        assert!(is_excluded_filename("bench_x.rs"));
    }

    #[test]
    fn test_is_excluded_filename_mock() {
        assert!(is_excluded_filename("mock_db.rs"));
    }

    #[test]
    fn test_is_excluded_filename_normal() {
        assert!(!is_excluded_filename("main.rs"));
        assert!(!is_excluded_filename("lib.rs"));
    }

    // ── is_excluded_directory ─────────────────────────────────────────────

    #[test]
    fn test_is_excluded_directory_target_in_path() {
        assert!(is_excluded_directory("/repo/target/debug/foo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_node_modules_in_path() {
        assert!(is_excluded_directory("/repo/node_modules/x/y.js"));
    }

    #[test]
    fn test_is_excluded_directory_tests_in_path() {
        // PIN: `/tests/` directory IS excluded (treated like build artifacts).
        assert!(is_excluded_directory("/repo/tests/integration/foo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_examples_in_path() {
        assert!(is_excluded_directory("/repo/examples/demo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_test_dash_prefix_pattern() {
        // PIN: `/test-foo/` style patterns are flagged via `/test-` substring.
        assert!(is_excluded_directory("/repo/test-fixtures/foo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_first_component_target() {
        // PIN: relative paths beginning with `./target/` strip the `./` prefix
        // and check first component.
        assert!(is_excluded_directory("./target/foo.rs"));
        assert!(is_excluded_directory("target/foo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_backslash_normalized() {
        // PIN: Windows-style backslashes are normalized to forward slashes
        // before matching. (Linux paths with backslashes shouldn't occur in
        // practice but the normalization is unconditional.)
        assert!(is_excluded_directory("C:\\repo\\target\\debug\\foo.rs"));
    }

    #[test]
    fn test_is_excluded_directory_normal_src_not_excluded() {
        assert!(!is_excluded_directory("/repo/src/main.rs"));
        assert!(!is_excluded_directory("src/main.rs"));
    }

    #[test]
    fn test_is_excluded_directory_dotted_dirs_excluded() {
        // PIN: `.git`, `.cargo`, `.idea`, `.vscode` are all flagged.
        assert!(is_excluded_directory("/repo/.git/HEAD"));
        assert!(is_excluded_directory("/repo/.idea/workspace.xml"));
    }

    // ── should_analyze_file ───────────────────────────────────────────────

    #[test]
    fn test_should_analyze_file_wrong_extension_rejected() {
        let path = Path::new("/repo/src/main.txt");
        let project = Path::new("/repo");
        assert!(!should_analyze_file(path, project, &["rs"], &[]));
    }

    #[test]
    fn test_should_analyze_file_correct_extension_no_include_uses_exclusions() {
        let path = Path::new("/repo/src/main.rs");
        let project = Path::new("/repo");
        assert!(should_analyze_file(path, project, &["rs"], &[]));
    }

    #[test]
    fn test_should_analyze_file_correct_extension_excluded_dir_rejected() {
        let path = Path::new("/repo/target/debug/build.rs");
        let project = Path::new("/repo");
        assert!(!should_analyze_file(path, project, &["rs"], &[]));
    }

    #[test]
    fn test_should_analyze_file_correct_extension_test_filename_rejected() {
        let path = Path::new("/repo/src/foo_test.rs");
        let project = Path::new("/repo");
        assert!(!should_analyze_file(path, project, &["rs"], &[]));
    }

    #[test]
    fn test_should_analyze_file_with_include_pattern_match() {
        let path = Path::new("/repo/src/main.rs");
        let project = Path::new("/repo");
        assert!(should_analyze_file(
            path,
            project,
            &["rs"],
            &["src/**/*.rs".to_string()]
        ));
    }

    #[test]
    fn test_should_analyze_file_with_include_pattern_no_match_rejected() {
        let path = Path::new("/repo/src/main.rs");
        let project = Path::new("/repo");
        assert!(!should_analyze_file(
            path,
            project,
            &["rs"],
            &["lib/**/*.rs".to_string()]
        ));
    }

    // ── add_top_files_ranking ─────────────────────────────────────────────

    fn fake_metric(file: &str) -> crate::services::complexity::FileComplexityMetrics {
        crate::services::complexity::FileComplexityMetrics {
            path: file.to_string(),
            total_complexity: crate::services::complexity::ComplexityMetrics::default(),
            functions: vec![],
            classes: vec![],
        }
    }

    #[test]
    fn test_add_top_files_ranking_zero_returns_all() {
        let files = vec![
            fake_metric("a.rs"),
            fake_metric("b.rs"),
            fake_metric("c.rs"),
        ];
        let result = add_top_files_ranking(files, 0);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_add_top_files_ranking_truncates_to_n() {
        let files = vec![
            fake_metric("a.rs"),
            fake_metric("b.rs"),
            fake_metric("c.rs"),
        ];
        let result = add_top_files_ranking(files, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "a.rs");
    }

    #[test]
    fn test_add_top_files_ranking_n_larger_than_input_returns_all() {
        let files = vec![fake_metric("a.rs")];
        let result = add_top_files_ranking(files, 10);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_add_top_files_ranking_empty_returns_empty() {
        let result = add_top_files_ranking(vec![], 5);
        assert!(result.is_empty());
    }
}
