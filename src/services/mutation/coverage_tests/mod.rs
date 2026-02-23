#![cfg_attr(coverage_nightly, coverage(off))]
//! EXTREME TDD coverage tests for mutation testing module
//!
//! These tests exercise the core mutation testing functionality through
//! the public API to ensure comprehensive coverage.

use super::*;

mod types_tests;
mod operators_tests;
mod scoring_tests;
mod engine_tests;
mod language_tests;
mod state_tests;
mod rust_adapter_tests;
mod property_tests;
