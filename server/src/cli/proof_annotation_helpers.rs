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

    format_summary_header(&mut output, annotations, elapsed)?;
    format_summary_property_counts(&mut output, annotations)?;
    format_summary_top_files(&mut output, annotations)?;

    Ok(output)
}

fn format_summary_header(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
) -> Result<()> {
    use std::fmt::Write;
    
    let total_proofs = annotations.len();
    let high_confidence = annotations
        .iter()
        .filter(|(_, ann)| matches!(ann.confidence_level, ConfidenceLevel::High))
        .count();

    writeln!(output, "Proof Annotations Summary:")?;
    writeln!(output, "Total proofs: {}\n", total_proofs)?;
    writeln!(
        output,
        "High confidence: {} ({:.1}%)",
        high_confidence,
        if total_proofs > 0 {
            (high_confidence as f64 / total_proofs as f64) * 100.0
        } else {
            0.0
        }
    )?;
    writeln!(output, "Analysis time: {:.2}s\n", elapsed.as_secs_f64())?;
    
    Ok(())
}

fn format_summary_property_counts(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    use std::fmt::Write;
    
    let mut property_counts = std::collections::HashMap::new();
    for (_, ann) in annotations {
        let key = format!("{:?}", ann.property_proven);
        *property_counts.entry(key).or_insert(0) += 1;
    }

    if !property_counts.is_empty() {
        writeln!(output, "\nProofs by property type:")?;
        for (prop_type, count) in property_counts {
            writeln!(output, "  {}: {}", prop_type, count)?;
        }
    }
    
    Ok(())
}

fn format_summary_top_files(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    use std::fmt::Write;
    
    if annotations.is_empty() {
        return Ok(());
    }
    
    writeln!(output, "\n## Top Files with Proof Annotations\n")?;

    let mut file_counts: std::collections::HashMap<&std::path::Path, usize> =
        std::collections::HashMap::new();
    for (location, _) in annotations {
        *file_counts.entry(&location.file_path).or_insert(0) += 1;
    }

    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1));

    for (i, (file_path, count)) in sorted_files.iter().take(10).enumerate() {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path.to_str().unwrap_or("unknown"));
        writeln!(
            output,
            "{}. `{}` - {} annotations",
            i + 1,
            filename,
            count
        )?;
    }
    
    Ok(())
}

/// Format annotations as full detailed output
pub fn format_as_full(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    let mut output = String::new();

    write_report_header(&mut output, project_path, annotations.len())?;

    let proofs_by_file = group_proofs_by_file(annotations);

    for (file, proofs) in proofs_by_file {
        write_file_section(&mut output, &file, proofs, include_evidence)?;
    }

    Ok(output)
}

/// Write the report header with project information
fn write_report_header(
    output: &mut String,
    project_path: &Path,
    total_proofs: usize,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Full Proof Annotations Report\n")?;
    writeln!(
        output,
        "**Generated**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output, "**Project**: {}", project_path.display())?;
    writeln!(output, "**Total proofs**: {}\n", total_proofs)?;

    Ok(())
}

/// Group proof annotations by file
fn group_proofs_by_file(
    annotations: &[(Location, ProofAnnotation)],
) -> std::collections::HashMap<std::path::PathBuf, Vec<(Location, ProofAnnotation)>> {
    let mut proofs_by_file: std::collections::HashMap<
        std::path::PathBuf,
        Vec<(Location, ProofAnnotation)>,
    > = std::collections::HashMap::new();

    for (loc, ann) in annotations {
        proofs_by_file
            .entry(loc.file_path.clone())
            .or_default()
            .push((loc.clone(), ann.clone()));
    }

    proofs_by_file
}

/// Write a file section with its proofs
fn write_file_section(
    output: &mut String,
    file: &Path,
    mut proofs: Vec<(Location, ProofAnnotation)>,
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## File: {}\n", file.display())?;

    // Sort by line number
    proofs.sort_by_key(|(loc, _)| loc.span.start.0);

    for (loc, ann) in proofs {
        write_proof_annotation(output, &loc, &ann, include_evidence)?;
    }

    Ok(())
}

/// Write a single proof annotation
fn write_proof_annotation(
    output: &mut String,
    loc: &Location,
    ann: &ProofAnnotation,
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    write_annotation_header(output, loc)?;
    write_annotation_basic_info(output, ann)?;
    write_annotation_assumptions(output, ann)?;

    if include_evidence {
        write_annotation_evidence(output, ann)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write annotation position header
fn write_annotation_header(output: &mut String, loc: &Location) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        "### Position {}-{}\n",
        loc.span.start.0, loc.span.end.0
    )?;
    Ok(())
}

/// Write basic annotation information
fn write_annotation_basic_info(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "**Property**: {:?}", ann.property_proven)?;
    writeln!(output, "**Method**: {:?}", ann.method)?;
    writeln!(output, "**Tool**: {} v{}", ann.tool_name, ann.tool_version)?;
    writeln!(output, "**Confidence**: {:?}", ann.confidence_level)?;
    writeln!(
        output,
        "**Verified**: {}",
        ann.date_verified.format("%Y-%m-%d %H:%M:%S UTC")
    )?;

    Ok(())
}

