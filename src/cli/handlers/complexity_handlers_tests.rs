//! Comprehensive coverage tests for complexity_handlers.rs
//!
//! This module provides extensive test coverage for the complexity handler functions,
//! focusing on pure helper functions, data structures, and error paths.

use super::*;
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

include!("complexity_tests_coverage.rs");
include!("complexity_tests_property.rs");
include!("complexity_tests_dead_code.rs");
include!("complexity_tests_satd.rs");
include!("complexity_tests_watch.rs");
include!("complexity_tests_edge.rs");
