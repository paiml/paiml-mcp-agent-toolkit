// Tests for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_message() {
        let analyzer = CargoDeadCodeAnalyzer::new(".");
        let (name, kind) = analyzer
            .parse_message("function `unused_func` is never used")
            .unwrap();
        assert_eq!(name, "unused_func");
        assert_eq!(kind, DeadCodeKind::Function);
    }

    #[test]
    fn test_parse_struct_message() {
        let analyzer = CargoDeadCodeAnalyzer::new(".");
        let (name, kind) = analyzer
            .parse_message("struct `UnusedStruct` is never constructed")
            .unwrap();
        assert_eq!(name, "UnusedStruct");
        assert_eq!(kind, DeadCodeKind::Struct);
    }

    #[test]
    fn test_parse_field_message() {
        let analyzer = CargoDeadCodeAnalyzer::new(".");
        let (name, kind) = analyzer
            .parse_message("field `data` is never read")
            .unwrap();
        assert_eq!(name, "data");
        assert_eq!(kind, DeadCodeKind::Field);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod suppression_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_suppression_scan_detects_allow_dead_code() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create a Rust file with dead_code suppression attributes
        let adc = format!("#[allow({})]", "dead_code");
        let rust_code = format!(
            r#"
{adc}
fn unused_function() {{
    println!("never called");
}}

{adc}
struct UnusedStruct {{
    field: i32,
}}

#[allow(unused)]
const UNUSED_CONST: i32 = 42;

// This one should NOT be detected (no suppression)
fn used_function() {{
    println!("called");
}}
"#
        );

        fs::write(src_dir.join("lib.rs"), rust_code).unwrap();

        let analyzer = CargoDeadCodeAnalyzer::new(temp_dir.path()).without_cache();
        let items = analyzer.scan_for_suppression_attributes().unwrap();

        // Should detect 3 suppressed items
        assert_eq!(
            items.len(),
            3,
            "Expected 3 suppressed items, found {}",
            items.len()
        );

        // Verify the items are marked as Suppressed
        for (_, item) in &items {
            assert_eq!(item.kind, DeadCodeKind::Suppressed);
        }

        // Check specific names
        let names: Vec<&str> = items.iter().map(|(_, i)| i.name.as_str()).collect();
        assert!(
            names.contains(&"unused_function"),
            "Should detect unused_function"
        );
        assert!(
            names.contains(&"UnusedStruct"),
            "Should detect UnusedStruct"
        );
        assert!(
            names.contains(&"UNUSED_CONST"),
            "Should detect UNUSED_CONST"
        );
    }

    #[test]
    fn test_suppression_scan_handles_nested_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Test with multiple stacked attributes
        let adc = format!("#[allow({})]", "dead_code");
        let rust_code = format!(
            r#"
#[derive(Debug)]
{adc}
#[derive(Clone)]
struct StackedAttributes {{
    value: i32,
}}
"#
        );

        fs::write(src_dir.join("lib.rs"), rust_code).unwrap();

        let analyzer = CargoDeadCodeAnalyzer::new(temp_dir.path()).without_cache();
        let items = analyzer.scan_for_suppression_attributes().unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].1.name, "StackedAttributes");
    }

    #[test]
    fn test_suppression_scan_module_level() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Module-level suppression (inner attribute)
        let adc_inner = format!("#![allow({})]", "dead_code");
        let rust_code = format!(
            "\n{adc_inner}\n\nfn function_in_suppressed_module() {{}}\n"
        );

        fs::write(src_dir.join("lib.rs"), rust_code).unwrap();

        let analyzer = CargoDeadCodeAnalyzer::new(temp_dir.path()).without_cache();
        let items = analyzer.scan_for_suppression_attributes().unwrap();

        // Inner attribute should also trigger detection
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].1.name, "function_in_suppressed_module");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that suppression scan works on the actual pmat codebase.
    /// This is an important dogfooding test per CB-128 spec.
    #[test]
    fn test_suppression_scan_on_pmat_codebase() {
        // Get the actual project path (not target directory)
        // Use CARGO_MANIFEST_DIR which is set during compilation
        let project_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Skip if we're not in the pmat project directory
        if !project_path.join("Cargo.toml").exists() {
            eprintln!("Skipping pmat integration test - not in project root");
            return;
        }

        // This used to assert `items.len() >= 20` for the DEFAULT analyzer,
        // which excludes tests. Of the ~47 item-matched suppression attributes
        // in this repo, all but one live in `tests/` or in `*_tests.rs`, so the
        // only way to reach 20 by default was for Layer 1 to ignore
        // `exclude_tests` entirely — exactly the defect that made
        // `--include-tests` inert. The dogfooding intent is kept, but the count
        // is now asserted where the items actually are, and the default scan is
        // checked for the property that matters: nothing from a test tree.
        let default_items = CargoDeadCodeAnalyzer::new(&project_path)
            .without_cache()
            .scan_for_suppression_attributes()
            .unwrap();

        for (path, item) in &default_items {
            assert_eq!(item.kind, DeadCodeKind::Suppressed);
            let relative = path.strip_prefix(&project_path).unwrap_or(path);
            assert!(
                !relative
                    .components()
                    .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "examples"),
                "{} is out of scope without --include-tests/--include-examples",
                relative.display()
            );
            assert!(
                !is_test_file_name(relative),
                "{} is a test module and is out of scope without --include-tests",
                relative.display()
            );
        }

        // With the test trees in scope the pmat codebase has many suppression
        // attributes (#[allow(unused)] etc.).
        // Note: Not all suppression attrs have items on the next line (some are on fields)
        let with_tests = CargoDeadCodeAnalyzer::new(&project_path)
            .without_cache()
            .include_tests()
            .scan_for_suppression_attributes()
            .unwrap();

        assert!(
            with_tests.len() >= 20,
            "Expected at least 20 suppressed items in the pmat codebase with \
             --include-tests, found {}. This suggests the suppression scan may \
             not be working correctly.",
            with_tests.len()
        );
        assert!(
            with_tests.len() > default_items.len(),
            "--include-tests must widen the scan: {} with tests vs {} without",
            with_tests.len(),
            default_items.len()
        );

        // Verify items are marked as Suppressed
        for (_, item) in &with_tests {
            assert_eq!(item.kind, DeadCodeKind::Suppressed);
        }

        eprintln!(
            "Layer 1 (suppression scan) detected {} suppressed items ({} with --include-tests)",
            default_items.len(),
            with_tests.len()
        );
    }
}

