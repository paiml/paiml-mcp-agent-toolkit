//! Incremental Coverage Analysis Facade
//!
//! Provides a simplified interface for incremental coverage analysis operations.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

include!("incremental_coverage_types.rs");
include!("incremental_coverage_impl.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("incremental_coverage_tests.rs");
    include!("incremental_coverage_tests_async.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;

    include!("incremental_coverage_proptests.rs");
}
