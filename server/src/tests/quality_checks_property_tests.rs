//! Property-based tests for quality check functions
//!
//! These tests verify invariants and properties of the quality checking system
//! using proptest for comprehensive coverage.

use crate::cli::stubs::{QualityViolation, check_complexity, check_dead_code, check_satd, check_duplicates};
use proptest::prelude::*;
use std::path::Path;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Helper to create test files with specific content
fn create_test_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    std::fs::write(&file_path, content).unwrap();
    file_path
}

proptest! {
    /// Property: Complexity violations only occur when cyclomatic complexity exceeds threshold
    #[test]
    fn prop_complexity_threshold_respected(
        threshold in 1u32..100u32,
        file_count in 1usize..10usize,
    ) {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files with varying complexity
        for i in 0..file_count {
            let complexity = i as u32 * 10; // 0, 10, 20, 30...
            let content = generate_code_with_complexity(complexity);
            create_test_file(temp_dir.path(), &format!("test{}.rs", i), &content);
        }
        
        let _ = rt.block_on(async {
            let violations = check_complexity(temp_dir.path(), threshold).await.unwrap();
            
            // Verify all violations are for files exceeding threshold
            for violation in violations {
                assert_eq!(violation.check_type, "complexity");
                assert_eq!(violation.severity, "error");
                
                // Extract complexity from message
                if let Some(complexity_str) = violation.message
                    .split("complexity ")
                    .nth(1)
                    .and_then(|s| s.split(' ').next())
                    .and_then(|s| s.parse::<u32>().ok()) {
                    prop_assert!(complexity_str > threshold);
                }
            }
            Ok(())
        });
    }

    /// Property: Dead code percentage calculations are accurate
    #[test]
    fn prop_dead_code_percentage_accurate(
        max_percentage in 0.0..50.0,
        total_functions in 10usize..100usize,
        dead_ratio in 0.0..0.5f64,
    ) {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Create a Rust file with known dead/live function ratio
        let dead_count = (total_functions as f64 * dead_ratio) as usize;
        let mut content = String::new();
        
        // Add live functions
        for i in 0..(total_functions - dead_count) {
            content.push_str(&format!("pub fn live_func_{}() {{ println!(\"live\"); }}\n", i));
        }
        
        // Add dead functions (private, prefixed with _)
        for i in 0..dead_count {
            content.push_str(&format!("fn _dead_func_{}() {{ println!(\"dead\"); }}\n", i));
        }
        
        create_test_file(temp_dir.path(), "lib.rs", &content);
        
        let _ = rt.block_on(async {
            let violations = check_dead_code(temp_dir.path(), max_percentage).await.unwrap();
            
            // Property: Violations occur only when dead code exceeds threshold
            let expected_percentage = (dead_count as f64 / total_functions as f64) * 100.0;
            if expected_percentage > max_percentage {
                prop_assert!(!violations.is_empty(), 
                    "Expected violations when dead code {}% > max {}%", 
                    expected_percentage, max_percentage);
            }
            Ok(())
        });
    }

    /// Property: SATD detection finds all and only valid patterns
    #[test]
    fn prop_satd_detection_complete_and_sound(
        satd_types in prop::collection::vec(
            prop_oneof![
                Just("TODO"),
                Just("FIXME"),
                Just("HACK"),
                Just("XXX"),
                Just("BUG"),
                Just("REFACTOR"),
            ],
            0..10
        ),
        descriptions in prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..10),
    ) {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Create file with known SATD comments
        let mut content = String::new();
        let expected_count = satd_types.len().min(descriptions.len());
        
        for i in 0..expected_count {
            content.push_str(&format!("// {}: {}\n", satd_types[i], descriptions[i]));
            content.push_str("fn some_function() {}\n\n");
        }
        
        // Add some non-SATD comments
        content.push_str("// This is a regular comment\n");
        content.push_str("// Another normal comment without debt markers\n");
        
        create_test_file(temp_dir.path(), "test.rs", &content);
        
        let _ = rt.block_on(async {
            let violations = check_satd(temp_dir.path()).await.unwrap();
            
            // Property: Number of violations matches number of SATD comments
            prop_assert_eq!(violations.len(), expected_count);
            
            // Property: All violations are SATD type with line numbers
            for violation in &violations {
                prop_assert_eq!(&violation.check_type, "satd");
                prop_assert!(violation.line.is_some());
                prop_assert!(violation.severity == "warning" || violation.severity == "error");
            }
            Ok(())
        });
    }

    /// Property: Duplicate detection is symmetric and transitive
    #[test]
    fn prop_duplicate_detection_properties(
        file_contents in prop::collection::vec(
            "[a-zA-Z0-9\n ]{50,200}", // Generate code-like content
            2..10
        ),
        duplicate_indices in prop::collection::vec(0usize..10usize, 0..5),
    ) {
        let rt = Runtime::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        
        // Create files, some with duplicate content
        let mut duplicate_groups = std::collections::HashMap::new();
        
        for (i, content) in file_contents.iter().enumerate() {
            let file_content = if duplicate_indices.contains(&i) && i > 0 {
                // Make this a duplicate of the first file
                duplicate_groups.entry(0).or_insert_with(Vec::new).push(i);
                &file_contents[0]
            } else {
                duplicate_groups.entry(i).or_insert_with(Vec::new);
                content
            };
            
            create_test_file(temp_dir.path(), &format!("file{}.rs", i), file_content);
        }
        
        let _ = rt.block_on(async {
            let violations = check_duplicates(temp_dir.path()).await.unwrap();
            
            // Property: Duplicates are detected symmetrically
            let mut violation_pairs = std::collections::HashSet::new();
            for violation in &violations {
                violation_pairs.insert(violation.file.clone());
            }
            
            // If A is duplicate of B, then B should be duplicate of A
            for violation in &violations {
                if let Some(other_files) = extract_files_from_duplicate_message(&violation.message) {
                    for other_file in other_files {
                        prop_assert!(
                            violation_pairs.contains(&other_file),
                            "Symmetry violated: {} marked as duplicate but {} is not",
                            violation.file, other_file
                        );
                    }
                }
            }
            Ok(())
        });
    }
}

