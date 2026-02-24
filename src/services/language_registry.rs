#![cfg_attr(coverage_nightly, coverage(off))]
//! Language Registry for 30+ language support per SPECIFICATION.md Section 6.2
//!
//! This module provides comprehensive language detection and parser selection
//! for modern software development ecosystems.
//!
//! ## Module Structure
//!
//! Split into submodules for maintainability:
//! - `language_registry_data.rs` — Static language metadata table (LanguageInfo, LANGUAGE_INFO)
//! - `language_registry_enum.rs` — Language enum definition (56 variants)
//! - `language_registry_impl.rs` — Language methods (detection, AST support, complexity)
//! - `language_registry_stats.rs` — LanguageStats struct and LanguageRegistry
//! - `language_registry_tests.rs` — Unit and property tests

use serde::{Deserialize, Serialize};
use std::path::Path;

// Static language metadata table
include!("language_registry_data.rs");

// Language enum definition
include!("language_registry_enum.rs");

// Language impl block (detection, AST support, complexity)
include!("language_registry_impl.rs");

// LanguageStats and LanguageRegistry
include!("language_registry_stats.rs");

// Tests
include!("language_registry_tests.rs");
