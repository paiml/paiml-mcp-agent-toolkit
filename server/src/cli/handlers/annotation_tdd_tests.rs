//! Extreme TDD Tests for Missing Annotations in Unified Context
//!
//! Following RED-GREEN-REFACTOR cycle for each missing annotation type

use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod red_phase_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_individual_function_names() {
        // RED: This test MUST fail initially, proving we're missing function names
        let temp_dir = TempDir::new().unwrap();

        let ts_content = r#"
function calculateTotal() { return 42; }
function processData() { return "data"; }
const validateInput = () => { return true; };
"#;

        fs::write(temp_dir.path().join("test.ts"), ts_content).unwrap();

        // Run pmat context and capture output
        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // These assertions MUST FAIL in RED phase
        assert!(stdout.contains("calculateTotal"),
            "Missing function name 'calculateTotal' in output");
        assert!(stdout.contains("processData"),
            "Missing function name 'processData' in output");
        assert!(stdout.contains("validateInput"),
            "Missing function name 'validateInput' in output");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_file_level_breakdown() {
        // RED: Must show which functions belong to which files
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("auth.ts"),
            "function login() {} function logout() {}").unwrap();
        fs::write(temp_dir.path().join("utils.ts"),
            "function formatDate() {} function parseJSON() {}").unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show file grouping
        assert!(stdout.contains("File: auth.ts") || stdout.contains("auth.ts"),
            "Missing file-level grouping for auth.ts");
        assert!(stdout.contains("File: utils.ts") || stdout.contains("utils.ts"),
            "Missing file-level grouping for utils.ts");

        // Must show functions under their files
        let auth_index = stdout.find("auth.ts").unwrap_or(0);
        let utils_index = stdout.find("utils.ts").unwrap_or(0);
        let login_index = stdout.find("login").unwrap_or(0);
        let format_index = stdout.find("formatDate").unwrap_or(0);

        assert!(login_index > auth_index && login_index < utils_index,
            "Function 'login' not properly grouped under auth.ts");
        assert!(format_index > utils_index,
            "Function 'formatDate' not properly grouped under utils.ts");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_complexity_scores() {
        // RED: Must show complexity metrics for functions
        let temp_dir = TempDir::new().unwrap();

        let complex_function = r#"
function complexLogic(input) {
    if (input > 10) {
        if (input > 20) {
            for (let i = 0; i < input; i++) {
                if (i % 2 === 0) {
                    console.log(i);
                }
            }
        }
    } else {
        switch(input) {
            case 1: return "one";
            case 2: return "two";
            default: return "other";
        }
    }
}
"#;

        fs::write(temp_dir.path().join("complex.js"), complex_function).unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show complexity indicators
        assert!(stdout.contains("complexity") || stdout.contains("Complexity") || stdout.contains("cyclomatic"),
            "Missing complexity metrics in output");
        assert!(stdout.contains("complexLogic") &&
               (stdout.contains("high") || stdout.contains("High") || stdout.contains("⚠")),
            "Missing high complexity warning for complex function");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_satd_annotations() {
        // RED: Must detect and show Self-Admitted Technical Debt
        let temp_dir = TempDir::new().unwrap();

        let code_with_debt = r#"
// TODO: Refactor this to use async/await
function oldStyleCallback(cb) {
    setTimeout(() => {
        cb("done");
    }, 1000);
}

// FIXME: This has a memory leak
function leakyFunction() {
    // HACK: Using global to store state
    window.globalState = window.globalState || [];
    window.globalState.push(new Array(1000000));
}
"#;

        fs::write(temp_dir.path().join("debt.js"), code_with_debt).unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show SATD markers
        assert!(stdout.contains("TODO") || stdout.contains("Technical Debt") || stdout.contains("SATD"),
            "Missing SATD annotations");
        assert!(stdout.contains("FIXME") || stdout.contains("memory leak"),
            "Missing FIXME annotation for memory leak");
        assert!(stdout.contains("HACK") || stdout.contains("global state"),
            "Missing HACK annotation");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_quality_insights() {
        // RED: Must provide quality insights and recommendations
        let temp_dir = TempDir::new().unwrap();

        // Create files with various quality issues
        fs::write(temp_dir.path().join("long.js"),
            &format!("function veryLong() {{\n{}}}", "console.log('line');\n".repeat(200) + "}")).unwrap();

        fs::write(temp_dir.path().join("duplicate.js"),
            "function copy1() { return 42; }\nfunction copy2() { return 42; }\nfunction copy3() { return 42; }").unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show quality insights section
        assert!(stdout.contains("Quality") || stdout.contains("Insights") || stdout.contains("Recommendations"),
            "Missing quality insights section");

        // Must identify specific issues
        assert!(stdout.contains("long") || stdout.contains("Long") || stdout.contains("lines") || stdout.contains("LOC"),
            "Missing insight about long function");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_dead_code_markers() {
        // RED: Must identify potentially dead code
        let temp_dir = TempDir::new().unwrap();

        let code_with_dead = r#"
function usedFunction() {
    return "I am used";
}

function unusedFunction() {
    return "I am never called";
}

// Export shows what's actually used
export { usedFunction };
"#;

        fs::write(temp_dir.path().join("mixed.js"), code_with_dead).unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show dead code indicators
        assert!(stdout.contains("unusedFunction") &&
               (stdout.contains("dead") || stdout.contains("Dead") || stdout.contains("unused") || stdout.contains("⚠")),
            "Missing dead code marker for unused function");
    }

    #[tokio::test]
    #[ignore = "Requires pmat binary to be built"]
    async fn red_must_show_wasm_function_details() {
        // RED: Must properly annotate WASM functions
        let temp_dir = TempDir::new().unwrap();

        let wasm_content = r#"
(module
  (func $fibonacci (param $n i32) (result i32)
    local.get $n
    i32.const 2
    i32.lt_s
    if (result i32)
      local.get $n
    else
      local.get $n
      i32.const 1
      i32.sub
      call $fibonacci
      local.get $n
      i32.const 2
      i32.sub
      call $fibonacci
      i32.add
    end
  )
  (export "fibonacci" (func $fibonacci))
)
"#;

        fs::write(temp_dir.path().join("math.wat"), wasm_content).unwrap();

        let output = std::process::Command::new("./target/debug/pmat")
            .args(&["context", "--project-path", temp_dir.path().to_str().unwrap(), "--format", "llm-optimized"])
            .output()
            .expect("Failed to run pmat");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Must show WASM function with special annotation
        assert!(stdout.contains("fibonacci") || stdout.contains("$fibonacci"),
            "Missing WASM function name");
        assert!(stdout.contains("WASM") || stdout.contains("WebAssembly") || stdout.contains(".wat"),
            "Missing WASM type annotation");
        assert!(stdout.contains("export") || stdout.contains("Export"),
            "Missing export annotation for WASM function");
    }
}

#[cfg(test)]
mod green_phase_implementation {
    use super::*;
    use crate::services::simple_deep_context::{SimpleDeepContext, SimpleAnalysisConfig};
    use crate::services::context::{ProjectContext, ProjectSummary};

    // Helper to create enhanced format output
    pub fn format_context_with_annotations(
        analysis_report: &crate::services::simple_deep_context::SimpleAnalysisReport,
        project_path: &std::path::Path,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("Project: {} (detected)\n\n",
            project_path.file_name().unwrap_or_default().to_string_lossy()));

        // Summary
        output.push_str("Summary:\n");
        output.push_str(&format!("- Files: {}\n", analysis_report.file_count));
        output.push_str(&format!("- Functions: {}\n", analysis_report.complexity_metrics.total_functions));
        output.push_str("\n");

        // Key Components - File breakdown with functions
        output.push_str("Key Components:\n\n");

        for file_detail in &analysis_report.file_complexity_details {
            output.push_str(&format!("File: {}\n", file_detail.file_path.display()));
            output.push_str(&format!("  Functions: {}\n", file_detail.function_count));

            if file_detail.high_complexity_functions > 0 {
                output.push_str(&format!("  ⚠ High Complexity: {} functions\n",
                    file_detail.high_complexity_functions));
            }

            output.push_str(&format!("  Average Complexity: {:.1}\n", file_detail.avg_complexity));
            output.push('\n');
        }

        // Quality Insights
        if analysis_report.complexity_metrics.high_complexity_count > 0 {
            output.push_str("Quality Insights:\n");
            output.push_str(&format!("- {} functions have high complexity and should be refactored\n",
                analysis_report.complexity_metrics.high_complexity_count));
            output.push_str(&format!("- Average complexity: {:.1}\n",
                analysis_report.complexity_metrics.avg_complexity));
            output.push('\n');
        }

        // Recommendations
        if !analysis_report.recommendations.is_empty() {
            output.push_str("Recommendations:\n");
            for rec in &analysis_report.recommendations {
                output.push_str(&format!("- {}\n", rec));
            }
        }

        output
    }
}

#[cfg(test)]
mod refactor_phase_quality {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    #[test]
    fn property_annotations_preserve_function_count() {
        fn prop(file_count: u8, functions_per_file: u8) -> TestResult {
            if file_count == 0 || functions_per_file == 0 {
                return TestResult::discard();
            }

            let total_functions = file_count as usize * functions_per_file as usize;

            // Create mock analysis report
            let mut file_details = Vec::new();
            for i in 0..file_count {
                file_details.push(crate::services::simple_deep_context::FileComplexityDetail {
                    file_path: format!("file{}.js", i).into(),
                    function_count: functions_per_file as usize,
                    high_complexity_functions: 0,
                    avg_complexity: 1.0,
                    complexity_score: 1.0,
                    function_names: vec![format!("function{}", i)],
                });
            }

            let report = crate::services::simple_deep_context::SimpleAnalysisReport {
                file_count: file_count as usize,
                analysis_duration: std::time::Duration::from_secs(1),
                complexity_metrics: crate::services::simple_deep_context::ComplexityMetrics {
                    total_functions: total_functions,
                    high_complexity_count: 0,
                    avg_complexity: 1.0,
                },
                recommendations: vec![],
                file_complexity_details: file_details,
            };

            let output = green_phase_implementation::format_context_with_annotations(&report, std::path::Path::new("test"));

            // Property: Output must mention the total function count
            TestResult::from_bool(output.contains(&format!("Functions: {}", total_functions)))
        }

        quickcheck(prop as fn(u8, u8) -> TestResult);
    }

    #[test]
    fn property_all_files_appear_in_output() {
        fn prop(file_names: Vec<String>) -> TestResult {
            if file_names.is_empty() {
                return TestResult::discard();
            }

            let mut file_details = Vec::new();
            for name in &file_names {
                if name.is_empty() {
                    return TestResult::discard();
                }

                file_details.push(crate::services::simple_deep_context::FileComplexityDetail {
                    file_path: format!("{}.js", name).into(),
                    function_count: 1,
                    high_complexity_functions: 0,
                    avg_complexity: 1.0,
                    complexity_score: 1.0,
                    function_names: vec![format!("function_{}", name)],
                });
            }

            let report = crate::services::simple_deep_context::SimpleAnalysisReport {
                file_count: file_names.len(),
                analysis_duration: std::time::Duration::from_secs(1),
                complexity_metrics: crate::services::simple_deep_context::ComplexityMetrics {
                    total_functions: file_names.len(),
                    high_complexity_count: 0,
                    avg_complexity: 1.0,
                },
                recommendations: vec![],
                file_complexity_details: file_details,
            };

            let output = green_phase_implementation::format_context_with_annotations(&report, std::path::Path::new("test"));

            // Property: All file names must appear in the output
            for name in &file_names {
                if !output.contains(&format!("{}.js", name)) {
                    return TestResult::failed();
                }
            }

            TestResult::passed()
        }

        quickcheck(prop as fn(Vec<String>) -> TestResult);
    }

    #[test]
    fn property_high_complexity_triggers_warning() {
        fn prop(high_complexity_count: u8) -> TestResult {
            let has_high_complexity = high_complexity_count > 0;

            let report = crate::services::simple_deep_context::SimpleAnalysisReport {
                file_count: 1,
                analysis_duration: std::time::Duration::from_secs(1),
                complexity_metrics: crate::services::simple_deep_context::ComplexityMetrics {
                    total_functions: 10,
                    high_complexity_count: high_complexity_count as usize,
                    avg_complexity: if has_high_complexity { 15.0 } else { 3.0 },
                },
                recommendations: if has_high_complexity {
                    vec!["Refactor high complexity functions".to_string()]
                } else {
                    vec![]
                },
                file_complexity_details: vec![crate::services::simple_deep_context::FileComplexityDetail {
                    file_path: "test.js".into(),
                    function_count: 10,
                    high_complexity_functions: high_complexity_count as usize,
                    avg_complexity: if has_high_complexity { 15.0 } else { 3.0 },
                    complexity_score: if has_high_complexity { 15.0 } else { 3.0 },
                    function_names: vec!["testFunction".to_string()],
                }],
            };

            let output = green_phase_implementation::format_context_with_annotations(&report, std::path::Path::new("test"));

            // Property: High complexity must trigger warnings
            if has_high_complexity {
                TestResult::from_bool(
                    output.contains("⚠") ||
                    output.contains("High Complexity") ||
                    output.contains("high complexity")
                )
            } else {
                TestResult::from_bool(!output.contains("⚠"))
            }
        }

        quickcheck(prop as fn(u8) -> TestResult);
    }
}