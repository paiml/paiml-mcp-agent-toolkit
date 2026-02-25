// Filter functions for proof annotations: confidence, property type, and verification method filtering.

/// Apply all filters to a proof annotation
#[must_use]
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
