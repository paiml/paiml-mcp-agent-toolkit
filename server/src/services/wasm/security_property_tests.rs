#[cfg(test)]
mod tests {
    use super::super::security::*;
    use super::super::types::Severity;
    use proptest::prelude::*;


    proptest! {
        fn validator_never_panics_on_arbitrary_input(
            data in prop::collection::vec(any::<u8>(), 0..10000)
        ) {
            let validator = WasmSecurityValidator::new();
            
            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                validator.validate(&data)
            }));
            
            prop_assert!(result.is_ok());
        }

        fn validation_result_is_consistent(
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let validator = WasmSecurityValidator::new();
            
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            if let Ok(validation) = result {
                // If there are critical issues, validation should fail
                let has_critical = validation.issues.iter()
                    .any(|issue| issue.severity == Severity::Critical);
                
                if has_critical {
                    prop_assert!(!validation.passed);
                }
                
                // passed should be false iff there are issues
                prop_assert_eq!(validation.passed, validation.issues.is_empty());
            }
        }

        fn small_files_detected(
            size in 0usize..8
        ) {
            let data = vec![0u8; size];
            let validator = WasmSecurityValidator::new();
            
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            let validation = result.unwrap();
            prop_assert!(!validation.passed);
            
            // Should have invalid format issue
            let has_size_issue = validation.issues.iter()
                .any(|issue| issue.category == SecurityCategory::InvalidFormat &&
                            issue.description.contains("too small"));
            prop_assert!(has_size_issue);
        }

        fn invalid_magic_detected(
            magic in prop::collection::vec(any::<u8>().prop_filter("not wasm magic", |&b| b != 0x00), 4..4),
            rest in prop::collection::vec(any::<u8>(), 4..100)
        ) {
            let mut data = magic;
            data.extend(rest);
            
            let validator = WasmSecurityValidator::new();
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            let validation = result.unwrap();
            prop_assert!(!validation.passed);
            
            // Should have invalid magic issue
            let has_magic_issue = validation.issues.iter()
                .any(|issue| issue.category == SecurityCategory::InvalidFormat &&
                            issue.description.contains("magic number"));
            prop_assert!(has_magic_issue);
        }

        fn large_files_flagged(
            size_mb in 101usize..200
        ) {
            // Create a large valid WASM file
            let mut data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
            data.resize(size_mb * 1024 * 1024, 0);
            
            let validator = WasmSecurityValidator::new();
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            let validation = result.unwrap();
            
            // Should have resource exhaustion issue
            let has_size_issue = validation.issues.iter()
                .any(|issue| issue.category == SecurityCategory::ResourceExhaustion &&
                            issue.severity == Severity::High);
            prop_assert!(has_size_issue);
        }

        fn valid_wasm_passes_basic_checks(
            data_size in 0usize..1000
        ) {
            // Create valid WASM
            let mut data = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
            data.extend(vec![0u8; data_size]);
            
            let validator = WasmSecurityValidator::new();
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            let validation = result.unwrap();
            prop_assert!(validation.passed);
            prop_assert!(validation.issues.is_empty());
        }

        fn severity_ordering_respected(
            data in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let validator = WasmSecurityValidator::new();
            let result = validator.validate(&data);
            prop_assert!(result.is_ok());
            
            if let Ok(validation) = result {
                // Check that severity levels make sense
                for issue in &validation.issues {
                    match issue.category {
                        SecurityCategory::InvalidFormat => {
                            // Format issues should be critical
                            prop_assert!(issue.severity == Severity::Critical || 
                                       issue.severity == Severity::High);
                        }
                        SecurityCategory::ResourceExhaustion => {
                            // Resource issues should be high or medium
                            prop_assert!(issue.severity == Severity::High || 
                                       issue.severity == Severity::Medium);
                        }
                        _ => {
                            // Other issues can be any severity
                        }
                    }
                }
            }
        }

        fn validate_text_never_panics(
            text in ".*"
        ) {
            let validator = WasmSecurityValidator::new();
            
            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                validator.validate_text(&text)
            }));
            
            prop_assert!(result.is_ok());
        }

        fn ast_validation_never_panics(
            node_count in 0usize..100
        ) {
            use crate::models::unified_ast::{AstDag, UnifiedAstNode, AstKind, Language, FunctionKind};
            
            let validator = WasmSecurityValidator::new();
            let mut dag = AstDag::new();
            
            // Add some nodes
            for _i in 0..node_count {
                let node = UnifiedAstNode::new(
                    AstKind::Function(FunctionKind::Regular),
                    Language::WebAssembly,
                );
                dag.add_node(node);
            }
            
            // Should not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                validator.validate_ast(&dag)
            }));
            
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn edge_case_file_sizes() {
        let validator = WasmSecurityValidator::new();
        
        // Exactly 8 bytes (minimum valid)
        let min_valid = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        let result = validator.validate(&min_valid).unwrap();
        assert!(result.passed);
        
        // 7 bytes (too small)
        let too_small = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00];
        let result = validator.validate(&too_small).unwrap();
        assert!(!result.passed);
        
        // Exactly 100MB (at limit)
        let mut at_limit = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        at_limit.resize(100 * 1024 * 1024, 0);
        let result = validator.validate(&at_limit).unwrap();
        assert!(result.passed);
        
        // Just over 100MB
        let mut over_limit = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        over_limit.resize(100 * 1024 * 1024 + 1, 0);
        let result = validator.validate(&over_limit).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn multiple_issues_reported() {
        let validator = WasmSecurityValidator::new();
        
        // Small file with invalid magic - should have 2 issues
        let data = vec![0xFF, 0xFF, 0xFF];
        let result = validator.validate(&data).unwrap();
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
        
        // Large file with invalid magic
        let large_invalid = vec![0xFF; 200 * 1024 * 1024];
        let result = validator.validate(&large_invalid).unwrap();
        assert!(!result.passed);
        assert!(result.issues.len() >= 2); // Both magic and size issues
    }
}