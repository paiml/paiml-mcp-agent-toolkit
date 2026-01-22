//! Tests for deep context
//! Extracted for file health compliance (CB-040)
//!
//! This module includes tests split across multiple files for maintainability.

use super::*;

#[path = "deep_context_tests_part1.rs"]
mod part1;

#[path = "deep_context_tests_part2.rs"]
mod part2;

#[path = "deep_context_tests_part3.rs"]
mod part3;

#[path = "deep_context_tests_part4.rs"]
mod part4;
