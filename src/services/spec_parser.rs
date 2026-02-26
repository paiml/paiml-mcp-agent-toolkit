//! Specification Parser Service (Part C: Specification Parsing Enhancement)
//!
//! Parses markdown specification files from docs/specifications/*.md
//! and extracts validation criteria for the pmat qa command.
//!
//! # Architecture (Toyota Way - Genchi Genbutsu)
//!
//! Go to the source: extract validation criteria directly from specification files
//! rather than duplicating them in separate configuration.
//!
//! # References
//!
//! - Specification: docs/specifications/enhance-pmat-work.md
//! - Related Issues: #102, #113, #114, #116

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// --- Type definitions (ParsedSpec, ValidationClaim, ClaimCategory, CodeExample, etc.) ---
include!("spec_parser_types.rs");

// --- SpecParser struct and all impl blocks ---
include!("spec_parser_impl.rs");

// --- Validation types (ClaimValidation, ValidationStatus, ValidationSummary) ---
include!("spec_parser_validation.rs");

// --- Tests ---
include!("spec_parser_tests.rs");
