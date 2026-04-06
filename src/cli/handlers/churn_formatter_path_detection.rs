/// Toyota Way: Extract Method - Check if path is source file (complexity ≤8)
/// Determines if a path represents a source code file
#[must_use]
pub fn is_source_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Check if it has a source code extension
    if !has_source_extension(path) {
        return false;
    }

    // Exclude test and example files by path
    if is_test_path(path) {
        return false;
    }

    // Exclude test files by name pattern
    if is_test_filename(path) {
        return false;
    }

    true
}

/// Toyota Way: Extract Method - Check source extension (complexity ≤3)
fn has_source_extension(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some(
            "rs" | "js"
                | "ts"
                | "py"
                | "java"
                | "cpp"
                | "c"
                | "go"
                | "kt"
                | "swift"
                | "php"
                | "rb"
                | "scala"
        )
    )
}

/// Toyota Way: Extract Method - Check if path contains test directory (complexity ≤5)
fn is_test_path(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let path_str = path.to_string_lossy();
    let test_patterns = [
        "/tests/",
        "/test/",
        "/examples/",
        "/benches/",
        "/fixtures/",
        "/testdata/",
        "/test_data/",
        "/debug_test/",
        "/test-",
        "/__tests__/",
    ];

    test_patterns
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

/// Toyota Way: Extract Method - Check if filename is test file (complexity ≤4)
fn is_test_filename(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|fname| {
            fname.ends_with("_test.rs")
                || fname.ends_with("_tests.rs")
                || fname.starts_with("test_")
                || fname.contains("_test_")
                || fname.ends_with(".test.js")
                || fname.ends_with(".spec.js")
                || fname.ends_with("_test.py")
                || fname.ends_with("Test.java")
        })
}