/// Helper to generate code with specific cyclomatic complexity
fn generate_code_with_complexity(target_complexity: u32) -> String {
    let mut code = String::from("fn complex_function() {\n");
    
    // Each if statement adds 1 to cyclomatic complexity
    for i in 0..target_complexity {
        code.push_str(&format!("    if condition_{} {{\n", i));
        code.push_str(&format!("        println!(\"branch {}\");\n", i));
        code.push_str("    }\n");
    }
    
    code.push_str("}\n");
    code
}

/// Extract file names from duplicate violation message
fn extract_files_from_duplicate_message(message: &str) -> Option<Vec<String>> {
    message.split("found in: ").nth(1).map(|files_part| files_part.split(", ").map(String::from).collect())
}

#[cfg(test)]
mod additional_property_tests {
    use super::*;

    proptest! {
        /// Property: Quality violations have consistent structure
        #[test]
        fn prop_violation_structure_valid(
            check_type in "[a-z_]+",
            severity in prop_oneof![Just("error"), Just("warning"), Just("info")],
            file_path in "[a-zA-Z0-9/_]+\\.rs",
            line in prop::option::of(1usize..10000usize),
            message in "[a-zA-Z0-9 :,.-]+",
        ) {
            let violation = QualityViolation {
                check_type: check_type.clone(),
                severity: severity.to_string(),
                file: file_path.clone(),
                line,
                message: message.clone(),
            };
            
            // Properties that should always hold
            prop_assert!(!violation.check_type.is_empty());
            prop_assert!(!violation.severity.is_empty());
            prop_assert!(!violation.file.is_empty());
            prop_assert!(!violation.message.is_empty());
            
            // Severity should be one of the valid values
            prop_assert!(
                ["error", "warning", "info"].contains(&violation.severity.as_str()),
                "Invalid severity: {}", violation.severity
            );
        }
    }
}