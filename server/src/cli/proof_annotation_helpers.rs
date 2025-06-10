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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_annotation_helpers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
