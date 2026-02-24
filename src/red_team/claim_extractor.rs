// Claim Extractor: Extract testable claims from commit messages
//
// Specification: Section 3.2 - Claim Categories
// Implements extraction of 8 categories of hallucination-prone claims

use regex::Regex;
use serde::{Deserialize, Serialize};

// Types: ClaimCategory, Claim, ClaimExtractor struct definition
include!("claim_extractor_types.rs");

// Constructor: ClaimExtractor::new()
include!("claim_extractor_new.rs");

// Extraction methods: extract(), extract_*, helpers, and Default impl
include!("claim_extractor_extract.rs");

// Tests: core unit tests
include!("claim_extractor_tests.rs");

// Tests: coverage-instrumented tests
include!("claim_extractor_tests_coverage.rs");
