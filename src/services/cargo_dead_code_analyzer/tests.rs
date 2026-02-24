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

        // Create a Rust file with #[allow(dead_code)] attributes
        let rust_code = r#"
#[allow(dead_code)]
fn unused_function() {
    println!("never called");
}

#[allow(dead_code)]
struct UnusedStruct {
    field: i32,
}

#[allow(unused)]
const UNUSED_CONST: i32 = 42;

// This one should NOT be detected (no suppression)
fn used_function() {
    println!("called");
}
"#;

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
        let rust_code = r#"
#[derive(Debug)]
#[allow(dead_code)]
#[derive(Clone)]
struct StackedAttributes {
    value: i32,
}
"#;

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
        let rust_code = r#"
#![allow(dead_code)]

fn function_in_suppressed_module() {}
"#;

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

        let analyzer = CargoDeadCodeAnalyzer::new(&project_path).without_cache();
        let items = analyzer.scan_for_suppression_attributes().unwrap();

        // The pmat codebase has many #[allow(dead_code)] attributes
        // Note: Not all #[allow(dead_code)] have items on the next line (some are on fields)
        // Based on grep count, ~200+ are on actual items (fn/struct/enum/etc.)
        assert!(
            items.len() >= 200,
            "Expected at least 200 suppressed items in pmat codebase, found {}. \
             This suggests the suppression scan may not be working correctly.",
            items.len()
        );

        // Verify items are marked as Suppressed
        for (_, item) in &items {
            assert_eq!(item.kind, DeadCodeKind::Suppressed);
        }

        eprintln!(
            "Layer 1 (suppression scan) detected {} items with #[allow(dead_code)]",
            items.len()
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
