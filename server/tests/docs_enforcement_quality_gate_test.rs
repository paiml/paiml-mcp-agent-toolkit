//! Quality Gate Integration Test for Documentation Enforcement
//!
//! TICKET: PMAT-7001 Phase 3 (REFACTOR)
//!
//! This test verifies that documentation enforcement is properly integrated
//! into the quality gate system and can be used as part of automated checks.

use pmat::services::quality_gate_service::{
    QualityCheck, QualityGateInput, QualityGateService,
};
use pmat::services::service_base::Service;
use std::path::PathBuf;

#[tokio::test]
async fn test_docs_enforcement_quality_gate_mcp_only() {
    let service = QualityGateService::new();

    let input = QualityGateInput {
        path: PathBuf::from("."),
        checks: vec![QualityCheck::DocsEnforcement {
            check_cli: false,
            check_mcp: true,
        }],
        strict: false,
    };

    let result = service.process(input).await;
    assert!(result.is_ok(), "Quality gate should execute successfully");

    let output = result.unwrap();
    assert_eq!(output.results.len(), 1, "Should have 1 check result");

    let docs_result = &output.results[0];
    assert_eq!(
        docs_result.check,
        "Documentation Enforcement (PMAT-7001)"
    );

    println!("Result: passed={}, message={}, violations={}",
        docs_result.passed, docs_result.message, docs_result.violations.len());
    for v in &docs_result.violations {
        println!("  Violation: {:?} - {}", v.severity, v.message);
    }

    assert!(
        docs_result.passed,
        "MCP documentation should pass: {} (violations: {})",
        docs_result.message,
        docs_result.violations.len()
    );
}

#[tokio::test]
async fn test_docs_enforcement_quality_gate_both() {
    let service = QualityGateService::new();

    let input = QualityGateInput {
        path: PathBuf::from("."),
        checks: vec![QualityCheck::DocsEnforcement {
            check_cli: true,
            check_mcp: true,
        }],
        strict: false,
    };

    let result = service.process(input).await;
    assert!(result.is_ok(), "Quality gate should execute successfully");

    let output = result.unwrap();
    assert_eq!(output.results.len(), 1);

    let docs_result = &output.results[0];
    assert_eq!(
        docs_result.check,
        "Documentation Enforcement (PMAT-7001)"
    );

    // MCP should pass, CLI shows info message
    assert!(
        docs_result.message.contains("MCP"),
        "Message should mention MCP: {}",
        docs_result.message
    );
}

#[tokio::test]
async fn test_docs_enforcement_with_other_checks() {
    let service = QualityGateService::new();

    let input = QualityGateInput {
        path: PathBuf::from("."),
        checks: vec![
            QualityCheck::Complexity { max: 10 },
            QualityCheck::DocsEnforcement {
                check_cli: false,
                check_mcp: true,
            },
            QualityCheck::Satd { tolerance: 10 },
        ],
        strict: false,
    };

    let result = service.process(input).await;
    assert!(result.is_ok(), "Quality gate should execute successfully");

    let output = result.unwrap();
    assert_eq!(output.results.len(), 3, "Should have 3 check results");

    // Find the docs enforcement result
    let docs_result = output
        .results
        .iter()
        .find(|r| r.check.contains("Documentation Enforcement"))
        .expect("Should find docs enforcement result");

    assert!(docs_result.passed, "Documentation enforcement should pass");
}

#[tokio::test]
async fn test_quality_gate_summary_with_docs_enforcement() {
    let service = QualityGateService::new();

    let input = QualityGateInput {
        path: PathBuf::from("."),
        checks: vec![
            QualityCheck::DocsEnforcement {
                check_cli: false,
                check_mcp: true,
            },
        ],
        strict: true,
    };

    let result = service.process(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.passed, "Quality gate should pass in strict mode");
    assert_eq!(output.summary.total_checks, 1);
    assert_eq!(output.summary.passed_checks, 1);
    assert_eq!(output.summary.failed_checks, 0);
}
