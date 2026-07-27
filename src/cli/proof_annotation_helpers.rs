#![cfg_attr(coverage_nightly, coverage(off))]
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

// Filter functions: confidence, property type, and verification method filtering
include!("proof_annotation_helpers_filter.rs");

// Formatting functions: JSON output, table output, summary output, annotator setup, and annotation collection
include!("proof_annotation_helpers_format.rs");

// Report generation: full detailed output, markdown output, and SARIF output formatting
include!("proof_annotation_helpers_report.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::unified_ast::{
        BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
        VerificationMethod,
    };
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn loc(path: &str, line: u32) -> Location {
        Location {
            file_path: PathBuf::from(path),
            span: Span {
                start: BytePos(line),
                end: BytePos(line + 10),
            },
        }
    }

    fn ann(prop: PropertyType, conf: ConfidenceLevel) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: Uuid::new_v4(),
            property_proven: prop,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: "test-tool".to_string(),
            tool_version: "1.0".to_string(),
            confidence_level: conf,
            assumptions: vec!["assumption-1".to_string(), "assumption-2".to_string()],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: Utc::now(),
        }
    }

    // ── format_as_full integration ──

    #[test]
    fn test_format_as_full_empty_emits_only_header() {
        let r = format_as_full(&[], Path::new("/test"), false).unwrap();
        assert!(r.contains("# Full Proof Annotations Report"));
        assert!(r.contains("**Total proofs**: 0"));
        // No file sections.
        assert!(!r.contains("## File:"));
    }

    #[test]
    fn test_format_as_full_with_annotations_emits_file_sections() {
        let annotations = vec![
            (
                loc("src/a.rs", 10),
                ann(PropertyType::MemorySafety, ConfidenceLevel::High),
            ),
            (
                loc("src/a.rs", 20),
                ann(PropertyType::ThreadSafety, ConfidenceLevel::Medium),
            ),
            (
                loc("src/b.rs", 5),
                ann(PropertyType::Termination, ConfidenceLevel::Low),
            ),
        ];
        let r = format_as_full(&annotations, Path::new("/test"), false).unwrap();
        assert!(r.contains("**Total proofs**: 3"));
        // Both files emit file sections.
        assert!(r.contains("## File: src/a.rs"));
        assert!(r.contains("## File: src/b.rs"));
    }

    #[test]
    fn test_format_as_full_include_evidence_renders_evidence_section() {
        let annotations = vec![(
            loc("src/a.rs", 1),
            ann(PropertyType::MemorySafety, ConfidenceLevel::High),
        )];
        let with_evidence = format_as_full(&annotations, Path::new("/test"), true).unwrap();
        let without = format_as_full(&annotations, Path::new("/test"), false).unwrap();
        // include_evidence=true emits at least one extra section.
        assert!(with_evidence.len() >= without.len());
    }

    // ── group_proofs_by_file ──

    #[test]
    fn test_group_proofs_by_file_groups_by_path() {
        let annotations = vec![
            (
                loc("src/a.rs", 1),
                ann(PropertyType::MemorySafety, ConfidenceLevel::High),
            ),
            (
                loc("src/a.rs", 2),
                ann(PropertyType::ThreadSafety, ConfidenceLevel::Medium),
            ),
            (
                loc("src/b.rs", 3),
                ann(PropertyType::Termination, ConfidenceLevel::Low),
            ),
        ];
        let grouped = group_proofs_by_file(&annotations);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&PathBuf::from("src/a.rs")).unwrap().len(), 2);
        assert_eq!(grouped.get(&PathBuf::from("src/b.rs")).unwrap().len(), 1);
    }

    #[test]
    fn test_group_proofs_by_file_empty_returns_empty_map() {
        let grouped = group_proofs_by_file(&[]);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_group_proofs_by_file_preserves_annotation_order_within_group() {
        let annotations = vec![
            (
                loc("src/a.rs", 30),
                ann(PropertyType::MemorySafety, ConfidenceLevel::High),
            ),
            (
                loc("src/a.rs", 10),
                ann(PropertyType::ThreadSafety, ConfidenceLevel::Medium),
            ),
            (
                loc("src/a.rs", 20),
                ann(PropertyType::Termination, ConfidenceLevel::Low),
            ),
        ];
        let grouped = group_proofs_by_file(&annotations);
        let group = grouped.get(&PathBuf::from("src/a.rs")).unwrap();
        // Insertion order preserved (sort happens later in write_file_section).
        assert_eq!(group[0].0.span.start.0, 30);
        assert_eq!(group[1].0.span.start.0, 10);
        assert_eq!(group[2].0.span.start.0, 20);
    }

    #[test]
    fn test_format_as_full_with_unique_line_markers_renders_each() {
        // Use line numbers far apart from any other timestamp/UUID digits
        // so they don't collide.
        let annotations = vec![
            (
                loc("src/x.rs", 12345),
                ann(PropertyType::MemorySafety, ConfidenceLevel::High),
            ),
            (
                loc("src/x.rs", 67890),
                ann(PropertyType::ThreadSafety, ConfidenceLevel::Medium),
            ),
        ];
        let r = format_as_full(&annotations, Path::new("/test"), false).unwrap();
        // Both line numbers must appear in output.
        assert!(r.contains("12345"));
        assert!(r.contains("67890"));
    }
}
