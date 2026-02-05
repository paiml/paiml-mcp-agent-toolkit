#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod red_tests_pmat_bug_002_003_004 {
    use super::*;

    /// RED TEST: PMAT-BUG-002 - JavaScript toolchain must return .js extensions
    ///
    /// This test MUST FAIL before the fix and PASS after the fix.
    ///
    /// Root cause: `get_file_extensions(Some("javascript"))` was hitting the
    /// `Some(_) => vec!["rs"]` catchall case, causing JavaScript projects to
    /// search for .rs files instead of .js files, resulting in 0 files found.
    #[test]
    fn red_test_javascript_toolchain_returns_javascript_extensions() {
        let extensions = get_file_extensions(Some("javascript"));

        assert!(
            extensions.contains(&"js"),
            "PMAT-BUG-002: JavaScript toolchain MUST return .js extension. \
             Got: {:?}. This causes JavaScript projects to return 0 files.",
            extensions
        );

        assert!(
            extensions.contains(&"jsx"),
            "PMAT-BUG-002: JavaScript toolchain MUST return .jsx extension. \
             Got: {:?}",
            extensions
        );
    }

    /// RED TEST: PMAT-BUG-003 - C toolchain must return .c extensions
    ///
    /// This test MUST FAIL before the fix and PASS after the fix.
    ///
    /// Root cause: Same as PMAT-BUG-002 - `get_file_extensions(Some("c"))`
    /// was hitting the catchall case and returning vec!["rs"].
    #[test]
    fn red_test_c_toolchain_returns_c_extensions() {
        let extensions = get_file_extensions(Some("c"));

        assert!(
            extensions.contains(&"c"),
            "PMAT-BUG-003: C toolchain MUST return .c extension. \
             Got: {:?}. This causes C projects to return 0 files.",
            extensions
        );

        assert!(
            extensions.contains(&"h"),
            "PMAT-BUG-003: C toolchain MUST return .h extension for headers. \
             Got: {:?}",
            extensions
        );
    }

    /// RED TEST: PMAT-BUG-004 - C++ toolchain must return .cpp extensions
    ///
    /// This test MUST FAIL before the fix and PASS after the fix.
    ///
    /// Root cause: Same as PMAT-BUG-002 and PMAT-BUG-003.
    #[test]
    fn red_test_cpp_toolchain_returns_cpp_extensions() {
        let extensions = get_file_extensions(Some("cpp"));

        assert!(
            extensions.contains(&"cpp"),
            "PMAT-BUG-004: C++ toolchain MUST return .cpp extension. \
             Got: {:?}. This causes C++ projects to return 0 files.",
            extensions
        );

        // Test C++ variant name
        let extensions_cxx = get_file_extensions(Some("c++"));
        assert!(
            extensions_cxx.contains(&"cpp"),
            "PMAT-BUG-004: C++ toolchain (c++ variant) MUST return .cpp extension. \
             Got: {:?}",
            extensions_cxx
        );
    }

    /// REGRESSION TEST: Existing toolchains must still work correctly
    #[test]
    fn regression_test_existing_toolchains_still_work() {
        // TypeScript should work (it already worked before)
        let ts_exts = get_file_extensions(Some("typescript"));
        assert!(ts_exts.contains(&"ts"));
        assert!(ts_exts.contains(&"tsx"));

        // Rust should work
        let rs_exts = get_file_extensions(Some("rust"));
        assert!(rs_exts.contains(&"rs"));

        // Python should work
        let py_exts = get_file_extensions(Some("python"));
        assert!(py_exts.contains(&"py"));

        // None (multi-language) should include all
        let all_exts = get_file_extensions(None);
        assert!(all_exts.contains(&"rs"));
        assert!(all_exts.contains(&"py"));
        assert!(all_exts.contains(&"js"));
        assert!(all_exts.contains(&"c"));
        assert!(all_exts.contains(&"cpp"));
    }
}
