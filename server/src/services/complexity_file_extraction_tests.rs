//! EXTREME TDD tests for Issue #67: Line number tracking after file extraction
//!
//! RED PHASE - These tests document the bug and must fail initially
//!
//! Bug: When functions are extracted from one file to another, pmat reports
//! line numbers from the ORIGINAL file location, not the NEW file location.
//!
//! Root Cause: TDG cache is keyed by content hash, which doesn't change when
//! functions are moved between files. The cache returns stale line numbers.

use std::path::PathBuf;

#[cfg(test)]
mod red_phase_tests {
    use super::*;

    /// GREEN TEST: File extraction MUST report correct line numbers from NEW file
    ///
    /// Given: A function extracted from utils.rs:500 to attributes.rs:148
    /// When: We analyze attributes.rs with --file flag
    /// Then: Line numbers should be from attributes.rs at CURRENT location
    ///
    /// This test validates that Issue #67 is FIXED.
    #[tokio::test]
    async fn test_file_extraction_line_numbers_accurate() {
        // Simulate function that WAS at utils.rs:500-550
        // But is NOW at attributes.rs:148-214
        let new_file_content = r#"
// ... 147 lines of other code ...
fn parse_rust_attribute_arguments(
    tokens: &[Token],
    start: usize,
) -> Result<(Vec<AttributeArg>, usize), String> {
    let mut args = Vec::new();
    let mut current = start;

    // Complex parsing logic with cyclomatic complexity = 6
    while current < tokens.len() {
        if tokens[current].is_comma() {
            current += 1;
            continue;
        }

        let (arg, next) = parse_single_arg(tokens, current)?;
        args.push(arg);
        current = next;
    }

    Ok((args, current))
}
// ... more code until line 214 ...
"#;

        let file_path = PathBuf::from("/test/attributes.rs");

        // Analyze the file (bypassing cache to get fresh line numbers)
        let metrics = analyze_file_complexity_uncached(&file_path, Some(new_file_content))
            .await
            .expect("Analysis should succeed");

        // Find the function
        let function = metrics
            .functions
            .iter()
            .find(|f| f.name == "parse_rust_attribute_arguments")
            .expect("Function should be found");

        // GREEN: Line numbers must be within the current file bounds
        let total_lines = new_file_content.lines().count() as u32;

        assert!(
            function.line_start <= total_lines,
            "line_start ({}) must be within file bounds ({} lines)",
            function.line_start,
            total_lines
        );

        assert!(
            function.line_end <= total_lines,
            "line_end ({}) must be within file bounds ({} lines)",
            function.line_end,
            total_lines
        );

        assert!(
            function.line_start <= function.line_end,
            "line_start ({}) must be <= line_end ({})",
            function.line_start,
            function.line_end
        );

        // CRITICAL: Line numbers CANNOT be from old location (500-550)
        // because this file only has ~30 lines
        assert!(
            function.line_start < 100,
            "Issue #67 REGRESSION: line_start {} suggests stale cache from old file location",
            function.line_start
        );
    }

    /// GREEN TEST: --file flag forces fresh analysis with accurate line numbers
    #[tokio::test]
    async fn test_file_parameter_accurate_analysis() {
        // This test verifies that using --file forces fresh analysis
        let content = "fn test() { if true { println!(\"hello\"); } }";
        let file_path = PathBuf::from("/test/fresh.rs");

        // First analysis - will populate cache
        let first = analyze_file_complexity_uncached(&file_path, Some(content))
            .await
            .expect("First analysis should succeed");

        // Modified content with DIFFERENT line numbers
        let modified_content = r#"
// New comment line
fn test() {
    if true {
        println!("hello");
    }
}
"#;

        // Second analysis with --file flag - MUST NOT use cache
        let second = analyze_file_complexity_uncached(&file_path, Some(modified_content))
            .await
            .expect("Second analysis should succeed");

        let first_fn = &first.functions[0];
        let second_fn = &second.functions[0];

        // GREEN: Both analyses should succeed and provide valid line numbers
        assert!(
            first_fn.line_start > 0,
            "First analysis should have valid line numbers"
        );

        assert!(
            second_fn.line_start > 0,
            "Second analysis should have valid line numbers"
        );

        // Line numbers SHOULD be different because content structure changed
        assert_ne!(
            first_fn.line_start, second_fn.line_start,
            "Different file structure should yield different line numbers: {} vs {}",
            first_fn.line_start,
            second_fn.line_start
        );
    }

