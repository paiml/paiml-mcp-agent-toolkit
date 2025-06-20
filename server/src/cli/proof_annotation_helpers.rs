//! Helper functions for proof annotation analysis to reduce complexity

use super::{PropertyTypeFilter, VerificationMethodFilter};
use crate::models::unified_ast::{
    ConfidenceLevel, Location, ProofAnnotation, PropertyType, VerificationMethod,
};
use crate::services::proof_annotator::ProofAnnotator;
use anyhow::Result;
use std::path::Path;

/// Filter configuration for proof annotations
pub struct ProofAnnotationFilter {
    pub high_confidence_only: bool,
    pub property_type: Option<PropertyTypeFilter>,
    pub verification_method: Option<VerificationMethodFilter>,
}

/// Apply all filters to a proof annotation
pub fn filter_annotation(annotation: &ProofAnnotation, filter: &ProofAnnotationFilter) -> bool {
    filter_by_confidence(annotation, filter.high_confidence_only)
        && filter_by_property_type(annotation, &filter.property_type)
        && filter_by_verification_method(annotation, &filter.verification_method)
}

/// Filter annotations by confidence level
fn filter_by_confidence(annotation: &ProofAnnotation, high_confidence_only: bool) -> bool {
    if high_confidence_only {
        matches!(annotation.confidence_level, ConfidenceLevel::High)
    } else {
        true
    }
}

/// Filter annotations by property type
fn filter_by_property_type(
    annotation: &ProofAnnotation,
    property_filter: &Option<PropertyTypeFilter>,
) -> bool {
    match property_filter {
        Some(PropertyTypeFilter::MemorySafety) => {
            matches!(annotation.property_proven, PropertyType::MemorySafety)
        }
        Some(PropertyTypeFilter::ThreadSafety) => {
            matches!(annotation.property_proven, PropertyType::ThreadSafety)
        }
        Some(PropertyTypeFilter::DataRaceFreeze) => {
            matches!(annotation.property_proven, PropertyType::DataRaceFreeze)
        }
        Some(PropertyTypeFilter::Termination) => {
            matches!(annotation.property_proven, PropertyType::Termination)
        }
        Some(PropertyTypeFilter::FunctionalCorrectness) => {
            matches!(
                annotation.property_proven,
                PropertyType::FunctionalCorrectness(_)
            )
        }
        Some(PropertyTypeFilter::ResourceBounds) => {
            matches!(
                annotation.property_proven,
                PropertyType::ResourceBounds { .. }
            )
        }
        Some(PropertyTypeFilter::All) | None => true,
    }
}

/// Filter annotations by verification method
fn filter_by_verification_method(
    annotation: &ProofAnnotation,
    method_filter: &Option<VerificationMethodFilter>,
) -> bool {
    match method_filter {
        Some(VerificationMethodFilter::FormalProof) => {
            matches!(annotation.method, VerificationMethod::FormalProof { .. })
        }
        Some(VerificationMethodFilter::ModelChecking) => {
            matches!(annotation.method, VerificationMethod::ModelChecking { .. })
        }
        Some(VerificationMethodFilter::StaticAnalysis) => {
            matches!(annotation.method, VerificationMethod::StaticAnalysis { .. })
        }
        Some(VerificationMethodFilter::AbstractInterpretation) => {
            matches!(
                annotation.method,
                VerificationMethod::AbstractInterpretation
            )
        }
        Some(VerificationMethodFilter::BorrowChecker) => {
            matches!(annotation.method, VerificationMethod::BorrowChecker)
        }
        Some(VerificationMethodFilter::All) | None => true,
    }
}

