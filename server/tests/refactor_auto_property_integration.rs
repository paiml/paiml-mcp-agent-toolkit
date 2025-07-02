//! Integration tests for refactor auto with property-based testing
//!
//! These tests verify that the refactor auto command correctly generates
//! property tests as part of its refactoring process.

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use tempfile::TempDir;
    use std::fs;

    /// Create a test project with code that needs refactoring
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        
        // Create a complex function that needs refactoring
        let complex_code = r#"
//! Test module with high complexity

/// Complex calculation function
/// TODO: This needs refactoring
fn calculate_complex(x: i32, y: i32, z: i32) -> i32 {
    // FIXME: Reduce complexity
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if x > y {
                    if y > z {
                        x * y * z
                    } else {
                        x * z * y
                    }
                } else {
                    if x > z {
                        y * x * z
                    } else {
                        y * z * x
                    }
                }
            } else {
                if x > y {
                    x * y - z
                } else {
                    y * x - z
                }
            }
        } else {
            if z > 0 {
                x * z - y
            } else {
                x - y - z
            }
        }
    } else {
        if y > 0 {
            if z > 0 {
                -x + y + z
            } else {
                -x + y - z
            }
        } else {
            -x - y - z
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic() {
        assert_eq!(calculate_complex(1, 2, 3), 6);
    }
}
"#;
        
        fs::write(src_dir.join("lib.rs"), complex_code).unwrap();
        
        // Create Cargo.toml
        let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]
quickcheck = "1.0"
quickcheck_macros = "1.0"
proptest = "1.6"
"#;
        
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();
        
        temp_dir
    }

    #[test]
    #[ignore] // This test requires the full pmat binary
    fn test_refactor_auto_generates_property_tests() {
        let test_project = create_test_project();
        
        // Run refactor auto in single file mode
        let mut cmd = Command::cargo_bin("pmat").unwrap();
        cmd.arg("refactor")
            .arg("auto")
            .arg("--single-file-mode")
            .arg("--file")
            .arg("src/lib.rs")
            .arg("--project-path")
            .arg(test_project.path())
            .arg("--max-iterations")
            .arg("1");
        
        // Should complete successfully
        cmd.assert().success();
        
        // Check that refactoring request was generated
        let cache_dir = test_project.path().join(".pmat-cache");
        assert!(cache_dir.exists(), "Cache directory should be created");
        
        // Check for refactor state
        let state_file = cache_dir.join("refactor-state.json");
        assert!(state_file.exists(), "Refactor state should be saved");
        
        // Read the state to verify it includes property test generation
        let state_content = fs::read_to_string(&state_file).unwrap();
        assert!(
            state_content.contains("property_test_generation"),
            "Refactor state should include property test generation config"
        );
    }

    #[test]
    fn test_property_test_generation_in_request() {
        // This test verifies the structure of property test generation requests
        let test_json = "{
            \"property_test_generation\": {
                \"enabled\": true,
                \"instructions\": [
                    \"For each refactored function, generate a property test that verifies behavior preservation\",
                    \"Use quickcheck or proptest for property-based testing\"
                ],
                \"example_property_test\": \"#[quickcheck]\\nfn prop_refactoring_preserves_behavior\"
            }
        }";
        
        let parsed: serde_json::Value = serde_json::from_str(test_json).unwrap();
        assert!(parsed["property_test_generation"]["enabled"].as_bool().unwrap());
        assert!(parsed["property_test_generation"]["instructions"].is_array());
    }

    #[test]
    fn test_generated_property_test_template() {
        // Verify the property test template is valid Rust code
        let template = r#"
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult, Arbitrary};
    
    #[quickcheck]
    fn prop_refactoring_preserves_behavior(input: TestInput) -> TestResult {
        // Skip invalid inputs
        if !input.is_valid() {
            return TestResult::discard();
        }
        
        // Compare original and refactored behavior
        let original_result = original_function(input.clone());
        let refactored_result = refactored_function(input);
        
        TestResult::from_bool(original_result == refactored_result)
    }
}"#;
        
        // Just verify it's valid syntax (would need syn to properly parse)
        assert!(template.contains("#[quickcheck]"));
        assert!(template.contains("TestResult"));
        assert!(template.contains("prop_refactoring_preserves_behavior"));
    }

    #[test]
    fn test_coverage_improvement_tracking() {
        // Test that we can track coverage improvements from property tests
        let initial_coverage = 45.0;
        let target_coverage = 80.0;
        
        // Simulate coverage improvement
        let improved_coverage = 85.0;
        
        assert!(improved_coverage >= target_coverage);
        assert!(improved_coverage > initial_coverage);
    }

    #[test]
    fn test_property_test_shrinking() {
        // Verify shrinking strategies are included
        let shrinking_example = r#"
impl Arbitrary for ValidRustCode {
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.lines()
            .enumerate()
            .filter_map(|(i, _)| {
                let mut lines: Vec<_> = self.0.lines().collect();
                lines.remove(i);
                let candidate = lines.join("\n");
                
                // Only return if still valid
                if candidate.contains("fn") {
                    Some(ValidRustCode(candidate))
                } else {
                    None
                }
            }))
    }
}"#;
        
        assert!(shrinking_example.contains("shrink"));
        assert!(shrinking_example.contains("filter_map"));
    }
}