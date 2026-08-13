#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Hallucination Detection Service - Sprint 37
//!
//! Semantic entropy-based hallucination detection for documentation validation.
//! Prevents AI-generated documentation from containing false claims about code capabilities.
//!
//! Based on peer-reviewed research:
//! - Semantic Entropy (Farquhar et al., Nature 2024)
//! - MIND framework (IJCAI 2025)
//! - Unified Detection Framework (Complex & Intelligent Systems 2025)

use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Data Types
// ============================================================================

/// Type of claim extracted from documentation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimType {
    /// Claims about code capabilities ("PMAT can analyze...")
    Capability,
    /// Claims about code structure ("File X contains...")
    Structure,
    /// Claims about APIs ("Function foo() accepts...")
    Api,
    /// Claims about commands ("Run pmat xyz...")
    Command,
    /// External reference (link, paper, etc.)
    ExternalRef,
}

/// Entity extracted from a claim
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entity {
    /// Programming language (e.g., "Rust", "TypeScript")
    Language(String),
    /// Function name
    Function(String),
    /// File path
    File(String),
    /// Module/namespace
    Module(String),
    /// Capability/feature name
    Capability(String),
}

/// Factual claim extracted from documentation
#[derive(Debug, Clone)]
pub struct Claim {
    /// Source file containing the claim
    pub source_file: PathBuf,
    /// Line number in source file
    pub line_number: usize,
    /// The claim text
    pub text: String,
    /// Claim type
    pub claim_type: ClaimType,
    /// Extracted entities (functions, files, modules)
    pub entities: Vec<Entity>,
    /// True if this is a negative claim ("PMAT cannot...")
    pub is_negative: bool,
}

/// Validation status for a claim
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Claim verified against codebase
    Verified,
    /// Claim could not be verified (potential hallucination)
    Unverified,
    /// Claim contradicts codebase (confirmed hallucination)
    Contradiction,
    /// Reference not found (404, missing file)
    NotFound,
    /// Claim is outdated
    Outdated,
    /// Insufficient evidence to validate
    Inconclusive,
}

/// Evidence supporting or contradicting a claim
#[derive(Debug, Clone)]
pub struct Evidence {
    /// Source of evidence (file, line, AST node)
    pub source: String,
    /// Semantic similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// Supporting text/code
    pub content: String,
}

/// Result of claim validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The claim being validated
    pub claim: Claim,
    /// Validation status
    pub status: ValidationStatus,
    /// Supporting or contradicting evidence
    pub evidence: Option<Evidence>,
    /// Error message if validation failed
    pub error_message: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

// ============================================================================
// Struct Definitions
// ============================================================================

/// Extracts factual claims from documentation text
pub struct ClaimExtractor {
    /// Regex patterns for capability claims
    capability_patterns: Vec<Regex>,
    /// Known programming languages
    known_languages: Vec<String>,
    /// Repository-relative file paths cited by the documentation
    file_pattern: Regex,
    /// Function/method names cited by the documentation (`foo()`)
    function_pattern: Regex,
}

/// Database of code facts extracted from deep context analysis
pub struct CodeFactDatabase {
    /// Functions indexed by name
    functions: HashMap<String, Vec<String>>,
    /// Supported languages
    languages: Vec<String>,
    /// Capabilities (features that exist in codebase)
    capabilities: Vec<String>,
}

/// Calculates semantic similarity between claims and facts
pub struct SemanticSimilarity {
    /// Common stopwords to filter out
    stopwords: Vec<String>,
}

/// Detects hallucinated claims in documentation
pub struct HallucinationDetector {
    /// Code facts from codebase
    code_facts: CodeFactDatabase,
    /// Similarity calculator
    // Reserved for future semantic similarity Phase 2 integration
    similarity: SemanticSimilarity,
}

/// End-to-end documentation accuracy validator
pub struct DocAccuracyValidator {
    /// Claim extractor
    extractor: ClaimExtractor,
    /// Hallucination detector
    detector: HallucinationDetector,
}

// ClaimExtractor: claim extraction and entity recognition
include!("hallucination_detector_extraction.rs");

// CodeFactDatabase: ground truth storage and lookup
include!("hallucination_detector_facts.rs");

// SemanticSimilarity: keyword-based similarity scoring
include!("hallucination_detector_similarity.rs");

// HallucinationDetector + DocAccuracyValidator: validation logic and tests
include!("hallucination_detector_validation.rs");