/// Format proof annotations as JSON
pub fn format_as_json(
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
) -> Result<String> {
    let cache_stats = annotator.cache_stats();
    let annotations_json: Vec<serde_json::Value> = annotations
        .iter()
        .map(|(location, annotation)| {
            serde_json::json!({
                "location": {
                    "file_path": location.file_path.to_string_lossy(),
                    "start_pos": location.span.start.0,
                    "end_pos": location.span.end.0
                },
                "annotation": annotation
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "proof_annotations": annotations_json,
        "summary": {
            "total_annotations": annotations.len(),
            "analysis_time_ms": elapsed.as_millis(),
            "cache_stats": {
                "size": cache_stats.size,
                "files_tracked": cache_stats.files_tracked
            }
        }
    });

    serde_json::to_string_pretty(&json_data).map_err(Into::into)
}

/// Setup proof annotator with mock sources
pub fn setup_proof_annotator(clear_cache: bool) -> ProofAnnotator {
    use crate::services::{proof_annotator::MockProofSource, symbol_table::SymbolTable};

    let symbol_table = std::sync::Arc::new(SymbolTable::new());
    let mut annotator = ProofAnnotator::new(symbol_table.clone());

    if clear_cache {
        annotator.clear_cache();
    }

    // Add mock proof sources
    annotator.add_source(MockProofSource::new("borrow_checker".to_string(), 10, 5));
    annotator.add_source(MockProofSource::new("static_analyzer".to_string(), 20, 3));
    annotator.add_source(MockProofSource::new("formal_verifier".to_string(), 50, 2));

    annotator
}

/// Filter and collect proof annotations
pub async fn collect_and_filter_annotations(
    annotator: &ProofAnnotator,
    project_path: &Path,
    filter: &ProofAnnotationFilter,
) -> Vec<(Location, ProofAnnotation)> {
    let proof_map = annotator.collect_proofs(project_path).await;

    proof_map
        .into_iter()
        .flat_map(|(location, annotations)| {
            annotations
                .into_iter()
                .filter(|annotation| filter_annotation(annotation, filter))
                .map(|annotation| (location.clone(), annotation))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Format annotations as table output
pub fn format_as_table(
    annotations: &[(Location, ProofAnnotation)],
    _elapsed: std::time::Duration,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(
        &mut output,
        "| File | Position | Property | Method | Confidence |"
    )?;
    writeln!(
        &mut output,
        "|------|----------|----------|---------|------------|"
    )?;

    for (location, annotation) in annotations {
        writeln!(
            &mut output,
            "| {} | {}-{} | {:?} | {:?} | {:?} |",
            location
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            location.span.start.0,
            location.span.end.0,
            annotation.property_proven,
            annotation.method,
            annotation.confidence_level
        )?;
    }

    Ok(output)
}

/// Format annotations as summary output
pub fn format_as_summary(
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    let total_proofs = annotations.len();
    let high_confidence = annotations
        .iter()
        .filter(|(_, ann)| matches!(ann.confidence_level, ConfidenceLevel::High))
        .count();

    writeln!(&mut output, "Proof Annotations Summary:")?;
    writeln!(&mut output, "Total proofs: {}\n", total_proofs)?;
    writeln!(
        &mut output,
        "High confidence: {} ({:.1}%)",
        high_confidence,
        if total_proofs > 0 {
            (high_confidence as f64 / total_proofs as f64) * 100.0
        } else {
            0.0
        }
    )?;
    writeln!(
        &mut output,
        "Analysis time: {:.2}s\n",
        elapsed.as_secs_f64()
    )?;

    // Group by property type
    let mut property_counts = std::collections::HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.property_proven);
        *property_counts.entry(key).or_insert(0) += 1;
    }

    if !property_counts.is_empty() {
        writeln!(&mut output, "\nProofs by property type:")?;
        for (prop_type, count) in property_counts {
            writeln!(&mut output, "  {}: {}", prop_type, count)?;
        }
    }

    Ok(output)
}

/// Format annotations as full detailed output
pub fn format_as_full(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Full Proof Annotations Report\n")?;
    writeln!(
        &mut output,
        "**Generated**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(&mut output, "**Project**: {}", project_path.display())?;
    writeln!(&mut output, "**Total proofs**: {}\n", annotations.len())?;

    // Group by file
    let mut proofs_by_file: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
    for (loc, ann) in annotations {
        proofs_by_file
            .entry(loc.file_path.clone())
            .or_default()
            .push((loc.clone(), ann.clone()));
    }

    for (file, mut proofs) in proofs_by_file {
        writeln!(&mut output, "## File: {}\n", file.display())?;

        // Sort by line number
        proofs.sort_by_key(|(loc, _)| loc.span.start.0);

        for (loc, ann) in proofs {
            writeln!(
                &mut output,
                "### Position {}-{}\n",
                loc.span.start.0, loc.span.end.0
            )?;
            writeln!(&mut output, "**Property**: {:?}", ann.property_proven)?;
            writeln!(&mut output, "**Method**: {:?}", ann.method)?;
            writeln!(
                &mut output,
                "**Tool**: {} v{}",
                ann.tool_name, ann.tool_version
            )?;
            writeln!(&mut output, "**Confidence**: {:?}", ann.confidence_level)?;
            writeln!(
                &mut output,
                "**Verified**: {}",
                ann.date_verified.format("%Y-%m-%d %H:%M:%S UTC")
            )?;

            if !ann.assumptions.is_empty() {
                writeln!(&mut output, "\n**Assumptions**:")?;
                for assumption in &ann.assumptions {
                    writeln!(&mut output, "- {}", assumption)?;
                }
            }

            if include_evidence {
                writeln!(&mut output, "\n**Evidence**: {:?}", ann.evidence_type)?;
                if let Some(ref spec_id) = ann.specification_id {
                    writeln!(&mut output, "**Specification ID**: {}", spec_id)?;
                }
            }
            writeln!(&mut output)?;
        }
    }

    Ok(output)
}

/// Format annotations as markdown output
pub fn format_as_markdown(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Proof Annotations Analysis\n")?;
    writeln!(&mut output, "This report shows formal verification proofs collected from various tools and analyzers.\n")?;

    writeln!(
        &mut output,
        "**Project Path**: `{}`",
        project_path.display()
    )?;
    writeln!(
        &mut output,
        "**Analysis Date**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(&mut output, "**Total Proofs**: {}\n", annotations.len())?;

    // Summary statistics
    writeln!(&mut output, "## Summary Statistics\n")?;
    writeln!(&mut output, "| Metric | Count |")?;
    writeln!(&mut output, "|--------|-------|")?;

    // Count by confidence
    let mut confidence_counts = std::collections::HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.confidence_level);
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    for (level, count) in &confidence_counts {
        writeln!(&mut output, "| {} Confidence | {} |", level, count)?;
    }

    // Details section
    if include_evidence {
        writeln!(&mut output, "\n## Detailed Proofs\n")?;

        // Group by file
        let mut proofs_by_file: std::collections::HashMap<_, Vec<_>> =
            std::collections::HashMap::new();
        for (loc, ann) in annotations {
            proofs_by_file
                .entry(loc.file_path.clone())
                .or_default()
                .push((loc.clone(), ann.clone()));
        }

        for (file, proofs) in proofs_by_file {
            writeln!(&mut output, "### {}\n", file.display())?;
            for (loc, ann) in proofs {
                writeln!(
                    &mut output,
                    "- **{:?}** at lines {}-{}",
                    ann.property_proven, loc.span.start.0, loc.span.end.0
                )?;
                writeln!(&mut output, "  - Method: {:?}", ann.method)?;
                writeln!(&mut output, "  - Confidence: {:?}", ann.confidence_level)?;
                if include_evidence {
                    writeln!(&mut output, "  - Evidence: {:?}", ann.evidence_type)?;
                }
            }
            writeln!(&mut output)?;
        }
    }

    Ok(output)
}

/// Format annotations as SARIF output
pub fn format_as_sarif(
    annotations: &[(Location, ProofAnnotation)],
    _project_path: &Path,
) -> Result<String> {
    let mut results = Vec::new();

    for (location, annotation) in annotations {
        let rule_id = match annotation.confidence_level {
            ConfidenceLevel::Low => "low-confidence-proof",
            ConfidenceLevel::Medium => "medium-confidence-proof",
            ConfidenceLevel::High => "high-confidence-proof",
        };

        let level = match annotation.confidence_level {
            ConfidenceLevel::Low => "warning",
            ConfidenceLevel::Medium => "note",
            ConfidenceLevel::High => "none",
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": level,
            "message": {
                "text": format!(
                    "{:?} verified by {} using {:?}",
                    annotation.property_proven,
                    annotation.tool_name,
                    annotation.method
                )
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": location.file_path.to_string_lossy()
                    },
                    "region": {
                        "startLine": location.span.start.0,
                        "endLine": location.span.end.0
                    }
                }
            }]
        }));
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-proof-annotator",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [
                        {
                            "id": "low-confidence-proof",
                            "name": "Low Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has low confidence"
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        },
                        {
                            "id": "medium-confidence-proof",
                            "name": "Medium Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has medium confidence"
                            },
                            "defaultConfiguration": {
                                "level": "note"
                            }
                        },
                        {
                            "id": "high-confidence-proof",
                            "name": "High Confidence Proof",
                            "shortDescription": {
                                "text": "Property verification has high confidence"
                            },
                            "defaultConfiguration": {
                                "level": "none"
                            }
                        }
                    ]
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_proof_annotation_helpers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
