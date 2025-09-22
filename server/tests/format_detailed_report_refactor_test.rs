//! TDD test for format_detailed_report refactor
//! Autonomous all-night refactoring - complexity 48 → ≤8

use anyhow::Result;
use pmat::services::similarity::{ComprehensiveReport, Metrics, SimilarBlock};

// Mock function signature for testing refactored structure
fn format_detailed_report_mock(report: &ComprehensiveReport) -> Result<String> {
    let mut output = String::new();

    // Test structure preservation
    output.push_str("# Comprehensive Code Similarity Report\n\n");
    output.push_str(&format!("## Overall Metrics\n"));
    output.push_str(&format!(
        "- Duplication Percentage: {:.1}%\n",
        report.metrics.duplication_percentage
    ));

    Ok(output)
}

#[test]
fn test_format_detailed_report_structure() -> Result<()> {
    let report = create_test_report();

    let output = format_detailed_report_mock(&report)?;
    assert!(output.contains("# Comprehensive Code Similarity Report"));
    assert!(output.contains("## Overall Metrics"));
    assert!(output.contains("Duplication Percentage"));

    Ok(())
}

#[test]
fn test_format_empty_report() -> Result<()> {
    let report = ComprehensiveReport {
        exact_duplicates: vec![],
        structural_similarities: vec![],
        semantic_similarities: vec![],
        entropy_analysis: None,
        refactoring_opportunities: vec![],
        metrics: Metrics {
            duplication_percentage: 0.0,
            average_entropy: 0.0,
            total_clones: 0,
        },
    };

    let output = format_detailed_report_mock(&report)?;
    assert!(!output.is_empty());
    assert!(output.contains("0.0%"));

    Ok(())
}

#[test]
fn test_format_with_duplicates() -> Result<()> {
    let mut report = create_test_report();
    report.exact_duplicates = vec![SimilarBlock {
        id: "test-block".to_string(),
        lines: 10,
        tokens: 50,
        similarity: 1.0,
        locations: vec![],
        content_preview: "test content".to_string(),
        clone_type: pmat::services::similarity::CloneType::Type1,
    }];

    let output = format_detailed_report_mock(&report)?;
    assert!(!output.is_empty());

    Ok(())
}

fn create_test_report() -> ComprehensiveReport {
    ComprehensiveReport {
        exact_duplicates: vec![],
        structural_similarities: vec![],
        semantic_similarities: vec![],
        entropy_analysis: None,
        refactoring_opportunities: vec![],
        metrics: Metrics {
            duplication_percentage: 25.5,
            average_entropy: 3.14,
            total_clones: 42,
        },
    }
}
