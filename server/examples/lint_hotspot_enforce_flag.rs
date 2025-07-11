//! Example demonstrating the --enforce flag behavior in lint-hotspot analysis
//!
//! This example shows how the --enforce flag affects exit status, which is critical
//! for CI/CD integration. The fix for issue #34 ensures that when --enforce is set
//! and violations are found, the command exits with non-zero status even if the
//! quality gate passes.
//!
//! # Usage
//!
//! ```bash
//! # Run this example to see the enforce flag behavior
//! cargo run --example lint_hotspot_enforce_flag
//! 
//! # Test enforce flag behavior in CI/CD (these commands will exit with status 1):
//! pmat analyze lint-hotspot --enforce
//! pmat analyze lint-hotspot --enforce --max-density 0.01
//! ```

use pmat::cli::handlers::lint_hotspot_handlers::{
    LintHotspot, LintHotspotResult, QualityGateStatus, SeverityDistribution, ViolationDetail,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Test function for checking exit behavior (from the test module)
fn should_exit_with_error(
    quality_gate_passed: bool,
    enforce: bool, 
    total_violations: usize
) -> bool {
    !quality_gate_passed || (enforce && total_violations > 0)
}

fn main() {
    println!("🔍 Lint Hotspot Enforce Flag Behavior Example");
    println!("{}", "=".repeat(50));
    
    // Create a realistic test scenario with violations
    let result_with_violations = create_result_with_violations();
    let result_without_violations = create_result_without_violations();
    
    println!("\n📊 Test Scenario 1: Quality Gate Passes, Enforce Flag Set, Violations Present");
    println!("   - Quality Gate: ✅ PASSED");
    println!("   - Enforce Flag: ✅ SET");
    println!("   - Total Violations: {}", result_with_violations.total_project_violations);
    
    let should_exit = should_exit_with_error(
        result_with_violations.quality_gate.passed,
        true, // enforce flag set
        result_with_violations.total_project_violations
    );
    
    if should_exit {
        println!("   - Exit Status: ❌ NON-ZERO (1) - ENFORCEMENT TRIGGERED");
        println!("   - Reason: Enforce flag is set and violations were found");
        println!("   - CI/CD Impact: Build will fail, preventing deployment");
    } else {
        println!("   - Exit Status: ✅ ZERO (0) - SUCCESS");
    }
    
    println!("\n📊 Test Scenario 2: Quality Gate Passes, Enforce Flag Set, No Violations");
    println!("   - Quality Gate: ✅ PASSED");
    println!("   - Enforce Flag: ✅ SET"); 
    println!("   - Total Violations: {}", result_without_violations.total_project_violations);
    
    let should_exit = should_exit_with_error(
        result_without_violations.quality_gate.passed,
        true, // enforce flag set
        result_without_violations.total_project_violations
    );
    
    if should_exit {
        println!("   - Exit Status: ❌ NON-ZERO (1)");
    } else {
        println!("   - Exit Status: ✅ ZERO (0) - SUCCESS");
        println!("   - Reason: No violations found despite enforce flag");
        println!("   - CI/CD Impact: Build succeeds, deployment allowed");
    }
    
    println!("\n📊 Test Scenario 3: Quality Gate Passes, No Enforce Flag, Violations Present");
    println!("   - Quality Gate: ✅ PASSED");
    println!("   - Enforce Flag: ❌ NOT SET");
    println!("   - Total Violations: {}", result_with_violations.total_project_violations);
    
    let should_exit = should_exit_with_error(
        result_with_violations.quality_gate.passed,
        false, // enforce flag not set
        result_with_violations.total_project_violations
    );
    
    if should_exit {
        println!("   - Exit Status: ❌ NON-ZERO (1)");
    } else {
        println!("   - Exit Status: ✅ ZERO (0) - SUCCESS");
        println!("   - Reason: Quality gate passed and no enforcement requested");
        println!("   - CI/CD Impact: Build succeeds, violations logged but not blocking");
    }
    
    println!("\n📊 Test Scenario 4: Quality Gate Failed (Always Blocks)");
    let failed_result = create_result_with_violations();
    // Simulate failed quality gate
    let failed_quality_gate_passed = false;
    
    println!("   - Quality Gate: ❌ FAILED");
    println!("   - Enforce Flag: ❌ NOT SET");
    println!("   - Total Violations: {}", failed_result.total_project_violations);
    
    let should_exit = should_exit_with_error(
        failed_quality_gate_passed,
        false, // enforce flag not set
        failed_result.total_project_violations
    );
    
    if should_exit {
        println!("   - Exit Status: ❌ NON-ZERO (1) - QUALITY GATE FAILED");
        println!("   - Reason: Quality gate failure always blocks regardless of enforce flag");
        println!("   - CI/CD Impact: Build fails, critical quality thresholds exceeded");
    } else {
        println!("   - Exit Status: ✅ ZERO (0) - SUCCESS");
    }
    
    println!("\n🔧 Key Behavior Changes (Issue #34 Fix):");
    println!("   ✅ BEFORE: --enforce flag had no effect on exit status");  
    println!("   ✅ AFTER:  --enforce flag triggers non-zero exit when violations exist");
    println!("   ✅ RESULT: CI/CD can now enforce zero-violation policies");
    
    println!("\n💡 Usage Recommendations:");
    println!("   🚀 CI/CD: Use --enforce for strict quality enforcement");
    println!("   📊 Local: Use without --enforce for analysis without blocking");
    println!("   🎯 Custom: Use --max-density with --enforce for specific thresholds");
    
    println!("\n🎉 Example completed successfully!");
}

fn create_result_with_violations() -> LintHotspotResult {
    let mut summary_by_file = HashMap::new();
    summary_by_file.insert(
        PathBuf::from("src/lib.rs"),
        pmat::cli::handlers::lint_hotspot_handlers::FileSummary {
            total_violations: 8,
            errors: 2,
            warnings: 6,
            sloc: 150,
            defect_density: 0.053,
        }
    );
    
    LintHotspotResult {
        hotspot: LintHotspot {
            file: PathBuf::from("src/lib.rs"),
            defect_density: 0.053,
            total_violations: 8,
            sloc: 150,
            severity_distribution: SeverityDistribution {
                error: 2,
                warning: 6,
                suggestion: 0,
                note: 0,
            },
            top_lints: vec![
                ("clippy::too_many_arguments".to_string(), 3),
                ("unused_variable".to_string(), 2),
                ("clippy::cognitive_complexity".to_string(), 2),
                ("dead_code".to_string(), 1),
            ],
            detailed_violations: vec![
                ViolationDetail {
                    file: PathBuf::from("src/lib.rs"),
                    line: 45,
                    column: 8,
                    end_line: 45,
                    end_column: 20,
                    lint_name: "clippy::too_many_arguments".to_string(),
                    message: "this function has too many arguments (8/7)".to_string(),
                    severity: "warning".to_string(),
                    suggestion: Some("consider using a struct".to_string()),
                    machine_applicable: false,
                },
                ViolationDetail {
                    file: PathBuf::from("src/lib.rs"),
                    line: 78,
                    column: 9,
                    end_line: 78,
                    end_column: 20,
                    lint_name: "unused_variable".to_string(), 
                    message: "unused variable: `temp_value`".to_string(),
                    severity: "warning".to_string(),
                    suggestion: Some("prefix with underscore".to_string()),
                    machine_applicable: true,
                },
            ],
        },
        all_violations: vec![],
        summary_by_file,
        total_project_violations: 8,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true, // Quality gate passes but violations exist
            violations: vec![],
            blocking: false,
        },
    }
}

fn create_result_without_violations() -> LintHotspotResult {
    LintHotspotResult {
        hotspot: LintHotspot {
            file: PathBuf::from("src/main.rs"),
            defect_density: 0.0,
            total_violations: 0,
            sloc: 50,
            severity_distribution: SeverityDistribution {
                error: 0,
                warning: 0,
                suggestion: 0,
                note: 0,
            },
            top_lints: vec![],
            detailed_violations: vec![],
        },
        all_violations: vec![],
        summary_by_file: HashMap::new(),
        total_project_violations: 0,
        enforcement: None,
        refactor_chain: None,
        quality_gate: QualityGateStatus {
            passed: true,
            violations: vec![],
            blocking: false,
        },
    }
}