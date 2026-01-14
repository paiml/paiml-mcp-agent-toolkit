//! Integration tests for pattern helpers with FileFilter
//!
//! Demonstrates the unified include/exclude system working end-to-end

use super::file_filter::FileFilter;
use super::pattern_helpers::*;
use std::path::Path;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_unified_pattern_system_end_to_end() {
        // Test the unified system with various pattern types

        // Scenario 1: Single patterns (Option<String> style)
        let include_single = Some("**/*.rs".to_string());
        let exclude_single = Some("target/**".to_string());

        let filter = FileFilter::from_optional(&include_single, &exclude_single).unwrap();

        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(!filter.should_include(Path::new("target/debug/main")));
        assert!(!filter.should_include(Path::new("src/main.py")));

        // Scenario 2: Multiple patterns (Vec<String> style)
        let include_multi = vec!["**/*.rs".to_string(), "**/*.toml".to_string()];
        let exclude_multi = vec!["target/**".to_string(), ".*/**".to_string()];

        let filter2 = FileFilter::new(include_multi, exclude_multi).unwrap();

        assert!(filter2.should_include(Path::new("src/lib.rs")));
        assert!(filter2.should_include(Path::new("Cargo.toml")));
        assert!(!filter2.should_include(Path::new("target/debug/app")));
        assert!(!filter2.should_include(Path::new(".git/config")));
        assert!(!filter2.should_include(Path::new("README.md")));

        // Scenario 3: Comma-separated patterns
        let comma_patterns = vec!["**/*.rs,**/*.py".to_string(), "tests/**".to_string()];
        let expanded = expand_patterns(&comma_patterns);
        assert_eq!(expanded, vec!["**/*.rs", "**/*.py", "tests/**"]);

        let filter3 = FileFilter::new(expanded, vec![]).unwrap();
        assert!(filter3.should_include(Path::new("src/main.rs")));
        assert!(filter3.should_include(Path::new("script.py")));
        assert!(filter3.should_include(Path::new("tests/unit.rs")));

        // Scenario 4: Default patterns
        let defaults = default_exclude_patterns();
        let filter4 = FileFilter::new(vec![], defaults).unwrap();

        assert!(filter4.should_include(Path::new("src/main.rs")));
        assert!(!filter4.should_include(Path::new("target/debug/app")));
        assert!(!filter4.should_include(Path::new("node_modules/package/index.js")));
        assert!(!filter4.should_include(Path::new(".git/HEAD")));

        // Scenario 5: Common code patterns
        let code_patterns = common_code_patterns();
        let filter5 = FileFilter::new(code_patterns, vec![]).unwrap();

        assert!(filter5.should_include(Path::new("src/main.rs")));
        assert!(filter5.should_include(Path::new("script.py")));
        assert!(filter5.should_include(Path::new("app.js")));
        assert!(filter5.should_include(Path::new("types.ts")));
        assert!(filter5.should_include(Path::new("main.cpp")));
        assert!(!filter5.should_include(Path::new("README.md")));
        assert!(!filter5.should_include(Path::new("image.png")));
    }

    #[test]
    fn test_pattern_validation_integration() {
        // Valid patterns should work
        let valid_patterns = vec!["**/*.rs".to_string(), "src/**".to_string()];
        assert!(FileFilter::new(valid_patterns, vec![]).is_ok());

        // Invalid patterns should fail
        let invalid_patterns = vec!["**/*[invalid".to_string()];
        assert!(FileFilter::new(invalid_patterns, vec![]).is_err());

        // Mixed valid/invalid should fail
        let mixed_patterns = vec!["**/*.rs".to_string(), "**/*[invalid".to_string()];
        assert!(FileFilter::new(mixed_patterns, vec![]).is_err());
    }

    #[test]
    fn test_backward_compatibility() {
        // Old Option<String> interface should work identically to new Vec<String>
        let include_opt = Some("**/*.rs".to_string());
        let exclude_opt = Some("target/**".to_string());

        let filter_old = FileFilter::from_optional(&include_opt, &exclude_opt).unwrap();

        let include_vec = vec!["**/*.rs".to_string()];
        let exclude_vec = vec!["target/**".to_string()];

        let filter_new = FileFilter::new(include_vec, exclude_vec).unwrap();

        let test_paths = [
            "src/main.rs",
            "target/debug/app",
            "src/lib.py",
            "tests/integration.rs",
        ];

        for path in test_paths {
            let path = Path::new(path);
            assert_eq!(
                filter_old.should_include(path),
                filter_new.should_include(path),
                "Filters should behave identically for path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_command_interface_examples() {
        // Examples showing how different command interfaces would use the unified system

        // Example 1: analyze duplicates command (currently Option<String>)
        fn analyze_duplicates_example(
            include: Option<String>,
            exclude: Option<String>,
        ) -> anyhow::Result<FileFilter> {
            FileFilter::from_optional(&include, &exclude)
        }

        let filter1 =
            analyze_duplicates_example(Some("**/*.rs".to_string()), Some("target/**".to_string()))
                .unwrap();

        assert!(filter1.should_include(Path::new("src/main.rs")));
        assert!(!filter1.should_include(Path::new("target/debug/app")));

        // Example 2: analyze complexity command (currently Vec<String>)
        fn analyze_complexity_example(
            include: Vec<String>,
            exclude: Vec<String>,
        ) -> anyhow::Result<FileFilter> {
            FileFilter::new(include, exclude)
        }

        let filter2 = analyze_complexity_example(
            vec!["**/*.rs".to_string(), "**/*.py".to_string()],
            vec!["target/**".to_string(), "node_modules/**".to_string()],
        )
        .unwrap();

        assert!(filter2.should_include(Path::new("src/main.rs")));
        assert!(filter2.should_include(Path::new("script.py")));
        assert!(!filter2.should_include(Path::new("target/debug/app")));
        assert!(!filter2.should_include(Path::new("node_modules/pkg/index.js")));

        // Example 3: Future unified interface (recommended)
        fn unified_command_example(
            include: Vec<String>,
            exclude: Vec<String>,
            use_defaults: bool,
        ) -> anyhow::Result<FileFilter> {
            let mut final_exclude = exclude;
            if use_defaults {
                final_exclude.extend(default_exclude_patterns());
            }
            FileFilter::new(include, final_exclude)
        }

        let filter3 = unified_command_example(
            vec!["**/*.rs".to_string()],
            vec!["benches/**".to_string()],
            true, // use default excludes
        )
        .unwrap();

        assert!(filter3.should_include(Path::new("src/main.rs")));
        assert!(!filter3.should_include(Path::new("benches/benchmark.rs")));
        assert!(!filter3.should_include(Path::new("target/debug/app"))); // default exclude
        assert!(!filter3.should_include(Path::new(".git/config"))); // default exclude
    }
}
