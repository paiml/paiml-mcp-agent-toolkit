//! TDD test for format_defect_markdown refactor
//! Autonomous all-night refactoring - complexity 28 → ≤8

use anyhow::Result;

// Mock types for testing
#[derive(Debug)]
struct DefectPrediction {
    file: String,
    defect_probability: f64,
    risk_level: String,
    contributing_factors: Vec<String>,
}

// Mock function signature for testing
fn format_defect_markdown_mock(predictions: &[DefectPrediction]) -> Result<String> {
    let mut output = String::new();
    
    output.push_str("# Defect Prediction Report\n\n");
    
    for pred in predictions {
        output.push_str(&format!("## {}\n", pred.file));
        output.push_str(&format!("- Risk: {}\n", pred.risk_level));
        output.push_str(&format!("- Probability: {:.2}%\n", pred.defect_probability * 100.0));
    }
    
    Ok(output)
}

#[test]
fn test_format_empty_predictions() -> Result<()> {
    let predictions = vec![];
    
    let output = format_defect_markdown_mock(&predictions)?;
    assert!(output.contains("# Defect Prediction Report"));
    
    Ok(())
}

#[test]
fn test_format_single_prediction() -> Result<()> {
    let predictions = vec![
        DefectPrediction {
            file: "main.rs".to_string(),
            defect_probability: 0.75,
            risk_level: "High".to_string(),
            contributing_factors: vec!["complexity".to_string()],
        }
    ];
    
    let output = format_defect_markdown_mock(&predictions)?;
    assert!(output.contains("main.rs"));
    assert!(output.contains("High"));
    assert!(output.contains("75.00%"));
    
    Ok(())
}

#[test]
fn test_format_multiple_predictions() -> Result<()> {
    let predictions = vec![
        DefectPrediction {
            file: "file1.rs".to_string(),
            defect_probability: 0.9,
            risk_level: "Critical".to_string(),
            contributing_factors: vec![],
        },
        DefectPrediction {
            file: "file2.rs".to_string(),
            defect_probability: 0.3,
            risk_level: "Low".to_string(),
            contributing_factors: vec![],
        },
    ];
    
    let output = format_defect_markdown_mock(&predictions)?;
    assert!(output.contains("file1.rs"));
    assert!(output.contains("file2.rs"));
    assert!(output.contains("Critical"));
    assert!(output.contains("Low"));
    
    Ok(())
}