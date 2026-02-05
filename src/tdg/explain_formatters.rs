//! Output formatters for TDG --explain mode (Issue #78)
//!
//! Provides text and JSON formatters for function-level complexity analysis.

use anyhow::{Context, Result};
use serde::Serialize;

use super::explain::ExplainedTDGScore;

/// Format ExplainedTDGScore as JSON
///
/// Produces JSON suitable for CI/CD integration with structure:
/// ```json
/// {
///   "functions": [
///     {
///       "name": "function_name",
///       "line": 42,
///       "cyclomatic": 15,
///       "cognitive": 18,
///       "tdg_impact": 3.2,
///       "severity": "High"
///     }
///   ],
///   "recommendations": [...],
///   "score": {...}
/// }
/// ```
pub fn format_explain_json(explained: &ExplainedTDGScore) -> Result<String> {
    // Create a serializable version with field name adjustments
    let output = ExplainJsonOutput {
        functions: explained
            .functions
            .iter()
            .map(|f| FunctionJson {
                name: f.name.clone(),
                line: f.line_number,
                cyclomatic: f.cyclomatic,
                cognitive: f.cognitive,
                tdg_impact: f.tdg_impact,
                severity: format!("{}", f.severity),
            })
            .collect(),
        recommendations: explained.recommendations.clone(),
        score: explained.score.clone(),
    };

    serde_json::to_string_pretty(&output).context("Failed to serialize to JSON")
}

/// Format ExplainedTDGScore as human-readable text
///
/// Produces terminal-friendly output with:
/// - Function-level complexity breakdown
/// - TDG impact scores
/// - Actionable recommendations
pub fn format_explain_text(explained: &ExplainedTDGScore) -> Result<String> {
    let mut output = String::new();

    // Header
    output.push_str("Function-Level Complexity Breakdown\n");
    output.push_str("===================================\n\n");

    // Functions section
    if explained.functions.is_empty() {
        output.push_str("No functions analyzed.\n");
    } else {
        for func in &explained.functions {
            output.push_str(&format!("{} (line {})\n", func.name, func.line_number));
            output.push_str(&format!("  Complexity: {}\n", func.cyclomatic));
            output.push_str(&format!("  Cognitive: {}\n", func.cognitive));
            output.push_str(&format!("  TDG Impact: {:.2}\n", func.tdg_impact));
            output.push_str(&format!("  Severity: {}\n", func.severity));
            output.push('\n');
        }
    }

    // Recommendations section
    if !explained.recommendations.is_empty() {
        output.push_str("\nRecommendations\n");
        output.push_str("===============\n\n");

        for rec in &explained.recommendations {
            output.push_str(&format!(
                "[+{:.1} pts] {}\n",
                rec.expected_impact, rec.action
            ));
            output.push_str(&format!("  Lines: {:?}\n", rec.lines));
            output.push_str(&format!("  Effort: {:.1} hours\n", rec.estimated_hours));
            output.push_str(&format!("  Priority: {}\n", rec.priority));
            output.push('\n');
        }
    }

    // Known Defects v2.1: Show defect information
    if explained.score.has_critical_defects {
        output.push_str("\n🔴 CRITICAL DEFECTS DETECTED\n");
        output.push_str("===========================\n\n");
        output.push_str(&format!(
            "Critical Defects: {}\n",
            explained.score.critical_defects_count
        ));
        output.push_str("Status: AUTO-FAIL (Score: 0.0, Grade: F)\n\n");
        output.push_str("Run 'pmat analyze defects' for detailed defect report.\n");
    }

    Ok(output)
}

/// JSON output structure with field name adjustments
#[derive(Debug, Serialize)]
struct ExplainJsonOutput {
    functions: Vec<FunctionJson>,
    recommendations: Vec<super::explain::ActionableRecommendation>,
    score: super::TdgScore,
}

