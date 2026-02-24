#![cfg_attr(coverage_nightly, coverage(off))]
//! YAML-first configuration for pmat comply checks.
//!
//! Implements COMPLY-044 from improve-pmat-comply.md v2.8:
//! "Every quality check should be configurable via .pmat.yaml without code changes."
//!
//! # Configuration File
//!
//! Create a `.pmat.yaml` file in your project root:
//!
//! ```yaml
//! comply:
//!   checks:
//!     cb-050: { enabled: true, severity: critical }
//!     cb-060: { enabled: true, severity: high }
//!     cb-128: { enabled: true, threshold: 5.0 }
//!   thresholds:
//!     coverage: 95.0
//!     complexity: 20
//!     dead_code_pct: 1.0
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use pmat::models::comply_config::ComplyConfig;
//! use std::path::Path;
//!
//! let config = ComplyConfig::load(Path::new(".")).unwrap_or_default();
//! if config.is_check_enabled("cb-050") {
//!     // Run the check
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Type definitions: structs, enums, and their doc comments
include!("comply_config_types.rs");

// Default value functions for serde and default_checks()
include!("comply_config_defaults.rs");

// Impl blocks: Default impls, PmatYamlConfig, ComplyConfig, ConfigError
include!("comply_config_impls.rs");

// Tests
include!("comply_config_tests.rs");
