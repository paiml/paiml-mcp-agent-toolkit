#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for proof_annotations_handler.rs
//!
//! Tests the proof annotations handler including:
//! - ProofAnnotationFilter construction and filtering logic
//! - filter_annotation() with various confidence levels
//! - filter_by_property_type() for all PropertyTypeFilter variants
//! - filter_by_verification_method() for all VerificationMethodFilter variants
//! - format_as_json() output structure and content
//! - format_as_summary() output format
//! - format_as_full() with and without evidence
//! - format_as_markdown() with and without evidence
//! - format_as_sarif() SARIF 2.1.0 compliance
//! - format_as_table() table formatting
//! - setup_proof_annotator() mock source setup
//! - collect_and_filter_annotations() async collection
//! - handle_analyze_proof_annotations() full handler workflow
//! - Output format enum Display implementations
//! - Property type filter Display implementations
//! - Verification method filter Display implementations

use crate::cli::proof_annotation_helpers::{
    filter_annotation, format_as_full, format_as_json, format_as_markdown, format_as_sarif,
    format_as_summary, format_as_table, setup_proof_annotator, ProofAnnotationFilter,
};
use crate::cli::{ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter};
use crate::models::unified_ast::{
    BytePos, ConfidenceLevel, EvidenceType, Location, ProofAnnotation, PropertyType, Span,
    VerificationMethod,
};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

// Test helpers, ProofAnnotationFilter construction, and filter_annotation tests
include!("proof_annotations_helpers_and_filter.rs");

// PropertyTypeFilter and VerificationMethodFilter matching tests
include!("proof_annotations_type_filters.rs");

// format_as_json, format_as_summary, format_as_table, format_as_full,
// format_as_markdown, format_as_sarif, and setup_proof_annotator tests
include!("proof_annotations_format_functions.rs");

// ProofAnnotationOutputFormat, PropertyTypeFilter, VerificationMethodFilter
// Display implementation tests, plus ConfidenceLevel tests
include!("proof_annotations_display_tests.rs");

// EvidenceType, Location, Span, PropertyType, VerificationMethod,
// ProofAnnotation construction, and edge case tests
include!("proof_annotations_type_and_edge_cases.rs");
