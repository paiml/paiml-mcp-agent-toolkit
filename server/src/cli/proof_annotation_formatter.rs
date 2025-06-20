//! Formatting functions for proof annotations to reduce complexity

use crate::models::unified_ast::{Location, ProofAnnotation};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;

/// Format confidence level statistics
pub fn format_confidence_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    let mut confidence_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.confidence_level);
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    if !confidence_counts.is_empty() {
        writeln!(output, "\n## Confidence Levels\n")?;
        for (level, count) in confidence_counts {
            writeln!(output, "- {}: {} proofs", level, count)?;
        }
    }
    Ok(())
}

/// Format verification method statistics
pub fn format_method_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    use crate::models::unified_ast::VerificationMethod;

    let mut method_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = match &ann.method {
            VerificationMethod::FormalProof { .. } => "Formal Proof",
            VerificationMethod::ModelChecking { .. } => "Model Checking",
            VerificationMethod::StaticAnalysis { .. } => "Static Analysis",
            VerificationMethod::AbstractInterpretation => "Abstract Interpretation",
            VerificationMethod::BorrowChecker => "Borrow Checker",
        };
        *method_counts.entry(key).or_insert(0) += 1;
    }

    if !method_counts.is_empty() {
        writeln!(output, "\n## Verification Methods\n")?;
        for (method, count) in method_counts {
            writeln!(output, "- {}: {} proofs", method, count)?;
        }
    }
    Ok(())
}

/// Format property type statistics
pub fn format_property_stats(
    annotations: &[(Location, ProofAnnotation)],
    output: &mut String,
) -> Result<()> {
    let mut property_counts = HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.property_proven);
        *property_counts.entry(key).or_insert(0) += 1;
    }

    if !property_counts.is_empty() {
        writeln!(output, "## Properties Proven\n")?;
        for (prop, count) in property_counts {
            writeln!(output, "- {}: {} proofs", prop, count)?;
        }
    }
    Ok(())
}

/// Group annotations by file
pub fn group_by_file(
    annotations: &[(Location, ProofAnnotation)],
) -> HashMap<std::path::PathBuf, Vec<(Location, ProofAnnotation)>> {
    let mut proofs_by_file = HashMap::new();
    for (loc, ann) in annotations {
        proofs_by_file
            .entry(loc.file_path.clone())
            .or_insert_with(Vec::new)
            .push((loc.clone(), ann.clone()));
    }

    // Sort each file's proofs by line number
    for proofs in proofs_by_file.values_mut() {
        proofs.sort_by_key(|(loc, _)| loc.span.start.0);
    }

    proofs_by_file
}

/// Format a single proof annotation
pub fn format_single_proof(
    location: &Location,
    annotation: &ProofAnnotation,
    output: &mut String,
    include_evidence: bool,
) -> Result<()> {
    writeln!(
        output,
        "### Position {}-{}\n",
        location.span.start.0, location.span.end.0
    )?;
    writeln!(output, "**Property**: {:?}", annotation.property_proven)?;
    writeln!(output, "**Method**: {:?}", annotation.method)?;
    writeln!(
        output,
        "**Tool**: {} v{}",
        annotation.tool_name, annotation.tool_version
    )?;
    writeln!(output, "**Confidence**: {:?}", annotation.confidence_level)?;
    writeln!(
        output,
        "**Verified**: {}",
        annotation.date_verified.format("%Y-%m-%d %H:%M:%S UTC")
    )?;

    if !annotation.assumptions.is_empty() {
        writeln!(output, "\n**Assumptions**:")?;
        for assumption in &annotation.assumptions {
            writeln!(output, "- {}", assumption)?;
        }
    }

    if include_evidence {
        writeln!(output, "\n**Evidence**: {:?}", annotation.evidence_type)?;
        if let Some(ref spec_id) = annotation.specification_id {
            writeln!(output, "**Specification ID**: {}", spec_id)?;
        }
    }
    writeln!(output)?;
    Ok(())
}

/// Format provability-specific output
pub fn format_provability_summary(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
    output: &mut String,
    _include_evidence: bool,
) -> Result<()> {
    let total_functions = summaries.len();
    let high_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.8)
        .count();
    let medium_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.5 && s.provability_score < 0.8)
        .count();
    let low_provability = summaries
        .iter()
        .filter(|s| s.provability_score < 0.5)
        .count();

    writeln!(output, "## Provability Analysis Summary\n")?;
    writeln!(output, "**Total Functions**: {}", total_functions)?;
    writeln!(
        output,
        "**High Provability (≥80%)**: {} ({:.1}%)",
        high_provability,
        (high_provability as f64 / total_functions as f64) * 100.0
    )?;
    writeln!(
        output,
        "**Medium Provability (50-79%)**: {} ({:.1}%)",
        medium_provability,
        (medium_provability as f64 / total_functions as f64) * 100.0
    )?;
    writeln!(
        output,
        "**Low Provability (<50%)**: {} ({:.1}%)",
        low_provability,
        (low_provability as f64 / total_functions as f64) * 100.0
    )?;

    let avg_score =
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / total_functions as f64;
    writeln!(output, "**Average Score**: {:.1}%\n", avg_score * 100.0)?;

    Ok(())
}

/// Generate SARIF rules for proof annotations
pub fn generate_proof_sarif_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "low-confidence-proof",
            "name": "Low Confidence Proof",
            "shortDescription": {
                "text": "Property verification has low confidence"
            },
            "fullDescription": {
                "text": "The verification method used has low confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "warning"
            }
        }),
        serde_json::json!({
            "id": "medium-confidence-proof",
            "name": "Medium Confidence Proof",
            "shortDescription": {
                "text": "Property verification has medium confidence"
            },
            "fullDescription": {
                "text": "The verification method used has medium confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
        serde_json::json!({
            "id": "high-confidence-proof",
            "name": "High Confidence Proof",
            "shortDescription": {
                "text": "Property verification has high confidence"
            },
            "fullDescription": {
                "text": "The verification method used has high confidence in the proof"
            },
            "defaultConfiguration": {
                "level": "none"
            }
        }),
        serde_json::json!({
            "id": "unverified-property",
            "name": "Unverified Safety Property",
            "shortDescription": {
                "text": "Critical safety property could not be verified"
            },
            "fullDescription": {
                "text": "Important properties like memory safety or null safety could not be formally verified"
            },
            "defaultConfiguration": {
                "level": "note"
            }
        }),
    ]
}
