//! Performance testing module for SPECIFICATION.md Section 30
//!
//! This module provides the public API for performance testing functionality
//! that can be used by the CLI and other components.

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// Performance characteristics from SPECIFICATION.md Section 1.4
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    /// Startup latency targets
    pub startup_cold_ms: u64, // 127ms max
    pub startup_hot_ms: u64, // 4ms max

    /// Analysis throughput targets
    pub loc_per_sec_st: u64, // 487,000 LOC/s single-threaded
    pub loc_per_sec_mt: u64, // 3,921,000 LOC/s multi-threaded

    /// Memory usage targets
    pub base_rss_mb: u64, // 47MB base
    pub per_kloc_kb: u64, // 312KB per KLOC
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            startup_cold_ms: 127,
            startup_hot_ms: 4,
            loc_per_sec_st: 487_000,
            loc_per_sec_mt: 3_921_000,
            base_rss_mb: 47,
            per_kloc_kb: 312,
        }
    }
}

/// Performance test configuration
pub struct PerformanceTestConfig {
    pub enable_regression_tests: bool,
    pub enable_memory_tests: bool,
    pub enable_throughput_tests: bool,
    pub test_iterations: usize,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 3,
        }
    }
}

// Code generation helpers
include!("test_performance_codegen.rs");

// Performance test suite (async test functions, memory helpers, suite runner)
include!("test_performance_suite.rs");

// Unit tests
include!("test_performance_unit_tests.rs");
