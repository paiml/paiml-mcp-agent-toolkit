#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

// Use concat! to avoid self-detection by CB-501 scanner
const DOT_UNWRAP_STR: &str = concat!(".unwr", "ap()");
const UNWRAP_OR_STR: &str = concat!("unwra", "p_or");

// SIMD intrinsic patterns for CB-021 detection
const SIMD_INTRINSIC_PATTERNS: &[(&str, &str)] = &[
    (concat!("_mm", "256_"), "SIMD intrinsic"),
    (concat!("_mm", "512_"), "SIMD intrinsic"),
];
const PORTABLE_SIMD_PATTERNS: &[(&str, &str)] = &[
    (concat!("i8x", "16::"), "Portable SIMD"),
    (concat!("i16x", "8::"), "Portable SIMD"),
    (concat!("i32x", "4::"), "Portable SIMD"),
    (concat!("f32x", "4::"), "Portable SIMD"),
    (concat!("Simd", "::<"), "Portable SIMD"),
];

// CB-021: SIMD intrinsics without #[target_feature]
include!("safety_checks_simd.rs");

// CB-001, CB-002: WGSL bounds checking and barrier divergence
include!("safety_checks_wgsl.rs");

// CB-BUDGET: ComputeBrick assertions and profiler anomaly detection
include!("safety_checks_profiler.rs");

// OIP Tarantula common scanner infrastructure and CB-120 NaN-unsafe detection
include!("safety_checks_oip_scanner.rs");

// CB-121 through CB-124: OIP Tarantula pattern detectors
include!("safety_checks_oip_detectors.rs");
