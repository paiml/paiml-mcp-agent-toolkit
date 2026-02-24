#![cfg_attr(coverage_nightly, coverage(off))]
//! Configuration management service for PMAT system
//!
//! This module provides a centralized configuration management system that
//! consolidates all configuration patterns in the codebase following the
//! Toyota Way ONE implementation principle.
//!
//! ## Module Organization
//!
//! - `configuration_types.rs` — Config struct definitions (PmatConfig, SystemConfig, etc.)
//! - `configuration_defaults.rs` — Default configuration values
//! - `configuration_impl.rs` — ConfigurationService struct and core logic
//! - `configuration_tests.rs` — Unit tests, property tests, getter tests

use crate::services::service_base::ServiceMetrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Type definitions: PmatConfig, SystemConfig, QualityConfig, AnalysisConfig,
// PerformanceConfig, McpConfig, RoadmapConfig, GitConfig, TelemetryConfig,
// SemanticConfig, and serde default functions
include!("configuration_types.rs");

// Default configuration values (ConfigurationService::default_config)
include!("configuration_defaults.rs");

// ConfigurationService struct, ConfigWatcher trait, impl blocks,
// service lifecycle (start/stop/status/health_check), and global singleton
include!("configuration_impl.rs");

// Unit tests, property tests, and getter tests
include!("configuration_tests.rs");
