//! Proof Annotations Analysis Handler
//!
//! Refactored handler for formal proof annotation analysis.

use crate::cli::proof_annotation_helpers::*;
use crate::cli::{ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter};
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;
use crate::models::unified_ast::{Location, ProofAnnotation};
use crate::services::proof_annotator::ProofAnnotator;

/// Refactored handler for proof annotations analysis.
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_proof_annotations(
    project_path: PathBuf,
    format: ProofAnnotationOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    property_type: Option<PropertyTypeFilter>,
    verification_method: Option<VerificationMethodFilter>,
    output: Option<PathBuf>,
    _perf: bool,
    clear_cache: bool,
) -> Result<()> {
    eprintln!("🔍 Collecting proof annotations from project...");
    let start = Instant::now();

    // Setup annotator
    let annotator = setup_proof_annotator(clear_cache);

    // Create filter
    let filter = ProofAnnotationFilter {
        high_confidence_only,
        property_type,
        verification_method,
    };

    // Collect and filter annotations
    let annotations = collect_and_filter_annotations(&annotator, &project_path, &filter).await;
    let elapsed = start.elapsed();

    eprintln!("✅ Found {} matching proof annotations", annotations.len());

    // Format output using helpers
    let content = format_proof_annotations(
        format, 
        &annotations, 
        elapsed, 
        &annotator, 
        &project_path, 
        include_evidence
    )?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Proof annotations written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Format proof annotations based on output format (complexity: 6)
fn format_proof_annotations(
    format: ProofAnnotationOutputFormat,
    annotations: &[(Location, ProofAnnotation)],
    elapsed: std::time::Duration,
    annotator: &ProofAnnotator,
    project_path: &PathBuf,
    include_evidence: bool,
) -> Result<String> {
    match format {
        ProofAnnotationOutputFormat::Json => format_as_json(annotations, elapsed, annotator),
        ProofAnnotationOutputFormat::Summary => format_as_summary(annotations, elapsed),
        ProofAnnotationOutputFormat::Full => {
            format_as_full(annotations, project_path, include_evidence)
        }
        ProofAnnotationOutputFormat::Markdown => {
            format_as_markdown(annotations, project_path, include_evidence)
        }
        ProofAnnotationOutputFormat::Sarif => format_as_sarif(annotations, project_path),
    }
}
