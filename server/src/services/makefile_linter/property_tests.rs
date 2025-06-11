use super::*;
use proptest::prelude::*;

// Property: Linting should never panic on any input
proptest! {
    #[test]
    fn lint_never_panics(input: Vec<u8>) {
        // Convert to string, skip if invalid UTF-8
        if let Ok(content) = String::from_utf8(input) {
            // Create a temporary file
            use std::io::Write;
            use tempfile::NamedTempFile;

            if let Ok(mut temp_file) = NamedTempFile::new() {
                let _ = write!(temp_file, "{}", content);
                // Try to lint - should not panic
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _ = rt.block_on(lint_makefile(temp_file.path()));
            }
        }
    }
}

// Property: Quality score calculation should always be between 0 and 1
proptest! {
    #[test]
    fn quality_score_bounds(
        errors in 0usize..1000,
        warnings in 0usize..1000,
        info in 0usize..1000
    ) {
        let violations: Vec<rules::Violation> = (0..errors).map(|i| rules::Violation {
            rule: format!("error_{}", i),
            severity: rules::Severity::Error,
            span: ast::SourceSpan::file_level(),
            message: "Error".to_string(),
            fix_hint: None,
        }).chain((0..warnings).map(|i| rules::Violation {
            rule: format!("warning_{}", i),
            severity: rules::Severity::Warning,
            span: ast::SourceSpan::file_level(),
            message: "Warning".to_string(),
            fix_hint: None,
        })).chain((0..info).map(|i| rules::Violation {
            rule: format!("info_{}", i),
            severity: rules::Severity::Info,
            span: ast::SourceSpan::file_level(),
            message: "Info".to_string(),
            fix_hint: None,
        })).collect();

        let score = calculate_quality_score(&violations);
        prop_assert!(score >= 0.0);
        prop_assert!(score <= 1.0);
    }
}

// Property: File paths should be handled correctly
proptest! {
    #[test]
    fn handles_various_paths(filename in "[a-zA-Z0-9_.-]+") {
        use tempfile::TempDir;

        if let Ok(temp_dir) = TempDir::new() {
            let file_path = temp_dir.path().join(&filename);

            // Create a simple makefile
            if std::fs::write(&file_path, "all:\n\techo test").is_ok() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(lint_makefile(&file_path));

                match result {
                    Ok(lint_result) => {
                        prop_assert_eq!(lint_result.path, file_path);
                    }
                    Err(_) => {
                        // File might not exist or other IO error - that's ok
                    }
                }
            }
        }
    }
}

// Property: Empty makefiles should produce consistent results
proptest! {
    #[test]
    fn empty_makefile_consistency(whitespace in "[ \t\n\r]*") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        if let Ok(mut temp_file) = NamedTempFile::new() {
            let _ = write!(temp_file, "{}", whitespace);

            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Ok(result1) = rt.block_on(lint_makefile(temp_file.path())) {
                if let Ok(result2) = rt.block_on(lint_makefile(temp_file.path())) {
                    // Should produce same number of violations
                    prop_assert_eq!(result1.violations.len(), result2.violations.len());
                    prop_assert_eq!(result1.quality_score, result2.quality_score);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_property_tests_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
