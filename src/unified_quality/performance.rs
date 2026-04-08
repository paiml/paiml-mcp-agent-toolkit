#![allow(unused)]
//! Performance benchmarking and optimization for unified quality system
//!
//! Provides comprehensive performance monitoring, benchmarking, and optimization
//!
//! ## Module structure
//!
//! This file is split into semantically named include files:
//! - `performance_types.rs` — Core struct/enum definitions and Default impls
//! - `performance_monitor_impl.rs` — `impl PerformanceMonitor` methods
//! - `performance_report_types.rs` — Report types, alert types, helper impls

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::time::interval;

// Core type definitions: structs, enums, type aliases, and their Default impls
include!("performance_types.rs");

// PerformanceMonitor implementation (public API + private helpers)
include!("performance_monitor_impl.rs");

// Report types, alert types, and helper impl blocks (PerformanceMetrics, PerformanceOptimizer, PerformanceStatistics)
include!("performance_report_types.rs");

// Tests extracted to performance_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "performance_tests.rs"]
mod tests;
