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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{ComplexitySeverity, FunctionComplexity, TdgScore};

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
}
