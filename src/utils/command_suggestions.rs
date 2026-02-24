#![cfg_attr(coverage_nightly, coverage(off))]
//! Command Suggestion System
//!
//! Provides "did you mean?" functionality for CLI commands using Levenshtein distance
//! and semantic command mapping to improve discoverability.

use std::collections::HashMap;

// Levenshtein distance algorithm (free functions)
include!("command_suggestions_levenshtein.rs");

// CommandSuggester struct, impl, and Default
include!("command_suggestions_suggester.rs");

// Unit tests, extended tests, and property tests
include!("command_suggestions_tests.rs");

// Comprehensive coverage tests
include!("command_suggestions_comprehensive_tests.rs");
