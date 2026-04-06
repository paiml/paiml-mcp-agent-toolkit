#![cfg_attr(coverage_nightly, coverage(off))]
//! Analytics Backend Abstraction (Issue #79, P0-3)
//!
//! Provides unified backend selection for GPU/SIMD/Scalar compute operations.
//! Implements graceful degradation: GPU → SIMD → Scalar.
//!
//! # Backend Selection
//!
//! ```rust
//! use pmat::services::analytics_backend::{Backend, BackendSelector};
//!
//! // Automatic backend selection (graceful degradation)
//! let backend = BackendSelector::auto_select();
//!
//! // Manual backend selection
//! let backend = Backend::Simd;
//! ```
//!
//! # Statistical Equivalence Testing
//!
//! The primary use case is validating GPU floating-point results against SIMD
//! to ensure CI stability despite GPU non-associativity.
//!
//! Reference: Higham (1993) - "The Accuracy of Floating Point Summation" (SIAM)

/// Compute backend selection for analytics operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// GPU-accelerated compute using wgpu (requires analytics-gpu feature)
    #[cfg(feature = "analytics-gpu")]
    Gpu,

    /// SIMD-accelerated compute using trueno (requires analytics-simd feature)
    #[cfg(feature = "analytics-simd")]
    Simd,

    /// Scalar fallback (always available)
    Scalar,
}

/// Backend selector with graceful degradation
pub struct BackendSelector;

impl BackendSelector {
    /// Automatically select best available backend
    ///
    /// Preference order: GPU > SIMD > Scalar
    ///
    /// # Example
    ///
    /// ```rust
    /// use pmat::services::analytics_backend::BackendSelector;
    ///
    /// let backend = BackendSelector::auto_select();
    /// println!("Selected backend: {:?}", backend);
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn auto_select() -> Backend {
        #[cfg(feature = "analytics-gpu")]
        {
            // GPU availability check not yet implemented; fall through to SIMD
        }

        #[cfg(feature = "analytics-simd")]
        {
            return Backend::Simd;
        }

        #[allow(unreachable_code)]
        Backend::Scalar
    }

    /// Check if GPU backend is available
    #[cfg(feature = "analytics-gpu")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_gpu_available() -> bool {
        // TODO: Implement GPU device detection
        false
    }

    /// Check if SIMD backend is available
    #[cfg(feature = "analytics-simd")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_simd_available() -> bool {
        true // Always available when feature is enabled
    }
}

/// Statistical helper functions for equivalence testing
pub mod stats {
    use super::Backend;
    use anyhow::Result;

    include!("analytics_backend_stats.rs");
}

/// GPU compute backend (Issue #79, P0-3 and P0-5)
///
/// Implements GPU-accelerated compute operations using wgpu.
/// Provides PCIe bandwidth calibration for cost-based query optimization.
#[cfg(feature = "analytics-gpu")]
pub mod gpu {
    use anyhow::{bail, Context, Result};
    use std::sync::Once;
    use wgpu::util::DeviceExt;

    include!("analytics_backend_gpu.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::stats::*;
    use super::*;

    include!("analytics_backend_tests.rs");
}

// Design-by-contract specifications (Verus-style)
// #[requires(project_path.is_dir())]
// #[ensures(result.is_ok() ==> ret.len() > 0)]