/// Function JSON representation with "line" instead of "line_number"
#[derive(Debug, Serialize)]
struct FunctionJson {
    name: String,
    line: usize,
    cyclomatic: u32,
    cognitive: u32,
    tdg_impact: f64,
    severity: String,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{ActionableRecommendation, ComplexitySeverity, FunctionComplexity, TdgScore};

    // Helper to create a test score
    fn create_test_score() -> ExplainedTDGScore {
        ExplainedTDGScore::new(TdgScore::default())
    }

    // Helper to create a test function complexity
    fn create_test_function(
        name: &str,
        line: usize,
        cyclomatic: u32,
        cognitive: u32,
    ) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_number: line,
            cyclomatic,
            cognitive,
            tdg_impact: (cyclomatic as f64) * 0.5,
            severity: if cyclomatic > 20 {
                ComplexitySeverity::Critical
            } else if cyclomatic > 10 {
                ComplexitySeverity::High
            } else if cyclomatic > 5 {
                ComplexitySeverity::Medium
            } else {
                ComplexitySeverity::Low
            },
        }
    }

    // === JSON Formatter Tests ===

    #[test]
    fn test_format_explain_json() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "test_function".to_string(),
            line_number: 42,
            cyclomatic: 15,
            cognitive: 18,
            tdg_impact: 3.2,
            severity: ComplexitySeverity::High,
        });

        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Verify structure
        assert!(json.get("functions").is_some());
        let functions = json["functions"].as_array().unwrap();
        assert_eq!(functions.len(), 1);

        let func = &functions[0];
        assert_eq!(func["name"].as_str().unwrap(), "test_function");
        assert_eq!(func["line"].as_u64().unwrap(), 42);
        assert_eq!(func["cyclomatic"].as_u64().unwrap(), 15);
        assert_eq!(func["tdg_impact"].as_f64().unwrap(), 3.2);
    }

    #[test]
    fn test_format_explain_json_empty() {
        let explained = create_test_score();
        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(json.get("functions").is_some());
        let functions = json["functions"].as_array().unwrap();
        assert!(functions.is_empty());
    }

    #[test]
    fn test_format_explain_json_multiple_functions() {
        let mut explained = create_test_score();

        explained.add_function(create_test_function("func_a", 10, 5, 8));
        explained.add_function(create_test_function("func_b", 50, 15, 20));
        explained.add_function(create_test_function("func_c", 100, 25, 30));

        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        let functions = json["functions"].as_array().unwrap();
        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0]["name"].as_str().unwrap(), "func_a");
        assert_eq!(functions[1]["name"].as_str().unwrap(), "func_b");
        assert_eq!(functions[2]["name"].as_str().unwrap(), "func_c");
    }

    #[test]
    fn test_format_explain_json_severity_format() {
        let mut explained = create_test_score();
        explained.add_function(create_test_function("low_complexity", 10, 3, 4));

        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        let func = &json["functions"].as_array().unwrap()[0];
        // Severity should be a string
        assert!(func["severity"].as_str().is_some());
    }

    #[test]
    fn test_format_explain_json_has_recommendations() {
        let explained = create_test_score();
        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(json.get("recommendations").is_some());
    }

    #[test]
    fn test_format_explain_json_has_score() {
        let explained = create_test_score();
        let output = format_explain_json(&explained).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(json.get("score").is_some());
    }

    // === Text Formatter Tests ===

    #[test]
    fn test_format_explain_text() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "test_function".to_string(),
            line_number: 42,
            cyclomatic: 15,
            cognitive: 18,
            tdg_impact: 3.2,
            severity: ComplexitySeverity::High,
        });

        let output = format_explain_text(&explained).unwrap();

        // Verify output contains key sections
        assert!(output.contains("Function-Level Complexity"));
        assert!(output.contains("test_function"));
        assert!(output.contains("line 42"));
        assert!(output.contains("Complexity: 15"));
    }

    #[test]
    fn test_format_explain_text_empty() {
        let explained = create_test_score();
        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("Function-Level Complexity Breakdown"));
        assert!(output.contains("No functions analyzed"));
    }

    #[test]
    fn test_format_explain_text_header() {
        let explained = create_test_score();
        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("Function-Level Complexity Breakdown"));
        assert!(output.contains("==================================="));
    }

    #[test]
    fn test_format_explain_text_multiple_functions() {
        let mut explained = create_test_score();

        explained.add_function(create_test_function("first_func", 10, 5, 8));
        explained.add_function(create_test_function("second_func", 50, 15, 20));

        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("first_func"));
        assert!(output.contains("line 10"));
        assert!(output.contains("second_func"));
        assert!(output.contains("line 50"));
    }

    #[test]
    fn test_format_explain_text_function_details() {
        let mut explained = create_test_score();
        explained.add_function(FunctionComplexity {
            name: "complex_func".to_string(),
            line_number: 100,
            cyclomatic: 25,
            cognitive: 30,
            tdg_impact: 12.5,
            severity: ComplexitySeverity::Critical,
        });

        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("Complexity: 25"));
        assert!(output.contains("Cognitive: 30"));
        assert!(output.contains("TDG Impact: 12.50"));
        assert!(output.contains("Severity:"));
    }

    #[test]
    fn test_format_explain_text_with_recommendations() {
        let mut explained = create_test_score();

        explained.add_recommendation(ActionableRecommendation {
            rec_type: crate::tdg::RecommendationType::ExtractFunction,
            action: "Extract complex logic into helper function".to_string(),
            lines: vec![10, 20, 30],
            expected_impact: 5.5,
            estimated_hours: 2.0,
            priority: 1,
            pattern: "high_complexity".to_string(),
        });

        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("Recommendations"));
        assert!(output.contains("Extract complex logic"));
        assert!(output.contains("[+5.5 pts]"));
        assert!(output.contains("Effort: 2.0 hours"));
        assert!(output.contains("Priority: 1"));
    }

    #[test]
    fn test_format_explain_text_with_critical_defects() {
        let mut score = TdgScore::default();
        score.has_critical_defects = true;
        score.critical_defects_count = 3;

        let explained = ExplainedTDGScore::new(score);
        let output = format_explain_text(&explained).unwrap();

        assert!(output.contains("CRITICAL DEFECTS DETECTED"));
        assert!(output.contains("Critical Defects: 3"));
        assert!(output.contains("AUTO-FAIL"));
        assert!(output.contains("Grade: F"));
    }

    #[test]
    fn test_format_explain_text_no_critical_defects() {
        let explained = create_test_score();
        let output = format_explain_text(&explained).unwrap();

        assert!(!output.contains("CRITICAL DEFECTS DETECTED"));
    }

    // === FunctionJson Tests ===

    #[test]
    fn test_function_json_serialization() {
        let func = FunctionJson {
            name: "test".to_string(),
            line: 42,
            cyclomatic: 10,
            cognitive: 15,
            tdg_impact: 5.0,
            severity: "High".to_string(),
        };

        let json = serde_json::to_string(&func).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"line\":42"));
        assert!(json.contains("\"cyclomatic\":10"));
    }

    #[test]
    fn test_function_json_debug() {
        let func = FunctionJson {
            name: "test".to_string(),
            line: 42,
            cyclomatic: 10,
            cognitive: 15,
            tdg_impact: 5.0,
            severity: "High".to_string(),
        };

        let debug = format!("{:?}", func);
        assert!(debug.contains("FunctionJson"));
        assert!(debug.contains("test"));
    }

    // === ExplainJsonOutput Tests ===

    #[test]
    fn test_explain_json_output_serialization() {
        let output = ExplainJsonOutput {
            functions: vec![],
            recommendations: vec![],
            score: TdgScore::default(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"functions\":[]"));
        assert!(json.contains("\"recommendations\":[]"));
    }
}
