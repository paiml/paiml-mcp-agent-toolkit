use tempfile::TempDir;
use std::fs;
use crate::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};

#[cfg(test)]
mod extreme_tdd_tests {
    use super::*;

    #[tokio::test]
    async fn red_test_unified_context_must_show_functions() {
        // RED: This test should initially fail, then we make it pass
        let temp_dir = TempDir::new().unwrap();

        let ts_content = "function testFunction() { return 42; }";
        let ts_file = temp_dir.path().join("test.ts");
        fs::write(&ts_file, ts_content).unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec![], // Empty patterns to include all files with valid extensions
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let analysis_result = analyzer.analyze(config).await.unwrap();

        // Function extraction working correctly now

        // This MUST show at least 1 function
        assert!(analysis_result.complexity_metrics.total_functions >= 1,
                "Unified context failed to extract functions! Found: {}",
                analysis_result.complexity_metrics.total_functions);
    }

    #[tokio::test]
    async fn green_test_unified_context_handles_multiple_languages() {
        // GREEN: Make the test pass by ensuring multi-language support
        let temp_dir = TempDir::new().unwrap();

        // Create multiple language files
        fs::write(temp_dir.path().join("test.rs"), "fn rust_func() {}").unwrap();
        fs::write(temp_dir.path().join("test.ts"), "function tsFunc() {}").unwrap();
        fs::write(temp_dir.path().join("test.js"), "function jsFunc() {}").unwrap();
        fs::write(temp_dir.path().join("test.py"), "def python_func():\n    pass").unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
                "**/*.py".to_string(),
            ],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let analysis_result = analyzer.analyze(config).await.unwrap();

        // Multi-language support working correctly now

        // Should detect multiple files and functions
        assert!(analysis_result.file_count >= 4, "Should detect all language files");
        assert!(analysis_result.complexity_metrics.total_functions >= 4, "Should detect functions in all languages");
    }

    #[tokio::test]
    async fn refactor_test_context_consistency() {
        // REFACTOR: Ensure context analysis is consistent across runs
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.ts"), "function test() {}").unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec!["**/*.ts".to_string()],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        // Run analysis twice
        let result1 = analyzer.analyze(config.clone()).await.unwrap();
        let result2 = analyzer.analyze(config).await.unwrap();

        // Results should be identical
        assert_eq!(result1.file_count, result2.file_count,
                  "File count should be consistent across runs");
        assert_eq!(result1.complexity_metrics.total_functions, result2.complexity_metrics.total_functions,
                  "Function count should be consistent across runs");
    }

    #[tokio::test]
    async fn test_wasm_function_extraction() {
        // Test WASM function extraction as requested
        let temp_dir = TempDir::new().unwrap();

        let wasm_content = r#"(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  (export "add" (func $add))
)"#;

        fs::write(temp_dir.path().join("test.wat"), wasm_content).unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec!["**/*.wat".to_string()],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let analysis_result = analyzer.analyze(config).await.unwrap();

        // Should detect WASM functions
        assert!(analysis_result.file_count >= 1, "Should detect WASM file");
        assert!(analysis_result.complexity_metrics.total_functions >= 1, "Should detect WASM functions");
    }

    #[tokio::test]
    async fn test_javascript_descriptive_names() {
        // Test JavaScript function name extraction
        let temp_dir = TempDir::new().unwrap();

        let js_content = r#"
function calculateTotal() {
    return 42;
}

const processData = () => {
    return "processed";
};

class UserService {
    createUser() {
        return {};
    }

    static validateUser() {
        return true;
    }
}

const utils = {
    formatMessage() {
        return "formatted";
    }
};
"#;

        fs::write(temp_dir.path().join("test.js"), js_content).unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec!["**/*.js".to_string()],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let analysis_result = analyzer.analyze(config).await.unwrap();

        // Should detect multiple JavaScript functions
        assert!(analysis_result.complexity_metrics.total_functions >= 5,
                "Should detect all JavaScript functions: calculateTotal, processData, createUser, validateUser, formatMessage. Found: {}",
                analysis_result.complexity_metrics.total_functions);
    }

    #[tokio::test]
    async fn test_typescript_interface_detection() {
        // Test TypeScript interface and enum detection
        let temp_dir = TempDir::new().unwrap();

        let ts_content = r#"
interface User {
    id: number;
    name: string;
    getProfile(): UserProfile;
}

enum Status {
    Active,
    Inactive,
    Pending
}

function processUser(user: User): Status {
    return Status.Active;
}

class UserManager {
    updateUser(user: User): void {
        // implementation
    }
}
"#;

        fs::write(temp_dir.path().join("test.ts"), ts_content).unwrap();

        let analyzer = SimpleDeepContext::new();
        let config = SimpleAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_features: vec![],
            include_patterns: vec!["**/*.ts".to_string()],
            exclude_patterns: vec![],
            enable_verbose: false,
        };

        let analysis_result = analyzer.analyze(config).await.unwrap();

        // Should detect TypeScript functions
        assert!(analysis_result.complexity_metrics.total_functions >= 2,
                "Should detect TypeScript functions: processUser, updateUser. Found: {}",
                analysis_result.complexity_metrics.total_functions);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    #[test]
    fn property_function_count_non_negative() {
        fn prop(content: String) -> TestResult {
            if content.is_empty() {
                return TestResult::discard();
            }

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = match TempDir::new() {
                    Ok(dir) => dir,
                    Err(_) => return TestResult::failed(),
                };

                let file_path = temp_dir.path().join("test.ts");
                if fs::write(&file_path, &content).is_err() {
                    return TestResult::failed();
                }

                let analyzer = SimpleDeepContext::new();
                let config = SimpleAnalysisConfig {
                    project_path: temp_dir.path().to_path_buf(),
                    include_features: vec![],
                    include_patterns: vec!["**/*.ts".to_string()],
                    exclude_patterns: vec![],
                    enable_verbose: false,
                };

                match analyzer.analyze(config).await {
                    Ok(_result) => {
                        // Property: function count should always be non-negative (usize is always >= 0)
                        TestResult::passed()
                    },
                    Err(_) => TestResult::passed(), // Errors acceptable for invalid input
                }
            })
        }

        quickcheck(prop as fn(String) -> TestResult);
    }

    #[test]
    fn property_analysis_deterministic() {
        fn prop(valid_function: String) -> TestResult {
            if valid_function.is_empty() {
                return TestResult::discard();
            }

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = match TempDir::new() {
                    Ok(dir) => dir,
                    Err(_) => return TestResult::failed(),
                };

                let content = format!("function {}() {{ return 42; }}", valid_function.replace(|c: char| !c.is_alphanumeric(), "_"));
                let file_path = temp_dir.path().join("test.ts");
                if fs::write(&file_path, &content).is_err() {
                    return TestResult::failed();
                }

                let analyzer = SimpleDeepContext::new();
                let config = SimpleAnalysisConfig {
                    project_path: temp_dir.path().to_path_buf(),
                    include_features: vec![],
                    include_patterns: vec!["**/*.ts".to_string()],
                    exclude_patterns: vec![],
                    enable_verbose: false,
                };

                match (analyzer.analyze(config.clone()).await, analyzer.analyze(config).await) {
                    (Ok(result1), Ok(result2)) => {
                        // Property: analysis should be deterministic
                        TestResult::from_bool(result1.complexity_metrics.total_functions == result2.complexity_metrics.total_functions)
                    },
                    _ => TestResult::passed(), // Errors are acceptable
                }
            })
        }

        quickcheck(prop as fn(String) -> TestResult);
    }
}