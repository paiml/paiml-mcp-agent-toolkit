#![cfg_attr(coverage_nightly, coverage(off))]
//! Spec Falsification Engine — RAG-powered Popperian falsification for specifications
//!
//! Extracts falsifiable claims from markdown specifications and validates them
//! against the codebase using code search, filesystem checks, and metric measurement.
//!
//! ## Pipeline
//! 1. Parse markdown → extract atomic claims (RFC-2119 keywords, path refs, metrics)
//! 2. Categorize claims → dispatch to falsification strategies
//! 3. Run strategies → collect evidence (supporting or contradicting)
//! 4. Score verdicts → produce falsification report

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// --- Types: enums, structs ---
include!("spec_falsification_types.rs");

// --- Extractor: SpecClaimExtractor ---
include!("spec_falsification_extractor.rs");

// --- Engine: FalsificationEngine ---
include!("spec_falsification_engine.rs");

// --- Display: report formatting ---
include!("spec_falsification_display.rs");

// --- Tests ---
include!("spec_falsification_tests.rs");
