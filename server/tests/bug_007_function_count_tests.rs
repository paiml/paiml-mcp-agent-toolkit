//! BUG-007: Function Count Always Zero - RED Phase Tests
//!
//! These tests define expected behavior for function counting in context generation.
//!
//! Current Status: 🔴 RED - These tests will FAIL until implementation complete
//!
//! Test Strategy (Extreme TDD):
//! 1. RED: Write failing tests that define expected behavior
//! 2. GREEN: Implement minimum code to make tests pass
//! 3. REFACTOR: Clean up implementation
//! 4. COMMIT: Single atomic commit with fix

use tempfile::TempDir;

// =============================================================================
// RED TEST 1: Function Count Reflects Actual Functions
// =============================================================================

#[test]
#[ignore = "BUG-007: RED test - will fail until function counting fixed"]
fn test_function_count_reflects_actual_functions() {
    // Arrange: Create Rust file with 3 functions
    let project = create_rust_file_with_functions(3);

    // Act: Generate context
    let context = generate_context_markdown(project.path());

    // Assert: Function count should be 3, not 0
    assert!(context.is_ok(), "Context generation should succeed");
    let output = context.unwrap();

    // Should show "Functions: 3" not "Functions: 0"
    assert!(
        output.contains("Functions: 3") || output.contains("function_count: 3"),
        "Output should show Functions: 3, got: {}",
        extract_function_count_line(&output)
    );
}

// =============================================================================
// RED TEST 2: Function Count Zero When No Functions
// =============================================================================

#[test]
#[ignore = "BUG-007: RED test - empty file should show 0"]
fn test_function_count_zero_when_no_functions() {
    // Arrange: Create Rust file with no functions (only constants)
    let project = create_rust_file_no_functions();

    // Act: Generate context
    let context = generate_context_markdown(project.path());

    // Assert: Function count should correctly be 0
    assert!(context.is_ok(), "Context generation should succeed");
    let output = context.unwrap();

    assert!(
        output.contains("Functions: 0") || output.contains("function_count: 0"),
        "Output should show Functions: 0"
    );
}

// =============================================================================
// RED TEST 3: Function Count Aggregates Per File
// =============================================================================

#[test]
#[ignore = "BUG-007: RED test - per-file counting"]
fn test_function_count_per_file() {
    // Arrange: Create project with multiple files
    let project = create_multi_file_project();

    // Act: Generate context
    let context = generate_context_markdown(project.path());

    // Assert: Each file should show its own function count
    assert!(context.is_ok(), "Context generation should succeed");
    let output = context.unwrap();

    // file1.rs has 2 functions
    assert!(
        output.contains("file1.rs") && count_functions_in_section(&output, "file1.rs") == 2,
        "file1.rs should show 2 functions"
    );

    // file2.rs has 5 functions
    assert!(
        output.contains("file2.rs") && count_functions_in_section(&output, "file2.rs") == 5,
        "file2.rs should show 5 functions"
    );
}

// =============================================================================
// RED TEST 4: Function Count Includes All Function Types
// =============================================================================

#[test]
#[ignore = "BUG-007: RED test - count all function types"]
fn test_function_count_includes_all_types() {
    // Arrange: Create file with different function types
    let project = create_rust_file_with_various_functions();

    // Act: Generate context
    let context = generate_context_markdown(project.path());

    // Assert: Should count impl methods, trait methods, standalone functions
    assert!(context.is_ok(), "Context generation should succeed");
    let output = context.unwrap();

    // Should count: standalone fn, impl method, trait impl method, async fn
    // Total: 4 functions
    assert!(
        output.contains("Functions: 4") || output.contains("function_count: 4"),
        "Output should show Functions: 4 (all types), got: {}",
        extract_function_count_line(&output)
    );
}

// =============================================================================
// RED TEST 5: Function Count Displayed in Summary
// =============================================================================

#[test]
#[ignore = "BUG-007: RED test - summary display"]
fn test_function_count_in_summary() {
    // Arrange: Create simple Rust file
    let project = create_rust_file_with_functions(3);

    // Act: Generate context
    let context = generate_context_markdown(project.path());

    // Assert: Summary section should show function count
    assert!(context.is_ok(), "Context generation should succeed");
    let output = context.unwrap();

    // Look for summary pattern like "File Complexity: X | Functions: Y"
    let has_summary = output.contains("File Complexity:") && output.contains("Functions:");
    assert!(
        has_summary,
        "Output should have file summary with function count"
    );

    // Ensure it's not showing the broken "Functions: 0"
    assert!(
        !output.contains("Functions: 0"),
        "Should not show Functions: 0 when functions exist"
    );
}

// =============================================================================
// Helper Functions (Test Support)
// =============================================================================

fn create_rust_file_with_functions(count: usize) -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let mut code = String::from("// Test file\n\n");

    for i in 0..count {
        code.push_str(&format!(
            "pub fn function_{}() {{\n    println!(\"Hello {}\");\n}}\n\n",
            i, i
        ));
    }

    fs::write(temp.path().join("main.rs"), code).unwrap();
    fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    temp
}

fn create_rust_file_no_functions() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
        // File with no functions
        const VALUE: i32 = 42;
        static NAME: &str = "test";

        struct MyStruct {
            field: i32,
        }
    "#;

    fs::write(temp.path().join("constants.rs"), code).unwrap();
    fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    temp
}

fn create_multi_file_project() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();

    // file1.rs: 2 functions
    let file1 = r#"
        pub fn func1() { }
        pub fn func2() { }
    "#;
    fs::write(temp.path().join("src/file1.rs"), file1).unwrap();

    // file2.rs: 5 functions
    let file2 = r#"
        pub fn func1() { }
        pub fn func2() { }
        pub fn func3() { }
        pub fn func4() { }
        pub fn func5() { }
    "#;
    fs::write(temp.path().join("src/file2.rs"), file2).unwrap();

    fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    temp
}

fn create_rust_file_with_various_functions() -> TempDir {
    use std::fs;

    let temp = TempDir::new().unwrap();
    let code = r#"
        // Standalone function
        pub fn standalone_fn() { }

        struct MyStruct;

        // Impl method
        impl MyStruct {
            pub fn impl_method(&self) { }
        }

        // Trait implementation
        impl std::fmt::Display for MyStruct {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "MyStruct")
            }
        }

        // Async function
        pub async fn async_fn() { }
    "#;

    fs::write(temp.path().join("main.rs"), code).unwrap();
    fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    temp
}

fn generate_context_markdown(path: &std::path::Path) -> Result<String, String> {
    // TODO: Implement in GREEN phase
    // This should call the actual context generation logic
    Err("Not implemented yet".to_string())
}

fn extract_function_count_line(output: &str) -> String {
    output
        .lines()
        .find(|line| line.contains("Functions:"))
        .unwrap_or("(no function count line found)")
        .to_string()
}

fn count_functions_in_section(output: &str, filename: &str) -> usize {
    // Extract the section for this file and count function mentions
    let section = output
        .split(&format!("### {}", filename))
        .nth(1)
        .and_then(|s| s.split("###").next())
        .unwrap_or("");

    section.lines().filter(|line| line.contains("**Function**")).count()
}