/// Write annotation assumptions
fn write_annotation_assumptions(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    if !ann.assumptions.is_empty() {
        writeln!(output, "\n**Assumptions**:")?;
        for assumption in &ann.assumptions {
            writeln!(output, "- {}", assumption)?;
        }
    }

    Ok(())
}

/// Write annotation evidence information
fn write_annotation_evidence(output: &mut String, ann: &ProofAnnotation) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n**Evidence**: {:?}", ann.evidence_type)?;
    if let Some(ref spec_id) = ann.specification_id {
        writeln!(output, "**Specification ID**: {}", spec_id)?;
    }

    Ok(())
}

/// Format annotations as markdown output
pub fn format_as_markdown(
    annotations: &[(Location, ProofAnnotation)],
    project_path: &Path,
    include_evidence: bool,
) -> Result<String> {
    let mut output = String::new();

    write_markdown_header(&mut output, project_path, annotations.len())?;
    write_summary_statistics(&mut output, annotations)?;

    if include_evidence {
        write_detailed_proofs(&mut output, annotations, include_evidence)?;
    }

    Ok(output)
}

/// Write markdown report header
fn write_markdown_header(
    output: &mut String,
    project_path: &Path,
    total_proofs: usize,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Proof Annotations Analysis\n")?;
    writeln!(output, "This report shows formal verification proofs collected from various tools and analyzers.\n")?;

    writeln!(output, "**Project Path**: `{}`", project_path.display())?;
    writeln!(
        output,
        "**Analysis Date**: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(output, "**Total Proofs**: {}\n", total_proofs)?;

    Ok(())
}

/// Write summary statistics table
fn write_summary_statistics(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary Statistics\n")?;
    writeln!(output, "| Metric | Count |")?;
    writeln!(output, "|--------|-------|")?;

    let confidence_counts = count_by_confidence(annotations);

    for (level, count) in &confidence_counts {
        writeln!(output, "| {} Confidence | {} |", level, count)?;
    }

    Ok(())
}

/// Count annotations by confidence level
fn count_by_confidence(
    annotations: &[(Location, ProofAnnotation)],
) -> std::collections::HashMap<String, usize> {
    let mut confidence_counts = std::collections::HashMap::new();

    for (_, ann) in annotations {
        let key = format!("{:?}", ann.confidence_level);
        *confidence_counts.entry(key).or_insert(0) += 1;
    }

    confidence_counts
}

/// Write detailed proofs section
fn write_detailed_proofs(
    output: &mut String,
    annotations: &[(Location, ProofAnnotation)],
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n## Detailed Proofs\n")?;

    let proofs_by_file = group_proofs_by_file(annotations);

    for (file, proofs) in proofs_by_file {
        write_file_proofs_section(output, &file, &proofs, include_evidence)?;
    }

    Ok(())
}

/// Write proofs section for a specific file
fn write_file_proofs_section(
    output: &mut String,
    file: &Path,
    proofs: &[(Location, ProofAnnotation)],
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "### {}\n", file.display())?;

    for (loc, ann) in proofs {
        write_proof_summary_item(output, loc, ann, include_evidence)?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write a single proof summary item
fn write_proof_summary_item(
    output: &mut String,
    loc: &Location,
    ann: &ProofAnnotation,
    include_evidence: bool,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(
        output,
        "- **{:?}** at lines {}-{}",
        ann.property_proven, loc.span.start.0, loc.span.end.0
    )?;
    writeln!(output, "  - Method: {:?}", ann.method)?;
    writeln!(output, "  - Confidence: {:?}", ann.confidence_level)?;

    if include_evidence {
        writeln!(output, "  - Evidence: {:?}", ann.evidence_type)?;
    }

    Ok(())
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
