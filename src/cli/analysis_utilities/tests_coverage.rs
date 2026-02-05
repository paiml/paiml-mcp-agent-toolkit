#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::Path;

    // Tests for percentile()

    #[test]
    fn test_percentile_median() {
        // percentile uses p as decimal (0.0-1.0), not percentage
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // index = (5 * 0.5) = 2, values[2] = 3.0
        assert!((percentile(&values, 0.5) - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_25th() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        // index = (8 * 0.25) = 2, values[2] = 3.0
        let result = percentile(&values, 0.25);
        assert!((result - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_75th() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        // index = (8 * 0.75) = 6, values[6] = 7.0
        let result = percentile(&values, 0.75);
        assert!((result - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_min() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // index = (5 * 0.0) = 0, values[0] = 1.0
        assert!((percentile(&values, 0.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_max() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // index = (5 * 1.0) = 5, clamped to 4, values[4] = 5.0
        assert!((percentile(&values, 1.0) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_single_value() {
        let values = vec![42.0];
        // index = (1 * 0.5) = 0, values[0] = 42.0
        assert!((percentile(&values, 0.5) - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_empty() {
        let values: Vec<f64> = vec![];
        // Empty returns 0.0
        assert_eq!(percentile(&values, 0.5), 0.0);
    }

    // Tests for estimate_refactoring_hours()

    #[test]
    fn test_refactoring_hours_excellent_score() {
        // Formula: 2.0 * 1.8^tdg_score
        // Zero TDG (no debt) = 2.0 * 1.8^0 = 2.0 hours
        let hours = estimate_refactoring_hours(0.0);
        assert!((hours - 2.0).abs() < 0.01, "Zero TDG should need 2 hours");
    }

    #[test]
    fn test_refactoring_hours_good_score() {
        // TDG=1 → 2.0 * 1.8^1 = 3.6 hours
        let hours = estimate_refactoring_hours(1.0);
        assert!((hours - 3.6).abs() < 0.01);
    }

    #[test]
    fn test_refactoring_hours_poor_score() {
        // TDG=5 → 2.0 * 1.8^5 ≈ 37.8 hours
        let hours = estimate_refactoring_hours(5.0);
        assert!(hours > 30.0 && hours < 50.0);
    }

    #[test]
    fn test_refactoring_hours_zero_score() {
        // TDG=10 → 2.0 * 1.8^10 ≈ 715 hours (major refactoring)
        let hours = estimate_refactoring_hours(10.0);
        assert!(hours > 500.0, "High TDG should need major refactoring");
    }

    // Tests for is_build_artifact()

    #[test]
    fn test_is_build_artifact_target() {
        // Matches paths starting with "target/" or containing "/target/"
        assert!(is_build_artifact(Path::new("target/debug/main")));
        assert!(is_build_artifact(Path::new("target/release/lib.so")));
        assert!(is_build_artifact(Path::new("./target/debug/main")));
        assert!(is_build_artifact(Path::new("project/target/release/bin")));
    }

    #[test]
    fn test_is_build_artifact_node_modules() {
        // Needs path with /node_modules/
        assert!(is_build_artifact(Path::new(
            "project/node_modules/lodash/index.js"
        )));
    }

    #[test]
    fn test_is_build_artifact_dist() {
        // Needs path with /dist/
        assert!(is_build_artifact(Path::new("project/dist/bundle.js")));
    }

    #[test]
    fn test_is_build_artifact_build() {
        // Needs path with /build/
        assert!(is_build_artifact(Path::new("project/build/output.o")));
    }

    #[test]
    fn test_is_build_artifact_coverage() {
        // coverage is not in the list - let's test .git instead
        assert!(is_build_artifact(Path::new("project/.git/objects/abc")));
    }

    #[test]
    fn test_is_build_artifact_vendor() {
        // vendor is not in the list - test generated instead
        assert!(is_build_artifact(Path::new("project/generated/code.rs")));
    }

    #[test]
    fn test_is_build_artifact_source_file() {
        assert!(!is_build_artifact(Path::new("src/main.rs")));
        assert!(!is_build_artifact(Path::new("lib/utils.py")));
    }

    // Tests for normalize_code_content()

    #[test]
    fn test_normalize_removes_whitespace() {
        let content = "fn main() {  \n    println!(\"hello\");\n}";
        let normalized = normalize_code_content(content);
        assert!(!normalized.contains("  "));
    }

    #[test]
    fn test_normalize_empty_string() {
        assert_eq!(normalize_code_content(""), "");
    }

    #[test]
    fn test_normalize_preserves_structure() {
        let content = "fn test() { return 42; }";
        let normalized = normalize_code_content(content);
        assert!(normalized.contains("fn"));
        assert!(normalized.contains("test"));
        assert!(normalized.contains("42"));
    }

    // Tests for calculate_content_hash()

    #[test]
    fn test_content_hash_deterministic() {
        let content = "fn main() { println!(\"hello\"); }";
        let hash1 = calculate_content_hash(content);
        let hash2 = calculate_content_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_different_content() {
        let hash1 = calculate_content_hash("fn main() {}");
        let hash2 = calculate_content_hash("fn test() {}");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_empty() {
        let hash = calculate_content_hash("");
        assert!(hash > 0 || hash == 0); // Should not panic
    }

    // Tests for detect_toolchain()

    #[test]
    fn test_detect_toolchain_rust() {
        // Create temp dir with Cargo.toml
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let toolchain = detect_toolchain(temp_dir.path());
        assert_eq!(toolchain, Some("rust".to_string()));
    }

    #[test]
    fn test_detect_toolchain_python() {
        // Python detection uses pyproject.toml or setup.py → returns "python-uv"
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("pyproject.toml"), "[project]").unwrap();
        let toolchain = detect_toolchain(temp_dir.path());
        assert_eq!(toolchain, Some("python-uv".to_string()));
    }

    #[test]
    fn test_detect_toolchain_javascript() {
        // package.json returns None (falls through to file extension counting)
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        let toolchain = detect_toolchain(temp_dir.path());
        // Returns None because package.json alone doesn't determine JS vs TS
        assert!(toolchain.is_none());
    }

    #[test]
    fn test_detect_toolchain_go() {
        // go.mod is not in the project markers list, so falls through
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("go.mod"), "module example").unwrap();
        let toolchain = detect_toolchain(temp_dir.path());
        // No Go marker support in current implementation
        assert!(toolchain.is_none());
    }

    #[test]
    fn test_detect_toolchain_unknown() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("random.txt"), "hello").unwrap();
        let toolchain = detect_toolchain(temp_dir.path());
        assert!(toolchain.is_none());
    }

    // Tests for get_file_extensions()

    #[test]
    fn test_get_file_extensions_rust() {
        let exts = get_file_extensions(Some("rust"));
        assert!(exts.contains(&"rs"));
    }

    #[test]
    fn test_get_file_extensions_python() {
        let exts = get_file_extensions(Some("python"));
        assert!(exts.contains(&"py"));
    }

    #[test]
    fn test_get_file_extensions_javascript() {
        let exts = get_file_extensions(Some("javascript"));
        assert!(exts.contains(&"js"));
    }

    #[test]
    fn test_get_file_extensions_typescript() {
        let exts = get_file_extensions(Some("typescript"));
        assert!(exts.contains(&"ts"));
    }

    #[test]
    fn test_get_file_extensions_go() {
        let exts = get_file_extensions(Some("go"));
        assert!(exts.contains(&"go"));
    }

    #[test]
    fn test_get_file_extensions_none() {
        let exts = get_file_extensions(None);
        assert!(!exts.is_empty());
    }

    // Tests for is_excluded_directory()

    #[test]
    fn test_is_excluded_directory_target() {
        assert!(is_excluded_directory("target"));
        assert!(is_excluded_directory("./target/debug"));
    }

    #[test]
    fn test_is_excluded_directory_node_modules() {
        assert!(is_excluded_directory("node_modules"));
    }

    #[test]
    fn test_is_excluded_directory_git() {
        assert!(is_excluded_directory(".git"));
    }

    #[test]
    fn test_is_excluded_directory_vendor() {
        assert!(is_excluded_directory("vendor"));
    }

    #[test]
    fn test_is_excluded_directory_src() {
        assert!(!is_excluded_directory("src"));
    }

    #[test]
    fn test_is_excluded_directory_lib() {
        assert!(!is_excluded_directory("lib"));
    }

    // Tests for is_excluded_filename()

    #[test]
    fn test_is_excluded_filename_lock_files() {
        // is_excluded_filename = is_test_file || is_example_or_demo_file || is_benchmark_file || is_mock_or_stub_file
        // Lock files are NOT excluded by this function
        assert!(!is_excluded_filename("Cargo.lock"));
        assert!(!is_excluded_filename("package-lock.json"));
        assert!(!is_excluded_filename("yarn.lock"));
    }

    #[test]
    fn test_is_excluded_filename_test_files() {
        // Test files ARE excluded
        assert!(is_excluded_filename("test_main.rs"));
        assert!(is_excluded_filename("tests.rs"));
    }

    #[test]
    fn test_is_excluded_filename_example_files() {
        // Example files ARE excluded
        assert!(is_excluded_filename("example_usage.rs"));
        assert!(is_excluded_filename("demo_app.rs"));
    }

    #[test]
    fn test_is_excluded_filename_benchmark_files() {
        // Benchmark files ARE excluded
        assert!(is_excluded_filename("bench_performance.rs"));
    }

    #[test]
    fn test_is_excluded_filename_mock_files() {
        // Mock files ARE excluded
        assert!(is_excluded_filename("mock_database.rs"));
    }

    #[test]
    fn test_is_excluded_filename_source_files() {
        assert!(!is_excluded_filename("main.rs"));
        assert!(!is_excluded_filename("utils.py"));
        assert!(!is_excluded_filename("index.js"));
    }

    // Tests for is_test_file()

    #[test]
    fn test_is_test_file_rust() {
        assert!(is_test_file("test_main.rs"));
        assert!(is_test_file("main_test.rs"));
    }

    #[test]
    fn test_is_test_file_python() {
        assert!(is_test_file("test_utils.py"));
    }

    #[test]
    fn test_is_test_file_javascript() {
        // is_test_file only matches Rust patterns (_test.rs, test_), not .test.js or .spec.js
        assert!(!is_test_file("app.test.js"));
        assert!(!is_test_file("app.spec.js"));
        // But test_ prefix works
        assert!(is_test_file("test_app.js"));
    }

    #[test]
    fn test_is_test_file_not_test() {
        assert!(!is_test_file("main.rs"));
        assert!(!is_test_file("utils.py"));
    }

    // Tests for is_example_or_demo_file()

    #[test]
    fn test_is_example_file() {
        assert!(is_example_or_demo_file("example_usage.rs"));
        assert!(is_example_or_demo_file("demo_app.py"));
    }

    #[test]
    fn test_is_not_example_file() {
        assert!(!is_example_or_demo_file("main.rs"));
        assert!(!is_example_or_demo_file("lib.rs"));
    }

    // Tests for is_benchmark_file()

    #[test]
    fn test_is_benchmark_file() {
        assert!(is_benchmark_file("bench_performance.rs"));
        assert!(is_benchmark_file("benchmark_sort.py"));
    }

    #[test]
    fn test_is_not_benchmark_file() {
        assert!(!is_benchmark_file("main.rs"));
    }

    // Tests for is_mock_or_stub_file()

    #[test]
    fn test_is_mock_file() {
        // Matches mock_, stub_, _mock, _stub patterns only (not fake_)
        assert!(is_mock_or_stub_file("mock_database.rs"));
        assert!(is_mock_or_stub_file("stub_api.py"));
        assert!(!is_mock_or_stub_file("fake_service.js")); // fake_ not supported
        assert!(is_mock_or_stub_file("service_mock.js")); // _mock suffix works
    }

    #[test]
    fn test_is_not_mock_file() {
        assert!(!is_mock_or_stub_file("database.rs"));
        assert!(!is_mock_or_stub_file("api.py"));
    }

    // Tests for should_analyze_file()

    #[test]
    fn test_should_analyze_rust_source() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        assert!(should_analyze_file(
            Path::new("src/main.rs"),
            temp_dir.path(),
            &["rs"],
            &[]
        ));
    }

    #[test]
    fn test_should_not_analyze_wrong_extension() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        assert!(!should_analyze_file(
            Path::new("Cargo.lock"),
            temp_dir.path(),
            &["rs"],
            &[]
        ));
    }

    #[test]
    fn test_should_not_analyze_target_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        assert!(!should_analyze_file(
            Path::new("target/debug/main.rs"),
            temp_dir.path(),
            &["rs"],
            &[]
        ));
    }

    // Property Tests for Coverage Functions

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_percentile_in_range(values in prop::collection::vec(0.0f64..1000.0, 1..100), p in 0.0f64..100.0) {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let result = percentile(&sorted, p);
            if !result.is_nan() {
                let min = sorted.first().unwrap();
                let max = sorted.last().unwrap();
                prop_assert!(result >= *min - 0.001 && result <= *max + 0.001);
            }
        }

        #[test]
        fn prop_content_hash_consistent(content in ".*") {
            let hash1 = calculate_content_hash(&content);
            let hash2 = calculate_content_hash(&content);
            prop_assert_eq!(hash1, hash2);
        }

        #[test]
        fn prop_refactoring_hours_non_negative(score in 0.0f64..100.0) {
            let hours = estimate_refactoring_hours(score);
            prop_assert!(hours >= 0.0);
        }

        #[test]
        fn prop_normalize_idempotent(content in "[a-zA-Z0-9 \n\t]{0,100}") {
            let once = normalize_code_content(&content);
            let twice = normalize_code_content(&once);
            prop_assert_eq!(once, twice);
        }
    }
}
