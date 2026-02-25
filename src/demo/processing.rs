#![cfg_attr(coverage_nightly, coverage(off))]
//! Data processing functions for demo analysis results.

use anyhow::Result;

// Step processing and extraction/parsing functions
include!("processing_extraction.rs");

// Web demo analysis helper functions
include!("processing_analyzers.rs");

// Unit tests and property tests
include!("processing_tests.rs");