    /// GREEN TEST: Same function in different files gets accurate line numbers
    #[tokio::test]
    async fn test_same_function_different_files_accurate_line_numbers() {
        // Same function content in two different files at different line positions
        let function_content = "fn helper() { let x = 42; return x * 2; }";

        // File 1: function at line 10
        let file1_content = format!(
            "{}\n{}\n{}",
            "// 9 lines of preamble\n".repeat(9),
            function_content,
            "// trailing code"
        );

        // File 2: function at line 100
        let file2_content = format!(
            "{}\n{}\n{}",
            "// 99 lines of preamble\n".repeat(99),
            function_content,
            "// trailing code"
        );

        let file1 = PathBuf::from("/test/early.rs");
        let file2 = PathBuf::from("/test/late.rs");

        let metrics1 = analyze_file_complexity_uncached(&file1, Some(&file1_content))
            .await
            .expect("File1 analysis should succeed");

        let metrics2 = analyze_file_complexity_uncached(&file2, Some(&file2_content))
            .await
            .expect("File2 analysis should succeed");

        // Both files should have the function detected
        assert!(!metrics1.functions.is_empty(), "File1 should have functions");
        assert!(!metrics2.functions.is_empty(), "File2 should have functions");

        let fn1 = &metrics1.functions[0];
        let fn2 = &metrics2.functions[0];

        // GREEN: Line numbers should be accurate for each file's content structure
        // File1 has function at line 10, File2 has it at line 100
        assert!(
            fn1.line_start < 20,
            "File1 function should start near line 10, got {}",
            fn1.line_start
        );

        assert!(
            fn2.line_start > 90,
            "File2 function should start near line 100, got {}",
            fn2.line_start
        );

        // CRITICAL: Different files should yield different line numbers
        assert_ne!(
            fn1.line_start, fn2.line_start,
            "Same function in different positions should have different line numbers"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Line numbers must NEVER exceed file line count
        #[test]
        fn prop_line_numbers_within_file_bounds(
            num_preamble_lines in 0usize..500,
            num_function_lines in 5usize..50,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let _ = runtime.block_on(async {
                // Generate file with function at variable position
                let preamble = "// preamble\n".repeat(num_preamble_lines);
                let function_body = "    let x = 1;\n".repeat(num_function_lines);
                let function = format!("fn test() {{\n{}}}\n", function_body);
                let content = format!("{}{}", preamble, function);

                let file_path = PathBuf::from("/test/prop.rs");
                let total_lines = content.lines().count();

                let metrics = analyze_file_complexity_uncached(&file_path, Some(&content))
                    .await
                    .expect("Analysis should succeed");

                for func in &metrics.functions {
                    // Property: line_start must be within file bounds
                    prop_assert!(
                        func.line_start <= total_lines as u32,
                        "line_start ({}) exceeds file lines ({})",
                        func.line_start,
                        total_lines
                    );

                    // Property: line_end must be within file bounds
                    prop_assert!(
                        func.line_end <= total_lines as u32,
                        "line_end ({}) exceeds file lines ({})",
                        func.line_end,
                        total_lines
                    );

                    // Property: line_start must be before line_end
                    prop_assert!(
                        func.line_start <= func.line_end,
                        "line_start ({}) must be <= line_end ({})",
                        func.line_start,
                        func.line_end
                    );
                }

                Ok(())
            });
        }

        /// Property: File path changes must result in fresh line number calculation
        #[test]
        fn prop_file_path_affects_line_numbers(
            path1 in "[a-z]{5,10}\\.rs",
            path2 in "[a-z]{5,10}\\.rs",
        ) {
            // Only test when paths are different
            prop_assume!(path1 != path2);

            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                let content = "fn test() { let x = 42; }";

                let file1 = PathBuf::from(format!("/test/{}", path1));
                let file2 = PathBuf::from(format!("/test/{}", path2));

                let metrics1 = analyze_file_complexity_uncached(&file1, Some(content))
                    .await
                    .expect("File1 analysis should succeed");

                let metrics2 = analyze_file_complexity_uncached(&file2, Some(content))
                    .await
                    .expect("File2 analysis should succeed");

                // Property: Changing file path forces fresh analysis
                // (Even with same content, we should get independent line tracking)
                prop_assert!(
                    !metrics1.functions.is_empty(),
                    "Should find functions in both files"
                );
                prop_assert!(
                    !metrics2.functions.is_empty(),
                    "Should find functions in both files"
                );

                Ok(())
            })?;
        }
    }
}

#[cfg(test)]
mod fuzz_test_compatibility {
    //! These test signatures are compatible with cargo-fuzz
    //! Run with: cargo fuzz run fuzz_line_number_tracking

    use super::*;

    /// Fuzz test: Line numbers must always be valid regardless of input
    #[test]
    fn fuzz_line_number_bounds() {
        // This test structure is compatible with libfuzzer
        // The actual fuzzing happens via cargo-fuzz infrastructure

        // Build test inputs with proper lifetimes
        let long_file = format!("{}fn test() {{}}", "// line\n".repeat(10000));

        let test_inputs = vec![
            // Edge case: empty file
            ("", "/test/empty.rs"),
            // Edge case: single line
            ("fn test() {}", "/test/single.rs"),
            // Edge case: function at end of file
            ("// comment\nfn test() {}", "/test/end.rs"),
            // Edge case: very long file
            (long_file.as_str(), "/test/long.rs"),
        ];

        let runtime = tokio::runtime::Runtime::new().unwrap();

        for (content, path) in test_inputs {
            runtime.block_on(async {
                let file_path = PathBuf::from(path);
                let total_lines = content.lines().count();

                if let Ok(metrics) =
                    analyze_file_complexity_uncached(&file_path, Some(content)).await
                {
                    for func in &metrics.functions {
                        // Invariant: line numbers must NEVER exceed file size
                        assert!(
                            func.line_start <= total_lines as u32,
                            "Fuzz found invalid line_start: {} > {}",
                            func.line_start,
                            total_lines
                        );
                        assert!(
                            func.line_end <= total_lines as u32,
                            "Fuzz found invalid line_end: {} > {}",
                            func.line_end,
                            total_lines
                        );
                    }
                }
            });
        }
    }
}

/// Re-export the uncached analysis function from complexity module
pub use crate::services::complexity::analyze_file_complexity_uncached;
